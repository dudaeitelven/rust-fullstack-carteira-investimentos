use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true);

    Ok((jar.add(cookie), Redirect::to("/")))
}

async fn index(maybe_user: Option<User>) -> Result<Response, AppError> {
    match maybe_user {
        Some(user) => Ok(Html(format!("Hello, {}", user.username())).into_response()),
        None => Ok(Redirect::to("/login").into_response()),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn test_login_registers_new_user(db: PgPool) {
        let request = LoginForm {
            username: "alice".to_string(),
            password: "s3cret".to_string(),
        };

        let response = login(db.into(), CookieJar::new(), Form(request))
            .await
            .expect("success")
            .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    #[sqlx::test]
    async fn test_login_wrong_password_fails(db: PgPool) {
        let repository: Repository = db.into();
        UnauthenticatedUser::new("alice".to_string(), "s3cret".to_string())
            .register(&repository)
            .await
            .expect("registration succeeds");

        let request = LoginForm {
            username: "alice".to_string(),
            password: "wrong".to_string(),
        };

        let result = login(repository, CookieJar::new(), Form(request)).await;

        assert!(matches!(result, Err(AppError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_index_redirects_when_logged_out() {
        let response = index(None).await.expect("success");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    #[sqlx::test]
    async fn test_index_greets_logged_in_user(db: PgPool) {
        let repository: Repository = db.into();
        let user = UnauthenticatedUser::new("alice".to_string(), "s3cret".to_string())
            .register(&repository)
            .await
            .expect("registration succeeds");

        let response = index(Some(user)).await.expect("success");

        assert_eq!(response.status(), StatusCode::OK);
    }
}

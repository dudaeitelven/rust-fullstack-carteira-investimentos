use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing Authorization Headers")]
    MissingAuthorization,
    #[error("Invalid Credentials")]
    InvalidCredentials,
    #[error("Asset does not exist")]
    AssetDoesNotExist,
    #[error("User does not exist")]
    UserDoesNotExist,
    #[error("This username is already registered")]
    UsernameTaken,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[error(transparent)]
    Jwt(#[from] jwt_simple::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let error_response = ErrorResponse {
            error: self.to_string(),
        };

        let status = match self {
            Self::UsernameTaken | Self::MissingAuthorization => StatusCode::BAD_REQUEST,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AssetDoesNotExist | Self::UserDoesNotExist => StatusCode::NOT_FOUND,
            Self::Database(_) | Self::Template(_) | Self::Jwt(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, Json(error_response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_request_status_codes() {
        assert_eq!(
            AppError::UsernameTaken.into_response().status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::MissingAuthorization.into_response().status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_unauthorized_status_code() {
        assert_eq!(
            AppError::InvalidCredentials.into_response().status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_not_found_status_codes() {
        assert_eq!(
            AppError::AssetDoesNotExist.into_response().status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::UserDoesNotExist.into_response().status(),
            StatusCode::NOT_FOUND
        );
    }
}

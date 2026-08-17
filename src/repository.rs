use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{Asset, UserRecord},
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT id, name, unit_value
             FROM assets;"
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value)
             VALUES ($1, $2)
             RETURNING id, name, unit_value;",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets
             SET name=COALESCE($2, name),
                 unit_value=COALESCE($3, unit_value)
             WHERE id=$1
             RETURNING id, name, unit_value;",
            asset_id,
            name,
            unit_value
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash)
             VALUES ($1, $2)
             RETURNING id, username, password_hash;",
            username,
            password_hash,
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash
             FROM users
             WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn test_create_and_list_assets(db: PgPool) {
        let repository: Repository = db.into();

        let created = repository
            .create_asset("Bitcoin".to_string(), 10.0)
            .await
            .expect("success");

        let assets = repository.list_assets().await.expect("success");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, created.id);
        assert_eq!(assets[0].name, "Bitcoin");
    }

    #[sqlx::test]
    async fn test_update_asset_only_name(db: PgPool) {
        let repository: Repository = db.into();
        let created = repository
            .create_asset("Bitcoin".to_string(), 10.0)
            .await
            .expect("success");

        let updated = repository
            .update_asset(created.id, Some("Ethereum".to_string()), None)
            .await
            .expect("success")
            .expect("asset exists");

        assert_eq!(updated.name, "Ethereum");
        assert_eq!(updated.unit_value, 10.0);
    }

    #[sqlx::test]
    async fn test_update_asset_only_unit_value(db: PgPool) {
        let repository: Repository = db.into();
        let created = repository
            .create_asset("Bitcoin".to_string(), 10.0)
            .await
            .expect("success");

        let updated = repository
            .update_asset(created.id, None, Some(99.5))
            .await
            .expect("success")
            .expect("asset exists");

        assert_eq!(updated.name, "Bitcoin");
        assert_eq!(updated.unit_value, 99.5);
    }

    #[sqlx::test]
    async fn test_update_nonexistent_asset_returns_none(db: PgPool) {
        let repository: Repository = db.into();

        let updated = repository
            .update_asset(999, Some("Ghost".to_string()), None)
            .await
            .expect("success");

        assert!(updated.is_none());
    }

    #[sqlx::test]
    async fn test_add_and_get_user_by_name(db: PgPool) {
        let repository: Repository = db.into();

        let created = repository
            .add_user("alice", "hashed-password")
            .await
            .expect("success");

        let fetched = repository
            .get_user_by_name("alice")
            .await
            .expect("success")
            .expect("user exists");

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.username, "alice");
        assert_eq!(fetched.password_hash, "hashed-password");
    }

    #[sqlx::test]
    async fn test_get_user_by_name_missing_returns_none(db: PgPool) {
        let repository: Repository = db.into();

        let fetched = repository
            .get_user_by_name("nobody")
            .await
            .expect("success");

        assert!(fetched.is_none());
    }

    #[sqlx::test]
    async fn test_add_duplicate_user_is_unique_violation(db: PgPool) {
        let repository: Repository = db.into();

        repository
            .add_user("alice", "hash1")
            .await
            .expect("first insert succeeds");

        let err = repository
            .add_user("alice", "hash2")
            .await
            .expect_err("duplicate username should fail");

        match err {
            sqlx::Error::Database(db_err) => assert!(db_err.is_unique_violation()),
            other => panic!("expected database error, got {other:?}"),
        }
    }
}

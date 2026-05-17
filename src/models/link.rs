use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Link {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub short_code: String,
    pub original_url: String,
    pub title: Option<String>,
    pub is_active: Option<bool>,
    pub click_count: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    pub custom_code: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub id: Uuid,
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub title: Option<String>,
    pub click_count: i64,
    pub created_at: Option<DateTime<Utc>>,
}

impl Link {
    pub async fn create(
        pool: &PgPool,
        user_id: Option<Uuid>,
        short_code: &str,
        original_url: &str,
        title: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Link>(
            "INSERT INTO links (user_id, short_code, original_url, title) VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(user_id)
        .bind(short_code)
        .bind(original_url)
        .bind(title)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Link>(
            "SELECT * FROM links WHERE short_code = $1 AND is_active = TRUE"
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Link>(
            "SELECT * FROM links WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM links WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn increment_clicks(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE links SET click_count = click_count + 1 WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn code_exists(pool: &PgPool, code: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM links WHERE short_code = $1)"
        )
        .bind(code)
        .fetch_one(pool)
        .await?;
        Ok(result)
    }
}

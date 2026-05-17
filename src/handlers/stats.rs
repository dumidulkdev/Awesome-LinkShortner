use axum::extract::{Path, State};
use axum::response::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::click::{Click, ClickStats};
use crate::models::link::Link;

pub async fn get_stats(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ClickStats>, AppError> {
    let link = Link::find_by_user(&pool, auth.user_id)
        .await?
        .into_iter()
        .find(|l| l.id == id)
        .ok_or_else(|| AppError::NotFound("Link not found".to_string()))?;

    let stats = Click::get_stats(&pool, link.id).await?;
    Ok(Json(stats))
}

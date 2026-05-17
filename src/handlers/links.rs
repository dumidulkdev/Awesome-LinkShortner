use axum::extract::{Path, State};
use axum::response::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::link::{CreateLinkRequest, Link, LinkResponse};
use crate::utils::shortcode;

fn to_response(link: &Link, host: &str) -> LinkResponse {
    LinkResponse {
        id: link.id,
        short_code: link.short_code.clone(),
        short_url: format!("{}/{}", host, link.short_code),
        original_url: link.original_url.clone(),
        title: link.title.clone(),
        click_count: link.click_count.unwrap_or(0),
        created_at: link.created_at,
    }
}

pub async fn create_link(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Json(body): Json<CreateLinkRequest>,
) -> Result<Json<LinkResponse>, AppError> {
    if body.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".to_string()));
    }

    if !body.url.starts_with("http://") && !body.url.starts_with("https://") {
        return Err(AppError::BadRequest("URL must start with http:// or https://".to_string()));
    }

    let short_code = if let Some(custom) = &body.custom_code {
        if custom.len() < 3 || custom.len() > 10 {
            return Err(AppError::BadRequest("Custom code must be 3-10 characters".to_string()));
        }
        if Link::code_exists(&pool, custom).await? {
            return Err(AppError::Conflict("Custom code already taken".to_string()));
        }
        custom.clone()
    } else {
        let mut code = shortcode::generate();
        while Link::code_exists(&pool, &code).await? {
            code = shortcode::generate();
        }
        code
    };

    let link = Link::create(
        &pool,
        Some(auth.user_id),
        &short_code,
        &body.url,
        body.title.as_deref(),
    )
    .await?;

    Ok(Json(to_response(&link, "http://localhost:3000")))
}

pub async fn list_links(
    State(pool): State<PgPool>,
    auth: AuthUser,
) -> Result<Json<Vec<LinkResponse>>, AppError> {
    let links = Link::find_by_user(&pool, auth.user_id).await?;
    let responses: Vec<LinkResponse> = links
        .iter()
        .map(|l| to_response(l, "http://localhost:3000"))
        .collect();
    Ok(Json(responses))
}

pub async fn delete_link(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = Link::delete(&pool, id, auth.user_id).await?;
    if !deleted {
        return Err(AppError::NotFound("Link not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "message": "Link deleted" })))
}

use axum::extract::State;
use axum::response::Json;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::middleware::auth::create_token;
use crate::models::user::{AuthResponse, CreateUserRequest, LoginRequest, User};
use crate::utils::hash::{hash_password, verify_password};
use crate::config::Config;

pub async fn register(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if body.email.is_empty() || body.password.is_empty() {
        return Err(AppError::BadRequest("Email and password are required".to_string()));
    }

    if body.password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    let existing = User::find_by_email(&pool, &body.email).await?;
    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let password_hash = hash_password(&body.password)?;
    let user = User::create(&pool, &body.email, &password_hash).await?;

    let token = create_token(user.id, &user.email, &config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        email: user.email,
    }))
}

pub async fn login(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = User::find_by_email(&pool, &body.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let valid = verify_password(&body.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    let token = create_token(user.id, &user.email, &config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        email: user.email,
    }))
}

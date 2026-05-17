use axum::routing::{delete, get, post};
use axum::Router;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod utils;

use config::Config;
use middleware::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("linkshort=info".parse().unwrap()))
        .init();

    let config = Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    let statements = [
        r#"CREATE EXTENSION IF NOT EXISTS "pgcrypto""#,
        r#"CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            plan VARCHAR(20) DEFAULT 'free',
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
        r#"CREATE TABLE IF NOT EXISTS links (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID REFERENCES users(id) ON DELETE CASCADE,
            short_code VARCHAR(10) UNIQUE NOT NULL,
            original_url TEXT NOT NULL,
            title VARCHAR(255),
            is_active BOOLEAN DEFAULT TRUE,
            click_count BIGINT DEFAULT 0,
            expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
        r#"CREATE TABLE IF NOT EXISTS clicks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            link_id UUID REFERENCES links(id) ON DELETE CASCADE,
            ip_address VARCHAR(45),
            user_agent TEXT,
            referer TEXT,
            clicked_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
        "CREATE INDEX IF NOT EXISTS idx_links_short_code ON links(short_code)",
        "CREATE INDEX IF NOT EXISTS idx_links_user_id ON links(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_clicks_link_id ON clicks(link_id)",
        "CREATE INDEX IF NOT EXISTS idx_clicks_clicked_at ON clicks(clicked_at)",
    ];

    for stmt in &statements {
        sqlx::query(stmt)
            .execute(&pool)
            .await
            .expect("Failed to run migration statement");
    }

    tracing::info!("Migrations applied");

    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    let _rate_limiter = RateLimiter::new(100, 60);

    let api_routes = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/links", post(handlers::links::create_link))
        .route("/links", get(handlers::links::list_links))
        .route("/links/{id}", delete(handlers::links::delete_link))
        .route("/links/{id}/stats", get(handlers::stats::get_stats));

    let app = Router::new()
        .route("/", get(handlers::pages::index))
        .route("/login", get(handlers::pages::login_page))
        .route("/register", get(handlers::pages::register_page))
        .route("/dashboard", get(handlers::pages::dashboard_page))
        .route("/stats/{id}", get(handlers::pages::stats_page))
        .nest("/api", api_routes)
        .route("/{code}", get(handlers::redirect::redirect))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

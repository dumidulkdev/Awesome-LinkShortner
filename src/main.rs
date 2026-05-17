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

    sqlx::raw_sql(include_str!("../migrations/001_initial.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations applied");

    let state = AppState {
        db: pool,
        config: config.clone(),
    };

    let api_routes = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/links", post(handlers::links::create_link))
        .route("/links", get(handlers::links::list_links))
        .route("/links/{id}", delete(handlers::links::delete_link))
        .route("/links/{id}/stats", get(handlers::stats::get_stats));

    let app = Router::new()
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

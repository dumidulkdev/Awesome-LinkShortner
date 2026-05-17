use axum::extract::{ConnectInfo, Path, State};
use axum::http::HeaderMap;
use axum::response::Redirect;
use std::net::SocketAddr;

use crate::errors::AppError;
use crate::models::click::Click;
use crate::models::link::Link;
use crate::AppState;

pub async fn redirect(
    State(state): State<AppState>,
    Path(code): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Redirect, AppError> {
    let link = Link::find_by_code(&state.db, &code)
        .await?
        .ok_or_else(|| AppError::NotFound("Link not found".to_string()))?;

    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let referer = headers
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let db = state.db.clone();
    let link_id = link.id;
    tokio::spawn(async move {
        let _ = Click::create(
            &db,
            link_id,
            Some(&ip),
            user_agent.as_deref(),
            referer.as_deref(),
        )
        .await;
        let _ = Link::increment_clicks(&db, link_id).await;
    });

    Ok(Redirect::temporary(&link.original_url))
}

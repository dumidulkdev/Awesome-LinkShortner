use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    async fn check(&self, key: &str) -> bool {
        let mut map = self.requests.lock().await;
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let timestamps = map.entry(key.to_string()).or_default();
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= self.max_requests {
            return false;
        }

        timestamps.push(now);
        true
    }
}

pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let limiter = req.extensions().get::<RateLimiter>().cloned();

    if let Some(limiter) = limiter {
        let ip = addr.ip().to_string();
        if !limiter.check(&ip).await {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(json!({ "error": "Too many requests. Please slow down." })),
            )
                .into_response();
        }
    }

    next.run(req).await
}

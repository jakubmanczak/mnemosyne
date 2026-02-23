use axum::{Router, routing::get};

pub fn api_router() -> Router {
    Router::new().route("/api/live", get(async || "Mnemosyne lives"))
}

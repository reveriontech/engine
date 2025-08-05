// use axum::{Router, routing::{get, post}, middleware};
use axum::{Router, routing::{get, post}};
use crate::api::auth_handler::handler as google;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(|| async { "Server is running." }))
        .route("/google", post(google))
}
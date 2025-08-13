// use axum::{Router, routing::{get, post}, middleware};
use axum::{Router, routing::{get, post}};
use crate::api::auth_handler::handler as google;
use crate::api::article_handler::handler as article;
use crate::api::glossary_handler::handler as glossary;
use crate::api::image_handler::handler as image;
use crate::api::audio_handler::handler as audio;
use crate::api::glossary_selector::selector as glosselector;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(|| async { "Server is running." }))
        .route("/google", post(google))
        .route("/article", post(article))
        .route("/glossary", post(glossary))
        .route("/image", post(image))
        .route("/audio", post(audio))
        .route("/glosselector", get(glosselector))
}
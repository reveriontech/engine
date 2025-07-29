use axum::{Router, routing::{get, post}, middleware};
// use crate::api::google::handler as google;
// use crate::api::session::handler as session;
// use crate::api::create_docs::handler as create_docs; 
// use crate::api::document_data::handler as document_data;
// use crate::api::auth_token::middleware as auth_token;

pub fn routes() -> Router {
    
    Router::new()
        .route("/", get(|| async { "Server is running." }))
        // .route("/google", post(google))
        // .route("/session", post(session))
        // .route(
        //     "/createDocs",
        //     post(create_docs).layer(middleware::from_fn(auth_token)),
        // )
        // .route("/documentData", post(document_data))
        
}
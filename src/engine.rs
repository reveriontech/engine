// pub mod api;
// mod routes;
// pub mod db;

// use axum::{body::Body, extract::State, http::{Method, Request, StatusCode}, middleware::{self, Next}, response::Response, Extension};
// use dotenvy::dotenv;
// use std::{env, sync::Arc};
// use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer, trace::TraceLayer,};
// use db::init_db_pool;
// use routes::auth_routes;

// async fn enforce_origin(
//     State(allowed_origin): State<Arc<String>>,
//     req: Request<Body>,
//     next: Next,
// ) -> Result<Response, StatusCode> {
//     let origin = req.headers().get("origin").and_then(|v| v.to_str().ok());

//     if let Some(origin) = origin {
//         if origin == allowed_origin.as_str() {
//             return Ok(next.run(req).await);
//         }
//     }

//     Err(StatusCode::FORBIDDEN)
// }

// #[tokio::main]
// async fn main() {
//     tracing_subscriber::fmt::init();
//     dotenv().ok();

//     let client_origin = env::var("CLIENT_URL")
//         .unwrap_or_else(|_| "http://localhost:4000".to_string());
//     let allowed_origin = Arc::new(client_origin.clone());

//     let port = env::var("PORT").unwrap_or_else(|_| "5000".to_string());

//     let cors = CorsLayer::new()
//         .allow_origin(client_origin.parse::<axum::http::HeaderValue>().unwrap())
//         .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
//         .allow_headers([
//             axum::http::header::CONTENT_TYPE,
//             axum::http::header::AUTHORIZATION,
//             axum::http::header::ACCEPT,
//             axum::http::header::ACCEPT_LANGUAGE,
//             axum::http::header::ACCEPT_ENCODING,
//         ])
//         .allow_credentials(true);

//     let db_pool = init_db_pool().await;

//     let app = auth_routes::routes()
//         .layer(Extension(db_pool))
//         .route_layer(middleware::from_fn_with_state(
//             allowed_origin.clone(),
//             enforce_origin,
//         ))
//         .layer(cors)
//         .layer(SetResponseHeaderLayer::if_not_present(
//             axum::http::header::STRICT_TRANSPORT_SECURITY,
//             axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
//         ))
//         .layer(TraceLayer::new_for_http());

//     let addr = format!("0.0.0.0:{}", port);
//     println!("Server running on http://{}", addr);

//     let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
//     axum::serve(listener, app).await.unwrap();
// }
pub mod api;
mod routes;
pub mod db;

// use axum::{body::Body, extract::State, http::{Method, Request, StatusCode}, middleware::{self, Next}, response::Response, Extension};
use axum::{http::Method, Extension};
use dotenvy::dotenv;
// use std::{env, sync::Arc};
use std::env;
use tower_http::{cors::{CorsLayer, Any}, set_header::SetResponseHeaderLayer, trace::TraceLayer,};
use db::init_db_pool;
use routes::auth_routes;

// async fn enforce_origin(
//     State(allowed_origin): State<Arc<String>>,
//     req: Request<Body>,
//     next: Next,
// ) -> Result<Response, StatusCode> {
//     let origin = req.headers().get("origin").and_then(|v| v.to_str().ok());

//     if let Some(origin) = origin {
//         if origin == allowed_origin.as_str() {
//             return Ok(next.run(req).await);
//         }
//     }

//     Err(StatusCode::FORBIDDEN)
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    // let client_origin = env::var("CLIENT_URL").unwrap_or_else(|_| "http://localhost:4000".to_string());
    // let allowed_origin = Arc::new(client_origin.clone());

    let port = env::var("PORT").unwrap_or_else(|_| "5000".to_string());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::ACCEPT_LANGUAGE,
            axum::http::header::ACCEPT_ENCODING,
        ])
        .allow_credentials(false);

    let db_pool = init_db_pool().await;

    let app = auth_routes::routes()
        .layer(Extension(db_pool))
        // .route_layer(middleware::from_fn_with_state(
        //     allowed_origin.clone(),
        //     enforce_origin,
        // ))
        .layer(cors)
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", port);
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// use axum::{Extension, Json};
// use axum::http::StatusCode;
// use serde::{Deserialize, Serialize};
// use sqlx::PgPool;
// use uuid::Uuid;

// #[derive(Debug, Deserialize)]
// pub struct ImageData {
//     pub url: String,
//     pub alt_text: String,
//     pub data: String,
//     pub is_indb: String,
// }

// #[derive(Debug, Serialize)]
// pub struct ImageResponse {
//     pub id: Option<String>,
//     pub url: Option<String>,
//     pub alt_text: Option<String>,
//     pub data: Option<String>,
//     pub is_indb: Option<String>,
// }

// pub async fn handler(
//     Extension(pool): Extension<PgPool>,
//     Json(payload): Json<ImageData>,
// ) -> Result<Json<ImageResponse>, (StatusCode, String)> {
//     let new_id = Uuid::new_v4().to_string();

//     let ImageData {
//         url,
//         alt_text,
//         data,
//         is_indb
//     } = payload;

//     sqlx::query!(
//         r#"
//         INSERT INTO public.images (id, url, alt_text, data, is_indb)
//         VALUES ($1, $2, $3, $4, $5)
//         "#,
//         new_id,
//         url,
//         alt_text,
//         data,
//         is_indb
//     )
//     .execute(&pool)
//     .await
//     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert failed: {}", e)))?;

//     let row = sqlx::query_as!(
//         ImageResponse,
//         r#"
//         SELECT 
//             id,
//             url,
//             alt_text,
//             data,
//             is_indb
//         FROM public.images
//         WHERE id = $1
//         "#,
//         new_id
//     )
//     .fetch_one(&pool)
//     .await
//     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch failed: {}", e)))?;

//     Ok(Json(row))
// }
use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use reqwest::Client;

#[derive(Debug, Deserialize)]
pub struct ImageData {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ImageResponse {
    pub id: Option<String>,
    pub url: Option<String>,
    pub is_indb: Option<i32>,
}

pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<ImageData>,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {

    let client = Client::new();
    let resp = client
        .get(&payload.url)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Download failed: {}", e)))?;

    if !resp.status().is_success() {
        return Err((StatusCode::BAD_REQUEST, format!("Failed to fetch image: HTTP {}", resp.status())));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read bytes failed: {}", e)))?;

    let updated = sqlx::query_as!(
        ImageResponse,
        r#"
        UPDATE public.images
        SET data = $1, is_indb = 1
        WHERE url = $2
        RETURNING id, url, is_indb
        "#,
        bytes.as_ref(),
        payload.url
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)))?;

    Ok(Json(updated))
}

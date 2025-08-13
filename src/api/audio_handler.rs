use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use reqwest::Client;

#[derive(Debug, Deserialize)]
pub struct AudioData {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct AudioResponse {
    pub id: Option<String>,
    pub url: Option<String>,
    pub is_indb: Option<i32>,
}

pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<AudioData>,
) -> Result<Json<AudioResponse>, (StatusCode, String)> {

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
        AudioResponse,
        r#"
        UPDATE public.audio
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

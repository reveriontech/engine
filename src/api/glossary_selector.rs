use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct GlossaryResponse {
    pub id: Option<String>,
    pub title: Option<String>,
    pub definition: Option<String>,
}

pub async fn selector(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<GlossaryResponse>>, (StatusCode, String)> {

    let rows = sqlx::query_as!(
        GlossaryResponse,
        r#"
        SELECT 
            id,
            title,
            definition
        FROM public.glossary
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch failed: {}", e)))?;

    Ok(Json(rows))
}

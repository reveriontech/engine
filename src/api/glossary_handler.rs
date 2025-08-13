use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct GlossaryData {
    pub title: String,
    pub definition: String,
}

#[derive(Debug, Serialize)]
pub struct GlossaryResponse {
    pub id: Option<String>,
    pub title: Option<String>,
    pub definition: Option<String>,
}

pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<GlossaryData>,
) -> Result<Json<GlossaryResponse>, (StatusCode, String)> {
    let new_id = Uuid::new_v4().to_string();

    let GlossaryData {
        title,
        definition,
    } = payload;

    sqlx::query!(
        r#"
        INSERT INTO public.glossary (id, title, definition)
        VALUES ($1, $2, $3)
        "#,
        new_id,
        title,
        definition
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert failed: {}", e)))?;

    let row = sqlx::query_as!(
        GlossaryResponse,
        r#"
        SELECT 
            id,
            title,
            definition
        FROM public.glossary
        WHERE id = $1
        "#,
        new_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch failed: {}", e)))?;

    Ok(Json(row))
}

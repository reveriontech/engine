use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ArticleData {
    pub title: String,
    pub content: String,
    pub authorid: String,
    pub ispublished: String,
    pub language: String,
    pub readtime: String,
    pub subheading: String,
    pub isarchived: String,
    pub realtitle: String,
    pub ismainpage: String,
    pub isurltitledifferent: String,
    pub mobiletitle: String,
}

#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub authorid: Option<String>,
    pub ispublished: Option<String>,
    pub language: Option<String>,
    pub readtime: Option<String>,
    pub subheading: Option<String>,
    pub isarchived: Option<String>,
    pub realtitle: Option<String>,
    pub ismainpage: Option<String>,
    pub isurltitledifferent: Option<String>,
    pub mobiletitle: Option<String>,
}

pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<ArticleData>,
) -> Result<Json<ArticleResponse>, (StatusCode, String)> {
    let new_id = Uuid::new_v4().to_string();

    let ArticleData {
        title,
        content,
        authorid,
        ispublished,
        language,
        readtime,
        subheading,
        isarchived,
        realtitle,
        ismainpage,
        isurltitledifferent,
        mobiletitle,
    } = payload;

    sqlx::query!(
        r#"
        INSERT INTO public.articles (id, title, content, authorid, ispublished, language, readtime, subheading, isarchived, realtitle, ismainpage, isurltitledifferent, mobiletitle)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
        new_id,
        title,
        content,
        authorid,
        ispublished,
        language,
        readtime,
        subheading,
        isarchived,
        realtitle,
        ismainpage,
        isurltitledifferent,
        mobiletitle
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert failed: {}", e)))?;

    let row = sqlx::query_as!(
        ArticleResponse,
        r#"
        SELECT 
            id,
            title,
            content,
            authorid,
            ispublished,
            language,
            readtime,
            subheading,
            isarchived,
            realtitle,
            ismainpage,
            isurltitledifferent,
            mobiletitle
        FROM public.articles
        WHERE id = $1
        "#,
        new_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch failed: {}", e)))?;

    Ok(Json(row))
}

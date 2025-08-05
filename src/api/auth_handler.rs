use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation, TokenData};
use std::time::{Duration, Instant};
use once_cell::sync::OnceCell;
use tokio::sync::RwLock;
use reqwest;

#[derive(Debug, Deserialize)]
pub struct TokenPayload {
    credential: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: String,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
    #[serde(rename = "iat")]
    pub issued_at: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    sub: Option<String>,
    email: Option<String>,
    fullname: Option<String>,
    picture: Option<String>,
    provider: Option<String>,
    last_sign_in_at: i64,
}

#[derive(Debug, Deserialize)]
struct GooglePublicKeys {
    keys: Vec<GoogleKey>,
}

#[derive(Debug, Deserialize, Clone)]
struct GoogleKey {
    kid: String,
    n: String,
    e: String,
}

struct CertCache {
    keys: Vec<GoogleKey>,
    expires_at: Instant,
}

static GOOGLE_CERT_CACHE: OnceCell<RwLock<Option<CertCache>>> = OnceCell::new();

async fn get_google_key(kid: &str) -> Result<GoogleKey, String> {
    let cache = GOOGLE_CERT_CACHE.get_or_init(|| RwLock::new(None));
    {
        let guard = cache.read().await;
        if let Some(ref certs) = *guard {
            if certs.expires_at > Instant::now() {
                if let Some(key) = certs.keys.iter().find(|k| k.kid == kid).cloned() {
                    return Ok(key);
                }
            }
        }
    }

    let res = reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
        .await
        .map_err(|e| format!("Failed to fetch certs: {}", e))?;

    let keys: GooglePublicKeys = res.json().await.map_err(|e| format!("Invalid certs JSON: {}", e))?;

    let expires = Instant::now() + Duration::from_secs(300);
    {
        let mut guard = cache.write().await;
        *guard = Some(CertCache {
            keys: keys.keys.clone(),
            expires_at: expires,
        });
    }

    keys.keys.into_iter().find(|k| k.kid == kid).ok_or("Key not found".into())
}

async fn verify_google_token(token: &str, expected_aud: &str) -> Result<GoogleClaims, String> {
    let header = decode_header(token).map_err(|e| e.to_string())?;
    let kid = header.kid.ok_or("Missing 'kid' in JWT header")?;

    let key = get_google_key(&kid).await?;
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[expected_aud]);
    validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);

    let token_data: TokenData<GoogleClaims> =
        decode(token, &decoding_key, &validation).map_err(|e| format!("Token decode error: {}", e))?;

    Ok(token_data.claims)
}

pub async fn handler(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<TokenPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing GOOGLE_CLIENT_ID".into()))?;

    let claims = verify_google_token(&payload.credential, &client_id)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Token verification failed: {}", e)))?;

    let now = Utc::now().timestamp();

    let auth = sqlx::query!(
        r#"
        SELECT id FROM public.auth
        WHERE provider = $1 AND sub = $2
        "#,
        "Google",
        claims.sub,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let auth_id = if let Some(existing) = auth {
        sqlx::query!(
            r#"
            UPDATE public.auth
            SET last_sign_in_at = $1
            WHERE id = $2
            "#,
            now,
            existing.id
        )
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        existing.id
    } else {
        let new_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO public.auth (id, provider, sub, email, fullname, picture, created_at, updated_at, last_sign_in_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $7)
            "#,
            new_id,
            "Google",
            claims.sub,
            claims.email,
            claims.name,
            claims.picture,
            now,
        )
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to insert auth: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO public.users (auth_id, picture, username, email, fullname, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            "#,
            new_id,
            claims.picture,
            claims.email,
            claims.email,
            claims.name,
            now,
        )
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to insert user: {}", e)))?;

        new_id
    };

    let row = sqlx::query_as!(
        AuthResponse,
        r#"
        SELECT 
            sub,
            email,
            fullname,
            picture,
            provider,
            last_sign_in_at
        FROM public.auth
        WHERE id = $1
        "#,
        auth_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch after insert failed: {}", e)))?;

    Ok(Json(row))
}

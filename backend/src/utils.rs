use axum::http::HeaderMap;
use hmac::{Hmac, Mac};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::Sha256;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

use crate::config::{env_secs, env_secs_opt, env_u32};
use crate::error::AppError;

type HmacSha256 = Hmac<Sha256>;

pub async fn make_pool() -> anyhow::Result<PgPool> {
    let database_url =
        env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let max_connections = env_u32("DATABASE_MAX_CONNECTIONS", 20)?;
    let min_connections = env_u32("DATABASE_MIN_CONNECTIONS", 2)?;
    let acquire_timeout = env_secs("DATABASE_ACQUIRE_TIMEOUT_SECS", 5)?;
    let idle_timeout = env_secs_opt("DATABASE_IDLE_TIMEOUT_SECS", 60 * 5)?;
    let max_lifetime = env_secs_opt("DATABASE_MAX_LIFETIME_SECS", 0)?;

    let mut options = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(acquire_timeout)
        .idle_timeout(idle_timeout);

    if let Some(max_lifetime) = max_lifetime {
        options = options.max_lifetime(max_lifetime);
    }

    let pool = options.connect(&database_url).await?;

    tracing::info!(
        max_connections = max_connections,
        min_connections = min_connections,
        "connected to PostgreSQL"
    );
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("migrations applied");
    Ok(())
}

pub fn hash_api_key(secret: &[u8], api_key: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret).expect("HMAC-SHA256 accepts a key of any length");
    mac.update(api_key.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn generate_api_key() -> String {
    let mut buf = [0; 32];
    OsRng.fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn bearer(headers: &HeaderMap) -> Result<String, AppError> {
    headers
        .get("authorization")
        .ok_or_else(|| AppError::Unauthorized("missing Authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("invalid Authorization header".to_string()))?
        .strip_prefix("Bearer ")
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AppError::Unauthorized("expected `Bearer <api_key>`".to_string()))
}

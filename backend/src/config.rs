use axum_extra::extract::cookie::Key;
use std::env;
use std::time::Duration;

use crate::entities::admin::{Admin, LoginRequest};
use crate::push::crypto::derive_key;

#[derive(Clone)]
pub struct AppConfig {
    pub bind_addr: String,
    pub admin: Admin,
    pub default_vapid_subject: String,
    pub secure_mode: bool,
    pub send_max_concurrency: u32,
    pub cookie_key: Key,
    pub api_key_secret: Vec<u8>,
    pub vapid_key_encryption_key: [u8; 32],
}

impl AppConfig {
    pub fn is_admin(&self, request: LoginRequest) -> bool {
        self.admin.login == request.login && self.admin.password == request.password
    }
}

pub fn env_string(name: &'static str, default: &'static str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

pub fn env_u32(name: &str, default: u32) -> anyhow::Result<u32> {
    match env::var(name) {
        Ok(v) => v
            .parse()
            .map_err(|_| anyhow::anyhow!("{name} must be a positive integer, got {v:?}")),
        Err(_) => Ok(default),
    }
}

pub fn env_secs(name: &str, default_secs: u64) -> anyhow::Result<Duration> {
    match env::var(name) {
        Ok(v) => v
            .parse::<u64>()
            .map(Duration::from_secs)
            .map_err(|_| anyhow::anyhow!("{name} must be seconds as an integer, got {v:?}")),
        Err(_) => Ok(Duration::from_secs(default_secs)),
    }
}

pub fn env_secs_opt(name: &str, default_secs: u64) -> anyhow::Result<Option<Duration>> {
    let secs = env_secs(name, default_secs)?.as_secs();
    Ok(if secs == 0 {
        None
    } else {
        Some(Duration::from_secs(secs))
    })
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            bind_addr: env_string("BIND_ADDR", "0.0.0.0:8080"),
            default_vapid_subject: env_string("DEFAULT_VAPID_SUBJECT", "mailto:admin@example.com"),
            admin: Admin {
                login: env_string("ADMIN_LOGIN", "admin"),
                password: env_string("ADMIN_PASSWORD", "admin"),
            },
            cookie_key: match env::var("COOKIE_SECRET") {
                Ok(secret) => Key::from(secret.as_bytes()),
                Err(_) => Key::generate(),
            },
            api_key_secret: env::var("API_KEY_HASH_SECRET")
                .map_err(|_| anyhow::anyhow!("API_KEY_HASH_SECRET must be set"))?
                .into_bytes(),
            vapid_key_encryption_key: derive_key(
                env::var("VAPID_KEY_ENCRYPTION_SECRET")
                    .map_err(|_| anyhow::anyhow!("VAPID_KEY_ENCRYPTION_SECRET must be set"))?
                    .as_bytes(),
            ),
            secure_mode: env::var("SECURE_MODE")
                .unwrap_or_default()
                .parse::<bool>()
                .unwrap_or(false),
            send_max_concurrency: env_u32("SEND_MAX_CONCURRENCY", 50)?,
        })
    }
}

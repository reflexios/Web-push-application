pub mod auth;
pub mod clients;
pub mod send;
pub mod subscriptions;

use axum::Json;
use serde_json::{Value, json};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "push-platform"
    }))
}

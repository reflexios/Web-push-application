use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SendRequest {
    #[serde(default)]
    pub subscription_id: Option<Uuid>,

    #[serde(default)]
    pub subscription_ids: Option<Vec<Uuid>>,

    #[serde(default)]
    pub to_all: bool,

    pub payload: serde_json::Value,

    #[serde(default)]
    pub ttl: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SendReport {
    pub client_id: Uuid,
    pub requested: usize,
    pub delivered: usize,
    pub gone: usize,
    pub failed: usize,
    pub details: HashMap<Uuid, String>,
}

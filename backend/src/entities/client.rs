use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub vapid_private_key: String,
    pub vapid_public_key: String,
    pub vapid_subject: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ClientPublic {
    #[sqlx(rename = "id")]
    pub client_id: Uuid,
    pub name: String,
    pub vapid_public_key: String,
    pub vapid_subject: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ClientCreated {
    pub client_id: Uuid,
    pub name: String,
    pub api_key: String,
    pub vapid_public_key: String,
    pub vapid_subject: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
    pub name: String,
    #[serde(default)]
    pub vapid_subject: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListClientsQuery {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub per_page: Option<i64>,
    #[serde(default)]
    pub search: Option<String>,
}

impl ListClientsQuery {
    const DEFAULT_PER_PAGE: i64 = 20;
    const MAX_PER_PAGE: i64 = 100;

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page
            .unwrap_or(Self::DEFAULT_PER_PAGE)
            .clamp(1, Self::MAX_PER_PAGE)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }

    pub fn search_pattern(&self) -> Option<String> {
        self.search
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{s}%"))
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedClients {
    pub items: Vec<ClientPublic>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

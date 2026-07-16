use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub client_id: Uuid,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SubscriptionPublic {
    pub id: Uuid,
    pub client_id: Uuid,
    pub endpoint: String,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub client_id: Uuid,
    pub subscription: SubscriptionData,
    #[serde(default)]
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionData {
    pub endpoint: String,
    pub keys: SubscriptionKeys,
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionCreated {
    pub subscription_id: Uuid,
    pub client_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListSubscriptionsQuery {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub per_page: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct PaginatedSubscriptions {
    pub items: Vec<SubscriptionPublic>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

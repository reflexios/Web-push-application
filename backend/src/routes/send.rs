use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use uuid::Uuid;

use crate::AppState;
use crate::entities::client::Client;
use crate::entities::send::{SendReport, SendRequest};
use crate::entities::subscription::Subscription;
use crate::error::AppError;
use crate::push::crypto::decrypt_private_key;
use crate::push::sender::{DeliveryOutcome, send_with_retry};
use crate::utils::{bearer, hash_api_key};

async fn lookup_client_by_api_key(
    pool: &sqlx::PgPool,
    api_key_secret: &[u8],
    api_key: &str,
) -> Result<Client, AppError> {
    let hash = hash_api_key(api_key_secret, api_key);

    let row  = sqlx::query_as::<_, Client>(
        r#"
        SELECT id, name, api_key_hash, vapid_private_key, vapid_public_key, vapid_subject, created_at
        FROM clients
        WHERE api_key_hash = $1
        "#,
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?;

    row.ok_or_else(|| AppError::Unauthorized("invalid api_key".to_string()))
}

async fn collect_subscriptions(
    pool: &sqlx::PgPool,
    client_id: Uuid,
    req: &SendRequest,
) -> Result<Vec<Subscription>, AppError> {
    if req.to_all {
        let rows = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT id, client_id, endpoint, p256dh, auth
            FROM subscriptions
            WHERE client_id = $1
            "#,
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;
        return Ok(rows);
    }

    if let Some(ids) = &req.subscription_ids {
        if ids.is_empty() {
            return Err(AppError::BadRequest(
                "subscription_ids is empty".to_string(),
            ));
        }
        let rows = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT id, client_id, endpoint, p256dh, auth
            FROM subscriptions
            WHERE client_id = $1 AND id = ANY($2)
            "#,
        )
        .bind(client_id)
        .bind(ids)
        .fetch_all(pool)
        .await?;
        return Ok(rows);
    }

    if let Some(id) = req.subscription_id {
        let row = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT id, client_id, endpoint, p256dh, auth
            FROM subscriptions
            WHERE id = $1 AND client_id = $2
            "#,
        )
        .bind(id)
        .bind(client_id)
        .fetch_optional(pool)
        .await?;
        return Ok(row.into_iter().collect());
    }

    Err(AppError::BadRequest(
        "specify either `subscription_id`, `subscription_ids` or `to_all: true`".to_string(),
    ))
}

pub async fn send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SendRequest>,
) -> Result<(StatusCode, Json<SendReport>), AppError> {
    let api_key = bearer(&headers)?;
    let client =
        lookup_client_by_api_key(&state.pool, &state.config.api_key_secret, &api_key).await?;

    let subs = collect_subscriptions(&state.pool, client.id, &req).await?;
    if subs.is_empty() {
        return Err(AppError::BadRequest(
            "no subscriptions to send to".to_string(),
        ));
    }

    let decrypted_vapid_private_key = decrypt_private_key(
        &state.config.vapid_key_encryption_key,
        &client.vapid_private_key,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("vapid key decryption: {e}")))?;

    let payload = req.payload.to_string();

    let total = subs.len();
    let results = stream::iter(subs)
        .map(|sub| {
            let push_http = state.push_client.clone();
            let payload = payload.clone();
            let vapid_private = decrypted_vapid_private_key.clone();
            let vapid_subject = client.vapid_subject.clone();
            let ttl = req.ttl;
            async move {
                let res = send_with_retry(
                    &push_http,
                    &sub,
                    &vapid_private,
                    &vapid_subject,
                    &payload,
                    ttl,
                )
                .await;
                (sub.id, res)
            }
        })
        .buffer_unordered(state.config.send_max_concurrency as usize)
        .collect::<Vec<_>>()
        .await;

    let mut delivered = 0;
    let mut gone = 0;
    let mut failed = 0;
    let mut details: HashMap<Uuid, String> = HashMap::new();
    let mut to_delete: Vec<Uuid> = Vec::new();

    for (sub_id, outcome) in results {
        match outcome {
            Ok(DeliveryOutcome::Delivered) => {
                delivered += 1;
                details.insert(sub_id, "delivered".to_string());
            }
            Ok(DeliveryOutcome::SubscriptionGone) => {
                gone += 1;
                to_delete.push(sub_id);
                details.insert(sub_id, "gone - will be deleted".to_string());
            }
            Ok(DeliveryOutcome::Retriable(reason)) => {
                failed += 1;
                details.insert(sub_id, format!("retriable: {reason}"));
            }
            Ok(DeliveryOutcome::Failed(reason)) => {
                failed += 1;
                details.insert(sub_id, format!("failed: {reason}"));
            }
            Err(e) => {
                failed += 1;
                details.insert(sub_id, format!("error: {e}"));
            }
        }
    }

    if !to_delete.is_empty() {
        if let Err(e) = sqlx::query("DELETE FROM subscriptions WHERE id = ANY($1)")
            .bind(&to_delete)
            .execute(&state.pool)
            .await
        {
            tracing::error!(error = ?e, "failed to delete gone subscriptions");
        } else {
            tracing::info!(count = to_delete.len(), "gone subscriptions deleted");
        }
    }

    tracing::info!(
        client_id = %client.id,
        requested = total,
        delivered, gone, failed,
        "send batch finished"
    );

    Ok((
        StatusCode::OK,
        Json(SendReport {
            client_id: client.id,
            requested: total,
            delivered,
            gone,
            failed,
            details,
        }),
    ))
}

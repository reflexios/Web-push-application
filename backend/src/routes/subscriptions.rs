use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use uuid::Uuid;

use crate::AppState;
use crate::entities::subscription::{
    CreateSubscriptionRequest, ListSubscriptionsQuery, PaginatedSubscriptions, Subscription,
    SubscriptionCreated, SubscriptionPublic,
};
use crate::error::AppError;
use crate::routes::auth::RequireAuth;
use crate::utils::{bearer, hash_api_key};

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    expected_client_id: Uuid,
) -> Result<(), AppError> {
    let token = bearer(headers)?;
    let token_hash = hash_api_key(&state.config.api_key_secret, &token);

    let exists: Option<(Uuid,)> =
        sqlx::query_as(r#"SELECT id FROM clients WHERE id = $1 AND api_key_hash = $2"#)
            .bind(expected_client_id)
            .bind(token_hash)
            .fetch_optional(&state.pool)
            .await?;

    match exists {
        Some(_) => Ok(()),
        None => Err(AppError::Unauthorized(
            "invalid api_key for client_id".to_string(),
        )),
    }
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<(StatusCode, Json<SubscriptionCreated>), AppError> {
    authorize(&state, &headers, req.client_id).await?;

    let row = sqlx::query_as::<_, Subscription>(
        r#"
        INSERT INTO subscriptions (client_id, endpoint, p256dh, auth, user_agent)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (client_id, endpoint) DO UPDATE
        SET p256dh    = EXCLUDED.p256dh,
            auth      = EXCLUDED.auth,
            updated_at = NOW()
        RETURNING id, client_id, endpoint, p256dh, auth
        "#,
    )
    .bind(req.client_id)
    .bind(&req.subscription.endpoint)
    .bind(&req.subscription.keys.p256dh)
    .bind(&req.subscription.keys.auth)
    .bind(req.user_agent.as_deref())
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(
        subscription_id = %row.id,
        client_id = %row.client_id,
        endpoint = %row.endpoint,
        "subscription upserted"
    );

    Ok((
        StatusCode::CREATED,
        Json(SubscriptionCreated {
            subscription_id: row.id,
            client_id: row.client_id,
        }),
    ))
}

pub async fn list(
    _auth: RequireAuth,
    State(state): State<AppState>,
    Path(client_id): Path<Uuid>,
    Query(query): Query<ListSubscriptionsQuery>,
) -> Result<Json<PaginatedSubscriptions>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total = sqlx::query_scalar(r#"SELECT COUNT(*) FROM subscriptions WHERE client_id = $1"#)
        .bind(client_id)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, SubscriptionPublic>(
        r#"
        SELECT id, client_id, endpoint, user_agent, created_at, updated_at
        FROM subscriptions
        WHERE client_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(client_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total_pages = if total == 0 {
        0
    } else {
        (total + per_page - 1) / per_page
    };

    Ok(Json(PaginatedSubscriptions {
        items,
        page,
        per_page,
        total,
        total_pages,
    }))
}

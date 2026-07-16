use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::AppState;
use crate::entities::client::{
    Client, ClientCreated, ClientPublic, CreateClientRequest, ListClientsQuery, PaginatedClients,
};
use crate::error::AppError;
use crate::push::crypto::encrypt_private_key;
use crate::push::vapid::generate_vapid;
use crate::routes::auth::RequireAuth;
use crate::utils::{generate_api_key, hash_api_key};

pub async fn list(
    _auth: RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<ListClientsQuery>,
) -> Result<Json<PaginatedClients>, AppError> {
    let page = query.page();
    let per_page = query.per_page();
    let offset = query.offset();
    let search = query.search_pattern();

    let total = match &search {
        Some(pattern) => {
            sqlx::query_scalar(r#"SELECT COUNT(*) FROM clients WHERE name ILIKE $1"#)
                .bind(pattern)
                .fetch_one(&state.pool)
                .await?
        }
        None => {
            sqlx::query_scalar(r#"SELECT COUNT(*) FROM clients"#)
                .fetch_one(&state.pool)
                .await?
        }
    };

    let items = match &search {
        Some(pattern) => {
            sqlx::query_as::<_, ClientPublic>(
                r#"
                SELECT id, name, vapid_public_key, vapid_subject, created_at
                FROM clients
                WHERE name ILIKE $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(pattern)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, ClientPublic>(
                r#"
                SELECT id, name, vapid_public_key, vapid_subject, created_at
                FROM clients
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
    };

    let total_pages = if total == 0 {
        0
    } else {
        (total + per_page - 1) / per_page
    };

    Ok(Json(PaginatedClients {
        items,
        page,
        per_page,
        total,
        total_pages,
    }))
}

pub async fn get_one(
    _auth: RequireAuth,
    State(state): State<AppState>,
    Path(client_id): Path<Uuid>,
) -> Result<Json<ClientPublic>, AppError> {
    let row = sqlx::query_as::<_, ClientPublic>(
        r#"
        SELECT id, name, vapid_public_key, vapid_subject, created_at
        FROM clients
        WHERE id = $1
        "#,
    )
    .bind(client_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(row))
}

pub async fn create(
    _auth: RequireAuth,
    State(state): State<AppState>,
    Json(req): Json<CreateClientRequest>,
) -> Result<(StatusCode, Json<ClientCreated>), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }

    let keys =
        generate_vapid().map_err(|e| AppError::Internal(anyhow::anyhow!("vapid generate: {e}")))?;

    let api_key = generate_api_key();
    let api_key_hash = hash_api_key(&state.config.api_key_secret, &api_key);

    let subject = req
        .vapid_subject
        .unwrap_or_else(|| state.config.default_vapid_subject.clone());

    let encrypted_private_key =
        encrypt_private_key(&state.config.vapid_key_encryption_key, &keys.private_pem)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("vapid key encryption: {e}")))?;

    let row = sqlx::query_as::<_, Client>(
        r#"
        INSERT INTO clients (name, api_key_hash, vapid_private_key, vapid_public_key, vapid_subject)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, api_key_hash, vapid_private_key, vapid_public_key, vapid_subject, created_at
        "#,
    )
    .bind(&req.name)
    .bind(&api_key_hash)
    .bind(&encrypted_private_key)
    .bind(&keys.public_base64)
    .bind(&subject)
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(client_id = %row.id, "new client registered");

    Ok((
        StatusCode::CREATED,
        Json(ClientCreated {
            client_id: row.id,
            name: row.name,
            api_key,
            vapid_public_key: row.vapid_public_key,
            vapid_subject: row.vapid_subject,
            created_at: row.created_at,
        }),
    ))
}

pub async fn regenerate_api_key(
    _auth: RequireAuth,
    State(state): State<AppState>,
    Path(client_id): Path<Uuid>,
) -> Result<Json<ClientCreated>, AppError> {
    let api_key = generate_api_key();
    let api_key_hash = hash_api_key(&state.config.api_key_secret, &api_key);

    let row = sqlx::query_as::<_, Client>(
        r#"
        UPDATE clients
        SET api_key_hash = $2
        WHERE id = $1
        RETURNING id, name, api_key_hash, vapid_private_key, vapid_public_key, vapid_subject, created_at
        "#,
    )
    .bind(client_id)
    .bind(&api_key_hash)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    tracing::info!(client_id = %row.id, "api key regenerated");

    Ok(Json(ClientCreated {
        client_id: row.id,
        name: row.name,
        api_key,
        vapid_public_key: row.vapid_public_key,
        vapid_subject: row.vapid_subject,
        created_at: row.created_at,
    }))
}

pub async fn delete(
    _auth: RequireAuth,
    State(state): State<AppState>,
    Path(client_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(r#"DELETE FROM clients WHERE id = $1"#)
        .bind(client_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    tracing::info!(client_id = %client_id, "client deleted");

    Ok(StatusCode::NO_CONTENT)
}

mod config;
mod entities;
mod error;
mod push;
mod routes;
mod utils;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{get, post};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use web_push::IsahcWebPushClient;

use crate::config::{AppConfig, env_string};
use crate::utils::{make_pool, run_migrations};

#[derive(Clone)]
struct AppState {
    pool: sqlx::PgPool,
    push_client: Arc<IsahcWebPushClient>,
    config: Arc<AppConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .init();

    let cfg = AppConfig::from_env()?;
    let pool = make_pool().await?;
    run_migrations(&pool).await?;

    let addr: SocketAddr = cfg.bind_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("push-platform listening on http://{addr}");

    let push_client = Arc::new(IsahcWebPushClient::new()?);
    let config = Arc::new(cfg);

    let state = AppState {
        pool,
        push_client,
        config,
    };

    let admin_origin = env_string("ADMIN_ORIGIN", "http://localhost:5173")
        .parse()
        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:5173"));

    let admin_routes = Router::new()
        .route("/login", post(routes::auth::login))
        .route("/logout", post(routes::auth::logout))
        .route(
            "/clients",
            get(routes::clients::list).post(routes::clients::create),
        )
        .route(
            "/clients/:client_id",
            get(routes::clients::get_one).delete(routes::clients::delete),
        )
        .route(
            "/clients/:client_id/regenerate-api-key",
            post(routes::clients::regenerate_api_key),
        )
        .route(
            "/clients/:client_id/subscriptions",
            get(routes::subscriptions::list),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(admin_origin)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE]),
        );

    let public_routes = Router::new()
        .route("/health", get(routes::health))
        .route("/subscriptions", post(routes::subscriptions::create))
        .route("/send", post(routes::send::send))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let app = Router::new()
        .merge(admin_routes)
        .merge(public_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::serve(listener, app).await?;

    Ok(())
}

mod config;
mod entities;
mod error;
mod push;
mod routes;
mod utils;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{get, post};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower::Service;
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

    let use_socket = env::var("USE_UNIX_SOCKET")
        .unwrap_or_default()
        .parse::<bool>()
        .unwrap_or(false);

    if use_socket {
        let socket_path = "/tmp/push-platform.sock";
        let path = PathBuf::from(&socket_path);

        let _ = std::fs::remove_file(&path);
        std::fs::create_dir_all(path.parent().unwrap())?;

        let uds = tokio::net::UnixListener::bind(&path)?;
        tracing::info!("push-platform listening on unix socket {socket_path}");

        let mut make_service = app.into_make_service();

        loop {
            let (socket, _remote_addr) = uds.accept().await?;
            let tower_service = make_service.call(&socket).await?;

            tokio::spawn(async move {
                let socket = TokioIo::new(socket);

                let hyper_service =
                    hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
                        tower_service.clone().call(request)
                    });

                if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(socket, hyper_service)
                    .await
                {
                    tracing::error!("failed to serve connection: {err:#}");
                }
            });
        }
    } else {
        let port = env_string("BACKEND_PORT", "8080");
        let bind_addr = format!("0.0.0.0:{}", port);
        let addr: SocketAddr = bind_addr.parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("push-platform listening on http://{addr}");
        axum::serve(listener, app).await?;
    }

    Ok(())
}

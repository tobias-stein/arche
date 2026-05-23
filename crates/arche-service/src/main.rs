mod config;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info;

use config::Config;

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

fn build_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arche_service=info,tower_http=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let config = Config::from_env();

    info!(
        port = config.port,
        mode = if config.redis_url.is_some() { "cluster" } else { "single" },
        db_host = %extract_db_host(&config.database_url),
        "starting server",
    );

    let router = build_router();

    let addr: SocketAddr = config.bind_addr().parse().expect("invalid bind address");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

fn extract_db_host(database_url: &str) -> &str {
    database_url
        .strip_prefix("postgres://")
        .and_then(|rest| rest.find('@').map(|i| &rest[..i]))
        .unwrap_or("unknown")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received SIGINT, shutting down"),
        _ = terminate => info!("received SIGTERM, shutting down"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let router = build_router();

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();

        assert_eq!(body, json!({ "status": "ok" }));
    }
}

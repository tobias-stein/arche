pub mod affixes;
pub mod audit_log;
pub mod auth;
pub mod batch;
mod bootstrap;
pub mod blueprints;
pub mod clients;
pub mod me;
mod config;
pub mod export;
pub mod generate;
pub mod generation;
pub mod global_meta_attributes;
pub mod import;
mod schema;
pub mod cache;
pub mod pagination;
pub mod error;
pub mod redis_pubsub;

use axum::extract::State;
use axum::middleware;
use axum::{routing::{delete, get, post}, Json, Router};
use arche_types::crud::BootstrapResponse;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::warn;

use bootstrap::bootstrap_super_admin;
use cache::Cache;
use config::Config;
use import::ImportStaging;
use redis_pubsub::RedisPubSubHandle;
use schema::{get_affixes_schema, get_batch_generate_schema, get_blueprints_schema, get_generate_schema};

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct AppState {
    cache: Arc<RwLock<Cache>>,
    pool: Arc<PgPool>,
    redis: Option<RedisPubSubHandle>,
    pub(crate) import_staging: Arc<ImportStaging>,
}

impl axum::extract::FromRef<AppState> for Arc<PgPool> {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn bootstrap_handler(State(state): State<AppState>) -> Json<BootstrapResponse> {
    let key = bootstrap_super_admin(&state.pool).await;
    if let Some(raw_key) = key {
        Json(BootstrapResponse {
            bootstrapped: true,
            key: Some(raw_key),
            message: None,
        })
    } else {
        Json(BootstrapResponse {
            bootstrapped: false,
            key: None,
            message: Some(
                "Already bootstrapped. Super admin key available in server logs.".into(),
            ),
        })
    }
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/bootstrap", get(bootstrap_handler))
        .route("/api/schema/blueprints", get(get_blueprints_schema))
        .route("/api/schema/affixes", get(get_affixes_schema))
        .route("/api/schema/generate", get(get_generate_schema))
        .route("/api/schema/generate/batch", get(get_batch_generate_schema))
        .route("/api/me", get(me::me_handler))
        .route("/api/blueprints", get(blueprints::list_blueprints).post(blueprints::create_blueprint))
        .route("/api/blueprints/{id}", get(blueprints::get_blueprint).put(blueprints::update_blueprint).delete(blueprints::delete_blueprint))
        .route("/api/blueprints/batch/edit", post(batch::batch_edit_blueprints))
        .route("/api/blueprints/batch/delete", post(batch::batch_delete_blueprints))
        .route("/api/blueprints/batch/assign", post(batch::batch_assign_blueprints))
        .route("/api/affixes", get(affixes::list_affixes).post(affixes::create_affix))
        .route("/api/affixes/{id}", get(affixes::get_affix).put(affixes::update_affix).delete(affixes::delete_affix))
        .route("/api/affixes/batch/delete", post(batch::batch_delete_affixes))
        .route("/api/affixes/batch/assign", post(batch::batch_assign_affixes))
        .route("/api/global-meta-attributes", get(global_meta_attributes::list_global_meta_attributes).post(global_meta_attributes::create_global_meta_attribute))
        .route("/api/global-meta-attributes/{id}", get(global_meta_attributes::get_global_meta_attribute).put(global_meta_attributes::update_global_meta_attribute).delete(global_meta_attributes::delete_global_meta_attribute))
        .route("/api/audit-log", get(audit_log::list_audit_log))
        .route("/api/generate", post(generate::generate_handler))
        .route("/api/generate/batch", post(generate::generate_batch_handler))
        .route("/api/clients", get(clients::list_clients).post(clients::create_client))
        .route("/api/clients/{id}", get(clients::get_client).delete(clients::delete_client))
        .route("/api/clients/{client_id}/keys", get(clients::list_api_keys).post(clients::create_api_key))
        .route("/api/clients/{client_id}/keys/{key_id}", delete(clients::delete_api_key))
        .route("/api/export", post(export::export_handler))
        .route("/api/import", post(import::import_parse_handler))
        .route("/api/import/resolve", post(import::import_resolve_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::permission::permission_middleware,
        ))
        .with_state(state)
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
        db_pool_size = config.db_pool_size,
        "starting server",
    );

    let pool = PgPoolOptions::new()
        .max_connections(config.db_pool_size)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("../../arche-service/migrations")
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    let pool = Arc::new(pool);

    let super_admin_key = bootstrap_super_admin(&pool).await;

    if let Some(ref key) = super_admin_key {
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║               === SUPER ADMIN API KEY ===                  ║");
        println!("║                                                            ║");
        println!("║  {:<58}║", key);
        println!("║                                                            ║");
        println!("║  Store this key securely. It will not be shown again.      ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
    }

    info!("loading cache from database");
    let cache = Cache::load(&pool)
        .await
        .expect("failed to load cache from database");
    info!(
        clients = cache.clients.len(),
        blueprints = cache.blueprints.len(),
        affixes = cache.affixes.len(),
        "cache loaded"
    );

    let cache = Arc::new(RwLock::new(cache));

    let _poll_handle = cache::start_cache_poller(
        (*pool).clone(),
        cache.clone(),
        config.cache_poll_interval_ms,
    );

    let redis_handle = if let Some(ref redis_url) = config.redis_url {
        match redis_pubsub::connect_redis(redis_url).await {
            Ok(client) => {
                info!("redis: connected, starting pub/sub");
                let handle = redis_pubsub::start_redis_pubsub(client, pool.clone(), cache.clone());
                Some(handle)
            }
            Err(e) => {
                warn!(error = %e, "redis: connection failed, falling back to polling-only mode");
                None
            }
        }
    } else {
        None
    };

    let state = AppState { cache, pool, redis: redis_handle, import_staging: Arc::new(ImportStaging::new()) };
    let router = build_router(state);

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
    let without_scheme = match database_url.strip_prefix("postgres://") {
        Some(s) => s,
        None => return "unknown",
    };
    let host_part = without_scheme.split('@').next_back().unwrap_or(without_scheme);
    let host = host_part
        .split_once(':')
        .or_else(|| host_part.split_once('/'))
        .map(|(h, _)| h)
        .unwrap_or(host_part);
    if host.is_empty() { "unknown" } else { host }
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

    fn make_test_state() -> AppState {
        AppState {
            cache: Arc::new(RwLock::new(Cache::new(
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
            ))),
            pool: Arc::new(PgPool::connect_lazy("postgres://localhost/test").expect("lazy pool")),
            redis: None,
            import_staging: Arc::new(ImportStaging::new()),
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let router = build_router(make_test_state());

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

    #[test]
    fn test_extract_db_host_standard() {
        assert_eq!(
            extract_db_host("postgres://user:pass@localhost:5432/mydb"),
            "localhost",
        );
    }

    #[test]
    fn test_extract_db_host_no_credentials() {
        assert_eq!(
            extract_db_host("postgres://localhost:5432/mydb"),
            "localhost",
        );
    }

    #[test]
    fn test_extract_db_host_no_port() {
        assert_eq!(
            extract_db_host("postgres://localhost/mydb"),
            "localhost",
        );
    }

    #[test]
    fn test_extract_db_host_host_only() {
        assert_eq!(
            extract_db_host("postgres://localhost"),
            "localhost",
        );
    }

    #[test]
    fn test_extract_db_host_non_postgres_url() {
        assert_eq!(extract_db_host("sqlite://foo.db"), "unknown");
    }

    #[test]
    fn test_extract_db_host_empty_after_scheme() {
        assert_eq!(extract_db_host("postgres://"), "unknown");
    }

    #[tokio::test]
    async fn test_cors_preflight_returns_200_with_headers() {
        let router = build_router(make_test_state());

        let response = router
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/health")
                    .header("Origin", "http://localhost:3000")
                    .header("Access-Control-Request-Method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .headers()
            .get("Access-Control-Allow-Origin")
            .is_some());
    }

    #[tokio::test]
    async fn test_cors_actual_response_includes_allow_origin() {
        let router = build_router(make_test_state());

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("Origin", "http://localhost:3000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .headers()
            .get("Access-Control-Allow-Origin")
            .is_some());
    }
}

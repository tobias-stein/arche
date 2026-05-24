use std::sync::Arc;

use arche_types::Permission;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::ProblemResponse;

pub mod permission;

#[derive(Debug, Clone)]
pub struct AuthenticatedKey {
    pub id: Uuid,
    pub name: String,
    pub client_id: Option<Uuid>,
    pub permissions: Vec<Permission>,
    pub is_super: bool,
}

impl<S> FromRequestParts<S> for AuthenticatedKey
where
    S: Send + Sync,
    Arc<PgPool>: FromRef<S>,
{
    type Rejection = ProblemResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Arc::<PgPool>::from_ref(state);

        let api_key = parts
            .headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ProblemResponse::unauthorized("Missing X-API-Key header"))?;

        verify_api_key(api_key, &pool).await
    }
}

pub(super) async fn verify_api_key(
    raw_key: &str,
    pool: &PgPool,
) -> Result<AuthenticatedKey, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT id, client_id, name, key_hash, permissions, is_super, expires_at FROM api_keys",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "auth: failed to query api_keys");
        ProblemResponse::unauthorized("Authentication failed")
    })?;

    for row in &rows {
        let key_hash: String = row.get("key_hash");
        if bcrypt::verify(raw_key, &key_hash).unwrap_or(false) {
            let id: Uuid = row.get("id");
            let client_id: Option<Uuid> = row.get("client_id");
            let name: String = row.get("name");
            let permissions_str: Vec<String> = row.get("permissions");
            let is_super: bool = row.get("is_super");
            let expires_at: Option<chrono::DateTime<Utc>> = row.get("expires_at");

            if let Some(expires_at) = expires_at {
                if Utc::now() > expires_at {
                    return Err(ProblemResponse::unauthorized("API key has expired"));
                }
            }

            let permissions: Vec<Permission> = permissions_str
                .iter()
                .filter_map(|p| parse_permission(p.as_str()))
                .collect();

            return Ok(AuthenticatedKey {
                id,
                name,
                client_id,
                permissions,
                is_super,
            });
        }
    }

    Err(ProblemResponse::unauthorized("Invalid API key"))
}

fn parse_permission(s: &str) -> Option<Permission> {
    match s {
        "read" => Some(Permission::Read),
        "write" => Some(Permission::Write),
        "delete" => Some(Permission::Delete),
        "generate" => Some(Permission::Generate),
        "admin" => Some(Permission::Admin),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;
    use axum::response::IntoResponse;

    #[test]
    fn test_authenticated_key_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AuthenticatedKey>();
    }

    #[test]
    fn test_authenticated_key_debug_clone() {
        fn assert_debug_clone<T: std::fmt::Debug + Clone>() {}
        assert_debug_clone::<AuthenticatedKey>();
    }

    #[test]
    fn test_problem_response_unauthorized_format() {
        let problem = ProblemResponse::unauthorized("Missing X-API-Key header");
        assert_eq!(problem.type_, "/errors/unauthorized");
        assert_eq!(problem.title, "Unauthorized");
        assert_eq!(problem.status, 401);
        assert_eq!(problem.detail, Some("Missing X-API-Key header".into()));
        assert!(problem.errors.is_none());
    }

    #[tokio::test]
    async fn test_missing_header_returns_401() {
        use axum::body::Body;
        use axum::http::Request;
        use axum::routing::get;
        use axum::Router;
        use tower::ServiceExt;

        async fn protected_handler(_: AuthenticatedKey) -> &'static str {
            "ok"
        }

        let pool = Arc::new(
            PgPool::connect_lazy("postgres://localhost/unreachable")
                .expect("lazy pool"),
        );

        let app = Router::new()
            .route("/protected", get(protected_handler))
            .with_state(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 401);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body.get("type").unwrap(), "/errors/unauthorized");
        assert_eq!(body.get("title").unwrap(), "Unauthorized");
        assert_eq!(body.get("status").unwrap(), 401);
        assert_eq!(
            body.get("detail").unwrap(),
            "Missing X-API-Key header"
        );
    }

    #[tokio::test]
    async fn test_invalid_key_returns_401() {
        use axum::body::Body;
        use axum::http::Request;
        use axum::routing::get;
        use axum::Router;
        use tower::ServiceExt;

        async fn protected_handler(_: AuthenticatedKey) -> &'static str {
            "ok"
        }

        let pool = Arc::new(
            PgPool::connect_lazy("postgres://localhost/unreachable")
                .expect("lazy pool"),
        );

        let app = Router::new()
            .route("/protected", get(protected_handler))
            .with_state(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("X-API-Key", "invalid-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    async fn test_problem_response_into_response_has_correct_content_type() {
        let problem = ProblemResponse::unauthorized("test");
        let response = problem.into_response();
        let ct = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, "application/problem+json");
    }

    #[test]
    fn test_permission_parsing() {
        let input = ["read", "write", "generate"];
        let permissions: Vec<Permission> = input
            .iter()
            .filter_map(|p| parse_permission(p))
            .collect();
        assert_eq!(permissions.len(), 3);
        assert!(permissions.contains(&Permission::Read));
        assert!(permissions.contains(&Permission::Write));
        assert!(permissions.contains(&Permission::Generate));
    }

    #[test]
    fn test_permission_parsing_unknown_skipped() {
        let input = ["read", "unknown", "admin"];
        let permissions: Vec<Permission> = input
            .iter()
            .filter_map(|p| parse_permission(p))
            .collect();
        assert_eq!(permissions.len(), 2);
        assert!(permissions.contains(&Permission::Read));
        assert!(permissions.contains(&Permission::Admin));
    }
}

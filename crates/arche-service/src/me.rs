use axum::Json;

use super::auth::AuthenticatedKey;
use super::error::ProblemResponse;

pub async fn me_handler(
    key: AuthenticatedKey,
) -> Result<Json<AuthenticatedKey>, ProblemResponse> {
    Ok(Json(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn make_test_app() -> Router {
        let pool = Arc::new(
            sqlx::PgPool::connect_lazy("postgres://localhost/test").expect("lazy pool"),
        );
        Router::new()
            .route("/api/me", get(me_handler))
            .with_state(pool)
    }

    #[tokio::test]
    async fn test_missing_header_returns_401() {
        let app = make_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    async fn test_invalid_key_returns_401() {
        let app = make_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .header("X-API-Key", "invalid-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 401);
    }
}

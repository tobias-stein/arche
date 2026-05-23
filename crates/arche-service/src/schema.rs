use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use once_cell::sync::Lazy;
use schemars::schema_for;
use serde_json::Value;

use arche_types::crud::{CreateAffixRequest, CreateBlueprintRequest};
use arche_types::generate::{GenerateRequest, GenerateResponse};
use arche_types::{Affix, Blueprint};

static BLUEPRINTS_SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::to_value(schema_for!(BlueprintSchemas)).unwrap());

static AFFIXES_SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::to_value(schema_for!(AffixSchemas)).unwrap());

static GENERATE_SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::to_value(schema_for!(GenerateSchemas)).unwrap());

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintSchemas {
    pub create_request: CreateBlueprintRequest,
    pub response: Blueprint,
}

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AffixSchemas {
    pub create_request: CreateAffixRequest,
    pub response: Affix,
}

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSchemas {
    pub request: GenerateRequest,
    pub response: GenerateResponse,
}

pub async fn get_blueprints_schema() -> Response {
    let body = serde_json::to_string(&*BLUEPRINTS_SCHEMA).unwrap();
    (
        StatusCode::OK,
        [
            ("content-type", HeaderValue::from_static("application/schema+json")),
        ],
        body,
    )
        .into_response()
}

pub async fn get_affixes_schema() -> Response {
    let body = serde_json::to_string(&*AFFIXES_SCHEMA).unwrap();
    (
        StatusCode::OK,
        [
            ("content-type", HeaderValue::from_static("application/schema+json")),
        ],
        body,
    )
        .into_response()
}

pub async fn get_generate_schema() -> Response {
    let body = serde_json::to_string(&*GENERATE_SCHEMA).unwrap();
    (
        StatusCode::OK,
        [
            ("content-type", HeaderValue::from_static("application/schema+json")),
        ],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
        routing::get,
    };
    use tower::ServiceExt;

    fn build_schema_router() -> Router {
        Router::new()
            .route("/api/schema/blueprints", get(get_blueprints_schema))
            .route("/api/schema/affixes", get(get_affixes_schema))
            .route("/api/schema/generate", get(get_generate_schema))
    }

    fn assert_valid_json_schema(body: &Value, label: &str) {
        assert!(body.is_object(), "{} schema should be an object", label);
        let obj = body.as_object().unwrap();
        assert!(
            obj.contains_key("$schema") || obj.contains_key("title") || obj.contains_key("type") || obj.contains_key("properties"),
            "{} schema should have at least one JSON Schema keyword",
            label,
        );
    }

    #[tokio::test]
    async fn test_blueprints_schema_endpoint() {
        let router = build_schema_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/schema/blueprints")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("missing content-type");
        assert!(content_type.to_str().unwrap().starts_with("application/schema+json"));

        let body: Value = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();

        assert_valid_json_schema(&body, "blueprints");
        let obj = body.as_object().unwrap();
        assert!(obj.contains_key("properties"), "blueprints schema should have properties");
        assert!(obj.contains_key("definitions") || obj.contains_key("$defs"), "blueprints schema should have definitions");
    }

    #[tokio::test]
    async fn test_affixes_schema_endpoint() {
        let router = build_schema_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/schema/affixes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("missing content-type");
        assert!(content_type.to_str().unwrap().starts_with("application/schema+json"));

        let body: Value = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();

        assert_valid_json_schema(&body, "affixes");
        let obj = body.as_object().unwrap();
        assert!(obj.contains_key("properties"), "affixes schema should have properties");
    }

    #[tokio::test]
    async fn test_generate_schema_endpoint() {
        let router = build_schema_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/schema/generate")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("missing content-type");
        assert!(content_type.to_str().unwrap().starts_with("application/schema+json"));

        let body: Value = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();

        assert_valid_json_schema(&body, "generate");
        let obj = body.as_object().unwrap();
        assert!(obj.contains_key("properties"), "generate schema should have properties");
    }

    #[tokio::test]
    async fn test_schemas_are_stable() {
        let router = build_schema_router();

        let resp1 = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/schema/blueprints")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body1: Value = axum::body::to_bytes(resp1.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();

        let resp2 = router
            .oneshot(
                Request::builder()
                    .uri("/api/schema/blueprints")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body2: Value = axum::body::to_bytes(resp2.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();

        assert_eq!(body1, body2, "blueprints schema should be stable across calls");
    }
}
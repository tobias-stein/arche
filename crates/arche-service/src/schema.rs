use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use once_cell::sync::Lazy;
use schemars::schema_for;
use serde_json::Value;

use arche_types::crud::{CreateAffixRequest, CreateBlueprintRequest};
use arche_types::generate::{
    BatchGenerateRequest, BatchGenerateResponse, GenerateRequest, GenerateResponse,
};
use arche_types::{Affix, Blueprint};

static BLUEPRINTS_SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::to_value(schema_for!(BlueprintSchemas)).unwrap());

static AFFIXES_SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::to_value(schema_for!(AffixSchemas)).unwrap());

static GENERATE_SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::to_value(schema_for!(GenerateSchemas)).unwrap());

static BATCH_GENERATE_SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::to_value(schema_for!(BatchGenerateSchemas)).unwrap());

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct BlueprintSchemas {
    create_request: CreateBlueprintRequest,
    response: Blueprint,
}

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AffixSchemas {
    create_request: CreateAffixRequest,
    response: Affix,
}

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GenerateSchemas {
    request: GenerateRequest,
    response: GenerateResponse,
}

#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct BatchGenerateSchemas {
    request: BatchGenerateRequest,
    response: BatchGenerateResponse,
}

fn schema_response(schema: &Value) -> Response {
    let body = serde_json::to_string(schema).unwrap();
    (
        StatusCode::OK,
        [("content-type", HeaderValue::from_static("application/schema+json"))],
        body,
    )
        .into_response()
}

pub async fn get_blueprints_schema() -> Response {
    schema_response(&BLUEPRINTS_SCHEMA)
}

pub async fn get_affixes_schema() -> Response {
    schema_response(&AFFIXES_SCHEMA)
}

pub async fn get_generate_schema() -> Response {
    schema_response(&GENERATE_SCHEMA)
}

pub async fn get_batch_generate_schema() -> Response {
    schema_response(&BATCH_GENERATE_SCHEMA)
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
            .route("/api/schema/generate/batch", get(get_batch_generate_schema))
    }

    async fn fetch_schema(router: Router, uri: &str) -> Value {
        let response = router
            .oneshot(
                Request::builder()
                    .uri(uri)
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

        axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap()
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
        let body = fetch_schema(build_schema_router(), "/api/schema/blueprints").await;
        assert_valid_json_schema(&body, "blueprints");
        let obj = body.as_object().unwrap();
        assert!(obj.contains_key("properties"), "blueprints schema should have properties");
        assert!(obj.contains_key("definitions") || obj.contains_key("$defs"), "blueprints schema should have definitions");
    }

    #[tokio::test]
    async fn test_affixes_schema_endpoint() {
        let body = fetch_schema(build_schema_router(), "/api/schema/affixes").await;
        assert_valid_json_schema(&body, "affixes");
        assert!(body.as_object().unwrap().contains_key("properties"), "affixes schema should have properties");
    }

    #[tokio::test]
    async fn test_generate_schema_endpoint() {
        let body = fetch_schema(build_schema_router(), "/api/schema/generate").await;
        assert_valid_json_schema(&body, "generate");
        assert!(body.as_object().unwrap().contains_key("properties"), "generate schema should have properties");
    }

    #[tokio::test]
    async fn test_batch_generate_schema_endpoint() {
        let body = fetch_schema(build_schema_router(), "/api/schema/generate/batch").await;
        assert_valid_json_schema(&body, "batch generate");
        assert!(body.as_object().unwrap().contains_key("properties"), "batch generate schema should have properties");
    }

    #[tokio::test]
    async fn test_schemas_are_stable() {
        let router = build_schema_router();
        let body1 = fetch_schema(router.clone(), "/api/schema/blueprints").await;
        let body2 = fetch_schema(router, "/api/schema/blueprints").await;
        assert_eq!(body1, body2, "blueprints schema should be stable across calls");
    }
}

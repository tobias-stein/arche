use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldError {
    pub path: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProblemResponse {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,
}

impl ProblemResponse {
    pub fn validation_error(detail: impl Into<String>, errors: Vec<FieldError>) -> Self {
        Self {
            type_: "/errors/validation-error".into(),
            title: "Validation Error".into(),
            status: 400,
            detail: Some(detail.into()),
            instance: None,
            errors: if errors.is_empty() {
                None
            } else {
                Some(errors)
            },
        }
    }

    pub fn unauthorized(detail: impl Into<String>) -> Self {
        Self {
            type_: "/errors/unauthorized".into(),
            title: "Unauthorized".into(),
            status: 401,
            detail: Some(detail.into()),
            instance: None,
            errors: None,
        }
    }

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self {
            type_: "/errors/forbidden".into(),
            title: "Forbidden".into(),
            status: 403,
            detail: Some(detail.into()),
            instance: None,
            errors: None,
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self {
            type_: "/errors/not-found".into(),
            title: "Not Found".into(),
            status: 404,
            detail: Some(detail.into()),
            instance: None,
            errors: None,
        }
    }

    pub fn no_matching_blueprints(detail: impl Into<String>) -> Self {
        Self {
            type_: "/errors/no-matching-blueprints".into(),
            title: "No Matching Blueprints".into(),
            status: 404,
            detail: Some(detail.into()),
            instance: None,
            errors: None,
        }
    }

    pub fn delete_referenced_resource(detail: impl Into<String>) -> Self {
        Self {
            type_: "/errors/delete-referenced-resource".into(),
            title: "Delete Referenced Resource".into(),
            status: 409,
            detail: Some(detail.into()),
            instance: None,
            errors: None,
        }
    }

    pub fn import_conflict(detail: impl Into<String>) -> Self {
        Self {
            type_: "/errors/import-conflict".into(),
            title: "Import Conflict".into(),
            status: 409,
            detail: Some(detail.into()),
            instance: None,
            errors: None,
        }
    }

    pub fn unprocessable_entity(detail: impl Into<String>) -> Self {
        Self {
            type_: "/errors/unprocessable-entity".into(),
            title: "Unprocessable Entity".into(),
            status: 422,
            detail: Some(detail.into()),
            instance: None,
            errors: None,
        }
    }
}

impl IntoResponse for ProblemResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_string(&self).unwrap_or_default();
        let mut response = (status, body).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            "application/problem+json".parse().unwrap(),
        );
        response
    }
}

#[derive(Debug)]
pub enum AppError {
    ValidationError {
        detail: String,
        errors: Vec<FieldError>,
    },
    Unauthorized {
        detail: Option<String>,
    },
    Forbidden {
        detail: Option<String>,
    },
    NotFound {
        detail: Option<String>,
    },
    NoMatchingBlueprints {
        detail: Option<String>,
    },
    DeleteReferencedResource {
        detail: Option<String>,
    },
    ImportConflict {
        detail: Option<String>,
    },
    UnprocessableEntity {
        detail: Option<String>,
    },
}

impl AppError {
    fn into_problem(self) -> ProblemResponse {
        match self {
            AppError::ValidationError { detail, errors } => ProblemResponse::validation_error(
                detail,
                errors,
            ),
            AppError::Unauthorized { detail } => ProblemResponse::unauthorized(
                detail.unwrap_or_else(|| "Authentication required".into()),
            ),
            AppError::Forbidden { detail } => ProblemResponse::forbidden(
                detail.unwrap_or_else(|| "Access denied".into()),
            ),
            AppError::NotFound { detail } => ProblemResponse::not_found(
                detail.unwrap_or_else(|| "Resource not found".into()),
            ),
            AppError::NoMatchingBlueprints { detail } => ProblemResponse::no_matching_blueprints(
                detail.unwrap_or_else(|| "No blueprints match the given constraints".into()),
            ),
            AppError::DeleteReferencedResource { detail } => ProblemResponse::delete_referenced_resource(
                detail.unwrap_or_else(|| "Resource is referenced by other resources".into()),
            ),
            AppError::ImportConflict { detail } => ProblemResponse::import_conflict(
                detail.unwrap_or_else(|| "Import conflicts require resolution".into()),
            ),
            AppError::UnprocessableEntity { detail } => ProblemResponse::unprocessable_entity(
                detail.unwrap_or_else(|| "Request could not be processed".into()),
            ),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.into_problem().into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use serde_json::json;

    #[test]
    fn test_field_error_serialization() {
        let fe = FieldError {
            path: "attributes.damage".into(),
            message: "min must be <= max".into(),
        };
        let value = serde_json::to_value(&fe).unwrap();
        assert_eq!(
            value,
            json!({"path": "attributes.damage", "message": "min must be <= max"})
        );
    }

    #[test]
    fn test_field_error_round_trip() {
        let fe = FieldError {
            path: "name".into(),
            message: "is required".into(),
        };
        let deserialized: FieldError = serde_json::from_value(serde_json::to_value(&fe).unwrap()).unwrap();
        assert_eq!(deserialized, fe);
    }

    #[test]
    fn test_problem_response_validation_error_with_errors() {
        let problem = ProblemResponse::validation_error(
            "attribute 'damage': range min (25.0) exceeds max (10.0)",
            vec![
                FieldError {
                    path: "attributes.damage".into(),
                    message: "min must be <= max".into(),
                },
            ],
        );
        let value = serde_json::to_value(&problem).unwrap();
        assert_eq!(value.get("type").unwrap(), "/errors/validation-error");
        assert_eq!(value.get("title").unwrap(), "Validation Error");
        assert_eq!(value.get("status").unwrap(), 400);
        assert_eq!(
            value.get("detail").unwrap(),
            "attribute 'damage': range min (25.0) exceeds max (10.0)"
        );
        assert!(value.get("instance").is_none());
        let errors = value.get("errors").unwrap().as_array().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].get("path").unwrap(), "attributes.damage");
        assert_eq!(errors[0].get("message").unwrap(), "min must be <= max");
    }

    #[test]
    fn test_problem_response_validation_error_no_errors() {
        let problem = ProblemResponse::validation_error("bad input", vec![]);
        let value = serde_json::to_value(&problem).unwrap();
        assert!(value.get("errors").is_none());
    }

    #[test]
    fn test_problem_response_unauthorized() {
        let problem = ProblemResponse::unauthorized("Invalid API key");
        assert_eq!(problem.type_, "/errors/unauthorized");
        assert_eq!(problem.title, "Unauthorized");
        assert_eq!(problem.status, 401);
        assert_eq!(problem.detail, Some("Invalid API key".into()));
    }

    #[test]
    fn test_problem_response_forbidden() {
        let problem = ProblemResponse::forbidden("Insufficient permissions");
        assert_eq!(problem.type_, "/errors/forbidden");
        assert_eq!(problem.title, "Forbidden");
        assert_eq!(problem.status, 403);
    }

    #[test]
    fn test_problem_response_not_found() {
        let problem = ProblemResponse::not_found("Blueprint not found");
        assert_eq!(problem.type_, "/errors/not-found");
        assert_eq!(problem.title, "Not Found");
        assert_eq!(problem.status, 404);
    }

    #[test]
    fn test_problem_response_no_matching_blueprints() {
        let problem = ProblemResponse::no_matching_blueprints("No matching blueprints");
        assert_eq!(problem.type_, "/errors/no-matching-blueprints");
        assert_eq!(problem.title, "No Matching Blueprints");
        assert_eq!(problem.status, 404);
    }

    #[test]
    fn test_problem_response_delete_referenced_resource() {
        let problem = ProblemResponse::delete_referenced_resource("Blueprint is referenced by affixes");
        assert_eq!(problem.type_, "/errors/delete-referenced-resource");
        assert_eq!(problem.title, "Delete Referenced Resource");
        assert_eq!(problem.status, 409);
    }

    #[test]
    fn test_problem_response_import_conflict() {
        let problem = ProblemResponse::import_conflict("Import conflicts require resolution");
        assert_eq!(problem.type_, "/errors/import-conflict");
        assert_eq!(problem.title, "Import Conflict");
        assert_eq!(problem.status, 409);
    }

    #[test]
    fn test_problem_response_unprocessable_entity() {
        let problem = ProblemResponse::unprocessable_entity("Malformed JSON");
        assert_eq!(problem.type_, "/errors/unprocessable-entity");
        assert_eq!(problem.title, "Unprocessable Entity");
        assert_eq!(problem.status, 422);
    }

    #[test]
    fn test_problem_response_serialization_round_trip() {
        let problem = ProblemResponse::validation_error(
            "test detail",
            vec![FieldError {
                path: "field1".into(),
                message: "bad value".into(),
            }],
        );
        let json = serde_json::to_string(&problem).unwrap();
        let deserialized: ProblemResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, problem);
    }

    #[test]
    fn test_problem_response_instance_field() {
        let mut problem = ProblemResponse::not_found("Not found");
        problem.instance = Some("/blueprints/123".into());
        let value = serde_json::to_value(&problem).unwrap();
        assert_eq!(value.get("instance").unwrap(), "/blueprints/123");
    }

    #[test]
    fn test_problem_response_instance_null_when_none() {
        let problem = ProblemResponse::not_found("Not found");
        let value = serde_json::to_value(&problem).unwrap();
        assert!(value.get("instance").is_none());
    }

    #[tokio::test]
    async fn test_problem_response_into_response_status_code() {
        let problem = ProblemResponse::not_found("Blueprint xyz not found");
        let response = problem.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_problem_response_into_response_content_type() {
        let problem = ProblemResponse::validation_error("bad", vec![]);
        let response = problem.into_response();
        let ct = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, "application/problem+json");
    }

    #[tokio::test]
    async fn test_problem_response_into_response_body() {
        let problem = ProblemResponse::not_found("Blueprint 123 not found");
        let response = problem.into_response();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed.get("type").unwrap(), "/errors/not-found");
        assert_eq!(parsed.get("title").unwrap(), "Not Found");
        assert_eq!(parsed.get("status").unwrap(), 404);
        assert_eq!(parsed.get("detail").unwrap(), "Blueprint 123 not found");
    }

    #[test]
    fn test_app_error_validation_error() {
        let err = AppError::ValidationError {
            detail: "Bad input".into(),
            errors: vec![FieldError {
                path: "name".into(),
                message: "is required".into(),
            }],
        };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/validation-error");
        assert_eq!(problem.status, 400);
        assert!(problem.errors.is_some());
        assert_eq!(problem.errors.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_app_error_unauthorized_default_detail() {
        let err = AppError::Unauthorized { detail: None };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/unauthorized");
        assert_eq!(problem.status, 401);
        assert_eq!(problem.detail, Some("Authentication required".into()));
    }

    #[test]
    fn test_app_error_unauthorized_custom_detail() {
        let err = AppError::Unauthorized {
            detail: Some("Invalid API key".into()),
        };
        let problem = err.into_problem();
        assert_eq!(problem.detail, Some("Invalid API key".into()));
    }

    #[test]
    fn test_app_error_forbidden_default_detail() {
        let err = AppError::Forbidden { detail: None };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/forbidden");
        assert_eq!(problem.detail, Some("Access denied".into()));
    }

    #[test]
    fn test_app_error_not_found_default_detail() {
        let err = AppError::NotFound { detail: None };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/not-found");
        assert_eq!(problem.detail, Some("Resource not found".into()));
    }

    #[test]
    fn test_app_error_no_matching_blueprints() {
        let err = AppError::NoMatchingBlueprints { detail: None };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/no-matching-blueprints");
        assert_eq!(problem.status, 404);
    }

    #[test]
    fn test_app_error_delete_referenced_resource() {
        let err = AppError::DeleteReferencedResource { detail: None };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/delete-referenced-resource");
        assert_eq!(problem.status, 409);
    }

    #[test]
    fn test_app_error_import_conflict() {
        let err = AppError::ImportConflict { detail: None };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/import-conflict");
        assert_eq!(problem.status, 409);
    }

    #[test]
    fn test_app_error_unprocessable_entity() {
        let err = AppError::UnprocessableEntity { detail: None };
        let problem = err.into_problem();
        assert_eq!(problem.type_, "/errors/unprocessable-entity");
        assert_eq!(problem.status, 422);
    }

    #[test]
    fn test_app_result_type_alias() {
        fn returns_ok() -> AppResult<String> {
            Ok("success".into())
        }
        fn returns_err() -> AppResult<String> {
            Err(AppError::NotFound {
                detail: Some("gone".into()),
            })
        }
        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }

    #[tokio::test]
    async fn test_app_error_into_response_status_codes() {
        let cases: Vec<(AppError, StatusCode)> = vec![
            (AppError::ValidationError { detail: "x".into(), errors: vec![] }, StatusCode::BAD_REQUEST),
            (AppError::Unauthorized { detail: None }, StatusCode::UNAUTHORIZED),
            (AppError::Forbidden { detail: None }, StatusCode::FORBIDDEN),
            (AppError::NotFound { detail: None }, StatusCode::NOT_FOUND),
            (AppError::NoMatchingBlueprints { detail: None }, StatusCode::NOT_FOUND),
            (AppError::DeleteReferencedResource { detail: None }, StatusCode::CONFLICT),
            (AppError::ImportConflict { detail: None }, StatusCode::CONFLICT),
            (AppError::UnprocessableEntity { detail: None }, StatusCode::UNPROCESSABLE_ENTITY),
        ];
        for (err, expected_status) in cases {
            let response = err.into_response();
            assert_eq!(response.status(), expected_status);
        }
    }

    #[tokio::test]
    async fn test_app_error_into_response_content_type() {
        let err = AppError::NotFound { detail: None };
        let response = err.into_response();
        let ct = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, "application/problem+json");
    }

    #[test]
    fn test_all_error_type_uris_match_spec() {
        let validation = ProblemResponse::validation_error("x", vec![]);
        assert_eq!(validation.type_, "/errors/validation-error");
        assert_eq!(validation.status, 400);

        let unauthorized = ProblemResponse::unauthorized("x");
        assert_eq!(unauthorized.type_, "/errors/unauthorized");
        assert_eq!(unauthorized.status, 401);

        let forbidden = ProblemResponse::forbidden("x");
        assert_eq!(forbidden.type_, "/errors/forbidden");
        assert_eq!(forbidden.status, 403);

        let not_found = ProblemResponse::not_found("x");
        assert_eq!(not_found.type_, "/errors/not-found");
        assert_eq!(not_found.status, 404);

        let no_match = ProblemResponse::no_matching_blueprints("x");
        assert_eq!(no_match.type_, "/errors/no-matching-blueprints");
        assert_eq!(no_match.status, 404);

        let delete_ref = ProblemResponse::delete_referenced_resource("x");
        assert_eq!(delete_ref.type_, "/errors/delete-referenced-resource");
        assert_eq!(delete_ref.status, 409);

        let import = ProblemResponse::import_conflict("x");
        assert_eq!(import.type_, "/errors/import-conflict");
        assert_eq!(import.status, 409);

        let unprocessable = ProblemResponse::unprocessable_entity("x");
        assert_eq!(unprocessable.type_, "/errors/unprocessable-entity");
        assert_eq!(unprocessable.status, 422);
    }
}

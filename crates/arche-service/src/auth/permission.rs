#![allow(clippy::result_large_err)]

use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::Method;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use arche_types::Permission;

use super::verify_api_key;
use crate::error::ProblemResponse;

use crate::AppState;

/// The permission required for an endpoint.
#[derive(Debug, Clone, PartialEq)]
pub enum RequiredPermission {
    Regular(Permission),
    SuperAdmin,
}

/// Public routes that do not require authentication or permission checks.
const PUBLIC_ROUTES: &[&str] = &["/health", "/api/bootstrap"];
const SCHEMA_PREFIX: &str = "/api/schema/";

/// Determine the required permission for a given HTTP method and path.
pub fn required_permission(method: &Method, path: &str) -> Option<RequiredPermission> {
    if PUBLIC_ROUTES.contains(&path) || path.starts_with(SCHEMA_PREFIX) {
        return None;
    }

    if !path.starts_with("/api/") {
        return None;
    }

    match *method {
        Method::GET => {
            if path == "/api/audit-log" || path.starts_with("/api/audit-log/") {
                return Some(RequiredPermission::SuperAdmin);
            }
            Some(RequiredPermission::Regular(Permission::Read))
        }
        Method::POST => {
            if path == "/api/import" || path == "/api/import/resolve" {
                return Some(RequiredPermission::SuperAdmin);
            }
            if path == "/api/generate" {
                return Some(RequiredPermission::Regular(Permission::Generate));
            }
            if extract_path_client_id(path).is_some() && path.ends_with("/keys") {
                return Some(RequiredPermission::Regular(Permission::Admin));
            }
            Some(RequiredPermission::Regular(Permission::Write))
        }
        Method::PUT | Method::PATCH => Some(RequiredPermission::Regular(Permission::Write)),
        Method::DELETE => Some(RequiredPermission::Regular(Permission::Delete)),
        _ => None,
    }
}

/// Extract a client_id UUID from a path like `/api/clients/{uuid}/...`.
pub fn extract_path_client_id(path: &str) -> Option<Uuid> {
    let path = path.strip_prefix("/api/")?;
    let rest = path.strip_prefix("clients/")?;
    let client_id_str = rest.split('/').next()?;
    Uuid::parse_str(client_id_str).ok()
}

impl super::AuthenticatedKey {
    /// Check that the key has the given permission (or is super admin).
    /// Returns `Ok(())` if allowed, `Err(ProblemResponse::forbidden(...))` otherwise.
    pub fn require_permission(&self, permission: Permission) -> Result<(), ProblemResponse> {
        if self.is_super {
            return Ok(());
        }
        if self.permissions.contains(&permission) {
            return Ok(());
        }
        Err(ProblemResponse::forbidden("Insufficient permissions"))
    }

    /// Check that the key is allowed to access the given client's resources.
    /// Super admin keys can access any client. Per-client keys are scoped to their own client.
    pub fn require_client_access(&self, client_id: Uuid) -> Result<(), ProblemResponse> {
        if self.is_super {
            return Ok(());
        }
        match self.client_id {
            Some(cid) if cid == client_id => Ok(()),
            Some(_) => Err(ProblemResponse::forbidden(
                "Access denied: key is scoped to a different client",
            )),
            None => Err(ProblemResponse::forbidden(
                "Access denied: key is not associated with any client",
            )),
        }
    }

    /// Require super admin access.
    pub fn require_super_admin(&self) -> Result<(), ProblemResponse> {
        if self.is_super {
            return Ok(());
        }
        Err(ProblemResponse::forbidden("Super admin access required"))
    }
}

/// A lightweight extractor that reads the already-authenticated key from request extensions.
/// This is used inside routes after the `permission_middleware` has run.
#[derive(Debug, Clone)]
pub struct CurrentUser(pub super::AuthenticatedKey);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = ProblemResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let key = parts
            .extensions
            .get::<super::AuthenticatedKey>()
            .cloned()
            .ok_or_else(|| {
                ProblemResponse::unauthorized(
                    "Not authenticated. Missing permission middleware.",
                )
            })?;
        Ok(CurrentUser(key))
    }
}

/// Middleware that enforces the permission matrix.
/// Public routes pass through; API routes require a valid API key with matching permission.
pub(crate) async fn permission_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_owned();

    let required = match required_permission(request.method(), &path) {
        Some(r) => r,
        None => return next.run(request).await,
    };

    let api_key = match request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
    {
        Some(k) => k,
        None => {
            return ProblemResponse::unauthorized("Missing X-API-Key header").into_response();
        }
    };

    let key = match verify_api_key(api_key, &state.pool).await {
        Ok(k) => k,
        Err(e) => return e.into_response(),
    };

    match required {
        RequiredPermission::SuperAdmin => {
            if let Err(e) = key.require_super_admin() {
                return e.into_response();
            }
        }
        RequiredPermission::Regular(perm) => {
            if let Err(e) = key.require_permission(perm) {
                return e.into_response();
            }
        }
    }

    if let Some(path_client_id) = extract_path_client_id(&path) {
        if let Err(e) = key.require_client_access(path_client_id) {
            return e.into_response();
        }
    }

    request.extensions_mut().insert(key);

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- required_permission tests ---

    #[test]
    fn test_public_health_check() {
        assert_eq!(required_permission(&Method::GET, "/health"), None);
    }

    #[test]
    fn test_public_bootstrap() {
        assert_eq!(required_permission(&Method::GET, "/api/bootstrap"), None);
    }

    #[test]
    fn test_public_schema_blueprints() {
        assert_eq!(
            required_permission(&Method::GET, "/api/schema/blueprints"),
            None
        );
    }

    #[test]
    fn test_public_schema_affixes() {
        assert_eq!(
            required_permission(&Method::GET, "/api/schema/affixes"),
            None
        );
    }

    #[test]
    fn test_public_schema_generate() {
        assert_eq!(
            required_permission(&Method::GET, "/api/schema/generate"),
            None
        );
    }

    #[test]
    fn test_non_api_route() {
        assert_eq!(required_permission(&Method::GET, "/some-other-path"), None);
    }

    #[test]
    fn test_get_blueprints_requires_read() {
        assert_eq!(
            required_permission(&Method::GET, "/api/blueprints"),
            Some(RequiredPermission::Regular(Permission::Read))
        );
    }

    #[test]
    fn test_get_blueprint_by_id_requires_read() {
        assert_eq!(
            required_permission(&Method::GET, "/api/blueprints/123e4567-e89b-12d3-a456-426614174000"),
            Some(RequiredPermission::Regular(Permission::Read))
        );
    }

    #[test]
    fn test_get_affixes_requires_read() {
        assert_eq!(
            required_permission(&Method::GET, "/api/affixes"),
            Some(RequiredPermission::Regular(Permission::Read))
        );
    }

    #[test]
    fn test_get_clients_requires_read() {
        assert_eq!(
            required_permission(&Method::GET, "/api/clients"),
            Some(RequiredPermission::Regular(Permission::Read))
        );
    }

    #[test]
    fn test_post_blueprints_requires_write() {
        assert_eq!(
            required_permission(&Method::POST, "/api/blueprints"),
            Some(RequiredPermission::Regular(Permission::Write))
        );
    }

    #[test]
    fn test_put_blueprint_requires_write() {
        assert_eq!(
            required_permission(
                &Method::PUT,
                "/api/blueprints/123e4567-e89b-12d3-a456-426614174000"
            ),
            Some(RequiredPermission::Regular(Permission::Write))
        );
    }

    #[test]
    fn test_patch_blueprint_requires_write() {
        assert_eq!(
            required_permission(
                &Method::PATCH,
                "/api/blueprints/123e4567-e89b-12d3-a456-426614174000"
            ),
            Some(RequiredPermission::Regular(Permission::Write))
        );
    }

    #[test]
    fn test_post_affixes_requires_write() {
        assert_eq!(
            required_permission(&Method::POST, "/api/affixes"),
            Some(RequiredPermission::Regular(Permission::Write))
        );
    }

    #[test]
    fn test_delete_blueprint_requires_delete() {
        assert_eq!(
            required_permission(
                &Method::DELETE,
                "/api/blueprints/123e4567-e89b-12d3-a456-426614174000"
            ),
            Some(RequiredPermission::Regular(Permission::Delete))
        );
    }

    #[test]
    fn test_delete_affix_requires_delete() {
        assert_eq!(
            required_permission(
                &Method::DELETE,
                "/api/affixes/123e4567-e89b-12d3-a456-426614174000"
            ),
            Some(RequiredPermission::Regular(Permission::Delete))
        );
    }

    #[test]
    fn test_post_generate_requires_generate() {
        assert_eq!(
            required_permission(&Method::POST, "/api/generate"),
            Some(RequiredPermission::Regular(Permission::Generate))
        );
    }

    #[test]
    fn test_post_client_keys_requires_admin() {
        assert_eq!(
            required_permission(
                &Method::POST,
                "/api/clients/123e4567-e89b-12d3-a456-426614174000/keys"
            ),
            Some(RequiredPermission::Regular(Permission::Admin))
        );
    }

    #[test]
    fn test_post_client_resource_requires_write() {
        assert_eq!(
            required_permission(
                &Method::POST,
                "/api/clients/123e4567-e89b-12d3-a456-426614174000/blueprints"
            ),
            Some(RequiredPermission::Regular(Permission::Write))
        );
    }

    #[test]
    fn test_get_audit_log_requires_super_admin() {
        assert_eq!(
            required_permission(&Method::GET, "/api/audit-log"),
            Some(RequiredPermission::SuperAdmin)
        );
    }

    #[test]
    fn test_get_audit_log_sub_path_requires_super_admin() {
        assert_eq!(
            required_permission(&Method::GET, "/api/audit-log/123"),
            Some(RequiredPermission::SuperAdmin)
        );
    }

    #[test]
    fn test_options_method_not_guarded() {
        assert_eq!(required_permission(&Method::OPTIONS, "/api/blueprints"), None);
    }

    #[test]
    fn test_post_import_requires_super_admin() {
        assert_eq!(
            required_permission(&Method::POST, "/api/import"),
            Some(RequiredPermission::SuperAdmin)
        );
    }

    #[test]
    fn test_post_import_resolve_requires_super_admin() {
        assert_eq!(
            required_permission(&Method::POST, "/api/import/resolve"),
            Some(RequiredPermission::SuperAdmin)
        );
    }

    #[test]
    fn test_head_method_not_guarded() {
        assert_eq!(required_permission(&Method::HEAD, "/api/blueprints"), None);
    }

    // --- extract_path_client_id tests ---

    #[test]
    fn test_extract_client_id_from_keys_path() {
        let id = "123e4567-e89b-12d3-a456-426614174000";
        let path = format!("/api/clients/{}/keys", id);
        assert_eq!(
            extract_path_client_id(&path),
            Some(Uuid::parse_str(id).unwrap())
        );
    }

    #[test]
    fn test_extract_client_id_from_blueprints_path() {
        let id = "123e4567-e89b-12d3-a456-426614174000";
        let path = format!("/api/clients/{}/blueprints", id);
        assert_eq!(
            extract_path_client_id(&path),
            Some(Uuid::parse_str(id).unwrap())
        );
    }

    #[test]
    fn test_extract_client_id_no_match() {
        assert_eq!(extract_path_client_id("/api/blueprints"), None);
    }

    #[test]
    fn test_extract_client_id_non_api() {
        assert_eq!(extract_path_client_id("/health"), None);
    }

    #[test]
    fn test_extract_client_id_invalid_uuid() {
        assert_eq!(extract_path_client_id("/api/clients/not-a-uuid/keys"), None);
    }

    #[test]
    fn test_extract_client_id_short_path() {
        assert_eq!(extract_path_client_id("/api/clients"), None);
    }

    // --- require_permission tests ---

    fn make_key(
        client_id: Option<Uuid>,
        permissions: Vec<Permission>,
        is_super: bool,
    ) -> crate::auth::AuthenticatedKey {
        crate::auth::AuthenticatedKey {
            id: Uuid::nil(),
            name: "test".into(),
            client_id,
            permissions,
            is_super,
        }
    }

    #[test]
    fn test_super_admin_bypasses_all_permission_checks() {
        let key = make_key(None, vec![], true);
        assert!(key.require_permission(Permission::Read).is_ok());
        assert!(key.require_permission(Permission::Write).is_ok());
        assert!(key.require_permission(Permission::Delete).is_ok());
        assert!(key.require_permission(Permission::Generate).is_ok());
        assert!(key.require_permission(Permission::Admin).is_ok());
    }

    #[test]
    fn test_super_admin_bypasses_client_access() {
        let key = make_key(None, vec![], true);
        assert!(key
            .require_client_access(Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap())
            .is_ok());
    }

    #[test]
    fn test_super_admin_require_super_admin_ok() {
        let key = make_key(None, vec![], true);
        assert!(key.require_super_admin().is_ok());
    }

    #[test]
    fn test_key_with_read_can_read() {
        let key = make_key(None, vec![Permission::Read], false);
        assert!(key.require_permission(Permission::Read).is_ok());
    }

    #[test]
    fn test_key_without_read_cannot_read() {
        let key = make_key(None, vec![Permission::Write], false);
        let result = key.require_permission(Permission::Read);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_key_with_write_can_write() {
        let key = make_key(None, vec![Permission::Write], false);
        assert!(key.require_permission(Permission::Write).is_ok());
    }

    #[test]
    fn test_key_without_write_cannot_write() {
        let key = make_key(None, vec![Permission::Read], false);
        let result = key.require_permission(Permission::Write);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_key_with_delete_can_delete() {
        let key = make_key(None, vec![Permission::Delete], false);
        assert!(key.require_permission(Permission::Delete).is_ok());
    }

    #[test]
    fn test_key_without_delete_cannot_delete() {
        let key = make_key(None, vec![Permission::Read], false);
        let result = key.require_permission(Permission::Delete);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_key_with_generate_can_generate() {
        let key = make_key(None, vec![Permission::Generate], false);
        assert!(key.require_permission(Permission::Generate).is_ok());
    }

    #[test]
    fn test_key_without_generate_cannot_generate() {
        let key = make_key(None, vec![Permission::Read], false);
        let result = key.require_permission(Permission::Generate);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_key_with_admin_can_admin() {
        let key = make_key(None, vec![Permission::Admin], false);
        assert!(key.require_permission(Permission::Admin).is_ok());
    }

    #[test]
    fn test_key_without_admin_cannot_admin() {
        let key = make_key(None, vec![Permission::Read], false);
        let result = key.require_permission(Permission::Admin);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_write_can_write_and_read() {
        let key = make_key(None, vec![Permission::Write, Permission::Read], false);
        assert!(key.require_permission(Permission::Read).is_ok());
        assert!(key.require_permission(Permission::Write).is_ok());
        assert!(key.require_permission(Permission::Delete).is_err());
    }

    // --- require_client_access tests ---

    #[test]
    fn test_key_with_matching_client_id_ok() {
        let cid = Uuid::new_v4();
        let key = make_key(Some(cid), vec![Permission::Read], false);
        assert!(key.require_client_access(cid).is_ok());
    }

    #[test]
    fn test_key_with_different_client_id_forbidden() {
        let cid = Uuid::new_v4();
        let other = Uuid::new_v4();
        let key = make_key(Some(cid), vec![Permission::Read], false);
        let result = key.require_client_access(other);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_key_without_client_id_forbidden() {
        let key = make_key(None, vec![Permission::Read], false);
        let cid = Uuid::new_v4();
        let result = key.require_client_access(cid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    // --- require_super_admin tests ---

    #[test]
    fn test_non_super_admin_require_super_admin_forbidden() {
        let key = make_key(None, vec![Permission::Admin], false);
        let result = key.require_super_admin();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_forbidden_response_format() {
        let key = make_key(None, vec![Permission::Read], false);
        let err = key.require_permission(Permission::Admin).unwrap_err();
        assert_eq!(err.type_, "/errors/forbidden");
        assert_eq!(err.title, "Forbidden");
        assert_eq!(err.status, 403);
        assert_eq!(err.detail, Some("Insufficient permissions".into()));
    }

    #[test]
    fn test_current_user_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CurrentUser>();
    }

    #[test]
    fn test_current_user_debug_clone() {
        fn assert_debug_clone<T: std::fmt::Debug + Clone>() {}
        assert_debug_clone::<CurrentUser>();
    }
}

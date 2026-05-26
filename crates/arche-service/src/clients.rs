use arche_types::common::PaginatedResponse;
use arche_types::crud::{
    ApiKeyResponse, ApiKeySummary, ClientResponse, CreateApiKeyRequest, CreateApiKeyResponse,
    CreateClientRequest,
};
use arche_types::Permission;
use axum::extract::{Path, Query, State};
use axum::Json;
use sqlx::Row;
use uuid::Uuid;

use crate::audit_log::record_audit;
use crate::auth::permission::CurrentUser;
use crate::error::{FieldError, ProblemResponse};

fn permission_to_db(p: &Permission) -> String {
    match p {
        Permission::Read => "read".into(),
        Permission::Write => "write".into(),
        Permission::Delete => "delete".into(),
        Permission::Generate => "generate".into(),
        Permission::Admin => "admin".into(),
    }
}

fn db_to_permission(s: &str) -> Option<Permission> {
    match s {
        "read" => Some(Permission::Read),
        "write" => Some(Permission::Write),
        "delete" => Some(Permission::Delete),
        "generate" => Some(Permission::Generate),
        "admin" => Some(Permission::Admin),
        _ => None,
    }
}

pub async fn list_clients(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<crate::pagination::PaginationParams>,
) -> Result<Json<PaginatedResponse<ClientResponse>>, ProblemResponse> {
    user.require_super_admin()?;
    let mode = query.mode().map_err(|e| {
        ProblemResponse::validation_error(e.to_string(), vec![])
    })?;

    let rows = match &mode {
        crate::pagination::PaginationMode::Cursor { after, limit } => {
            let fetch_limit = limit + 1;
            if let &Some(cursor) = after {
                sqlx::query(
                    "SELECT id, name, created_at FROM clients WHERE id > $1 ORDER BY id ASC LIMIT $2",
                )
                .bind(cursor)
                .bind(fetch_limit)
                .fetch_all(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "clients: list cursor query failed");
                    ProblemResponse::unprocessable_entity("Failed to list clients")
                })?
            } else {
                sqlx::query(
                    "SELECT id, name, created_at FROM clients ORDER BY id ASC LIMIT $1",
                )
                .bind(fetch_limit)
                .fetch_all(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "clients: list query failed");
                    ProblemResponse::unprocessable_entity("Failed to list clients")
                })?
            }
        }
        crate::pagination::PaginationMode::Offset {
            per_page, offset, ..
        } => {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
                .fetch_one(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "clients: list count query failed");
                    ProblemResponse::unprocessable_entity("Failed to count clients")
                })?;

            let rows = sqlx::query(
                "SELECT id, name, created_at FROM clients ORDER BY id ASC LIMIT $1 OFFSET $2",
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&*state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "clients: list offset query failed");
                ProblemResponse::unprocessable_entity("Failed to list clients")
            })?;

            let client_ids: Vec<Uuid> =
                rows.iter().map(|r: &sqlx::postgres::PgRow| r.get("id")).collect();
            let client_map = fetch_api_key_summaries_batch(&state.pool, &client_ids).await?;

            let clients: Vec<ClientResponse> = rows
                .iter()
                .map(|r| {
                    let id: Uuid = r.get("id");
                    ClientResponse {
                        id,
                        name: r.get("name"),
                        created_at: r.get("created_at"),
                        api_keys: client_map.get(&id).cloned().unwrap_or_default(),
                    }
                })
                .collect();

            return Ok(Json(crate::pagination::paginate_offset(clients, total)));
        }
    };

    let client_ids: Vec<Uuid> =
        rows.iter().map(|r: &sqlx::postgres::PgRow| r.get("id")).collect();
    let client_map = fetch_api_key_summaries_batch(&state.pool, &client_ids).await?;

    let clients: Vec<ClientResponse> = rows
        .iter()
        .map(|r| {
            let id: Uuid = r.get("id");
            ClientResponse {
                id,
                name: r.get("name"),
                created_at: r.get("created_at"),
                api_keys: client_map.get(&id).cloned().unwrap_or_default(),
            }
        })
        .collect();

    if let crate::pagination::PaginationMode::Cursor { limit, .. } = mode {
        Ok(Json(crate::pagination::paginate_cursor(clients, limit)))
    } else {
        unreachable!("offset handled above")
    }
}

pub async fn get_client(
    State(state): State<crate::AppState>,
    CurrentUser(_user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ClientResponse>, ProblemResponse> {
    let row = sqlx::query("SELECT id, name, created_at FROM clients WHERE id = $1")
        .bind(id)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "clients: get query failed");
            ProblemResponse::unprocessable_entity("Failed to get client")
        })?
        .ok_or_else(|| ProblemResponse::not_found(format!("Client not found: {id}")))?;

    let api_keys = fetch_api_key_summaries(&state.pool, id).await?;

    Ok(Json(ClientResponse {
        id: row.get("id"),
        name: row.get("name"),
        created_at: row.get("created_at"),
        api_keys,
    }))
}

fn validate_client_name(name: &str) -> Result<(), ProblemResponse> {
    if name.is_empty() {
        return Err(ProblemResponse::validation_error(
            "Client name must not be empty",
            vec![FieldError {
                path: "name".into(),
                message: "must not be empty".into(),
            }],
        ));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(ProblemResponse::validation_error(
            format!(
                "Invalid client name '{}': must contain only alphanumeric characters and underscores (a-z, A-Z, 0-9, _)",
                name
            ),
            vec![FieldError {
                path: "name".into(),
                message: "must contain only [a-zA-Z0-9_]".into(),
            }],
        ));
    }
    Ok(())
}

pub async fn create_client(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreateClientRequest>,
) -> Result<Json<ClientResponse>, ProblemResponse> {
    user.require_super_admin()?;
    validate_client_name(&req.name)?;

    let row = sqlx::query(
        "INSERT INTO clients (name) VALUES ($1) RETURNING id, name, created_at",
    )
    .bind(&req.name)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.code().is_some_and(|c| c == "23505") {
                return ProblemResponse::conflict(format!(
                    "Client name '{}' already exists",
                    req.name
                ));
            }
        }
        tracing::error!(error = %e, "clients: create failed");
        ProblemResponse::validation_error(
            format!("Failed to create client: {e}"),
            vec![],
        )
    })?;

    let client_id: Uuid = row.get("id");
    let client_name: String = row.get("name");

    if let Err(e) = record_audit(
        &*state.pool,
        &user,
        Some(client_id),
        "client",
        client_id,
        "created",
        None,
        Some(serde_json::json!({ "id": client_id, "name": &client_name })),
    )
    .await
    {
        tracing::error!(error = %e, "clients: audit log insert failed");
    }

    Ok(Json(ClientResponse {
        id: client_id,
        name: client_name,
        created_at: row.get("created_at"),
        api_keys: vec![],
    }))
}

pub async fn delete_client(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ProblemResponse> {
    user.require_super_admin()?;
    let row = sqlx::query("SELECT id, name FROM clients WHERE id = $1")
        .bind(id)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "clients: check existence failed");
            ProblemResponse::unprocessable_entity("Failed to check client existence")
        })?
        .ok_or_else(|| ProblemResponse::not_found(format!("Client not found: {id}")))?;

    let client_name: String = row.get("name");

    let key_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_keys WHERE client_id = $1 AND is_super = false",
    )
    .bind(id)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "clients: key count query failed");
        ProblemResponse::unprocessable_entity("Failed to check active keys")
    })?;

    if key_count > 0 {
        return Err(ProblemResponse::delete_referenced_resource(format!(
            "Client {id} has {key_count} active API key(s). Revoke all keys before deleting the client."
        )));
    }

    let before_snapshot = serde_json::json!({ "id": id, "name": &client_name });

    sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "clients: delete failed");
            ProblemResponse::unprocessable_entity("Failed to delete client")
        })?;

    if let Err(e) = record_audit(
        &*state.pool,
        &user,
        Some(id),
        "client",
        id,
        "deleted",
        Some(before_snapshot),
        None,
    )
    .await
    {
        tracing::error!(error = %e, "clients: audit log insert failed");
    }

    Ok(Json(serde_json::json!({"deleted": true})))
}

pub async fn list_api_keys(
    State(state): State<crate::AppState>,
    CurrentUser(_user): CurrentUser,
    Path(client_id): Path<Uuid>,
) -> Result<Json<Vec<ApiKeyResponse>>, ProblemResponse> {
    let client_exists = sqlx::query("SELECT id FROM clients WHERE id = $1")
        .bind(client_id)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "api-keys: client check failed");
            ProblemResponse::unprocessable_entity("Failed to check client existence")
        })?
        .is_some();

    if !client_exists {
        return Err(ProblemResponse::not_found(format!(
            "Client not found: {client_id}"
        )));
    }

    let rows = sqlx::query(
        "SELECT id, name, permissions, created_at, expires_at FROM api_keys \
         WHERE client_id = $1 AND is_super = false ORDER BY created_at DESC",
    )
    .bind(client_id)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "api-keys: list query failed");
        ProblemResponse::unprocessable_entity("Failed to list API keys")
    })?;

    let keys: Vec<ApiKeyResponse> = rows
        .iter()
        .map(|r| {
            let perm_strs: Vec<String> = r.get("permissions");
            let permissions: Vec<Permission> =
                perm_strs.iter().filter_map(|s| db_to_permission(s)).collect();
            ApiKeyResponse {
                id: r.get("id"),
                name: r.get("name"),
                permissions,
                created_at: r.get("created_at"),
                expires_at: r.get("expires_at"),
            }
        })
        .collect();

    Ok(Json(keys))
}

pub async fn create_api_key(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(client_id): Path<Uuid>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, ProblemResponse> {
    let client_exists = sqlx::query("SELECT id FROM clients WHERE id = $1")
        .bind(client_id)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "api-keys: client check failed");
            ProblemResponse::unprocessable_entity("Failed to check client existence")
        })?
        .is_some();

    if !client_exists {
        return Err(ProblemResponse::not_found(format!(
            "Client not found: {client_id}"
        )));
    }

    let raw_key = generate_api_key();
    let key_hash =
        bcrypt::hash(&raw_key, bcrypt::DEFAULT_COST).map_err(|e| {
            tracing::error!(error = %e, "api-keys: hash failed");
            ProblemResponse::unprocessable_entity("Failed to hash API key")
        })?;

    let perm_strs: Vec<String> = req.permissions.iter().map(permission_to_db).collect();

    let row = sqlx::query(
        "INSERT INTO api_keys (client_id, name, key_hash, permissions, is_super) \
         VALUES ($1, $2, $3, $4, false) RETURNING id, created_at",
    )
    .bind(client_id)
    .bind(&req.name)
    .bind(&key_hash)
    .bind(&perm_strs)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "api-keys: create failed");
        ProblemResponse::validation_error(
            format!("Failed to create API key: {e}"),
            vec![],
        )
    })?;

    let key_id: Uuid = row.get("id");

    if let Err(e) = record_audit(
        &*state.pool,
        &user,
        Some(client_id),
        "api_key",
        key_id,
        "created",
        None,
        Some(serde_json::json!({
            "id": key_id,
            "client_id": client_id,
            "name": &req.name,
            "permissions": &perm_strs,
        })),
    )
    .await
    {
        tracing::error!(error = %e, "api-keys: audit log insert failed");
    }

    Ok(Json(CreateApiKeyResponse {
        id: key_id,
        name: req.name,
        permissions: req.permissions,
        key: raw_key,
    }))
}

pub async fn delete_api_key(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path((client_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ProblemResponse> {
    let row = sqlx::query(
        "SELECT id, name, permissions FROM api_keys WHERE id = $1 AND client_id = $2 AND is_super = false",
    )
    .bind(key_id)
    .bind(client_id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "api-keys: existence check failed");
        ProblemResponse::unprocessable_entity("Failed to check API key existence")
    })?
    .ok_or_else(|| ProblemResponse::not_found(format!("API key not found: {key_id}")))?;

    let key_name: String = row.get("name");
    let perm_strs: Vec<String> = row.get("permissions");

    let before_snapshot = serde_json::json!({
        "id": key_id,
        "client_id": client_id,
        "name": &key_name,
        "permissions": &perm_strs,
    });

    sqlx::query("DELETE FROM api_keys WHERE id = $1 AND is_super = false")
        .bind(key_id)
        .execute(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "api-keys: delete failed");
            ProblemResponse::unprocessable_entity("Failed to delete API key")
        })?;

    if let Err(e) = record_audit(
        &*state.pool,
        &user,
        Some(client_id),
        "api_key",
        key_id,
        "deleted",
        Some(before_snapshot),
        None,
    )
    .await
    {
        tracing::error!(error = %e, "api-keys: audit log insert failed");
    }

    Ok(Json(serde_json::json!({"deleted": true})))
}

fn generate_api_key() -> String {
    use rand::Rng;
    let random_part: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();
    format!("arche_k_{}", random_part)
}

async fn fetch_api_key_summaries(
    pool: &sqlx::PgPool,
    client_id: Uuid,
) -> Result<Vec<ApiKeySummary>, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT id, name, permissions, created_at FROM api_keys \
         WHERE client_id = $1 AND is_super = false ORDER BY created_at DESC",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "clients: fetch api key summaries failed");
        ProblemResponse::unprocessable_entity("Failed to fetch API key summaries")
    })?;

    let keys = rows
        .iter()
        .map(|r| {
            let perm_strs: Vec<String> = r.get("permissions");
            let permissions: Vec<Permission> =
                perm_strs.iter().filter_map(|s| db_to_permission(s)).collect();
            ApiKeySummary {
                id: r.get("id"),
                name: r.get("name"),
                permissions,
                created_at: r.get("created_at"),
            }
        })
        .collect();

    Ok(keys)
}

async fn fetch_api_key_summaries_batch(
    pool: &sqlx::PgPool,
    client_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<ApiKeySummary>>, ProblemResponse> {
    if client_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let rows = sqlx::query(
        "SELECT id, client_id, name, permissions, created_at FROM api_keys \
         WHERE client_id = ANY($1) AND is_super = false ORDER BY created_at DESC",
    )
    .bind(client_ids)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "clients: batch fetch api key summaries failed");
        ProblemResponse::unprocessable_entity("Failed to fetch API key summaries")
    })?;

    let mut map: std::collections::HashMap<Uuid, Vec<ApiKeySummary>> =
        std::collections::HashMap::new();

    for row in &rows {
        let cid: Uuid = row.get("client_id");
        let perm_strs: Vec<String> = row.get("permissions");
        let permissions: Vec<Permission> =
            perm_strs.iter().filter_map(|s| db_to_permission(s)).collect();
        let summary = ApiKeySummary {
            id: row.get("id"),
            name: row.get("name"),
            permissions,
            created_at: row.get("created_at"),
        };
        map.entry(cid).or_default().push(summary);
    }

    Ok(map)
}

impl crate::pagination::HasId for ClientResponse {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_to_db_all() {
        assert_eq!(permission_to_db(&Permission::Read), "read");
        assert_eq!(permission_to_db(&Permission::Write), "write");
        assert_eq!(permission_to_db(&Permission::Delete), "delete");
        assert_eq!(permission_to_db(&Permission::Generate), "generate");
        assert_eq!(permission_to_db(&Permission::Admin), "admin");
    }

    #[test]
    fn test_db_to_permission_all() {
        assert_eq!(db_to_permission("read"), Some(Permission::Read));
        assert_eq!(db_to_permission("write"), Some(Permission::Write));
        assert_eq!(db_to_permission("delete"), Some(Permission::Delete));
        assert_eq!(db_to_permission("generate"), Some(Permission::Generate));
        assert_eq!(db_to_permission("admin"), Some(Permission::Admin));
    }

    #[test]
    fn test_db_to_permission_unknown() {
        assert_eq!(db_to_permission("super"), None);
        assert_eq!(db_to_permission(""), None);
    }

    #[test]
    fn test_permission_round_trip() {
        let perms = vec![
            Permission::Read,
            Permission::Write,
            Permission::Delete,
            Permission::Generate,
            Permission::Admin,
        ];
        let strings: Vec<String> = perms.iter().map(permission_to_db).collect();
        let back: Vec<Permission> =
            strings.iter().filter_map(|s| db_to_permission(s)).collect();
        assert_eq!(perms, back);
    }

    #[test]
    fn test_generate_api_key_prefix() {
        let key = generate_api_key();
        assert!(
            key.starts_with("arche_k_"),
            "key should start with 'arche_k_', got: {}",
            key
        );
    }

    #[test]
    fn test_generate_api_key_length() {
        let key = generate_api_key();
        assert_eq!(key.len(), 56);
    }

    #[test]
    fn test_generate_api_key_randomness() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_client_response_has_id_impl() {
        let id = Uuid::new_v4();
        let resp = ClientResponse {
            id,
            name: "test".into(),
            created_at: chrono::Utc::now(),
            api_keys: vec![],
        };
        assert_eq!(resp.id, id);
    }

    #[test]
    fn test_validate_client_name_valid() {
        assert!(validate_client_name("My_Game").is_ok());
        assert!(validate_client_name("test_client_1").is_ok());
        assert!(validate_client_name("ABC").is_ok());
        assert!(validate_client_name("a").is_ok());
    }

    #[test]
    fn test_validate_client_name_empty() {
        let err = validate_client_name("").unwrap_err();
        assert_eq!(err.status, 400);
        assert_eq!(err.type_, "/errors/validation-error");
    }

    #[test]
    fn test_validate_client_name_invalid_chars() {
        for name in &["hello world", "test-name", "name@domain", "hello.there"] {
            let err = validate_client_name(name).unwrap_err();
            assert_eq!(err.status, 400);
            assert_eq!(err.type_, "/errors/validation-error");
        }
    }

    mod db_tests {
        use super::*;
        use crate::AppState;
        use arche_types::Permission;
        use sqlx::PgPool;
        use std::collections::HashMap;
        use std::sync::Arc;
        use uuid::Uuid;

        async fn ensure_db() -> Option<PgPool> {
            let url = std::env::var("DATABASE_URL").ok()?;
            let pool = PgPool::connect(&url).await.ok()?;
            sqlx::migrate!("../../arche-service/migrations")
                .run(&pool)
                .await
                .ok()?;
            Some(pool)
        }

        async fn create_test_client(pool: &PgPool, name: &str) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO clients (name) VALUES ($1) RETURNING id",
            )
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("Failed to create test client");
            row.get("id")
        }

        fn test_state(pool: Arc<PgPool>) -> AppState {
            AppState {
                cache: Arc::new(tokio::sync::RwLock::new(crate::cache::Cache::new(
                    HashMap::new(), HashMap::new(), HashMap::new(),
                    HashMap::new(), HashMap::new(),
                ))),
                pool,
                redis: None,
                import_staging: Arc::new(crate::import::ImportStaging::new()),
            }
        }

        fn super_user() -> CurrentUser {
            CurrentUser(crate::auth::AuthenticatedKey {
                id: Uuid::nil(),
                name: "super".into(),
                client_id: None,
                permissions: vec![],
                is_super: true,
            })
        }

        fn test_user(client_id: Uuid, permission: Permission) -> CurrentUser {
            CurrentUser(crate::auth::AuthenticatedKey {
                id: Uuid::nil(),
                name: "test".into(),
                client_id: Some(client_id),
                permissions: vec![permission],
                is_super: false,
            })
        }

        fn test_user_with_permissions(client_id: Uuid, permissions: Vec<Permission>) -> CurrentUser {
            CurrentUser(crate::auth::AuthenticatedKey {
                id: Uuid::nil(),
                name: "test".into(),
                client_id: Some(client_id),
                permissions,
                is_super: false,
            })
        }

        // --- Acceptance: Create client with valid name returns 201 with client object ---

        #[tokio::test]
        async fn test_create_client_valid_name_returns_201() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let req = CreateClientRequest {
                name: "My_Game".into(),
            };

            let result = create_client(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            let resp = result.expect("create client should succeed");
            assert_eq!(resp.name, "My_Game");
            assert!(!resp.id.is_nil());
            assert!(resp.api_keys.is_empty());

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(resp.id)
                .execute(&pool)
                .await
                .unwrap();
        }

        // --- Acceptance: Create client with duplicate name returns 409 ---

        #[tokio::test]
        async fn test_create_client_duplicate_name_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "DuplicateGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let req = CreateClientRequest {
                name: "DuplicateGame".into(),
            };

            let result = create_client(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert_eq!(err.type_, "/errors/conflict");
            assert!(err.detail.unwrap().contains("already exists"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        // --- Acceptance: Create client with invalid name characters returns 400 ---

        #[tokio::test]
        async fn test_create_client_invalid_name_empty() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let req = CreateClientRequest {
                name: "".into(),
            };

            let result = create_client(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert_eq!(err.type_, "/errors/validation-error");
        }

        #[tokio::test]
        async fn test_create_client_invalid_name_special_chars() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let req = CreateClientRequest {
                name: "hello world".into(),
            };

            let result = create_client(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert_eq!(err.type_, "/errors/validation-error");
        }

        // --- Acceptance: List clients returns all clients (super admin only) ---

        #[tokio::test]
        async fn test_list_clients_super_admin_returns_all() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool, "ClientAlpha").await;
            let client_b = create_test_client(&pool, "ClientBeta").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let query = crate::pagination::PaginationParams {
                cursor: None,
                limit: Some(50),
                page: None,
                per_page: None,
            };

            let result = list_clients(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            let resp = result.unwrap();
            let ids: Vec<Uuid> = resp.data.iter().map(|c| c.id).collect();
            assert!(ids.contains(&client_a));
            assert!(ids.contains(&client_b));

            sqlx::query("DELETE FROM clients WHERE id = ANY($1)")
                .bind(&vec![client_a, client_b])
                .execute(&pool)
                .await
                .unwrap();
        }

        // --- Acceptance: Delete client with no active keys succeeds (cascade deletes all data) ---

        #[tokio::test]
        async fn test_delete_client_no_active_keys_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "DeletableGame").await;

            // Create some child data to verify cascade
            sqlx::query(
                "INSERT INTO blueprints (client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1, 'sword', 'sword', 1.0, '{}'::jsonb, ARRAY[]::text[], 0, 0, 0, 0)",
            )
            .bind(client_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = delete_client(
                axum::extract::State(state),
                user,
                axum::extract::Path(client_id),
            )
            .await;

            let resp = result.unwrap();
            assert_eq!(resp["deleted"], true);

            // Verify cascade: client should be gone
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE id = $1")
                .bind(client_id)
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(count, 0);

            // Blueprints should also be cascade-deleted
            let bp_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blueprints WHERE client_id = $1")
                .bind(client_id)
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(bp_count, 0);
        }

        // --- Acceptance: Delete client with active keys returns 409 with key count in error detail ---

        #[tokio::test]
        async fn test_delete_client_with_active_keys_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "ProtectedGame").await;

            // Insert an active (non-super) API key
            sqlx::query(
                "INSERT INTO api_keys (client_id, name, key_hash, permissions, is_super) \
                 VALUES ($1, 'active-key', 'hashed', ARRAY['read'], false)",
            )
            .bind(client_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = delete_client(
                axum::extract::State(state),
                user,
                axum::extract::Path(client_id),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert_eq!(err.type_, "/errors/delete-referenced-resource");
            let detail = err.detail.unwrap();
            assert!(detail.contains("1 active API key"));
            assert!(detail.contains(&client_id.to_string()));

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_client_with_multiple_active_keys_returns_409_with_count() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "MultiKeyGame").await;

            // Insert multiple active API keys
            for i in 1..=3 {
                sqlx::query(
                    "INSERT INTO api_keys (client_id, name, key_hash, permissions, is_super) \
                     VALUES ($1, $2, 'hashed', ARRAY['read'], false)",
                )
                .bind(client_id)
                .bind(format!("key-{}", i))
                .execute(&pool)
                .await
                .unwrap();
            }

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = delete_client(
                axum::extract::State(state),
                user,
                axum::extract::Path(client_id),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            let detail = err.detail.unwrap();
            assert!(detail.contains("3 active API key"));

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_client_not_found() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();
            let nonexistent_id = Uuid::new_v4();

            let result = delete_client(
                axum::extract::State(state),
                user,
                axum::extract::Path(nonexistent_id),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
        }

        // --- Acceptance: Non-super-admin key gets 403 on all client endpoints ---

        #[tokio::test]
        async fn test_create_client_non_super_returns_403() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "SomeGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateClientRequest {
                name: "NewGame".into(),
            };

            let result = create_client(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 403);

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_list_clients_non_super_returns_403() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "ReadOnlyGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Read);

            let query = crate::pagination::PaginationParams {
                cursor: None,
                limit: Some(50),
                page: None,
                per_page: None,
            };

            let result = list_clients(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 403);

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_client_non_super_returns_403() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "DeleteForbiddenGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_client(
                axum::extract::State(state),
                user,
                axum::extract::Path(client_id),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 403);

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        // --- API Key CRUD ---

        #[tokio::test]
        async fn test_create_api_key_returns_raw_key_and_stores_hash() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "KeyTestGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user_with_permissions(client_id, vec![Permission::Admin]);

            let req = CreateApiKeyRequest {
                name: "game-server-key".into(),
                permissions: vec![Permission::Read, Permission::Generate],
            };

            let result = create_api_key(
                axum::extract::State(state),
                user.clone(),
                axum::extract::Path(client_id),
                axum::Json(req),
            )
            .await;

            let resp = result.expect("create api key should succeed");
            assert_eq!(resp.name, "game-server-key");
            assert!(!resp.id.is_nil());
            assert_eq!(resp.permissions.len(), 2);
            assert!(resp.permissions.contains(&Permission::Read));
            assert!(resp.permissions.contains(&Permission::Generate));
            assert!(resp.key.starts_with("arche_k_"));
            assert!(!resp.key.is_empty());

            // Verify the key hash in DB is not the raw key
            let row = sqlx::query("SELECT key_hash FROM api_keys WHERE id = $1")
                .bind(resp.id)
                .fetch_one(&pool)
                .await
                .unwrap();
            let key_hash: String = row.get("key_hash");
            assert_ne!(key_hash, resp.key);
            // Verify bcrypt verify works with the raw key
            assert!(bcrypt::verify(&resp.key, &key_hash).unwrap());

            // The response should NOT include the key_hash
            let resp_json = serde_json::to_value(&*resp).unwrap();
            assert!(!resp_json.as_object().unwrap().contains_key("keyHash"));

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_list_api_keys_returns_metadata_only() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "ListKeyGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user_with_permissions(client_id, vec![Permission::Admin, Permission::Read]);

            // Create a key first
            let req = CreateApiKeyRequest {
                name: "my-key".into(),
                permissions: vec![Permission::Read],
            };
            let create_resp = create_api_key(
                axum::extract::State(test_state(Arc::new(pool.clone()))),
                user.clone(),
                axum::extract::Path(client_id),
                axum::Json(req),
            )
            .await
            .unwrap();

            // Now list keys
            let list_result = list_api_keys(
                axum::extract::State(state),
                user,
                axum::extract::Path(client_id),
            )
            .await;

            let keys = list_result.unwrap();
            assert_eq!(keys.len(), 1);
            assert_eq!(keys[0].id, create_resp.id);
            assert_eq!(keys[0].name, "my-key");
            assert_eq!(keys[0].permissions, vec![Permission::Read]);

            // The list response must NOT contain raw key or hash
            let key_json = serde_json::to_value(&keys[0]).unwrap();
            let obj = key_json.as_object().unwrap();
            assert!(obj.contains_key("id"));
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("permissions"));
            assert!(obj.contains_key("createdAt"));
            assert!(!obj.contains_key("key"), "list should not expose raw key");
            assert!(!obj.contains_key("keyHash"), "list should not expose key hash");

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_api_key_succeeds_and_key_stops_working() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "DeleteKeyGame").await;
            let user = test_user_with_permissions(client_id, vec![Permission::Admin]);

            // Create a key
            let req = CreateApiKeyRequest {
                name: "revocable-key".into(),
                permissions: vec![Permission::Read],
            };
            let create_resp = create_api_key(
                axum::extract::State(test_state(Arc::new(pool.clone()))),
                user.clone(),
                axum::extract::Path(client_id),
                axum::Json(req),
            )
            .await
            .unwrap();

            let raw_key = create_resp.key.clone();

            // Verify the key authenticates
            let auth_result = crate::auth::verify_api_key(&raw_key, &pool).await;
            assert!(auth_result.is_ok(), "key should authenticate before deletion");

            // Delete the key
            let state = test_state(Arc::new(pool.clone()));
            let delete_result = delete_api_key(
                axum::extract::State(state),
                user,
                axum::extract::Path((client_id, create_resp.id)),
            )
            .await;

            let resp = delete_result.unwrap();
            assert_eq!(resp["deleted"], true);

            // Verify the key no longer authenticates
            let auth_result = crate::auth::verify_api_key(&raw_key, &pool).await;
            assert!(auth_result.is_err(), "key should NOT authenticate after deletion");

            // Cleanup
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_api_key_nonexistent_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "NoKeyGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user_with_permissions(client_id, vec![Permission::Admin]);
            let nonexistent_key_id = Uuid::new_v4();

            let result = delete_api_key(
                axum::extract::State(state),
                user,
                axum::extract::Path((client_id, nonexistent_key_id)),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains(&nonexistent_key_id.to_string()));

            // Cleanup
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_api_key_wrong_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool, "ClientA").await;
            let client_b = create_test_client(&pool, "ClientB").await;

            // Create a key for client_a
            let req = CreateApiKeyRequest {
                name: "key-a".into(),
                permissions: vec![Permission::Read],
            };
            let user_a = test_user_with_permissions(client_a, vec![Permission::Admin]);
            let create_resp = create_api_key(
                axum::extract::State(test_state(Arc::new(pool.clone()))),
                user_a.clone(),
                axum::extract::Path(client_a),
                axum::Json(req),
            )
            .await
            .unwrap();

            // Try to delete the key using client_b's path
            let state = test_state(Arc::new(pool.clone()));
            let user_b = test_user_with_permissions(client_b, vec![Permission::Admin]);
            let result = delete_api_key(
                axum::extract::State(state),
                user_b,
                axum::extract::Path((client_b, create_resp.id)),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains(&create_resp.id.to_string()));

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = $1")
                .bind(client_a)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = ANY($1)")
                .bind(&vec![client_a, client_b])
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_api_key_nonexistent_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let nonexistent_client_id = Uuid::new_v4();
            // Use super user to bypass client-scope checks
            let user = super_user();

            let req = CreateApiKeyRequest {
                name: "key".into(),
                permissions: vec![Permission::Read],
            };

            let result = create_api_key(
                axum::extract::State(state),
                user,
                axum::extract::Path(nonexistent_client_id),
                axum::Json(req),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains(&nonexistent_client_id.to_string()));
        }

        #[tokio::test]
        async fn test_list_api_keys_nonexistent_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let nonexistent_client_id = Uuid::new_v4();
            let user = super_user();

            let result = list_api_keys(
                axum::extract::State(state),
                user,
                axum::extract::Path(nonexistent_client_id),
            )
            .await;

            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
        }

        #[tokio::test]
        async fn test_create_api_key_key_prefix_format() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "PrefixGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user_with_permissions(client_id, vec![Permission::Admin]);

            // Create multiple keys and verify prefix
            for i in 1..=3 {
                let req = CreateApiKeyRequest {
                    name: format!("key-{}", i),
                    permissions: vec![Permission::Read],
                };
                let result = create_api_key(
                    axum::extract::State(test_state(Arc::new(pool.clone()))),
                    user.clone(),
                    axum::extract::Path(client_id),
                    axum::Json(req),
                )
                .await;

                let resp = result.unwrap();
                assert!(resp.key.starts_with("arche_k_"), "key {i} missing prefix");
                assert_eq!(resp.key.len(), 56, "key {i} wrong length");
            }

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_api_key_invalid_permission_returns_error() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "InvalidPermGame").await;

            // Send invalid permission string as raw JSON to test deserialization error
            let raw_json = serde_json::json!({
                "name": "bad-key",
                "permissions": ["read", "super_admin", "write"]
            });

            use axum::body::Body;
            use axum::http::Request;
            use axum::routing::post;
            use axum::Router;
            use tower::ServiceExt;

            async fn create_key_handler(
                State(state): State<AppState>,
                Path(client_id): Path<Uuid>,
                Json(req): Json<CreateApiKeyRequest>,
            ) -> Result<Json<CreateApiKeyResponse>, ProblemResponse> {
                create_api_key(
                    axum::extract::State(state),
                    CurrentUser(crate::auth::AuthenticatedKey {
                        id: Uuid::nil(),
                        name: "super".into(),
                        client_id: None,
                        permissions: vec![],
                        is_super: true,
                    }),
                    axum::extract::Path(client_id),
                    axum::Json(req),
                )
                .await
            }

            let app = Router::new()
                .route("/api/clients/:client_id/keys", post(create_key_handler))
                .with_state(test_state(Arc::new(pool.clone())));

            let resp = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/api/clients/{client_id}/keys"))
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&raw_json).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();

            let status = resp.status();
            assert!(status.is_client_error(), "expected client error, got {status}");
            // Axum's Json extractor returns 422 for deserialization errors;
            // either 400 or 422 is acceptable as a validation error
            assert!(
                status.as_u16() == 400 || status.as_u16() == 422,
                "expected 400 or 422, got {status}"
            );

            // Cleanup
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_api_key_key_scoped_to_client() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool, "ScopeA").await;
            let client_b = create_test_client(&pool, "ScopeB").await;

            // Create a key for client_a
            let req = CreateApiKeyRequest {
                name: "scoped-key".into(),
                permissions: vec![Permission::Read],
            };
            let user_a = test_user_with_permissions(client_a, vec![Permission::Admin]);
            let create_resp = create_api_key(
                axum::extract::State(test_state(Arc::new(pool.clone()))),
                user_a,
                axum::extract::Path(client_a),
                axum::Json(req),
            )
            .await
            .unwrap();

            // Verify the key authenticates and has the correct client_id
            let auth_key = crate::auth::verify_api_key(&create_resp.key, &pool)
                .await
                .unwrap();
            assert_eq!(auth_key.client_id, Some(client_a));
            assert!(!auth_key.is_super);
            assert_eq!(auth_key.name, "scoped-key");

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = ANY($1)")
                .bind(&vec![client_a, client_b])
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = ANY($1)")
                .bind(&vec![client_a, client_b])
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_api_key_empty_permissions_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool, "EmptyPermGame").await;
            let state = test_state(Arc::new(pool.clone()));
            let user = test_user_with_permissions(client_id, vec![Permission::Admin]);

            let req = CreateApiKeyRequest {
                name: "no-perm-key".into(),
                permissions: vec![],
            };

            let result = create_api_key(
                axum::extract::State(state),
                user,
                axum::extract::Path(client_id),
                axum::Json(req),
            )
            .await;

            let resp = result.unwrap();
            assert!(resp.permissions.is_empty());
            assert!(resp.key.starts_with("arche_k_"));

            // Cleanup
            sqlx::query("DELETE FROM api_keys WHERE client_id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }
    }
}

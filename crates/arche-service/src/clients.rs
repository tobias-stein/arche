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

use crate::auth::permission::CurrentUser;
use crate::error::ProblemResponse;

fn permission_to_db(p: &Permission) -> &'static str {
    match p {
        Permission::Read => "read",
        Permission::Write => "write",
        Permission::Delete => "delete",
        Permission::Generate => "generate",
        Permission::Admin => "admin",
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

fn resolve_client_id_for_clients(
    user: &crate::auth::AuthenticatedKey,
) -> Result<Option<Uuid>, ProblemResponse> {
    if user.is_super {
        Ok(None)
    } else {
        user.client_id.map(Some).ok_or_else(|| {
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })
    }
}

pub async fn list_clients(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<crate::pagination::PaginationParams>,
) -> Result<Json<PaginatedResponse<ClientResponse>>, ProblemResponse> {
    let client_id_filter = resolve_client_id_for_clients(&user)?;

    if let Some(cid) = client_id_filter {
        let rows = sqlx::query("SELECT id, name, created_at FROM clients WHERE id = $1")
            .bind(cid)
            .fetch_all(&*state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "clients: list query failed");
                ProblemResponse::unprocessable_entity("Failed to list clients")
            })?;

        let client_ids: Vec<Uuid> = rows.iter().map(|r: &sqlx::postgres::PgRow| r.get("id")).collect();
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

        let total = clients.len() as i64;
        return Ok(Json(crate::pagination::paginate_offset(clients, total)));
    }

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

pub async fn create_client(
    State(state): State<crate::AppState>,
    CurrentUser(_user): CurrentUser,
    Json(req): Json<CreateClientRequest>,
) -> Result<Json<ClientResponse>, ProblemResponse> {
    let row = sqlx::query(
        "INSERT INTO clients (name) VALUES ($1) RETURNING id, name, created_at",
    )
    .bind(&req.name)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "clients: create failed");
        ProblemResponse::validation_error(
            format!("Failed to create client: {e}"),
            vec![],
        )
    })?;

    Ok(Json(ClientResponse {
        id: row.get("id"),
        name: row.get("name"),
        created_at: row.get("created_at"),
        api_keys: vec![],
    }))
}

pub async fn delete_client(
    State(state): State<crate::AppState>,
    CurrentUser(_user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ProblemResponse> {
    let exists = sqlx::query("SELECT id FROM clients WHERE id = $1")
        .bind(id)
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "clients: check existence failed");
            ProblemResponse::unprocessable_entity("Failed to check client existence")
        })?
        .is_some();

    if !exists {
        return Err(ProblemResponse::not_found(format!(
            "Client not found: {id}"
        )));
    }

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

    sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "clients: delete failed");
            ProblemResponse::unprocessable_entity("Failed to delete client")
        })?;

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
        "SELECT id, name, permissions, created_at FROM api_keys \
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
            }
        })
        .collect();

    Ok(Json(keys))
}

pub async fn create_api_key(
    State(state): State<crate::AppState>,
    CurrentUser(_user): CurrentUser,
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

    let perm_strs: Vec<String> = req.permissions.iter().map(|p| permission_to_db(p).to_string()).collect();

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

    let id: Uuid = row.get("id");

    Ok(Json(CreateApiKeyResponse {
        id,
        name: req.name,
        permissions: req.permissions,
        key: raw_key,
    }))
}

pub async fn delete_api_key(
    State(state): State<crate::AppState>,
    CurrentUser(_user): CurrentUser,
    Path((client_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ProblemResponse> {
    let exists = sqlx::query(
        "SELECT id FROM api_keys WHERE id = $1 AND client_id = $2 AND is_super = false",
    )
    .bind(key_id)
    .bind(client_id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "api-keys: existence check failed");
        ProblemResponse::unprocessable_entity("Failed to check API key existence")
    })?
    .is_some();

    if !exists {
        return Err(ProblemResponse::not_found(format!(
            "API key not found: {key_id}"
        )));
    }

    sqlx::query("DELETE FROM api_keys WHERE id = $1 AND is_super = false")
        .bind(key_id)
        .execute(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "api-keys: delete failed");
            ProblemResponse::unprocessable_entity("Failed to delete API key")
        })?;

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
    use crate::pagination::HasId;

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
        let strings: Vec<&str> = perms.iter().map(permission_to_db).collect();
        let back: Vec<Permission> =
            strings.iter().filter_map(|s| db_to_permission(s)).collect();
        assert_eq!(perms, back);
    }

    #[test]
    fn test_resolve_client_id_super_admin() {
        let key = crate::auth::AuthenticatedKey {
            id: Uuid::nil(),
            name: "test".into(),
            client_id: None,
            permissions: vec![Permission::Read],
            is_super: true,
        };
        let result = resolve_client_id_for_clients(&key).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_client_id_regular_key() {
        let cid = Uuid::new_v4();
        let key = crate::auth::AuthenticatedKey {
            id: Uuid::nil(),
            name: "test".into(),
            client_id: Some(cid),
            permissions: vec![Permission::Read],
            is_super: false,
        };
        let result = resolve_client_id_for_clients(&key).unwrap();
        assert_eq!(result, Some(cid));
    }

    #[test]
    fn test_resolve_client_id_regular_key_no_client() {
        let key = crate::auth::AuthenticatedKey {
            id: Uuid::nil(),
            name: "test".into(),
            client_id: None,
            permissions: vec![Permission::Read],
            is_super: false,
        };
        let result = resolve_client_id_for_clients(&key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
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
        assert_eq!(resp.id(), id);
    }
}

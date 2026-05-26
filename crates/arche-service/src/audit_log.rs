use arche_types::common::{AuditLogListQuery, PaginatedResponse};
use arche_types::AuditLogEntry;
use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::auth::permission::CurrentUser;
use crate::auth::AuthenticatedKey;
use crate::error::ProblemResponse;

const AUDIT_LOG_COLUMNS: &str =
    "id, timestamp, actor_key_id, actor_key_name, client_id, resource_type, resource_id, action, before, after";

pub async fn record_audit<'a, E>(
    executor: E,
    actor: &AuthenticatedKey,
    client_id: Option<Uuid>,
    resource_type: &str,
    resource_id: Uuid,
    action: &str,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO audit_log (actor_key_id, actor_key_name, client_id, \
         resource_type, resource_id, action, before, after) \
         VALUES ($1, $2, $3, $4, $5, $6::audit_action, $7, $8)",
    )
    .bind(actor.id)
    .bind(&actor.name)
    .bind(client_id)
    .bind(resource_type)
    .bind(resource_id)
    .bind(action)
    .bind(before)
    .bind(after)
    .execute(executor)
    .await?;
    Ok(())
}

const VALID_AUDIT_ACTIONS: &[&str] = &["created", "updated", "deleted", "force_deleted", "adjusted"];

fn push_separator<'a>(builder: &mut QueryBuilder<'a, sqlx::Postgres>, first: &mut bool) {
    if *first {
        builder.push(" WHERE ");
        *first = false;
    } else {
        builder.push(" AND ");
    }
}

impl crate::pagination::HasId for AuditLogEntry {
    fn id(&self) -> Uuid {
        self.id
    }
}

pub async fn list_audit_log(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<AuditLogListQuery>,
) -> Result<Json<PaginatedResponse<AuditLogEntry>>, ProblemResponse> {
    if !user.is_super {
        return Err(ProblemResponse::forbidden("Super admin access required"));
    }

    let limit = query
        .limit
        .unwrap_or(50)
        .clamp(1, 200);

    let cursor: Option<Uuid> = match query.cursor.as_deref() {
        None | Some("") => None,
        Some(s) => Some(Uuid::parse_str(s).map_err(|_| {
            ProblemResponse::validation_error(
                format!("Invalid cursor: {s} is not a valid UUID"),
                vec![],
            )
        })?),
    };

    let cursor_ts: Option<DateTime<Utc>> = match cursor {
        Some(c) => {
            let row = sqlx::query("SELECT timestamp FROM audit_log WHERE id = $1")
                .bind(c)
                .fetch_optional(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "audit_log: cursor lookup failed");
                    ProblemResponse::unprocessable_entity("Failed to query audit log")
                })?;
            match row {
                Some(r) => Some(r.get("timestamp")),
                None => {
                    return Ok(Json(PaginatedResponse {
                        data: vec![],
                        next_cursor: None,
                        total: None,
                    }));
                }
            }
        }
        None => None,
    };

    let action_db_str: Option<&str> = match query.action.as_deref() {
        Some(a) if VALID_AUDIT_ACTIONS.contains(&a) => Some(a),
        Some(invalid) => {
            return Err(ProblemResponse::validation_error(
                format!("Invalid action: {invalid}. Expected one of: {}", VALID_AUDIT_ACTIONS.join(", ")),
                vec![],
            ));
        }
        None => None,
    };

    let fetch_limit = limit + 1;

    let mut builder = QueryBuilder::new(format!(
        "SELECT {AUDIT_LOG_COLUMNS} FROM audit_log"
    ));

    let mut first = true;

    if let (Some(c_ts), Some(c_id)) = (cursor_ts, cursor) {
        push_separator(&mut builder, &mut first);
        builder.push("(timestamp, id) < (");
        builder.push_bind(c_ts);
        builder.push(", ");
        builder.push_bind(c_id);
        builder.push(")");
    }

    if let Some(cid) = query.client_id {
        push_separator(&mut builder, &mut first);
        builder.push("client_id = ");
        builder.push_bind(cid);
    }

    if let Some(rt) = &query.resource_type {
        push_separator(&mut builder, &mut first);
        builder.push("resource_type = ");
        builder.push_bind(rt);
    }

    if let Some(action_str) = action_db_str {
        push_separator(&mut builder, &mut first);
        builder.push("action = ");
        builder.push_bind(action_str);
        builder.push("::audit_action");
    }

    if let Some(akid) = query.actor_key_id {
        push_separator(&mut builder, &mut first);
        builder.push("actor_key_id = ");
        builder.push_bind(akid);
    }

    if let Some(from) = query.from {
        push_separator(&mut builder, &mut first);
        builder.push("timestamp >= ");
        builder.push_bind(from);
    }

    if let Some(to) = query.to {
        push_separator(&mut builder, &mut first);
        builder.push("timestamp <= ");
        builder.push_bind(to);
    }

    builder.push(" ORDER BY timestamp DESC, id DESC LIMIT ");
    builder.push_bind(fetch_limit);

    let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
        tracing::error!(error = %e, "audit_log: list query failed");
        ProblemResponse::unprocessable_entity("Failed to query audit log")
    })?;

    let entries: Vec<AuditLogEntry> = rows.iter().map(row_to_audit_entry).collect();
    let response = crate::pagination::paginate_cursor(entries, limit);
    Ok(Json(response))
}

fn parse_audit_action(s: &str) -> arche_types::AuditAction {
    match s {
        "created" => arche_types::AuditAction::Created,
        "updated" => arche_types::AuditAction::Updated,
        "deleted" => arche_types::AuditAction::Deleted,
        "force_deleted" => arche_types::AuditAction::ForceDeleted,
        "adjusted" => arche_types::AuditAction::Adjusted,
        _ => {
            tracing::warn!(action = %s, "audit_log: unknown audit action, defaulting to Created");
            arche_types::AuditAction::Created
        }
    }
}

fn row_to_audit_entry(row: &sqlx::postgres::PgRow) -> AuditLogEntry {
    let action_str: String = row.get("action");
    AuditLogEntry {
        id: row.get("id"),
        timestamp: row.get("timestamp"),
        actor_key_id: row.get("actor_key_id"),
        actor_key_name: row.get("actor_key_name"),
        client_id: row.get("client_id"),
        resource_type: row.get("resource_type"),
        resource_id: row.get("resource_id"),
        action: parse_audit_action(&action_str),
        before: row.get("before"),
        after: row.get("after"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pagination::HasId;
    use arche_types::common::{AuditLogListQuery, PaginatedResponse};
    use serde_json::json;

    #[test]
    fn test_audit_log_list_query_deserialization_empty() {
        let query: AuditLogListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.cursor, None);
        assert_eq!(query.limit, None);
        assert_eq!(query.client_id, None);
        assert_eq!(query.resource_type, None);
        assert_eq!(query.action, None);
        assert_eq!(query.actor_key_id, None);
        assert_eq!(query.from, None);
        assert_eq!(query.to, None);
    }

    #[test]
    fn test_audit_log_list_query_deserialization_full() {
        let cid = Uuid::new_v4();
        let akid = Uuid::new_v4();
        let from = DateTime::from_timestamp_millis(1000).unwrap();
        let to = DateTime::from_timestamp_millis(2000).unwrap();
        let json = json!({
            "cursor": "550e8400-e29b-41d4-a716-446655440000",
            "limit": 25,
            "clientId": cid.to_string(),
            "resourceType": "blueprint",
            "action": "created",
            "actorKeyId": akid.to_string(),
            "from": "1970-01-01T00:00:01Z",
            "to": "1970-01-01T00:00:02Z",
        });
        let query: AuditLogListQuery = serde_json::from_value(json).unwrap();
        assert_eq!(query.cursor, Some("550e8400-e29b-41d4-a716-446655440000".into()));
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.client_id, Some(cid));
        assert_eq!(query.resource_type, Some("blueprint".into()));
        assert_eq!(query.action, Some("created".into()));
        assert_eq!(query.actor_key_id, Some(akid));
        assert_eq!(query.from, Some(from));
        assert_eq!(query.to, Some(to));
    }

    #[test]
    fn test_audit_log_list_query_camel_case() {
        let cid = Uuid::new_v4();
        let akid = Uuid::new_v4();
        let from = DateTime::from_timestamp_millis(1000).unwrap();
        let to = DateTime::from_timestamp_millis(2000).unwrap();
        let q = AuditLogListQuery {
            cursor: None,
            limit: None,
            client_id: Some(cid),
            resource_type: Some("blueprint".into()),
            action: Some("created".into()),
            actor_key_id: Some(akid),
            from: Some(from),
            to: Some(to),
        };
        let obj = serde_json::to_value(&q).unwrap().as_object().unwrap().clone();
        assert!(obj.contains_key("clientId"));
        assert!(obj.contains_key("resourceType"));
        assert!(obj.contains_key("action"));
        assert!(obj.contains_key("actorKeyId"));
        assert_eq!(obj.get("from").unwrap(), "1970-01-01T00:00:01Z");
        assert_eq!(obj.get("to").unwrap(), "1970-01-01T00:00:02Z");
    }

    #[test]
    fn test_audit_log_entry_has_id() {
        let id = Uuid::new_v4();
        let entry = AuditLogEntry {
            id,
            timestamp: Utc::now(),
            actor_key_id: Uuid::new_v4(),
            actor_key_name: "test-key".into(),
            client_id: None,
            resource_type: "blueprint".into(),
            resource_id: Uuid::new_v4(),
            action: arche_types::AuditAction::Created,
            before: None,
            after: None,
        };
        assert_eq!(entry.id(), id);
    }

    #[test]
    fn test_paginate_cursor_with_audit_entries() {
        let ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let entries: Vec<AuditLogEntry> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| AuditLogEntry {
                id: *id,
                timestamp: Utc::now(),
                actor_key_id: Uuid::new_v4(),
                actor_key_name: format!("key-{i}"),
                client_id: None,
                resource_type: "blueprint".into(),
                resource_id: Uuid::new_v4(),
                action: arche_types::AuditAction::Created,
                before: None,
                after: None,
            })
            .collect();

        let result = crate::pagination::paginate_cursor(entries, 3);
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.next_cursor, Some(ids[2].to_string()));
    }

    #[test]
    fn test_action_validation_valid_values() {
        let valid = &["created", "updated", "deleted", "force_deleted", "adjusted"];
        for action in valid {
            let q = AuditLogListQuery {
                cursor: None,
                limit: None,
                client_id: None,
                resource_type: None,
                action: Some(action.to_string()),
                actor_key_id: None,
                from: None,
                to: None,
            };
            let json = serde_json::to_string(&q).unwrap();
            let deserialized: AuditLogListQuery = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.action, Some(action.to_string()));
        }
    }

    #[test]
    fn test_paginated_response_with_audit_entries_serialization() {
        let uuid = Uuid::new_v4();
        let resp: PaginatedResponse<AuditLogEntry> = PaginatedResponse {
            data: vec![],
            next_cursor: Some(uuid.to_string()),
            total: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("data"));
        assert!(obj.contains_key("nextCursor"));
        assert!(!obj.contains_key("total"));
        assert_eq!(obj.get("nextCursor").unwrap(), &serde_json::json!(uuid.to_string()));
    }

    mod db_tests {
        use super::*;
        use arche_types::AuditAction;
        use serde_json::json;
        use sqlx::PgPool;
        use std::sync::Arc;

        async fn ensure_db() -> Option<PgPool> {
            let url = std::env::var("DATABASE_URL").ok()?;
            let pool = PgPool::connect(&url).await.ok()?;
            sqlx::migrate!("../../arche-service/migrations")
                .run(&pool)
                .await
                .ok()?;
            Some(pool)
        }

        fn test_state(pool: Arc<PgPool>) -> crate::AppState {
            crate::AppState {
                cache: Arc::new(tokio::sync::RwLock::new(crate::cache::Cache::new(
                    std::collections::HashMap::new(),
                    std::collections::HashMap::new(),
                    std::collections::HashMap::new(),
                    std::collections::HashMap::new(),
                    std::collections::HashMap::new(),
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

        fn non_super_user(client_id: Uuid) -> CurrentUser {
            CurrentUser(crate::auth::AuthenticatedKey {
                id: Uuid::nil(),
                name: "nonsuper".into(),
                client_id: Some(client_id),
                permissions: vec![arche_types::Permission::Read],
                is_super: false,
            })
        }

        async fn create_test_client(pool: &PgPool) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO clients (name) VALUES ('audit-test-client') RETURNING id",
            )
            .fetch_one(pool)
            .await
            .expect("Failed to create test client");
            row.get("id")
        }

        async fn create_test_api_key(pool: &PgPool) -> (Uuid, String) {
            let id = Uuid::new_v4();
            let row = sqlx::query(
                "INSERT INTO api_keys (id, name, key_hash, permissions, is_super) \
                 VALUES ($1, 'test-key', 'hash', ARRAY['read']::text[], true) \
                 RETURNING id, name",
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .expect("Failed to create test api key");
            let name: String = row.get("name");
            (id, name)
        }

        async fn insert_audit_entry(
            pool: &PgPool,
            actor_key_id: Uuid,
            actor_key_name: &str,
            client_id: Option<Uuid>,
            resource_type: &str,
            resource_id: Uuid,
            action: &str,
            before: Option<serde_json::Value>,
            after: Option<serde_json::Value>,
        ) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO audit_log (actor_key_id, actor_key_name, client_id, resource_type, resource_id, action, before, after) \
                 VALUES ($1, $2, $3, $4, $5, $6::audit_action, $7, $8) \
                 RETURNING id",
            )
            .bind(actor_key_id)
            .bind(actor_key_name)
            .bind(client_id)
            .bind(resource_type)
            .bind(resource_id)
            .bind(action)
            .bind(before)
            .bind(after)
            .fetch_one(pool)
            .await
            .expect("Failed to insert audit entry");
            row.get("id")
        }

        #[tokio::test]
        async fn test_list_audit_log_empty() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: None,
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 0);
            assert_eq!(resp.next_cursor, None);
        }

        #[tokio::test]
        async fn test_non_super_admin_gets_403() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: None,
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                non_super_user(client_id),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 403);

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_list_audit_log_pagination() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let (key_id, key_name) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            let mut entry_ids = Vec::new();
            for i in 0..5 {
                let id = insert_audit_entry(
                    &pool,
                    key_id,
                    &key_name,
                    Some(client_id),
                    "blueprint",
                    resource_id,
                    "created",
                    None,
                    Some(json!({"name": format!("bp-{i}")})),
                )
                .await;
                entry_ids.push(id);
            }

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(3),
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query.clone()),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 3);
            assert!(resp.next_cursor.is_some(), "should have next_cursor");

            let cursor = resp.next_cursor.clone().unwrap();

            let state2 = test_state(Arc::new(pool.clone()));
            let query2 = AuditLogListQuery {
                cursor: Some(cursor),
                limit: Some(3),
                ..query
            };

            let result2 = list_audit_log(
                axum::extract::State(state2),
                super_user(),
                axum::extract::Query(query2),
            )
            .await;

            assert!(result2.is_ok(), "page 2 failed: {:?}", result2.err());
            let resp2 = result2.unwrap();
            assert_eq!(resp2.data.len(), 2);
            assert_eq!(resp2.next_cursor, None);

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
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
        async fn test_list_audit_log_default_sort_timestamp_desc() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let (key_id, key_name) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_id),
                "blueprint", resource_id, "created",
                None, Some(json!({"name": "oldest"})),
            ).await;

            std::thread::sleep(std::time::Duration::from_millis(10));

            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_id),
                "blueprint", resource_id, "updated",
                None, Some(json!({"name": "newest"})),
            ).await;

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 2);
            assert_eq!(resp.data[0].action, AuditAction::Updated);
            assert_eq!(resp.data[1].action, AuditAction::Created);
            assert!(resp.data[0].timestamp >= resp.data[1].timestamp,
                "newest should come first");

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
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
        async fn test_list_audit_log_filter_by_client_id() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;
            let (key_id, key_name) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_a),
                "blueprint", resource_id, "created",
                None, Some(json!({"client": "a"})),
            ).await;
            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_b),
                "blueprint", resource_id, "deleted",
                None, Some(json!({"client": "b"})),
            ).await;

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: Some(client_a),
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "filter by client failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 1);
            assert_eq!(resp.data[0].client_id, Some(client_a));

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = ANY($1)")
                .bind(&[client_a, client_b])
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_list_audit_log_filter_by_resource_type() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let (key_id, key_name) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_id),
                "blueprint", resource_id, "created",
                None, None,
            ).await;
            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_id),
                "affix", resource_id, "created",
                None, None,
            ).await;

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: None,
                resource_type: Some("blueprint".into()),
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "filter by type failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 1);
            assert_eq!(resp.data[0].resource_type, "blueprint");

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
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
        async fn test_list_audit_log_filter_by_action() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let (key_id, key_name) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_id),
                "blueprint", resource_id, "created",
                None, None,
            ).await;
            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_id),
                "blueprint", resource_id, "deleted",
                None, None,
            ).await;

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: None,
                resource_type: None,
                action: Some("deleted".into()),
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "filter by action failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 1);
            assert_eq!(resp.data[0].action, AuditAction::Deleted);

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
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
        async fn test_list_audit_log_filter_by_actor_key_id() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let (key_a, name_a) = create_test_api_key(&pool).await;
            let (key_b, name_b) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            insert_audit_entry(
                &pool, key_a, &name_a, Some(client_id),
                "blueprint", resource_id, "created",
                None, None,
            ).await;
            insert_audit_entry(
                &pool, key_b, &name_b, Some(client_id),
                "blueprint", resource_id, "created",
                None, None,
            ).await;

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: Some(key_a),
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "filter by actor failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 1);
            assert_eq!(resp.data[0].actor_key_id, key_a);

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = ANY($1)")
                .bind(&[key_a, key_b])
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = ANY($1)")
                .bind(&[key_a, key_b])
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
        async fn test_list_audit_log_filter_from_to() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let (key_id, key_name) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            let mid = Utc::now();

            sqlx::query(
                "INSERT INTO audit_log (actor_key_id, actor_key_name, client_id, resource_type, resource_id, action, timestamp) \
                 VALUES ($1, $2, $3, 'blueprint', $4, 'created'::audit_action, $5)"
            )
            .bind(key_id)
            .bind(&key_name)
            .bind(client_id)
            .bind(resource_id)
            .bind(mid - chrono::Duration::hours(2))
            .execute(&pool)
            .await
            .expect("insert old entry");

            sqlx::query(
                "INSERT INTO audit_log (actor_key_id, actor_key_name, client_id, resource_type, resource_id, action, timestamp) \
                 VALUES ($1, $2, $3, 'blueprint', $4, 'deleted'::audit_action, $5)"
            )
            .bind(key_id)
            .bind(&key_name)
            .bind(client_id)
            .bind(resource_id)
            .bind(mid + chrono::Duration::hours(2))
            .execute(&pool)
            .await
            .expect("insert future entry");

            let from = mid - chrono::Duration::minutes(30);
            let to = mid + chrono::Duration::minutes(30);

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: Some(from),
                to: Some(to),
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "from/to filter failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 0, "should find no entries in the from/to window");

            let state2 = test_state(Arc::new(pool.clone()));
            let query2 = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: Some(mid - chrono::Duration::hours(3)),
                to: Some(mid + chrono::Duration::hours(3)),
            };

            let result2 = list_audit_log(
                axum::extract::State(state2),
                super_user(),
                axum::extract::Query(query2),
            )
            .await;

            assert!(result2.is_ok(), "wide from/to filter failed: {:?}", result2.err());
            let resp2 = result2.unwrap();
            assert_eq!(resp2.data.len(), 2, "should find both entries in wide window");

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
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
        async fn test_list_audit_log_invalid_cursor() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: Some("not-a-uuid".into()),
                limit: None,
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
        }

        #[tokio::test]
        async fn test_list_audit_log_invalid_action() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: None,
                client_id: None,
                resource_type: None,
                action: Some("invalid_action".into()),
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
        }

        #[tokio::test]
        async fn test_list_audit_log_combined_filters() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;
            let (key_id, key_name) = create_test_api_key(&pool).await;
            let resource_id = Uuid::new_v4();

            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_a),
                "blueprint", resource_id, "created",
                None, None,
            ).await;
            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_a),
                "affix", resource_id, "deleted",
                None, None,
            ).await;
            insert_audit_entry(
                &pool, key_id, &key_name, Some(client_b),
                "blueprint", resource_id, "created",
                None, None,
            ).await;

            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: None,
                limit: Some(10),
                client_id: Some(client_a),
                resource_type: Some("blueprint".into()),
                action: Some("created".into()),
                actor_key_id: Some(key_id),
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "combined filters failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 1);
            assert_eq!(resp.data[0].client_id, Some(client_a));
            assert_eq!(resp.data[0].resource_type, "blueprint");
            assert_eq!(resp.data[0].action, AuditAction::Created);

            sqlx::query("DELETE FROM audit_log WHERE actor_key_id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM api_keys WHERE id = $1")
                .bind(key_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = ANY($1)")
                .bind(&[client_a, client_b])
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_list_audit_log_nonexistent_cursor_returns_empty() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let state = test_state(Arc::new(pool.clone()));
            let query = AuditLogListQuery {
                cursor: Some(Uuid::new_v4().to_string()),
                limit: Some(10),
                client_id: None,
                resource_type: None,
                action: None,
                actor_key_id: None,
                from: None,
                to: None,
            };

            let result = list_audit_log(
                axum::extract::State(state),
                super_user(),
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok());
            let resp = result.unwrap();
            assert_eq!(resp.data.len(), 0);
            assert_eq!(resp.next_cursor, None);
        }
    }
}

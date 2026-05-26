use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::audit_log::record_audit;
use crate::auth::permission::CurrentUser;
use crate::error::{ProblemResponse, ReferenceInfo};

const GMA_COLUMNS: &str =
    "id, client_id, name, description, value_type, payload, created_at, updated_at";

#[derive(Debug, Deserialize)]
pub struct DeleteGlobalMetaAttributeQuery {
    #[serde(default)]
    pub force: bool,
}

pub async fn delete_global_meta_attribute(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteGlobalMetaAttributeQuery>,
) -> Result<Json<serde_json::Value>, ProblemResponse> {
    let row = sqlx::query(&format!(
        "SELECT {GMA_COLUMNS} FROM global_meta_attributes WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "gma: get for delete failed");
        ProblemResponse::unprocessable_entity("Failed to get global meta attribute for deletion")
    })?
    .ok_or_else(|| {
        ProblemResponse::not_found(format!("Global meta attribute not found: {id}"))
    })?;

    let gma_client_id: Uuid = row.get("client_id");
    let gma_name: String = row.get("name");
    let gma_description: Option<String> = row.get("description");
    let gma_value_type: String = row.get("value_type");
    let gma_payload: serde_json::Value = row.get("payload");

    let before_snapshot = serde_json::json!({
        "id": id,
        "client_id": gma_client_id,
        "name": &gma_name,
        "description": &gma_description,
        "value_type": &gma_value_type,
        "payload": &gma_payload,
    });

    if !user.is_super {
        let user_cid = user.client_id.ok_or_else(|| {
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })?;
        if gma_client_id != user_cid {
            return Err(ProblemResponse::not_found(format!(
                "Global meta attribute not found: {id}"
            )));
        }
    }

    let bp_ref_rows = sqlx::query(
        "SELECT b.id, b.name, bp.key as attr_key \
         FROM blueprints b, jsonb_each(b.attributes) as bp \
         WHERE b.client_id = $1 AND bp.value->>'$ref_id' = $2",
    )
    .bind(gma_client_id)
    .bind(id.to_string())
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "gma: check blueprint references failed");
        ProblemResponse::unprocessable_entity("Failed to check global meta attribute references")
    })?;

    let affix_ref_rows = sqlx::query(
        "SELECT id, name FROM affixes \
         WHERE client_id = $1 AND attribute->>'$ref_id' = $2",
    )
    .bind(gma_client_id)
    .bind(id.to_string())
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "gma: check affix references failed");
        ProblemResponse::unprocessable_entity("Failed to check global meta attribute references")
    })?;

    let total_refs = bp_ref_rows.len() + affix_ref_rows.len();

    if total_refs > 0 {
        if !query.force {
            let mut references: Vec<ReferenceInfo> = Vec::new();

            for row in &bp_ref_rows {
                let bp_id: Uuid = row.get("id");
                let bp_name: String = row.get("name");
                references.push(ReferenceInfo {
                    resource_type: "blueprint".into(),
                    resource_id: bp_id,
                    resource_name: bp_name,
                });
            }

            for row in &affix_ref_rows {
                let affix_id: Uuid = row.get("id");
                let affix_name: String = row.get("name");
                references.push(ReferenceInfo {
                    resource_type: "affix".into(),
                    resource_id: affix_id,
                    resource_name: affix_name,
                });
            }

            return Err(ProblemResponse::delete_referenced_resource_with_refs(
                format!(
                    "Global meta attribute {id} is referenced by {total_refs} resources ({} blueprints, {} affixes). Use force=true to cascade delete.",
                    bp_ref_rows.len(),
                    affix_ref_rows.len()
                ),
                references,
            ));
        }

        let mut tx = state.pool.begin().await.map_err(|e| {
            tracing::error!(error = %e, "gma: begin transaction failed");
            ProblemResponse::unprocessable_entity("Failed to begin transaction for force delete")
        })?;

        for row in &bp_ref_rows {
            let bp_id: Uuid = row.get("id");
            let attr_key: String = row.get("attr_key");

            sqlx::query(
                "UPDATE blueprints SET attributes = attributes - $1, updated_at = now() \
                 WHERE id = $2",
            )
            .bind(&attr_key)
            .bind(bp_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, bp_id = %bp_id, "gma: remove blueprint attribute failed");
                ProblemResponse::unprocessable_entity("Failed to remove blueprint attribute reference")
            })?;

            sqlx::query(
                "INSERT INTO audit_log (actor_key_id, actor_key_name, client_id, \
                 resource_type, resource_id, action, before, after) \
                 VALUES ($1, $2, $3, 'blueprint', $4, 'adjusted'::audit_action, \
                 $5, $6)",
            )
            .bind(user.id)
            .bind(&user.name)
            .bind(gma_client_id)
            .bind(bp_id)
            .bind(serde_json::json!({"attribute_key": &attr_key, "removed_ref_id": id.to_string()}))
            .bind(serde_json::json!({"attribute_key": &attr_key, "removed": true}))
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "gma: audit log insert failed");
                ProblemResponse::unprocessable_entity("Failed to write audit log")
            })?;
        }

        for row in &affix_ref_rows {
            let affix_id: Uuid = row.get("id");

            sqlx::query(
                "UPDATE affixes SET attribute = '{}'::jsonb, updated_at = now() \
                 WHERE id = $1",
            )
            .bind(affix_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, affix_id = %affix_id, "gma: clear affix attribute failed");
                ProblemResponse::unprocessable_entity("Failed to clear affix attribute reference")
            })?;

            sqlx::query(
                "INSERT INTO audit_log (actor_key_id, actor_key_name, client_id, \
                 resource_type, resource_id, action, before, after) \
                 VALUES ($1, $2, $3, 'affix', $4, 'adjusted'::audit_action, \
                 $5, $6)",
            )
            .bind(user.id)
            .bind(&user.name)
            .bind(gma_client_id)
            .bind(affix_id)
            .bind(serde_json::json!({"removed_ref_id": id.to_string()}))
            .bind(serde_json::json!({"attribute": {}}))
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "gma: audit log insert failed");
                ProblemResponse::unprocessable_entity("Failed to write audit log")
            })?;
        }

        sqlx::query("DELETE FROM global_meta_attributes WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "gma: delete gma failed");
                ProblemResponse::unprocessable_entity("Failed to delete global meta attribute")
            })?;

        record_audit(
            &mut *tx,
            &user,
            Some(gma_client_id),
            "global_meta_attribute",
            id,
            "force_deleted",
            Some(before_snapshot),
            None,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "gma: audit log insert failed");
            ProblemResponse::unprocessable_entity("Failed to write audit log")
        })?;

        tx.commit().await.map_err(|e| {
            tracing::error!(error = %e, "gma: commit transaction failed");
            ProblemResponse::unprocessable_entity("Failed to commit force delete transaction")
        })?;
    } else {
        sqlx::query("DELETE FROM global_meta_attributes WHERE id = $1")
            .bind(id)
            .execute(&*state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "gma: delete gma failed");
                ProblemResponse::unprocessable_entity("Failed to delete global meta attribute")
            })?;

        if let Err(e) = record_audit(
            &*state.pool,
            &user,
            Some(gma_client_id),
            "global_meta_attribute",
            id,
            "deleted",
            Some(before_snapshot),
            None,
        )
        .await
        {
            tracing::error!(error = %e, "gma: audit log insert failed");
        }
    }

    Ok(Json(serde_json::json!({"deleted": true})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_gma_query_default_force_false() {
        let query: DeleteGlobalMetaAttributeQuery = serde_json::from_str("{}").unwrap();
        assert!(!query.force);
    }

    #[test]
    fn test_delete_gma_query_force_true() {
        let query: DeleteGlobalMetaAttributeQuery =
            serde_json::from_str(r#"{"force":true}"#).unwrap();
        assert!(query.force);
    }

    mod db_tests {
        use super::*;
        use crate::AppState;
        use arche_types::Permission;
        use serde_json::json;
        use sqlx::PgPool;
        use std::collections::HashMap;
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

        async fn create_test_client(pool: &PgPool) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO clients (name) VALUES ('test-client') RETURNING id",
            )
            .fetch_one(pool)
            .await
            .expect("Failed to create test client");
            row.get("id")
        }

        async fn create_test_gma(
            pool: &PgPool,
            client_id: Uuid,
            name: &str,
        ) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, value_type, payload) \
                 VALUES ($1, $2, 'range'::value_type, $3) RETURNING id",
            )
            .bind(client_id)
            .bind(name)
            .bind(&json!({"min": 1.0, "max": 10.0}))
            .fetch_one(pool)
            .await
            .expect("Failed to create test GMA");
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

        fn test_user(client_id: Uuid, permission: Permission) -> CurrentUser {
            CurrentUser(crate::auth::AuthenticatedKey {
                id: Uuid::nil(),
                name: "test".into(),
                client_id: Some(client_id),
                permissions: vec![permission],
                is_super: false,
            })
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

        #[tokio::test]
        async fn test_delete_gma_no_references_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::extract::Query(DeleteGlobalMetaAttributeQuery { force: false }),
            )
            .await;

            assert!(result.is_ok(), "delete failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0, json!({"deleted": true}));

            let count: i64 =
                sqlx::query("SELECT COUNT(*) FROM global_meta_attributes WHERE id = $1")
                    .bind(gma_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .get(0);
            assert_eq!(count, 0);

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_gma_referenced_by_blueprint_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind("Sword")
            .bind("sword")
            .bind(1.0)
            .bind(&json!({"rarity": {"$ref_id": gma_id.to_string()}}))
            .bind(&vec!["rarity".to_string()])
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::extract::Query(DeleteGlobalMetaAttributeQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("referenced by 1"));
            assert!(err.references.is_some());
            let refs = err.references.unwrap();
            assert_eq!(refs.len(), 1);
            assert_eq!(refs[0].resource_type, "blueprint");
            assert_eq!(refs[0].resource_id, bp_id);
            assert_eq!(refs[0].resource_name, "Sword");

            let gma_count: i64 =
                sqlx::query("SELECT COUNT(*) FROM global_meta_attributes WHERE id = $1")
                    .bind(gma_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .get(0);
            assert_eq!(gma_count, 1);

            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM global_meta_attributes WHERE id = $1")
                .bind(gma_id)
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
        async fn test_delete_gma_referenced_by_affix_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let affix_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO affixes (id, client_id, name, type, attribute) \
                 VALUES ($1,$2,$3,'prefix'::affix_location,$4)",
            )
            .bind(affix_id)
            .bind(client_id)
            .bind("Fire")
            .bind(&json!({"$ref_id": gma_id.to_string()}))
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::extract::Query(DeleteGlobalMetaAttributeQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("referenced by 1"));
            assert!(err.references.is_some());
            let refs = err.references.unwrap();
            assert_eq!(refs.len(), 1);
            assert_eq!(refs[0].resource_type, "affix");
            assert_eq!(refs[0].resource_id, affix_id);
            assert_eq!(refs[0].resource_name, "Fire");

            let gma_count: i64 =
                sqlx::query("SELECT COUNT(*) FROM global_meta_attributes WHERE id = $1")
                    .bind(gma_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .get(0);
            assert_eq!(gma_count, 1);

            sqlx::query("DELETE FROM affixes WHERE id = $1")
                .bind(affix_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM global_meta_attributes WHERE id = $1")
                .bind(gma_id)
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
        async fn test_force_delete_gma_removes_blueprint_ref() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind("Sword")
            .bind("sword")
            .bind(1.0)
            .bind(&json!({"rarity": {"$ref_id": gma_id.to_string()}, "damage": {"value_type": "single", "value": 10.0}}))
            .bind(&vec!["rarity".to_string(), "damage".to_string()])
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = delete_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::extract::Query(DeleteGlobalMetaAttributeQuery { force: true }),
            )
            .await;

            assert!(result.is_ok(), "force delete failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0, json!({"deleted": true}));

            let gma_count: i64 =
                sqlx::query("SELECT COUNT(*) FROM global_meta_attributes WHERE id = $1")
                    .bind(gma_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .get(0);
            assert_eq!(gma_count, 0);

            let bp_attrs: serde_json::Value =
                sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                    .bind(bp_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(bp_attrs.get("rarity").is_none(), "rarity attribute should be removed");
            assert!(bp_attrs.get("damage").is_some(), "damage attribute should remain");

            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp_id)
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
        async fn test_force_delete_gma_removes_affix_ref() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let affix_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO affixes (id, client_id, name, type, attribute) \
                 VALUES ($1,$2,$3,'prefix'::affix_location,$4)",
            )
            .bind(affix_id)
            .bind(client_id)
            .bind("Fire")
            .bind(&json!({"$ref_id": gma_id.to_string()}))
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = delete_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::extract::Query(DeleteGlobalMetaAttributeQuery { force: true }),
            )
            .await;

            assert!(result.is_ok(), "force delete failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0, json!({"deleted": true}));

            let gma_count: i64 =
                sqlx::query("SELECT COUNT(*) FROM global_meta_attributes WHERE id = $1")
                    .bind(gma_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .get(0);
            assert_eq!(gma_count, 0);

            let affix_attr: serde_json::Value =
                sqlx::query_scalar("SELECT attribute FROM affixes WHERE id = $1")
                    .bind(affix_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(affix_attr, json!({}), "affix attribute should be cleared");

            sqlx::query("DELETE FROM affixes WHERE id = $1")
                .bind(affix_id)
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
        async fn test_delete_nonexistent_gma_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(Uuid::new_v4()),
                axum::extract::Query(DeleteGlobalMetaAttributeQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains("Global meta attribute not found"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_gma_wrong_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_a, "rarity").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_b, Permission::Delete);

            let result = delete_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::extract::Query(DeleteGlobalMetaAttributeQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains("Global meta attribute not found"));

            let count: i64 =
                sqlx::query("SELECT COUNT(*) FROM global_meta_attributes WHERE id = $1")
                    .bind(gma_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .get(0);
            assert_eq!(count, 1);

            sqlx::query("DELETE FROM global_meta_attributes WHERE id = $1")
                .bind(gma_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_a)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_b)
                .execute(&pool)
                .await
                .unwrap();
        }
    }
}

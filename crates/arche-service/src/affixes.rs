use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::audit_log::record_audit;
use crate::auth::permission::CurrentUser;
use crate::error::{ProblemResponse, ReferenceInfo};

const AFFIX_COLUMNS: &str =
    "id, client_id, name, type, description, attribute, created_at, updated_at";

#[derive(Debug, Deserialize)]
pub struct DeleteAffixQuery {
    #[serde(default)]
    pub force: bool,
}

pub async fn delete_affix(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteAffixQuery>,
) -> Result<Json<serde_json::Value>, ProblemResponse> {
    let row = sqlx::query(&format!(
        "SELECT {AFFIX_COLUMNS} FROM affixes WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: get for delete failed");
        ProblemResponse::unprocessable_entity("Failed to get affix for deletion")
    })?
    .ok_or_else(|| ProblemResponse::not_found(format!("Affix not found: {id}")))?;

    let affix_client_id: Uuid = row.get("client_id");
    let affix_name: String = row.get("name");
    let affix_type: String = row.get("type");
    let affix_description: Option<String> = row.get("description");
    let affix_attribute: serde_json::Value = row.get("attribute");

    let before_snapshot = serde_json::json!({
        "id": id,
        "client_id": affix_client_id,
        "name": &affix_name,
        "type": &affix_type,
        "description": &affix_description,
        "attribute": &affix_attribute,
    });

    if !user.is_super {
        let user_cid = user.client_id.ok_or_else(|| {
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })?;
        if affix_client_id != user_cid {
            return Err(ProblemResponse::not_found(format!("Affix not found: {id}")));
        }
    }

    let ref_rows = sqlx::query(
        "SELECT ba.blueprint_id, b.name as blueprint_name \
         FROM blueprint_affixes ba \
         JOIN blueprints b ON b.id = ba.blueprint_id \
         WHERE ba.affix_id = $1",
    )
    .bind(id)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: check references failed");
        ProblemResponse::unprocessable_entity("Failed to check affix references")
    })?;

    if !ref_rows.is_empty() {
        if !query.force {
            let references: Vec<ReferenceInfo> = ref_rows
                .iter()
                .map(|r| {
                    let bp_id: Uuid = r.get("blueprint_id");
                    let bp_name: String = r.get("blueprint_name");
                    ReferenceInfo {
                        resource_type: "blueprint".into(),
                        resource_id: bp_id,
                        resource_name: bp_name,
                    }
                })
                .collect();

            return Err(ProblemResponse::delete_referenced_resource_with_refs(
                format!(
                    "Affix {id} is referenced by {} blueprint pool entries. Use force=true to cascade delete.",
                    ref_rows.len()
                ),
                references,
            ));
        }

        let mut tx = state.pool.begin().await.map_err(|e| {
            tracing::error!(error = %e, "affixes: begin transaction failed");
            ProblemResponse::unprocessable_entity("Failed to begin transaction for force delete")
        })?;

        let mut unique_blueprint_ids: Vec<Uuid> = Vec::new();
        for row in &ref_rows {
            let bp_id: Uuid = row.get("blueprint_id");
            if !unique_blueprint_ids.contains(&bp_id) {
                unique_blueprint_ids.push(bp_id);
            }
        }

        let bp_rows = sqlx::query(
            "SELECT id, min_prefixes, max_prefixes, min_suffixes, max_suffixes \
             FROM blueprints WHERE id = ANY($1)",
        )
        .bind(&unique_blueprint_ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "affixes: fetch blueprints for adjustment failed");
            ProblemResponse::unprocessable_entity("Failed to fetch blueprints for adjustment")
        })?;

        for bp_row in &bp_rows {
            let bp_id: Uuid = bp_row.get("id");
            let current_min_p: i32 = bp_row.get("min_prefixes");
            let current_max_p: i32 = bp_row.get("max_prefixes");
            let current_min_s: i32 = bp_row.get("min_suffixes");
            let current_max_s: i32 = bp_row.get("max_suffixes");

            let remaining_prefixes: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprint_affixes \
                 WHERE blueprint_id = $1 AND affix_id != $2 AND location = 'prefix'",
            )
            .bind(bp_id)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "affixes: count remaining prefixes failed");
                ProblemResponse::unprocessable_entity("Failed to count remaining affix entries")
            })?
            .get(0);

            let remaining_suffixes: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprint_affixes \
                 WHERE blueprint_id = $1 AND affix_id != $2 AND location = 'suffix'",
            )
            .bind(bp_id)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "affixes: count remaining suffixes failed");
                ProblemResponse::unprocessable_entity("Failed to count remaining affix entries")
            })?
            .get(0);

            let remaining_p = remaining_prefixes as i32;
            let remaining_s = remaining_suffixes as i32;

            let new_max_p = std::cmp::min(current_max_p, remaining_p);
            let new_min_p = std::cmp::min(current_min_p, new_max_p);
            let new_max_s = std::cmp::min(current_max_s, remaining_s);
            let new_min_s = std::cmp::min(current_min_s, new_max_s);

            if new_min_p != current_min_p
                || new_max_p != current_max_p
                || new_min_s != current_min_s
                || new_max_s != current_max_s
            {
                sqlx::query(
                    "UPDATE blueprints SET min_prefixes=$1, max_prefixes=$2, \
                     min_suffixes=$3, max_suffixes=$4, updated_at=now() \
                     WHERE id=$5",
                )
                .bind(new_min_p)
                .bind(new_max_p)
                .bind(new_min_s)
                .bind(new_max_s)
                .bind(bp_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "affixes: update blueprint min/max failed");
                    ProblemResponse::unprocessable_entity("Failed to adjust blueprint affix counts")
                })?;

                record_audit(
                    &mut *tx,
                    &user,
                    Some(affix_client_id),
                    "blueprint",
                    bp_id,
                    "adjusted",
                    Some(serde_json::json!({
                        "min_prefixes": current_min_p,
                        "max_prefixes": current_max_p,
                        "min_suffixes": current_min_s,
                        "max_suffixes": current_max_s,
                    })),
                    Some(serde_json::json!({
                        "min_prefixes": new_min_p,
                        "max_prefixes": new_max_p,
                        "min_suffixes": new_min_s,
                        "max_suffixes": new_max_s,
                    })),
                )
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "affixes: audit log insert failed");
                    ProblemResponse::unprocessable_entity("Failed to write audit log")
                })?;
            }
        }

        sqlx::query("DELETE FROM blueprint_affixes WHERE affix_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "affixes: delete blueprint_affixes failed");
                ProblemResponse::unprocessable_entity("Failed to remove affix pool entries")
            })?;

        sqlx::query("DELETE FROM affixes WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "affixes: delete affix failed");
                ProblemResponse::unprocessable_entity("Failed to delete affix")
            })?;

        record_audit(
            &mut *tx,
            &user,
            Some(affix_client_id),
            "affix",
            id,
            "force_deleted",
            Some(before_snapshot),
            None,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "affixes: audit log insert failed");
            ProblemResponse::unprocessable_entity("Failed to write audit log")
        })?;

        tx.commit().await.map_err(|e| {
            tracing::error!(error = %e, "affixes: commit transaction failed");
            ProblemResponse::unprocessable_entity("Failed to commit force delete transaction")
        })?;
    } else {
        sqlx::query("DELETE FROM affixes WHERE id = $1")
            .bind(id)
            .execute(&*state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "affixes: delete affix failed");
                ProblemResponse::unprocessable_entity("Failed to delete affix")
            })?;

        if let Err(e) = record_audit(
            &*state.pool,
            &user,
            Some(affix_client_id),
            "affix",
            id,
            "deleted",
            Some(before_snapshot),
            None,
        )
        .await
        {
            tracing::error!(error = %e, "affixes: audit log insert failed");
        }
    }

    Ok(Json(serde_json::json!({"deleted": true})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_affix_query_default_force_false() {
        let query: DeleteAffixQuery = serde_json::from_str("{}").unwrap();
        assert!(!query.force);
    }

    #[test]
    fn test_delete_affix_query_force_true() {
        let query: DeleteAffixQuery = serde_json::from_str(r#"{"force":true}"#).unwrap();
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

        async fn create_test_affix(
            pool: &PgPool,
            client_id: Uuid,
            name: &str,
            location: &str,
        ) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO affixes (client_id, name, type, attribute) \
                 VALUES ($1, $2, $3::affix_location, $4) RETURNING id",
            )
            .bind(client_id)
            .bind(name)
            .bind(location)
            .bind(&json!({"value_type": "single", "value": 1.0}))
            .fetch_one(pool)
            .await
            .expect("Failed to create test affix");
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
        async fn test_delete_affix_no_references_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix_id = create_test_affix(&pool, client_id, "Fire", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_id),
                axum::extract::Query(DeleteAffixQuery { force: false }),
            )
            .await;

            assert!(result.is_ok(), "delete failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0, json!({"deleted": true}));

            let count: i64 = sqlx::query("SELECT COUNT(*) FROM affixes WHERE id = $1")
                .bind(affix_id)
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
        async fn test_delete_affix_referenced_by_blueprints_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix_id = create_test_affix(&pool, client_id, "Fire", "prefix").await;

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind("Sword")
            .bind("sword")
            .bind(1.0)
            .bind(&json!({}))
            .bind(&vec!["damage".to_string()])
            .bind(0)
            .bind(2)
            .bind(0)
            .bind(0)
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1,$2,$3,'prefix',0)",
            )
            .bind(bp_id)
            .bind(affix_id)
            .bind(1.0)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_id),
                axum::extract::Query(DeleteAffixQuery { force: false }),
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

            let affix_count: i64 = sqlx::query("SELECT COUNT(*) FROM affixes WHERE id = $1")
                .bind(affix_id)
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
            assert_eq!(affix_count, 1);

            sqlx::query("DELETE FROM blueprint_affixes WHERE blueprint_id = $1")
                .bind(bp_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp_id)
                .execute(&pool)
                .await
                .unwrap();
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
        async fn test_force_delete_affix_removes_from_pools_and_adjusts() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix_a = create_test_affix(&pool, client_id, "Fire", "prefix").await;
            let affix_b = create_test_affix(&pool, client_id, "Ice", "prefix").await;

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind("Sword")
            .bind("sword")
            .bind(1.0)
            .bind(&json!({}))
            .bind(&vec!["damage".to_string()])
            .bind(1)
            .bind(3)
            .bind(0)
            .bind(0)
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1,$2,$3,'prefix',0)",
            )
            .bind(bp_id)
            .bind(affix_a)
            .bind(1.0)
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1,$2,$3,'prefix',1)",
            )
            .bind(bp_id)
            .bind(affix_b)
            .bind(1.0)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = delete_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_a),
                axum::extract::Query(DeleteAffixQuery { force: true }),
            )
            .await;

            assert!(result.is_ok(), "force delete failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0, json!({"deleted": true}));

            let affix_count: i64 = sqlx::query("SELECT COUNT(*) FROM affixes WHERE id = $1")
                .bind(affix_a)
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
            assert_eq!(affix_count, 0);

            let ba_count: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprint_affixes WHERE affix_id = $1",
            )
            .bind(affix_a)
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
            assert_eq!(ba_count, 0);

            let bp_row = sqlx::query(
                "SELECT min_prefixes, max_prefixes FROM blueprints WHERE id = $1",
            )
            .bind(bp_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            let new_min: i32 = bp_row.get("min_prefixes");
            let new_max: i32 = bp_row.get("max_prefixes");
            assert_eq!(new_min, 1);
            assert_eq!(new_max, 1);

            sqlx::query("DELETE FROM blueprint_affixes WHERE blueprint_id = $1")
                .bind(bp_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM affixes WHERE id = ANY($1)")
                .bind(&[affix_a, affix_b])
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
        async fn test_force_delete_affix_adjusts_max_then_min() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix = create_test_affix(&pool, client_id, "Fire", "prefix").await;

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind("Sword")
            .bind("sword")
            .bind(1.0)
            .bind(&json!({}))
            .bind(&vec!["damage".to_string()])
            .bind(0)
            .bind(1)
            .bind(0)
            .bind(0)
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1,$2,$3,'prefix',0)",
            )
            .bind(bp_id)
            .bind(affix)
            .bind(1.0)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = delete_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix),
                axum::extract::Query(DeleteAffixQuery { force: true }),
            )
            .await;

            assert!(result.is_ok(), "force delete failed: {:?}", result.err());

            let bp_row = sqlx::query(
                "SELECT min_prefixes, max_prefixes FROM blueprints WHERE id = $1",
            )
            .bind(bp_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            let new_min: i32 = bp_row.get("min_prefixes");
            let new_max: i32 = bp_row.get("max_prefixes");
            assert_eq!(new_max, 0, "max should be downsized to 0 (no prefixes left)");
            assert_eq!(new_min, 0, "min should be downsized to 0 (no prefixes left)");

            sqlx::query("DELETE FROM blueprint_affixes WHERE blueprint_id = $1")
                .bind(bp_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp_id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM affixes WHERE id = ANY($1)")
                .bind(&[affix])
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
        async fn test_delete_nonexistent_affix_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(Uuid::new_v4()),
                axum::extract::Query(DeleteAffixQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains("Affix not found"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_affix_wrong_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;
            let affix_id = create_test_affix(&pool, client_a, "Fire", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_b, Permission::Delete);

            let result = delete_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_id),
                axum::extract::Query(DeleteAffixQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains("Affix not found"));

            let count: i64 = sqlx::query("SELECT COUNT(*) FROM affixes WHERE id = $1")
                .bind(affix_id)
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
            assert_eq!(count, 1);

            sqlx::query("DELETE FROM affixes WHERE id = $1")
                .bind(affix_id)
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

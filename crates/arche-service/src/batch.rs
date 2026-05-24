use arche_types::batch::{BatchEditRequest, BatchEditResponse};
use axum::extract::State;
use axum::Json;
use serde_json::Value;
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

use crate::auth::permission::CurrentUser;
use crate::error::{FieldError, ProblemResponse};

pub async fn batch_edit_blueprints(
    State(state): State<crate::AppState>,
    CurrentUser(_user): CurrentUser,
    Json(req): Json<BatchEditRequest>,
) -> Result<Json<BatchEditResponse>, ProblemResponse> {
    if req.blueprint_ids.is_empty() {
        return Err(ProblemResponse::validation_error(
            "blueprint_ids must not be empty",
            vec![FieldError {
                path: "blueprint_ids".into(),
                message: "must provide at least one blueprint ID".into(),
            }],
        ));
    }

    if req.attributes.is_empty() {
        return Err(ProblemResponse::validation_error(
            "attributes must not be empty",
            vec![FieldError {
                path: "attributes".into(),
                message: "must provide at least one attribute to edit".into(),
            }],
        ));
    }

    let rows = sqlx::query(
        "SELECT id, attributes FROM blueprints WHERE id = ANY($1)",
    )
    .bind(&req.blueprint_ids)
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "batch edit: fetch blueprints query failed");
        ProblemResponse::unprocessable_entity("Failed to fetch blueprints for batch edit")
    })?;

    let found_ids: HashSet<Uuid> = rows.iter().map(|r| r.get("id")).collect();
    let missing: Vec<String> = req
        .blueprint_ids
        .iter()
        .filter(|id| !found_ids.contains(id))
        .map(|id| id.to_string())
        .collect();

    if !missing.is_empty() {
        return Err(ProblemResponse::validation_error(
            format!("Blueprints not found: {}", missing.join(", ")),
            vec![FieldError {
                path: "blueprint_ids".into(),
                message: format!("unknown blueprint ids: {}", missing.join(", ")),
            }],
        ));
    }

    let mut to_update: Vec<(Uuid, Value)> = Vec::new();

    for row in &rows {
        let bp_id: Uuid = row.get("id");
        let existing_attrs: Value = row.get("attributes");

        let existing_obj = match existing_attrs.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        let mut has_all_keys = true;

        for (key, new_attr) in &req.attributes {
            match existing_obj.get(key) {
                None => {
                    has_all_keys = false;
                    break;
                }
                Some(existing) => {
                    let existing_type = existing
                        .get("value_type")
                        .and_then(|v| v.as_str());
                    let new_type = new_attr
                        .get("value_type")
                        .and_then(|v| v.as_str());

                    match (existing_type, new_type) {
                        (Some(et), Some(nt)) if et == nt => {}
                        _ => {
                            let detail = if existing_type.is_none() {
                                format!(
                                    "Blueprint {bp_id} attribute '{key}' is a $ref_id reference and cannot be batch-edited"
                                )
                            } else {
                                format!(
                                    "Blueprint {bp_id} attribute '{key}' type mismatch: existing {existing_type:?} != {new_type:?}"
                                )
                            };
                            return Err(ProblemResponse::validation_error(
                                detail,
                                vec![FieldError {
                                    path: format!("attributes.{key}"),
                                    message: "value_type mismatch".into(),
                                }],
                            ));
                        }
                    }
                }
            }
        }

        if has_all_keys {
            let merged = Value::Object(
                req.attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            );
            to_update.push((bp_id, merged));
        }
    }

    if to_update.is_empty() {
        return Ok(Json(BatchEditResponse { updated_count: 0 }));
    }

    let mut tx = state.pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "batch edit: begin transaction failed");
        ProblemResponse::unprocessable_entity("Failed to begin transaction for batch edit")
    })?;

    for (bp_id, merged) in &to_update {
        sqlx::query(
            "UPDATE blueprints SET attributes = attributes || $1::jsonb, updated_at = now() WHERE id = $2",
        )
        .bind(merged)
        .bind(bp_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, bp_id = %bp_id, "batch edit: update query failed");
            ProblemResponse::unprocessable_entity("Failed to update blueprint in batch edit")
        })?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "batch edit: commit transaction failed");
        ProblemResponse::unprocessable_entity("Failed to commit batch edit transaction")
    })?;

    Ok(Json(BatchEditResponse {
        updated_count: to_update.len() as i64,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arche_types::batch::BatchEditRequest;
    use serde_json::json;

    fn make_attrs_json(value_type: &str, extra: Value) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("value_type".into(), Value::String(value_type.into()));
        if let Value::Object(extra_map) = extra {
            map.extend(extra_map);
        }
        Value::Object(map)
    }

    #[test]
    fn test_batch_edit_request_deserialization() {
        let json = json!({
            "blueprintIds": [
                "550e8400-e29b-41d4-a716-446655440000",
                "550e8400-e29b-41d4-a716-446655440001"
            ],
            "attributes": {
                "damage": {
                    "value_type": "range",
                    "min": 5.0,
                    "max": 20.0
                }
            }
        });

        let req: BatchEditRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.blueprint_ids.len(), 2);
        assert_eq!(req.attributes.len(), 1);
        assert!(req.attributes.contains_key("damage"));

        let damage = &req.attributes["damage"];
        assert_eq!(damage["value_type"], "range");
        assert_eq!(damage["min"], 5.0);
        assert_eq!(damage["max"], 20.0);
    }

    #[test]
    fn test_batch_edit_request_multiple_attributes() {
        let json = json!({
            "blueprintIds": ["550e8400-e29b-41d4-a716-446655440000"],
            "attributes": {
                "damage": {"value_type": "range", "min": 1.0, "max": 10.0},
                "speed": {"value_type": "single", "value": 5.0}
            }
        });

        let req: BatchEditRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.attributes.len(), 2);
    }

    #[test]
    fn test_batch_edit_response_serialization() {
        let resp = BatchEditResponse { updated_count: 3 };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value, json!({"updatedCount": 3}));
    }

    #[test]
    fn test_batch_edit_response_round_trip() {
        let resp = BatchEditResponse { updated_count: 5 };
        let json = serde_json::to_value(&resp).unwrap();
        let deserialized: BatchEditResponse = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.updated_count, 5);
    }

    #[test]
    fn test_make_attrs_helper_range() {
        let attrs = make_attrs_json(
            "range",
            json!({"min": 1.0, "max": 10.0}),
        );
        assert_eq!(attrs["value_type"], "range");
        assert_eq!(attrs["min"], 1.0);
        assert_eq!(attrs["max"], 10.0);
    }

    #[test]
    fn test_make_attrs_helper_single() {
        let attrs = make_attrs_json("single", json!({"value": 42.0}));
        assert_eq!(attrs["value_type"], "single");
        assert_eq!(attrs["value"], 42.0);
    }

    #[test]
    fn test_make_attrs_helper_enum() {
        let attrs = make_attrs_json(
            "enum",
            json!({"values": ["a", "b"]}),
        );
        assert_eq!(attrs["value_type"], "enum");
        assert_eq!(attrs["values"].as_array().unwrap().len(), 2);
    }

    mod db_tests {
        use super::*;
        use axum::body::Body;
        use axum::http::{Method, Request, StatusCode};
        use serde_json::json;
        use sqlx::PgPool;
        use std::collections::HashMap;
        use tower::ServiceExt;
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

        fn build_test_state(pool: PgPool) -> crate::AppState {
            crate::AppState {
                cache: std::sync::Arc::new(tokio::sync::RwLock::new(
                    crate::cache::Cache::new(
                        HashMap::new(),
                        HashMap::new(),
                        HashMap::new(),
                        HashMap::new(),
                        HashMap::new(),
                    ),
                )),
                pool: std::sync::Arc::new(pool),
                redis: None,
                import_staging: std::sync::Arc::new(
                    crate::import::ImportStaging::new(),
                ),
            }
        }

        fn build_router(state: crate::AppState) -> axum::Router {
            axum::Router::new()
                .route(
                    "/api/blueprints/batch/edit",
                    axum::routing::post(super::batch_edit_blueprints),
                )
                .with_state(state)
        }

        async fn insert_test_client(pool: &PgPool) -> Uuid {
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO clients (id, name) VALUES ($1, $2)")
                .bind(id)
                .bind("test-client")
                .execute(pool)
                .await
                .unwrap();
            id
        }

        async fn insert_test_blueprint(
            pool: &PgPool,
            client_id: Uuid,
            name: &str,
            attributes: serde_json::Value,
            attribute_order: &[&str],
        ) -> Uuid {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order) \
                 VALUES ($1, $2, $3, 'sword', 1.0, $4, $5)",
            )
            .bind(id)
            .bind(client_id)
            .bind(name)
            .bind(attributes)
            .bind(
                attribute_order
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            )
            .execute(pool)
            .await
            .unwrap();
            id
        }

        fn make_super_admin_key() -> crate::auth::AuthenticatedKey {
            crate::auth::AuthenticatedKey {
                id: Uuid::new_v4(),
                name: "test-super-admin".into(),
                client_id: None,
                permissions: vec![arche_types::Permission::Write],
                is_super: true,
            }
        }

        fn make_authenticated_request(
            body: serde_json::Value,
        ) -> Request<Body> {
            let mut req = Request::builder()
                .method(Method::POST)
                .uri("/api/blueprints/batch/edit")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();
            req.extensions_mut().insert(make_super_admin_key());
            req
        }

        #[tokio::test]
        async fn test_batch_edit_updates_all_matching_blueprints() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;

            let attrs_a = json!({"damage": {"value_type": "range", "min": 1.0, "max": 10.0}});
            let attrs_b = json!({"damage": {"value_type": "range", "min": 5.0, "max": 15.0}});
            let attrs_c = json!({"damage": {"value_type": "range", "min": 10.0, "max": 20.0}});

            let bp_a =
                insert_test_blueprint(&pool, client_id, "Sword A", attrs_a, &["damage"]).await;
            let bp_b =
                insert_test_blueprint(&pool, client_id, "Sword B", attrs_b, &["damage"]).await;
            let bp_c =
                insert_test_blueprint(&pool, client_id, "Sword C", attrs_c, &["damage"]).await;

            let body = json!({
                "blueprintIds": [bp_a, bp_b, bp_c],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool.clone());
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let response_body =
                axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
            let parsed: BatchEditResponse = serde_json::from_slice(&response_body).unwrap();
            assert_eq!(parsed.updated_count, 3);

            for bp_id in &[bp_a, bp_b, bp_c] {
                let updated: serde_json::Value =
                    sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                        .bind(bp_id)
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                assert_eq!(updated["damage"]["min"], 3.0);
                assert_eq!(updated["damage"]["max"], 7.0);
            }
        }

        #[tokio::test]
        async fn test_batch_edit_skips_blueprint_without_key() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;

            let attrs_with = json!({"damage": {"value_type": "range", "min": 1.0, "max": 10.0}});
            let attrs_without = json!({"speed": {"value_type": "single", "value": 5.0}});

            let bp_a =
                insert_test_blueprint(&pool, client_id, "Sword A", attrs_with, &["damage"]).await;
            let bp_b =
                insert_test_blueprint(&pool, client_id, "Sword B", attrs_without, &["speed"]).await;

            let body = json!({
                "blueprintIds": [bp_a, bp_b],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool.clone());
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let response_body =
                axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
            let parsed: BatchEditResponse = serde_json::from_slice(&response_body).unwrap();
            assert_eq!(parsed.updated_count, 1);

            let b_attrs: serde_json::Value =
                sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                    .bind(bp_b)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(b_attrs["speed"]["value_type"], "single");
            assert_eq!(b_attrs["speed"]["value"], 5.0);
        }

        #[tokio::test]
        async fn test_batch_edit_type_mismatch_returns_422() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;

            let attrs = json!({"damage": {"value_type": "single", "value": 10.0}});
            let bp =
                insert_test_blueprint(&pool, client_id, "Sword A", attrs, &["damage"]).await;

            let body = json!({
                "blueprintIds": [bp],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool.clone());
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

            let original: serde_json::Value =
                sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                    .bind(bp)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(original["damage"]["value_type"], "single");
            assert_eq!(original["damage"]["value"], 10.0);
        }

        #[tokio::test]
        async fn test_batch_edit_preserves_unrelated_attributes() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;

            let attrs = json!({
                "damage": {"value_type": "range", "min": 1.0, "max": 10.0},
                "speed": {"value_type": "single", "value": 5.0}
            });
            let bp = insert_test_blueprint(
                &pool,
                client_id,
                "Sword A",
                attrs,
                &["damage", "speed"],
            )
            .await;

            let body = json!({
                "blueprintIds": [bp],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool.clone());
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let updated: serde_json::Value =
                sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                    .bind(bp)
                    .fetch_one(&pool)
                    .await
                    .unwrap();

            assert_eq!(updated["damage"]["min"], 3.0);
            assert_eq!(updated["damage"]["max"], 7.0);
            assert_eq!(updated["speed"]["value_type"], "single");
            assert_eq!(updated["speed"]["value"], 5.0);
        }

        #[tokio::test]
        async fn test_batch_edit_preserves_attribute_order() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;

            let attrs = json!({
                "damage": {"value_type": "range", "min": 1.0, "max": 10.0},
                "speed": {"value_type": "single", "value": 5.0}
            });
            let bp = insert_test_blueprint(
                &pool,
                client_id,
                "Sword A",
                attrs,
                &["damage", "speed"],
            )
            .await;

            let body = json!({
                "blueprintIds": [bp],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool.clone());
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let order: Vec<String> =
                sqlx::query_scalar("SELECT attribute_order FROM blueprints WHERE id = $1")
                    .bind(bp)
                    .fetch_one(&pool)
                    .await
                    .unwrap();

            assert_eq!(order, vec!["damage", "speed"]);
        }

        #[tokio::test]
        async fn test_batch_edit_type_mismatch_rolls_back_all() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;

            let attrs_a = json!({"damage": {"value_type": "range", "min": 1.0, "max": 10.0}});
            let attrs_b = json!({"damage": {"value_type": "single", "value": 10.0}});

            let bp_a =
                insert_test_blueprint(&pool, client_id, "Sword A", attrs_a, &["damage"]).await;
            let bp_b =
                insert_test_blueprint(&pool, client_id, "Sword B", attrs_b, &["damage"]).await;

            let body = json!({
                "blueprintIds": [bp_a, bp_b],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool.clone());
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

            let updated_a: serde_json::Value =
                sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                    .bind(bp_a)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let updated_b: serde_json::Value =
                sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                    .bind(bp_b)
                    .fetch_one(&pool)
                    .await
                    .unwrap();

            assert_eq!(updated_a["damage"]["min"], 1.0);
            assert_eq!(updated_a["damage"]["max"], 10.0);
            assert_eq!(updated_b["damage"]["value_type"], "single");
            assert_eq!(updated_b["damage"]["value"], 10.0);
        }

        #[tokio::test]
        async fn test_batch_edit_unknown_blueprint_ids_error() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;
            let attrs = json!({"damage": {"value_type": "range", "min": 1.0, "max": 10.0}});
            let bp =
                insert_test_blueprint(&pool, client_id, "Sword A", attrs, &["damage"]).await;
            let unknown_id = Uuid::new_v4();

            let body = json!({
                "blueprintIds": [bp, unknown_id],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool.clone());
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let original: serde_json::Value =
                sqlx::query_scalar("SELECT attributes FROM blueprints WHERE id = $1")
                    .bind(bp)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(original["damage"]["min"], 1.0);
        }

        #[tokio::test]
        async fn test_batch_edit_empty_blueprint_ids_error() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let body = json!({
                "blueprintIds": [],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool);
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn test_batch_edit_empty_attributes_error() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;
            let attrs = json!({"damage": {"value_type": "range", "min": 1.0, "max": 10.0}});
            let bp =
                insert_test_blueprint(&pool, client_id, "Sword A", attrs, &["damage"]).await;

            let body = json!({
                "blueprintIds": [bp],
                "attributes": {}
            });

            let state = build_test_state(pool);
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn test_batch_edit_all_blueprints_missing_key_returns_zero() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = insert_test_client(&pool).await;
            let attrs = json!({"speed": {"value_type": "single", "value": 5.0}});
            let bp =
                insert_test_blueprint(&pool, client_id, "Sword A", attrs, &["speed"]).await;

            let body = json!({
                "blueprintIds": [bp],
                "attributes": {
                    "damage": {"value_type": "range", "min": 3.0, "max": 7.0}
                }
            });

            let state = build_test_state(pool);
            let router = build_router(state);

            let response = router
                .oneshot(make_authenticated_request(body))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let response_body =
                axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
            let parsed: BatchEditResponse = serde_json::from_slice(&response_body).unwrap();
            assert_eq!(parsed.updated_count, 0);
        }
    }
}
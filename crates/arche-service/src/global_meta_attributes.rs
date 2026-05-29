use arche_types::attribute::AttributePayload;
use arche_types::common::{GlobalMetaAttributeListQuery, PaginatedResponse, WriteClientQuery};
use arche_types::crud::{CreateGlobalMetaAttributeRequest, UpdateGlobalMetaAttributeRequest};
use arche_types::{GlobalMetaAttribute, ValueType};
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::postgres::PgRow;
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::audit_log::record_audit;
use crate::auth::permission::CurrentUser;
use crate::error::{FieldError, ProblemResponse, ReferenceInfo};

impl crate::pagination::HasId for GlobalMetaAttribute {
    fn id(&self) -> Uuid {
        self.id
    }
}

const GMA_COLUMNS: &str =
    "id, client_id, name, description, value_type::text AS value_type, payload, created_at, updated_at";

fn value_type_from_payload(payload: &AttributePayload) -> &'static str {
    match payload {
        AttributePayload::Single { .. } => "single",
        AttributePayload::Enum { .. } => "enum",
        AttributePayload::Range { .. } => "range",
        AttributePayload::String { .. } => "string",
        AttributePayload::Boolean { .. } => "boolean",
    }
}

fn value_type_from_pg(s: &str) -> Option<ValueType> {
    match s {
        "single" => Some(ValueType::Single),
        "enum" => Some(ValueType::Enum),
        "range" => Some(ValueType::Range),
        "string" => Some(ValueType::String),
        "boolean" => Some(ValueType::Boolean),
        _ => None,
    }
}

fn resolve_client_id(
    user: &crate::auth::AuthenticatedKey,
    query_client_id: Option<Uuid>,
) -> Result<Option<Uuid>, ProblemResponse> {
    if user.is_super {
        Ok(query_client_id)
    } else {
        user.client_id.map(Some).ok_or_else(|| {
            ProblemResponse::forbidden(
                "Access denied: key is not associated with any client",
            )
        })
    }
}

fn resolve_client_id_for_write(
    user: &crate::auth::AuthenticatedKey,
    client_id: Option<Uuid>,
) -> Result<Uuid, ProblemResponse> {
    if user.is_super {
        client_id.ok_or_else(|| {
            ProblemResponse::forbidden(
                "Super admin must specify a client context for write operations",
            )
        })
    } else {
        user.client_id.ok_or_else(|| {
            ProblemResponse::forbidden(
                "Access denied: key is not associated with any client",
            )
        })
    }
}

fn strip_value_type_from_payload(mut payload: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = payload.as_object_mut() {
        obj.remove("value_type");
    }
    payload
}

fn prepare_and_validate_payload(
    req: &CreateGlobalMetaAttributeRequest,
) -> Result<(ValueType, &'static str, serde_json::Value), ProblemResponse> {
    let value_type_str = value_type_from_payload(&req.payload);
    let payload_json =
        serde_json::to_value(&req.payload).map_err(|e| {
            ProblemResponse::unprocessable_entity(format!("Invalid payload: {e}"))
        })?;
    let clean_payload = strip_value_type_from_payload(payload_json);

    let value_type = value_type_from_pg(value_type_str).unwrap_or(ValueType::String);
    let validation_errors =
        arche_types::validation::validate_attribute_payload(&value_type, &clean_payload);
    if !validation_errors.is_empty() {
        let errors: Vec<FieldError> = validation_errors
            .into_iter()
            .map(|ve| FieldError {
                path: format!("payload.{}", ve.path),
                message: ve.message,
            })
            .collect();
        return Err(ProblemResponse::validation_error(
            "Global meta attribute validation failed",
            errors,
        ));
    }

    Ok((value_type, value_type_str, clean_payload))
}

pub async fn list_global_meta_attributes(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<GlobalMetaAttributeListQuery>,
) -> Result<Json<PaginatedResponse<GlobalMetaAttribute>>, ProblemResponse> {
    let client_id_filter = resolve_client_id(&user, query.client_id)?;

    let params = crate::pagination::PaginationParams {
        cursor: query.cursor.clone(),
        limit: query.limit,
        page: query.page,
        per_page: query.per_page,
    };

    let mode = params.mode().map_err(|e| {
        ProblemResponse::validation_error(e.to_string(), vec![])
    })?;

    match mode {
        crate::pagination::PaginationMode::Cursor { after, limit } => {
            let fetch_limit = limit + 1;

            let mut builder = QueryBuilder::new(
                format!("SELECT {GMA_COLUMNS} FROM global_meta_attributes"),
            );
            apply_gma_filters(
                &mut builder,
                client_id_filter,
                query.value_type.as_deref(),
                query.search.as_deref(),
                after,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(fetch_limit);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "gma: list cursor query failed");
                ProblemResponse::unprocessable_entity("Failed to list global meta attributes")
            })?;

            let gmas: Vec<GlobalMetaAttribute> = rows.iter().map(row_to_gma).collect();
            let response = crate::pagination::paginate_cursor(gmas, limit);
            Ok(Json(response))
        }
        crate::pagination::PaginationMode::Offset {
            per_page,
            offset,
            ..
        } => {
            let mut count_builder =
                QueryBuilder::new("SELECT COUNT(*) FROM global_meta_attributes");
            apply_gma_filters(
                &mut count_builder,
                client_id_filter,
                query.value_type.as_deref(),
                query.search.as_deref(),
                None,
            );

            let total: i64 = count_builder
                .build()
                .fetch_one(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "gma: list count query failed");
                    ProblemResponse::unprocessable_entity("Failed to count global meta attributes")
                })?
                .get(0);

            let mut builder = QueryBuilder::new(
                format!("SELECT {GMA_COLUMNS} FROM global_meta_attributes"),
            );
            apply_gma_filters(
                &mut builder,
                client_id_filter,
                query.value_type.as_deref(),
                query.search.as_deref(),
                None,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(per_page);
            builder.push(" OFFSET ");
            builder.push_bind(offset);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "gma: list offset query failed");
                ProblemResponse::unprocessable_entity("Failed to list global meta attributes")
            })?;

            let gmas: Vec<GlobalMetaAttribute> = rows.iter().map(row_to_gma).collect();
            let response = crate::pagination::paginate_offset(gmas, total);
            Ok(Json(response))
        }
    }
}

pub async fn get_global_meta_attribute(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GlobalMetaAttribute>, ProblemResponse> {
    let row = sqlx::query(&format!(
        "SELECT {GMA_COLUMNS} FROM global_meta_attributes WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "gma: get query failed");
        ProblemResponse::unprocessable_entity("Failed to get global meta attribute")
    })?
    .ok_or_else(|| {
        ProblemResponse::not_found(format!("Global meta attribute not found: {id}"))
    })?;

    let gma = row_to_gma(&row);

    user.require_client_access(gma.client_id)?;

    Ok(Json(gma))
}

pub async fn create_global_meta_attribute(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<WriteClientQuery>,
    Json(req): Json<CreateGlobalMetaAttributeRequest>,
) -> Result<Json<GlobalMetaAttribute>, ProblemResponse> {
    let client_id = resolve_client_id_for_write(&user, query.client_id)?;

    let (value_type, value_type_str, clean_payload) = prepare_and_validate_payload(&req)?;

    let row = sqlx::query(
        "INSERT INTO global_meta_attributes (client_id, name, description, value_type, payload) \
         VALUES ($1, $2, $3, $4::value_type, $5) \
         RETURNING id, created_at, updated_at",
    )
    .bind(client_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(value_type_str)
    .bind(&clean_payload)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.constraint() == Some("global_meta_attributes_client_id_name_key") {
                return ProblemResponse::conflict(format!(
                    "A global meta attribute with name '{}' already exists for this client",
                    req.name
                ));
            }
        }
        tracing::error!(error = %e, "gma: insert failed");
        ProblemResponse::validation_error(
            format!("Failed to create global meta attribute: {e}"),
            vec![],
        )
    })?;

    let id: Uuid = row.get("id");
    let created_at = row.get("created_at");
    let updated_at = row.get("updated_at");

    let gma = GlobalMetaAttribute {
        id,
        client_id,
        name: req.name,
        description: req.description,
        value_type,
        payload: clean_payload,
        created_at,
        updated_at,
    };

    state.reload_client_cache(client_id).await;

    Ok(Json(gma))
}

pub async fn update_global_meta_attribute(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGlobalMetaAttributeRequest>,
) -> Result<Json<GlobalMetaAttribute>, ProblemResponse> {
    let existing = sqlx::query(&format!(
        "SELECT {GMA_COLUMNS} FROM global_meta_attributes WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "gma: get for update failed");
        ProblemResponse::unprocessable_entity("Failed to get global meta attribute for update")
    })?
    .ok_or_else(|| {
        ProblemResponse::not_found(format!("Global meta attribute not found: {id}"))
    })?;

    let existing_gma = row_to_gma(&existing);

    user.require_client_access(existing_gma.client_id)?;

    let client_id = existing_gma.client_id;

    let (value_type, value_type_str, clean_payload) = prepare_and_validate_payload(&req)?;

    let row = sqlx::query(
        "UPDATE global_meta_attributes \
         SET name = $1, description = $2, value_type = $3::value_type, payload = $4, updated_at = now() \
         WHERE id = $5 AND client_id = $6 \
         RETURNING created_at, updated_at",
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(value_type_str)
    .bind(&clean_payload)
    .bind(id)
    .bind(client_id)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.constraint() == Some("global_meta_attributes_client_id_name_key") {
                return ProblemResponse::conflict(format!(
                    "A global meta attribute with name '{}' already exists for this client",
                    req.name
                ));
            }
        }
        tracing::error!(error = %e, "gma: update failed");
        ProblemResponse::validation_error(
            format!("Failed to update global meta attribute: {e}"),
            vec![],
        )
    })?;

    let created_at = row.get("created_at");
    let updated_at = row.get("updated_at");

    let gma = GlobalMetaAttribute {
        id,
        client_id,
        name: req.name,
        description: req.description,
        value_type,
        payload: clean_payload,
        created_at,
        updated_at,
    };

    state.reload_client_cache(client_id).await;

    Ok(Json(gma))
}

#[derive(Debug, Deserialize)]
pub struct DeleteGlobalMetaAttributeQuery {
    #[serde(default)]
    pub force: bool,
}

fn apply_gma_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    client_id: Option<Uuid>,
    value_type: Option<&'a str>,
    search: Option<&'a str>,
    cursor: Option<Uuid>,
) {
    let mut first = true;

    if let Some(cid) = client_id {
        builder.push(" WHERE client_id = ");
        builder.push_bind(cid);
        first = false;
    }

    if let Some(vt) = value_type {
        if first {
            builder.push(" WHERE ");
            first = false;
        } else {
            builder.push(" AND ");
        }
        builder.push("value_type = ");
        builder.push_bind(vt);
    }

    if let Some(s) = search {
        if first {
            builder.push(" WHERE ");
            first = false;
        } else {
            builder.push(" AND ");
        }
        builder.push("name ILIKE ");
        builder.push_bind(format!("%{s}%"));
    }

    if let Some(c) = cursor {
        if first {
            builder.push(" WHERE ");
        } else {
            builder.push(" AND ");
        }
        builder.push("id > ");
        builder.push_bind(c);
    }
}

fn row_to_gma(row: &PgRow) -> GlobalMetaAttribute {
    let value_type_str: String = row.get("value_type");
    GlobalMetaAttribute {
        id: row.get("id"),
        client_id: row.get("client_id"),
        name: row.get("name"),
        description: row.get("description"),
        value_type: value_type_from_pg(&value_type_str).unwrap_or(ValueType::String),
        payload: row.get("payload"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
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

            record_audit(
                &mut *tx,
                &user,
                Some(gma_client_id),
                "blueprint",
                bp_id,
                "adjusted",
                Some(serde_json::json!({"attribute_key": &attr_key, "removed_ref_id": id.to_string()})),
                Some(serde_json::json!({"attribute_key": &attr_key, "removed": true})),
            )
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

            record_audit(
                &mut *tx,
                &user,
                Some(gma_client_id),
                "affix",
                affix_id,
                "adjusted",
                Some(serde_json::json!({"removed_ref_id": id.to_string()})),
                Some(serde_json::json!({"attribute": {}})),
            )
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

    state.reload_client_cache(gma_client_id).await;

    Ok(Json(serde_json::json!({"deleted": true})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arche_types::attribute::DistributionConfig;
    use crate::pagination::HasId;

    #[test]
    fn test_has_id_returns_id() {
        let id = uuid::Uuid::new_v4();
        let gma = GlobalMetaAttribute {
            id,
            client_id: uuid::Uuid::new_v4(),
            name: "test".into(),
            description: None,
            value_type: ValueType::String,
            payload: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(gma.id(), id);
    }

    #[test]
    fn test_value_type_from_payload_single() {
        let p = AttributePayload::Single {
            value: 10.0,
            distribution: None,
        };
        assert_eq!(value_type_from_payload(&p), "single");
    }

    #[test]
    fn test_value_type_from_payload_enum() {
        let p = AttributePayload::Enum {
            values: vec!["a".into()],
        };
        assert_eq!(value_type_from_payload(&p), "enum");
    }

    #[test]
    fn test_value_type_from_payload_range() {
        let p = AttributePayload::Range {
            min: 1.0,
            max: 10.0,
            distribution: None,
        };
        assert_eq!(value_type_from_payload(&p), "range");
    }

    #[test]
    fn test_value_type_from_payload_string() {
        let p = AttributePayload::String {
            min_length: None,
            max_length: None,
        };
        assert_eq!(value_type_from_payload(&p), "string");
    }

    #[test]
    fn test_value_type_from_payload_boolean() {
        let p = AttributePayload::Boolean { value: true };
        assert_eq!(value_type_from_payload(&p), "boolean");
    }

    #[test]
    fn test_value_type_from_pg_known() {
        assert_eq!(value_type_from_pg("single"), Some(ValueType::Single));
        assert_eq!(value_type_from_pg("enum"), Some(ValueType::Enum));
        assert_eq!(value_type_from_pg("range"), Some(ValueType::Range));
        assert_eq!(value_type_from_pg("string"), Some(ValueType::String));
        assert_eq!(value_type_from_pg("boolean"), Some(ValueType::Boolean));
    }

    #[test]
    fn test_value_type_from_pg_unknown() {
        assert_eq!(value_type_from_pg("unknown"), None);
    }

    #[test]
    fn test_strip_value_type_removes_key() {
        let payload = serde_json::json!({"value_type": "enum", "values": ["a", "b"]});
        let stripped = strip_value_type_from_payload(payload);
        assert_eq!(
            stripped,
            serde_json::json!({"values": ["a", "b"]})
        );
    }

    #[test]
    fn test_strip_value_type_no_value_type_key() {
        let payload = serde_json::json!({"min": 1.0, "max": 10.0});
        let stripped = strip_value_type_from_payload(payload);
        assert_eq!(
            stripped,
            serde_json::json!({"min": 1.0, "max": 10.0})
        );
    }

    #[test]
    fn test_strip_value_type_non_object() {
        let payload = serde_json::json!("not_an_object");
        let stripped = strip_value_type_from_payload(payload);
        assert_eq!(stripped, serde_json::json!("not_an_object"));
    }

    #[test]
    fn test_resolve_client_id_super_no_filter() {
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "super".into(),
            client_id: None,
            permissions: vec![],
            is_super: true,
        };
        assert_eq!(resolve_client_id(&key, None).unwrap(), None);
    }

    #[test]
    fn test_resolve_client_id_super_with_filter() {
        let cid = uuid::Uuid::new_v4();
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "super".into(),
            client_id: None,
            permissions: vec![],
            is_super: true,
        };
        assert_eq!(resolve_client_id(&key, Some(cid)).unwrap(), Some(cid));
    }

    #[test]
    fn test_resolve_client_id_regular_key() {
        let cid = uuid::Uuid::new_v4();
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "test".into(),
            client_id: Some(cid),
            permissions: vec![arche_types::Permission::Read],
            is_super: false,
        };
        assert_eq!(resolve_client_id(&key, None).unwrap(), Some(cid));
    }

    #[test]
    fn test_resolve_client_id_regular_key_no_client() {
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "test".into(),
            client_id: None,
            permissions: vec![arche_types::Permission::Read],
            is_super: false,
        };
        let err = resolve_client_id(&key, None).unwrap_err();
        assert_eq!(err.status, 403);
    }

    #[test]
    fn test_resolve_client_id_for_write_regular_key() {
        let cid = uuid::Uuid::new_v4();
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "test".into(),
            client_id: Some(cid),
            permissions: vec![arche_types::Permission::Write],
            is_super: false,
        };
        assert_eq!(resolve_client_id_for_write(&key, None).unwrap(), cid);
    }

    #[test]
    fn test_resolve_client_id_for_write_no_client() {
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "test".into(),
            client_id: None,
            permissions: vec![arche_types::Permission::Write],
            is_super: false,
        };
        let err = resolve_client_id_for_write(&key, None).unwrap_err();
        assert_eq!(err.status, 403);
    }

    #[test]
    fn test_resolve_client_id_for_write_super_admin_with_id() {
        let cid = uuid::Uuid::new_v4();
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "super".into(),
            client_id: None,
            permissions: vec![],
            is_super: true,
        };
        assert_eq!(resolve_client_id_for_write(&key, Some(cid)).unwrap(), cid);
    }

    #[test]
    fn test_resolve_client_id_for_write_super_admin_no_client_id() {
        let key = crate::auth::AuthenticatedKey {
            id: uuid::Uuid::nil(),
            name: "super".into(),
            client_id: None,
            permissions: vec![],
            is_super: true,
        };
        let err = resolve_client_id_for_write(&key, None).unwrap_err();
        assert_eq!(err.status, 403);
    }

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

    #[test]
    fn test_gma_list_query_deserialization() {
        let query: GlobalMetaAttributeListQuery =
            serde_json::from_str(r#"{"value_type":"enum","search":"rarity","limit":10}"#).unwrap();
        assert_eq!(query.value_type, Some("enum".into()));
        assert_eq!(query.search, Some("rarity".into()));
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_gma_list_query_empty() {
        let query: GlobalMetaAttributeListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.cursor, None);
        assert_eq!(query.limit, None);
        assert_eq!(query.page, None);
        assert_eq!(query.per_page, None);
        assert_eq!(query.client_id, None);
        assert_eq!(query.value_type, None);
        assert_eq!(query.search, None);
    }

    #[test]
    fn test_create_gma_request_serialization() {
        let req = CreateGlobalMetaAttributeRequest {
            name: "rarity".into(),
            description: Some("Quality tier".into()),
            payload: AttributePayload::Range {
                min: 1.0,
                max: 10.0,
                distribution: Some(DistributionConfig::Normal { std_dev: 2.0 }),
            },
        };
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(value.get("name").unwrap(), "rarity");
        assert_eq!(value.get("value_type").unwrap(), "range");
        assert_eq!(value.get("min").unwrap(), 1.0);
        assert_eq!(value.get("max").unwrap(), 10.0);
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
                api_key_cache: Arc::new(crate::cache::ApiKeyCache::new()),
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

        #[tokio::test]
        async fn test_create_gma_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "rarity".into(),
                description: Some("Quality tier".into()),
                payload: AttributePayload::Enum {
                    values: vec!["common".into(), "rare".into(), "legendary".into()],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Write);

            let result = create_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "create failed: {:?}", result.err());
            let resp = result.unwrap();
            let gma = resp.0;
            assert_eq!(gma.name, "rarity");
            assert_eq!(gma.description, Some("Quality tier".into()));
            assert_eq!(gma.value_type, ValueType::Enum);
            assert_eq!(gma.payload, serde_json::json!({"values": ["common", "rare", "legendary"]}));

            sqlx::query("DELETE FROM global_meta_attributes WHERE client_id = $1")
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
        async fn test_create_gma_duplicate_name_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            create_test_gma(&pool, client_id, "rarity").await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "rarity".into(),
                description: None,
                payload: AttributePayload::Range {
                    min: 1.0,
                    max: 5.0,
                    distribution: None,
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Write);

            let result = create_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("already exists"));

            sqlx::query("DELETE FROM global_meta_attributes WHERE client_id = $1")
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
        async fn test_create_gma_invalid_range_min_gt_max_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "damage".into(),
                description: None,
                payload: AttributePayload::Range {
                    min: 100.0,
                    max: 10.0,
                    distribution: None,
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Write);

            let result = create_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.detail.unwrap().contains("validation"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_gma_invalid_enum_empty_values_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "empty_enum".into(),
                description: None,
                payload: AttributePayload::Enum { values: vec![] },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Write);

            let result = create_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.errors.is_some());
            let errors = err.errors.unwrap();
            assert_eq!(errors.len(), 1);
            assert!(errors[0].message.contains("at least 1 entry"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_get_gma_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Read);

            let result = get_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
            )
            .await;

            assert!(result.is_ok(), "get failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0.id, gma_id);
            assert_eq!(resp.0.name, "rarity");
            assert_eq!(resp.0.value_type, ValueType::Range);

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
        async fn test_get_gma_not_found_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Read);

            let result = get_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(uuid::Uuid::new_v4()),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_get_gma_wrong_client_returns_403() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_a, "rarity").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_b, arche_types::Permission::Read);

            let result = get_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 403);

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

        #[tokio::test]
        async fn test_update_gma_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "rarity_v2".into(),
                description: Some("Updated quality tier".into()),
                payload: AttributePayload::Enum {
                    values: vec!["common".into(), "rare".into(), "epic".into(), "legendary".into()],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Write);

            let result = update_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "update failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0.id, gma_id);
            assert_eq!(resp.0.name, "rarity_v2");
            assert_eq!(resp.0.description, Some("Updated quality tier".into()));
            assert_eq!(resp.0.value_type, ValueType::Enum);
            assert_eq!(
                resp.0.payload,
                serde_json::json!({"values": ["common", "rare", "epic", "legendary"]})
            );

            let db_name: String = sqlx::query_scalar(
                "SELECT name FROM global_meta_attributes WHERE id = $1",
            )
            .bind(gma_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(db_name, "rarity_v2");

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
        async fn test_update_gma_invalid_payload_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let gma_id = create_test_gma(&pool, client_id, "rarity").await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "rarity".into(),
                description: None,
                payload: AttributePayload::Range {
                    min: 100.0,
                    max: 10.0,
                    distribution: None,
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Write);

            let result = update_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);

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
        async fn test_update_gma_duplicate_name_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            create_test_gma(&pool, client_id, "rarity").await;
            let gma_id = create_test_gma(&pool, client_id, "quality").await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "rarity".into(),
                description: None,
                payload: AttributePayload::Range {
                    min: 1.0,
                    max: 10.0,
                    distribution: None,
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Write);

            let result = update_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::extract::Path(gma_id),
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("already exists"));

            sqlx::query("DELETE FROM global_meta_attributes WHERE client_id = $1")
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
        async fn test_list_gma_cursor_pagination() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            for i in 0..5 {
                sqlx::query(
                    "INSERT INTO global_meta_attributes (client_id, name, value_type, payload) \
                     VALUES ($1, $2, 'single'::value_type, $3)",
                )
                .bind(client_id)
                .bind(format!("attr_{i}"))
                .bind(&serde_json::json!({"value": i as f64}))
                .execute(&pool)
                .await
                .unwrap();
            }

            let query = GlobalMetaAttributeListQuery {
                cursor: None,
                limit: Some(3),
                page: None,
                per_page: None,
                client_id: None,
                value_type: None,
                search: None,
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Read);

            let result = list_global_meta_attributes(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0.data.len(), 3);
            assert!(resp.0.next_cursor.is_some());

            sqlx::query("DELETE FROM global_meta_attributes WHERE client_id = $1")
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
        async fn test_list_gma_filter_by_value_type() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, value_type, payload) \
                 VALUES ($1, $2, 'enum'::value_type, $3)",
            )
            .bind(client_id)
            .bind("rarity")
            .bind(&serde_json::json!({"values": ["common"]}))
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, value_type, payload) \
                 VALUES ($1, $2, 'range'::value_type, $3)",
            )
            .bind(client_id)
            .bind("damage")
            .bind(&serde_json::json!({"min": 1.0, "max": 10.0}))
            .execute(&pool)
            .await
            .unwrap();

            let query = GlobalMetaAttributeListQuery {
                cursor: None,
                limit: Some(10),
                page: None,
                per_page: None,
                client_id: None,
                value_type: Some("enum".into()),
                search: None,
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Read);

            let result = list_global_meta_attributes(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0.data.len(), 1);
            assert_eq!(resp.0.data[0].name, "rarity");

            sqlx::query("DELETE FROM global_meta_attributes WHERE client_id = $1")
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
        async fn test_list_gma_search_by_name() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, value_type, payload) \
                 VALUES ($1, $2, 'string'::value_type, $3)",
            )
            .bind(client_id)
            .bind("fire_damage")
            .bind(&serde_json::json!({}))
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, value_type, payload) \
                 VALUES ($1, $2, 'string'::value_type, $3)",
            )
            .bind(client_id)
            .bind("ice_shield")
            .bind(&serde_json::json!({}))
            .execute(&pool)
            .await
            .unwrap();

            let query = GlobalMetaAttributeListQuery {
                cursor: None,
                limit: Some(10),
                page: None,
                per_page: None,
                client_id: None,
                value_type: None,
                search: Some("fire".into()),
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, arche_types::Permission::Read);

            let result = list_global_meta_attributes(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0.data.len(), 1);
            assert_eq!(resp.0.data[0].name, "fire_damage");

            sqlx::query("DELETE FROM global_meta_attributes WHERE client_id = $1")
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
        async fn test_create_gma_super_admin_write_returns_403() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let req = CreateGlobalMetaAttributeRequest {
                name: "rarity".into(),
                description: None,
                payload: AttributePayload::Range {
                    min: 1.0,
                    max: 10.0,
                    distribution: None,
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = super_user();

            let result = create_global_meta_attribute(
                axum::extract::State(state),
                user,
                axum::Json(req),
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
    }
}

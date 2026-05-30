use arche_types::attribute::{AffixAttribute, AttributePayload};
use arche_types::common::{AffixListQuery, PaginatedResponse, WriteClientQuery};
use arche_types::crud::CreateAffixRequest;
use arche_types::{Affix, AffixLocation};
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::audit_log::record_audit;
use crate::auth::permission::CurrentUser;
use crate::error::{FieldError, ProblemResponse, ReferenceInfo};

const AFFIX_COLUMNS: &str =
    "id, client_id, name, type::text, description, attribute, created_at, updated_at";

impl crate::pagination::HasId for Affix {
    fn id(&self) -> Uuid {
        self.id
    }
}

fn affix_location_str(loc: &AffixLocation) -> &'static str {
    match loc {
        AffixLocation::Prefix => "prefix",
        AffixLocation::Suffix => "suffix",
    }
}

fn affix_row_to_affix(row: &PgRow) -> Affix {
    let type_str: String = row.get("type");
    Affix {
        id: row.get("id"),
        client_id: row.get("client_id"),
        name: row.get("name"),
        location: match type_str.as_str() {
            "prefix" => AffixLocation::Prefix,
            "suffix" => AffixLocation::Suffix,
            _ => panic!("Unexpected affix type from DB: {type_str}"),
        },
        description: row.get("description"),
        attribute: row.get("attribute"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
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
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
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
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })
    }
}

fn check_affix_access(
    user: &crate::auth::AuthenticatedKey,
    affix_client_id: Uuid,
    id: Uuid,
) -> Result<(), ProblemResponse> {
    if !user.is_super {
        let user_cid = user.client_id.ok_or_else(|| {
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })?;
        if affix_client_id != user_cid {
            return Err(ProblemResponse::not_found(format!("Affix not found: {id}")));
        }
    }
    Ok(())
}

fn apply_affix_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    client_id: Option<Uuid>,
    location: Option<&'a AffixLocation>,
    search: Option<&'a str>,
    cursor: Option<Uuid>,
) {
    let mut first = true;

    if let Some(cid) = client_id {
        builder.push(" WHERE client_id = ");
        builder.push_bind(cid);
        first = false;
    }

    if let Some(loc) = location {
        if first {
            builder.push(" WHERE ");
            first = false;
        } else {
            builder.push(" AND ");
        }
        builder.push("type = ");
        builder.push_bind(affix_location_str(loc));
    }

    if let Some(s) = search {
        if first {
            builder.push(" WHERE ");
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

#[derive(Debug, Deserialize)]
pub struct DeleteAffixQuery {
    #[serde(default)]
    pub force: bool,
}

pub async fn list_affixes(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<AffixListQuery>,
) -> Result<Json<PaginatedResponse<Affix>>, ProblemResponse> {
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

            let mut builder =
                QueryBuilder::new(format!("SELECT {AFFIX_COLUMNS} FROM affixes"));
            apply_affix_filters(
                &mut builder,
                client_id_filter,
                query.location.as_ref(),
                query.search.as_deref(),
                after,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(fetch_limit);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "affixes: list cursor query failed");
                ProblemResponse::unprocessable_entity("Failed to list affixes")
            })?;

            let affixes: Vec<Affix> = rows.iter().map(affix_row_to_affix).collect();
            let response = crate::pagination::paginate_cursor(affixes, limit);
            Ok(Json(response))
        }
        crate::pagination::PaginationMode::Offset {
            per_page,
            offset,
            ..
        } => {
            let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM affixes");
            apply_affix_filters(
                &mut count_builder,
                client_id_filter,
                query.location.as_ref(),
                query.search.as_deref(),
                None,
            );

            let total: i64 = count_builder
                .build()
                .fetch_one(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "affixes: list count query failed");
                    ProblemResponse::unprocessable_entity("Failed to count affixes")
                })?
                .get(0);

            let mut builder =
                QueryBuilder::new(format!("SELECT {AFFIX_COLUMNS} FROM affixes"));
            apply_affix_filters(
                &mut builder,
                client_id_filter,
                query.location.as_ref(),
                query.search.as_deref(),
                None,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(per_page);
            builder.push(" OFFSET ");
            builder.push_bind(offset);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "affixes: list offset query failed");
                ProblemResponse::unprocessable_entity("Failed to list affixes")
            })?;

            let affixes: Vec<Affix> = rows.iter().map(affix_row_to_affix).collect();
            let response = crate::pagination::paginate_offset(affixes, total);
            Ok(Json(response))
        }
    }
}

async fn validate_affix_attribute(
    pool: &sqlx::PgPool,
    client_id: Uuid,
    attribute: &AffixAttribute,
) -> Result<serde_json::Value, ProblemResponse> {
    match attribute {
        AffixAttribute::Ref { ref_id } => {
            let exists = sqlx::query(
                "SELECT id FROM global_meta_attributes WHERE id = $1 AND client_id = $2",
            )
            .bind(ref_id)
            .bind(client_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "affixes: check ref_id failed");
                ProblemResponse::unprocessable_entity("Failed to validate attribute reference")
            })?;

            if exists.is_none() {
                return Err(ProblemResponse::validation_error(
                    format!("Global meta attribute not found: {ref_id}"),
                    vec![FieldError {
                        path: "attribute.$ref_id".into(),
                        message: format!(
                            "Referenced global meta attribute {ref_id} does not exist"
                        ),
                    }],
                ));
            }

            let json = serde_json::to_value(attribute).map_err(|e| {
                ProblemResponse::validation_error(
                    format!("Failed to serialize attribute: {e}"),
                    vec![],
                )
            })?;
            Ok(json)
        }
        AffixAttribute::Inline(inline) => {
            if inline.name.is_empty() {
                return Err(ProblemResponse::validation_error(
                    "Attribute name must not be empty",
                    vec![FieldError {
                        path: "attribute.name".into(),
                        message: "name must not be empty".into(),
                    }],
                ));
            }

            let value_type = match &inline.payload {
                AttributePayload::Single { .. } => arche_types::ValueType::Single,
                AttributePayload::Enum { .. } => arche_types::ValueType::Enum,
                AttributePayload::Range { .. } => arche_types::ValueType::Range,
                AttributePayload::String { .. } => arche_types::ValueType::String,
                AttributePayload::Boolean { .. } => arche_types::ValueType::Boolean,
            };

            let payload_json = serde_json::to_value(&inline.payload).map_err(|e| {
                ProblemResponse::validation_error(
                    format!("Failed to serialize attribute for validation: {e}"),
                    vec![],
                )
            })?;

            let payload_errors =
                arche_types::validation::validate_attribute_payload(&value_type, &payload_json);
            if !payload_errors.is_empty() {
                let field_errors: Vec<FieldError> = payload_errors
                    .into_iter()
                    .map(|e| FieldError {
                        path: format!("attribute.{}", e.path),
                        message: e.message,
                    })
                    .collect();
                return Err(ProblemResponse::validation_error(
                    "Attribute payload validation failed",
                    field_errors,
                ));
            }

            let attr_json = serde_json::to_value(attribute).map_err(|e| {
                ProblemResponse::validation_error(
                    format!("Failed to serialize attribute: {e}"),
                    vec![],
                )
            })?;
            Ok(attr_json)
        }
    }
}

fn reject_ambiguous_attribute(
    attr_obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), ProblemResponse> {
    let has_ref_id = attr_obj.contains_key("$ref_id");
    let has_inline = attr_obj.contains_key("name");
    if has_ref_id && has_inline {
        return Err(ProblemResponse::validation_error(
            "affix attribute must be either an inline definition (with 'name') or a $ref_id reference, not both",
            vec![FieldError {
                path: "attribute".into(),
                message: "attribute must be either an inline definition or a $ref_id reference, not both".into(),
            }],
        ));
    }
    Ok(())
}

pub async fn create_affix(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<WriteClientQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Affix>, ProblemResponse> {
    let client_id = resolve_client_id_for_write(&user, query.client_id)?;

    let attr_obj = body
        .get("attribute")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            ProblemResponse::validation_error(
                "attribute is required",
                vec![FieldError {
                    path: "attribute".into(),
                    message: "attribute is required".into(),
                }],
            )
        })?;

    reject_ambiguous_attribute(attr_obj)?;

    let req: CreateAffixRequest = serde_json::from_value(body).map_err(|e| {
        ProblemResponse::validation_error(
            format!("Invalid request body: {e}"),
            vec![],
        )
    })?;

    if req.name.is_empty() {
        return Err(ProblemResponse::validation_error(
            "Affix name must not be empty",
            vec![FieldError {
                path: "name".into(),
                message: "name must not be empty".into(),
            }],
        ));
    }

    let existing = sqlx::query(
        "SELECT id FROM affixes WHERE client_id = $1 AND name = $2",
    )
    .bind(client_id)
    .bind(&req.name)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: check name uniqueness failed");
        ProblemResponse::unprocessable_entity("Failed to check affix name uniqueness")
    })?;

    if existing.is_some() {
        return Err(ProblemResponse::conflict(format!(
            "An affix with name '{}' already exists for this client",
            req.name
        )));
    }

    let attribute_json =
        validate_affix_attribute(&state.pool, client_id, &req.attribute).await?;

    let row = sqlx::query(
        "INSERT INTO affixes (client_id, name, type, description, attribute) \
         VALUES ($1, $2, $3::affix_location, $4, $5) \
         RETURNING id, created_at, updated_at",
    )
    .bind(client_id)
    .bind(&req.name)
    .bind(affix_location_str(&req.location))
    .bind(&req.description)
    .bind(&attribute_json)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: insert failed");
        ProblemResponse::validation_error(
            format!("Failed to create affix: {e}"),
            vec![],
        )
    })?;

    let affix = Affix {
        id: row.get("id"),
        client_id,
        name: req.name,
        location: req.location,
        description: req.description,
        attribute: attribute_json,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    let after_snapshot = serde_json::to_value(&affix).unwrap_or_default();
    if let Err(e) = record_audit(
        &*state.pool,
        &user,
        Some(client_id),
        "affix",
        affix.id,
        "created",
        None,
        Some(after_snapshot),
    )
    .await
    {
        tracing::error!(error = %e, "affixes: audit log insert failed");
    }

    state.reload_client_cache(client_id).await;

    Ok(Json(affix))
}

pub async fn get_affix(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Affix>, ProblemResponse> {
    let row = sqlx::query(&format!(
        "SELECT {AFFIX_COLUMNS} FROM affixes WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: get query failed");
        ProblemResponse::unprocessable_entity("Failed to get affix")
    })?
    .ok_or_else(|| ProblemResponse::not_found(format!("Affix not found: {id}")))?;

    let affix = affix_row_to_affix(&row);

    check_affix_access(&user, affix.client_id, id)?;

    Ok(Json(affix))
}

pub async fn update_affix(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Affix>, ProblemResponse> {
    let existing = sqlx::query(&format!(
        "SELECT {AFFIX_COLUMNS} FROM affixes WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: get for update failed");
        ProblemResponse::unprocessable_entity("Failed to get affix for update")
    })?
    .ok_or_else(|| ProblemResponse::not_found(format!("Affix not found: {id}")))?;

    let existing_affix = affix_row_to_affix(&existing);

    check_affix_access(&user, existing_affix.client_id, id)?;

    let client_id = existing_affix.client_id;

    if let Some(attr_obj) = body.get("attribute").and_then(|v| v.as_object()) {
        reject_ambiguous_attribute(attr_obj)?;
    }

    let req: CreateAffixRequest = serde_json::from_value(body).map_err(|e| {
        ProblemResponse::validation_error(
            format!("Invalid request body: {e}"),
            vec![],
        )
    })?;

    if req.name.is_empty() {
        return Err(ProblemResponse::validation_error(
            "Affix name must not be empty",
            vec![FieldError {
                path: "name".into(),
                message: "name must not be empty".into(),
            }],
        ));
    }

    let name_conflict = sqlx::query(
        "SELECT id FROM affixes WHERE client_id = $1 AND name = $2 AND id != $3",
    )
    .bind(client_id)
    .bind(&req.name)
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: check name uniqueness failed");
        ProblemResponse::unprocessable_entity("Failed to check affix name uniqueness")
    })?;

    if name_conflict.is_some() {
        return Err(ProblemResponse::conflict(format!(
            "An affix with name '{}' already exists for this client",
            req.name
        )));
    }

    let attribute_json =
        validate_affix_attribute(&state.pool, client_id, &req.attribute).await?;

    let row = sqlx::query(
        "UPDATE affixes SET name=$1, type=$2::affix_location, description=$3, \
         attribute=$4, updated_at=now() \
         WHERE id=$5 \
         RETURNING created_at, updated_at",
    )
    .bind(&req.name)
    .bind(affix_location_str(&req.location))
    .bind(&req.description)
    .bind(&attribute_json)
    .bind(id)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "affixes: update failed");
        ProblemResponse::validation_error(
            format!("Failed to update affix: {e}"),
            vec![],
        )
    })?;

    let affix = Affix {
        id,
        client_id,
        name: req.name,
        location: req.location,
        description: req.description,
        attribute: attribute_json,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    let before_snapshot = serde_json::to_value(&existing_affix).unwrap_or_default();
    let after_snapshot = serde_json::to_value(&affix).unwrap_or_default();
    if let Err(e) = record_audit(
        &*state.pool,
        &user,
        Some(client_id),
        "affix",
        affix.id,
        "updated",
        Some(before_snapshot),
        Some(after_snapshot),
    )
    .await
    {
        tracing::error!(error = %e, "affixes: audit log insert failed");
    }

    state.reload_client_cache(client_id).await;

    Ok(Json(affix))
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

    check_affix_access(&user, affix_client_id, id)?;

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

    state.reload_client_cache(affix_client_id).await;

    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AffixBlueprintRef {
    pub blueprint_id: Uuid,
    pub blueprint_name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AffixReferencesResponse {
    pub references: std::collections::HashMap<String, Vec<AffixBlueprintRef>>,
}

pub async fn list_affix_references(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<AffixReferencesResponse>, ProblemResponse> {
    let client_id_filter = resolve_client_id(&user, None)?;

    let mut query = String::from(
        "SELECT ba.affix_id, ba.blueprint_id, b.name AS blueprint_name \
         FROM blueprint_affixes ba \
         JOIN blueprints b ON b.id = ba.blueprint_id"
    );

    if let Some(cid) = client_id_filter {
        query.push_str(" WHERE b.client_id = ");
        query.push_str(&cid.to_string());
    }

    query.push_str(" ORDER BY ba.affix_id, b.name");

    let rows = sqlx::query(&query)
        .fetch_all(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "affixes: list references query failed");
            ProblemResponse::unprocessable_entity("Failed to list affix references")
        })?;

    let mut refs: std::collections::HashMap<String, Vec<AffixBlueprintRef>> =
        std::collections::HashMap::new();

    for row in &rows {
        let affix_id: Uuid = row.get("affix_id");
        let blueprint_id: Uuid = row.get("blueprint_id");
        let blueprint_name: String = row.get("blueprint_name");

        refs
            .entry(affix_id.to_string())
            .or_default()
            .push(AffixBlueprintRef { blueprint_id, blueprint_name });
    }

    Ok(Json(AffixReferencesResponse { references: refs }))
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

        #[tokio::test]
        async fn test_create_affix_inline_attribute_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "Fire".into(),
                location: AffixLocation::Prefix,
                description: Some("Burning effect".into()),
                attribute: arche_types::attribute::AffixAttribute::Inline(
                    arche_types::attribute::AffixInlineAttributeDef {
                        name: "fire_damage".into(),
                        description: Some("Fire damage bonus".into()),
                        payload: arche_types::attribute::AttributePayload::Range {
                            min: 5.0,
                            max: 15.0,
                            distribution: None,
                        },
                    },
                ),
            };

            let result = create_affix(
                axum::extract::State(state),
                user,
                axum::extract::Query(WriteClientQuery { client_id: None }),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_ok(), "create failed: {:?}", result.err());
            let resp = result.unwrap();
            let affix = resp.0;
            assert_eq!(affix.name, "Fire");
            assert_eq!(affix.location, AffixLocation::Prefix);
            assert_eq!(affix.description, Some("Burning effect".into()));
            assert_eq!(affix.client_id, client_id);

            let count: i64 = sqlx::query("SELECT COUNT(*) FROM affixes WHERE name = 'Fire' AND client_id = $1")
                .bind(client_id)
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
            assert_eq!(count, 1);

            sqlx::query("DELETE FROM affixes WHERE id = $1")
                .bind(affix.id)
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
        async fn test_create_affix_ref_id_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let gma_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO global_meta_attributes (id, client_id, name, value_type, payload) \
                 VALUES ($1,$2,$3,'range','{\"min\":1,\"max\":10}')",
            )
            .bind(gma_id)
            .bind(client_id)
            .bind("test_gma")
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "Ice".into(),
                location: AffixLocation::Suffix,
                description: None,
                attribute: arche_types::attribute::AffixAttribute::Ref {
                    ref_id: gma_id,
                },
            };

            let result = create_affix(
                axum::extract::State(state),
                user,
                axum::extract::Query(WriteClientQuery { client_id: None }),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_ok(), "create failed: {:?}", result.err());
            let resp = result.unwrap();
            let affix = resp.0;
            assert_eq!(affix.name, "Ice");
            assert_eq!(affix.location, AffixLocation::Suffix);

            sqlx::query("DELETE FROM affixes WHERE id = $1")
                .bind(affix.id)
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
        async fn test_create_affix_nonexistent_ref_id_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "Ghost".into(),
                location: AffixLocation::Prefix,
                description: None,
                attribute: arche_types::attribute::AffixAttribute::Ref {
                    ref_id: Uuid::new_v4(),
                },
            };

            let result = create_affix(
                axum::extract::State(state),
                user,
                axum::extract::Query(WriteClientQuery { client_id: None }),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.detail.unwrap().contains("not found"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_affix_invalid_attribute_payload_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "BadRange".into(),
                location: AffixLocation::Prefix,
                description: None,
                attribute: arche_types::attribute::AffixAttribute::Inline(
                    arche_types::attribute::AffixInlineAttributeDef {
                        name: "bad_range".into(),
                        description: None,
                        payload: arche_types::attribute::AttributePayload::Range {
                            min: 100.0,
                            max: 1.0,
                            distribution: None,
                        },
                    },
                ),
            };

            let result = create_affix(
                axum::extract::State(state),
                user,
                axum::extract::Query(WriteClientQuery { client_id: None }),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.detail.unwrap().contains("payload"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_affix_empty_name_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "".into(),
                location: AffixLocation::Prefix,
                description: None,
                attribute: arche_types::attribute::AffixAttribute::Inline(
                    arche_types::attribute::AffixInlineAttributeDef {
                        name: "test".into(),
                        description: None,
                        payload: arche_types::attribute::AttributePayload::Single {
                            value: 1.0,
                            distribution: None,
                        },
                    },
                ),
            };

            let result = create_affix(
                axum::extract::State(state),
                user,
                axum::extract::Query(WriteClientQuery { client_id: None }),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.detail.unwrap().contains("name"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_affix_both_inline_and_ref_id_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let body = json!({
                "name": "Conflict",
                "type": "prefix",
                "attribute": {
                    "name": "conflict_attr",
                    "value_type": "single",
                    "value": 1.0,
                    "$ref_id": Uuid::new_v4().to_string()
                }
            });

            let result = create_affix(
                axum::extract::State(state),
                user,
                axum::extract::Query(WriteClientQuery { client_id: None }),
                axum::Json(body),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.detail.unwrap().contains("not both"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_affix_duplicate_name_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix_id = create_test_affix(&pool, client_id, "Fire", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "Fire".into(),
                location: AffixLocation::Suffix,
                description: None,
                attribute: arche_types::attribute::AffixAttribute::Inline(
                    arche_types::attribute::AffixInlineAttributeDef {
                        name: "fire_dmg".into(),
                        description: None,
                        payload: arche_types::attribute::AttributePayload::Single {
                            value: 1.0,
                            distribution: None,
                        },
                    },
                ),
            };

            let result = create_affix(
                axum::extract::State(state),
                user,
                axum::extract::Query(WriteClientQuery { client_id: None }),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("already exists"));

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
        async fn test_get_affix_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix_id = create_test_affix(&pool, client_id, "Fire", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Read);

            let result = get_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_id),
            )
            .await;

            assert!(result.is_ok(), "get failed: {:?}", result.err());
            let resp = result.unwrap();
            let affix = resp.0;
            assert_eq!(affix.id, affix_id);
            assert_eq!(affix.name, "Fire");
            assert_eq!(affix.location, AffixLocation::Prefix);
            assert_eq!(affix.client_id, client_id);

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
        async fn test_get_nonexistent_affix_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Read);

            let result = get_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(Uuid::new_v4()),
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
        async fn test_get_affix_wrong_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;
            let affix_id = create_test_affix(&pool, client_a, "Fire", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_b, Permission::Read);

            let result = get_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_id),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains("Affix not found"));

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

        #[tokio::test]
        async fn test_update_affix_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix_id = create_test_affix(&pool, client_id, "Fire", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "Inferno".into(),
                location: AffixLocation::Suffix,
                description: Some("Updated".into()),
                attribute: arche_types::attribute::AffixAttribute::Inline(
                    arche_types::attribute::AffixInlineAttributeDef {
                        name: "inferno_dmg".into(),
                        description: None,
                        payload: arche_types::attribute::AttributePayload::Single {
                            value: 42.0,
                            distribution: None,
                        },
                    },
                ),
            };

            let result = update_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_id),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_ok(), "update failed: {:?}", result.err());
            let resp = result.unwrap();
            let affix = resp.0;
            assert_eq!(affix.id, affix_id);
            assert_eq!(affix.name, "Inferno");
            assert_eq!(affix.location, AffixLocation::Suffix);
            assert_eq!(affix.description, Some("Updated".into()));

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
        async fn test_update_affix_nonexistent_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "Ghost".into(),
                location: AffixLocation::Prefix,
                description: None,
                attribute: arche_types::attribute::AffixAttribute::Inline(
                    arche_types::attribute::AffixInlineAttributeDef {
                        name: "ghost".into(),
                        description: None,
                        payload: arche_types::attribute::AttributePayload::Single {
                            value: 1.0,
                            distribution: None,
                        },
                    },
                ),
            };

            let result = update_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(Uuid::new_v4()),
                axum::Json(serde_json::to_value(req).unwrap()),
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
        async fn test_update_affix_duplicate_name_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let affix_a = create_test_affix(&pool, client_id, "Fire", "prefix").await;
            let affix_b = create_test_affix(&pool, client_id, "Ice", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let req = CreateAffixRequest {
                name: "Ice".into(),
                location: AffixLocation::Prefix,
                description: None,
                attribute: arche_types::attribute::AffixAttribute::Inline(
                    arche_types::attribute::AffixInlineAttributeDef {
                        name: "test".into(),
                        description: None,
                        payload: arche_types::attribute::AttributePayload::Single {
                            value: 1.0,
                            distribution: None,
                        },
                    },
                ),
            };

            let result = update_affix(
                axum::extract::State(state),
                user,
                axum::extract::Path(affix_a),
                axum::Json(serde_json::to_value(req).unwrap()),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("already exists"));

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
        async fn test_list_affixes_cursor_pagination() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let a1 = create_test_affix(&pool, client_id, "Fire", "prefix").await;
            let a2 = create_test_affix(&pool, client_id, "Ice", "suffix").await;
            let a3 = create_test_affix(&pool, client_id, "Poison", "prefix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Read);

            let query = AffixListQuery {
                cursor: None,
                limit: Some(2),
                page: None,
                per_page: None,
                client_id: None,
                location: None,
                search: None,
            };

            let result = list_affixes(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0.data.len(), 2);
            assert!(resp.0.next_cursor.is_some());

            sqlx::query("DELETE FROM affixes WHERE id = ANY($1)")
                .bind(&[a1, a2, a3])
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
        async fn test_list_affixes_filter_by_type() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let a1 = create_test_affix(&pool, client_id, "Fire", "prefix").await;
            let a2 = create_test_affix(&pool, client_id, "Ice", "suffix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Read);

            let query = AffixListQuery {
                cursor: None,
                limit: None,
                page: Some(1),
                per_page: Some(10),
                client_id: None,
                location: Some(AffixLocation::Suffix),
                search: None,
            };

            let result = list_affixes(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            let ids: Vec<Uuid> = resp.0.data.iter().map(|a| a.id).collect();
            assert!(ids.contains(&a2));
            assert!(!ids.contains(&a1));

            sqlx::query("DELETE FROM affixes WHERE id = ANY($1)")
                .bind(&[a1, a2])
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
        async fn test_list_affixes_search_by_name() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let a1 = create_test_affix(&pool, client_id, "FireBlade", "prefix").await;
            let a2 = create_test_affix(&pool, client_id, "IceShard", "suffix").await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Read);

            let query = AffixListQuery {
                cursor: None,
                limit: None,
                page: Some(1),
                per_page: Some(10),
                client_id: None,
                location: None,
                search: Some("Blade".into()),
            };

            let result = list_affixes(
                axum::extract::State(state),
                user,
                axum::extract::Query(query),
            )
            .await;

            assert!(result.is_ok(), "list failed: {:?}", result.err());
            let resp = result.unwrap();
            let ids: Vec<Uuid> = resp.0.data.iter().map(|a| a.id).collect();
            assert!(ids.contains(&a1));
            assert!(!ids.contains(&a2));

            sqlx::query("DELETE FROM affixes WHERE id = ANY($1)")
                .bind(&[a1, a2])
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

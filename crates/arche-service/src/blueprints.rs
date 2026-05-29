use arche_types::attribute::{AffixPoolEntry, BlueprintAffixConfig, BlueprintAttribute};
use arche_types::common::{BlueprintListQuery, PaginatedResponse, WriteClientQuery};
use arche_types::crud::{BlueprintResponse, CreateBlueprintRequest};
use arche_types::validation;
use arche_types::{AffixLocation, Blueprint};
use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use sqlx::postgres::PgRow;
use sqlx::{QueryBuilder, Row};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::audit_log::record_audit;
use crate::auth::permission::CurrentUser;
use crate::error::{FieldError, ProblemResponse, ReferenceInfo};

impl crate::pagination::HasId for Blueprint {
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

fn parse_affix_location(s: &str) -> Option<AffixLocation> {
    match s {
        "prefix" => Some(AffixLocation::Prefix),
        "suffix" => Some(AffixLocation::Suffix),
        _ => None,
    }
}

const BLUEPRINT_COLUMNS: &str = "id, client_id, name, archetype, weight, description, \
    attributes, attribute_order, min_prefixes, max_prefixes, \
    min_suffixes, max_suffixes, created_at, updated_at";

pub async fn list_blueprints(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<BlueprintListQuery>,
) -> Result<Json<PaginatedResponse<Blueprint>>, ProblemResponse> {
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
                format!("SELECT {BLUEPRINT_COLUMNS} FROM blueprints"),
            );
            apply_filters(
                &mut builder,
                client_id_filter,
                query.archetype.as_deref(),
                query.search.as_deref(),
                after,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(fetch_limit);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "blueprints: list cursor query failed");
                ProblemResponse::unprocessable_entity("Failed to list blueprints")
            })?;

            let blueprints: Vec<Blueprint> = rows.iter().map(row_to_blueprint).collect();
            let response = crate::pagination::paginate_cursor(blueprints, limit);
            Ok(Json(response))
        }
        crate::pagination::PaginationMode::Offset {
            per_page,
            offset,
            ..
        } => {
            let mut count_builder =
                QueryBuilder::new("SELECT COUNT(*) FROM blueprints");
            apply_filters(
                &mut count_builder,
                client_id_filter,
                query.archetype.as_deref(),
                query.search.as_deref(),
                None,
            );

            let total: i64 = count_builder
                .build()
                .fetch_one(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "blueprints: list count query failed");
                    ProblemResponse::unprocessable_entity("Failed to count blueprints")
                })?
                .get(0);

            let mut builder = QueryBuilder::new(
                format!("SELECT {BLUEPRINT_COLUMNS} FROM blueprints"),
            );
            apply_filters(
                &mut builder,
                client_id_filter,
                query.archetype.as_deref(),
                query.search.as_deref(),
                None,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(per_page);
            builder.push(" OFFSET ");
            builder.push_bind(offset);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "blueprints: list offset query failed");
                ProblemResponse::unprocessable_entity("Failed to list blueprints")
            })?;

            let blueprints: Vec<Blueprint> = rows.iter().map(row_to_blueprint).collect();
            let response = crate::pagination::paginate_offset(blueprints, total);
            Ok(Json(response))
        }
    }
}

pub async fn get_blueprint(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BlueprintResponse>, ProblemResponse> {
    let row = sqlx::query(&format!(
        "SELECT {BLUEPRINT_COLUMNS} FROM blueprints WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: get query failed");
        ProblemResponse::unprocessable_entity("Failed to get blueprint")
    })?
    .ok_or_else(|| ProblemResponse::not_found(format!("Blueprint not found: {id}")))?;

    let bp = row_to_blueprint(&row);

    if !user.is_super {
        let user_cid = user.client_id.ok_or_else(|| {
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })?;
        if bp.client_id != user_cid {
            return Err(ProblemResponse::forbidden(
                "Access denied: key is scoped to a different client",
            ));
        }
    }

    let affixes = fetch_affix_pool(&state.pool, id).await?;
    let bp_response = assemble_response(&bp, &affixes);

    Ok(Json(bp_response))
}

pub async fn create_blueprint(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<WriteClientQuery>,
    Json(req): Json<CreateBlueprintRequest>,
) -> Result<Json<BlueprintResponse>, ProblemResponse> {
    let client_id = resolve_client_id_for_write(&user, query.client_id)?;

    validate_affix_config(&req.affixes)?;

    validate_blueprint_attributes(
        &req.attributes,
        &req.attribute_order,
        req.weight,
        &req.affixes,
    )?;
    check_duplicate_name(&state.pool, client_id, &req.name, None).await?;

    let affix_rows = validate_affix_pool(
        &state.pool,
        client_id,
        &req.affixes.prefixes,
        AffixLocation::Prefix,
    )
    .await?;
    let suffix_affix_rows = validate_affix_pool(
        &state.pool,
        client_id,
        &req.affixes.suffixes,
        AffixLocation::Suffix,
    )
    .await?;

    let attributes_json =
        serde_json::to_value(&req.attributes).map_err(|e| {
            ProblemResponse::unprocessable_entity(format!("Invalid attributes: {e}"))
        })?;

    let mut tx = state.pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "blueprints: begin transaction failed");
        ProblemResponse::unprocessable_entity("Failed to create blueprint")
    })?;

    let row = sqlx::query(
        "INSERT INTO blueprints (client_id, name, archetype, weight, description, \
         attributes, attribute_order, min_prefixes, max_prefixes, \
         min_suffixes, max_suffixes) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) \
         RETURNING id, created_at, updated_at",
    )
    .bind(client_id)
    .bind(&req.name)
    .bind(&req.archetype)
    .bind(req.weight)
    .bind(&req.description)
    .bind(&attributes_json)
    .bind(&req.attribute_order)
    .bind(req.affixes.min_prefixes)
    .bind(req.affixes.max_prefixes)
    .bind(req.affixes.min_suffixes)
    .bind(req.affixes.max_suffixes)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: insert failed");
        ProblemResponse::validation_error(
            format!("Failed to create blueprint: {e}"),
            vec![],
        )
    })?;

    let bp_id: Uuid = row.get("id");
    let created_at = row.get("created_at");
    let updated_at = row.get("updated_at");

    let mut sort_order: i32 = 0;
    insert_affix_entries(&mut tx, bp_id, &affix_rows, AffixLocation::Prefix, &mut sort_order).await?;
    insert_affix_entries(&mut tx, bp_id, &suffix_affix_rows, AffixLocation::Suffix, &mut sort_order).await?;

    record_audit(
        &mut *tx,
        &user,
        Some(client_id),
        "blueprint",
        bp_id,
        "created",
        None,
        Some(serde_json::json!({
            "id": bp_id,
            "client_id": client_id,
            "name": &req.name,
            "archetype": &req.archetype,
            "weight": req.weight,
            "description": &req.description,
            "attributes": &attributes_json,
            "attribute_order": &req.attribute_order,
            "min_prefixes": req.affixes.min_prefixes,
            "max_prefixes": req.affixes.max_prefixes,
            "min_suffixes": req.affixes.min_suffixes,
            "max_suffixes": req.affixes.max_suffixes,
        })),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: audit log insert failed");
        ProblemResponse::unprocessable_entity("Failed to create blueprint")
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "blueprints: commit transaction failed");
        ProblemResponse::unprocessable_entity("Failed to create blueprint")
    })?;

    let bp = Blueprint {
        id: bp_id,
        client_id,
        name: req.name,
        archetype: req.archetype,
        weight: req.weight,
        description: req.description,
        attributes: attributes_json,
        attribute_order: req.attribute_order,
        min_prefixes: req.affixes.min_prefixes,
        max_prefixes: req.affixes.max_prefixes,
        min_suffixes: req.affixes.min_suffixes,
        max_suffixes: req.affixes.max_suffixes,
        created_at,
        updated_at,
    };

    let response = assemble_response(
        &bp,
        &BlueprintAffixConfig {
            min_prefixes: req.affixes.min_prefixes,
            max_prefixes: req.affixes.max_prefixes,
            min_suffixes: req.affixes.min_suffixes,
            max_suffixes: req.affixes.max_suffixes,
            prefixes: affix_rows,
            suffixes: suffix_affix_rows,
        },
    );

    Ok(Json(response))
}

pub async fn update_blueprint(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateBlueprintRequest>,
) -> Result<Json<BlueprintResponse>, ProblemResponse> {
    let existing = sqlx::query(&format!(
        "SELECT {BLUEPRINT_COLUMNS} FROM blueprints WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: get for update failed");
        ProblemResponse::unprocessable_entity("Failed to get blueprint for update")
    })?
    .ok_or_else(|| ProblemResponse::not_found(format!("Blueprint not found: {id}")))?;

    let existing_bp = row_to_blueprint(&existing);

    if !user.is_super {
        let user_cid = user.client_id.ok_or_else(|| {
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })?;
        if existing_bp.client_id != user_cid {
            return Err(ProblemResponse::forbidden(
                "Access denied: key is scoped to a different client",
            ));
        }
    }

    let client_id = existing_bp.client_id;

    validate_affix_config(&req.affixes)?;

    validate_blueprint_attributes(
        &req.attributes,
        &req.attribute_order,
        req.weight,
        &req.affixes,
    )?;
    check_duplicate_name(&state.pool, client_id, &req.name, Some(id)).await?;

    validate_affix_pool(
        &state.pool,
        client_id,
        &req.affixes.prefixes,
        AffixLocation::Prefix,
    )
    .await?;
    validate_affix_pool(
        &state.pool,
        client_id,
        &req.affixes.suffixes,
        AffixLocation::Suffix,
    )
    .await?;

    let attributes_json =
        serde_json::to_value(&req.attributes).map_err(|e| {
            ProblemResponse::unprocessable_entity(format!("Invalid attributes: {e}"))
        })?;

    let mut tx = state.pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "blueprints: begin transaction for update failed");
        ProblemResponse::unprocessable_entity("Failed to update blueprint")
    })?;

    let row = sqlx::query(
        "UPDATE blueprints SET name=$1, archetype=$2, weight=$3, description=$4, \
         attributes=$5, attribute_order=$6, min_prefixes=$7, max_prefixes=$8, \
         min_suffixes=$9, max_suffixes=$10, updated_at=now() \
         WHERE id=$11 \
         RETURNING created_at, updated_at",
    )
    .bind(&req.name)
    .bind(&req.archetype)
    .bind(req.weight)
    .bind(&req.description)
    .bind(&attributes_json)
    .bind(&req.attribute_order)
    .bind(req.affixes.min_prefixes)
    .bind(req.affixes.max_prefixes)
    .bind(req.affixes.min_suffixes)
    .bind(req.affixes.max_suffixes)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: update failed");
        ProblemResponse::validation_error(
            format!("Failed to update blueprint: {e}"),
            vec![],
        )
    })?;

    let created_at = row.get("created_at");
    let updated_at = row.get("updated_at");

    let existing_affixes =
        sqlx::query("SELECT affix_id, weight, location FROM blueprint_affixes WHERE blueprint_id = $1")
            .bind(id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "blueprints: fetch existing affixes failed");
                ProblemResponse::unprocessable_entity("Failed to read existing affix pool")
            })?;

    let existing_set: HashMap<(Uuid, AffixLocation), f64> = existing_affixes
        .iter()
        .map(|r| {
            let affix_id: Uuid = r.get("affix_id");
            let loc_str: String = r.get("location");
            let location = parse_affix_location(&loc_str).unwrap();
            let weight: f64 = r.get("weight");
            ((affix_id, location), weight)
        })
        .collect();

    let mut desired: HashMap<(Uuid, AffixLocation), f64> = HashMap::new();
    for entry in &req.affixes.prefixes {
        desired.insert((entry.affix_id, AffixLocation::Prefix), entry.weight);
    }
    for entry in &req.affixes.suffixes {
        desired.insert((entry.affix_id, AffixLocation::Suffix), entry.weight);
    }

    let existing_keys: HashSet<(Uuid, AffixLocation)> = existing_set.keys().cloned().collect();
    let desired_keys: HashSet<(Uuid, AffixLocation)> = desired.keys().cloned().collect();

    let to_remove: Vec<&(Uuid, AffixLocation)> =
        existing_keys.difference(&desired_keys).collect();

    for key in &to_remove {
        sqlx::query(
            "DELETE FROM blueprint_affixes WHERE blueprint_id = $1 AND affix_id = $2 AND location = $3",
        )
        .bind(id)
        .bind(key.0)
        .bind(affix_location_str(&key.1))
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "blueprints: delete affix failed");
            ProblemResponse::unprocessable_entity("Failed to update affix pool")
        })?;
    }

    let mut sort_order: i32 = 0;
    upsert_affix_pool_entries(&mut tx, id, &req.affixes.prefixes, AffixLocation::Prefix, &existing_set, &mut sort_order).await?;
    upsert_affix_pool_entries(&mut tx, id, &req.affixes.suffixes, AffixLocation::Suffix, &existing_set, &mut sort_order).await?;

    let before_snapshot = serde_json::to_value(&existing_bp).map_err(|e| {
        tracing::error!(error = %e, "blueprints: serialize before snapshot failed");
        ProblemResponse::unprocessable_entity("Failed to update blueprint")
    })?;

    record_audit(
        &mut *tx,
        &user,
        Some(client_id),
        "blueprint",
        id,
        "updated",
        Some(before_snapshot),
        Some(serde_json::json!({
            "id": id,
            "client_id": client_id,
            "name": &req.name,
            "archetype": &req.archetype,
            "weight": req.weight,
            "description": &req.description,
            "attributes": &attributes_json,
            "attribute_order": &req.attribute_order,
            "min_prefixes": req.affixes.min_prefixes,
            "max_prefixes": req.affixes.max_prefixes,
            "min_suffixes": req.affixes.min_suffixes,
            "max_suffixes": req.affixes.max_suffixes,
        })),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: audit log insert failed");
        ProblemResponse::unprocessable_entity("Failed to update blueprint")
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "blueprints: commit update transaction failed");
        ProblemResponse::unprocessable_entity("Failed to update blueprint")
    })?;

    let bp = Blueprint {
        id,
        client_id,
        name: req.name,
        archetype: req.archetype,
        weight: req.weight,
        description: req.description,
        attributes: attributes_json,
        attribute_order: req.attribute_order,
        min_prefixes: req.affixes.min_prefixes,
        max_prefixes: req.affixes.max_prefixes,
        min_suffixes: req.affixes.min_suffixes,
        max_suffixes: req.affixes.max_suffixes,
        created_at,
        updated_at,
    };

    let response = assemble_response(&bp, &req.affixes);

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct DeleteBlueprintQuery {
    #[serde(default)]
    pub force: bool,
}

pub async fn delete_blueprint(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteBlueprintQuery>,
) -> Result<Json<serde_json::Value>, ProblemResponse> {
    let row = sqlx::query(&format!(
        "SELECT {BLUEPRINT_COLUMNS} FROM blueprints WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: get for delete failed");
        ProblemResponse::unprocessable_entity("Failed to get blueprint for deletion")
    })?
    .ok_or_else(|| ProblemResponse::not_found(format!("Blueprint not found: {id}")))?;

    let bp = row_to_blueprint(&row);

    if !user.is_super {
        let user_cid = user.client_id.ok_or_else(|| {
            ProblemResponse::forbidden("Access denied: key is not associated with any client")
        })?;
        if bp.client_id != user_cid {
            return Err(ProblemResponse::not_found(format!("Blueprint not found: {id}")));
        }
    }

    if !query.force {
        let ref_rows = sqlx::query(
            "SELECT a.id, a.name FROM blueprint_affixes ba \
             JOIN affixes a ON a.id = ba.affix_id \
             WHERE ba.blueprint_id = $1",
        )
        .bind(id)
        .fetch_all(&*state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "blueprints: check references failed");
            ProblemResponse::unprocessable_entity("Failed to check blueprint references")
        })?;

        if !ref_rows.is_empty() {
            let references: Vec<ReferenceInfo> = ref_rows
                .iter()
                .map(|r| {
                    let affix_id: Uuid = r.get("id");
                    let affix_name: String = r.get("name");
                    ReferenceInfo {
                        resource_type: "affix".into(),
                        resource_id: affix_id,
                        resource_name: affix_name,
                    }
                })
                .collect();

            return Err(ProblemResponse::delete_referenced_resource_with_refs(
                format!(
                    "Blueprint {id} is referenced by {} affix pool entries. Use force=true to cascade delete.",
                    ref_rows.len()
                ),
                references,
            ));
        }
    }

    let before_snapshot = serde_json::to_value(&bp).map_err(|e| {
        tracing::error!(error = %e, "blueprints: serialize before snapshot failed");
        ProblemResponse::unprocessable_entity("Failed to delete blueprint")
    })?;

    let action = if query.force { "force_deleted" } else { "deleted" };

    let mut tx = state.pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "blueprints: begin transaction for delete failed");
        ProblemResponse::unprocessable_entity("Failed to delete blueprint")
    })?;

    sqlx::query("DELETE FROM blueprints WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "blueprints: delete failed");
            ProblemResponse::unprocessable_entity("Failed to delete blueprint")
        })?;

    record_audit(
        &mut *tx,
        &user,
        Some(bp.client_id),
        "blueprint",
        id,
        action,
        Some(before_snapshot),
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: audit log insert failed");
        ProblemResponse::unprocessable_entity("Failed to delete blueprint")
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "blueprints: commit delete transaction failed");
        ProblemResponse::unprocessable_entity("Failed to delete blueprint")
    })?;

    Ok(Json(serde_json::json!({"deleted": true})))
}

fn apply_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    client_id: Option<Uuid>,
    archetype: Option<&'a str>,
    search: Option<&'a str>,
    cursor: Option<Uuid>,
) {
    let mut first = true;

    if let Some(cid) = client_id {
        builder.push(" WHERE client_id = ");
        builder.push_bind(cid);
        first = false;
    }

    if let Some(at) = archetype {
        if first {
            builder.push(" WHERE ");
            first = false;
        } else {
            builder.push(" AND ");
        }
        builder.push("archetype = ");
        builder.push_bind(at);
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

fn row_to_blueprint(row: &PgRow) -> Blueprint {
    Blueprint {
        id: row.get("id"),
        client_id: row.get("client_id"),
        name: row.get("name"),
        archetype: row.get("archetype"),
        weight: row.get("weight"),
        description: row.get("description"),
        attributes: row.get("attributes"),
        attribute_order: row.get("attribute_order"),
        min_prefixes: row.get("min_prefixes"),
        max_prefixes: row.get("max_prefixes"),
        min_suffixes: row.get("min_suffixes"),
        max_suffixes: row.get("max_suffixes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn validate_affix_config(config: &BlueprintAffixConfig) -> Result<(), ProblemResponse> {
    let mut errors = Vec::new();

    if config.min_prefixes < 0 {
        errors.push(FieldError {
            path: "affixes.min_prefixes".into(),
            message: "min_prefixes must be >= 0".into(),
        });
    }
    if config.min_suffixes < 0 {
        errors.push(FieldError {
            path: "affixes.min_suffixes".into(),
            message: "min_suffixes must be >= 0".into(),
        });
    }
    if config.max_prefixes < config.min_prefixes {
        errors.push(FieldError {
            path: "affixes.max_prefixes".into(),
            message: format!(
                "max_prefixes ({}) must be >= min_prefixes ({})",
                config.max_prefixes, config.min_prefixes
            ),
        });
    }
    if config.max_suffixes < config.min_suffixes {
        errors.push(FieldError {
            path: "affixes.max_suffixes".into(),
            message: format!(
                "max_suffixes ({}) must be >= min_suffixes ({})",
                config.max_suffixes, config.min_suffixes
            ),
        });
    }

    if !errors.is_empty() {
        return Err(ProblemResponse::validation_error(
            "Affix pool validation failed",
            errors,
        ));
    }

    let mut prefix_ids = HashSet::new();
    for entry in &config.prefixes {
        if !prefix_ids.insert(entry.affix_id) {
            return Err(ProblemResponse::conflict(format!(
                "duplicate affix_id in prefixes: {}",
                entry.affix_id
            )));
        }
    }

    let mut suffix_ids = HashSet::new();
    for entry in &config.suffixes {
        if !suffix_ids.insert(entry.affix_id) {
            return Err(ProblemResponse::conflict(format!(
                "duplicate affix_id in suffixes: {}",
                entry.affix_id
            )));
        }
    }

    Ok(())
}

async fn validate_affix_pool(
    pool: &sqlx::PgPool,
    client_id: Uuid,
    entries: &[AffixPoolEntry],
    expected_location: AffixLocation,
) -> Result<Vec<AffixPoolEntry>, ProblemResponse> {
    let mut valid_entries = Vec::new();

    for entry in entries {
        let affix_row = sqlx::query(
            "SELECT id, client_id, type::text FROM affixes WHERE id = $1",
        )
        .bind(entry.affix_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "blueprints: affix lookup failed");
            ProblemResponse::unprocessable_entity("Failed to validate affix pool")
        })?;

        match affix_row {
            None => {
                return Err(ProblemResponse::not_found(format!(
                    "Affix not found: {}",
                    entry.affix_id
                )));
            }
            Some(row) => {
                let affix_client_id: Uuid = row.get("client_id");
                if affix_client_id != client_id {
                    return Err(ProblemResponse::not_found(format!(
                        "Affix not found: {}",
                        entry.affix_id
                    )));
                }
                let affix_type_str: String = row.get("type");
                let affix_type = parse_affix_location(&affix_type_str).unwrap_or(AffixLocation::Prefix);
                if affix_type != expected_location {
                    return Err(ProblemResponse::validation_error(
                    format!(
                        "Affix type mismatch: affix {} is of type {}, expected {}",
                        entry.affix_id, affix_location_str(&affix_type), affix_location_str(&expected_location)
                    ),
                    vec![FieldError {
                        path: "affixes".into(),
                        message: format!(
                            "affix {} has type {}, cannot be used as {}",
                            entry.affix_id, affix_location_str(&affix_type), affix_location_str(&expected_location)
                        ),
                        }],
                    ));
                }
                valid_entries.push(entry.clone());
            }
        }
    }

    Ok(valid_entries)
}

async fn fetch_affix_pool(
    pool: &sqlx::PgPool,
    blueprint_id: Uuid,
) -> Result<BlueprintAffixConfig, ProblemResponse> {
    let bp_row = sqlx::query(
        "SELECT min_prefixes, max_prefixes, min_suffixes, max_suffixes FROM blueprints WHERE id = $1",
    )
    .bind(blueprint_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: fetch bp for affix pool failed");
        ProblemResponse::unprocessable_entity("Failed to fetch affix pool")
    })?;

    let (min_prefixes, max_prefixes, min_suffixes, max_suffixes) = if let Some(row) = bp_row {
        (
            row.get("min_prefixes"),
            row.get("max_prefixes"),
            row.get("min_suffixes"),
            row.get("max_suffixes"),
        )
    } else {
        return Err(ProblemResponse::not_found("Blueprint not found"));
    };

    let affix_rows = sqlx::query(
        "SELECT affix_id, weight, location FROM blueprint_affixes \
         WHERE blueprint_id = $1 ORDER BY sort_order ASC",
    )
    .bind(blueprint_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "blueprints: fetch affix pool entries failed");
        ProblemResponse::unprocessable_entity("Failed to fetch affix pool entries")
    })?;

    let mut prefixes = Vec::new();
    let mut suffixes = Vec::new();

    for row in &affix_rows {
        let affix_id: Uuid = row.get("affix_id");
        let weight: f64 = row.get("weight");
        let location_str: String = row.get("location");
        let location = parse_affix_location(&location_str).unwrap_or(AffixLocation::Prefix);

        let entry = AffixPoolEntry { affix_id, weight };
        match location {
            AffixLocation::Prefix => prefixes.push(entry),
            AffixLocation::Suffix => suffixes.push(entry),
        }
    }

    Ok(BlueprintAffixConfig {
        min_prefixes,
        max_prefixes,
        min_suffixes,
        max_suffixes,
        prefixes,
        suffixes,
    })
}

async fn insert_affix_entries(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    blueprint_id: Uuid,
    entries: &[AffixPoolEntry],
    location: AffixLocation,
    sort_order: &mut i32,
) -> Result<(), ProblemResponse> {
    let loc_str = affix_location_str(&location);
    for entry in entries {
        sqlx::query(
            "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(blueprint_id)
        .bind(entry.affix_id)
        .bind(entry.weight)
        .bind(loc_str)
        .bind(*sort_order)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "blueprints: insert blueprint_affixes failed");
            ProblemResponse::unprocessable_entity("Failed to create blueprint affix pool")
        })?;
        *sort_order += 1;
    }
    Ok(())
}

async fn upsert_affix_pool_entries(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    blueprint_id: Uuid,
    entries: &[AffixPoolEntry],
    location: AffixLocation,
    existing_set: &HashMap<(Uuid, AffixLocation), f64>,
    sort_order: &mut i32,
) -> Result<(), ProblemResponse> {
    let loc_str = affix_location_str(&location);
    for entry in entries {
        let key = (entry.affix_id, location.clone());
        if existing_set.contains_key(&key) {
            if (existing_set[&key] - entry.weight).abs() > f64::EPSILON {
                sqlx::query(
                    "UPDATE blueprint_affixes SET weight=$1, sort_order=$2 \
                     WHERE blueprint_id=$3 AND affix_id=$4 AND location=$5",
                )
                .bind(entry.weight)
                .bind(*sort_order)
                .bind(blueprint_id)
                .bind(entry.affix_id)
                .bind(loc_str)
                .execute(&mut **tx)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "blueprints: update affix weight failed");
                    ProblemResponse::unprocessable_entity("Failed to update affix pool")
                })?;
            }
        } else {
            sqlx::query(
                "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(blueprint_id)
            .bind(entry.affix_id)
            .bind(entry.weight)
            .bind(loc_str)
            .bind(*sort_order)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "blueprints: insert affix failed");
                ProblemResponse::unprocessable_entity("Failed to update affix pool")
            })?;
        }
        *sort_order += 1;
    }
    Ok(())
}

fn assemble_response(bp: &Blueprint, affixes: &BlueprintAffixConfig) -> BlueprintResponse {
    BlueprintResponse {
        id: bp.id,
        client_id: bp.client_id,
        name: bp.name.clone(),
        archetype: bp.archetype.clone(),
        weight: bp.weight,
        description: bp.description.clone(),
        attributes: bp.attributes.clone(),
        attribute_order: bp.attribute_order.clone(),
        min_prefixes: bp.min_prefixes,
        max_prefixes: bp.max_prefixes,
        min_suffixes: bp.min_suffixes,
        max_suffixes: bp.max_suffixes,
        affixes: affixes.clone(),
        created_at: bp.created_at,
        updated_at: bp.updated_at,
    }
}

fn validate_blueprint_attributes(
    attributes: &HashMap<String, BlueprintAttribute>,
    attribute_order: &[String],
    weight: f64,
    affix_config: &BlueprintAffixConfig,
) -> Result<(), ProblemResponse> {
    let attributes_json =
        serde_json::to_value(attributes).map_err(|e| {
            ProblemResponse::unprocessable_entity(format!("Invalid attributes: {e}"))
        })?;

    let bp = Blueprint {
        id: Uuid::nil(),
        client_id: Uuid::nil(),
        name: String::new(),
        archetype: String::new(),
        weight,
        description: None,
        attributes: attributes_json,
        attribute_order: attribute_order.to_vec(),
        min_prefixes: affix_config.min_prefixes,
        max_prefixes: affix_config.max_prefixes,
        min_suffixes: affix_config.min_suffixes,
        max_suffixes: affix_config.max_suffixes,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let errors: Vec<FieldError> = validation::validate_blueprint(&bp)
        .into_iter()
        .map(|e| FieldError {
            path: e.path,
            message: e.message,
        })
        .collect();

    if !errors.is_empty() {
        return Err(ProblemResponse::validation_error(
            "Attribute validation failed",
            errors,
        ));
    }

    Ok(())
}

async fn check_duplicate_name(
    pool: &sqlx::PgPool,
    client_id: Uuid,
    name: &str,
    exclude_id: Option<Uuid>,
) -> Result<(), ProblemResponse> {
    let existing = if let Some(exclude) = exclude_id {
        sqlx::query(
            "SELECT id FROM blueprints WHERE client_id = $1 AND name = $2 AND id != $3",
        )
        .bind(client_id)
        .bind(name)
        .bind(exclude)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query("SELECT id FROM blueprints WHERE client_id = $1 AND name = $2")
            .bind(client_id)
            .bind(name)
            .fetch_optional(pool)
            .await
    };

    match existing {
        Ok(Some(_)) => Err(ProblemResponse::conflict(format!(
            "A blueprint named \"{name}\" already exists for this client"
        ))),
        Ok(None) => Ok(()),
        Err(e) => {
            tracing::error!(error = %e, "blueprints: duplicate name check failed");
            Err(ProblemResponse::unprocessable_entity(
                "Failed to check for duplicate blueprint name",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthenticatedKey;
    use crate::pagination::HasId;
    use arche_types::attribute::BlueprintAffixConfig;
    use arche_types::Permission;
    use chrono::Utc;

    fn make_key(
        client_id: Option<Uuid>,
        is_super: bool,
    ) -> AuthenticatedKey {
        AuthenticatedKey {
            id: Uuid::nil(),
            name: "test".into(),
            client_id,
            permissions: vec![Permission::Read],
            is_super,
        }
    }

    #[test]
    fn test_blueprint_has_id() {
        let id = Uuid::new_v4();
        let bp = Blueprint {
            id,
            client_id: Uuid::new_v4(),
            name: "test".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({}),
            attribute_order: vec![],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(bp.id(), id);
    }

    #[test]
    fn test_resolve_client_id_super_admin_no_filter() {
        let key = make_key(None, true);
        let result = resolve_client_id(&key, None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_client_id_super_admin_with_filter() {
        let key = make_key(None, true);
        let cid = Uuid::new_v4();
        let result = resolve_client_id(&key, Some(cid)).unwrap();
        assert_eq!(result, Some(cid));
    }

    #[test]
    fn test_resolve_client_id_regular_key() {
        let cid = Uuid::new_v4();
        let key = make_key(Some(cid), false);
        let result = resolve_client_id(&key, None).unwrap();
        assert_eq!(result, Some(cid));
    }

    #[test]
    fn test_resolve_client_id_regular_key_no_client() {
        let key = make_key(None, false);
        let result = resolve_client_id(&key, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_resolve_client_id_regular_key_ignores_query_param() {
        let cid = Uuid::new_v4();
        let other = Uuid::new_v4();
        let key = make_key(Some(cid), false);
        let result = resolve_client_id(&key, Some(other)).unwrap();
        assert_eq!(result, Some(cid));
    }

    #[test]
    fn test_blueprint_list_query_json_deserialization() {
        let cid = Uuid::new_v4();
        let json = format!(
            r#"{{"cursor":"abc","limit":5,"page":2,"perPage":20,"clientId":"{}","archetype":"sword","search":"fire"}}"#,
            cid
        );
        let query: BlueprintListQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(query.cursor, Some("abc".into()));
        assert_eq!(query.limit, Some(5));
        assert_eq!(query.page, Some(2));
        assert_eq!(query.per_page, Some(20));
        assert_eq!(query.client_id, Some(cid));
        assert_eq!(query.archetype, Some("sword".into()));
        assert_eq!(query.search, Some("fire".into()));
    }

    #[test]
    fn test_blueprint_list_query_empty_json() {
        let query: BlueprintListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.cursor, None);
        assert_eq!(query.limit, None);
        assert_eq!(query.page, None);
        assert_eq!(query.per_page, None);
        assert_eq!(query.client_id, None);
        assert_eq!(query.archetype, None);
        assert_eq!(query.search, None);
    }

    #[test]
    fn test_pagination_params_from_query() {
        let q = BlueprintListQuery {
            cursor: None,
            limit: Some(10),
            page: Some(2),
            per_page: Some(25),
            client_id: None,
            archetype: None,
            search: None,
        };
        let params = crate::pagination::PaginationParams {
            cursor: q.cursor,
            limit: q.limit,
            page: q.page,
            per_page: q.per_page,
        };
        let mode = params.mode().unwrap();
        assert!(matches!(mode, crate::pagination::PaginationMode::Offset { .. }));
        if let crate::pagination::PaginationMode::Offset {
            page,
            per_page,
            offset,
        } = mode
        {
            assert_eq!(page, 2);
            assert_eq!(per_page, 25);
            assert_eq!(offset, 25);
        }
    }

    #[test]
    fn test_pagination_params_cursor_wins() {
        let q = BlueprintListQuery {
            cursor: Some(Uuid::new_v4().to_string()),
            limit: Some(5),
            page: Some(3),
            per_page: Some(10),
            client_id: None,
            archetype: None,
            search: None,
        };
        let params = crate::pagination::PaginationParams {
            cursor: q.cursor,
            limit: q.limit,
            page: q.page,
            per_page: q.per_page,
        };
        let mode = params.mode().unwrap();
        assert!(matches!(mode, crate::pagination::PaginationMode::Cursor { .. }));
    }

    #[test]
    fn test_pagination_params_offset_mode() {
        let q = BlueprintListQuery {
            cursor: None,
            limit: None,
            page: Some(2),
            per_page: Some(15),
            client_id: None,
            archetype: None,
            search: None,
        };
        let params = crate::pagination::PaginationParams {
            cursor: q.cursor,
            limit: q.limit,
            page: q.page,
            per_page: q.per_page,
        };
        let mode = params.mode().unwrap();
        assert!(matches!(mode, crate::pagination::PaginationMode::Offset { .. }));
        if let crate::pagination::PaginationMode::Offset {
            page,
            per_page,
            offset,
        } = mode
        {
            assert_eq!(page, 2);
            assert_eq!(per_page, 15);
            assert_eq!(offset, 15);
        }
    }

    #[test]
    fn test_validate_affix_config_valid() {
        let config = BlueprintAffixConfig {
            min_prefixes: 0,
            max_prefixes: 2,
            min_suffixes: 1,
            max_suffixes: 1,
            prefixes: vec![],
            suffixes: vec![],
        };
        assert!(validate_affix_config(&config).is_ok());
    }

    #[test]
    fn test_validate_affix_config_negative_min_prefixes() {
        let config = BlueprintAffixConfig {
            min_prefixes: -1,
            max_prefixes: 2,
            min_suffixes: 0,
            max_suffixes: 0,
            prefixes: vec![],
            suffixes: vec![],
        };
        let err = validate_affix_config(&config).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn test_validate_affix_config_max_less_than_min_prefixes() {
        let config = BlueprintAffixConfig {
            min_prefixes: 3,
            max_prefixes: 1,
            min_suffixes: 0,
            max_suffixes: 0,
            prefixes: vec![],
            suffixes: vec![],
        };
        let err = validate_affix_config(&config).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn test_validate_affix_config_negative_min_suffixes() {
        let config = BlueprintAffixConfig {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: -1,
            max_suffixes: 2,
            prefixes: vec![],
            suffixes: vec![],
        };
        let err = validate_affix_config(&config).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn test_validate_affix_config_max_less_than_min_suffixes() {
        let config = BlueprintAffixConfig {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 3,
            max_suffixes: 1,
            prefixes: vec![],
            suffixes: vec![],
        };
        let err = validate_affix_config(&config).unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn test_validate_affix_config_duplicate_prefix() {
        let id = Uuid::new_v4();
        let config = BlueprintAffixConfig {
            min_prefixes: 0,
            max_prefixes: 2,
            min_suffixes: 0,
            max_suffixes: 0,
            prefixes: vec![
                AffixPoolEntry {
                    affix_id: id,
                    weight: 1.0,
                },
                AffixPoolEntry {
                    affix_id: id,
                    weight: 0.5,
                },
            ],
            suffixes: vec![],
        };
        let err = validate_affix_config(&config).unwrap_err();
        assert_eq!(err.status, 409);
        assert!(err.detail.unwrap().contains("duplicate"));
    }

    #[test]
    fn test_validate_affix_config_duplicate_suffix() {
        let id = Uuid::new_v4();
        let config = BlueprintAffixConfig {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 2,
            prefixes: vec![],
            suffixes: vec![
                AffixPoolEntry {
                    affix_id: id,
                    weight: 1.0,
                },
                AffixPoolEntry {
                    affix_id: id,
                    weight: 0.5,
                },
            ],
        };
        let err = validate_affix_config(&config).unwrap_err();
        assert_eq!(err.status, 409);
        assert!(err.detail.unwrap().contains("duplicate"));
    }

    #[test]
    fn test_assemble_response() {
        let id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let now = Utc::now();

        let bp = Blueprint {
            id,
            client_id,
            name: "TestSword".into(),
            archetype: "sword".into(),
            weight: 1.5,
            description: Some("A test sword".into()),
            attributes: serde_json::json!({"damage": {}}),
            attribute_order: vec!["damage".into()],
            min_prefixes: 0,
            max_prefixes: 2,
            min_suffixes: 0,
            max_suffixes: 2,
            created_at: now,
            updated_at: now,
        };

        let affixes = BlueprintAffixConfig {
            min_prefixes: 0,
            max_prefixes: 2,
            min_suffixes: 0,
            max_suffixes: 2,
            prefixes: vec![],
            suffixes: vec![],
        };

        let resp = assemble_response(&bp, &affixes);
        assert_eq!(resp.id, id);
        assert_eq!(resp.client_id, client_id);
        assert_eq!(resp.name, "TestSword");
        assert_eq!(resp.archetype, "sword");
        assert_eq!(resp.weight, 1.5);
        assert_eq!(resp.description, Some("A test sword".into()));
        assert_eq!(resp.affixes.min_prefixes, 0);
        assert_eq!(resp.affixes.max_prefixes, 2);
    }

    #[test]
    fn test_resolve_client_id_for_write_regular_key() {
        let cid = Uuid::new_v4();
        let key = make_key(Some(cid), false);
        let result = resolve_client_id_for_write(&key, None).unwrap();
        assert_eq!(result, cid);
    }

    #[test]
    fn test_resolve_client_id_for_write_no_client() {
        let key = make_key(None, false);
        let result = resolve_client_id_for_write(&key, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_resolve_client_id_for_write_super_admin_with_client_id() {
        let cid = Uuid::new_v4();
        let key = make_key(None, true);
        let result = resolve_client_id_for_write(&key, Some(cid)).unwrap();
        assert_eq!(result, cid);
    }

    #[test]
    fn test_resolve_client_id_for_write_super_admin_no_client_id() {
        let key = make_key(None, true);
        let result = resolve_client_id_for_write(&key, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_blueprint_query_default_force_false() {
        let query: DeleteBlueprintQuery = serde_json::from_str("{}").unwrap();
        assert!(!query.force);
    }

    #[test]
    fn test_delete_blueprint_query_force_true() {
        let query: DeleteBlueprintQuery = serde_json::from_str(r#"{"force":true}"#).unwrap();
        assert!(query.force);
    }

    // DB integration tests
    mod db_tests {
        use super::*;
        use crate::AppState;
        use arche_types::attribute::{AffixPoolEntry, BlueprintAffixConfig};
        use arche_types::crud::CreateBlueprintRequest;
        use arche_types::{AffixLocation, Permission};
        use serde_json::json;
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
            location: AffixLocation,
        ) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO affixes (client_id, name, type, attribute) \
                 VALUES ($1, $2, $3, '{}'::jsonb) RETURNING id",
            )
            .bind(client_id)
            .bind(name)
            .bind(affix_location_str(&location))
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

        #[tokio::test]
        async fn test_create_blueprint_with_affix_pool() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let prefix_id = create_test_affix(&pool, client_id, "Fire", AffixLocation::Prefix).await;
            let suffix_id = create_test_affix(&pool, client_id, "of Strength", AffixLocation::Suffix).await;

            let req = CreateBlueprintRequest {
                name: "Sword".into(),
                archetype: "sword".into(),
                weight: 1.0,
                description: None,
                attributes: HashMap::new(),
                attribute_order: vec![],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 1,
                    min_suffixes: 0,
                    max_suffixes: 2,
                    prefixes: vec![AffixPoolEntry {
                        affix_id: prefix_id,
                        weight: 1.0,
                    }],
                    suffixes: vec![AffixPoolEntry {
                        affix_id: suffix_id,
                        weight: 0.5,
                    }],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "create failed: {:?}", result.err());

            let resp = result.unwrap();
            let bp = resp.0;
            assert_eq!(bp.name, "Sword");
            assert_eq!(bp.min_prefixes, 0);
            assert_eq!(bp.max_prefixes, 1);
            assert_eq!(bp.min_suffixes, 0);
            assert_eq!(bp.max_suffixes, 2);
            assert_eq!(bp.affixes.prefixes.len(), 1);
            assert_eq!(bp.affixes.prefixes[0].affix_id, prefix_id);
            assert_eq!(bp.affixes.prefixes[0].weight, 1.0);
            assert_eq!(bp.affixes.suffixes.len(), 1);
            assert_eq!(bp.affixes.suffixes[0].affix_id, suffix_id);
            assert_eq!(bp.affixes.suffixes[0].weight, 0.5);

            let ba_rows = sqlx::query(
                "SELECT COUNT(*) as count FROM blueprint_affixes WHERE blueprint_id = $1",
            )
            .bind(bp.id)
            .fetch_one(&pool)
            .await
            .unwrap();
            let count: i64 = ba_rows.get("count");
            assert_eq!(count, 2);

            sqlx::query("DELETE FROM blueprint_affixes WHERE blueprint_id = $1")
                .bind(bp.id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp.id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM affixes WHERE client_id = $1")
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
        async fn test_update_blueprint_affix_pool_diff() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
                let prefix_a = create_test_affix(&pool, client_id, "Fire", AffixLocation::Prefix).await;
                let prefix_b = create_test_affix(&pool, client_id, "Ice", AffixLocation::Prefix).await;
                let suffix_a = create_test_affix(&pool, client_id, "of Bear", AffixLocation::Suffix).await;
                let suffix_b = create_test_affix(&pool, client_id, "of Wolf", AffixLocation::Suffix).await;

                let bp_id = Uuid::new_v4();
                sqlx::query(
                    "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                     min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
                )
                .bind(bp_id)
                .bind(client_id)
                .bind("OldSword")
                .bind("sword")
                .bind(1.0)
                .bind(&json!({}))
                .bind(&vec!["damage".to_string()])
                .bind(0)
                .bind(2)
                .bind(0)
                .bind(2)
                .execute(&pool)
                .await
                .unwrap();

                sqlx::query(
                    "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                     VALUES ($1,$2,$3,$4,$5)",
                )
                .bind(bp_id)
                .bind(prefix_a)
                .bind(1.0)
                .bind(affix_location_str(&AffixLocation::Prefix))
                .bind(0)
                .execute(&pool)
                .await
                .unwrap();

                sqlx::query(
                    "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                     VALUES ($1,$2,$3,$4,$5)",
                )
                .bind(bp_id)
                .bind(suffix_a)
                .bind(0.5)
                .bind(affix_location_str(&AffixLocation::Suffix))
                .bind(0)
                .execute(&pool)
                .await
                .unwrap();

                let req = CreateBlueprintRequest {
                    name: "NewSword".into(),
                    archetype: "sword".into(),
                    weight: 1.0,
                    description: None,
                    attributes: HashMap::new(),
                    attribute_order: vec!["damage".into()],
                    affixes: BlueprintAffixConfig {
                        min_prefixes: 0,
                        max_prefixes: 1,
                        min_suffixes: 0,
                        max_suffixes: 2,
                        prefixes: vec![AffixPoolEntry {
                            affix_id: prefix_b,
                            weight: 2.0,
                        }],
                        suffixes: vec![
                            AffixPoolEntry {
                                affix_id: suffix_a,
                                weight: 0.25,
                            },
                            AffixPoolEntry {
                                affix_id: suffix_b,
                                weight: 0.75,
                            },
                        ],
                    },
                };

                let state = test_state(Arc::new(pool.clone()));
                let user = test_user(client_id, Permission::Write);

                let result = update_blueprint(
                    axum::extract::State(state),
                    user,
                    axum::extract::Path(bp_id),
                    axum::Json(req),
                )
                .await;

                assert!(result.is_ok(), "update failed: {:?}", result.err());

                let resp = result.unwrap();
                let bp = resp.0;
                assert_eq!(bp.name, "NewSword");

                let ba_rows = sqlx::query(
                    "SELECT affix_id, weight, location FROM blueprint_affixes \
                     WHERE blueprint_id = $1 ORDER BY sort_order ASC",
                )
                .bind(bp.id)
                .fetch_all(&pool)
                .await
                .unwrap();

                assert_eq!(ba_rows.len(), 3);

                let affix_ids: Vec<Uuid> = ba_rows.iter().map(|r| r.get("affix_id")).collect();
                assert!(affix_ids.contains(&prefix_b));
                assert!(!affix_ids.contains(&prefix_a));
                assert!(affix_ids.contains(&suffix_a));
                assert!(affix_ids.contains(&suffix_b));

                let suffix_a_weight: f64 = ba_rows
                    .iter()
                    .find(|r| {
                        let aid: Uuid = r.get("affix_id");
                        aid == suffix_a
                    })
                    .map(|r| r.get("weight"))
                    .unwrap();
                assert!((suffix_a_weight - 0.25).abs() < 1e-9);

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
                sqlx::query("DELETE FROM affixes WHERE client_id = $1")
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
        async fn test_create_blueprint_invalid_affix_id_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
                let client_id = create_test_client(&pool).await;
                let nonexistent = Uuid::new_v4();

                let req = CreateBlueprintRequest {
                    name: "Sword".into(),
                    archetype: "sword".into(),
                    weight: 1.0,
                    description: None,
                    attributes: HashMap::new(),
                    attribute_order: vec![],
                    affixes: BlueprintAffixConfig {
                        min_prefixes: 0,
                        max_prefixes: 1,
                        min_suffixes: 0,
                        max_suffixes: 0,
                        prefixes: vec![AffixPoolEntry {
                            affix_id: nonexistent,
                            weight: 1.0,
                        }],
                        suffixes: vec![],
                    },
                };

                let state = test_state(Arc::new(pool.clone()));
                let user = test_user(client_id, Permission::Write);

                let result = create_blueprint(
                    axum::extract::State(state),
                    user,
                    axum::Json(req),
                )
                .await;

                assert!(result.is_err());
                let err = result.unwrap_err();
                assert_eq!(err.status, 404);
                assert!(err.detail.unwrap().contains("Affix not found"));

                sqlx::query("DELETE FROM affixes WHERE client_id = $1")
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
        async fn test_create_blueprint_affix_wrong_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
                let client_a = create_test_client(&pool).await;
                let client_b = create_test_client(&pool).await;
                let affix_b = create_test_affix(&pool, client_b, "Ice", AffixLocation::Prefix).await;

                let req = CreateBlueprintRequest {
                    name: "Sword".into(),
                    archetype: "sword".into(),
                    weight: 1.0,
                    description: None,
                    attributes: HashMap::new(),
                    attribute_order: vec![],
                    affixes: BlueprintAffixConfig {
                        min_prefixes: 0,
                        max_prefixes: 1,
                        min_suffixes: 0,
                        max_suffixes: 0,
                        prefixes: vec![AffixPoolEntry {
                            affix_id: affix_b,
                            weight: 1.0,
                        }],
                        suffixes: vec![],
                    },
                };

                let state = test_state(Arc::new(pool.clone()));
                let user = test_user(client_a, Permission::Write);

                let result = create_blueprint(
                    axum::extract::State(state),
                    user,
                    axum::Json(req),
                )
                .await;

                assert!(result.is_err());
                let err = result.unwrap_err();
                assert_eq!(err.status, 404);
                assert!(err.detail.unwrap().contains("Affix not found"));

                sqlx::query("DELETE FROM affixes WHERE client_id = $1")
                    .bind(client_a)
                    .execute(&pool)
                    .await
                    .unwrap();
                sqlx::query("DELETE FROM affixes WHERE client_id = $1")
                    .bind(client_b)
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
        async fn test_create_blueprint_duplicate_affix_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
                let client_id = create_test_client(&pool).await;
                let prefix_id = create_test_affix(&pool, client_id, "Fire", AffixLocation::Prefix).await;

                let req = CreateBlueprintRequest {
                    name: "Sword".into(),
                    archetype: "sword".into(),
                    weight: 1.0,
                    description: None,
                    attributes: HashMap::new(),
                    attribute_order: vec![],
                    affixes: BlueprintAffixConfig {
                        min_prefixes: 0,
                        max_prefixes: 2,
                        min_suffixes: 0,
                        max_suffixes: 0,
                        prefixes: vec![
                            AffixPoolEntry {
                                affix_id: prefix_id,
                                weight: 1.0,
                            },
                            AffixPoolEntry {
                                affix_id: prefix_id,
                                weight: 0.5,
                            },
                        ],
                        suffixes: vec![],
                    },
                };

                let state = test_state(Arc::new(pool.clone()));
                let user = test_user(client_id, Permission::Write);

                let result = create_blueprint(
                    axum::extract::State(state),
                    user,
                    axum::Json(req),
                )
                .await;

                assert!(result.is_err());
                let err = result.unwrap_err();
                assert_eq!(err.status, 409);
                assert!(err.detail.unwrap().contains("duplicate"));

                sqlx::query("DELETE FROM affixes WHERE client_id = $1")
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
        async fn test_create_blueprint_invalid_min_max_counts_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
                let client_id = create_test_client(&pool).await;

                let req = CreateBlueprintRequest {
                    name: "Sword".into(),
                    archetype: "sword".into(),
                    weight: 1.0,
                    description: None,
                    attributes: HashMap::new(),
                    attribute_order: vec![],
                    affixes: BlueprintAffixConfig {
                        min_prefixes: 5,
                        max_prefixes: 1,
                        min_suffixes: -1,
                        max_suffixes: 0,
                        prefixes: vec![],
                        suffixes: vec![],
                    },
                };

                let state = test_state(Arc::new(pool.clone()));
                let user = test_user(client_id, Permission::Write);

                let result = create_blueprint(
                    axum::extract::State(state),
                    user,
                    axum::Json(req),
                )
                .await;

                assert!(result.is_err());
                let err = result.unwrap_err();
                assert_eq!(err.status, 400);
                assert!(err.errors.is_some());
                let errors = err.errors.as_ref().unwrap();
                assert!(errors.iter().any(|e| e.path == "affixes.max_prefixes"));
                assert!(errors.iter().any(|e| e.path == "affixes.min_suffixes"));

                sqlx::query("DELETE FROM clients WHERE id = $1")
                    .bind(client_id)
                    .execute(&pool)
                    .await
                    .unwrap();
        }

        #[tokio::test]
        async fn test_get_blueprint_with_affixes() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
                let client_id = create_test_client(&pool).await;
                let prefix_id = create_test_affix(&pool, client_id, "Fire", AffixLocation::Prefix).await;

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
                .bind(2)
                .execute(&pool)
                .await
                .unwrap();

                sqlx::query(
                    "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                     VALUES ($1,$2,$3,$4,$5)",
                )
                .bind(bp_id)
                .bind(prefix_id)
                .bind(1.0)
                .bind(affix_location_str(&AffixLocation::Prefix))
                .bind(0)
                .execute(&pool)
                .await
                .unwrap();

                let state = test_state(Arc::new(pool.clone()));
                let user = test_user(client_id, Permission::Read);

                let result = get_blueprint(
                    axum::extract::State(state),
                    user,
                    axum::extract::Path(bp_id),
                )
                .await;

                assert!(result.is_ok(), "get failed: {:?}", result.err());
                let resp = result.unwrap();
                let bp = resp.0;
                assert_eq!(bp.name, "Sword");
                assert_eq!(bp.affixes.prefixes.len(), 1);
                assert_eq!(bp.affixes.prefixes[0].affix_id, prefix_id);
                assert_eq!(bp.affixes.prefixes[0].weight, 1.0);
                assert_eq!(bp.affixes.suffixes.len(), 0);

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
                sqlx::query("DELETE FROM affixes WHERE client_id = $1")
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
        async fn test_delete_blueprint_no_affixes_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
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
            .bind(2)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_blueprint(
                axum::extract::State(state),
                user,
                axum::extract::Path(bp_id),
                axum::extract::Query(DeleteBlueprintQuery { force: false }),
            )
            .await;

            assert!(result.is_ok(), "delete failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0, json!({"deleted": true}));

            let count: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprints WHERE id = $1",
            )
            .bind(bp_id)
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
        async fn test_delete_blueprint_with_affixes_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let prefix_id = create_test_affix(&pool, client_id, "Fire", AffixLocation::Prefix).await;
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
            .bind(2)
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1,$2,$3,$4,$5)",
            )
            .bind(bp_id)
            .bind(prefix_id)
            .bind(1.0)
            .bind(affix_location_str(&AffixLocation::Prefix))
            .bind(0)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_blueprint(
                axum::extract::State(state),
                user,
                axum::extract::Path(bp_id),
                axum::extract::Query(DeleteBlueprintQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("referenced by 1 affix pool"));

            let count: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprints WHERE id = $1",
            )
            .bind(bp_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
            assert_eq!(count, 1);

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
            sqlx::query("DELETE FROM affixes WHERE client_id = $1")
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
        async fn test_force_delete_blueprint_with_affixes_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let prefix_id = create_test_affix(&pool, client_id, "Fire", AffixLocation::Prefix).await;
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
            .bind(2)
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO blueprint_affixes (blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1,$2,$3,$4,$5)",
            )
            .bind(bp_id)
            .bind(prefix_id)
            .bind(1.0)
            .bind(affix_location_str(&AffixLocation::Prefix))
            .bind(0)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_blueprint(
                axum::extract::State(state),
                user,
                axum::extract::Path(bp_id),
                axum::extract::Query(DeleteBlueprintQuery { force: true }),
            )
            .await;

            assert!(result.is_ok(), "force delete failed: {:?}", result.err());
            let resp = result.unwrap();
            assert_eq!(resp.0, json!({"deleted": true}));

            let count: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprints WHERE id = $1",
            )
            .bind(bp_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
            assert_eq!(count, 0);

            let affix_count: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprint_affixes WHERE blueprint_id = $1",
            )
            .bind(bp_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
            assert_eq!(affix_count, 0);

            sqlx::query("DELETE FROM affixes WHERE client_id = $1")
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
        async fn test_delete_nonexistent_blueprint_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Delete);

            let result = delete_blueprint(
                axum::extract::State(state),
                user,
                axum::extract::Path(Uuid::new_v4()),
                axum::extract::Query(DeleteBlueprintQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains("Blueprint not found"));

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_delete_blueprint_wrong_client_returns_404() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;
            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(bp_id)
            .bind(client_a)
            .bind("Sword")
            .bind("sword")
            .bind(1.0)
            .bind(&json!({}))
            .bind(&vec!["damage".to_string()])
            .bind(0)
            .bind(2)
            .bind(0)
            .bind(2)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_b, Permission::Delete);

            let result = delete_blueprint(
                axum::extract::State(state),
                user,
                axum::extract::Path(bp_id),
                axum::extract::Query(DeleteBlueprintQuery { force: false }),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 404);
            assert!(err.detail.unwrap().contains("Blueprint not found"));

            let count: i64 = sqlx::query(
                "SELECT COUNT(*) FROM blueprints WHERE id = $1",
            )
            .bind(bp_id)
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
            assert_eq!(count, 1);

            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp_id)
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
        async fn test_create_blueprint_with_inline_attributes() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let mut attributes = HashMap::new();
            attributes.insert(
                "damage".into(),
                BlueprintAttribute::Inline(arche_types::attribute::InlineAttributeDef {
                    description: Some("Physical damage".into()),
                    payload: arche_types::attribute::AttributePayload::Range {
                        min: 10.0,
                        max: 23.0,
                        distribution: None,
                    },
                }),
            );

            let req = CreateBlueprintRequest {
                name: "Sword".into(),
                archetype: "sword".into(),
                weight: 1.0,
                description: None,
                attributes,
                attribute_order: vec!["damage".into()],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 0,
                    min_suffixes: 0,
                    max_suffixes: 0,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "create failed: {:?}", result.err());
            let resp = result.unwrap();
            let bp = resp.0;
            assert_eq!(bp.name, "Sword");
            assert_eq!(bp.attribute_order, vec!["damage"]);
            let attrs = bp.attributes.as_object().unwrap();
            assert!(attrs.contains_key("damage"));

            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp.id)
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
        async fn test_create_blueprint_with_ref_id_attributes() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let mut attributes = HashMap::new();
            attributes.insert(
                "rarity".into(),
                BlueprintAttribute::Ref(arche_types::attribute::BlueprintRefAttribute {
                    ref_id: Uuid::new_v4(),
                }),
            );

            let req = CreateBlueprintRequest {
                name: "Bow".into(),
                archetype: "bow".into(),
                weight: 1.0,
                description: None,
                attributes,
                attribute_order: vec!["rarity".into()],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 0,
                    min_suffixes: 0,
                    max_suffixes: 0,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "create failed: {:?}", result.err());
            let resp = result.unwrap();
            let bp = resp.0;
            assert_eq!(bp.name, "Bow");
            let attrs = bp.attributes.as_object().unwrap();
            let rarity = attrs.get("rarity").unwrap();
            assert!(rarity.get("$ref_id").is_some());

            sqlx::query("DELETE FROM blueprints WHERE id = $1")
                .bind(bp.id)
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
        async fn test_create_blueprint_ref_id_and_inline_on_same_key_returns_400() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let req = CreateBlueprintRequest {
                name: "Axe".into(),
                archetype: "axe".into(),
                weight: 1.0,
                description: None,
                attributes: HashMap::from([(
                    "bad_attr".into(),
                    BlueprintAttribute::Inline(arche_types::attribute::InlineAttributeDef {
                        description: None,
                        payload: arche_types::attribute::AttributePayload::Boolean { value: true },
                    }),
                )]),
                attribute_order: vec!["bad_attr".into()],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 0,
                    min_suffixes: 0,
                    max_suffixes: 0,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_blueprint_ref_id_and_inline_same_key_json() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let body = serde_json::json!({
                "name": "Axe",
                "archetype": "axe",
                "weight": 1.0,
                "attributes": {
                    "bad_attr": {
                        "$ref_id": "550e8400-e29b-41d4-a716-446655440000",
                        "value_type": "boolean",
                        "value": true
                    }
                },
                "attributeOrder": ["bad_attr"],
                "affixes": {
                    "minPrefixes": 0,
                    "maxPrefixes": 0,
                    "minSuffixes": 0,
                    "maxSuffixes": 0,
                    "prefixes": [],
                    "suffixes": []
                }
            });
            let req: CreateBlueprintRequest = serde_json::from_value(body).unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.errors.is_some());
            let errors = err.errors.as_ref().unwrap();
            assert!(
                errors.iter().any(|e| e.message.contains("cannot have both")
                    || e.message.contains("$ref_id")),
                "expected $ref_id/inline error, got: {errors:?}"
            );

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_blueprint_invalid_attribute_order_missing_key() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let mut attributes = HashMap::new();
            attributes.insert(
                "damage".into(),
                BlueprintAttribute::Inline(arche_types::attribute::InlineAttributeDef {
                    description: None,
                    payload: arche_types::attribute::AttributePayload::String {
                        min_length: None,
                        max_length: None,
                    },
                }),
            );
            attributes.insert(
                "speed".into(),
                BlueprintAttribute::Inline(arche_types::attribute::InlineAttributeDef {
                    description: None,
                    payload: arche_types::attribute::AttributePayload::Single {
                        value: 5.0,
                        distribution: None,
                    },
                }),
            );

            let req = CreateBlueprintRequest {
                name: "Dagger".into(),
                archetype: "dagger".into(),
                weight: 1.0,
                description: None,
                attributes,
                attribute_order: vec!["damage".into()],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 0,
                    min_suffixes: 0,
                    max_suffixes: 0,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.errors.is_some());
            let errors = err.errors.as_ref().unwrap();
            assert!(
                errors.iter().any(|e| e.message.contains("missing")),
                "expected missing key error, got: {errors:?}"
            );

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_blueprint_invalid_attribute_order_extra_key() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let mut attributes = HashMap::new();
            attributes.insert(
                "damage".into(),
                BlueprintAttribute::Inline(arche_types::attribute::InlineAttributeDef {
                    description: None,
                    payload: arche_types::attribute::AttributePayload::String {
                        min_length: None,
                        max_length: None,
                    },
                }),
            );

            let req = CreateBlueprintRequest {
                name: "Mace".into(),
                archetype: "mace".into(),
                weight: 1.0,
                description: None,
                attributes,
                attribute_order: vec!["damage".into(), "speed".into()],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 0,
                    min_suffixes: 0,
                    max_suffixes: 0,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.errors.is_some());
            let errors = err.errors.as_ref().unwrap();
            assert!(
                errors.iter().any(|e| e.message.contains("extra")),
                "expected extra key error, got: {errors:?}"
            );

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_blueprint_invalid_attribute_order_duplicate_key() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let mut attributes = HashMap::new();
            attributes.insert(
                "damage".into(),
                BlueprintAttribute::Inline(arche_types::attribute::InlineAttributeDef {
                    description: None,
                    payload: arche_types::attribute::AttributePayload::String {
                        min_length: None,
                        max_length: None,
                    },
                }),
            );

            let req = CreateBlueprintRequest {
                name: "Spear".into(),
                archetype: "spear".into(),
                weight: 1.0,
                description: None,
                attributes,
                attribute_order: vec!["damage".into(), "damage".into()],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 0,
                    min_suffixes: 0,
                    max_suffixes: 0,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.status, 400);
            assert!(err.errors.is_some());
            let errors = err.errors.as_ref().unwrap();
            assert!(
                errors.iter().any(|e| e.message.contains("duplicate")),
                "expected duplicate key error, got: {errors:?}"
            );

            sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn test_create_blueprint_duplicate_name_returns_409() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let req = CreateBlueprintRequest {
                name: "Sword".into(),
                archetype: "sword".into(),
                weight: 1.0,
                description: None,
                attributes: HashMap::new(),
                attribute_order: vec![],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 0,
                    min_suffixes: 0,
                    max_suffixes: 0,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result1 = create_blueprint(
                axum::extract::State(test_state(Arc::new(pool.clone()))),
                test_user(client_id, Permission::Write),
                axum::Json(req.clone()),
            )
            .await;
            assert!(result1.is_ok(), "first create failed: {:?}", result1.err());
            let bp_id = result1.unwrap().0.id;

            let result2 = create_blueprint(
                axum::extract::State(state),
                user,
                axum::Json(req),
            )
            .await;

            assert!(result2.is_err());
            let err = result2.unwrap_err();
            assert_eq!(err.status, 409);
            assert!(err.detail.unwrap().contains("already exists"));

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
        async fn test_get_blueprint_returns_full_object_with_attributes() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let bp_id = Uuid::new_v4();
            let attrs = serde_json::json!({
                "damage": {"value_type": "range", "min": 10.0, "max": 23.0},
                "speed": {"value_type": "single", "value": 1.5}
            });
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind("Longsword")
            .bind("sword")
            .bind(1.5)
            .bind(&attrs)
            .bind(&vec!["damage".to_string(), "speed".to_string()])
            .bind(0)
            .bind(2)
            .bind(0)
            .bind(1)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Read);

            let result = get_blueprint(
                axum::extract::State(state),
                user,
                axum::extract::Path(bp_id),
            )
            .await;

            assert!(result.is_ok(), "get failed: {:?}", result.err());
            let resp = result.unwrap();
            let bp = resp.0;
            assert_eq!(bp.name, "Longsword");
            assert_eq!(bp.archetype, "sword");
            assert_eq!(bp.weight, 1.5);
            assert_eq!(bp.attribute_order, vec!["damage", "speed"]);
            assert_eq!(bp.min_prefixes, 0);
            assert_eq!(bp.max_prefixes, 2);
            assert_eq!(bp.min_suffixes, 0);
            assert_eq!(bp.max_suffixes, 1);
            let bp_attrs = bp.attributes.as_object().unwrap();
            assert!(bp_attrs.contains_key("damage"));
            assert!(bp_attrs.contains_key("speed"));

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
        async fn test_update_blueprint_succeeds() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind("OldSword")
            .bind("sword")
            .bind(1.0)
            .bind(&json!({"damage": {"value_type": "string"}}))
            .bind(&vec!["damage".to_string()])
            .bind(0)
            .bind(0)
            .bind(0)
            .bind(0)
            .execute(&pool)
            .await
            .unwrap();

            let mut attributes = HashMap::new();
            attributes.insert(
                "damage".into(),
                BlueprintAttribute::Inline(arche_types::attribute::InlineAttributeDef {
                    description: None,
                    payload: arche_types::attribute::AttributePayload::Range {
                        min: 5.0,
                        max: 15.0,
                        distribution: None,
                    },
                }),
            );

            let req = CreateBlueprintRequest {
                name: "NewSword".into(),
                archetype: "sword".into(),
                weight: 2.0,
                description: Some("An updated sword".into()),
                attributes,
                attribute_order: vec!["damage".into()],
                affixes: BlueprintAffixConfig {
                    min_prefixes: 0,
                    max_prefixes: 1,
                    min_suffixes: 1,
                    max_suffixes: 2,
                    prefixes: vec![],
                    suffixes: vec![],
                },
            };

            let state = test_state(Arc::new(pool.clone()));
            let user = test_user(client_id, Permission::Write);

            let result = update_blueprint(
                axum::extract::State(state),
                user,
                axum::extract::Path(bp_id),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "update failed: {:?}", result.err());
            let resp = result.unwrap();
            let bp = resp.0;
            assert_eq!(bp.name, "NewSword");
            assert_eq!(bp.weight, 2.0);
            assert_eq!(bp.description, Some("An updated sword".into()));
            assert_eq!(bp.min_prefixes, 0);
            assert_eq!(bp.max_prefixes, 1);
            assert_eq!(bp.min_suffixes, 1);
            assert_eq!(bp.max_suffixes, 2);

            let db_row = sqlx::query("SELECT name, weight, description FROM blueprints WHERE id = $1")
                .bind(bp_id)
                .fetch_one(&pool)
                .await
                .unwrap();
            let db_name: String = db_row.get("name");
            let db_weight: f64 = db_row.get("weight");
            let db_desc: Option<String> = db_row.get("description");
            assert_eq!(db_name, "NewSword");
            assert!((db_weight - 2.0).abs() < 1e-9);
            assert_eq!(db_desc, Some("An updated sword".into()));

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
    }
}

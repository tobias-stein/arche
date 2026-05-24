use std::collections::HashMap;
use std::io::Cursor;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::body::Bytes;
use dashmap::DashMap;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;
use zip::ZipArchive;

use arche_types::export_import::*;
use arche_types::*;

use crate::auth::permission::CurrentUser;
use crate::cache::Cache;
use crate::error::ProblemResponse;

const STAGING_TTL: Duration = Duration::from_secs(300);

pub enum ImportParseResponse {
    Success(ImportSuccessResponse),
    Conflict(serde_json::Value),
}

impl IntoResponse for ImportParseResponse {
    fn into_response(self) -> Response {
        match self {
            ImportParseResponse::Success(success) => {
                (StatusCode::OK, Json(success)).into_response()
            }
            ImportParseResponse::Conflict(body) => {
                let response = (
                    StatusCode::from_u16(409).unwrap(),
                    [("content-type", "application/problem+json")],
                    body.to_string(),
                );
                response.into_response()
            }
        }
    }
}

#[derive(Debug)]
pub struct ImportStaging {
    data: DashMap<String, (Instant, StagedImport)>,
}

impl ImportStaging {
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
        }
    }

    pub fn store(&self, token: String, import: StagedImport) {
        self.data.insert(token, (Instant::now(), import));
    }

    pub fn take(&self, token: &str) -> Result<StagedImport, ProblemResponse> {
        let entry = self.data.remove(token);
        self.cleanup();
        match entry {
            Some((_, (ts, import))) if ts.elapsed() <= STAGING_TTL => Ok(import),
            Some(_) => Err(ProblemResponse::not_found(
                "Import token has expired. Please re-upload the archive.",
            )),
            None => Err(ProblemResponse::not_found(
                "Invalid or expired import token",
            )),
        }
    }

    fn cleanup(&self) {
        self.data
            .retain(|_, (ts, _)| ts.elapsed() <= STAGING_TTL);
    }
}

#[derive(Debug, Clone)]
pub struct StagedImport {
    pub clients: Vec<ClientImport>,
    pub conflicts: Vec<ConflictDetail>,
}

#[derive(Debug, Clone)]
pub struct ClientImport {
    pub client_name: String,
    pub client_id: Option<Uuid>,
    pub blueprints: Vec<Blueprint>,
    pub affixes: Vec<Affix>,
    pub global_meta_attributes: Vec<GlobalMetaAttribute>,
    pub blueprint_affixes: Vec<BlueprintAffix>,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ClientJson {
    id: Uuid,
    name: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct BlueprintJson {
    id: Uuid,
    name: String,
    archetype: String,
    weight: f64,
    description: Option<String>,
    attributes: serde_json::Value,
    attribute_order: Vec<String>,
    min_prefixes: i32,
    max_prefixes: i32,
    min_suffixes: i32,
    max_suffixes: i32,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AffixJson {
    id: Uuid,
    name: String,
    #[serde(rename = "type")]
    affix_type: String,
    description: Option<String>,
    attribute: serde_json::Value,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GmaJson {
    id: Uuid,
    name: String,
    description: Option<String>,
    value_type: String,
    payload: serde_json::Value,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct BaJson {
    id: Uuid,
    blueprint_id: Uuid,
    affix_id: Uuid,
    weight: f64,
    location: String,
    sort_order: i32,
}

pub async fn import_parse_handler(
    State(state): State<crate::AppState>,
    _user: CurrentUser,
    body: Bytes,
) -> Result<ImportParseResponse, ProblemResponse> {
    let staging = &state.import_staging;
    let pool = &state.pool;

    let parsed = parse_zip(body).map_err(|e| {
        ProblemResponse::validation_error(format!("Invalid ZIP archive: {}", e), vec![])
    })?;

    let existing_cache = state.cache.read().await;

    let mut client_imports = Vec::new();
    let mut all_conflicts = Vec::new();

    for client_dir in &parsed {
        let client_import = build_client_import(client_dir)?;
        let conflicts =
            detect_client_conflicts(&client_import, &existing_cache).await?;
        if !conflicts.is_empty() {
            all_conflicts.extend(conflicts);
        }
        client_imports.push(client_import);
    }

    drop(existing_cache);

    if !all_conflicts.is_empty() {
        let token = Uuid::new_v4().to_string();

        let staged = StagedImport {
            clients: client_imports,
            conflicts: all_conflicts.clone(),
        };
        staging.store(token.clone(), staged);

        let conflict_response = ImportConflictResponse {
            problem: arche_types::common::ProblemJson {
                type_: "/errors/import-conflict".into(),
                title: "Import conflicts require resolution".into(),
                status: 409,
                detail: Some(format!(
                    "{} conflict(s) detected. Submit resolutions to POST /api/import/resolve",
                    all_conflicts.len()
                )),
            },
            conflicts: all_conflicts,
            import_token: token,
        };

        let body = serde_json::to_value(&conflict_response).unwrap_or_default();
        return Ok(ImportParseResponse::Conflict(body));
    }

    let clients_created = commit_import(&client_imports, pool, &state).await
        .map_err(|e| ProblemResponse::unprocessable_entity(format!("Import commit failed: {}", e)))?;

    Ok(ImportParseResponse::Success(ImportSuccessResponse {
        status: "imported".into(),
        clients_created,
        resources_imported: count_resources(&client_imports),
    }))
}

pub async fn import_resolve_handler(
    State(state): State<crate::AppState>,
    _user: CurrentUser,
    Json(request): Json<ConflictResolutionRequest>,
) -> Result<Json<ImportSuccessResponse>, ProblemResponse> {
    let staging = &state.import_staging;
    let pool = &state.pool;

    let mut staged = staging.take(&request.import_token)?;

    apply_resolutions(
        &mut staged.clients,
        &staged.conflicts,
        &request.resolutions,
    )?;

    let clients_created = commit_import(&staged.clients, pool, &state).await
        .map_err(|e| ProblemResponse::unprocessable_entity(format!("Import commit failed: {}", e)))?;

    info!(
        clients_created,
        clients = staged.clients.len(),
        "import: resolved and committed"
    );

    Ok(Json(ImportSuccessResponse {
        status: "imported".into(),
        clients_created,
        resources_imported: count_resources(&staged.clients),
    }))
}

fn apply_resolutions(
    clients: &mut [ClientImport],
    conflicts: &[ConflictDetail],
    resolutions: &HashMap<Uuid, ResourceResolution>,
) -> Result<(), ProblemResponse> {
    if conflicts.is_empty() {
        return Ok(());
    }

    for conflict in conflicts {
        let resolution = resolutions.get(&conflict.resource_id).ok_or_else(|| {
            ProblemResponse::validation_error(
                format!(
                    "Missing resolution for conflicting resource '{}' ({})",
                    conflict.resource_name, conflict.resource_id
                ),
                vec![],
            )
        })?;

        match resolution.strategy {
            ResolutionStrategy::KeepOld => {
                remove_resource(clients, conflict);
            }
            ResolutionStrategy::KeepNew => {
                // Import version wins - already in client import data
            }
            ResolutionStrategy::PerAttribute => {
                apply_per_attribute_resolution(clients, conflict, resolution)?;
            }
        }
    }

    Ok(())
}

fn remove_resource(clients: &mut [ClientImport], conflict: &ConflictDetail) {
    for client in clients.iter_mut() {
        match conflict.resource_type.as_str() {
            "blueprint" => {
                client.blueprints.retain(|b| b.id != conflict.resource_id);
                client
                    .blueprint_affixes
                    .retain(|ba| ba.blueprint_id != conflict.resource_id);
            }
            "affix" => {
                client.affixes.retain(|a| a.id != conflict.resource_id);
                client
                    .blueprint_affixes
                    .retain(|ba| ba.affix_id != conflict.resource_id);
            }
            "global_meta_attribute" => {
                client
                    .global_meta_attributes
                    .retain(|g| g.id != conflict.resource_id);
            }
            _ => {}
        }
    }
}

fn apply_per_attribute_resolution(
    clients: &mut [ClientImport],
    conflict: &ConflictDetail,
    resolution: &ResourceResolution,
) -> Result<(), ProblemResponse> {
    let attr_resolutions = resolution.attributes.as_ref().ok_or_else(|| {
        ProblemResponse::validation_error(
            format!(
                "perAttribute resolution requires 'attributes' map for resource '{}'",
                conflict.resource_name
            ),
            vec![],
        )
    })?;

    for attr_conflict in &conflict.attributes {
        let attr_res = attr_resolutions.get(&attr_conflict.key).ok_or_else(|| {
            ProblemResponse::validation_error(
                format!(
                    "Missing resolution for attribute '{}' on resource '{}'",
                    attr_conflict.key, conflict.resource_name
                ),
                vec![],
            )
        })?;

        match attr_res {
            ResolutionStrategy::KeepNew => {
                // Import value wins - already in client import data
            }
            ResolutionStrategy::KeepOld => {
                revert_attribute(clients, conflict, attr_conflict);
            }
            ResolutionStrategy::PerAttribute => {
                return Err(ProblemResponse::validation_error(
                    "Nested perAttribute is not supported".into(),
                    vec![],
                ));
            }
        }
    }

    Ok(())
}

fn revert_attribute(
    clients: &mut [ClientImport],
    conflict: &ConflictDetail,
    attr: &ConflictAttribute,
) {
    for client in clients.iter_mut() {
        match conflict.resource_type.as_str() {
            "blueprint" => {
                for bp in client.blueprints.iter_mut() {
                    if bp.id != conflict.resource_id {
                        continue;
                    }
                    apply_blueprint_field(bp, &attr.key, &attr.old_value);
                }
            }
            "affix" => {
                for a in client.affixes.iter_mut() {
                    if a.id != conflict.resource_id {
                        continue;
                    }
                    apply_affix_field(a, &attr.key, &attr.old_value);
                }
            }
            "global_meta_attribute" => {
                for gma in client.global_meta_attributes.iter_mut() {
                    if gma.id != conflict.resource_id {
                        continue;
                    }
                    apply_gma_field(gma, &attr.key, &attr.old_value);
                }
            }
            _ => {}
        }
    }
}

fn apply_blueprint_field(bp: &mut Blueprint, key: &str, value: &serde_json::Value) {
    match key {
        "name" => bp.name = value.as_str().unwrap_or("").to_string(),
        "archetype" => bp.archetype = value.as_str().unwrap_or("").to_string(),
        "weight" => bp.weight = value.as_f64().unwrap_or(1.0),
        "description" => bp.description = opt_string(value),
        "attributes" => bp.attributes = value.clone(),
        "attribute_order" => {
            bp.attribute_order = serde_json::from_value(value.clone()).unwrap_or_default()
        }
        "min_prefixes" => bp.min_prefixes = value.as_i64().unwrap_or(0) as i32,
        "max_prefixes" => bp.max_prefixes = value.as_i64().unwrap_or(0) as i32,
        "min_suffixes" => bp.min_suffixes = value.as_i64().unwrap_or(0) as i32,
        "max_suffixes" => bp.max_suffixes = value.as_i64().unwrap_or(0) as i32,
        _ => {}
    }
}

fn apply_affix_field(a: &mut Affix, key: &str, value: &serde_json::Value) {
    match key {
        "name" => a.name = value.as_str().unwrap_or("").to_string(),
        "description" => a.description = opt_string(value),
        "type" | "location" => {
            a.location = match value.as_str().unwrap_or("prefix") {
                "suffix" => AffixLocation::Suffix,
                _ => AffixLocation::Prefix,
            }
        }
        "attribute" => a.attribute = value.clone(),
        _ => {}
    }
}

fn apply_gma_field(gma: &mut GlobalMetaAttribute, key: &str, value: &serde_json::Value) {
    match key {
        "name" => gma.name = value.as_str().unwrap_or("").to_string(),
        "description" => gma.description = opt_string(value),
        "value_type" => {
            gma.value_type = match value.as_str().unwrap_or("range") {
                "single" => ValueType::Single,
                "enum" => ValueType::Enum,
                "range" => ValueType::Range,
                "string" => ValueType::String,
                "boolean" => ValueType::Boolean,
                _ => ValueType::Range,
            }
        }
        "payload" => gma.payload = value.clone(),
        _ => {}
    }
}

fn opt_string(value: &serde_json::Value) -> Option<String> {
    if value.is_null() {
        None
    } else {
        Some(value.as_str().unwrap_or("").to_string())
    }
}

fn parse_zip(body: Bytes) -> Result<Vec<Vec<(String, Vec<u8>)>>, String> {
    let cursor = Cursor::new(body.as_ref());
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Failed to read ZIP: {}", e))?;

    let mut client_folders: HashMap<String, Vec<(String, Vec<u8>)>> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("ZIP entry error: {}", e))?;

        if file.is_dir() {
            continue;
        }

        let name = file.name().to_string();

        if name == "manifest.json" {
            continue;
        }

        let parts: Vec<&str> = name.splitn(2, '/').collect();
        if parts.len() != 2 {
            continue;
        }

        let folder_name = parts[0].to_string();
        let file_name = parts[1].to_string();

        if file_name.is_empty() {
            continue;
        }

        let mut contents = Vec::new();
        std::io::copy(&mut file, &mut contents)
            .map_err(|e| format!("Failed to read ZIP entry '{}': {}", name, e))?;

        client_folders
            .entry(folder_name)
            .or_default()
            .push((file_name, contents));
    }

    if client_folders.is_empty() {
        return Err("No client folders found in archive".into());
    }

    Ok(client_folders.into_values().collect())
}

fn build_client_import(files: &[(String, Vec<u8>)]) -> Result<ClientImport, ProblemResponse> {
    let mut client_id: Option<Uuid> = None;
    let mut client_name = String::new();
    let mut blueprints = Vec::new();
    let mut affixes = Vec::new();
    let mut global_meta_attributes = Vec::new();
    let mut blueprint_affixes = Vec::new();

    let ts = chrono::Utc::now();

    for (name, contents) in files {
        match name.as_str() {
            "client.json" => {
                let cj: ClientJson = serde_json::from_slice(contents).map_err(|e| {
                    ProblemResponse::validation_error(
                        format!("Invalid client.json: {}", e),
                        vec![],
                    )
                })?;
                client_id = Some(cj.id);
                client_name = cj.name;
            }
            "blueprints.json" => {
                let bps: Vec<BlueprintJson> =
                    serde_json::from_slice(contents).map_err(|e| {
                        ProblemResponse::validation_error(
                            format!("Invalid blueprints.json: {}", e),
                            vec![],
                        )
                    })?;
                for bp in bps {
                    blueprints.push(Blueprint {
                        id: bp.id,
                        client_id: client_id.unwrap_or(Uuid::nil()),
                        name: bp.name,
                        archetype: bp.archetype,
                        weight: bp.weight,
                        description: bp.description,
                        attributes: bp.attributes,
                        attribute_order: bp.attribute_order,
                        min_prefixes: bp.min_prefixes,
                        max_prefixes: bp.max_prefixes,
                        min_suffixes: bp.min_suffixes,
                        max_suffixes: bp.max_suffixes,
                        created_at: ts,
                        updated_at: ts,
                    });
                }
            }
            "affixes.json" => {
                let afs: Vec<AffixJson> = serde_json::from_slice(contents).map_err(|e| {
                    ProblemResponse::validation_error(
                        format!("Invalid affixes.json: {}", e),
                        vec![],
                    )
                })?;
                for af in afs {
                    let location = match af.affix_type.as_str() {
                        "prefix" => AffixLocation::Prefix,
                        "suffix" => AffixLocation::Suffix,
                        other => {
                            return Err(ProblemResponse::validation_error(
                                format!("Invalid affix type '{}'", other),
                                vec![],
                            ));
                        }
                    };
                    affixes.push(Affix {
                        id: af.id,
                        client_id: client_id.unwrap_or(Uuid::nil()),
                        name: af.name,
                        location,
                        description: af.description,
                        attribute: af.attribute,
                        created_at: ts,
                        updated_at: ts,
                    });
                }
            }
            "global_meta_attributes.json" => {
                let gmas: Vec<GmaJson> =
                    serde_json::from_slice(contents).map_err(|e| {
                        ProblemResponse::validation_error(
                            format!("Invalid global_meta_attributes.json: {}", e),
                            vec![],
                        )
                    })?;
                for gm in gmas {
                    let value_type = match gm.value_type.as_str() {
                        "single" => ValueType::Single,
                        "enum" => ValueType::Enum,
                        "range" => ValueType::Range,
                        "string" => ValueType::String,
                        "boolean" => ValueType::Boolean,
                        other => {
                            return Err(ProblemResponse::validation_error(
                                format!("Invalid value_type '{}'", other),
                                vec![],
                            ));
                        }
                    };
                    global_meta_attributes.push(GlobalMetaAttribute {
                        id: gm.id,
                        client_id: client_id.unwrap_or(Uuid::nil()),
                        name: gm.name,
                        description: gm.description,
                        value_type,
                        payload: gm.payload,
                        created_at: ts,
                        updated_at: ts,
                    });
                }
            }
            "blueprint_affixes.json" => {
                let bas: Vec<BaJson> = serde_json::from_slice(contents).map_err(|e| {
                    ProblemResponse::validation_error(
                        format!("Invalid blueprint_affixes.json: {}", e),
                        vec![],
                    )
                })?;
                for ba in bas {
                    let location = match ba.location.as_str() {
                        "prefix" => AffixLocation::Prefix,
                        "suffix" => AffixLocation::Suffix,
                        other => {
                            return Err(ProblemResponse::validation_error(
                                format!("Invalid location '{}'", other),
                                vec![],
                            ));
                        }
                    };
                    blueprint_affixes.push(BlueprintAffix {
                        id: ba.id,
                        blueprint_id: ba.blueprint_id,
                        affix_id: ba.affix_id,
                        weight: ba.weight,
                        location,
                        sort_order: ba.sort_order,
                    });
                }
            }
            _ => {}
        }
    }

    if client_name.is_empty() {
        return Err(ProblemResponse::validation_error(
            "Missing client.json in client folder",
            vec![],
        ));
    }

    Ok(ClientImport {
        client_name,
        client_id,
        blueprints,
        affixes,
        global_meta_attributes,
        blueprint_affixes,
    })
}

async fn detect_client_conflicts(
    import: &ClientImport,
    cache: &Cache,
) -> Result<Vec<ConflictDetail>, ProblemResponse> {
    let mut conflicts = Vec::new();

    let client_id = import.client_id;

    if let Some(cid) = client_id {
        if let Some(existing_client) = cache.clients.get(&cid) {
            if existing_client.name != import.client_name {
                let detail = ConflictDetail {
                    resource_type: "client".into(),
                    resource_id: cid,
                    resource_name: existing_client.name.clone(),
                    attributes: vec![ConflictAttribute {
                        key: "name".into(),
                        old_value: serde_json::Value::String(existing_client.name.clone()),
                        new_value: serde_json::Value::String(import.client_name.clone()),
                        value_type: ValueType::String,
                    }],
                };
                if !conflicts.iter().any(|c: &ConflictDetail| c.resource_id == cid) {
                    conflicts.push(detail);
                }
            }
        }
    }

    for bp in &import.blueprints {
        if let Some(existing) = cache.blueprints.get(&bp.id) {
            let diffs = compare_blueprints(existing, bp);
            if !diffs.is_empty() {
                conflicts.push(ConflictDetail {
                    resource_type: "blueprint".into(),
                    resource_id: bp.id,
                    resource_name: bp.name.clone(),
                    attributes: diffs,
                });
            }
        }
    }

    for a in &import.affixes {
        if let Some(existing) = cache.affixes.get(&a.id) {
            let diffs = compare_affixes(existing, a);
            if !diffs.is_empty() {
                conflicts.push(ConflictDetail {
                    resource_type: "affix".into(),
                    resource_id: a.id,
                    resource_name: a.name.clone(),
                    attributes: diffs,
                });
            }
        }
    }

    for gma in &import.global_meta_attributes {
        if let Some(existing) = cache.global_meta_attributes.get(&gma.id) {
            let diffs = compare_gmas(existing, gma);
            if !diffs.is_empty() {
                conflicts.push(ConflictDetail {
                    resource_type: "global_meta_attribute".into(),
                    resource_id: gma.id,
                    resource_name: gma.name.clone(),
                    attributes: diffs,
                });
            }
        }
    }

    Ok(conflicts)
}

fn compare_blueprints(existing: &Blueprint, imported: &Blueprint) -> Vec<ConflictAttribute> {
    let mut diffs = Vec::new();

    compare_field(
        &mut diffs,
        "name",
        &serde_json::Value::String(existing.name.clone()),
        &serde_json::Value::String(imported.name.clone()),
    );
    compare_field(
        &mut diffs,
        "archetype",
        &serde_json::Value::String(existing.archetype.clone()),
        &serde_json::Value::String(imported.archetype.clone()),
    );
    compare_field_json(
        &mut diffs,
        "weight",
        &serde_json::json!(existing.weight),
        &serde_json::json!(imported.weight),
    );
    compare_field_opt(
        &mut diffs,
        "description",
        &existing.description.clone().map(serde_json::Value::String),
        &imported.description.clone().map(serde_json::Value::String),
    );
    compare_field_json(&mut diffs, "attributes", &existing.attributes, &imported.attributes);
    compare_field_json(
        &mut diffs,
        "attribute_order",
        &serde_json::to_value(&existing.attribute_order).unwrap_or_default(),
        &serde_json::to_value(&imported.attribute_order).unwrap_or_default(),
    );
    compare_field_json(
        &mut diffs,
        "min_prefixes",
        &serde_json::json!(existing.min_prefixes),
        &serde_json::json!(imported.min_prefixes),
    );
    compare_field_json(
        &mut diffs,
        "max_prefixes",
        &serde_json::json!(existing.max_prefixes),
        &serde_json::json!(imported.max_prefixes),
    );
    compare_field_json(
        &mut diffs,
        "min_suffixes",
        &serde_json::json!(existing.min_suffixes),
        &serde_json::json!(imported.min_suffixes),
    );
    compare_field_json(
        &mut diffs,
        "max_suffixes",
        &serde_json::json!(existing.max_suffixes),
        &serde_json::json!(imported.max_suffixes),
    );

    diffs
}

fn compare_affixes(existing: &Affix, imported: &Affix) -> Vec<ConflictAttribute> {
    let mut diffs = Vec::new();

    compare_field(
        &mut diffs,
        "name",
        &serde_json::Value::String(existing.name.clone()),
        &serde_json::Value::String(imported.name.clone()),
    );
    compare_field(
        &mut diffs,
        "type",
        &serde_json::Value::String(affix_location_str(&existing.location)),
        &serde_json::Value::String(affix_location_str(&imported.location)),
    );
    compare_field_opt(
        &mut diffs,
        "description",
        &existing.description.clone().map(serde_json::Value::String),
        &imported.description.clone().map(serde_json::Value::String),
    );
    compare_field_json(&mut diffs, "attribute", &existing.attribute, &imported.attribute);

    diffs
}

fn compare_gmas(
    existing: &GlobalMetaAttribute,
    imported: &GlobalMetaAttribute,
) -> Vec<ConflictAttribute> {
    let mut diffs = Vec::new();

    compare_field(
        &mut diffs,
        "name",
        &serde_json::Value::String(existing.name.clone()),
        &serde_json::Value::String(imported.name.clone()),
    );
    compare_field_opt(
        &mut diffs,
        "description",
        &existing.description.clone().map(serde_json::Value::String),
        &imported.description.clone().map(serde_json::Value::String),
    );
    compare_field(
        &mut diffs,
        "value_type",
        &serde_json::Value::String(value_type_str(&existing.value_type)),
        &serde_json::Value::String(value_type_str(&imported.value_type)),
    );
    compare_field_json(&mut diffs, "payload", &existing.payload, &imported.payload);

    diffs
}

fn compare_field(
    diffs: &mut Vec<ConflictAttribute>,
    key: &str,
    old: &serde_json::Value,
    new: &serde_json::Value,
) {
    if old != new {
        diffs.push(ConflictAttribute {
            key: key.to_string(),
            old_value: old.clone(),
            new_value: new.clone(),
            value_type: ValueType::String,
        });
    }
}

fn compare_field_json(
    diffs: &mut Vec<ConflictAttribute>,
    key: &str,
    old: &serde_json::Value,
    new: &serde_json::Value,
) {
    if old != new {
        diffs.push(ConflictAttribute {
            key: key.to_string(),
            old_value: old.clone(),
            new_value: new.clone(),
            value_type: ValueType::Range,
        });
    }
}

fn compare_field_opt(
    diffs: &mut Vec<ConflictAttribute>,
    key: &str,
    old: &Option<serde_json::Value>,
    new: &Option<serde_json::Value>,
) {
    let old_val = old.clone().unwrap_or(serde_json::Value::Null);
    let new_val = new.clone().unwrap_or(serde_json::Value::Null);
    if old_val != new_val {
        diffs.push(ConflictAttribute {
            key: key.to_string(),
            old_value: old_val,
            new_value: new_val,
            value_type: ValueType::String,
        });
    }
}

fn affix_location_str(loc: &AffixLocation) -> String {
    match loc {
        AffixLocation::Prefix => "prefix".into(),
        AffixLocation::Suffix => "suffix".into(),
    }
}

fn value_type_str(vt: &ValueType) -> String {
    match vt {
        ValueType::Single => "single".into(),
        ValueType::Enum => "enum".into(),
        ValueType::Range => "range".into(),
        ValueType::String => "string".into(),
        ValueType::Boolean => "boolean".into(),
    }
}

async fn commit_import(
    clients: &[ClientImport],
    pool: &PgPool,
    state: &crate::AppState,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let mut clients_created: i64 = 0;
    let mut affected_client_ids: Vec<Uuid> = Vec::new();

    let mut tx = pool.begin().await?;

    for client in clients {
        let client_id = client.client_id.unwrap_or_else(Uuid::new_v4);

        let existing = sqlx::query_scalar::<_, Uuid>("SELECT id FROM clients WHERE id = $1")
            .bind(client_id)
            .fetch_optional(&mut *tx)
            .await?;

        let effective_client_id = if let Some(cid) = existing {
            cid
        } else {
            sqlx::query("INSERT INTO clients (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(client_id)
                .bind(&client.client_name)
                .execute(&mut *tx)
                .await?;
            clients_created += 1;
            client_id
        };

        affected_client_ids.push(effective_client_id);

        for gma in &client.global_meta_attributes {
            let vt = value_type_str(&gma.value_type);
            sqlx::query(
                "INSERT INTO global_meta_attributes (id, client_id, name, description, value_type, payload) \
                 VALUES ($1, $2, $3, $4, $5::value_type, $6) \
                 ON CONFLICT (id) DO UPDATE SET \
                 name = EXCLUDED.name, description = EXCLUDED.description, \
                 value_type = EXCLUDED.value_type, payload = EXCLUDED.payload, \
                 updated_at = now()",
            )
            .bind(gma.id)
            .bind(effective_client_id)
            .bind(&gma.name)
            .bind(&gma.description)
            .bind(vt)
            .bind(&gma.payload)
            .execute(&mut *tx)
            .await?;
        }

        for bp in &client.blueprints {
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, description, \
                 attributes, attribute_order, min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
                 ON CONFLICT (id) DO UPDATE SET \
                 name = EXCLUDED.name, archetype = EXCLUDED.archetype, \
                 weight = EXCLUDED.weight, description = EXCLUDED.description, \
                 attributes = EXCLUDED.attributes, attribute_order = EXCLUDED.attribute_order, \
                 min_prefixes = EXCLUDED.min_prefixes, max_prefixes = EXCLUDED.max_prefixes, \
                 min_suffixes = EXCLUDED.min_suffixes, max_suffixes = EXCLUDED.max_suffixes, \
                 updated_at = now()",
            )
            .bind(bp.id)
            .bind(effective_client_id)
            .bind(&bp.name)
            .bind(&bp.archetype)
            .bind(bp.weight)
            .bind(&bp.description)
            .bind(&bp.attributes)
            .bind(&bp.attribute_order)
            .bind(bp.min_prefixes)
            .bind(bp.max_prefixes)
            .bind(bp.min_suffixes)
            .bind(bp.max_suffixes)
            .execute(&mut *tx)
            .await?;
        }

        for a in &client.affixes {
            let loc = affix_location_str(&a.location);
            sqlx::query(
                "INSERT INTO affixes (id, client_id, name, type, description, attribute) \
                 VALUES ($1, $2, $3, $4::affix_location, $5, $6) \
                 ON CONFLICT (id) DO UPDATE SET \
                 name = EXCLUDED.name, type = EXCLUDED.type, \
                 description = EXCLUDED.description, attribute = EXCLUDED.attribute, \
                 updated_at = now()",
            )
            .bind(a.id)
            .bind(effective_client_id)
            .bind(&a.name)
            .bind(loc)
            .bind(&a.description)
            .bind(&a.attribute)
            .execute(&mut *tx)
            .await?;
        }

        for ba in &client.blueprint_affixes {
            let loc = affix_location_str(&ba.location);
            sqlx::query(
                "INSERT INTO blueprint_affixes (id, blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1, $2, $3, $4, $5::affix_location, $6) \
                 ON CONFLICT (id) DO UPDATE SET \
                 blueprint_id = EXCLUDED.blueprint_id, affix_id = EXCLUDED.affix_id, \
                 weight = EXCLUDED.weight, location = EXCLUDED.location, \
                 sort_order = EXCLUDED.sort_order",
            )
            .bind(ba.id)
            .bind(ba.blueprint_id)
            .bind(ba.affix_id)
            .bind(ba.weight)
            .bind(loc)
            .bind(ba.sort_order)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    {
        let mut cache = state.cache.write().await;
        for client_id in &affected_client_ids {
            let data = Cache::fetch_client_data(pool, *client_id).await?;
            cache.apply_client_data(*client_id, data);
        }
    }

    if let Some(ref redis_handle) = state.redis {
        for client_id in &affected_client_ids {
            redis_handle.invalidate(*client_id).await;
        }
    }

    Ok(clients_created)
}

fn count_resources(clients: &[ClientImport]) -> i64 {
    let mut count: i64 = 0;
    for c in clients {
        count += c.global_meta_attributes.len() as i64;
        count += c.blueprints.len() as i64;
        count += c.affixes.len() as i64;
        count += c.blueprint_affixes.len() as i64;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ts() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    fn make_blueprint(id: Uuid, client_id: Uuid, name: &str, weight: f64) -> Blueprint {
        Blueprint {
            id,
            client_id,
            name: name.into(),
            archetype: "sword".into(),
            weight,
            description: None,
            attributes: serde_json::json!({"damage": {"value_type": "range", "min": 10.0, "max": 20.0}}),
            attribute_order: vec!["damage".into()],
            min_prefixes: 0,
            max_prefixes: 1,
            min_suffixes: 0,
            max_suffixes: 1,
            created_at: ts(),
            updated_at: ts(),
        }
    }

    fn make_affix(id: Uuid, client_id: Uuid, name: &str, loc: AffixLocation) -> Affix {
        Affix {
            id,
            client_id,
            name: name.into(),
            location: loc,
            description: None,
            attribute: serde_json::json!({"fire_damage": {"value_type": "range", "min": 5.0, "max": 15.0}}),
            created_at: ts(),
            updated_at: ts(),
        }
    }

    fn make_gma(id: Uuid, client_id: Uuid, name: &str) -> GlobalMetaAttribute {
        GlobalMetaAttribute {
            id,
            client_id,
            name: name.into(),
            description: None,
            value_type: ValueType::Range,
            payload: serde_json::json!({"min": 1.0, "max": 10.0}),
            created_at: ts(),
            updated_at: ts(),
        }
    }

    fn make_client_import(
        client_id: Uuid,
        blueprints: Vec<Blueprint>,
        affixes: Vec<Affix>,
    ) -> ClientImport {
        ClientImport {
            client_name: "Test Client".into(),
            client_id: Some(client_id),
            blueprints,
            affixes,
            global_meta_attributes: vec![],
            blueprint_affixes: vec![],
        }
    }

    #[test]
    fn test_keep_old_removes_resource_from_import() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();

        let mut clients = vec![make_client_import(
            client_id,
            vec![make_blueprint(bp_id, client_id, "Longsword", 1.0)],
            vec![make_affix(affix_id, client_id, "Fire", AffixLocation::Prefix)],
        )];

        let conflicts = vec![
            ConflictDetail {
                resource_type: "blueprint".into(),
                resource_id: bp_id,
                resource_name: "Longsword".into(),
                attributes: vec![ConflictAttribute {
                    key: "weight".into(),
                    old_value: serde_json::json!(1.0),
                    new_value: serde_json::json!(2.0),
                    value_type: ValueType::Range,
                }],
            },
            ConflictDetail {
                resource_type: "affix".into(),
                resource_id: affix_id,
                resource_name: "Fire".into(),
                attributes: vec![ConflictAttribute {
                    key: "name".into(),
                    old_value: serde_json::json!("Fire"),
                    new_value: serde_json::json!("Blaze"),
                    value_type: ValueType::String,
                }],
            },
        ];

        let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        resolutions.insert(
            bp_id,
            ResourceResolution {
                strategy: ResolutionStrategy::KeepOld,
                attributes: None,
            },
        );
        resolutions.insert(
            affix_id,
            ResourceResolution {
                strategy: ResolutionStrategy::KeepOld,
                attributes: None,
            },
        );

        apply_resolutions(&mut clients, &conflicts, &resolutions).unwrap();

        assert!(clients[0].blueprints.is_empty());
        assert!(clients[0].affixes.is_empty());
    }

    #[test]
    fn test_keep_new_preserves_imported_value() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let mut clients = vec![make_client_import(
            client_id,
            vec![make_blueprint(bp_id, client_id, "Longsword", 2.0)],
            vec![],
        )];

        let conflicts = vec![ConflictDetail {
            resource_type: "blueprint".into(),
            resource_id: bp_id,
            resource_name: "Longsword".into(),
            attributes: vec![ConflictAttribute {
                key: "weight".into(),
                old_value: serde_json::json!(1.0),
                new_value: serde_json::json!(2.0),
                value_type: ValueType::Range,
            }],
        }];

        let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        resolutions.insert(
            bp_id,
            ResourceResolution {
                strategy: ResolutionStrategy::KeepNew,
                attributes: None,
            },
        );

        apply_resolutions(&mut clients, &conflicts, &resolutions).unwrap();

        assert_eq!(clients[0].blueprints.len(), 1);
        assert!(
            (clients[0].blueprints[0].weight - 2.0).abs() < f64::EPSILON,
            "keep_new should preserve imported weight"
        );
    }

    #[test]
    fn test_per_attribute_keep_new() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let mut clients = vec![make_client_import(
            client_id,
            vec![make_blueprint(bp_id, client_id, "Longsword", 2.0)],
            vec![],
        )];

        let conflicts = vec![ConflictDetail {
            resource_type: "blueprint".into(),
            resource_id: bp_id,
            resource_name: "Longsword".into(),
            attributes: vec![ConflictAttribute {
                key: "weight".into(),
                old_value: serde_json::json!(1.0),
                new_value: serde_json::json!(2.0),
                value_type: ValueType::Range,
            }],
        }];

        let mut attrs = HashMap::new();
        attrs.insert("weight".into(), ResolutionStrategy::KeepNew);

        let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        resolutions.insert(
            bp_id,
            ResourceResolution {
                strategy: ResolutionStrategy::PerAttribute,
                attributes: Some(attrs),
            },
        );

        apply_resolutions(&mut clients, &conflicts, &resolutions).unwrap();

        assert!(
            (clients[0].blueprints[0].weight - 2.0).abs() < f64::EPSILON,
            "per_attribute with keepNew should preserve imported value"
        );
    }

    #[test]
    fn test_per_attribute_keep_old() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let mut clients = vec![make_client_import(
            client_id,
            vec![make_blueprint(bp_id, client_id, "Longsword", 2.0)],
            vec![],
        )];

        let conflicts = vec![ConflictDetail {
            resource_type: "blueprint".into(),
            resource_id: bp_id,
            resource_name: "Longsword".into(),
            attributes: vec![ConflictAttribute {
                key: "weight".into(),
                old_value: serde_json::json!(1.0),
                new_value: serde_json::json!(2.0),
                value_type: ValueType::Range,
            }],
        }];

        let mut attrs = HashMap::new();
        attrs.insert("weight".into(), ResolutionStrategy::KeepOld);

        let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        resolutions.insert(
            bp_id,
            ResourceResolution {
                strategy: ResolutionStrategy::PerAttribute,
                attributes: Some(attrs),
            },
        );

        apply_resolutions(&mut clients, &conflicts, &resolutions).unwrap();

        assert!(
            (clients[0].blueprints[0].weight - 1.0).abs() < f64::EPSILON,
            "per_attribute with keepOld should revert to old value"
        );
    }

    #[test]
    fn test_per_attribute_multiple_keys() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let mut clients = vec![make_client_import(
            client_id,
            vec![make_blueprint(bp_id, client_id, "ImportedName", 2.5)],
            vec![],
        )];

        let conflicts = vec![ConflictDetail {
            resource_type: "blueprint".into(),
            resource_id: bp_id,
            resource_name: "ImportedName".into(),
            attributes: vec![
                ConflictAttribute {
                    key: "weight".into(),
                    old_value: serde_json::json!(1.0),
                    new_value: serde_json::json!(2.5),
                    value_type: ValueType::Range,
                },
                ConflictAttribute {
                    key: "name".into(),
                    old_value: serde_json::Value::String("OldName".into()),
                    new_value: serde_json::Value::String("ImportedName".into()),
                    value_type: ValueType::String,
                },
            ],
        }];

        let mut attrs = HashMap::new();
        attrs.insert("weight".into(), ResolutionStrategy::KeepNew);
        attrs.insert("name".into(), ResolutionStrategy::KeepOld);

        let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        resolutions.insert(
            bp_id,
            ResourceResolution {
                strategy: ResolutionStrategy::PerAttribute,
                attributes: Some(attrs),
            },
        );

        apply_resolutions(&mut clients, &conflicts, &resolutions).unwrap();

        assert!((clients[0].blueprints[0].weight - 2.5).abs() < f64::EPSILON);
        assert_eq!(clients[0].blueprints[0].name, "OldName");
    }

    #[test]
    fn test_invalid_import_token() {
        let staging = ImportStaging::new();
        let result = staging.take("nonexistent-token");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 404);
    }

    #[test]
    fn test_expired_import_token() {
        let staging = ImportStaging::new();
        let token = "expired-token".to_string();

        let bp_id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let import = StagedImport {
            clients: vec![make_client_import(
                client_id,
                vec![make_blueprint(bp_id, client_id, "Test", 1.0)],
                vec![],
            )],
            conflicts: vec![],
        };

        let old = Instant::now()
            .checked_sub(STAGING_TTL + Duration::from_secs(1))
            .unwrap();
        staging.data.insert(token.clone(), (old, import));

        let result = staging.take(&token);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 404);
    }

    #[test]
    fn test_missing_resolution_returns_error() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let mut clients = vec![make_client_import(
            client_id,
            vec![make_blueprint(bp_id, client_id, "Test", 1.0)],
            vec![],
        )];

        let conflicts = vec![ConflictDetail {
            resource_type: "blueprint".into(),
            resource_id: bp_id,
            resource_name: "Test".into(),
            attributes: vec![ConflictAttribute {
                key: "weight".into(),
                old_value: serde_json::json!(1.0),
                new_value: serde_json::json!(2.0),
                value_type: ValueType::Range,
            }],
        }];

        let resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        let result = apply_resolutions(&mut clients, &conflicts, &resolutions);
        assert!(result.is_err());
    }

    #[test]
    fn test_staging_cleanup_removes_expired() {
        let staging = ImportStaging::new();
        let token = "will-expire".to_string();

        let old = Instant::now()
            .checked_sub(STAGING_TTL + Duration::from_secs(60))
            .unwrap();
        staging.data.insert(
            token,
            (
                old,
                StagedImport {
                    clients: vec![],
                    conflicts: vec![],
                },
            ),
        );

        staging.cleanup();
        assert!(staging.data.is_empty());
    }

    #[test]
    fn test_compare_blueprints_detects_weight_change() {
        let id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let existing = make_blueprint(id, client_id, "Test", 1.0);
        let imported = make_blueprint(id, client_id, "Test", 2.0);

        let diffs = compare_blueprints(&existing, &imported);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].key, "weight");
        assert_eq!(diffs[0].old_value, serde_json::json!(1.0));
        assert_eq!(diffs[0].new_value, serde_json::json!(2.0));
    }

    #[test]
    fn test_compare_blueprints_detects_name_change() {
        let id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let existing = make_blueprint(id, client_id, "Old", 1.0);
        let imported = make_blueprint(id, client_id, "New", 1.0);

        let diffs = compare_blueprints(&existing, &imported);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].key, "name");
    }

    #[test]
    fn test_no_conflicts_when_identical() {
        let id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let bp = make_blueprint(id, client_id, "Test", 1.0);
        let diffs = compare_blueprints(&bp, &bp);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_compare_affixes_detects_name_change() {
        let id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let existing = make_affix(id, client_id, "Fire", AffixLocation::Prefix);
        let imported = make_affix(id, client_id, "Blaze", AffixLocation::Prefix);

        let diffs = compare_affixes(&existing, &imported);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].key, "name");
    }

    #[test]
    fn test_compare_gmas_detects_value_type_change() {
        let id = Uuid::new_v4();
        let client_id = Uuid::new_v4();
        let existing = make_gma(id, client_id, "damage");
        let mut imported = make_gma(id, client_id, "damage");
        imported.value_type = ValueType::Single;

        let diffs = compare_gmas(&existing, &imported);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].key, "value_type");
    }

    #[test]
    fn test_apply_resolutions_empty_conflicts() {
        let mut clients = vec![make_client_import(Uuid::new_v4(), vec![], vec![])];
        let resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        let result = apply_resolutions(&mut clients, &[], &resolutions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_keep_old_removes_blueprint_affixes() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let ba_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();

        let mut clients = vec![ClientImport {
            client_name: "Test".into(),
            client_id: Some(client_id),
            blueprints: vec![make_blueprint(bp_id, client_id, "Sword", 1.0)],
            affixes: vec![],
            global_meta_attributes: vec![],
            blueprint_affixes: vec![BlueprintAffix {
                id: ba_id,
                blueprint_id: bp_id,
                affix_id,
                weight: 1.0,
                location: AffixLocation::Prefix,
                sort_order: 0,
            }],
        }];

        let conflicts = vec![ConflictDetail {
            resource_type: "blueprint".into(),
            resource_id: bp_id,
            resource_name: "Sword".into(),
            attributes: vec![ConflictAttribute {
                key: "weight".into(),
                old_value: serde_json::json!(1.0),
                new_value: serde_json::json!(2.0),
                value_type: ValueType::Range,
            }],
        }];

        let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        resolutions.insert(
            bp_id,
            ResourceResolution {
                strategy: ResolutionStrategy::KeepOld,
                attributes: None,
            },
        );

        apply_resolutions(&mut clients, &conflicts, &resolutions).unwrap();

        assert!(clients[0].blueprints.is_empty());
        assert!(clients[0].blueprint_affixes.is_empty());
    }

    #[test]
    fn test_count_resources() {
        let client_id = Uuid::new_v4();
        let clients = vec![ClientImport {
            client_name: "Test".into(),
            client_id: Some(client_id),
            blueprints: vec![
                make_blueprint(Uuid::new_v4(), client_id, "a", 1.0),
                make_blueprint(Uuid::new_v4(), client_id, "b", 1.0),
            ],
            affixes: vec![make_affix(
                Uuid::new_v4(),
                client_id,
                "x",
                AffixLocation::Prefix,
            )],
            global_meta_attributes: vec![
                make_gma(Uuid::new_v4(), client_id, "g1"),
                make_gma(Uuid::new_v4(), client_id, "g2"),
                make_gma(Uuid::new_v4(), client_id, "g3"),
            ],
            blueprint_affixes: vec![BlueprintAffix {
                id: Uuid::new_v4(),
                blueprint_id: Uuid::new_v4(),
                affix_id: Uuid::new_v4(),
                weight: 1.0,
                location: AffixLocation::Prefix,
                sort_order: 0,
            }],
        }];
        assert_eq!(count_resources(&clients), 7);
    }

    #[test]
    fn test_staging_valid_token() {
        let staging = ImportStaging::new();
        let token = "valid-token".to_string();

        let client_id = Uuid::new_v4();
        let import = StagedImport {
            clients: vec![make_client_import(
                client_id,
                vec![make_blueprint(Uuid::new_v4(), client_id, "Test", 1.0)],
                vec![],
            )],
            conflicts: vec![],
        };

        staging.store(token.clone(), import);
        let result = staging.take(&token);
        assert!(result.is_ok());

        let result2 = staging.take(&token);
        assert!(result2.is_err());
    }

    #[test]
    fn test_apply_blueprint_field_description_null() {
        let client_id = Uuid::new_v4();
        let mut bp = make_blueprint(Uuid::new_v4(), client_id, "Test", 1.0);
        bp.description = Some("A test".into());
        apply_blueprint_field(&mut bp, "description", &serde_json::Value::Null);
        assert!(bp.description.is_none());
    }

    #[test]
    fn test_apply_affix_field_type() {
        let client_id = Uuid::new_v4();
        let mut a = make_affix(Uuid::new_v4(), client_id, "Test", AffixLocation::Prefix);
        apply_affix_field(
            &mut a,
            "type",
            &serde_json::Value::String("suffix".into()),
        );
        assert_eq!(a.location, AffixLocation::Suffix);
    }

    #[test]
    fn test_apply_gma_field_value_type() {
        let client_id = Uuid::new_v4();
        let mut gma = make_gma(Uuid::new_v4(), client_id, "test");
        apply_gma_field(
            &mut gma,
            "value_type",
            &serde_json::Value::String("single".into()),
        );
        assert_eq!(gma.value_type, ValueType::Single);
    }

    #[test]
    fn test_per_attribute_missing_attributes_map() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let mut clients = vec![make_client_import(
            client_id,
            vec![make_blueprint(bp_id, client_id, "Test", 1.0)],
            vec![],
        )];

        let conflicts = vec![ConflictDetail {
            resource_type: "blueprint".into(),
            resource_id: bp_id,
            resource_name: "Test".into(),
            attributes: vec![ConflictAttribute {
                key: "weight".into(),
                old_value: serde_json::json!(1.0),
                new_value: serde_json::json!(2.0),
                value_type: ValueType::Range,
            }],
        }];

        let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
        resolutions.insert(
            bp_id,
            ResourceResolution {
                strategy: ResolutionStrategy::PerAttribute,
                attributes: None,
            },
        );

        let result = apply_resolutions(&mut clients, &conflicts, &resolutions);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_staging_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ImportStaging>();
    }

    #[test]
    fn test_parse_zip_empty_body() {
        let result = parse_zip(Bytes::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_build_client_import_minimal() {
        let client_id = Uuid::new_v4();
        let files = vec![(
            "client.json".into(),
            serde_json::to_vec(&serde_json::json!({
                "id": client_id,
                "name": "Test Client"
            }))
            .unwrap(),
        )];

        let result = build_client_import(&files);
        assert!(result.is_ok());
        let import = result.unwrap();
        assert_eq!(import.client_id, Some(client_id));
        assert_eq!(import.client_name, "Test Client");
        assert!(import.blueprints.is_empty());
    }

    #[test]
    fn test_build_client_import_missing_client_json() {
        let files: Vec<(String, Vec<u8>)> = vec![];
        let result = build_client_import(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_client_import_with_resources() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();

        let files = vec![
            (
                "client.json".into(),
                serde_json::to_vec(&serde_json::json!({
                    "id": client_id,
                    "name": "Test Client"
                }))
                .unwrap(),
            ),
            (
                "blueprints.json".into(),
                serde_json::to_vec(&vec![serde_json::json!({
                    "id": bp_id,
                    "name": "Sword",
                    "archetype": "sword",
                    "weight": 1.0,
                    "attributes": {},
                    "attributeOrder": [],
                    "minPrefixes": 0,
                    "maxPrefixes": 0,
                    "minSuffixes": 0,
                    "maxSuffixes": 0
                })])
                .unwrap(),
            ),
            (
                "affixes.json".into(),
                serde_json::to_vec(&vec![serde_json::json!({
                    "id": affix_id,
                    "name": "Fire",
                    "type": "prefix",
                    "attribute": {}
                })])
                .unwrap(),
            ),
        ];

        let result = build_client_import(&files);
        assert!(result.is_ok());
        let import = result.unwrap();
        assert_eq!(import.blueprints.len(), 1);
        assert_eq!(import.affixes.len(), 1);
    }
}

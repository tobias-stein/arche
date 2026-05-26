use std::collections::HashMap;
use std::io::Cursor;

use axum::extract::State;
use axum::http::header;
use axum::response::Response;
use axum::Json;
use serde::Serialize;
use sqlx::Row;
use uuid::Uuid;
use zip::ZipWriter;
use zip::write::FileOptions;

use arche_types::export_import::ExportRequest;

use crate::auth::permission::CurrentUser;
use crate::error::ProblemResponse;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ClientJson {
    id: Uuid,
    name: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct BlueprintExport {
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

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AffixExport {
    id: Uuid,
    name: String,
    #[serde(rename = "type")]
    affix_type: String,
    description: Option<String>,
    attribute: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GmaExport {
    id: Uuid,
    name: String,
    description: Option<String>,
    value_type: String,
    payload: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct BaExport {
    id: Uuid,
    blueprint_id: Uuid,
    affix_id: Uuid,
    weight: f64,
    location: String,
    sort_order: i32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiKeyExport {
    id: Uuid,
    name: String,
    permissions: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AuditLogExport {
    id: Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
    actor_key_id: Uuid,
    actor_key_name: String,
    client_id: Option<Uuid>,
    resource_type: String,
    resource_id: Uuid,
    action: String,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
}

pub async fn export_handler(
    State(state): State<crate::AppState>,
    _user: CurrentUser,
    Json(req): Json<ExportRequest>,
) -> Result<Response, ProblemResponse> {
    let client_ids = if req.client_ids.is_empty() {
        sqlx::query("SELECT id FROM clients ORDER BY id ASC")
            .fetch_all(&*state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "export: failed to list all clients");
                ProblemResponse::unprocessable_entity("Failed to list clients for export")
            })?
            .iter()
            .map(|r| r.get::<Uuid, _>("id"))
            .collect::<Vec<_>>()
    } else {
        req.client_ids
    };

    let mut buf = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(&mut buf));
    let options = FileOptions::<()>::default();

    for client_id in client_ids {
        let client_row = sqlx::query("SELECT id, name FROM clients WHERE id = $1")
            .bind(client_id)
            .fetch_optional(&*state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "export: client lookup failed");
                ProblemResponse::unprocessable_entity("Failed to query client for export")
            })?;

        let client = match client_row {
            Some(r) => ClientJson {
                id: r.get("id"),
                name: r.get("name"),
            },
            None => continue,
        };

        let folder = format!("client-{}", client.id);

        let gmas = fetch_gmas(&state.pool, client_id).await?;
        let gma_map: HashMap<Uuid, &GmaExport> =
            gmas.iter().map(|g| (g.id, g)).collect();

        let mut blueprints = fetch_blueprints(&state.pool, client_id).await?;
        let mut affixes = fetch_affixes(&state.pool, client_id).await?;
        let blueprint_affixes = fetch_blueprint_affixes(&state.pool, client_id).await?;

        if req.inline_global_refs {
            inline_blueprint_refs(&mut blueprints, &gma_map);
            inline_affix_refs(&mut affixes, &gma_map);
        }

        write_entry(&mut zip, options, &format!("{}/client.json", folder), &client)?;
        write_entry(&mut zip, options, &format!("{}/global_meta_attributes.json", folder), &gmas)?;
        write_entry(&mut zip, options, &format!("{}/blueprints.json", folder), &blueprints)?;
        write_entry(&mut zip, options, &format!("{}/affixes.json", folder), &affixes)?;
        write_entry(&mut zip, options, &format!("{}/blueprint_affixes.json", folder), &blueprint_affixes)?;

        if req.include_api_keys {
            let api_keys = fetch_api_keys(&state.pool, client_id).await?;
            write_entry(&mut zip, options, &format!("{}/api-keys.json", folder), &api_keys)?;
        }

        if req.include_audit_log {
            let audit_entries = fetch_audit_log(&state.pool, client_id).await?;
            write_entry(&mut zip, options, &format!("{}/audit-log.json", folder), &audit_entries)?;
        }
    }

    let buf = zip
        .finish()
        .map_err(|e| {
            tracing::error!(error = %e, "export: ZIP finalization failed");
            ProblemResponse::unprocessable_entity("Failed to finalize ZIP")
        })?
        .into_inner();

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/zip")
        .body(axum::body::Body::from(buf.to_owned()))
        .unwrap();

    Ok(response)
}

fn write_json_entry<W: std::io::Write + std::io::Seek, T: Serialize>(
    zip: &mut ZipWriter<W>,
    options: FileOptions<()>,
    name: &str,
    data: &T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    zip.start_file(name, options)?;
    let json_bytes = serde_json::to_vec_pretty(data)?;
    std::io::Write::write_all(zip, &json_bytes)?;
    Ok(())
}

fn write_entry<W: std::io::Write + std::io::Seek, T: Serialize>(
    zip: &mut ZipWriter<W>,
    options: FileOptions<()>,
    name: &str,
    data: &T,
) -> Result<(), ProblemResponse> {
    write_json_entry(zip, options, name, data).map_err(|e| {
        tracing::error!(error = %e, "export: write {name} failed");
        ProblemResponse::unprocessable_entity("Failed to write ZIP entry")
    })
}

async fn fetch_gmas(
    pool: &sqlx::PgPool,
    client_id: Uuid,
) -> Result<Vec<GmaExport>, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT id, name, description, value_type::text AS value_type, payload \
         FROM global_meta_attributes WHERE client_id = $1 ORDER BY id ASC",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "export: gma query failed");
        ProblemResponse::unprocessable_entity("Failed to query global meta attributes")
    })?;

    Ok(rows
        .iter()
        .map(|r| {
            let vt_str: String = r.get("value_type");
            GmaExport {
                id: r.get("id"),
                name: r.get("name"),
                description: r.get("description"),
                value_type: vt_str,
                payload: r.get("payload"),
            }
        })
        .collect())
}

async fn fetch_blueprints(
    pool: &sqlx::PgPool,
    client_id: Uuid,
) -> Result<Vec<BlueprintExport>, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT id, name, archetype, weight, description, attributes, attribute_order, \
         min_prefixes, max_prefixes, min_suffixes, max_suffixes \
         FROM blueprints WHERE client_id = $1 ORDER BY id ASC",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "export: blueprints query failed");
        ProblemResponse::unprocessable_entity("Failed to query blueprints")
    })?;

    Ok(rows
        .iter()
        .map(|r| BlueprintExport {
            id: r.get("id"),
            name: r.get("name"),
            archetype: r.get("archetype"),
            weight: r.get("weight"),
            description: r.get("description"),
            attributes: r.get("attributes"),
            attribute_order: r.get("attribute_order"),
            min_prefixes: r.get("min_prefixes"),
            max_prefixes: r.get("max_prefixes"),
            min_suffixes: r.get("min_suffixes"),
            max_suffixes: r.get("max_suffixes"),
        })
        .collect())
}

async fn fetch_affixes(
    pool: &sqlx::PgPool,
    client_id: Uuid,
) -> Result<Vec<AffixExport>, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT id, name, type::text, description, attribute \
         FROM affixes WHERE client_id = $1 ORDER BY id ASC",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "export: affixes query failed");
        ProblemResponse::unprocessable_entity("Failed to query affixes")
    })?;

    Ok(rows
        .iter()
        .map(|r| {
            let type_str: String = r.get("type");
            AffixExport {
                id: r.get("id"),
                name: r.get("name"),
                affix_type: type_str,
                description: r.get("description"),
                attribute: r.get("attribute"),
            }
        })
        .collect())
}

async fn fetch_blueprint_affixes(
    pool: &sqlx::PgPool,
    client_id: Uuid,
) -> Result<Vec<BaExport>, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT ba.id, ba.blueprint_id, ba.affix_id, ba.weight, ba.location, ba.sort_order \
         FROM blueprint_affixes ba \
         JOIN blueprints b ON ba.blueprint_id = b.id \
         WHERE b.client_id = $1 ORDER BY ba.id ASC",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "export: blueprint_affixes query failed");
        ProblemResponse::unprocessable_entity("Failed to query blueprint affixes")
    })?;

    Ok(rows
        .iter()
        .map(|r| {
            let loc_str: String = r.get("location");
            BaExport {
                id: r.get("id"),
                blueprint_id: r.get("blueprint_id"),
                affix_id: r.get("affix_id"),
                weight: r.get("weight"),
                location: loc_str,
                sort_order: r.get("sort_order"),
            }
        })
        .collect())
}

async fn fetch_api_keys(
    pool: &sqlx::PgPool,
    client_id: Uuid,
) -> Result<Vec<ApiKeyExport>, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT id, name, permissions, created_at FROM api_keys \
         WHERE client_id = $1 AND is_super = false ORDER BY created_at DESC",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "export: api keys query failed");
        ProblemResponse::unprocessable_entity("Failed to query API keys")
    })?;

    Ok(rows
        .iter()
        .map(|r| {
            let perms: Vec<String> = r.get("permissions");
            ApiKeyExport {
                id: r.get("id"),
                name: r.get("name"),
                permissions: perms,
                created_at: r.get("created_at"),
            }
        })
        .collect())
}

async fn fetch_audit_log(
    pool: &sqlx::PgPool,
    client_id: Uuid,
) -> Result<Vec<AuditLogExport>, ProblemResponse> {
    let rows = sqlx::query(
        "SELECT id, timestamp, actor_key_id, actor_key_name, client_id, \
         resource_type, resource_id, action, before, after \
         FROM audit_log WHERE client_id = $1 ORDER BY timestamp DESC, id DESC",
    )
    .bind(client_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "export: audit log query failed");
        ProblemResponse::unprocessable_entity("Failed to query audit log")
    })?;

    Ok(rows
        .iter()
        .map(|r| {
            let action_str: String = r.get("action");
            AuditLogExport {
                id: r.get("id"),
                timestamp: r.get("timestamp"),
                actor_key_id: r.get("actor_key_id"),
                actor_key_name: r.get("actor_key_name"),
                client_id: r.get("client_id"),
                resource_type: r.get("resource_type"),
                resource_id: r.get("resource_id"),
                action: action_str,
                before: r.get("before"),
                after: r.get("after"),
            }
        })
        .collect())
}

fn inline_blueprint_refs(
    blueprints: &mut [BlueprintExport],
    gma_map: &HashMap<Uuid, &GmaExport>,
) {
    for bp in blueprints.iter_mut() {
        let mut attrs = match bp.attributes.clone() {
            serde_json::Value::Object(obj) => obj,
            _ => continue,
        };

        let mut keys_to_inline: Vec<(String, Uuid)> = Vec::new();
        for (key, value) in &attrs {
            if let Some(ref_id) = extract_ref_id(value) {
                keys_to_inline.push((key.clone(), ref_id));
            }
        }

        for (key, ref_id) in &keys_to_inline {
            if let Some(gma) = gma_map.get(ref_id) {
                let inline = build_inline_from_gma(gma);
                attrs.insert(key.clone(), inline);
            }
        }

        bp.attributes = serde_json::Value::Object(attrs);
    }
}

fn inline_affix_refs(
    affixes: &mut [AffixExport],
    gma_map: &HashMap<Uuid, &GmaExport>,
) {
    for affix in affixes.iter_mut() {
        if let Some(ref_id) = extract_ref_id(&affix.attribute) {
            if let Some(gma) = gma_map.get(&ref_id) {
                affix.attribute = build_affix_inline_from_gma(gma);
            }
        }
    }
}

fn extract_ref_id(value: &serde_json::Value) -> Option<Uuid> {
    let obj = value.as_object()?;
    if obj.len() != 1 {
        return None;
    }
    let ref_id_str = obj.get("$ref_id")?.as_str()?;
    Uuid::parse_str(ref_id_str).ok()
}

fn insert_payload_keys(map: &mut serde_json::Map<String, serde_json::Value>, payload: &serde_json::Value) {
    if let serde_json::Value::Object(obj) = payload {
        for (k, v) in obj {
            if k == "distribution" {
                continue;
            }
            map.insert(k.clone(), v.clone());
        }
        if let Some(dist) = obj.get("distribution") {
            map.insert("distribution".into(), dist.clone());
        }
    }
}

fn build_inline_from_gma(gma: &GmaExport) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(ref desc) = gma.description {
        map.insert("description".into(), serde_json::Value::String(desc.clone()));
    }
    map.insert("value_type".into(), serde_json::Value::String(gma.value_type.clone()));
    insert_payload_keys(&mut map, &gma.payload);
    serde_json::Value::Object(map)
}

fn build_affix_inline_from_gma(gma: &GmaExport) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("name".into(), serde_json::Value::String(gma.name.clone()));
    if let Some(ref desc) = gma.description {
        map.insert("description".into(), serde_json::Value::String(desc.clone()));
    }
    map.insert("value_type".into(), serde_json::Value::String(gma.value_type.clone()));
    insert_payload_keys(&mut map, &gma.payload);
    serde_json::Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_ref_id_finds_ref() {
        let val = json!({"$ref_id": "550e8400-e29b-41d4-a716-446655440000"});
        let result = extract_ref_id(&val);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_extract_ref_id_returns_none_for_inline() {
        let val = json!({"value_type": "range", "min": 10.0, "max": 20.0});
        let result = extract_ref_id(&val);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_ref_id_rejects_extra_keys() {
        let val = json!({"$ref_id": "550e8400-e29b-41d4-a716-446655440000", "extra": true});
        let result = extract_ref_id(&val);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_inline_from_gma_range() {
        let gma = GmaExport {
            id: Uuid::new_v4(),
            name: "damage".into(),
            description: Some("Physical damage".into()),
            value_type: "range".into(),
            payload: json!({"min": 10.0, "max": 23.0, "distribution": {"type": "uniform"}}),
        };
        let inline = build_inline_from_gma(&gma);
        let obj = inline.as_object().unwrap();
        assert_eq!(obj.get("description").unwrap(), "Physical damage");
        assert_eq!(obj.get("value_type").unwrap(), "range");
        assert_eq!(obj.get("min").unwrap(), &json!(10.0));
        assert_eq!(obj.get("max").unwrap(), &json!(23.0));
        assert_eq!(obj.get("distribution").unwrap(), &json!({"type": "uniform"}));
        assert!(obj.get("$ref_id").is_none());
    }

    #[test]
    fn test_build_inline_from_gma_string() {
        let gma = GmaExport {
            id: Uuid::new_v4(),
            name: "label".into(),
            description: None,
            value_type: "string".into(),
            payload: json!({"min_length": 1, "max_length": 100}),
        };
        let inline = build_inline_from_gma(&gma);
        let obj = inline.as_object().unwrap();
        assert_eq!(obj.get("value_type").unwrap(), "string");
        assert_eq!(obj.get("min_length").unwrap(), &json!(1));
        assert_eq!(obj.get("max_length").unwrap(), &json!(100));
        assert!(obj.get("description").is_none());
    }

    #[test]
    fn test_build_affix_inline_from_gma() {
        let gma = GmaExport {
            id: Uuid::new_v4(),
            name: "fire_damage".into(),
            description: Some("Fire damage affix".into()),
            value_type: "range".into(),
            payload: json!({"min": 5.0, "max": 15.0}),
        };
        let inline = build_affix_inline_from_gma(&gma);
        let obj = inline.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap(), "fire_damage");
        assert_eq!(obj.get("description").unwrap(), "Fire damage affix");
        assert_eq!(obj.get("value_type").unwrap(), "range");
        assert_eq!(obj.get("min").unwrap(), &json!(5.0));
        assert_eq!(obj.get("max").unwrap(), &json!(15.0));
    }

    #[test]
    fn test_inline_blueprint_refs_resolves_ref() {
        let gma_id = Uuid::new_v4();
        let gma = GmaExport {
            id: gma_id,
            name: "rarity".into(),
            description: None,
            value_type: "enum".into(),
            payload: json!({"values": ["common", "rare"]}),
        };
        let gma_map: HashMap<Uuid, &GmaExport> =
            [(gma_id, &gma)].into();

        let bp = BlueprintExport {
            id: Uuid::new_v4(),
            name: "Sword".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: None,
            attributes: json!({
                "damage": {"value_type": "range", "min": 10.0, "max": 20.0},
                "rarity": {"$ref_id": gma_id.to_string()}
            }),
            attribute_order: vec!["damage".into(), "rarity".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
        };

        let mut slice = [bp];
        inline_blueprint_refs(&mut slice, &gma_map);

        let attrs = slice[0].attributes.as_object().unwrap();
        let rarity = attrs.get("rarity").unwrap();
        assert!(rarity.get("$ref_id").is_none());
        assert_eq!(rarity.get("value_type").unwrap(), "enum");
        assert_eq!(rarity.get("values").unwrap(), &json!(["common", "rare"]));
        let damage = attrs.get("damage").unwrap();
        assert_eq!(damage.get("value_type").unwrap(), "range");
    }

    #[test]
    fn test_inline_blueprint_refs_preserves_when_not_in_map() {
        let gma_id = Uuid::new_v4();
        let ref_id = gma_id.to_string();
        let bp = BlueprintExport {
            id: Uuid::new_v4(),
            name: "Sword".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: None,
            attributes: json!({
                "rarity": {"$ref_id": ref_id.clone()}
            }),
            attribute_order: vec!["rarity".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
        };

        let empty_map: HashMap<Uuid, &GmaExport> = HashMap::new();
        inline_blueprint_refs(&mut [bp.clone()], &empty_map);

        let attrs = bp.attributes.as_object().unwrap();
        let rarity = attrs.get("rarity").unwrap();
        assert_eq!(rarity.get("$ref_id").unwrap(), &json!(ref_id));
    }

    #[test]
    fn test_inline_affix_refs_resolves_ref() {
        let gma_id = Uuid::new_v4();
        let gma = GmaExport {
            id: gma_id,
            name: "fire_damage".into(),
            description: Some("Fire".into()),
            value_type: "range".into(),
            payload: json!({"min": 5.0, "max": 15.0}),
        };
        let gma_map: HashMap<Uuid, &GmaExport> =
            [(gma_id, &gma)].into();

        let affix = AffixExport {
            id: Uuid::new_v4(),
            name: "Fire".into(),
            affix_type: "prefix".into(),
            description: None,
            attribute: json!({"$ref_id": gma_id.to_string()}),
        };

        let mut slice = [affix];
        inline_affix_refs(&mut slice, &gma_map);

        let attr = &slice[0].attribute;
        assert!(attr.get("$ref_id").is_none());
        assert_eq!(attr.get("name").unwrap(), "fire_damage");
        assert_eq!(attr.get("value_type").unwrap(), "range");
        assert_eq!(attr.get("min").unwrap(), &json!(5.0));
    }

    #[test]
    fn test_inline_affix_refs_preserves_when_missing() {
        let gma_id = Uuid::new_v4();
        let affix = AffixExport {
            id: Uuid::new_v4(),
            name: "Ice".into(),
            affix_type: "prefix".into(),
            description: None,
            attribute: json!({"$ref_id": gma_id.to_string()}),
        };

        let empty_map: HashMap<Uuid, &GmaExport> = HashMap::new();
        let mut slice = [affix];
        inline_affix_refs(&mut slice, &empty_map);

        let attr = &slice[0].attribute;
        assert_eq!(
            attr.get("$ref_id").unwrap(),
            &json!(gma_id.to_string())
        );
    }

    #[test]
    fn test_export_request_serialization_defaults() {
        let req = ExportRequest {
            client_ids: vec![],
            include_api_keys: false,
            include_audit_log: false,
            inline_global_refs: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ExportRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_client_json_serialization() {
        let c = ClientJson {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Test Game".into(),
        };
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json.get("id").unwrap(), "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(json.get("name").unwrap(), "Test Game");
    }

    mod db_tests {
        use super::*;
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

        async fn create_test_client(pool: &PgPool) -> Uuid {
            let row = sqlx::query(
                "INSERT INTO clients (name) VALUES ('export-test-client') RETURNING id",
            )
            .fetch_one(pool)
            .await
            .expect("Failed to create test client");
            row.get("id")
        }

        async fn cleanup_client_data(pool: &PgPool, client_id: Uuid) {
            let _ = sqlx::query("DELETE FROM blueprint_affixes WHERE blueprint_id IN (SELECT id FROM blueprints WHERE client_id = $1)")
                .bind(client_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM audit_log WHERE client_id = $1")
                .bind(client_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM api_keys WHERE client_id = $1 AND is_super = false")
                .bind(client_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM blueprints WHERE client_id = $1")
                .bind(client_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM affixes WHERE client_id = $1")
                .bind(client_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM global_meta_attributes WHERE client_id = $1")
                .bind(client_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM clients WHERE id = $1")
                .bind(client_id)
                .execute(pool)
                .await;
        }

        #[tokio::test]
        async fn test_export_single_client_empty_state() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;
            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![client_id],
                include_api_keys: false,
                include_audit_log: false,
                inline_global_refs: false,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export failed: {:?}", result.err());
            let response = result.unwrap();
            assert_eq!(
                response.headers().get("content-type").unwrap(),
                "application/zip"
            );

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            assert!(!body.is_empty(), "ZIP body should not be empty");

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();

            let folder = format!("client-{}", client_id);
            let expected_files = [
                format!("{}/client.json", folder),
                format!("{}/global_meta_attributes.json", folder),
                format!("{}/blueprints.json", folder),
                format!("{}/affixes.json", folder),
                format!("{}/blueprint_affixes.json", folder),
            ];

            let archive_entries: Vec<String> = (0..archive.len())
                .map(|i| archive.by_index(i).unwrap().name().to_string())
                .collect();

            for expected in &expected_files {
                let entry = archive.by_name(expected);
                assert!(
                    entry.is_ok(),
                    "missing ZIP entry: {}, entries: {:?}",
                    expected,
                    archive_entries
                );
            }

            assert!(
                archive.by_name(&format!("{}/api-keys.json", folder)).is_err(),
                "api-keys.json should not be present when include_api_keys=false"
            );
            assert!(
                archive.by_name(&format!("{}/audit-log.json", folder)).is_err(),
                "audit-log.json should not be present when include_audit_log=false"
            );

            cleanup_client_data(&pool, client_id).await;
        }

        #[tokio::test]
        async fn test_export_single_client_with_data() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1, $2, 'TestSword', 'sword', 1.0, $3, $4, 0, 0, 0, 0)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind(&json!({"damage": {"value_type": "range", "min": 10.0, "max": 20.0}}))
            .bind(&vec!["damage".to_string()])
            .execute(&pool)
            .await
            .unwrap();

            let affix_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO affixes (id, client_id, name, type, attribute) \
                 VALUES ($1, $2, 'Fire', 'prefix'::affix_location, $3)",
            )
            .bind(affix_id)
            .bind(client_id)
            .bind(&json!({"fire_damage": {"value_type": "range", "min": 5.0, "max": 15.0}}))
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO blueprint_affixes (id, blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1, $2, $3, 1.0, 'prefix'::affix_location, 0)",
            )
            .bind(Uuid::new_v4())
            .bind(bp_id)
            .bind(affix_id)
            .execute(&pool)
            .await
            .unwrap();

            let gma_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO global_meta_attributes (id, client_id, name, description, value_type, payload) \
                 VALUES ($1, $2, 'rarity', 'Quality tier', 'enum'::value_type, $3)",
            )
            .bind(gma_id)
            .bind(client_id)
            .bind(&json!({"values": ["common", "rare"]}))
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![client_id],
                include_api_keys: false,
                include_audit_log: false,
                inline_global_refs: false,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export failed: {:?}", result.err());
            let response = result.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();
            let folder = format!("client-{}", client_id);

            let blueprints: Vec<serde_json::Value> = {
                let mut file = archive.by_name(&format!("{}/blueprints.json", folder)).unwrap();
                let mut contents = Vec::new();
                std::io::copy(&mut file, &mut contents).unwrap();
                serde_json::from_slice(&contents).unwrap()
            };
            assert_eq!(blueprints.len(), 1);
            assert_eq!(blueprints[0].get("name").unwrap(), "TestSword");

            let affixes: Vec<serde_json::Value> = {
                let mut file = archive.by_name(&format!("{}/affixes.json", folder)).unwrap();
                let mut contents = Vec::new();
                std::io::copy(&mut file, &mut contents).unwrap();
                serde_json::from_slice(&contents).unwrap()
            };
            assert_eq!(affixes.len(), 1);
            assert_eq!(affixes[0].get("name").unwrap(), "Fire");

            let gmas: Vec<serde_json::Value> = {
                let mut file = archive.by_name(&format!("{}/global_meta_attributes.json", folder)).unwrap();
                let mut contents = Vec::new();
                std::io::copy(&mut file, &mut contents).unwrap();
                serde_json::from_slice(&contents).unwrap()
            };
            assert_eq!(gmas.len(), 1);
            assert_eq!(gmas[0].get("name").unwrap(), "rarity");

            cleanup_client_data(&pool, client_id).await;
        }

        #[tokio::test]
        async fn test_export_empty_client_ids_exports_all() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_a = create_test_client(&pool).await;
            let client_b = create_test_client(&pool).await;

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![],
                include_api_keys: false,
                include_audit_log: false,
                inline_global_refs: false,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export all failed: {:?}", result.err());
            let response = result.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();

            let folder_a = format!("client-{}", client_a);
            let folder_b = format!("client-{}", client_b);
            assert!(archive.by_name(&format!("{}/client.json", folder_a)).is_ok());
            assert!(archive.by_name(&format!("{}/client.json", folder_b)).is_ok());

            cleanup_client_data(&pool, client_a).await;
            cleanup_client_data(&pool, client_b).await;
        }

        #[tokio::test]
        async fn test_export_include_api_keys() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO api_keys (client_id, name, key_hash, permissions, is_super) \
                 VALUES ($1, 'my-key', 'test-hash', ARRAY['read']::text[], false)",
            )
            .bind(client_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![client_id],
                include_api_keys: true,
                include_audit_log: false,
                inline_global_refs: false,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export with api keys failed: {:?}", result.err());
            let response = result.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();
            let folder = format!("client-{}", client_id);

            let api_keys: Vec<serde_json::Value> = {
                let mut file = archive.by_name(&format!("{}/api-keys.json", folder)).unwrap();
                let mut contents = Vec::new();
                std::io::copy(&mut file, &mut contents).unwrap();
                serde_json::from_slice(&contents).unwrap()
            };
            assert_eq!(api_keys.len(), 1);
            assert_eq!(api_keys[0].get("name").unwrap(), "my-key");

            cleanup_client_data(&pool, client_id).await;
        }

        #[tokio::test]
        async fn test_export_include_audit_log() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO audit_log (actor_key_id, actor_key_name, client_id, resource_type, resource_id, action) \
                 VALUES ($1, 'test-actor', $2, 'blueprint', $3, 'created'::audit_action)",
            )
            .bind(Uuid::new_v4())
            .bind(client_id)
            .bind(Uuid::new_v4())
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![client_id],
                include_api_keys: false,
                include_audit_log: true,
                inline_global_refs: false,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export with audit log failed: {:?}", result.err());
            let response = result.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();
            let folder = format!("client-{}", client_id);

            let entries: Vec<serde_json::Value> = {
                let mut file = archive.by_name(&format!("{}/audit-log.json", folder)).unwrap();
                let mut contents = Vec::new();
                std::io::copy(&mut file, &mut contents).unwrap();
                serde_json::from_slice(&contents).unwrap()
            };
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].get("resourceType").unwrap(), "blueprint");

            cleanup_client_data(&pool, client_id).await;
        }

        #[tokio::test]
        async fn test_export_inline_global_refs() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let gma_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO global_meta_attributes (id, client_id, name, description, value_type, payload) \
                 VALUES ($1, $2, 'rarity', 'Quality tier', 'enum'::value_type, $3)",
            )
            .bind(gma_id)
            .bind(client_id)
            .bind(&json!({"values": ["common", "rare"]}))
            .execute(&pool)
            .await
            .unwrap();

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1, $2, 'Sword', 'sword', 1.0, $3, $4, 0, 0, 0, 0)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind(&json!({
                "rarity": {"$ref_id": gma_id.to_string()}
            }))
            .bind(&vec!["rarity".to_string()])
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![client_id],
                include_api_keys: false,
                include_audit_log: false,
                inline_global_refs: true,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export with inline refs failed: {:?}", result.err());
            let response = result.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();
            let folder = format!("client-{}", client_id);

            let blueprints: Vec<serde_json::Value> = {
                let mut file = archive.by_name(&format!("{}/blueprints.json", folder)).unwrap();
                let mut contents = Vec::new();
                std::io::copy(&mut file, &mut contents).unwrap();
                serde_json::from_slice(&contents).unwrap()
            };

            let attrs = blueprints[0].get("attributes").unwrap();
            let rarity = attrs.get("rarity").unwrap();
            assert!(
                rarity.get("$ref_id").is_none(),
                "$ref_id should be inlined, but got: {:?}", rarity
            );
            assert_eq!(rarity.get("value_type").unwrap(), "enum");
            assert_eq!(rarity.get("values").unwrap(), &json!(["common", "rare"]));

            cleanup_client_data(&pool, client_id).await;
        }

        #[tokio::test]
        async fn test_export_preserves_ref_id_when_inline_false() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let gma_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO global_meta_attributes (id, client_id, name, description, value_type, payload) \
                 VALUES ($1, $2, 'rarity', 'Quality', 'enum'::value_type, $3)",
            )
            .bind(gma_id)
            .bind(client_id)
            .bind(&json!({"values": ["common"]}))
            .execute(&pool)
            .await
            .unwrap();

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1, $2, 'Sword', 'sword', 1.0, $3, $4, 0, 0, 0, 0)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind(&json!({
                "rarity": {"$ref_id": gma_id.to_string()}
            }))
            .bind(&vec!["rarity".to_string()])
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![client_id],
                include_api_keys: false,
                include_audit_log: false,
                inline_global_refs: false,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export preserve refs failed: {:?}", result.err());
            let response = result.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();
            let folder = format!("client-{}", client_id);

            let blueprints: Vec<serde_json::Value> = {
                let mut file = archive.by_name(&format!("{}/blueprints.json", folder)).unwrap();
                let mut contents = Vec::new();
                std::io::copy(&mut file, &mut contents).unwrap();
                serde_json::from_slice(&contents).unwrap()
            };

            let attrs = blueprints[0].get("attributes").unwrap();
            let rarity = attrs.get("rarity").unwrap();
            assert_eq!(
                rarity.get("$ref_id").unwrap(),
                &json!(gma_id.to_string()),
                "$ref_id should be preserved when inline_global_refs=false"
            );

            cleanup_client_data(&pool, client_id).await;
        }

        #[tokio::test]
        async fn test_export_round_trip() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            let gma_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO global_meta_attributes (id, client_id, name, description, value_type, payload) \
                 VALUES ($1, $2, 'rarity', 'Quality tier', 'enum'::value_type, $3)",
            )
            .bind(gma_id)
            .bind(client_id)
            .bind(&json!({"values": ["common", "rare"]}))
            .execute(&pool)
            .await
            .unwrap();

            let bp_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO blueprints (id, client_id, name, archetype, weight, attributes, attribute_order, \
                 min_prefixes, max_prefixes, min_suffixes, max_suffixes) \
                 VALUES ($1, $2, 'Sword', 'sword', 1.0, $3, $4, 0, 2, 0, 0)",
            )
            .bind(bp_id)
            .bind(client_id)
            .bind(&json!({
                "damage": {"value_type": "range", "min": 10.0, "max": 20.0},
                "rarity": {"$ref_id": gma_id.to_string()}
            }))
            .bind(&vec!["damage".to_string(), "rarity".to_string()])
            .execute(&pool)
            .await
            .unwrap();

            let affix_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO affixes (id, client_id, name, type, attribute) \
                 VALUES ($1, $2, 'Fire', 'prefix'::affix_location, $3)",
            )
            .bind(affix_id)
            .bind(client_id)
            .bind(&json!({"fire_damage": {"value_type": "range", "min": 5.0, "max": 15.0}}))
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO blueprint_affixes (id, blueprint_id, affix_id, weight, location, sort_order) \
                 VALUES ($1, $2, $3, 1.0, 'prefix'::affix_location, 0)",
            )
            .bind(Uuid::new_v4())
            .bind(bp_id)
            .bind(affix_id)
            .execute(&pool)
            .await
            .unwrap();

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![client_id],
                include_api_keys: false,
                include_audit_log: false,
                inline_global_refs: false,
            };

            let export_result = export_handler(
                axum::extract::State(state.clone()),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(export_result.is_ok(), "export failed: {:?}", export_result.err());
            let export_response = export_result.unwrap();
            let export_body = axum::body::to_bytes(export_response.into_body(), usize::MAX)
                .await
                .unwrap();

            let parsed = crate::import::parse_zip(
                axum::body::Bytes::from(export_body.to_vec()),
            )
            .unwrap();

            assert_eq!(parsed.len(), 1, "should have 1 client folder");

            let client_import = crate::import::build_client_import(&parsed[0]).unwrap();

            assert_eq!(client_import.client_name, "export-test-client");
            assert_eq!(client_import.client_id, Some(client_id));
            assert_eq!(client_import.blueprints.len(), 1);
            assert_eq!(client_import.blueprints[0].name, "Sword");
            assert_eq!(client_import.affixes.len(), 1);
            assert_eq!(client_import.affixes[0].name, "Fire");
            assert_eq!(client_import.global_meta_attributes.len(), 1);
            assert_eq!(client_import.global_meta_attributes[0].name, "rarity");
            assert_eq!(client_import.blueprint_affixes.len(), 1);
            assert_eq!(client_import.blueprint_affixes[0].blueprint_id, bp_id);

            cleanup_client_data(&pool, client_id).await;
        }

        #[tokio::test]
        async fn test_export_nonexistent_client_skipped() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let real_client = create_test_client(&pool).await;
            let nonexistent = Uuid::new_v4();

            let state = test_state(Arc::new(pool.clone()));

            let req = ExportRequest {
                client_ids: vec![real_client, nonexistent],
                include_api_keys: false,
                include_audit_log: false,
                inline_global_refs: false,
            };

            let result = export_handler(
                axum::extract::State(state),
                super_user(),
                axum::Json(req),
            )
            .await;

            assert!(result.is_ok(), "export with nonexistent client failed: {:?}", result.err());
            let response = result.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            let cursor = Cursor::new(body.as_ref());
            let mut archive = zip::ZipArchive::new(cursor).unwrap();

            assert!(archive.by_name(&format!("client-{}/client.json", real_client)).is_ok());
            assert!(
                archive.by_name(&format!("client-{}/client.json", nonexistent)).is_err(),
                "nonexistent client should be skipped"
            );

            cleanup_client_data(&pool, real_client).await;
        }
    }
}

use arche_types::generate::{
    BatchGenerateRequest, BatchGenerateResponse, BatchGenerateResultItem, ConstraintValue,
    GenerateRequest, GenerateResponse,
};
use arche_types::{Affix, AffixLocation, Blueprint, BlueprintAffix, GlobalMetaAttribute};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::permission::CurrentUser;
use crate::cache::{Cache, ClientCache};
use crate::error::ProblemResponse;
use crate::generation::affix_selection::{select_affixes, AffixSelectionError};
use crate::generation::blueprint_selection::{select_blueprint, BlueprintSelectionError};
use crate::generation::output::{assemble_affix_attributes, compose_name};
use crate::generation::rolling::roll_blueprint_attributes;
use crate::AppState;

pub async fn generate_handler(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    headers: HeaderMap,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, ProblemResponse> {
    let is_cache_refresh = headers
        .get("X-Cache-Refresh")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let client_id = resolve_client_id(&user, req.client_id)?;
    let (client_cache, bp_affixes_lookup) =
        load_client_cache(&state, client_id, is_cache_refresh).await?;

    let seed = req.seed.unwrap_or_else(|| rand::random());

    match run_single_generation(&client_cache, bp_affixes_lookup.as_ref(), &req, seed, &mut None) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err(ProblemResponse::no_matching_blueprints(error)),
    }
}

pub async fn generate_batch_handler(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    headers: HeaderMap,
    Json(req): Json<BatchGenerateRequest>,
) -> Json<BatchGenerateResponse> {
    let is_cache_refresh = headers
        .get("X-Cache-Refresh")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let client_id = match resolve_client_id(&user, None) {
        Ok(cid) => cid,
        Err(problem) => {
            let error_msg = problem
                .detail
                .unwrap_or_else(|| "Auth/cache error".into());
            return Json(BatchGenerateResponse {
                results: req
                    .requests
                    .into_iter()
                    .map(|_| BatchGenerateResultItem {
                        result: None,
                        error: Some(error_msg.clone()),
                    })
                    .collect(),
            });
        }
    };

    let client_cache_result = load_client_cache(&state, client_id, is_cache_refresh).await;

    let (client_cache, bp_affixes_lookup) = match client_cache_result {
        Ok(cache) => cache,
        Err(problem) => {
            let error_msg = problem
                .detail
                .unwrap_or_else(|| "Cache error".into());
            return Json(BatchGenerateResponse {
                results: req
                    .requests
                    .into_iter()
                    .map(|_| BatchGenerateResultItem {
                        result: None,
                        error: Some(error_msg.clone()),
                    })
                    .collect(),
            });
        }
    };

    let results: Vec<BatchGenerateResultItem> = req
        .requests
        .into_iter()
        .map(|req| {
            let seed = req.seed.unwrap_or_else(|| rand::random());
            match run_single_generation(
                &client_cache,
                bp_affixes_lookup.as_ref(),
                &req,
                seed,
                &mut None,
            ) {
                Ok(response) => BatchGenerateResultItem {
                    result: Some(response),
                    error: None,
                },
                Err(error) => BatchGenerateResultItem {
                    result: None,
                    error: Some(error),
                },
            }
        })
        .collect();

    Json(BatchGenerateResponse { results })
}

fn run_single_generation(
    client_cache: &ClientCache,
    bp_affixes_lookup: &HashMap<Uuid, Vec<BlueprintAffix>>,
    req: &GenerateRequest,
    seed: u64,
    blueprint_attr_cache: &mut Option<
        HashMap<Uuid, Arc<BTreeMap<String, serde_json::Value>>>,
    >,
) -> Result<GenerateResponse, String> {
    let mut rng = StdRng::seed_from_u64(seed);

    let blueprint = match select_blueprint(client_cache, req, &mut rng) {
        Ok(bp) => bp,
        Err(BlueprintSelectionError::NoMatchingBlueprints) => {
            return Err(build_no_match_detail(req));
        }
    };

    let blueprint_attributes = if let Some(cache) = blueprint_attr_cache {
        cache
            .entry(blueprint.id)
            .or_insert_with(|| {
                Arc::new(roll_blueprint_attributes(&blueprint, client_cache, seed))
            })
            .as_ref()
            .clone()
    } else {
        roll_blueprint_attributes(&blueprint, client_cache, seed)
    };

    let bp_affix_entries = bp_affixes_lookup
        .get(&blueprint.id)
        .cloned()
        .unwrap_or_default();

    let selected_affixes = match select_affixes(
        &bp_affix_entries,
        &client_cache.affixes,
        &blueprint,
        req.affixes.as_ref(),
        &mut rng,
    ) {
        Ok(selected) => selected,
        Err(AffixSelectionError::InsufficientPool { kind, required, available }) => {
            return Err(format!(
                "not enough {} affixes: need {} but only {} available",
                kind, required, available,
            ));
        }
        Err(AffixSelectionError::RequiredAffixNotAvailable(id)) => {
            return Err(format!(
                "required affix {} not found in blueprint pool",
                id,
            ));
        }
    };

    let prefix_names: Vec<String> = selected_affixes
        .iter()
        .filter(|(a, _)| a.location == AffixLocation::Prefix)
        .map(|(a, _)| a.name.clone())
        .collect();
    let suffix_names: Vec<String> = selected_affixes
        .iter()
        .filter(|(a, _)| a.location == AffixLocation::Suffix)
        .map(|(a, _)| a.name.clone())
        .collect();

    let (name, name_parts) = compose_name(&blueprint.name, &prefix_names, &suffix_names);

    let affix_attributes = assemble_affix_attributes(&selected_affixes, client_cache, &mut rng);

    Ok(GenerateResponse {
        seed,
        name,
        name_parts,
        blueprint_id: blueprint.id,
        blueprint_attributes,
        affix_attributes,
    })
}

fn resolve_client_id(
    user: &crate::auth::AuthenticatedKey,
    request_client_id: Option<Uuid>,
) -> Result<Uuid, ProblemResponse> {
    match user.client_id {
        Some(cid) => Ok(cid),
        None => {
            let cid = request_client_id.ok_or_else(|| {
                ProblemResponse::validation_error(
                    "Super admin must specify a clientId in the request body for generation",
                    vec![],
                )
            })?;
            user.require_client_access(cid)?;
            Ok(cid)
        }
    }
}

async fn load_client_cache(
    state: &AppState,
    client_id: Uuid,
    is_cache_refresh: bool,
) -> Result<(Arc<ClientCache>, Arc<HashMap<Uuid, Vec<BlueprintAffix>>>), ProblemResponse> {
    if is_cache_refresh {
        let data = Cache::fetch_client_data(&state.pool, client_id)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "generate: failed to fetch client data");
                ProblemResponse::unauthorized("Failed to load client data")
            })?;

        let bp_affixes_lookup = data.blueprint_affixes;

        let blueprints: Vec<Arc<Blueprint>> =
            data.blueprints.values().map(|bp| Arc::new(bp.clone())).collect();
        let affixes: Vec<Arc<Affix>> =
            data.affixes.values().map(|a| Arc::new(a.clone())).collect();
        let gmas: Vec<Arc<GlobalMetaAttribute>> = data
            .global_meta_attributes
            .values()
            .map(|gma| Arc::new(gma.clone()))
            .collect();
        let blueprint_resolved_attributes =
            Cache::resolve_blueprint_attributes(&blueprints, &gmas);
        let affix_resolved_attributes =
            Cache::resolve_affix_attributes(&affixes, &gmas);
        let client_cache = Arc::new(ClientCache {
            blueprints,
            affixes,
            global_meta_attributes: gmas,
            blueprint_resolved_attributes,
            affix_resolved_attributes,
        });

        Ok((client_cache, Arc::new(bp_affixes_lookup)))
    } else {
        let cache = state.cache.read().await;

        let cc = cache
            .get_client_data(client_id)
            .ok_or_else(|| ProblemResponse::not_found("Client not found in cache"))?;

        let bp_affixes_lookup = cache.blueprint_affixes.clone();

        Ok((cc.clone(), bp_affixes_lookup))
    }
}

fn build_no_match_detail(req: &GenerateRequest) -> String {
    let mut parts = Vec::new();
    if let Some(ref archetype) = req.archetype {
        parts.push(format!("archetype '{}'", archetype));
    }
    if let Some(ref constraints) = req.constraints {
        for (key, constraint) in constraints {
            let desc = constraint_describe(constraint);
            if !desc.is_empty() {
                parts.push(format!("{} {}", key, desc));
            }
        }
    }
    if parts.is_empty() {
        "No blueprint matches the given constraints".into()
    } else {
        format!("No blueprint satisfies {}", parts.join(", "))
    }
}

fn constraint_describe(constraint: &ConstraintValue) -> String {
    match constraint {
        ConstraintValue::Config(config) => {
            let mut parts = Vec::new();
            if let Some(gte) = config.gte {
                parts.push(format!(">= {}", gte));
            }
            if let Some(lte) = config.lte {
                parts.push(format!("<= {}", lte));
            }
            if let Some(ref inv) = config.r#in {
                parts.push(format!("in {:?}", inv));
            }
            if let Some(ref contains) = config.contains {
                parts.push(format!("contains '{}'", contains));
            }
            if let Some(ref eq) = config.eq {
                parts.push(format!("= {}", eq));
            }
            parts.join(" and ")
        }
        ConstraintValue::Bare(value) => {
            format!("= {}", value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arche_types::generate::NameParts;
    use arche_types::Blueprint;
    use chrono::Utc;

    fn test_ts() -> chrono::DateTime<Utc> {
        chrono::DateTime::from_timestamp_millis(0).unwrap()
    }

    fn make_blueprint(id: Uuid, client_id: Uuid, name: &str) -> Blueprint {
        Blueprint {
            id,
            client_id,
            name: name.into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({
                "damage": {"value_type": "range", "min": 10.0, "max": 20.0},
                "quality": {"value_type": "enum", "values": ["common", "rare", "legendary"]},
            }),
            attribute_order: vec!["damage".into(), "quality".into()],
            min_prefixes: 0,
            max_prefixes: 1,
            min_suffixes: 0,
            max_suffixes: 1,
            created_at: test_ts(),
            updated_at: test_ts(),
        }
    }

    fn make_client_cache_with_bp() -> (ClientCache, HashMap<Uuid, Vec<BlueprintAffix>>) {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(bp_id, client_id, "Longsword");
        let bp_arcs = vec![Arc::new(bp)];
        let gmas = vec![];
        let blueprint_resolved_attributes =
            Cache::resolve_blueprint_attributes(&bp_arcs, &gmas);
        let bp_affixes = HashMap::new();
        let cache = ClientCache {
            blueprints: bp_arcs,
            affixes: vec![],
            global_meta_attributes: gmas,
            blueprint_resolved_attributes,
            affix_resolved_attributes: HashMap::new(),
        };
        (cache, bp_affixes)
    }

    #[test]
    fn test_run_single_generation_success() {
        let (cache, bp_affixes) = make_client_cache_with_bp();
        let req = GenerateRequest {
            archetype: Some("sword".into()),
            seed: Some(42),
            constraints: None,
            affixes: None,
            client_id: None,
        };

        let result = run_single_generation(&cache, &bp_affixes, &req, 42, &mut None);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.seed, 42);
        assert!(response.name.contains("Longsword"));
        assert!(response.blueprint_attributes.contains_key("damage"));
        assert!(response.blueprint_attributes.contains_key("quality"));
    }

    #[test]
    fn test_run_single_generation_deterministic() {
        let (cache, bp_affixes) = make_client_cache_with_bp();
        let req = GenerateRequest {
            archetype: Some("sword".into()),
            seed: Some(42),
            constraints: None,
            affixes: None,
            client_id: None,
        };

        let r1 = run_single_generation(&cache, &bp_affixes, &req, 42, &mut None).unwrap();
        let r2 = run_single_generation(&cache, &bp_affixes, &req, 42, &mut None).unwrap();
        assert_eq!(r1.name, r2.name);
        assert_eq!(r1.blueprint_attributes, r2.blueprint_attributes);
        assert_eq!(r1.affix_attributes, r2.affix_attributes);
    }

    #[test]
    fn test_run_single_generation_no_match() {
        let (cache, bp_affixes) = make_client_cache_with_bp();
        let req = GenerateRequest {
            archetype: Some("axe".into()),
            seed: Some(42),
            constraints: None,
            affixes: None,
            client_id: None,
        };

        let result = run_single_generation(&cache, &bp_affixes, &req, 42, &mut None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No blueprint"));
    }

    #[test]
    fn test_run_single_generation_seed_override() {
        let (cache, bp_affixes) = make_client_cache_with_bp();
        let req = GenerateRequest {
            archetype: Some("sword".into()),
            seed: None,
            constraints: None,
            affixes: None,
            client_id: None,
        };

        let r1 = run_single_generation(&cache, &bp_affixes, &req, 100, &mut None).unwrap();
        let r2 = run_single_generation(&cache, &bp_affixes, &req, 200, &mut None).unwrap();
        assert_eq!(r1.seed, 100);
        assert_eq!(r2.seed, 200);
        assert!(r1.name != r2.name || r1.blueprint_attributes != r2.blueprint_attributes);
    }

    #[test]
    fn test_blueprint_attribute_caching() {
        let (cache, bp_affixes) = make_client_cache_with_bp();
        let bp_id = cache.blueprints[0].id;
        let req = GenerateRequest {
            archetype: Some("sword".into()),
            seed: Some(42),
            constraints: None,
            affixes: None,
            client_id: None,
        };

        let mut attr_cache: Option<HashMap<Uuid, Arc<BTreeMap<String, serde_json::Value>>>> =
            Some(HashMap::new());

        let r1 = run_single_generation(&cache, &bp_affixes, &req, 42, &mut attr_cache).unwrap();
        assert_eq!(attr_cache.as_ref().unwrap().len(), 1);
        assert!(attr_cache.as_ref().unwrap().contains_key(&bp_id));

        let r2 = run_single_generation(&cache, &bp_affixes, &req, 42, &mut attr_cache).unwrap();
        assert_eq!(r1.blueprint_attributes, r2.blueprint_attributes);
    }

    #[test]
    fn test_batch_generate_mixed_success_failure() {
        let (cache, bp_affixes) = make_client_cache_with_bp();

        let req_ok = GenerateRequest {
            archetype: Some("sword".into()),
            seed: Some(42),
            constraints: None,
            affixes: None,
            client_id: None,
        };

        let req_err = GenerateRequest {
            archetype: Some("axe".into()),
            seed: Some(99),
            constraints: None,
            affixes: None,
            client_id: None,
        };

        let r1 = run_single_generation(&cache, &bp_affixes, &req_ok, 42, &mut None);
        let r2 = run_single_generation(&cache, &bp_affixes, &req_err, 99, &mut None);

        assert!(r1.is_ok());
        assert!(r2.is_err());
    }

    #[test]
    fn test_batch_generate_result_item_round_trip() {
        let bp_id = Uuid::new_v4();
        let result_item = BatchGenerateResultItem {
            result: Some(GenerateResponse {
                seed: 42,
                name: "Test".into(),
                name_parts: NameParts {
                    base: "Test".into(),
                    prefixes: vec![],
                    suffixes: vec![],
                },
                blueprint_id: bp_id,
                blueprint_attributes: BTreeMap::new(),
                affix_attributes: vec![],
            }),
            error: None,
        };
        let value = serde_json::to_value(&result_item).unwrap();
        let deserialized: BatchGenerateResultItem = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, result_item);
    }

    #[test]
    fn test_batch_generate_result_item_error() {
        let result_item = BatchGenerateResultItem {
            result: None,
            error: Some("No matching blueprint".into()),
        };
        let value = serde_json::to_value(&result_item).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("error"));
        assert!(!obj.contains_key("result"));
        let deserialized: BatchGenerateResultItem = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, result_item);
    }
}

use arche_types::generate::{ConstraintValue, GenerateRequest, GenerateResponse};
use arche_types::{AffixLocation, BlueprintAffix};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
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
    let client_id = user
        .client_id
        .ok_or_else(|| ProblemResponse::forbidden("API key is not scoped to a client"))?;

    let is_cache_refresh = headers
        .get("X-Cache-Refresh")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let seed = req.seed.unwrap_or_else(|| rand::random());

    let client_cache: ClientCache;
    let bp_affixes_lookup: HashMap<Uuid, Vec<BlueprintAffix>>;

    if is_cache_refresh {
        let data = Cache::fetch_client_data(&state.pool, client_id)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "generate: failed to fetch client data");
                ProblemResponse::unauthorized("Failed to load client data")
            })?;

        bp_affixes_lookup = data.blueprint_affixes;

        client_cache = ClientCache {
            blueprints: data
                .blueprints
                .values()
                .map(|bp| std::sync::Arc::new(bp.clone()))
                .collect(),
            affixes: data
                .affixes
                .values()
                .map(|a| std::sync::Arc::new(a.clone()))
                .collect(),
            global_meta_attributes: data
                .global_meta_attributes
                .values()
                .map(|gma| std::sync::Arc::new(gma.clone()))
                .collect(),
        };
    } else {
        let cache = state.cache.read().await;

        let cc = cache
            .get_client_data(client_id)
            .ok_or_else(|| ProblemResponse::not_found("Client not found in cache"))?;

        client_cache = ClientCache {
            blueprints: cc.blueprints.clone(),
            affixes: cc.affixes.clone(),
            global_meta_attributes: cc.global_meta_attributes.clone(),
        };

        bp_affixes_lookup = cache.blueprint_affixes.clone();
    }

    let mut rng = StdRng::seed_from_u64(seed);

    let blueprint = match select_blueprint(&client_cache, &req, &mut rng) {
        Ok(bp) => bp,
        Err(BlueprintSelectionError::NoMatchingBlueprints) => {
            let detail = build_no_match_detail(&req);
            return Err(ProblemResponse::no_matching_blueprints(detail));
        }
    };

    let blueprint_attributes = roll_blueprint_attributes(&blueprint, &client_cache, seed);

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
            return Err(ProblemResponse::unprocessable_entity(format!(
                "not enough {} affixes: need {} but only {} available",
                kind, required, available,
            )));
        }
        Err(AffixSelectionError::RequiredAffixNotAvailable(id)) => {
            return Err(ProblemResponse::unprocessable_entity(format!(
                "required affix {} not found in blueprint pool",
                id,
            )));
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

    let affix_attributes = assemble_affix_attributes(&selected_affixes, &client_cache, &mut rng);

    Ok(Json(GenerateResponse {
        seed,
        name,
        name_parts,
        blueprint_id: blueprint.id,
        blueprint_attributes,
        affix_attributes,
    }))
}

fn build_no_match_detail(req: &GenerateRequest) -> String {
    let mut parts: Vec<String> = Vec::new();
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
            let mut parts: Vec<String> = Vec::new();
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

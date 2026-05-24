use arche_types::attribute::{AttributePayload, BlueprintAttribute};
use arche_types::generate::{ConstraintValue, GenerateRequest};
use arche_types::{Blueprint, GlobalMetaAttribute};
use crate::cache::ClientCache;
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum BlueprintSelectionError {
    #[error("no blueprints match the requested criteria")]
    NoMatchingBlueprints,
}

pub fn select_blueprint(
    client_cache: &ClientCache,
    request: &GenerateRequest,
    rng: &mut impl Rng,
) -> Result<Arc<Blueprint>, BlueprintSelectionError> {
    let mut candidates: Vec<&Arc<Blueprint>> = client_cache.blueprints.iter().collect();

    if let Some(ref archetype) = request.archetype {
        candidates.retain(|bp| bp.archetype == *archetype);
    }

    if let Some(ref constraints) = request.constraints {
        candidates.retain(|bp| {
            constraints.iter().all(|(key, constraint)| {
                blueprint_satisfies_constraint(bp, key, constraint, &client_cache.global_meta_attributes)
            })
        });
    }

    if candidates.is_empty() {
        return Err(BlueprintSelectionError::NoMatchingBlueprints);
    }

    if candidates.len() == 1 {
        return Ok(Arc::clone(candidates[0]));
    }

    let weights: Vec<f64> = candidates.iter().map(|bp| bp.weight).collect();
    let dist = WeightedIndex::new(&weights).expect("all weights are > 0 by validation");
    let idx = dist.sample(rng);
    Ok(Arc::clone(candidates[idx]))
}

fn blueprint_satisfies_constraint(
    bp: &Blueprint,
    key: &str,
    constraint: &ConstraintValue,
    gmas: &[Arc<GlobalMetaAttribute>],
) -> bool {
    let attr_value = match bp.attributes.get(key) {
        Some(v) => v,
        None => return false,
    };

    let payload = match resolve_payload(attr_value, gmas) {
        Some(p) => p,
        None => return false,
    };

    payload_matches_constraint(&payload, constraint)
}

fn resolve_payload(
    attr_value: &serde_json::Value,
    gmas: &[Arc<GlobalMetaAttribute>],
) -> Option<AttributePayload> {
    let bp_attr: BlueprintAttribute = serde_json::from_value(attr_value.clone()).ok()?;
    match bp_attr {
        BlueprintAttribute::Inline(inline) => Some(inline.payload),
        BlueprintAttribute::Ref { ref_id } => {
            let gma = gmas.iter().find(|gma| gma.id == ref_id)?;
            serde_json::from_value(gma.payload.clone()).ok()
        }
    }
}

fn payload_matches_constraint(payload: &AttributePayload, constraint: &ConstraintValue) -> bool {
    match constraint {
        ConstraintValue::Config(config) => match payload {
            AttributePayload::Range { min, max, .. } => {
                config.gte.is_none_or(|gte| *max >= gte)
                    && config.lte.is_none_or(|lte| *min <= lte)
            }
            AttributePayload::Enum { values } => {
                if let Some(ref in_values) = config.r#in {
                    values.iter().any(|v| in_values.contains(v))
                } else if let Some(ref eq_val) = config.eq {
                    eq_val
                        .as_str()
                        .is_some_and(|s| values.iter().any(|v| v == s))
                } else {
                    false
                }
            }
            AttributePayload::String { .. } => {
                config.contains.is_some() || config.eq.as_ref().is_some_and(|v| v.is_string())
            }
            AttributePayload::Boolean { value: bp_value } => {
                config.eq.as_ref().is_some_and(|v| v.as_bool() == Some(*bp_value))
            }
            AttributePayload::Single { value, .. } => {
                config.eq.as_ref().is_some_and(|v| {
                    v.as_f64().is_some_and(|v2| (v2 - value).abs() < f64::EPSILON)
                })
            }
        },
        ConstraintValue::Bare(value) => match payload {
            AttributePayload::Boolean { value: bp_value } => {
                value.as_bool() == Some(*bp_value)
            }
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arche_types::attribute::*;
    use arche_types::generate::*;
    use chrono::{TimeZone, Utc};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn ts() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    }

    fn make_blueprint(
        id: Uuid,
        client_id: Uuid,
        name: &str,
        archetype: &str,
        weight: f64,
        attributes: serde_json::Value,
    ) -> Blueprint {
        Blueprint {
            id,
            client_id,
            name: name.into(),
            archetype: archetype.into(),
            weight,
            description: None,
            attributes,
            attribute_order: vec![],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts(),
            updated_at: ts(),
        }
    }

    fn make_gma(id: Uuid, client_id: Uuid, name: &str, payload: AttributePayload) -> GlobalMetaAttribute {
        GlobalMetaAttribute {
            id,
            client_id,
            name: name.into(),
            description: None,
            value_type: arche_types::ValueType::Range,
            payload: serde_json::to_value(&payload).unwrap(),
            created_at: ts(),
            updated_at: ts(),
        }
    }

    fn make_client_cache(
        blueprints: Vec<Blueprint>,
        gmas: Vec<GlobalMetaAttribute>,
    ) -> (ClientCache, Vec<Arc<Blueprint>>) {
        let bp_arcs: Vec<Arc<Blueprint>> = blueprints.into_iter().map(Arc::new).collect();
        let gma_arcs: Vec<Arc<GlobalMetaAttribute>> = gmas.into_iter().map(Arc::new).collect();
        let cc = ClientCache {
            blueprints: bp_arcs.clone(),
            affixes: vec![],
            global_meta_attributes: gma_arcs,
        };
        (cc, bp_arcs)
    }

    fn range_attr(min: f64, max: f64) -> serde_json::Value {
        serde_json::to_value(BlueprintAttribute::Inline(InlineAttributeDef {
            description: None,
            payload: AttributePayload::Range {
                min,
                max,
                distribution: None,
            },
        }))
        .unwrap()
    }

    fn enum_attr(values: Vec<&str>) -> serde_json::Value {
        serde_json::to_value(BlueprintAttribute::Inline(InlineAttributeDef {
            description: None,
            payload: AttributePayload::Enum {
                values: values.into_iter().map(String::from).collect(),
            },
        }))
        .unwrap()
    }

    fn string_attr() -> serde_json::Value {
        serde_json::to_value(BlueprintAttribute::Inline(InlineAttributeDef {
            description: None,
            payload: AttributePayload::String {
                min_length: Some(3),
                max_length: Some(30),
            },
        }))
        .unwrap()
    }

    fn boolean_attr(value: bool) -> serde_json::Value {
        serde_json::to_value(BlueprintAttribute::Inline(InlineAttributeDef {
            description: None,
            payload: AttributePayload::Boolean { value },
        }))
        .unwrap()
    }

    // --- Archetype filter ---

    #[test]
    fn test_archetype_filter_only_matching_considered() {
        let client_id = Uuid::new_v4();
        let sword_id = Uuid::new_v4();
        let axe_id = Uuid::new_v4();

        let sword = make_blueprint(sword_id, client_id, "Longsword", "sword", 1.0, serde_json::json!({}));
        let axe = make_blueprint(axe_id, client_id, "Battle Axe", "axe", 1.0, serde_json::json!({}));

        let (cc, _) = make_client_cache(vec![sword, axe], vec![]);

        let mut rng = StdRng::seed_from_u64(42);
        let req = GenerateRequest {
            archetype: Some("sword".into()),
            seed: Some(42),
            constraints: None,
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, sword_id);
        assert_eq!(result.archetype, "sword");
    }

    #[test]
    fn test_archetype_filter_no_match_returns_error() {
        let client_id = Uuid::new_v4();
        let sword = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Longsword",
            "sword",
            1.0,
            serde_json::json!({}),
        );
        let (cc, _) = make_client_cache(vec![sword], vec![]);

        let mut rng = StdRng::seed_from_u64(42);
        let req = GenerateRequest {
            archetype: Some("bow".into()),
            seed: Some(42),
            constraints: None,
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(
            matches!(err, BlueprintSelectionError::NoMatchingBlueprints)
        );
    }

    // --- Range constraint: gte ---

    #[test]
    fn test_range_gte_overlap_matches() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Longsword",
            "sword",
            1.0,
            serde_json::json!({"damage": range_attr(10.0, 23.0)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(15.0),
                lte: None,
                r#in: None,
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    #[test]
    fn test_range_gte_no_overlap_rejected() {
        let client_id = Uuid::new_v4();
        let bp = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Weak Sword",
            "sword",
            1.0,
            serde_json::json!({"damage": range_attr(5.0, 10.0)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(15.0),
                lte: None,
                r#in: None,
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(matches!(err, BlueprintSelectionError::NoMatchingBlueprints));
    }

    // --- Range constraint: lte ---

    #[test]
    fn test_range_lte_overlap_matches() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Longsword",
            "sword",
            1.0,
            serde_json::json!({"damage": range_attr(10.0, 23.0)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: Some(40.0),
                r#in: None,
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    #[test]
    fn test_range_lte_no_overlap_rejected() {
        let client_id = Uuid::new_v4();
        let bp = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Overpowered",
            "sword",
            1.0,
            serde_json::json!({"damage": range_attr(50.0, 100.0)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: Some(30.0),
                r#in: None,
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(matches!(err, BlueprintSelectionError::NoMatchingBlueprints));
    }

    // --- Range constraint: gte + lte combined ---

    #[test]
    fn test_range_gte_lte_combined_both_satisfied() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Longsword",
            "sword",
            1.0,
            serde_json::json!({"damage": range_attr(10.0, 23.0)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(10.0),
                lte: Some(25.0),
                r#in: None,
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    #[test]
    fn test_range_gte_lte_combined_one_fails_rejected() {
        let client_id = Uuid::new_v4();
        let bp = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Longsword",
            "sword",
            1.0,
            serde_json::json!({"damage": range_attr(10.0, 23.0)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(30.0),
                lte: Some(50.0),
                r#in: None,
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(matches!(err, BlueprintSelectionError::NoMatchingBlueprints));
    }

    // --- Enum constraint (in) ---

    #[test]
    fn test_enum_in_overlap_matches() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Rare Sword",
            "sword",
            1.0,
            serde_json::json!({"rarity": enum_attr(vec!["common", "rare", "legendary"])}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "rarity".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: None,
                r#in: Some(vec!["rare".into(), "legendary".into()]),
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    #[test]
    fn test_enum_in_no_overlap_rejected() {
        let client_id = Uuid::new_v4();
        let bp = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Common Sword",
            "sword",
            1.0,
            serde_json::json!({"rarity": enum_attr(vec!["common", "uncommon"])}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "rarity".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: None,
                r#in: Some(vec!["rare".into(), "legendary".into()]),
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(matches!(err, BlueprintSelectionError::NoMatchingBlueprints));
    }

    // --- String constraint (contains) ---

    #[test]
    fn test_string_contains_matches() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Magic Sword",
            "sword",
            1.0,
            serde_json::json!({"description": string_attr()}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "description".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: None,
                r#in: None,
                contains: Some("fire".into()),
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    // --- Boolean constraint ---

    #[test]
    fn test_boolean_exact_match_bare_constraint() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Magic Sword",
            "sword",
            1.0,
            serde_json::json!({"magical": boolean_attr(true)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert("magical".into(), ConstraintValue::Bare(serde_json::json!(true)));

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    #[test]
    fn test_boolean_exact_match_config_constraint() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Magic Sword",
            "sword",
            1.0,
            serde_json::json!({"magical": boolean_attr(true)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "magical".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: None,
                r#in: None,
                contains: None,
                eq: Some(serde_json::json!(true)),
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    // --- Multiple constraints (AND logic) ---

    #[test]
    fn test_multiple_constraints_all_match() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(
            bp_id,
            client_id,
            "Rare Fire Sword",
            "sword",
            1.0,
            serde_json::json!({
                "damage": range_attr(10.0, 23.0),
                "rarity": enum_attr(vec!["common", "rare", "legendary"]),
            }),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(15.0),
                lte: None,
                r#in: None,
                contains: None,
                eq: None,
            }),
        );
        constraints.insert(
            "rarity".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: None,
                r#in: Some(vec!["rare".into(), "legendary".into()]),
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    #[test]
    fn test_multiple_constraints_one_fails_rejected() {
        let client_id = Uuid::new_v4();
        let bp = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Rare Fire Sword",
            "sword",
            1.0,
            serde_json::json!({
                "damage": range_attr(5.0, 10.0),
                "rarity": enum_attr(vec!["common", "rare", "legendary"]),
            }),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(15.0),
                lte: None,
                r#in: None,
                contains: None,
                eq: None,
            }),
        );
        constraints.insert(
            "rarity".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: None,
                r#in: Some(vec!["rare".into(), "legendary".into()]),
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(matches!(err, BlueprintSelectionError::NoMatchingBlueprints));
    }

    // --- Blueprint without the constrained attribute does NOT match ---

    #[test]
    fn test_blueprint_without_constrained_attribute_rejected() {
        let client_id = Uuid::new_v4();
        let bp = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Plain Sword",
            "sword",
            1.0,
            serde_json::json!({"damage": range_attr(10.0, 23.0)}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "rarity".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: None,
                lte: None,
                r#in: Some(vec!["rare".into()]),
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(matches!(err, BlueprintSelectionError::NoMatchingBlueprints));
    }

    // --- $ref_id resolution ---

    #[test]
    fn test_ref_id_resolution_constraint_match() {
        let client_id = Uuid::new_v4();
        let gma_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let gma = make_gma(gma_id, client_id, "damage", AttributePayload::Range {
            min: 10.0,
            max: 50.0,
            distribution: None,
        });

        let bp = make_blueprint(
            bp_id,
            client_id,
            "Magic Staff",
            "staff",
            1.0,
            serde_json::json!({"damage": {"$ref_id": gma_id.to_string()}}),
        );
        let (cc, _) = make_client_cache(vec![bp], vec![gma]);

        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(30.0),
                lte: None,
                r#in: None,
                contains: None,
                eq: None,
            }),
        );

        let mut rng = StdRng::seed_from_u64(99);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: Some(constraints),
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    // --- Weighted random selection ---

    #[test]
    fn test_weighted_random_deterministic_with_seed() {
        let client_id = Uuid::new_v4();
        let bp1 = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Sword A",
            "sword",
            1.0,
            serde_json::json!({}),
        );
        let bp2 = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Sword B",
            "sword",
            2.0,
            serde_json::json!({}),
        );
        let bp3 = make_blueprint(
            Uuid::new_v4(),
            client_id,
            "Sword C",
            "sword",
            1.0,
            serde_json::json!({}),
        );

        let (cc, _) = make_client_cache(vec![bp1, bp2, bp3], vec![]);

        let mut rng1 = StdRng::seed_from_u64(12345);
        let req = GenerateRequest {
            archetype: None,
            seed: Some(12345),
            constraints: None,
            affixes: None,
        };
        let result1 = select_blueprint(&cc, &req, &mut rng1).unwrap();

        let mut rng2 = StdRng::seed_from_u64(12345);
        let result2 = select_blueprint(&cc, &req, &mut rng2).unwrap();

        assert_eq!(result1.id, result2.id, "same seed must produce same blueprint");
    }

    #[test]
    fn test_weighted_random_higher_weight_favored() {
        let client_id = Uuid::new_v4();
        let high_weight_id = Uuid::new_v4();
        let low_weight_id = Uuid::new_v4();

        let high = make_blueprint(
            high_weight_id,
            client_id,
            "High Weight",
            "sword",
            99.0,
            serde_json::json!({}),
        );
        let low = make_blueprint(
            low_weight_id,
            client_id,
            "Low Weight",
            "sword",
            1.0,
            serde_json::json!({}),
        );

        let (cc, _) = make_client_cache(vec![high, low], vec![]);
        let mut high_count = 0;
        let trials = 1000;

        for seed in 0..trials {
            let mut rng = StdRng::seed_from_u64(seed);
            let req = GenerateRequest {
                archetype: None,
                seed: Some(seed),
                constraints: None,
                affixes: None,
            };
            let result = select_blueprint(&cc, &req, &mut rng).unwrap();
            if result.id == high_weight_id {
                high_count += 1;
            }
        }

        assert!(
            high_count > 900,
            "high-weight blueprint should be selected most of the time, got {}/{}",
            high_count,
            trials,
        );
    }

    #[test]
    fn test_select_blueprint_single_candidate() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let bp = make_blueprint(bp_id, client_id, "Only Sword", "sword", 1.0, serde_json::json!({}));
        let (cc, _) = make_client_cache(vec![bp], vec![]);

        let mut rng = StdRng::seed_from_u64(42);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: None,
            affixes: None,
        };

        let result = select_blueprint(&cc, &req, &mut rng).unwrap();
        assert_eq!(result.id, bp_id);
    }

    #[test]
    fn test_no_matching_blueprints_returns_error() {
        let (cc, _) = make_client_cache(vec![], vec![]);

        let mut rng = StdRng::seed_from_u64(42);
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: None,
            affixes: None,
        };

        let err = select_blueprint(&cc, &req, &mut rng).unwrap_err();
        assert!(
            matches!(err, BlueprintSelectionError::NoMatchingBlueprints)
        );
    }
}

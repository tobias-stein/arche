use arche_types::attribute::*;
use arche_types::*;
use crate::cache::ClientCache;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, HashMap};


pub fn roll_blueprint_attributes(
    blueprint: &Blueprint,
    client_cache: &ClientCache,
    seed: u64,
) -> BTreeMap<String, serde_json::Value> {
    let resolved = resolve_attributes(blueprint, client_cache);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut rolled = BTreeMap::new();

    for key in &blueprint.attribute_order {
        if let Some(payload) = resolved.get(key) {
            rolled.insert(key.clone(), roll_attribute(payload, &mut rng));
        }
    }

    rolled
}

fn resolve_attributes(
    blueprint: &Blueprint,
    client_cache: &ClientCache,
) -> HashMap<String, AttributePayload> {
    let mut resolved: HashMap<String, AttributePayload> = HashMap::new();

    let attrs = match blueprint.attributes.as_object() {
        Some(obj) => obj,
        None => return resolved,
    };

    for (key, attr_value) in attrs {
        let bp_attr: BlueprintAttribute = match serde_json::from_value(attr_value.clone()) {
            Ok(a) => a,
            Err(_) => continue,
        };

        match bp_attr {
            BlueprintAttribute::Ref { ref_id } => {
                if let Some(gma) = client_cache
                    .global_meta_attributes
                    .iter()
                    .find(|g| g.id == ref_id)
                {
                    if let Ok(payload) = serde_json::from_value(gma.payload.clone()) {
                        resolved.insert(key.clone(), payload);
                    }
                }
            }
            BlueprintAttribute::Inline(inline_def) => {
                resolved.insert(key.clone(), inline_def.payload);
            }
        }
    }

    resolved
}

fn roll_attribute<R: Rng + ?Sized>(payload: &AttributePayload, rng: &mut R) -> serde_json::Value {
    match payload {
        AttributePayload::Single { value, .. } => serde_json::json!(value),
        AttributePayload::Enum { values } => {
            if values.is_empty() {
                return serde_json::Value::Null;
            }
            let idx = rng.gen_range(0..values.len());
            serde_json::json!(&values[idx])
        }
        AttributePayload::Range {
            min,
            max,
            distribution,
        } => {
            let val = roll_range(*min, *max, distribution.as_ref(), rng);
            serde_json::json!(val)
        }
        AttributePayload::String { .. } => serde_json::Value::Null,
        AttributePayload::Boolean { .. } => serde_json::json!(true),
    }
}

fn roll_range<R: Rng + ?Sized>(
    min: f64,
    max: f64,
    distribution: Option<&DistributionConfig>,
    rng: &mut R,
) -> f64 {
    match distribution {
        None | Some(DistributionConfig::Uniform) => min + (max - min) * rng.gen::<f64>(),
        Some(DistributionConfig::Normal { std_dev }) => {
            let u1: f64 = rng.gen::<f64>().max(f64::EPSILON);
            let u2: f64 = rng.gen();
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let mean = (min + max) / 2.0;
            (mean + z * std_dev).clamp(min, max)
        }
        Some(DistributionConfig::Exponential { rate }) => {
            let u: f64 = rng.gen::<f64>().max(f64::EPSILON);
            (min + (-u.ln()) / rate).clamp(min, max)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Utc;
    use proptest::prelude::*;
    use std::sync::Arc;
    use uuid::Uuid;

    fn ts() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    }

    fn make_global(name: &str, id: Uuid, payload: serde_json::Value) -> Arc<GlobalMetaAttribute> {
        Arc::new(GlobalMetaAttribute {
            id,
            client_id: Uuid::nil(),
            name: name.into(),
            description: None,
            value_type: ValueType::Range,
            payload,
            created_at: ts(),
            updated_at: ts(),
        })
    }

    fn make_client_cache(globals: Vec<Arc<GlobalMetaAttribute>>) -> ClientCache {
        ClientCache {
            blueprints: vec![],
            affixes: vec![],
            global_meta_attributes: globals,
        }
    }

    // --- Resolution tests ---

    #[test]
    fn test_resolve_inline_attribute() {
        let bp = Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::nil(),
            name: "Test".into(),
            archetype: "test".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({
                "damage": {"value_type": "range", "min": 1.0, "max": 10.0}
            }),
            attribute_order: vec!["damage".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts(),
            updated_at: ts(),
        };

        let cache = make_client_cache(vec![]);
        let resolved = resolve_attributes(&bp, &cache);

        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key("damage"));
        match resolved.get("damage").unwrap() {
            AttributePayload::Range { min, max, .. } => {
                assert!((*min - 1.0).abs() < 1e-10);
                assert!((*max - 10.0).abs() < 1e-10);
            }
            _ => panic!("expected Range"),
        }
    }

    #[test]
    fn test_resolve_ref_id_from_global_pool() {
        let global_id = Uuid::new_v4();
        let global = make_global(
            "damage",
            global_id,
            serde_json::json!({"value_type": "range", "min": 5.0, "max": 20.0}),
        );

        let bp = Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::nil(),
            name: "Test".into(),
            archetype: "test".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({
                "damage": {"$ref_id": global_id.to_string()}
            }),
            attribute_order: vec!["damage".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts(),
            updated_at: ts(),
        };

        let cache = make_client_cache(vec![global]);
        let resolved = resolve_attributes(&bp, &cache);

        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key("damage"));
        match resolved.get("damage").unwrap() {
            AttributePayload::Range { min, max, .. } => {
                assert!((*min - 5.0).abs() < 1e-10);
                assert!((*max - 20.0).abs() < 1e-10);
            }
            _ => panic!("expected Range"),
        }
    }

    #[test]
    fn test_inline_overrides_ref_on_same_key() {
        let global_id = Uuid::new_v4();
        let global = make_global(
            "rarity",
            global_id,
            serde_json::json!({"value_type": "enum", "values": ["common", "rare"]}),
        );

        let bp = Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::nil(),
            name: "Test".into(),
            archetype: "test".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({
                "rarity": {"$ref_id": global_id.to_string()},
                "damage": {"value_type": "range", "min": 1.0, "max": 10.0}
            }),
            attribute_order: vec!["damage".into(), "rarity".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts(),
            updated_at: ts(),
        };

        let cache = make_client_cache(vec![global]);
        let resolved = resolve_attributes(&bp, &cache);

        assert_eq!(resolved.len(), 2);
        assert!(resolved.contains_key("rarity"));
        match resolved.get("rarity").unwrap() {
            AttributePayload::Enum { values } => {
                assert_eq!(values, &vec!["common", "rare"]);
            }
            _ => panic!("expected Enum"),
        }
        assert!(resolved.contains_key("damage"));
    }

    #[test]
    fn test_resolve_missing_ref_skipped() {
        let bp = Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::nil(),
            name: "Test".into(),
            archetype: "test".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({
                "damage": {"$ref_id": Uuid::new_v4().to_string()}
            }),
            attribute_order: vec!["damage".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts(),
            updated_at: ts(),
        };

        let cache = make_client_cache(vec![]);
        let resolved = resolve_attributes(&bp, &cache);

        assert!(resolved.is_empty());
    }

    // --- Value rolling tests ---

    #[test]
    fn test_roll_single_value() {
        let payload = AttributePayload::Single {
            value: 42.0,
            distribution: None,
        };
        let mut rng = StdRng::seed_from_u64(12345);
        let result = roll_attribute(&payload, &mut rng);
        assert_eq!(result, serde_json::json!(42.0));
    }

    #[test]
    fn test_roll_enum_uniformly() {
        let payload = AttributePayload::Enum {
            values: vec!["a".into(), "b".into(), "c".into()],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = roll_attribute(&payload, &mut rng);
        assert!(matches!(result, serde_json::Value::String(_)));
    }

    #[test]
    fn test_roll_enum_deterministic() {
        let payload = AttributePayload::Enum {
            values: vec!["x".into(), "y".into(), "z".into()],
        };

        let mut rng1 = StdRng::seed_from_u64(999);
        let result1 = roll_attribute(&payload, &mut rng1);

        let mut rng2 = StdRng::seed_from_u64(999);
        let result2 = roll_attribute(&payload, &mut rng2);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_roll_enum_empty_returns_null() {
        let payload = AttributePayload::Enum { values: vec![] };
        let mut rng = StdRng::seed_from_u64(1);
        let result = roll_attribute(&payload, &mut rng);
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_roll_range_uniform_in_bounds() {
        let payload = AttributePayload::Range {
            min: 5.0,
            max: 15.0,
            distribution: Some(DistributionConfig::Uniform),
        };

        let mut rng = StdRng::seed_from_u64(777);
        let results: Vec<f64> = (0..100)
            .map(|_| {
                match roll_attribute(&payload, &mut rng) {
                    serde_json::Value::Number(n) => n.as_f64().unwrap(),
                    _ => panic!("expected number"),
                }
            })
            .collect();

        for &v in &results {
            assert!(v >= 5.0 && v <= 15.0, "value {v} out of bounds");
        }
    }

    #[test]
    fn test_roll_range_uniform_deterministic() {
        let payload = AttributePayload::Range {
            min: 0.0,
            max: 100.0,
            distribution: None,
        };

        let mut rng1 = StdRng::seed_from_u64(42);
        let v1 = match roll_attribute(&payload, &mut rng1) {
            serde_json::Value::Number(n) => n.as_f64().unwrap(),
            _ => panic!("expected number"),
        };

        let mut rng2 = StdRng::seed_from_u64(42);
        let v2 = match roll_attribute(&payload, &mut rng2) {
            serde_json::Value::Number(n) => n.as_f64().unwrap(),
            _ => panic!("expected number"),
        };

        assert!((v1 - v2).abs() < 1e-10);
    }

    #[test]
    fn test_roll_range_normal_clamped() {
        let payload = AttributePayload::Range {
            min: 10.0,
            max: 20.0,
            distribution: Some(DistributionConfig::Normal { std_dev: 1.0 }),
        };

        let mut rng = StdRng::seed_from_u64(123);
        let results: Vec<f64> = (0..1000)
            .map(|_| {
                match roll_attribute(&payload, &mut rng) {
                    serde_json::Value::Number(n) => n.as_f64().unwrap(),
                    _ => panic!("expected number"),
                }
            })
            .collect();

        for &v in &results {
            assert!(v >= 10.0 && v <= 20.0, "value {v} out of bounds");
        }
    }

    #[test]
    fn test_roll_range_exponential_clamped() {
        let payload = AttributePayload::Range {
            min: 5.0,
            max: 50.0,
            distribution: Some(DistributionConfig::Exponential { rate: 0.5 }),
        };

        let mut rng = StdRng::seed_from_u64(456);
        let results: Vec<f64> = (0..1000)
            .map(|_| {
                match roll_attribute(&payload, &mut rng) {
                    serde_json::Value::Number(n) => n.as_f64().unwrap(),
                    _ => panic!("expected number"),
                }
            })
            .collect();

        for &v in &results {
            assert!(v >= 5.0 && v <= 50.0, "value {v} out of bounds");
        }
    }

    #[test]
    fn test_roll_string_returns_null() {
        let payload = AttributePayload::String {
            min_length: None,
            max_length: None,
        };
        let mut rng = StdRng::seed_from_u64(1);
        let result = roll_attribute(&payload, &mut rng);
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_roll_boolean_returns_true() {
        let payload = AttributePayload::Boolean { value: true };
        let mut rng = StdRng::seed_from_u64(1);
        let result = roll_attribute(&payload, &mut rng);
        assert_eq!(result, serde_json::json!(true));
    }

    #[test]
    fn test_output_ordered_by_attribute_order() {
        let bp = Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::nil(),
            name: "Test".into(),
            archetype: "test".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({
                "z": {"value_type": "single", "value": 1.0},
                "a": {"value_type": "single", "value": 2.0},
                "m": {"value_type": "single", "value": 3.0},
            }),
            attribute_order: vec!["a".into(), "m".into(), "z".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts(),
            updated_at: ts(),
        };

        let cache = make_client_cache(vec![]);
        let rolled = roll_blueprint_attributes(&bp, &cache, 1);

        let keys: Vec<&str> = rolled.keys().map(|s| s.as_str()).collect();
        assert_eq!(keys, vec!["a", "m", "z"]);
    }

    #[test]
    fn test_deterministic_with_same_seed_and_input() {
        let bp = Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::nil(),
            name: "Test".into(),
            archetype: "test".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({
                "damage": {"value_type": "range", "min": 10.0, "max": 20.0, "distribution": {"type": "normal", "stdDev": 2.0}},
            }),
            attribute_order: vec!["damage".into()],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts(),
            updated_at: ts(),
        };

        let cache = make_client_cache(vec![]);
        let result1 = roll_blueprint_attributes(&bp, &cache, 42);
        let result2 = roll_blueprint_attributes(&bp, &cache, 42);

        assert_eq!(result1, result2);
    }

    // --- Proptest for distribution behavior ---

    proptest! {
        #[test]
        fn prop_range_uniform_is_in_bounds(
            seed in any::<u64>(),
            min in -1000.0f64..1000.0,
            max in -1000.0f64..1000.0,
        ) {
            let effective_min = min.min(max);
            let effective_max = max.max(min);
            let payload = AttributePayload::Range {
                min: effective_min,
                max: effective_max,
                distribution: Some(DistributionConfig::Uniform),
            };
            let mut rng = StdRng::seed_from_u64(seed);
            let v = match roll_attribute(&payload, &mut rng) {
                serde_json::Value::Number(n) => n.as_f64().unwrap(),
                other => panic!("expected number, got {other}"),
            };
            prop_assert!(v >= effective_min, "v={v} < min={effective_min}");
            prop_assert!(v <= effective_max, "v={v} > max={effective_max}");
        }

        #[test]
        fn prop_range_uniform_deterministic(
            seed in any::<u64>(),
            min in -1000.0f64..1000.0,
            max in -1000.0f64..1000.0,
        ) {
            let effective_min = min.min(max);
            let effective_max = max.max(min);
            let payload = AttributePayload::Range {
                min: effective_min,
                max: effective_max,
                distribution: Some(DistributionConfig::Uniform),
            };

            let mut rng1 = StdRng::seed_from_u64(seed);
            let v1 = match roll_attribute(&payload, &mut rng1) {
                serde_json::Value::Number(n) => n.as_f64().unwrap(),
                other => panic!("expected number, got {other}"),
            };

            let mut rng2 = StdRng::seed_from_u64(seed);
            let v2 = match roll_attribute(&payload, &mut rng2) {
                serde_json::Value::Number(n) => n.as_f64().unwrap(),
                other => panic!("expected number, got {other}"),
            };

            prop_assert!((v1 - v2).abs() < 1e-10);
        }

        #[test]
        fn prop_range_normal_is_clamped(
            seed in any::<u64>(),
            min in -100.0f64..100.0,
            max in -100.0f64..100.0,
            std_dev in 0.1f64..50.0,
        ) {
            let effective_min = min.min(max);
            let effective_max = max.max(min);
            let payload = AttributePayload::Range {
                min: effective_min,
                max: effective_max,
                distribution: Some(DistributionConfig::Normal { std_dev }),
            };
            let mut rng = StdRng::seed_from_u64(seed);
            let v = match roll_attribute(&payload, &mut rng) {
                serde_json::Value::Number(n) => n.as_f64().unwrap(),
                other => panic!("expected number, got {other}"),
            };
            prop_assert!(v >= effective_min, "v={v} < min={effective_min}");
            prop_assert!(v <= effective_max, "v={v} > max={effective_max}");
        }

        #[test]
        fn prop_range_exponential_is_clamped(
            seed in any::<u64>(),
            min in -100.0f64..100.0,
            max in -100.0f64..100.0,
            rate in 0.1f64..50.0,
        ) {
            let effective_min = min.min(max);
            let effective_max = max.max(min);
            let payload = AttributePayload::Range {
                min: effective_min,
                max: effective_max,
                distribution: Some(DistributionConfig::Exponential { rate }),
            };
            let mut rng = StdRng::seed_from_u64(seed);
            let v = match roll_attribute(&payload, &mut rng) {
                serde_json::Value::Number(n) => n.as_f64().unwrap(),
                other => panic!("expected number, got {other}"),
            };
            prop_assert!(v >= effective_min, "v={v} < min={effective_min}");
            prop_assert!(v <= effective_max, "v={v} > max={effective_max}");
        }

        #[test]
        fn prop_enum_selects_valid_value(
            seed in any::<u64>(),
            values in proptest::collection::vec(".*", 1..20),
        ) {
            let payload = AttributePayload::Enum { values: values.clone() };
            let mut rng = StdRng::seed_from_u64(seed);
            let result = match roll_attribute(&payload, &mut rng) {
                serde_json::Value::String(s) => s,
                other => panic!("expected string, got {other}"),
            };
            prop_assert!(values.contains(&result));
        }
    }
}

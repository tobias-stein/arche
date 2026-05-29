use arche_types::generate::{AffixAttributeEntry, NameParts};
use arche_types::Affix;
use crate::cache::ClientCache;
use crate::generation::rolling::resolve_and_roll_affix_attribute;
use rand::Rng;
use std::collections::BTreeMap;

pub fn compose_name(
    base: &str,
    prefixes: &[String],
    suffixes: &[String],
) -> (String, NameParts) {
    let mut parts: Vec<&str> = Vec::new();

    for p in prefixes {
        parts.push(p.as_str());
    }
    parts.push(base);
    for s in suffixes {
        parts.push(s.as_str());
    }

    let full_name = parts.join(" ");

    let name_parts = NameParts {
        base: base.to_string(),
        prefixes: prefixes.to_vec(),
        suffixes: suffixes.to_vec(),
    };

    (full_name, name_parts)
}

pub fn assemble_affix_attributes(
    affixes: &[(std::sync::Arc<Affix>, i32)],
    client_cache: &ClientCache,
    rng: &mut impl Rng,
) -> Vec<AffixAttributeEntry> {
    let mut entries = Vec::with_capacity(affixes.len());

    for (affix, _sort_order) in affixes {
        let attrs = match resolve_and_roll_affix_attribute(affix, client_cache, rng) {
            Some((name, value)) => {
                let mut map = BTreeMap::new();
                map.insert(name, value);
                serde_json::to_value(map).unwrap_or(serde_json::Value::Null)
            }
            None => serde_json::Value::Null,
        };

        entries.push(AffixAttributeEntry {
            affix_id: affix.id,
            affix_name: affix.name.clone(),
            attributes: attrs,
        });
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;
    use arche_types::*;
    use chrono::{TimeZone, Utc};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn ts() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    }

    fn make_affix_arc(
        id: Uuid,
        name: &str,
        location: AffixLocation,
        attribute: serde_json::Value,
    ) -> Arc<Affix> {
        Arc::new(Affix {
            id,
            client_id: Uuid::nil(),
            name: name.into(),
            location,
            description: None,
            attribute,
            created_at: ts(),
            updated_at: ts(),
        })
    }

    fn make_client_cache(affixes: &[Arc<Affix>]) -> ClientCache {
        ClientCache {
            blueprints: vec![],
            affixes: vec![],
            global_meta_attributes: vec![],
            blueprint_resolved_attributes: HashMap::new(),
            affix_resolved_attributes: Cache::resolve_affix_attributes(affixes, &[]),
        }
    }

    #[test]
    fn test_compose_name_no_affixes() {
        let (name, parts) = compose_name("Longsword", &[], &[]);
        assert_eq!(name, "Longsword");
        assert_eq!(parts.base, "Longsword");
        assert!(parts.prefixes.is_empty());
        assert!(parts.suffixes.is_empty());
    }

    #[test]
    fn test_compose_name_with_one_prefix() {
        let prefixes = vec!["Fire".to_string()];
        let (name, parts) = compose_name("Longsword", &prefixes, &[]);
        assert_eq!(name, "Fire Longsword");
        assert_eq!(parts.base, "Longsword");
        assert_eq!(parts.prefixes, prefixes);
        assert!(parts.suffixes.is_empty());
    }

    #[test]
    fn test_compose_name_with_one_suffix() {
        let suffixes = vec!["of the Bear".to_string()];
        let (name, parts) = compose_name("Longsword", &[], &suffixes);
        assert_eq!(name, "Longsword of the Bear");
        assert_eq!(parts.base, "Longsword");
        assert!(parts.prefixes.is_empty());
        assert_eq!(parts.suffixes, suffixes);
    }

    #[test]
    fn test_compose_name_with_multiple_prefixes_and_suffixes() {
        let prefixes = vec!["Fire".to_string(), "Blessed".to_string()];
        let suffixes = vec!["of the Bear".to_string(), "of Swiftness".to_string()];
        let (name, parts) = compose_name("Longsword", &prefixes, &suffixes);
        assert_eq!(name, "Fire Blessed Longsword of the Bear of Swiftness");
        assert_eq!(parts.base, "Longsword");
        assert_eq!(parts.prefixes, prefixes);
        assert_eq!(parts.suffixes, suffixes);
    }

    #[test]
    fn test_compose_name_empty_prefix_list() {
        let (name, parts) = compose_name("Dagger", &[], &["of Venom".to_string()]);
        assert_eq!(name, "Dagger of Venom");
        assert!(parts.prefixes.is_empty());
    }

    #[test]
    fn test_compose_name_empty_suffix_list() {
        let (name, parts) = compose_name("Staff", &["Arcane".to_string()], &[]);
        assert_eq!(name, "Arcane Staff");
        assert!(parts.suffixes.is_empty());
    }

    #[test]
    fn test_compose_name_empty_both_lists() {
        let (name, parts) = compose_name("Shield", &[], &[]);
        assert_eq!(name, "Shield");
        assert!(parts.prefixes.is_empty());
        assert!(parts.suffixes.is_empty());
    }

    #[test]
    fn test_name_parts_round_trip() {
        let prefixes = vec!["Fire".to_string()];
        let suffixes = vec!["of the Bear".to_string()];
        let (_, parts) = compose_name("Longsword", &prefixes, &suffixes);

        let serialized = serde_json::to_value(&parts).unwrap();
        let deserialized: NameParts = serde_json::from_value(serialized).unwrap();

        assert_eq!(deserialized.base, "Longsword");
        assert_eq!(deserialized.prefixes, vec!["Fire"]);
        assert_eq!(deserialized.suffixes, vec!["of the Bear"]);
    }

    // --- assemble_affix_attributes tests ---

    #[test]
    fn test_assemble_affix_attributes_returns_correct_ids_and_names() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let affix1 = make_affix_arc(
            id1,
            "Fire",
            AffixLocation::Prefix,
            serde_json::json!({"name": "fireDamage", "value_type": "single", "value": 10.0}),
        );
        let affix2 = make_affix_arc(
            id2,
            "of Ice",
            AffixLocation::Suffix,
            serde_json::json!({"name": "iceResist", "value_type": "single", "value": 5.0}),
        );

        let cache = make_client_cache(&[Arc::clone(&affix1), Arc::clone(&affix2)]);
        let mut rng = StdRng::seed_from_u64(42);
        let entries = assemble_affix_attributes(
            &[(Arc::clone(&affix1), 0), (Arc::clone(&affix2), 1)],
            &cache,
            &mut rng,
        );

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].affix_id, id1);
        assert_eq!(entries[0].affix_name, "Fire");
        assert_eq!(entries[1].affix_id, id2);
        assert_eq!(entries[1].affix_name, "of Ice");
    }

    #[test]
    fn test_assemble_affix_attributes_includes_rolled_attribute() {
        let affix = make_affix_arc(
            Uuid::new_v4(),
            "Fire",
            AffixLocation::Prefix,
            serde_json::json!({"name": "fireDamage", "value_type": "single", "value": 10.0}),
        );

        let cache = make_client_cache(&[Arc::clone(&affix)]);
        let mut rng = StdRng::seed_from_u64(42);
        let entries = assemble_affix_attributes(
            &[(Arc::clone(&affix), 0)],
            &cache,
            &mut rng,
        );

        assert_eq!(entries.len(), 1);
        let attrs = entries[0].attributes.as_object().unwrap();
        assert_eq!(attrs.get("fireDamage").unwrap(), &serde_json::json!(10.0));
    }

    #[test]
    fn test_assemble_affix_attributes_empty_list() {
        let cache = make_client_cache(&[]);
        let mut rng = StdRng::seed_from_u64(42);
        let entries = assemble_affix_attributes(&[], &cache, &mut rng);

        assert!(entries.is_empty());
    }

    #[test]
    fn test_assemble_affix_attributes_preserves_sort_order() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let affix1 = make_affix_arc(
            id1, "C", AffixLocation::Prefix,
            serde_json::json!({"name": "cAttr", "value_type": "single", "value": 1.0}),
        );
        let affix2 = make_affix_arc(
            id2, "A", AffixLocation::Prefix,
            serde_json::json!({"name": "aAttr", "value_type": "single", "value": 2.0}),
        );
        let affix3 = make_affix_arc(
            id3, "B", AffixLocation::Suffix,
            serde_json::json!({"name": "bAttr", "value_type": "single", "value": 3.0}),
        );

        let cache = make_client_cache(&[Arc::clone(&affix1), Arc::clone(&affix2), Arc::clone(&affix3)]);
        let mut rng = StdRng::seed_from_u64(42);
        let entries = assemble_affix_attributes(
            &[
                (Arc::clone(&affix1), 0),
                (Arc::clone(&affix2), 1),
                (Arc::clone(&affix3), 2),
            ],
            &cache,
            &mut rng,
        );

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].affix_id, id1);
        assert_eq!(entries[1].affix_id, id2);
        assert_eq!(entries[2].affix_id, id3);
    }

    #[test]
    fn test_assemble_affix_attributes_deterministic() {
        let affix = make_affix_arc(
            Uuid::new_v4(),
            "Fire",
            AffixLocation::Prefix,
            serde_json::json!({"name": "fireDamage", "value_type": "range", "min": 5.0, "max": 15.0}),
        );

        let cache = make_client_cache(&[Arc::clone(&affix)]);
        let mut rng1 = StdRng::seed_from_u64(123);
        let entries1 = assemble_affix_attributes(
            &[(Arc::clone(&affix), 0)],
            &cache,
            &mut rng1,
        );

        let mut rng2 = StdRng::seed_from_u64(123);
        let entries2 = assemble_affix_attributes(
            &[(Arc::clone(&affix), 0)],
            &cache,
            &mut rng2,
        );

        assert_eq!(entries1, entries2);
    }

    #[test]
    fn test_affix_attribute_entry_serialization() {
        let entry = AffixAttributeEntry {
            affix_id: Uuid::nil(),
            affix_name: "Fire".into(),
            attributes: serde_json::json!({"fireDamage": 12.7}),
        };

        let json = serde_json::to_value(&entry).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.get("affixId").unwrap(), &serde_json::json!(Uuid::nil().to_string()));
        assert_eq!(obj.get("affixName").unwrap(), &serde_json::json!("Fire"));
        assert_eq!(obj.get("fireDamage").unwrap(), &serde_json::json!(12.7));
    }
}

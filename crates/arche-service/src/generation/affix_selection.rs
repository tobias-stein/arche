use arche_types::generate::AffixConstraints;
use arche_types::{Affix, AffixLocation, Blueprint, BlueprintAffix};
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub enum PoolKind {
    Prefix,
    Suffix,
}

impl std::fmt::Display for PoolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolKind::Prefix => write!(f, "prefix"),
            PoolKind::Suffix => write!(f, "suffix"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AffixSelectionError {
    #[error("not enough {kind} affixes in pool: need {required}, available={available}")]
    InsufficientPool {
        kind: PoolKind,
        required: i32,
        available: usize,
    },
    #[error("required affix {0} not found in blueprint pool")]
    RequiredAffixNotAvailable(Uuid),
}

pub fn select_affixes(
    blueprint_affix_entries: &[BlueprintAffix],
    all_affixes: &[Arc<Affix>],
    blueprint: &Blueprint,
    constraints: Option<&AffixConstraints>,
    rng: &mut impl Rng,
) -> Result<Vec<(Arc<Affix>, i32)>, AffixSelectionError> {
    let affix_by_id: HashMap<Uuid, &Arc<Affix>> =
        all_affixes.iter().map(|a| (a.id, a)).collect();

    let eff_min_prefixes = constraints
        .and_then(|c| if c.min_prefixes > 0 { Some(c.min_prefixes) } else { None })
        .unwrap_or(blueprint.min_prefixes);
    let eff_max_prefixes = constraints
        .and_then(|c| if c.max_prefixes > 0 { Some(c.max_prefixes) } else { None })
        .unwrap_or(blueprint.max_prefixes);
    let eff_min_suffixes = constraints
        .and_then(|c| if c.min_suffixes > 0 { Some(c.min_suffixes) } else { None })
        .unwrap_or(blueprint.min_suffixes);
    let eff_max_suffixes = constraints
        .and_then(|c| if c.max_suffixes > 0 { Some(c.max_suffixes) } else { None })
        .unwrap_or(blueprint.max_suffixes);

    let require: Vec<Uuid> = constraints
        .map(|c| c.require.clone())
        .unwrap_or_default();
    let block: Vec<Uuid> = constraints
        .map(|c| c.block.clone())
        .unwrap_or_default();

    let prefix_pool: Vec<&BlueprintAffix> = blueprint_affix_entries
        .iter()
        .filter(|ba| ba.location == AffixLocation::Prefix)
        .collect();

    let suffix_pool: Vec<&BlueprintAffix> = blueprint_affix_entries
        .iter()
        .filter(|ba| ba.location == AffixLocation::Suffix)
        .collect();

    for required_id in &require {
        let in_full_pool = blueprint_affix_entries
            .iter()
            .any(|ba| ba.affix_id == *required_id);
        if !in_full_pool {
            return Err(AffixSelectionError::RequiredAffixNotAvailable(
                *required_id,
            ));
        }
    }

    let mut selected: Vec<(Arc<Affix>, i32)> = Vec::new();

    let prefixes = select_by_type(
        &prefix_pool,
        &affix_by_id,
        &require,
        &block,
        eff_min_prefixes,
        eff_max_prefixes,
        PoolKind::Prefix,
        rng,
    )?;
    selected.extend(prefixes);

    let suffixes = select_by_type(
        &suffix_pool,
        &affix_by_id,
        &require,
        &block,
        eff_min_suffixes,
        eff_max_suffixes,
        PoolKind::Suffix,
        rng,
    )?;
    selected.extend(suffixes);

    selected.sort_by_key(|(_, so)| *so);

    Ok(selected)
}

#[allow(clippy::too_many_arguments)]
fn select_by_type(
    pool: &[&BlueprintAffix],
    affix_by_id: &HashMap<Uuid, &Arc<Affix>>,
    require: &[Uuid],
    block: &[Uuid],
    min: i32,
    max: i32,
    kind: PoolKind,
    rng: &mut impl Rng,
) -> Result<Vec<(Arc<Affix>, i32)>, AffixSelectionError> {
    let mut required_entries: Vec<&BlueprintAffix> = Vec::new();
    let mut available_pool: Vec<&BlueprintAffix> = Vec::new();

    for entry in pool {
        if require.contains(&entry.affix_id) {
            required_entries.push(entry);
        } else if !block.contains(&entry.affix_id) {
            available_pool.push(entry);
        }
    }

    let required_count = required_entries.len() as i32;
    let remaining_min = (min - required_count).max(0);
    let remaining_max = (max - required_count).max(0);

    let effective_max = remaining_max.min(available_pool.len() as i32);
    let effective_min = remaining_min.min(effective_max);

    let total_possible = required_count as usize + available_pool.len();
    if (total_possible as i32) < min {
        return Err(AffixSelectionError::InsufficientPool {
            kind,
            required: min,
            available: total_possible,
        });
    }

    let rolled_count = if effective_min >= effective_max {
        effective_min
    } else {
        rng.gen_range(effective_min..=effective_max)
    };

    let mut result: Vec<(Arc<Affix>, i32)> = Vec::new();

    for entry in &required_entries {
        if let Some(affix) = affix_by_id.get(&entry.affix_id) {
            result.push((Arc::clone(*affix), entry.sort_order));
        }
    }

    if rolled_count > 0 {
        let rolled =
            weighted_select_without_replacement(&available_pool, affix_by_id, rolled_count as usize, rng);
        result.extend(rolled);
    }

    Ok(result)
}

fn weighted_select_without_replacement(
    pool: &[&BlueprintAffix],
    affix_by_id: &HashMap<Uuid, &Arc<Affix>>,
    n: usize,
    rng: &mut impl Rng,
) -> Vec<(Arc<Affix>, i32)> {
    if n == 0 || pool.is_empty() {
        return Vec::new();
    }

    let mut indices: Vec<usize> = (0..pool.len()).collect();
    let mut result = Vec::with_capacity(n);

    for _ in 0..n {
        if indices.is_empty() {
            break;
        }
        let weights: Vec<f64> = indices.iter().map(|&i| pool[i].weight).collect();
        let dist = WeightedIndex::new(&weights).expect("weights should be > 0");
        let pick = dist.sample(rng);
        let idx = indices[pick];
        if let Some(affix) = affix_by_id.get(&pool[idx].affix_id) {
            result.push((Arc::clone(*affix), pool[idx].sort_order));
        }
        indices.remove(pick);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use uuid::Uuid;

    fn ts() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    }

    fn make_affix(id: Uuid, name: &str, location: AffixLocation) -> Affix {
        Affix {
            id,
            client_id: Uuid::nil(),
            name: name.into(),
            location,
            description: None,
            attribute: serde_json::json!({}),
            created_at: ts(),
            updated_at: ts(),
        }
    }

    fn make_ba(
        id: Uuid,
        blueprint_id: Uuid,
        affix_id: Uuid,
        weight: f64,
        location: AffixLocation,
        sort_order: i32,
    ) -> BlueprintAffix {
        BlueprintAffix {
            id,
            blueprint_id,
            affix_id,
            weight,
            location,
            sort_order,
        }
    }

    fn make_blueprint(
        min_prefixes: i32,
        max_prefixes: i32,
        min_suffixes: i32,
        max_suffixes: i32,
    ) -> Blueprint {
        Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::nil(),
            name: "Test".into(),
            archetype: "test".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({}),
            attribute_order: vec![],
            min_prefixes,
            max_prefixes,
            min_suffixes,
            max_suffixes,
            created_at: ts(),
            updated_at: ts(),
        }
    }

    // --- Basic selection ---

    #[test]
    fn test_selects_prefixes_from_pool() {
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();

        let affix = make_affix(affix_id, "Fire", AffixLocation::Prefix);
        let ba = make_ba(Uuid::new_v4(), bp_id, affix_id, 1.0, AffixLocation::Prefix, 0);
        let bp = make_blueprint(1, 1, 0, 0);

        let mut rng = StdRng::seed_from_u64(42);
        let all_affixes = vec![Arc::new(affix)];
        let result = select_affixes(&[ba], &all_affixes, &bp, None, &mut rng).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.name, "Fire");
        assert_eq!(result[0].1, 0);
    }

    #[test]
    fn test_selects_suffixes_from_pool() {
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();

        let affix = make_affix(affix_id, "of Ice", AffixLocation::Suffix);
        let ba = make_ba(Uuid::new_v4(), bp_id, affix_id, 1.0, AffixLocation::Suffix, 0);
        let bp = make_blueprint(0, 0, 1, 1);

        let mut rng = StdRng::seed_from_u64(42);
        let all_affixes = vec![Arc::new(affix)];
        let result = select_affixes(&[ba], &all_affixes, &bp, None, &mut rng).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.name, "of Ice");
    }

    #[test]
    fn test_no_affixes_when_min_is_zero() {
        let bp = make_blueprint(0, 0, 0, 0);
        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(&[], &[], &bp, None, &mut rng).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_selects_zero_when_min_is_zero_but_pool_has_entries() {
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();

        let affix = make_affix(affix_id, "Fire", AffixLocation::Prefix);
        let ba = make_ba(Uuid::new_v4(), bp_id, affix_id, 1.0, AffixLocation::Prefix, 0);
        let bp = make_blueprint(0, 0, 0, 0);

        let mut rng = StdRng::seed_from_u64(42);
        let all_affixes = vec![Arc::new(affix)];
        let result = select_affixes(&[ba], &all_affixes, &bp, None, &mut rng).unwrap();
        assert!(result.is_empty());
    }

    // --- Determinism ---

    #[test]
    fn test_deterministic_with_same_seed() {
        let bp_id = Uuid::new_v4();
        let affix1_id = Uuid::new_v4();
        let affix2_id = Uuid::new_v4();

        let affix1 = make_affix(affix1_id, "A", AffixLocation::Prefix);
        let affix2 = make_affix(affix2_id, "B", AffixLocation::Prefix);
        let ba1 = make_ba(Uuid::new_v4(), bp_id, affix1_id, 1.0, AffixLocation::Prefix, 0);
        let ba2 = make_ba(Uuid::new_v4(), bp_id, affix2_id, 2.0, AffixLocation::Prefix, 1);

        let bp = make_blueprint(1, 1, 0, 0);
        let all_affixes = vec![Arc::new(affix1), Arc::new(affix2)];

        let mut rng1 = StdRng::seed_from_u64(123);
        let result1 = select_affixes(&[ba1.clone(), ba2.clone()], &all_affixes, &bp, None, &mut rng1).unwrap();

        let mut rng2 = StdRng::seed_from_u64(123);
        let result2 = select_affixes(&[ba1, ba2], &all_affixes, &bp, None, &mut rng2).unwrap();

        assert_eq!(result1.len(), result2.len());
        assert_eq!(result1[0].0.id, result2[0].0.id);
    }

    // --- Weighted random ---

    #[test]
    fn test_weighted_random_higher_weight_more_likely() {
        let bp_id = Uuid::new_v4();
        let high_id = Uuid::new_v4();
        let low_id = Uuid::new_v4();

        let high_affix = make_affix(high_id, "High", AffixLocation::Prefix);
        let low_affix = make_affix(low_id, "Low", AffixLocation::Prefix);

        let ba_high = make_ba(Uuid::new_v4(), bp_id, high_id, 99.0, AffixLocation::Prefix, 0);
        let ba_low = make_ba(Uuid::new_v4(), bp_id, low_id, 1.0, AffixLocation::Prefix, 1);

        let bp = make_blueprint(1, 1, 0, 0);
        let all_affixes = vec![Arc::new(high_affix), Arc::new(low_affix)];

        let mut high_count = 0;
        let trials = 1000;

        for seed in 0..trials {
            let mut rng = StdRng::seed_from_u64(seed);
            let result =
                select_affixes(&[ba_high.clone(), ba_low.clone()], &all_affixes, &bp, None, &mut rng).unwrap();
            if result[0].0.id == high_id {
                high_count += 1;
            }
        }

        assert!(high_count > 700, "high count = {}/{}", high_count, trials);
    }

    // --- No replacement ---

    #[test]
    fn test_no_duplicate_affixes() {
        let bp_id = Uuid::new_v4();
        let a1_id = Uuid::new_v4();
        let a2_id = Uuid::new_v4();
        let a3_id = Uuid::new_v4();

        let affix1 = make_affix(a1_id, "A", AffixLocation::Prefix);
        let affix2 = make_affix(a2_id, "B", AffixLocation::Prefix);
        let affix3 = make_affix(a3_id, "C", AffixLocation::Prefix);

        let ba1 = make_ba(Uuid::new_v4(), bp_id, a1_id, 1.0, AffixLocation::Prefix, 0);
        let ba2 = make_ba(Uuid::new_v4(), bp_id, a2_id, 1.0, AffixLocation::Prefix, 1);
        let ba3 = make_ba(Uuid::new_v4(), bp_id, a3_id, 1.0, AffixLocation::Prefix, 2);

        let bp = make_blueprint(2, 2, 0, 0);
        let all_affixes = vec![
            Arc::new(affix1),
            Arc::new(affix2),
            Arc::new(affix3),
        ];

        let mut rng = StdRng::seed_from_u64(42);
        let result =
            select_affixes(&[ba1, ba2, ba3], &all_affixes, &bp, None, &mut rng).unwrap();

        assert_eq!(result.len(), 2);

        let mut ids: Vec<Uuid> = result.iter().map(|(a, _)| a.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 2, "no duplicates allowed");
    }

    // --- Required affixes ---

    #[test]
    fn test_required_affix_always_included() {
        let bp_id = Uuid::new_v4();
        let fire_id = Uuid::new_v4();
        let ice_id = Uuid::new_v4();

        let fire = make_affix(fire_id, "Fire", AffixLocation::Prefix);
        let ice = make_affix(ice_id, "Ice", AffixLocation::Prefix);

        let ba_fire = make_ba(Uuid::new_v4(), bp_id, fire_id, 1.0, AffixLocation::Prefix, 0);
        let ba_ice = make_ba(Uuid::new_v4(), bp_id, ice_id, 1.0, AffixLocation::Prefix, 1);

        let bp = make_blueprint(2, 2, 0, 0);
        let all_affixes = vec![Arc::new(fire), Arc::new(ice)];

        let constraints = AffixConstraints {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![fire_id],
            block: vec![],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(
            &[ba_fire, ba_ice],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap();

        let ids: Vec<Uuid> = result.iter().map(|(a, _)| a.id).collect();
        assert!(ids.contains(&fire_id), "required affix must be included");
    }

    #[test]
    fn test_required_affix_counted_against_roll() {
        let bp_id = Uuid::new_v4();
        let fire_id = Uuid::new_v4();
        let ice_id = Uuid::new_v4();

        let fire = make_affix(fire_id, "Fire", AffixLocation::Prefix);
        let ice = make_affix(ice_id, "Ice", AffixLocation::Prefix);

        let ba_fire = make_ba(Uuid::new_v4(), bp_id, fire_id, 1.0, AffixLocation::Prefix, 0);
        let ba_ice = make_ba(Uuid::new_v4(), bp_id, ice_id, 1.0, AffixLocation::Prefix, 1);

        let bp = make_blueprint(1, 1, 0, 0);
        let all_affixes = vec![Arc::new(fire), Arc::new(ice)];

        let constraints = AffixConstraints {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![fire_id],
            block: vec![],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(
            &[ba_fire, ba_ice],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap();

        assert_eq!(result.len(), 1, "required affix counts toward min=1, so only 1 total");
        assert_eq!(result[0].0.id, fire_id);
    }

    #[test]
    fn test_required_minus_required_rolls_additional() {
        let bp_id = Uuid::new_v4();
        let fire_id = Uuid::new_v4();
        let ice_id = Uuid::new_v4();
        let dark_id = Uuid::new_v4();

        let fire = make_affix(fire_id, "Fire", AffixLocation::Prefix);
        let ice = make_affix(ice_id, "Ice", AffixLocation::Prefix);
        let dark = make_affix(dark_id, "Dark", AffixLocation::Prefix);

        let ba_fire = make_ba(Uuid::new_v4(), bp_id, fire_id, 1.0, AffixLocation::Prefix, 0);
        let ba_ice = make_ba(Uuid::new_v4(), bp_id, ice_id, 1.0, AffixLocation::Prefix, 1);
        let ba_dark = make_ba(Uuid::new_v4(), bp_id, dark_id, 1.0, AffixLocation::Prefix, 2);

        let bp = make_blueprint(2, 2, 0, 0);
        let all_affixes = vec![Arc::new(fire), Arc::new(ice), Arc::new(dark)];

        let constraints = AffixConstraints {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![fire_id],
            block: vec![],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(
            &[ba_fire, ba_ice, ba_dark],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap();

        assert_eq!(result.len(), 2);
        let ids: Vec<Uuid> = result.iter().map(|(a, _)| a.id).collect();
        assert!(ids.contains(&fire_id), "required must be there");
    }

    // --- Blocked affixes ---

    #[test]
    fn test_blocked_affix_never_selected() {
        let bp_id = Uuid::new_v4();
        let fire_id = Uuid::new_v4();
        let ice_id = Uuid::new_v4();

        let fire = make_affix(fire_id, "Fire", AffixLocation::Prefix);
        let ice = make_affix(ice_id, "Ice", AffixLocation::Prefix);

        let ba_fire = make_ba(Uuid::new_v4(), bp_id, fire_id, 1.0, AffixLocation::Prefix, 0);
        let ba_ice = make_ba(Uuid::new_v4(), bp_id, ice_id, 99.0, AffixLocation::Prefix, 1);

        let bp = make_blueprint(1, 1, 0, 0);
        let all_affixes = vec![Arc::new(fire), Arc::new(ice)];

        let constraints = AffixConstraints {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![],
            block: vec![ice_id],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(
            &[ba_fire, ba_ice],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id, fire_id);
    }

    #[test]
    fn test_require_takes_precedence_over_block() {
        let bp_id = Uuid::new_v4();
        let fire_id = Uuid::new_v4();

        let fire = make_affix(fire_id, "Fire", AffixLocation::Prefix);
        let ba_fire = make_ba(Uuid::new_v4(), bp_id, fire_id, 1.0, AffixLocation::Prefix, 0);

        let bp = make_blueprint(1, 1, 0, 0);
        let all_affixes = vec![Arc::new(fire)];

        let constraints = AffixConstraints {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![fire_id],
            block: vec![fire_id],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(
            &[ba_fire],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id, fire_id);
    }

    // --- Insufficient pool error ---

    #[test]
    fn test_insufficient_pool_returns_error() {
        let bp_id = Uuid::new_v4();
        let fire_id = Uuid::new_v4();

        let fire = make_affix(fire_id, "Fire", AffixLocation::Prefix);
        let ba_fire = make_ba(Uuid::new_v4(), bp_id, fire_id, 1.0, AffixLocation::Prefix, 0);

        let bp = make_blueprint(3, 5, 0, 0);
        let all_affixes = vec![Arc::new(fire)];

        let mut rng = StdRng::seed_from_u64(42);
        let err = select_affixes(&[ba_fire], &all_affixes, &bp, None, &mut rng).unwrap_err();

        assert!(matches!(
            err,
            AffixSelectionError::InsufficientPool {
                kind: PoolKind::Prefix,
                ..
            }
        ));
    }

    // --- Required affix not available ---

    #[test]
    fn test_required_affix_not_in_pool_is_error() {
        let bp_id = Uuid::new_v4();
        let fire_id = Uuid::new_v4();
        let missing_id = Uuid::new_v4();

        let fire = make_affix(fire_id, "Fire", AffixLocation::Prefix);
        let ba_fire = make_ba(Uuid::new_v4(), bp_id, fire_id, 1.0, AffixLocation::Prefix, 0);

        let bp = make_blueprint(1, 1, 0, 0);
        let all_affixes = vec![Arc::new(fire)];

        let constraints = AffixConstraints {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![missing_id],
            block: vec![],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let err = select_affixes(
            &[ba_fire],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap_err();

        assert!(matches!(err, AffixSelectionError::RequiredAffixNotAvailable(id) if id == missing_id));
    }

    // --- Request overrides blueprint min/max ---

    #[test]
    fn test_request_overrides_blueprint_min_max() {
        let bp_id = Uuid::new_v4();
        let a1_id = Uuid::new_v4();
        let a2_id = Uuid::new_v4();

        let affix1 = make_affix(a1_id, "A", AffixLocation::Prefix);
        let affix2 = make_affix(a2_id, "B", AffixLocation::Prefix);

        let ba1 = make_ba(Uuid::new_v4(), bp_id, a1_id, 1.0, AffixLocation::Prefix, 0);
        let ba2 = make_ba(Uuid::new_v4(), bp_id, a2_id, 1.0, AffixLocation::Prefix, 1);

        let bp = make_blueprint(0, 0, 0, 0);
        let all_affixes = vec![Arc::new(affix1), Arc::new(affix2)];

        let constraints = AffixConstraints {
            min_prefixes: 2,
            max_prefixes: 2,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![],
            block: vec![],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(
            &[ba1, ba2],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap();

        assert_eq!(result.len(), 2);
    }

    // --- Output sorted by sort_order ---

    #[test]
    fn test_output_ordered_by_sort_order() {
        let bp_id = Uuid::new_v4();
        let a1_id = Uuid::new_v4();
        let a2_id = Uuid::new_v4();
        let a3_id = Uuid::new_v4();

        let affix1 = make_affix(a1_id, "C", AffixLocation::Prefix);
        let affix2 = make_affix(a2_id, "A", AffixLocation::Prefix);
        let affix3 = make_affix(a3_id, "B", AffixLocation::Prefix);

        let ba1 = make_ba(Uuid::new_v4(), bp_id, a1_id, 1.0, AffixLocation::Prefix, 20);
        let ba2 = make_ba(Uuid::new_v4(), bp_id, a2_id, 1.0, AffixLocation::Prefix, 10);
        let ba3 = make_ba(Uuid::new_v4(), bp_id, a3_id, 1.0, AffixLocation::Prefix, 30);

        let bp = make_blueprint(3, 3, 0, 0);
        let all_affixes = vec![
            Arc::new(affix1),
            Arc::new(affix2),
            Arc::new(affix3),
        ];

        let mut rng = StdRng::seed_from_u64(42);
        let result =
            select_affixes(&[ba1, ba2, ba3], &all_affixes, &bp, None, &mut rng).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].1, 10);
        assert_eq!(result[1].1, 20);
        assert_eq!(result[2].1, 30);
    }

    // --- Prefixes and suffixes selected independently ---

    #[test]
    fn test_prefixes_and_suffixes_independent() {
        let bp_id = Uuid::new_v4();
        let pre_id = Uuid::new_v4();
        let suf_id = Uuid::new_v4();

        let prefix = make_affix(pre_id, "Fire", AffixLocation::Prefix);
        let suffix = make_affix(suf_id, "of Ice", AffixLocation::Suffix);

        let ba_pre = make_ba(Uuid::new_v4(), bp_id, pre_id, 1.0, AffixLocation::Prefix, 0);
        let ba_suf = make_ba(Uuid::new_v4(), bp_id, suf_id, 1.0, AffixLocation::Suffix, 1);

        let bp = make_blueprint(1, 1, 1, 1);
        let all_affixes = vec![Arc::new(prefix), Arc::new(suffix)];

        let mut rng = StdRng::seed_from_u64(42);
        let result =
            select_affixes(&[ba_pre, ba_suf], &all_affixes, &bp, None, &mut rng).unwrap();

        assert_eq!(result.len(), 2);
        let has_prefix = result.iter().any(|(a, _)| a.location == AffixLocation::Prefix);
        let has_suffix = result.iter().any(|(a, _)| a.location == AffixLocation::Suffix);
        assert!(has_prefix);
        assert!(has_suffix);
    }

    // --- Empty constraints treated as None-like ---

    #[test]
    fn test_empty_constraints_uses_blueprint_defaults() {
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();

        let affix = make_affix(affix_id, "Fire", AffixLocation::Prefix);
        let ba = make_ba(Uuid::new_v4(), bp_id, affix_id, 1.0, AffixLocation::Prefix, 0);
        let bp = make_blueprint(1, 1, 0, 0);

        let all_affixes = vec![Arc::new(affix)];

        let constraints = AffixConstraints {
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            require: vec![],
            block: vec![],
        };

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(
            &[ba],
            &all_affixes,
            &bp,
            Some(&constraints),
            &mut rng,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
    }

    // --- Uniform count distribution ---

    #[test]
    fn test_count_distribution_is_uniform() {
        let bp_id = Uuid::new_v4();
        let ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

        let affixes: Vec<Arc<Affix>> = ids
            .iter()
            .map(|&id| Arc::new(make_affix(id, "X", AffixLocation::Prefix)))
            .collect();

        let bas: Vec<BlueprintAffix> = ids
            .iter()
            .enumerate()
            .map(|(i, &id)| {
                make_ba(
                    Uuid::new_v4(),
                    bp_id,
                    id,
                    1.0,
                    AffixLocation::Prefix,
                    i as i32,
                )
            })
            .collect();

        let bp = make_blueprint(2, 5, 0, 0);

        let mut counts = [0usize; 6];

        let trials = 1000usize;
        for seed in 0..trials as u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let result = select_affixes(&bas, &affixes, &bp, None, &mut rng).unwrap();
            let n = result.len();
            assert!(n >= 2 && n <= 5, "count out of range: {}", n);
            counts[n] += 1;
        }

        let expected = trials / 4;
        let tolerance = 80;
        for n in 2..=5 {
            assert!(
                counts[n] >= expected.saturating_sub(tolerance)
                    && counts[n] <= expected.saturating_add(tolerance),
                "count {} appeared {} times, expected ~{} (±{})",
                n,
                counts[n],
                expected,
                tolerance
            );
        }
        assert_eq!(counts[0], 0);
        assert_eq!(counts[1], 0);
    }

    // --- Random without replacement exhausts pool correctly ---

    #[test]
    fn test_random_without_replacement_exhaust() {
        let bp_id = Uuid::new_v4();
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();

        let affixes: Vec<Arc<Affix>> = ids
            .iter()
            .map(|&id| Arc::new(make_affix(id, "X", AffixLocation::Prefix)))
            .collect();

        let bas: Vec<BlueprintAffix> = ids
            .iter()
            .enumerate()
            .map(|(i, &id)| {
                make_ba(
                    Uuid::new_v4(),
                    bp_id,
                    id,
                    1.0,
                    AffixLocation::Prefix,
                    i as i32,
                )
            })
            .collect();

        let bp = make_blueprint(3, 3, 0, 0);

        let mut rng = StdRng::seed_from_u64(42);
        let result = select_affixes(&bas, &affixes, &bp, None, &mut rng).unwrap();

        assert_eq!(result.len(), 3);
        let mut result_ids: Vec<Uuid> = result.iter().map(|(a, _)| a.id).collect();
        result_ids.sort();
        let mut expected_ids: Vec<Uuid> = ids.clone();
        expected_ids.sort();
        assert_eq!(result_ids, expected_ids);
    }
}

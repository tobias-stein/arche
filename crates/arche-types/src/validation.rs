use crate::{Blueprint, ValueType};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

pub fn validate_attribute_payload(value_type: &ValueType, payload: &Value) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let obj = match payload.as_object() {
        Some(obj) => obj,
        None => return errors,
    };

    if obj.contains_key("$ref_id") {
        if obj.len() > 1 {
            errors.push(ValidationError::new(
                "payload",
                "attribute cannot have both $ref_id and inline fields",
            ));
        }
        return errors;
    }

    match value_type {
        ValueType::Range => {
            if let (Some(min), Some(max)) = (
                obj.get("min").and_then(|v| v.as_f64()),
                obj.get("max").and_then(|v| v.as_f64()),
            ) {
                if min > max {
                    errors.push(ValidationError::new(
                        "payload",
                        "range.min must be ≤ range.max",
                    ));
                }
            }

            if let Some(dist) = obj.get("distribution").and_then(|d| d.as_object()) {
                if let Some(std_dev) = dist.get("std_dev").and_then(|v| v.as_f64()) {
                    if std_dev <= 0.0 {
                        errors.push(ValidationError::new(
                            "payload.distribution.std_dev",
                            "range.distribution.std_dev must be > 0",
                        ));
                    }
                }
                if let Some(rate) = dist.get("rate").and_then(|v| v.as_f64()) {
                    if rate <= 0.0 {
                        errors.push(ValidationError::new(
                            "payload.distribution.rate",
                            "range.distribution.rate must be > 0",
                        ));
                    }
                }
            }
        }
        ValueType::Enum => {
            if let Some(values) = obj.get("values").and_then(|v| v.as_array()) {
                if values.is_empty() {
                    errors.push(ValidationError::new(
                        "payload.values",
                        "enum.values must have at least 1 entry",
                    ));
                }
            }
        }
        ValueType::Single => {
            if let Some(value) = obj.get("value") {
                if value.is_null() {
                    errors.push(ValidationError::new(
                        "payload.value",
                        "single.value must not be null",
                    ));
                }
            }
        }
        ValueType::String | ValueType::Boolean => {}
    }

    errors
}

pub fn validate_attribute_order(
    attribute_order: &[String],
    attributes: &Value,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let attr_keys: Vec<&String> = match attributes.as_object() {
        Some(obj) => obj.keys().collect(),
        None => return errors,
    };

    let mut seen = HashSet::new();
    for key in attribute_order {
        if !seen.insert(key) {
            errors.push(ValidationError::new(
                format!("attribute_order[{key}]"),
                format!("duplicate key in attribute_order: {key}"),
            ));
        }
    }

    let order_set: HashSet<&String> = attribute_order.iter().collect();
    for key in &attr_keys {
        if !order_set.contains(key) {
            errors.push(ValidationError::new(
                "attribute_order",
                format!("missing key in attribute_order: {key}"),
            ));
        }
    }

    let attr_set: HashSet<&String> = attr_keys.iter().copied().collect();
    for key in attribute_order {
        if !attr_set.contains(key) {
            errors.push(ValidationError::new(
                "attribute_order",
                format!("extra key in attribute_order: {key}"),
            ));
        }
    }

    errors
}

fn check_affix_non_negative(
    errors: &mut Vec<ValidationError>,
    json_path: &str,
    field: &str,
    value: i32,
) {
    if value < 0 {
        errors.push(ValidationError::new(
            json_path,
            format!("{field} must be ≥ 0"),
        ));
    }
}

pub fn validate_blueprint(blueprint: &Blueprint) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if blueprint.weight <= 0.0 {
        errors.push(ValidationError::new(
            "weight",
            "blueprint weight must be > 0",
        ));
    }

    check_affix_non_negative(
        &mut errors,
        "min_prefixes",
        "min_prefixes",
        blueprint.min_prefixes,
    );
    check_affix_non_negative(
        &mut errors,
        "max_prefixes",
        "max_prefixes",
        blueprint.max_prefixes,
    );
    check_affix_non_negative(
        &mut errors,
        "min_suffixes",
        "min_suffixes",
        blueprint.min_suffixes,
    );
    check_affix_non_negative(
        &mut errors,
        "max_suffixes",
        "max_suffixes",
        blueprint.max_suffixes,
    );
    if blueprint.min_prefixes > blueprint.max_prefixes {
        errors.push(ValidationError::new(
            "affixCounts",
            "min_prefixes must be ≤ max_prefixes",
        ));
    }
    if blueprint.min_suffixes > blueprint.max_suffixes {
        errors.push(ValidationError::new(
            "affixCounts",
            "min_suffixes must be ≤ max_suffixes",
        ));
    }

    if let Some(attrs) = blueprint.attributes.as_object() {
        for (key, attr_value) in attrs {
            if let Some(attr_obj) = attr_value.as_object() {
                if attr_obj.contains_key("$ref_id") {
                    if attr_obj.len() > 1 {
                        errors.push(ValidationError::new(
                            format!("attributes.{key}"),
                            "attribute cannot have both $ref_id and inline fields",
                        ));
                    }
                } else if let Some(value_type_str) =
                    attr_obj.get("value_type").and_then(|v| v.as_str())
                {
                    let value_type = match value_type_str {
                        "single" => ValueType::Single,
                        "enum" => ValueType::Enum,
                        "range" => ValueType::Range,
                        "string" => ValueType::String,
                        "boolean" => ValueType::Boolean,
                        _ => continue,
                    };

                    let mut payload_map = serde_json::Map::new();
                    for (k, v) in attr_obj.iter() {
                        if k != "value_type" {
                            payload_map.insert(k.clone(), v.clone());
                        }
                    }
                    let payload = Value::Object(payload_map);

                    for e in validate_attribute_payload(&value_type, &payload) {
                        errors.push(ValidationError::new(
                            format!("attributes.{}.{}", key, e.path),
                            e.message,
                        ));
                    }
                }
            }
        }

        errors.extend(validate_attribute_order(
            &blueprint.attribute_order,
            &blueprint.attributes,
        ));
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_range_payload() {
        let payload =
            json!({"min": 1.0, "max": 10.0, "distribution": {"std_dev": 2.0, "rate": 1.0}});
        let errors = validate_attribute_payload(&ValueType::Range, &payload);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_range_min_greater_than_max() {
        let payload = json!({"min": 10.0, "max": 1.0});
        let errors = validate_attribute_payload(&ValueType::Range, &payload);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].path, "payload");
    }

    #[test]
    fn test_range_std_dev_zero() {
        let payload =
            json!({"min": 1.0, "max": 10.0, "distribution": {"std_dev": 0.0, "rate": 1.0}});
        let errors = validate_attribute_payload(&ValueType::Range, &payload);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("std_dev"));
    }

    #[test]
    fn test_range_std_dev_negative() {
        let payload =
            json!({"min": 1.0, "max": 10.0, "distribution": {"std_dev": -1.0, "rate": 1.0}});
        let errors = validate_attribute_payload(&ValueType::Range, &payload);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("std_dev"));
    }

    #[test]
    fn test_range_rate_zero() {
        let payload =
            json!({"min": 1.0, "max": 10.0, "distribution": {"std_dev": 2.0, "rate": 0.0}});
        let errors = validate_attribute_payload(&ValueType::Range, &payload);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("rate"));
    }

    #[test]
    fn test_range_rate_negative() {
        let payload =
            json!({"min": 1.0, "max": 10.0, "distribution": {"std_dev": 2.0, "rate": -0.5}});
        let errors = validate_attribute_payload(&ValueType::Range, &payload);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("rate"));
    }

    #[test]
    fn test_range_multiple_errors() {
        let payload =
            json!({"min": 10.0, "max": 1.0, "distribution": {"std_dev": 0.0, "rate": 0.0}});
        let errors = validate_attribute_payload(&ValueType::Range, &payload);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_valid_enum_payload() {
        let payload = json!({"values": ["common", "rare"]});
        let errors = validate_attribute_payload(&ValueType::Enum, &payload);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_enum_empty_values() {
        let payload = json!({"values": []});
        let errors = validate_attribute_payload(&ValueType::Enum, &payload);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("at least 1 entry"));
    }

    #[test]
    fn test_valid_single_payload() {
        let payload = json!({"value": "hello"});
        let errors = validate_attribute_payload(&ValueType::Single, &payload);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_single_value_null() {
        let payload = json!({"value": null});
        let errors = validate_attribute_payload(&ValueType::Single, &payload);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("must not be null"));
    }

    #[test]
    fn test_valid_string_payload() {
        let payload = json!({});
        let errors = validate_attribute_payload(&ValueType::String, &payload);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_boolean_payload() {
        let payload = json!({});
        let errors = validate_attribute_payload(&ValueType::Boolean, &payload);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_ref_id_alone_valid() {
        let payload = json!({"$ref_id": "550e8400-e29b-41d4-a716-446655440000"});
        let errors = validate_attribute_payload(&ValueType::String, &payload);
        assert!(errors.is_empty(), "expected no errors for $ref_id alone");
    }

    #[test]
    fn test_ref_id_with_inline_fields() {
        let payload = json!({"$ref_id": "some-uuid", "value_type": "string", "min": 1});
        let errors = validate_attribute_payload(&ValueType::String, &payload);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("cannot have both"));
    }

    #[test]
    fn test_non_object_payload() {
        let errors = validate_attribute_payload(&ValueType::String, &json!("not_an_object"));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_attribute_order() {
        let attributes = json!({"a": {"value_type": "string"}, "b": {"value_type": "string"}});
        let order = vec!["a".into(), "b".into()];
        let errors = validate_attribute_order(&order, &attributes);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_missing_key_in_order() {
        let attributes = json!({"a": {}, "b": {}, "c": {}});
        let order = vec!["a".into(), "b".into()];
        let errors = validate_attribute_order(&order, &attributes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("missing"));
        assert!(errors[0].message.contains("c"));
    }

    #[test]
    fn test_extra_key_in_order() {
        let attributes = json!({"a": {}});
        let order = vec!["a".into(), "b".into()];
        let errors = validate_attribute_order(&order, &attributes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("extra"));
        assert!(errors[0].message.contains("b"));
    }

    #[test]
    fn test_duplicate_key_in_order() {
        let attributes = json!({"a": {}, "b": {}});
        let order = vec!["a".into(), "b".into(), "a".into()];
        let errors = validate_attribute_order(&order, &attributes);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("duplicate"));
    }

    #[test]
    fn test_multiple_order_errors() {
        let attributes = json!({"a": {}, "b": {}, "c": {}});
        let order = vec!["a".into(), "a".into(), "d".into()];
        let errors = validate_attribute_order(&order, &attributes);
        assert_eq!(errors.len(), 4);
    }

    #[test]
    fn test_empty_attributes_no_errors() {
        let attributes = json!({});
        let order: Vec<String> = vec![];
        let errors = validate_attribute_order(&order, &attributes);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_non_object_attributes_no_errors() {
        let attributes = json!("not_an_object");
        let order: Vec<String> = vec![];
        let errors = validate_attribute_order(&order, &attributes);
        assert!(errors.is_empty());
    }

    fn valid_blueprint() -> Blueprint {
        Blueprint {
            id: uuid::Uuid::new_v4(),
            client_id: uuid::Uuid::new_v4(),
            name: "Test".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: None,
            attributes: json!({"material": {"value_type": "string"}}),
            attribute_order: vec!["material".into()],
            min_prefixes: 0,
            max_prefixes: 2,
            min_suffixes: 0,
            max_suffixes: 2,
            created_at: chrono::DateTime::from_timestamp_millis(0).unwrap(),
            updated_at: chrono::DateTime::from_timestamp_millis(0).unwrap(),
        }
    }

    #[test]
    fn test_valid_blueprint() {
        let bp = valid_blueprint();
        let errors = validate_blueprint(&bp);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_blueprint_weight_zero() {
        let mut bp = valid_blueprint();
        bp.weight = 0.0;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].path.contains("weight"));
    }

    #[test]
    fn test_blueprint_weight_negative() {
        let mut bp = valid_blueprint();
        bp.weight = -0.5;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].path.contains("weight"));
    }

    #[test]
    fn test_blueprint_min_prefixes_greater_than_max() {
        let mut bp = valid_blueprint();
        bp.min_prefixes = 5;
        bp.max_prefixes = 2;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("min_prefixes"));
    }

    #[test]
    fn test_blueprint_min_suffixes_greater_than_max() {
        let mut bp = valid_blueprint();
        bp.min_suffixes = 5;
        bp.max_suffixes = 2;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("min_suffixes"));
    }

    #[test]
    fn test_blueprint_negative_min_prefixes() {
        let mut bp = valid_blueprint();
        bp.min_prefixes = -1;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("min_prefixes"));
    }

    #[test]
    fn test_blueprint_negative_max_prefixes() {
        let mut bp = valid_blueprint();
        bp.max_prefixes = -1;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].message.contains("max_prefixes"));
    }

    #[test]
    fn test_blueprint_negative_max_suffixes() {
        let mut bp = valid_blueprint();
        bp.max_suffixes = -1;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].message.contains("max_suffixes"));
    }

    #[test]
    fn test_blueprint_negative_min_suffixes() {
        let mut bp = valid_blueprint();
        bp.min_suffixes = -1;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("min_suffixes"));
    }

    #[test]
    fn test_blueprint_invalid_attribute_payload() {
        let mut bp = valid_blueprint();
        bp.attributes = json!({"damage": {"value_type": "range", "min": 100, "max": 1}});
        bp.attribute_order = vec!["damage".into()];
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].path.contains("damage"));
        assert!(errors[0].message.contains("min"));
    }

    #[test]
    fn test_blueprint_invalid_attribute_order() {
        let mut bp = valid_blueprint();
        bp.attribute_order = vec![];
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("missing"));
    }

    #[test]
    fn test_blueprint_ref_id_valid() {
        let mut bp = valid_blueprint();
        bp.attributes = json!({"rarity": {"$ref_id": "550e8400-e29b-41d4-a716-446655440000"}});
        bp.attribute_order = vec!["rarity".into()];
        let errors = validate_blueprint(&bp);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_blueprint_ref_id_with_inline_fields() {
        let mut bp = valid_blueprint();
        bp.attributes =
            json!({"rarity": {"$ref_id": "some-uuid", "value_type": "string", "min": 1}});
        bp.attribute_order = vec!["rarity".into()];
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("cannot have both"));
    }

    #[test]
    fn test_blueprint_multiple_errors() {
        let mut bp = valid_blueprint();
        bp.weight = 0.0;
        bp.min_prefixes = 3;
        bp.max_prefixes = 1;
        bp.attributes = json!({"a": {"value_type": "range", "min": 10, "max": 1}});
        bp.attribute_order = vec!["a".into(), "b".into()];
        let errors = validate_blueprint(&bp);
        assert_eq!(errors.len(), 4);
    }

    #[test]
    fn test_blueprint_snake_case_paths() {
        let mut bp = valid_blueprint();
        bp.min_prefixes = -1;
        let errors = validate_blueprint(&bp);
        assert_eq!(errors[0].path, "min_prefixes");
    }
}

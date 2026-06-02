use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod attribute;
pub mod batch;
pub mod common;
pub mod crud;
pub mod export_import;
pub mod generate;
pub mod validation;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    Single,
    Enum,
    Range,
    String,
    Boolean,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AffixLocation {
    Prefix,
    Suffix,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Created,
    Updated,
    Deleted,
    ForceDeleted,
    Adjusted,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Generate,
    Admin,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Blueprint {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub archetype: String,
    pub weight: f64,
    pub description: Option<String>,
    pub attributes: serde_json::Value,
    pub attribute_order: Vec<String>,
    pub min_prefixes: i32,
    pub max_prefixes: i32,
    pub min_suffixes: i32,
    pub max_suffixes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Affix {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub location: AffixLocation,
    pub description: Option<String>,
    pub attribute: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GlobalMetaAttribute {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub value_type: ValueType,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ApiKey {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub name: String,
    pub key_hash: String,
    pub permissions: Vec<Permission>,
    pub is_super: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor_key_id: Uuid,
    pub actor_key_name: String,
    pub client_id: Option<Uuid>,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub action: AuditAction,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BlueprintAffix {
    pub id: Uuid,
    pub blueprint_id: Uuid,
    pub affix_id: Uuid,
    pub weight: f64,
    pub location: AffixLocation,
    pub sort_order: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use attribute::*;
    use batch::*;
    use common::*;
    use crud::*;
    use export_import::*;
    use generate::*;
    use serde::de::DeserializeOwned;
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};

    #[test]    fn test_distribution_config_round_trip() {
        let cases = vec![
            (DistributionConfig::Uniform, json!({"type": "uniform"})),
            (
                DistributionConfig::Normal { std_dev: 2.5 },
                json!({"type": "normal", "std_dev": 2.5}),
            ),
            (
                DistributionConfig::Exponential { rate: 1.5 },
                json!({"type": "exponential", "rate": 1.5}),
            ),
        ];
        for (variant, expected) in cases {
            let value = serde_json::to_value(&variant).unwrap();
            assert_eq!(value, expected);
            let deserialized: DistributionConfig = serde_json::from_value(expected).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_attribute_payload_round_trip() {
        let cases = vec![
            (
                AttributePayload::Single {
                    value: 10.0,
                    distribution: None,
                },
                json!({"value_type": "single", "value": 10.0}),
            ),
            (
                AttributePayload::Enum {
                    values: vec!["a".into(), "b".into()],
                },
                json!({"value_type": "enum", "values": ["a", "b"]}),
            ),
            (
                AttributePayload::Range {
                    min: 1.0,
                    max: 10.0,
                    distribution: Some(DistributionConfig::Uniform),
                },
                json!({"value_type": "range", "min": 1.0, "max": 10.0, "distribution": {"type": "uniform"}}),
            ),
            (
                AttributePayload::String {
                    min_length: Some(1),
                    max_length: None,
                },
                json!({"value_type": "string", "min_length": 1}),
            ),
            (
                AttributePayload::Boolean { value: true },
                json!({"value_type": "boolean", "value": true}),
            ),
        ];
        for (variant, expected) in cases {
            let value = serde_json::to_value(&variant).unwrap();
            assert_eq!(value, expected);
            let deserialized: AttributePayload = serde_json::from_value(expected).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_blueprint_attribute_inline() {
        let attr = BlueprintAttribute::Inline(InlineAttributeDef {
            description: Some("Physical damage".into()),
            payload: AttributePayload::Range {
                min: 10.0,
                max: 23.0,
                distribution: Some(DistributionConfig::Uniform),
            },
        });
        let value = serde_json::to_value(&attr).unwrap();
        assert_eq!(
            value,
            json!({"description": "Physical damage", "value_type": "range", "min": 10.0, "max": 23.0, "distribution": {"type": "uniform"}})
        );
        let deserialized: BlueprintAttribute = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, attr);
    }

    #[test]
    fn test_blueprint_attribute_ref() {
        let id = Uuid::new_v4();
        let attr = BlueprintAttribute::Ref(BlueprintRefAttribute { ref_id: id });
        let value = serde_json::to_value(&attr).unwrap();
        assert_eq!(value, json!({"$ref_id": id.to_string()}));
        let deserialized: BlueprintAttribute = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, attr);
    }

    #[test]
    fn test_affix_attribute_inline() {
        let attr = AffixAttribute::Inline(AffixInlineAttributeDef {
            name: "fire_damage".into(),
            description: Some("Fire damage".into()),
            payload: AttributePayload::Range {
                min: 5.0,
                max: 15.0,
                distribution: None,
            },
        });
        let value = serde_json::to_value(&attr).unwrap();
        assert_eq!(
            value,
            json!({"name": "fire_damage", "description": "Fire damage", "value_type": "range", "min": 5.0, "max": 15.0})
        );
        let deserialized: AffixAttribute = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, attr);
    }

    #[test]
    fn test_affix_attribute_ref() {
        let id = Uuid::new_v4();
        let attr = AffixAttribute::Ref { ref_id: id };
        let value = serde_json::to_value(&attr).unwrap();
        assert_eq!(value, json!({"$ref_id": id.to_string()}));
        let deserialized: AffixAttribute = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, attr);
    }

    #[test]
    fn test_blueprint_affix_config_round_trip() {
        let config = BlueprintAffixConfig {
            min_prefixes: 0,
            max_prefixes: 2,
            min_suffixes: 1,
            max_suffixes: 1,
            prefixes: vec![AffixPoolEntry {
                affix_id: Uuid::new_v4(),
                weight: 1.0,
            }],
            suffixes: vec![],
        };
        let value = serde_json::to_value(&config).unwrap();
        let deserialized: BlueprintAffixConfig = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_constraint_config_round_trip() {
        let cc = ConstraintConfig {
            gte: Some(15.0),
            lte: Some(40.0),
            r#in: None,
            contains: None,
            eq: None,
        };
        let value = serde_json::to_value(&cc).unwrap();
        assert_eq!(value, json!({"gte": 15.0, "lte": 40.0}));
        let deserialized: ConstraintConfig = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, cc);
    }

    #[test]
    fn test_constraint_config_with_in() {
        let cc = ConstraintConfig {
            gte: None,
            lte: None,
            r#in: Some(vec!["rare".into(), "legendary".into()]),
            contains: None,
            eq: None,
        };
        let value = serde_json::to_value(&cc).unwrap();
        assert_eq!(value, json!({"in": ["rare", "legendary"]}));
        let deserialized: ConstraintConfig = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, cc);
    }

    #[test]
    fn test_constraint_value_config() {
        let cv = ConstraintValue::Config(ConstraintConfig {
            gte: Some(15.0),
            lte: None,
            r#in: None,
            contains: None,
            eq: None,
        });
        let value = serde_json::to_value(&cv).unwrap();
        assert_eq!(value, json!({"gte": 15.0}));
        let deserialized: ConstraintValue = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, cv);
    }

    #[test]
    fn test_constraint_value_bare() {
        let cv = ConstraintValue::Bare(json!(true));
        let value = serde_json::to_value(&cv).unwrap();
        assert_eq!(value, json!(true));
        let deserialized: ConstraintValue = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, cv);
    }

    #[test]
    fn test_generate_request_minimal() {
        let req = GenerateRequest {
            archetype: None,
            seed: None,
            constraints: None,
            affixes: None,
            client_id: None,
        };
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(value, json!({}));
        let deserialized: GenerateRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_generate_request_full() {
        let mut constraints = HashMap::new();
        constraints.insert(
            "damage".into(),
            ConstraintValue::Config(ConstraintConfig {
                gte: Some(15.0),
                lte: Some(40.0),
                r#in: None,
                contains: None,
                eq: None,
            }),
        );
        constraints.insert("magical_defense".into(), ConstraintValue::Bare(json!(true)));

        let req = GenerateRequest {
            archetype: Some("sword".into()),
            seed: Some(172839465),
            constraints: Some(constraints),
            affixes: Some(AffixConstraints {
                min_prefixes: 1,
                max_prefixes: 2,
                min_suffixes: 0,
                max_suffixes: 1,
                require: vec![Uuid::new_v4(), Uuid::new_v4()],
                block: vec![Uuid::new_v4()],
            }),
            client_id: None,
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: GenerateRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_generate_request_snake_case() {
        let req = GenerateRequest {
            archetype: Some("sword".into()),
            seed: None,
            constraints: None,
            affixes: None,
            client_id: None,
        };
        let obj = serde_json::to_value(&req)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("archetype"));
    }

    #[test]
    fn test_generate_response_round_trip() {
        let resp = GenerateResponse {
            seed: 172839465,
            name: "Fire Longsword of the Bear".into(),
            name_parts: NameParts {
                base: "Longsword".into(),
                prefixes: vec!["Fire".into()],
                suffixes: vec!["of the Bear".into()],
            },
            blueprint_id: Uuid::new_v4(),
            archetype: "sword".into(),
            blueprint_attributes: {
                let mut m = BTreeMap::new();
                m.insert("damage".into(), json!(27.3));
                m
            },
            affix_attributes: vec![AffixAttributeEntry {
                affix_id: Uuid::new_v4(),
                affix_name: "Fire".into(),
                attributes: json!({"fire_damage": 12.7}),
            }],
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: GenerateResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_batch_generate_request_round_trip() {
        let req = BatchGenerateRequest {
            requests: vec![
                GenerateRequest {
                    archetype: Some("sword".into()),
                    seed: Some(42),
                    constraints: None,
                    affixes: None,
                    client_id: None,
                },
            ],
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: BatchGenerateRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_batch_generate_response_round_trip() {
        let resp = BatchGenerateResponse {
            results: vec![
                BatchGenerateResultItem {
                    result: Some(GenerateResponse {
                        seed: 42,
                        name: "Fire Longsword".into(),
                        name_parts: NameParts {
                            base: "Longsword".into(),
                            prefixes: vec!["Fire".into()],
                            suffixes: vec![],
                        },
                        blueprint_id: Uuid::new_v4(),
                        archetype: "sword".into(),
                        blueprint_attributes: BTreeMap::new(),
                        affix_attributes: vec![],
                    }),
                    error: None,
                },
                BatchGenerateResultItem {
                    result: None,
                    error: Some("No matching blueprint".into()),
                },
            ],
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: BatchGenerateResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_batch_generate_response_snake_case() {
        let resp = BatchGenerateResponse {
            results: vec![],
        };
        let obj = serde_json::to_value(&resp)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("results"));
    }

    #[test]
    fn test_generate_response_snake_case() {
        let resp = GenerateResponse {
            seed: 0,
            name: "".into(),
            name_parts: NameParts {
                base: "".into(),
                prefixes: vec![],
                suffixes: vec![],
            },
            blueprint_id: Uuid::nil(),
            archetype: "".into(),
            blueprint_attributes: BTreeMap::new(),
            affix_attributes: vec![],
        };
        let obj = serde_json::to_value(&resp)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("name_parts"));
        assert!(obj.contains_key("blueprint_id"));
        assert!(obj.contains_key("blueprint_attributes"));
        assert!(obj.contains_key("affix_attributes"));
    }

    #[test]
    fn test_create_blueprint_request_round_trip() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "damage".into(),
            BlueprintAttribute::Inline(InlineAttributeDef {
                description: Some("Physical damage".into()),
                payload: AttributePayload::Range {
                    min: 10.0,
                    max: 23.0,
                    distribution: None,
                },
            }),
        );
        let req = CreateBlueprintRequest {
            name: "Longsword".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: Some("A sturdy sword".into()),
            attributes,
            attribute_order: vec!["damage".into()],
            affixes: BlueprintAffixConfig {
                min_prefixes: 0,
                max_prefixes: 1,
                min_suffixes: 0,
                max_suffixes: 1,
                prefixes: vec![],
                suffixes: vec![],
            },
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: CreateBlueprintRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_create_blueprint_request_snake_case() {
        let req = CreateBlueprintRequest {
            name: "x".into(),
            archetype: "x".into(),
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
        let obj = serde_json::to_value(&req)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("attribute_order"));
    }

    #[test]
    fn test_blueprint_response_round_trip() {
        let resp = BlueprintResponse {
            id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            name: "Longsword".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: Some("A sturdy sword".into()),
            attributes: json!({"damage": {"min": 10, "max": 23}}),
            attribute_order: vec!["damage".into()],
            min_prefixes: 0,
            max_prefixes: 2,
            min_suffixes: 1,
            max_suffixes: 1,
            affixes: BlueprintAffixConfig {
                min_prefixes: 0,
                max_prefixes: 2,
                min_suffixes: 1,
                max_suffixes: 1,
                prefixes: vec![AffixPoolEntry {
                    affix_id: Uuid::new_v4(),
                    weight: 1.0,
                }],
                suffixes: vec![],
            },
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: BlueprintResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_blueprint_response_snake_case() {
        let resp = BlueprintResponse {
            id: Uuid::nil(),
            client_id: Uuid::nil(),
            name: "x".into(),
            archetype: "x".into(),
            weight: 1.0,
            description: None,
            attributes: json!({}),
            attribute_order: vec![],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            affixes: BlueprintAffixConfig {
                min_prefixes: 0,
                max_prefixes: 0,
                min_suffixes: 0,
                max_suffixes: 0,
                prefixes: vec![],
                suffixes: vec![],
            },
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let obj = serde_json::to_value(&resp)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("client_id"));
        assert!(obj.contains_key("attribute_order"));
        assert!(obj.contains_key("min_prefixes"));
        assert!(obj.contains_key("affixes"));
    }

    #[test]
    fn test_create_affix_request_round_trip() {
        let req = CreateAffixRequest {
            name: "Fire".into(),
            location: AffixLocation::Prefix,
            description: None,
            attribute: AffixAttribute::Inline(AffixInlineAttributeDef {
                name: "fire_damage".into(),
                description: None,
                payload: AttributePayload::Range {
                    min: 5.0,
                    max: 15.0,
                    distribution: None,
                },
            }),
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: CreateAffixRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_create_affix_request_type_field() {
        let req = CreateAffixRequest {
            name: "Fire".into(),
            location: AffixLocation::Prefix,
            description: None,
            attribute: AffixAttribute::Ref {
                ref_id: Uuid::new_v4(),
            },
        };
        let obj = serde_json::to_value(&req).unwrap();
        let obj_map = obj.as_object().unwrap();
        assert_eq!(obj_map.get("type").unwrap(), "prefix");
        let deserialized: CreateAffixRequest = serde_json::from_value(obj).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_create_global_meta_attribute_request_round_trip() {
        let req = CreateGlobalMetaAttributeRequest {
            name: "rarity".into(),
            description: Some("Quality tier".into()),
            payload: AttributePayload::Enum {
                values: vec!["common".into(), "rare".into(), "legendary".into()],
            },
        };
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            value,
            json!({"name": "rarity", "description": "Quality tier", "value_type": "enum", "values": ["common", "rare", "legendary"]})
        );
        let deserialized: CreateGlobalMetaAttributeRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_create_client_request_round_trip() {
        let req = CreateClientRequest {
            name: "My Game".into(),
        };
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(value, json!({"name": "My Game"}));
        let deserialized: CreateClientRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_client_response_round_trip() {
        let resp = ClientResponse {
            id: Uuid::new_v4(),
            name: "My Game".into(),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            api_keys: vec![],
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: ClientResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_client_response_snake_case() {
        let resp = ClientResponse {
            id: Uuid::nil(),
            name: "x".into(),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            api_keys: vec![],
        };
        let obj = serde_json::to_value(&resp)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("created_at"));
        assert!(obj.contains_key("api_keys"));
    }

    #[test]
    fn test_create_api_key_request_round_trip() {
        let req = CreateApiKeyRequest {
            name: "game-server-key".into(),
            permissions: vec![Permission::Read, Permission::Generate],
        };
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            value,
            json!({"name": "game-server-key", "permissions": ["read", "generate"]})
        );
        let deserialized: CreateApiKeyRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_create_api_key_response_round_trip() {
        let resp = CreateApiKeyResponse {
            id: Uuid::new_v4(),
            name: "game-server-key".into(),
            permissions: vec![Permission::Read],
            key: "arche_k_xxxx".into(),
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: CreateApiKeyResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_api_key_response_round_trip() {
        let resp = ApiKeyResponse {
            id: Uuid::new_v4(),
            name: "game-server-key".into(),
            permissions: vec![Permission::Read],
            created_at: Utc::now(),
            expires_at: None,
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: ApiKeyResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_batch_delete_request_round_trip() {
        let req = BatchDeleteRequest {
            ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: BatchDeleteRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_batch_assign_request_round_trip() {
        let req = BatchAssignRequest {
            blueprint_ids: vec![Uuid::new_v4()],
            affix_ids: vec![Uuid::new_v4()],
            weight: 1.0,
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: BatchAssignRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_batch_edit_request_round_trip() {
        let mut attributes = HashMap::new();
        attributes.insert("damage".into(), json!({"min": 5, "max": 20}));
        let req = BatchEditRequest {
            blueprint_ids: vec![Uuid::new_v4()],
            attributes,
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: BatchEditRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_batch_delete_response_round_trip() {
        let resp = BatchDeleteResponse { count: 3 };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value, json!({"count": 3}));
        let deserialized: BatchDeleteResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_export_request_round_trip() {
        let req = ExportRequest {
            client_ids: vec![Uuid::new_v4()],
            include_api_keys: false,
            include_audit_log: true,
            inline_global_refs: false,
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: ExportRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_export_request_snake_case() {
        let req = ExportRequest {
            client_ids: vec![],
            include_api_keys: false,
            include_audit_log: false,
            inline_global_refs: false,
        };
        let obj = serde_json::to_value(&req)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("client_ids"));
        assert!(obj.contains_key("include_api_keys"));
        assert!(obj.contains_key("include_audit_log"));
        assert!(obj.contains_key("inline_global_refs"));
    }

    #[test]
    fn test_export_manifest_round_trip() {
        let m = ExportManifest {
            version: "1.0".into(),
            timestamp: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            client_count: 2,
        };
        let value = serde_json::to_value(&m).unwrap();
        let deserialized: ExportManifest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, m);
    }

    #[test]
    fn test_import_success_response_round_trip() {
        let resp = ImportSuccessResponse {
            status: "imported".into(),
            clients_created: 1,
            resources_imported: 42,
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: ImportSuccessResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_import_conflict_response_round_trip() {
        let resp = ImportConflictResponse {
            problem: ProblemJson {
                type_: "/errors/import-conflict".into(),
                title: "Import conflicts require resolution".into(),
                status: 409,
                detail: None,
            },
            conflicts: vec![ConflictDetail {
                resource_type: "blueprint".into(),
                resource_id: Uuid::new_v4(),
                resource_name: "Longsword".into(),
                attributes: vec![ConflictAttribute {
                    key: "damage".into(),
                    old_value: json!({"min": 10, "max": 23}),
                    new_value: json!({"min": 15, "max": 30}),
                    value_type: ValueType::Range,
                }],
            }],
            import_token: "tok_abc".into(),
        };
        let value = serde_json::to_value(&resp).unwrap();
        let deserialized: ImportConflictResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_import_conflict_response_snake_case() {
        let resp = ImportConflictResponse {
            problem: ProblemJson {
                type_: "/x".into(),
                title: "x".into(),
                status: 409,
                detail: None,
            },
            conflicts: vec![ConflictDetail {
                resource_type: "bp".into(),
                resource_id: Uuid::nil(),
                resource_name: "x".into(),
                attributes: vec![],
            }],
            import_token: "tok".into(),
        };
        let value = serde_json::to_value(&resp).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("conflicts"), "should have conflicts key");
        assert!(obj.contains_key("import_token"), "should have importToken key");
        let conflicts = obj.get("conflicts").unwrap().as_array().unwrap();
        let first = conflicts[0].as_object().unwrap();
        assert!(
            first.contains_key("resource_type"),
            "conflict should have resourceType"
        );
        assert!(
            first.contains_key("resource_id"),
            "conflict should have resourceId"
        );
        assert!(
            first.contains_key("resource_name"),
            "conflict should have resourceName"
        );
    }

    #[test]
    fn test_resolution_strategy_round_trip() {
        let cases = vec![
            (ResolutionStrategy::KeepOld, "\"keep_old\""),
            (ResolutionStrategy::KeepNew, "\"keep_new\""),
            (ResolutionStrategy::PerAttribute, "\"per_attribute\""),
        ];
        for (variant, expected) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let deserialized: ResolutionStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_conflict_resolution_request_round_trip() {
        let mut resolutions = HashMap::new();
        resolutions.insert(
            Uuid::new_v4(),
            ResourceResolution {
                strategy: ResolutionStrategy::PerAttribute,
                attributes: {
                    let mut m = HashMap::new();
                    m.insert("damage".into(), ResolutionStrategy::KeepNew);
                    m.insert("weight".into(), ResolutionStrategy::KeepOld);
                    Some(m)
                },
            },
        );
        let req = ConflictResolutionRequest {
            import_token: "tok_abc".into(),
            resolutions,
        };
        let value = serde_json::to_value(&req).unwrap();
        let deserialized: ConflictResolutionRequest = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_paginated_response_round_trip() {
        let resp: PaginatedResponse<String> = PaginatedResponse {
            data: vec!["a".into(), "b".into()],
            next_cursor: Some("cursor-1".into()),
            total: Some(42),
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(
            value,
            json!({"data": ["a", "b"], "next_cursor": "cursor-1", "total": 42})
        );
        let deserialized: PaginatedResponse<String> = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_paginated_response_minimal() {
        let resp: PaginatedResponse<i32> = PaginatedResponse {
            data: vec![1, 2, 3],
            next_cursor: None,
            total: None,
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value, json!({"data": [1, 2, 3]}));
    }

    #[test]
    fn test_paginated_response_snake_case() {
        let resp: PaginatedResponse<i32> = PaginatedResponse {
            data: vec![],
            next_cursor: Some("x".into()),
            total: None,
        };
        let obj = serde_json::to_value(&resp)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("next_cursor"));
    }

    #[test]
    fn test_bootstrap_response_round_trip() {
        let resp = BootstrapResponse {
            bootstrapped: true,
            key: Some("arche_k_abc123".into()),
            message: None,
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value, json!({"bootstrapped": true, "key": "arche_k_abc123"}));
        let deserialized: BootstrapResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_bootstrap_response_already_bootstrapped() {
        let resp = BootstrapResponse {
            bootstrapped: false,
            key: None,
            message: Some("Already bootstrapped. Key available in server logs.".into()),
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(
            value,
            json!({"bootstrapped": false, "message": "Already bootstrapped. Key available in server logs."})
        );
        let deserialized: BootstrapResponse = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, resp);
    }

    #[test]
    fn test_bootstrap_response_snake_case() {
        let resp = BootstrapResponse {
            bootstrapped: true,
            key: Some("arche_k_abc123".into()),
            message: None,
        };
        let obj = serde_json::to_value(&resp)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("bootstrapped"));
        assert!(obj.contains_key("key"));
    }

    #[test]
    fn test_problem_json_round_trip() {
        let problem = ProblemJson {
            type_: "/errors/not-found".into(),
            title: "Resource not found".into(),
            status: 404,
            detail: Some("Blueprint with id x not found".into()),
        };
        let value = serde_json::to_value(&problem).unwrap();
        assert_eq!(
            value,
            json!({"type": "/errors/not-found", "title": "Resource not found", "status": 404, "detail": "Blueprint with id x not found"})
        );
        let deserialized: ProblemJson = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, problem);
    }

    #[test]
    fn test_problem_json_no_detail() {
        let problem = ProblemJson {
            type_: "/errors/unauthorized".into(),
            title: "Unauthorized".into(),
            status: 401,
            detail: None,
        };
        let value = serde_json::to_value(&problem).unwrap();
        assert_eq!(
            value,
            json!({"type": "/errors/unauthorized", "title": "Unauthorized", "status": 401})
        );
    }

    #[test]
    fn test_blueprint_list_query_round_trip() {
        let q = BlueprintListQuery {
            cursor: Some("abc".into()),
            limit: None,
            page: Some(1),
            per_page: None,
            client_id: None,
            archetype: Some("sword".into()),
            search: None,
        };
        let value = serde_json::to_value(&q).unwrap();
        let deserialized: BlueprintListQuery = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, q);
    }

    #[test]
    fn test_audit_log_list_query_snake_case() {
        let q = AuditLogListQuery {
            cursor: None,
            limit: None,
            client_id: Some(uuid::Uuid::nil()),
            resource_type: Some("blueprint".into()),
            action: None,
            actor_key_id: Some(uuid::Uuid::nil()),
            from: None,
            to: None,
        };
        let obj = serde_json::to_value(&q)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        assert!(obj.contains_key("client_id"));
        assert!(obj.contains_key("resource_type"));
        assert!(obj.contains_key("actor_key_id"));
    }

    #[test]
    fn test_new_type_trait_bounds() {
        fn assert_traits<T: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + JsonSchema>() {}
        assert_traits::<DistributionConfig>();
        assert_traits::<AttributePayload>();
        assert_traits::<InlineAttributeDef>();
        assert_traits::<BlueprintAttribute>();
        assert_traits::<AffixInlineAttributeDef>();
        assert_traits::<AffixAttribute>();
        assert_traits::<AffixPoolEntry>();
        assert_traits::<BlueprintAffixConfig>();
        assert_traits::<ConstraintConfig>();
        assert_traits::<ConstraintValue>();
        assert_traits::<AffixConstraints>();
        assert_traits::<GenerateRequest>();
        assert_traits::<NameParts>();
        assert_traits::<AffixAttributeEntry>();
        assert_traits::<GenerateResponse>();
        assert_traits::<BatchGenerateRequest>();
        assert_traits::<BatchGenerateResultItem>();
        assert_traits::<BatchGenerateResponse>();
        assert_traits::<CreateBlueprintRequest>();
        assert_traits::<CreateAffixRequest>();
        assert_traits::<CreateGlobalMetaAttributeRequest>();
        assert_traits::<CreateClientRequest>();
        assert_traits::<ClientResponse>();
        assert_traits::<ApiKeySummary>();
        assert_traits::<CreateApiKeyRequest>();
        assert_traits::<CreateApiKeyResponse>();
        assert_traits::<ApiKeyResponse>();
        assert_traits::<BatchDeleteRequest>();
        assert_traits::<BatchAssignRequest>();
        assert_traits::<BatchEditRequest>();
        assert_traits::<BatchDeleteResponse>();
        assert_traits::<ExportRequest>();
        assert_traits::<ExportManifest>();
        assert_traits::<ImportSuccessResponse>();
        assert_traits::<ConflictAttribute>();
        assert_traits::<ConflictDetail>();
        assert_traits::<ImportConflictResponse>();
        assert_traits::<ResolutionStrategy>();
        assert_traits::<ResourceResolution>();
        assert_traits::<ConflictResolutionRequest>();
        assert_traits::<PaginatedResponse<String>>();
        assert_traits::<ProblemJson>();
        assert_traits::<BlueprintListQuery>();
        assert_traits::<AuditLogListQuery>();
        assert_traits::<BootstrapResponse>();
        assert_traits::<BlueprintResponse>();
    }
}

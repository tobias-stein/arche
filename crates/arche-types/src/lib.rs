use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod validation;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ValueType {
    Single,
    Enum,
    Range,
    String,
    Boolean,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AffixLocation {
    Prefix,
    Suffix,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AuditAction {
    Created,
    Updated,
    Deleted,
    ForceDeleted,
    Adjusted,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Generate,
    Admin,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintAffix {
    pub id: Uuid,
    pub blueprint_id: Uuid,
    pub affix_id: Uuid,
    pub weight: f64,
    pub location: AffixLocation,
    pub sort_order: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStrategy {
    KeepOld,
    KeepNew,
    PerAttribute,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ResourceResolution {
    pub strategy: ResolutionStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, ResolutionStrategy>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ConflictResolution {
    pub import_token: Uuid,
    pub resolutions: HashMap<Uuid, ResourceResolution>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ExportManifest {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub client_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::json;

    #[test]
    fn test_value_type_round_trip() {
        let cases = vec![
            (ValueType::Single, "\"single\""),
            (ValueType::Enum, "\"enum\""),
            (ValueType::Range, "\"range\""),
            (ValueType::String, "\"string\""),
            (ValueType::Boolean, "\"boolean\""),
        ];
        for (variant, expected) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let deserialized: ValueType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_affix_location_round_trip() {
        let cases = vec![
            (AffixLocation::Prefix, "\"prefix\""),
            (AffixLocation::Suffix, "\"suffix\""),
        ];
        for (variant, expected) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let deserialized: AffixLocation = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_audit_action_round_trip() {
        let cases = vec![
            (AuditAction::Created, "\"created\""),
            (AuditAction::Updated, "\"updated\""),
            (AuditAction::Deleted, "\"deleted\""),
            (AuditAction::ForceDeleted, "\"forceDeleted\""),
            (AuditAction::Adjusted, "\"adjusted\""),
        ];
        for (variant, expected) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let deserialized: AuditAction = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_permission_round_trip() {
        let cases = vec![
            (Permission::Read, "\"read\""),
            (Permission::Write, "\"write\""),
            (Permission::Delete, "\"delete\""),
            (Permission::Generate, "\"generate\""),
            (Permission::Admin, "\"admin\""),
        ];
        for (variant, expected) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected);
            let deserialized: Permission = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn test_client_round_trip() {
        let client = Client {
            id: Uuid::new_v4(),
            name: "Test Client".into(),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let json = serde_json::to_string(&client).unwrap();
        let deserialized: Client = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, client);
    }

    #[test]
    fn test_client_camel_case() {
        let client = Client {
            id: Uuid::nil(),
            name: "x".into(),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let value: serde_json::Value = serde_json::to_value(&client).unwrap();
        let obj = value.as_object().unwrap();
        assert!(
            obj.contains_key("createdAt"),
            "expected camelCase createdAt"
        );
        assert!(
            obj.contains_key("updatedAt"),
            "expected camelCase updatedAt"
        );
    }

    #[test]
    fn test_blueprint_round_trip() {
        let bp = Blueprint {
            id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            name: "Test Blueprint".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: Some("A test blueprint".into()),
            attributes: json!({"material": {"value_type": "string"}}),
            attribute_order: vec!["material".into()],
            min_prefixes: 0,
            max_prefixes: 1,
            min_suffixes: 0,
            max_suffixes: 1,
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let json = serde_json::to_string(&bp).unwrap();
        let deserialized: Blueprint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, bp);
    }

    #[test]
    fn test_blueprint_camel_case() {
        let bp = Blueprint {
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
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let value: serde_json::Value = serde_json::to_value(&bp).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("clientId"));
        assert!(obj.contains_key("attributeOrder"));
        assert!(obj.contains_key("minPrefixes"));
        assert!(obj.contains_key("maxPrefixes"));
        assert!(obj.contains_key("minSuffixes"));
        assert!(obj.contains_key("maxSuffixes"));
    }

    #[test]
    fn test_affix_round_trip() {
        let affix = Affix {
            id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            name: "Test Affix".into(),
            location: AffixLocation::Prefix,
            description: None,
            attribute: json!({"$ref_id": Uuid::new_v4().to_string()}),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let json = serde_json::to_string(&affix).unwrap();
        let deserialized: Affix = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, affix);
    }

    #[test]
    fn test_global_meta_round_trip() {
        let gma = GlobalMetaAttribute {
            id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            name: "Rarity".into(),
            description: Some("Item rarity".into()),
            value_type: ValueType::Enum,
            payload: json!({"values": ["common", "rare"]}),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let json = serde_json::to_string(&gma).unwrap();
        let deserialized: GlobalMetaAttribute = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, gma);
    }

    #[test]
    fn test_global_meta_camel_case() {
        let gma = GlobalMetaAttribute {
            id: Uuid::nil(),
            client_id: Uuid::nil(),
            name: "x".into(),
            description: None,
            value_type: ValueType::String,
            payload: json!({}),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        };
        let value: serde_json::Value = serde_json::to_value(&gma).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("clientId"));
        assert!(obj.contains_key("valueType"));
    }

    #[test]
    fn test_api_key_round_trip() {
        let key = ApiKey {
            id: Uuid::new_v4(),
            client_id: Some(Uuid::new_v4()),
            name: "My Key".into(),
            key_hash: "$2b$12$abc123".into(),
            permissions: vec![Permission::Read, Permission::Generate],
            is_super: false,
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            expires_at: None,
        };
        let json = serde_json::to_string(&key).unwrap();
        let deserialized: ApiKey = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, key);
    }

    #[test]
    fn test_api_key_camel_case() {
        let key = ApiKey {
            id: Uuid::nil(),
            client_id: None,
            name: "x".into(),
            key_hash: "x".into(),
            permissions: vec![],
            is_super: true,
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            expires_at: None,
        };
        let value: serde_json::Value = serde_json::to_value(&key).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("clientId"));
        assert!(obj.contains_key("keyHash"));
        assert!(obj.contains_key("isSuper"));
        assert!(obj.contains_key("expiresAt"));
    }

    #[test]
    fn test_audit_log_entry_round_trip() {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            actor_key_id: Uuid::new_v4(),
            actor_key_name: "admin".into(),
            client_id: Some(Uuid::new_v4()),
            resource_type: "blueprint".into(),
            resource_id: Uuid::new_v4(),
            action: AuditAction::Created,
            before: None,
            after: Some(json!({"name": "new blueprint"})),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, entry);
    }

    #[test]
    fn test_audit_log_camel_case() {
        let entry = AuditLogEntry {
            id: Uuid::nil(),
            timestamp: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            actor_key_id: Uuid::nil(),
            actor_key_name: "".into(),
            client_id: None,
            resource_type: "".into(),
            resource_id: Uuid::nil(),
            action: AuditAction::Deleted,
            before: None,
            after: None,
        };
        let value: serde_json::Value = serde_json::to_value(&entry).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("actorKeyId"));
        assert!(obj.contains_key("actorKeyName"));
        assert!(obj.contains_key("clientId"));
        assert!(obj.contains_key("resourceType"));
        assert!(obj.contains_key("resourceId"));
    }

    #[test]
    fn test_blueprint_affix_round_trip() {
        let ba = BlueprintAffix {
            id: Uuid::new_v4(),
            blueprint_id: Uuid::new_v4(),
            affix_id: Uuid::new_v4(),
            weight: 2.5,
            location: AffixLocation::Suffix,
            sort_order: 1,
        };
        let json = serde_json::to_string(&ba).unwrap();
        let deserialized: BlueprintAffix = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ba);
    }

    #[test]
    fn test_blueprint_affix_camel_case() {
        let ba = BlueprintAffix {
            id: Uuid::nil(),
            blueprint_id: Uuid::nil(),
            affix_id: Uuid::nil(),
            weight: 1.0,
            location: AffixLocation::Prefix,
            sort_order: 0,
        };
        let value: serde_json::Value = serde_json::to_value(&ba).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("blueprintId"));
        assert!(obj.contains_key("affixId"));
        assert!(obj.contains_key("sortOrder"));
    }

    #[test]
    fn test_trait_bounds() {
        fn assert_traits<T: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq>() {}
        assert_traits::<Client>();
        assert_traits::<Blueprint>();
        assert_traits::<Affix>();
        assert_traits::<GlobalMetaAttribute>();
        assert_traits::<ApiKey>();
        assert_traits::<AuditLogEntry>();
        assert_traits::<BlueprintAffix>();
        assert_traits::<ValueType>();
        assert_traits::<AffixLocation>();
        assert_traits::<AuditAction>();
        assert_traits::<Permission>();
        assert_traits::<ExportManifest>();
        assert_traits::<ConflictResolution>();
        assert_traits::<ResolutionStrategy>();
        assert_traits::<ResourceResolution>();
    }

    #[test]
    fn test_export_manifest_round_trip() {
        let manifest = ExportManifest {
            version: "1.0".into(),
            timestamp: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            client_count: 3,
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: ExportManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, manifest);
    }

    #[test]
    fn test_export_manifest_json_shape() {
        let manifest = ExportManifest {
            version: "1.0".into(),
            timestamp: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            client_count: 3,
        };
        let value: serde_json::Value = serde_json::to_value(&manifest).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("version"), "expected version field");
        assert!(obj.contains_key("timestamp"), "expected timestamp field");
        assert!(
            obj.contains_key("client_count"),
            "expected client_count field"
        );
        assert_eq!(obj["version"], "1.0");
        assert_eq!(obj["client_count"], 3);
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
    fn test_resource_resolution_strategy_only() {
        let r = ResourceResolution {
            strategy: ResolutionStrategy::KeepNew,
            attributes: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let expected = r#"{"strategy":"keep_new"}"#;
        assert_eq!(json, expected);
        let deserialized: ResourceResolution = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, r);
    }

    #[test]
    fn test_resource_resolution_with_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert("damage".into(), ResolutionStrategy::KeepNew);
        attrs.insert("weight".into(), ResolutionStrategy::KeepOld);
        let r = ResourceResolution {
            strategy: ResolutionStrategy::PerAttribute,
            attributes: Some(attrs),
        };
        let value: serde_json::Value = serde_json::to_value(&r).unwrap();
        let obj = value.as_object().unwrap();
        assert_eq!(obj["strategy"], "per_attribute");
        let attrs_obj = obj["attributes"].as_object().unwrap();
        assert_eq!(attrs_obj["damage"], "keep_new");
        assert_eq!(attrs_obj["weight"], "keep_old");
        let deserialized: ResourceResolution = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, r);
    }

    #[test]
    fn test_resource_resolution_omits_attributes_when_none() {
        let r = ResourceResolution {
            strategy: ResolutionStrategy::KeepNew,
            attributes: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(
            !json.contains("attributes"),
            "should omit attributes when None"
        );
    }

    #[test]
    fn test_conflict_resolution_round_trip() {
        let token = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let res_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let mut resolutions = HashMap::new();
        resolutions.insert(
            res_id,
            ResourceResolution {
                strategy: ResolutionStrategy::KeepNew,
                attributes: None,
            },
        );
        let cr = ConflictResolution {
            import_token: token,
            resolutions,
        };
        let json = serde_json::to_string(&cr).unwrap();
        let deserialized: ConflictResolution = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cr);
    }

    #[test]
    fn test_conflict_resolution_json_shape() {
        let token = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let res_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let mut resolutions = HashMap::new();
        resolutions.insert(
            res_id,
            ResourceResolution {
                strategy: ResolutionStrategy::PerAttribute,
                attributes: Some(HashMap::from([(
                    "damage".into(),
                    ResolutionStrategy::KeepNew,
                )])),
            },
        );
        let cr = ConflictResolution {
            import_token: token,
            resolutions,
        };
        let value: serde_json::Value = serde_json::to_value(&cr).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("import_token"), "expected import_token");
        assert!(obj.contains_key("resolutions"), "expected resolutions");
        let resol_obj = obj["resolutions"].as_object().unwrap();
        let inner = resol_obj
            .get("550e8400-e29b-41d4-a716-446655440001")
            .unwrap();
        assert_eq!(inner["strategy"], "per_attribute");
        assert_eq!(inner["attributes"]["damage"], "keep_new");
    }
}

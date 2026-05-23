use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#in: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eq: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum ConstraintValue {
    Config(ConstraintConfig),
    Bare(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AffixConstraints {
    #[serde(default)]
    pub min_prefixes: i32,
    #[serde(default)]
    pub max_prefixes: i32,
    #[serde(default)]
    pub min_suffixes: i32,
    #[serde(default)]
    pub max_suffixes: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub require: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub block: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<HashMap<String, ConstraintValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affixes: Option<AffixConstraints>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NameParts {
    pub base: String,
    #[serde(default)]
    pub prefixes: Vec<String>,
    #[serde(default)]
    pub suffixes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AffixAttributeEntry {
    pub affix_id: Uuid,
    pub affix_name: String,
    #[serde(flatten)]
    pub attributes: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResponse {
    pub seed: u64,
    pub name: String,
    pub name_parts: NameParts,
    pub blueprint_id: Uuid,
    pub blueprint_attributes: HashMap<String, serde_json::Value>,
    pub affix_attributes: Vec<AffixAttributeEntry>,
}

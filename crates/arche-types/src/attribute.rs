use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DistributionConfig {
    Uniform,
    Normal { std_dev: f64 },
    Exponential { rate: f64 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(
    tag = "value_type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AttributePayload {
    Single {
        value: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        distribution: Option<DistributionConfig>,
    },
    #[serde(rename = "enum")]
    Enum {
        values: Vec<String>,
    },
    Range {
        min: f64,
        max: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        distribution: Option<DistributionConfig>,
    },
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<i32>,
    },
    Boolean {
        value: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InlineAttributeDef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub payload: AttributePayload,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum BlueprintAttribute {
    Inline(InlineAttributeDef),
    Ref {
        #[serde(rename = "$ref_id")]
        ref_id: Uuid,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AffixInlineAttributeDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub payload: AttributePayload,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum AffixAttribute {
    Inline(AffixInlineAttributeDef),
    Ref {
        #[serde(rename = "$ref_id")]
        ref_id: Uuid,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AffixPoolEntry {
    pub affix_id: Uuid,
    pub weight: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintAffixConfig {
    pub min_prefixes: i32,
    pub max_prefixes: i32,
    pub min_suffixes: i32,
    pub max_suffixes: i32,
    #[serde(default)]
    pub prefixes: Vec<AffixPoolEntry>,
    #[serde(default)]
    pub suffixes: Vec<AffixPoolEntry>,
}

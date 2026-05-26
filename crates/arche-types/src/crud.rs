use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::attribute::{
    AffixAttribute, AttributePayload, BlueprintAffixConfig, BlueprintAttribute,
};
use crate::{AffixLocation, Permission};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintResponse {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: String,
    pub archetype: String,
    pub weight: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub attributes: serde_json::Value,
    pub attribute_order: Vec<String>,
    pub min_prefixes: i32,
    pub max_prefixes: i32,
    pub min_suffixes: i32,
    pub max_suffixes: i32,
    pub affixes: BlueprintAffixConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBlueprintRequest {
    pub name: String,
    pub archetype: String,
    pub weight: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub attributes: HashMap<String, BlueprintAttribute>,
    pub attribute_order: Vec<String>,
    pub affixes: BlueprintAffixConfig,
}

pub type UpdateBlueprintRequest = CreateBlueprintRequest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAffixRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub location: AffixLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub attribute: AffixAttribute,
}

pub type UpdateAffixRequest = CreateAffixRequest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateGlobalMetaAttributeRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub payload: AttributePayload,
}

pub type UpdateGlobalMetaAttributeRequest = CreateGlobalMetaAttributeRequest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySummary {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub api_keys: Vec<ApiKeySummary>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub bootstrapped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

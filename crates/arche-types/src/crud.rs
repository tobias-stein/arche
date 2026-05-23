use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::attribute::{AffixAttribute, BlueprintAffixConfig, BlueprintAttribute};
use crate::{AffixLocation, Permission};

// ── Blueprint ──

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

// ── Affix ──

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

// ── Global Meta Attribute ──

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateGlobalMetaAttributeRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub payload: crate::attribute::AttributePayload,
}

pub type UpdateGlobalMetaAttributeRequest = CreateGlobalMetaAttributeRequest;

// ── Client ──

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySummary {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub api_keys: Vec<ApiKeySummary>,
}

// ── API Key ──

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
}

// ── Schema endpoints (no special types needed — returns serde_json::Value) ──

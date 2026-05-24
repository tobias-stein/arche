use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::common::ProblemJson;
use crate::ValueType;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub client_ids: Vec<Uuid>,
    #[serde(default)]
    pub include_api_keys: bool,
    #[serde(default)]
    pub include_audit_log: bool,
    #[serde(default)]
    pub inline_global_refs: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportManifest {
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub client_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportSuccessResponse {
    pub status: String,
    pub clients_created: i64,
    pub resources_imported: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConflictAttribute {
    pub key: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub value_type: ValueType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConflictDetail {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub resource_name: String,
    #[serde(default)]
    pub attributes: Vec<ConflictAttribute>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportConflictResponse {
    #[serde(flatten)]
    pub problem: ProblemJson,
    #[serde(default)]
    pub conflicts: Vec<ConflictDetail>,
    pub import_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResolutionStrategy {
    KeepOld,
    KeepNew,
    PerAttribute,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceResolution {
    pub strategy: ResolutionStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, ResolutionStrategy>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConflictResolutionRequest {
    pub import_token: String,
    pub resolutions: HashMap<Uuid, ResourceResolution>,
}

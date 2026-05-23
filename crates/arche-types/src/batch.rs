use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteResponse {
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatchAssignRequest {
    pub blueprint_ids: Vec<Uuid>,
    pub affix_ids: Vec<Uuid>,
    pub weight: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatchEditRequest {
    pub blueprint_ids: Vec<Uuid>,
    pub attributes: HashMap<String, serde_json::Value>,
}

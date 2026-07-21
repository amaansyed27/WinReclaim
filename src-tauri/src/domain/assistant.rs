use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageAssistantStatus {
    pub available: bool,
    pub busy: bool,
    pub provider: String,
    pub model: String,
    pub privacy_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeStorageRequest {
    pub scan_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageAssistantReport {
    pub scan_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub model: String,
    pub summary: String,
    pub observations: Vec<String>,
    pub advisory_only: bool,
}

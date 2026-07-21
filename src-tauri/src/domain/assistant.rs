use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageAssistantStatus {
    pub installed: bool,
    pub verified: bool,
    pub busy: bool,
    pub model: String,
    pub quantization: String,
    pub runtime: String,
    pub model_bytes: u64,
    pub expected_bytes: u64,
    pub model_path: String,
    pub license: String,
    pub privacy_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantDownloadProgress {
    pub phase: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeStorageRequest {
    pub scan_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantAnnotation {
    pub finding_id: Uuid,
    pub suggested_name: String,
    pub group: String,
    pub explanation: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageAssistantReport {
    pub scan_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub model: String,
    pub summary: String,
    pub observations: Vec<String>,
    pub annotations: Vec<AssistantAnnotation>,
    pub advisory_only: bool,
}

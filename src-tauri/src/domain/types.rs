use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RiskClass {
    SafeNow,
    RebuildOrRedownload,
    ReviewFirst,
    Protected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    UserTemp,
    CrashDumps,
    HuggingfacePrune,
    NpmCache,
    DockerPrune,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSnapshot {
    pub root: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: Uuid,
    pub rule_id: String,
    pub display_name: String,
    pub category: String,
    pub path: String,
    pub estimated_bytes: u64,
    pub risk_class: RiskClass,
    pub explanation: String,
    pub consequence: String,
    pub confidence: Confidence,
    pub action_kind: Option<ActionKind>,
    pub action_available: bool,
    pub selected_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub phase: String,
    pub current_path: Option<String>,
    pub completed_targets: usize,
    pub total_targets: usize,
    pub discovered_bytes: u64,
    pub scanned_entries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub scan_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub root: String,
    pub disk: DiskSnapshot,
    pub findings: Vec<Finding>,
    pub scanned_entries: u64,
    pub skipped_entries: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub configured: bool,
    pub model: String,
    pub privacy_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentRequest {
    pub scan_id: Uuid,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntentSuggestion {
    pub selected_finding_ids: Vec<Uuid>,
    pub target_reclaim_bytes: Option<u64>,
    pub estimated_reclaim_bytes: u64,
    pub allowed_risk_classes: Vec<RiskClass>,
    pub excluded_labels: Vec<String>,
    pub summary: String,
    pub model: String,
    pub remote_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanRequest {
    pub scan_id: Uuid,
    pub finding_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlanItem {
    pub finding_id: Uuid,
    pub display_name: String,
    pub category: String,
    pub path: String,
    pub estimated_bytes: u64,
    pub risk_class: RiskClass,
    pub consequence: String,
    pub action_kind: ActionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlan {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub estimated_reclaim_bytes: u64,
    pub items: Vec<CleanupPlanItem>,
    pub rule_set_version: String,
    pub plan_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanRequest {
    pub plan_id: Uuid,
    pub plan_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub finding_id: Uuid,
    pub display_name: String,
    pub estimated_bytes: u64,
    pub measured_bytes_before: u64,
    pub measured_bytes_after: u64,
    pub deleted_entries: u64,
    pub skipped_entries: u64,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupReceipt {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub plan_hash: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub disk_free_before: u64,
    pub disk_free_after: u64,
    pub actual_reclaimed_bytes: u64,
    pub estimated_reclaim_bytes: u64,
    pub results: Vec<ActionResult>,
    pub protected_summary: Vec<String>,
    pub rule_set_version: String,
}

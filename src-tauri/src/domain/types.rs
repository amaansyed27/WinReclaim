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
    SystemTemp,
    RecycleBin,
    CrashDumps,
    HuggingfacePrune,
    NpmCache,
    DockerPrune,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryClass {
    Reversible,
    Redownloadable,
    Rebuildable,
    #[default]
    Irreversible,
    Protected,
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
    pub scope_fingerprint: String,
    pub disk: DiskSnapshot,
    pub findings: Vec<Finding>,
    pub scanned_entries: u64,
    pub skipped_entries: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScanMode {
    Quick,
    #[default]
    Balanced,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub mode: ScanMode,
    #[serde(default = "default_true")]
    pub include_known_targets: bool,
    #[serde(default)]
    pub include_project_outputs: bool,
    #[serde(default)]
    pub discover_unknown: bool,
    #[serde(default)]
    pub include_app_data: bool,
    #[serde(default = "default_true")]
    pub include_system_drive_caches: bool,
    #[serde(default = "default_minimum_finding_bytes")]
    pub minimum_finding_bytes: u64,
    #[serde(default = "default_max_unknown_findings")]
    pub max_unknown_findings: usize,
}

impl Default for ScanRequest {
    fn default() -> Self {
        Self {
            root: None,
            mode: ScanMode::Balanced,
            include_known_targets: true,
            include_project_outputs: false,
            discover_unknown: false,
            include_app_data: false,
            include_system_drive_caches: true,
            minimum_finding_bytes: default_minimum_finding_bytes(),
            max_unknown_findings: default_max_unknown_findings(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_minimum_finding_bytes() -> u64 {
    512 * 1024 * 1024
}

fn default_max_unknown_findings() -> usize {
    20
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlanSimulation {
    pub current_free_bytes: u64,
    pub projected_free_bytes: u64,
    pub estimated_reclaim_bytes: u64,
    pub reversible_bytes: u64,
    pub redownloadable_bytes: u64,
    pub rebuildable_bytes: u64,
    pub irreversible_bytes: u64,
    pub affected_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlan {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub estimated_reclaim_bytes: u64,
    pub items: Vec<CleanupPlanItem>,
    #[serde(default)]
    pub simulation: PlanSimulation,
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
    #[serde(default)]
    pub recovery_class: RecoveryClass,
    #[serde(default)]
    pub vault_entry_ids: Vec<Uuid>,
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
    #[serde(default)]
    pub vault_entry_ids: Vec<Uuid>,
    pub rule_set_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReclaimPassport {
    pub finding_id: Uuid,
    pub last_changed_at: Option<DateTime<Utc>>,
    pub recovery_class: RecoveryClass,
    pub recovery_method: String,
    pub activity_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummary {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub captured_at: DateTime<Utc>,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineDelta {
    pub key: String,
    pub display_name: String,
    pub category: String,
    pub path: String,
    pub previous_bytes: u64,
    pub current_bytes: u64,
    pub delta_bytes: i64,
    pub action_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StorageTimeline {
    pub snapshots: Vec<SnapshotSummary>,
    pub deltas: Vec<TimelineDelta>,
    pub total_growth_bytes: Option<i64>,
    pub compared_with_at: Option<DateTime<Utc>>,
    pub baseline_available: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VaultStatus {
    Active,
    Restored,
    PartiallyRestored,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultEntry {
    pub id: Uuid,
    pub receipt_id: Uuid,
    pub finding_id: Uuid,
    pub display_name: String,
    pub original_root: String,
    pub payload_root: String,
    pub relative_paths: Vec<String>,
    pub stored_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub restored_at: Option<DateTime<Utc>>,
    pub status: VaultStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRequest {
    pub vault_entry_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub vault_entry_id: Uuid,
    pub restored_entries: u64,
    pub skipped_entries: u64,
    pub restored_bytes: u64,
    pub status: VaultStatus,
    pub message: String,
}

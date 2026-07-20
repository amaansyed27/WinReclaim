use crate::domain::{
    Finding, ReclaimPassport, RecoveryClass, ScanReport, SnapshotSummary, StorageTimeline,
    TimelineDelta,
};
use crate::policy::{recovery_class_for_finding, SNAPSHOT_LIMIT, SNAPSHOT_SCHEMA_VERSION};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotRecord {
    #[serde(default = "legacy_snapshot_schema_version")]
    schema_version: u32,
    #[serde(default)]
    scope_fingerprint: String,
    id: Uuid,
    scan_id: Uuid,
    captured_at: DateTime<Utc>,
    used_bytes: u64,
    free_bytes: u64,
    findings: Vec<SnapshotFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotFinding {
    key: String,
    display_name: String,
    category: String,
    path: String,
    estimated_bytes: u64,
    action_available: bool,
}

pub fn persist_scan_snapshot(report: &ScanReport) -> Result<()> {
    let root = snapshot_root();
    fs::create_dir_all(&root)?;
    let snapshot = SnapshotRecord {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        scope_fingerprint: report.scope_fingerprint.clone(),
        id: Uuid::new_v4(),
        scan_id: report.scan_id,
        captured_at: report.completed_at,
        used_bytes: report.disk.used_bytes,
        free_bytes: report.disk.free_bytes,
        findings: report.findings.iter().map(snapshot_finding).collect(),
    };
    let filename = format!(
        "{}-{}.json",
        snapshot.captured_at.timestamp_millis(),
        snapshot.id
    );
    fs::write(root.join(filename), serde_json::to_vec_pretty(&snapshot)?)?;
    prune_snapshots(&root)?;
    Ok(())
}

pub fn build_storage_timeline() -> Result<StorageTimeline> {
    Ok(build_storage_timeline_from_records(load_snapshot_records()?))
}

fn build_storage_timeline_from_records(records: Vec<SnapshotRecord>) -> StorageTimeline {
    let Some(latest) = records.last() else {
        return StorageTimeline::default();
    };

    let snapshots = records
        .iter()
        .map(|record| SnapshotSummary {
            id: record.id,
            scan_id: record.scan_id,
            captured_at: record.captured_at,
            used_bytes: record.used_bytes,
            free_bytes: record.free_bytes,
        })
        .collect::<Vec<_>>();

    let previous = if latest.scope_fingerprint.is_empty() {
        None
    } else {
        records[..records.len().saturating_sub(1)]
            .iter()
            .rev()
            .find(|record| {
                record.schema_version == SNAPSHOT_SCHEMA_VERSION
                    && record.scope_fingerprint == latest.scope_fingerprint
            })
    };

    let Some(previous) = previous else {
        return StorageTimeline {
            snapshots,
            deltas: Vec::new(),
            total_growth_bytes: None,
            compared_with_at: None,
            baseline_available: false,
        };
    };

    let previous_map = previous
        .findings
        .iter()
        .map(|finding| (finding.key.as_str(), finding))
        .collect::<HashMap<_, _>>();
    let current_map = latest
        .findings
        .iter()
        .map(|finding| (finding.key.as_str(), finding))
        .collect::<HashMap<_, _>>();
    let mut deltas = Vec::new();

    for finding in &latest.findings {
        let previous_bytes = previous_map
            .get(finding.key.as_str())
            .map(|candidate| candidate.estimated_bytes)
            .unwrap_or_default();
        let delta_bytes = signed_delta(finding.estimated_bytes, previous_bytes);
        if delta_bytes == 0 {
            continue;
        }
        deltas.push(delta_from(
            finding,
            previous_bytes,
            finding.estimated_bytes,
            delta_bytes,
        ));
    }

    for finding in &previous.findings {
        if current_map.contains_key(finding.key.as_str()) {
            continue;
        }
        deltas.push(delta_from(
            finding,
            finding.estimated_bytes,
            0,
            signed_delta(0, finding.estimated_bytes),
        ));
    }

    deltas.sort_by_key(|delta| std::cmp::Reverse(delta.delta_bytes.unsigned_abs()));

    StorageTimeline {
        snapshots,
        deltas,
        total_growth_bytes: Some(signed_delta(latest.used_bytes, previous.used_bytes)),
        compared_with_at: Some(previous.captured_at),
        baseline_available: true,
    }
}

pub fn build_reclaim_passports(report: &ScanReport) -> Vec<ReclaimPassport> {
    report.findings.iter().map(passport_for).collect()
}

fn snapshot_finding(finding: &Finding) -> SnapshotFinding {
    SnapshotFinding {
        key: finding_key(finding),
        display_name: finding.display_name.clone(),
        category: finding.category.clone(),
        path: finding.path.clone(),
        estimated_bytes: finding.estimated_bytes,
        action_available: finding.action_available,
    }
}

fn delta_from(
    finding: &SnapshotFinding,
    previous_bytes: u64,
    current_bytes: u64,
    delta_bytes: i64,
) -> TimelineDelta {
    TimelineDelta {
        key: finding.key.clone(),
        display_name: finding.display_name.clone(),
        category: finding.category.clone(),
        path: finding.path.clone(),
        previous_bytes,
        current_bytes,
        delta_bytes,
        action_available: finding.action_available,
    }
}

fn passport_for(finding: &Finding) -> ReclaimPassport {
    let recovery_class = recovery_class_for_finding(finding);
    let last_changed_at = modified_at(Path::new(&finding.path));
    ReclaimPassport {
        finding_id: finding.id,
        last_changed_at,
        recovery_class,
        recovery_method: recovery_method(finding, recovery_class),
        activity_note: activity_note(last_changed_at.as_ref()),
    }
}

fn recovery_method(finding: &Finding, recovery_class: RecoveryClass) -> String {
    match recovery_class {
        RecoveryClass::Reversible => {
            "Eligible files appear in Restore files until their recorded expiry".to_string()
        }
        RecoveryClass::Redownloadable => recovery_method_for_redownload(finding),
        RecoveryClass::Rebuildable => recovery_method_for_rebuild(finding),
        RecoveryClass::Irreversible => {
            "No automatic restore is available; review the cleanup consequence before continuing"
                .to_string()
        }
        RecoveryClass::Protected => "Automatic cleanup is disabled".to_string(),
    }
}

fn recovery_method_for_redownload(finding: &Finding) -> String {
    if finding.rule_id.starts_with("npm.") {
        "npm downloads packages again on the next install".to_string()
    } else if finding.rule_id.starts_with("huggingface.") {
        "The owning model or dataset command downloads missing revisions again".to_string()
    } else if finding.rule_id.starts_with("gradle.") {
        "Gradle downloads missing artifacts during the next build".to_string()
    } else {
        "The owning tool can download the data again".to_string()
    }
}

fn recovery_method_for_rebuild(finding: &Finding) -> String {
    if finding.rule_id == "project.node_modules" {
        "Run the project package-manager install command using its lockfile".to_string()
    } else if finding.rule_id == "project.rust_target" {
        "Run cargo build to recreate the target directory".to_string()
    } else if finding.rule_id == "project.python_venv" {
        "Recreate the virtual environment and reinstall dependencies".to_string()
    } else {
        "Run the owning project build to regenerate this output".to_string()
    }
}

fn activity_note(last_changed_at: Option<&DateTime<Utc>>) -> String {
    let Some(changed_at) = last_changed_at else {
        return "Filesystem activity time is unavailable".to_string();
    };
    let age = Utc::now().signed_duration_since(*changed_at);
    if age.num_hours() < 24 {
        "Changed within the last day".to_string()
    } else if age.num_days() < 7 {
        format!("Changed {} days ago", age.num_days().max(1))
    } else if age.num_days() < 30 {
        format!("No directory-level change for {} days", age.num_days())
    } else {
        format!(
            "No directory-level change for about {} months",
            age.num_days() / 30
        )
    }
}

fn modified_at(path: &Path) -> Option<DateTime<Utc>> {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .map(DateTime::<Utc>::from)
}

fn finding_key(finding: &Finding) -> String {
    format!("{}|{}", finding.rule_id, finding.path.to_ascii_lowercase())
}

fn signed_delta(current: u64, previous: u64) -> i64 {
    if current >= previous {
        current.saturating_sub(previous).min(i64::MAX as u64) as i64
    } else {
        -(previous.saturating_sub(current).min(i64::MAX as u64) as i64)
    }
}

fn load_snapshot_records() -> Result<Vec<SnapshotRecord>> {
    let root = snapshot_root();
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = match fs::read(entry.path()) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if let Ok(record) = serde_json::from_slice::<SnapshotRecord>(&bytes) {
            records.push(record);
        }
    }
    records.sort_by_key(|record| record.captured_at);
    Ok(records)
}

fn prune_snapshots(root: &Path) -> Result<()> {
    let mut files = fs::read_dir(root)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.file_name());
    let excess = files.len().saturating_sub(SNAPSHOT_LIMIT);
    for entry in files.into_iter().take(excess) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

fn snapshot_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("WinReclaim")
        .join("snapshots")
}

fn legacy_snapshot_schema_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn record(
        scope: &str,
        captured_at: DateTime<Utc>,
        used_bytes: u64,
        finding_bytes: u64,
    ) -> SnapshotRecord {
        SnapshotRecord {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            scope_fingerprint: scope.to_string(),
            id: Uuid::new_v4(),
            scan_id: Uuid::new_v4(),
            captured_at,
            used_bytes,
            free_bytes: 1_000_u64.saturating_sub(used_bytes),
            findings: vec![SnapshotFinding {
                key: "test|c:\\test".to_string(),
                display_name: "Test cache".to_string(),
                category: "Test".to_string(),
                path: "C:\\test".to_string(),
                estimated_bytes: finding_bytes,
                action_available: true,
            }],
        }
    }

    #[test]
    fn signed_delta_preserves_direction() {
        assert_eq!(signed_delta(12, 5), 7);
        assert_eq!(signed_delta(5, 12), -7);
    }

    #[test]
    fn history_does_not_compare_different_scan_scopes() {
        let now = Utc::now();
        let timeline = build_storage_timeline_from_records(vec![
            record("balanced", now - Duration::minutes(5), 500, 100),
            record("deep", now, 650, 250),
        ]);
        assert!(!timeline.baseline_available);
        assert_eq!(timeline.total_growth_bytes, None);
        assert!(timeline.deltas.is_empty());
    }

    #[test]
    fn history_compares_matching_scan_scopes() {
        let now = Utc::now();
        let timeline = build_storage_timeline_from_records(vec![
            record("balanced", now - Duration::minutes(10), 500, 100),
            record("deep", now - Duration::minutes(5), 700, 300),
            record("balanced", now, 550, 120),
        ]);
        assert!(timeline.baseline_available);
        assert_eq!(timeline.total_growth_bytes, Some(50));
        assert_eq!(timeline.compared_with_at, Some(now - Duration::minutes(10)));
        assert_eq!(timeline.deltas.len(), 1);
        assert_eq!(timeline.deltas[0].delta_bytes, 20);
    }
}

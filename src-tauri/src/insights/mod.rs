use crate::domain::{
    ActionKind, Confidence, Finding, ReclaimPassport, RecoveryClass, RiskClass, ScanReport,
    SnapshotSummary, StorageTimeline, TimelineDelta,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SNAPSHOT_LIMIT: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotRecord {
    id: Uuid,
    scan_id: Uuid,
    captured_at: DateTime<Utc>,
    used_bytes: u64,
    free_bytes: u64,
    classified_bytes: u64,
    findings: Vec<SnapshotFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotFinding {
    key: String,
    display_name: String,
    category: String,
    path: String,
    owner: String,
    estimated_bytes: u64,
    confidence_score: u8,
    recovery_class: RecoveryClass,
    action_available: bool,
}

pub fn persist_scan_snapshot(report: &ScanReport) -> Result<()> {
    let root = snapshot_root();
    fs::create_dir_all(&root)?;
    let classified_bytes = report
        .findings
        .iter()
        .map(|finding| finding.estimated_bytes)
        .sum();
    let snapshot = SnapshotRecord {
        id: Uuid::new_v4(),
        scan_id: report.scan_id,
        captured_at: report.completed_at,
        used_bytes: report.disk.used_bytes,
        free_bytes: report.disk.free_bytes,
        classified_bytes,
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
    let records = load_snapshot_records()?;
    let Some(latest) = records.last() else {
        return Ok(StorageTimeline::default());
    };

    let snapshots = records
        .iter()
        .map(|record| SnapshotSummary {
            id: record.id,
            scan_id: record.scan_id,
            captured_at: record.captured_at,
            used_bytes: record.used_bytes,
            free_bytes: record.free_bytes,
            classified_bytes: record.classified_bytes,
            finding_count: record.findings.len(),
        })
        .collect::<Vec<_>>();

    let previous = if records.len() >= 2 {
        records.get(records.len() - 2)
    } else {
        None
    };
    let mut deltas = Vec::new();
    let mut reclaimable_growth_bytes = 0_u64;

    if let Some(previous) = previous {
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

        for finding in &latest.findings {
            let previous_bytes = previous_map
                .get(finding.key.as_str())
                .map(|candidate| candidate.estimated_bytes)
                .unwrap_or_default();
            let delta_bytes = signed_delta(finding.estimated_bytes, previous_bytes);
            if delta_bytes == 0 {
                continue;
            }
            if delta_bytes > 0 && finding.action_available {
                reclaimable_growth_bytes =
                    reclaimable_growth_bytes.saturating_add(delta_bytes.unsigned_abs());
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
    }

    deltas.sort_by_key(|delta| std::cmp::Reverse(delta.delta_bytes.unsigned_abs()));
    let total_growth_bytes = previous
        .map(|record| signed_delta(latest.used_bytes, record.used_bytes))
        .unwrap_or_default();

    Ok(StorageTimeline {
        snapshots,
        deltas,
        total_growth_bytes,
        reclaimable_growth_bytes,
        baseline_available: previous.is_some(),
    })
}

pub fn build_reclaim_passports(report: &ScanReport) -> Vec<ReclaimPassport> {
    report.findings.iter().map(passport_for).collect()
}

pub fn recovery_class_for(finding: &Finding) -> RecoveryClass {
    match finding.action_kind {
        Some(ActionKind::UserTemp | ActionKind::CrashDumps) => RecoveryClass::Reversible,
        Some(ActionKind::HuggingfacePrune | ActionKind::NpmCache) => {
            RecoveryClass::Redownloadable
        }
        Some(ActionKind::DockerPrune) => RecoveryClass::Irreversible,
        None if finding.risk_class == RiskClass::Protected => RecoveryClass::Protected,
        None if finding.rule_id.starts_with("project.") => RecoveryClass::Rebuildable,
        None if finding.risk_class == RiskClass::RebuildOrRedownload => {
            RecoveryClass::Redownloadable
        }
        None => RecoveryClass::Irreversible,
    }
}

fn snapshot_finding(finding: &Finding) -> SnapshotFinding {
    SnapshotFinding {
        key: finding_key(finding),
        display_name: finding.display_name.clone(),
        category: finding.category.clone(),
        path: finding.path.clone(),
        owner: owner_for(finding),
        estimated_bytes: finding.estimated_bytes,
        confidence_score: confidence_score(finding),
        recovery_class: recovery_class_for(finding),
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
        owner: finding.owner.clone(),
        previous_bytes,
        current_bytes,
        delta_bytes,
        confidence_score: finding.confidence_score,
        recovery_class: finding.recovery_class,
    }
}

fn passport_for(finding: &Finding) -> ReclaimPassport {
    let recovery_class = recovery_class_for(finding);
    let last_changed_at = modified_at(Path::new(&finding.path));
    let activity_note = activity_note(last_changed_at.as_ref());
    let recovery_method = match recovery_class {
        RecoveryClass::Reversible => {
            "Restore from the WinReclaim Undo Vault before the seven-day expiry".to_string()
        }
        RecoveryClass::Redownloadable => recovery_method_for_redownload(finding),
        RecoveryClass::Rebuildable => recovery_method_for_rebuild(finding),
        RecoveryClass::Irreversible => {
            "No automatic restore is available; review the owning application before cleanup"
                .to_string()
        }
        RecoveryClass::Protected => "Automatic cleanup is disabled".to_string(),
    };
    let estimated_recovery_minutes = match recovery_class {
        RecoveryClass::Reversible => Some(1),
        RecoveryClass::Redownloadable => Some(8),
        RecoveryClass::Rebuildable => Some(6),
        RecoveryClass::Irreversible | RecoveryClass::Protected => None,
    };
    let mut evidence = vec![
        format!("Rule: {}", finding.rule_id),
        format!("Safety class: {:?}", finding.risk_class),
        format!("Classifier confidence: {:?}", finding.confidence),
    ];
    evidence.push(if finding.action_available {
        "A compiled Rust cleanup adapter is available".to_string()
    } else {
        "Inspection only; no cleanup adapter is exposed".to_string()
    });

    ReclaimPassport {
        finding_id: finding.id,
        owner: owner_for(finding),
        last_changed_at,
        recovery_class,
        recovery_method,
        estimated_recovery_minutes,
        confidence_score: confidence_score(finding),
        activity_note,
        evidence,
    }
}

fn owner_for(finding: &Finding) -> String {
    let rule = finding.rule_id.as_str();
    if rule.starts_with("npm.") || rule == "project.node_modules" {
        "Node.js / npm".to_string()
    } else if rule.starts_with("huggingface.") {
        "Hugging Face".to_string()
    } else if rule.starts_with("docker.") {
        "Docker Desktop".to_string()
    } else if rule.starts_with("android.") || rule.starts_with("gradle.") {
        "Android / Gradle".to_string()
    } else if rule.starts_with("cargo.") || rule == "project.rust_target" {
        "Rust / Cargo".to_string()
    } else if rule.starts_with("pip.")
        || rule.starts_with("uv.")
        || rule == "project.python_venv"
    {
        "Python tooling".to_string()
    } else if rule.starts_with("ollama.") {
        "Ollama".to_string()
    } else if rule.starts_with("browser.") {
        "Web browser".to_string()
    } else if rule.starts_with("editor.") {
        "Developer editor".to_string()
    } else if rule.starts_with("project.") {
        "Local project".to_string()
    } else if rule.starts_with("windows.") {
        "Windows / installed applications".to_string()
    } else if rule.starts_with("dynamic.") {
        "Unclassified local application".to_string()
    } else {
        finding.category.clone()
    }
}

fn confidence_score(finding: &Finding) -> u8 {
    let base: u8 = match finding.confidence {
        Confidence::High => 94,
        Confidence::Medium => 79,
        Confidence::Low => 58,
    };
    if finding.rule_id.starts_with("dynamic.") {
        base.saturating_sub(12)
    } else {
        base
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
        "Changed within the last day; treat as active".to_string()
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
    records.sort_by(|left, right| left.captured_at.cmp(&right.captured_at));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_delta_preserves_direction() {
        assert_eq!(signed_delta(12, 5), 7);
        assert_eq!(signed_delta(5, 12), -7);
    }
}

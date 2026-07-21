use crate::domain::{ActionKind, DriveInfo, Finding, RecoveryClass, RiskClass, ScanRequest};
use crate::rules::RULE_SET_VERSION;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub const VAULT_RETENTION_DAYS: i64 = 7;
pub const SNAPSHOT_LIMIT: usize = 40;
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 3;

pub fn scan_scope_fingerprint(
    roots: &[PathBuf],
    drives: &[DriveInfo],
    request: &ScanRequest,
) -> String {
    let mut normalized_roots = roots
        .iter()
        .map(|root| {
            root.canonicalize()
                .unwrap_or_else(|_| PathBuf::from(root))
                .to_string_lossy()
                .to_ascii_lowercase()
        })
        .collect::<Vec<_>>();
    normalized_roots.sort();

    let mut volumes = drives
        .iter()
        .map(|drive| {
            format!(
                "{}:{}:{}",
                drive.volume_id.to_ascii_lowercase(),
                drive.file_system.to_ascii_lowercase(),
                drive.total_bytes
            )
        })
        .collect::<Vec<_>>();
    volumes.sort();

    let signature = format!(
        "{}|{}|{:?}|{}|{}|{}|{}|{}|{}|{}|{RULE_SET_VERSION}",
        normalized_roots.join(","),
        volumes.join(","),
        request.mode,
        request.include_known_targets,
        request.include_project_outputs,
        request.discover_unknown,
        request.include_app_data,
        request.include_system_drive_caches,
        request.minimum_finding_bytes,
        request.max_unknown_findings,
    );
    hex::encode(Sha256::digest(signature.as_bytes()))
}

pub fn recovery_class_for_action(action: ActionKind) -> RecoveryClass {
    match action {
        ActionKind::CrashDumps => RecoveryClass::Reversible,
        ActionKind::HuggingfacePrune | ActionKind::NpmCache => RecoveryClass::Redownloadable,
        ActionKind::Prefetch | ActionKind::GenericDirectory => RecoveryClass::Rebuildable,
        ActionKind::UserTemp
        | ActionKind::SystemTemp
        | ActionKind::RecycleBin
        | ActionKind::DockerPrune => RecoveryClass::Irreversible,
    }
}

pub fn recovery_class_for_finding(finding: &Finding) -> RecoveryClass {
    match finding.action_kind {
        Some(action) => recovery_class_for_action(action),
        None if finding.risk_class == RiskClass::Protected => RecoveryClass::Protected,
        None if finding.rule_id.starts_with("project.") => RecoveryClass::Rebuildable,
        None if finding.risk_class == RiskClass::RebuildOrRedownload => {
            RecoveryClass::Redownloadable
        }
        None => RecoveryClass::Irreversible,
    }
}

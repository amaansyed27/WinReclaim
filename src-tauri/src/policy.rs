use crate::domain::{ActionKind, Finding, RecoveryClass, RiskClass};
use std::time::Duration;

pub const TEMP_MINIMUM_AGE_DAYS: u64 = 7;
pub const VAULT_RETENTION_DAYS: i64 = 7;
pub const SNAPSHOT_LIMIT: usize = 40;
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 2;

pub fn temp_minimum_age() -> Duration {
    Duration::from_secs(TEMP_MINIMUM_AGE_DAYS * 24 * 60 * 60)
}

pub fn recovery_class_for_action(action: ActionKind) -> RecoveryClass {
    match action {
        ActionKind::UserTemp | ActionKind::CrashDumps => RecoveryClass::Reversible,
        ActionKind::HuggingfacePrune | ActionKind::NpmCache => RecoveryClass::Redownloadable,
        ActionKind::SystemTemp | ActionKind::RecycleBin | ActionKind::DockerPrune => {
            RecoveryClass::Irreversible
        }
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

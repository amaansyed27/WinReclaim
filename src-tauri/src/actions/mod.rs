mod external;
mod filesystem;

use crate::domain::{ActionKind, ActionResult, CleanupPlanItem, RecoveryClass};
use crate::scanner::directory_size;
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use uuid::Uuid;

pub fn execute_item(item: &CleanupPlanItem, receipt_id: Uuid) -> ActionResult {
    let target = Path::new(&item.path);
    let cancel = AtomicBool::new(false);
    let measured_before = directory_size(target, &cancel).bytes;
    let recovery_class = recovery_class(item.action_kind);

    let outcome: Result<(u64, u64, String, Vec<Uuid>)> = match item.action_kind {
        ActionKind::UserTemp => filesystem::quarantine_user_temp(
            target,
            receipt_id,
            item.finding_id,
            &item.display_name,
        )
        .map(|outcome| {
            (
                outcome.affected_entries,
                outcome.skipped_entries,
                outcome.message,
                outcome.vault_entry_ids,
            )
        }),
        ActionKind::CrashDumps => filesystem::quarantine_crash_dumps(
            target,
            receipt_id,
            item.finding_id,
            &item.display_name,
        )
        .map(|outcome| {
            (
                outcome.affected_entries,
                outcome.skipped_entries,
                outcome.message,
                outcome.vault_entry_ids,
            )
        }),
        ActionKind::HuggingfacePrune => external::prune_huggingface()
            .map(|(deleted, skipped, message)| (deleted, skipped, message, Vec::new())),
        ActionKind::NpmCache => external::clean_npm_cache()
            .map(|(deleted, skipped, message)| (deleted, skipped, message, Vec::new())),
        ActionKind::DockerPrune => external::prune_docker()
            .map(|(deleted, skipped, message)| (deleted, skipped, message, Vec::new())),
    };
    let measured_after = directory_size(target, &cancel).bytes;

    match outcome {
        Ok((deleted, skipped, message, vault_entry_ids)) => ActionResult {
            finding_id: item.finding_id,
            display_name: item.display_name.clone(),
            estimated_bytes: item.estimated_bytes,
            measured_bytes_before: measured_before,
            measured_bytes_after: measured_after,
            deleted_entries: deleted,
            skipped_entries: skipped,
            success: true,
            message,
            recovery_class,
            vault_entry_ids,
        },
        Err(error) => ActionResult {
            finding_id: item.finding_id,
            display_name: item.display_name.clone(),
            estimated_bytes: item.estimated_bytes,
            measured_bytes_before: measured_before,
            measured_bytes_after: measured_after,
            deleted_entries: 0,
            skipped_entries: 0,
            success: false,
            message: error.to_string(),
            recovery_class,
            vault_entry_ids: Vec::new(),
        },
    }
}

fn recovery_class(action: ActionKind) -> RecoveryClass {
    match action {
        ActionKind::UserTemp | ActionKind::CrashDumps => RecoveryClass::Reversible,
        ActionKind::HuggingfacePrune | ActionKind::NpmCache => RecoveryClass::Redownloadable,
        ActionKind::DockerPrune => RecoveryClass::Irreversible,
    }
}

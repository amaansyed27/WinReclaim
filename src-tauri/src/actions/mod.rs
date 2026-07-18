mod external;
mod filesystem;

use crate::domain::{ActionKind, ActionResult, CleanupPlanItem};
use crate::scanner::directory_size;
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub fn execute_item(item: &CleanupPlanItem) -> ActionResult {
    let target = Path::new(&item.path);
    let cancel = AtomicBool::new(false);
    let measured_before = directory_size(target, &cancel).bytes;
    let outcome: Result<(u64, u64, String)> = match item.action_kind {
        ActionKind::UserTemp => filesystem::clean_user_temp(target),
        ActionKind::CrashDumps => filesystem::clean_crash_dumps(target),
        ActionKind::HuggingfacePrune => external::prune_huggingface(),
        ActionKind::NpmCache => external::clean_npm_cache(),
        ActionKind::DockerPrune => external::prune_docker(),
    };
    let measured_after = directory_size(target, &cancel).bytes;
    match outcome {
        Ok((deleted, skipped, message)) => ActionResult {
            finding_id: item.finding_id,
            display_name: item.display_name.clone(),
            estimated_bytes: item.estimated_bytes,
            measured_bytes_before: measured_before,
            measured_bytes_after: measured_after,
            deleted_entries: deleted,
            skipped_entries: skipped,
            success: true,
            message,
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
        },
    }
}

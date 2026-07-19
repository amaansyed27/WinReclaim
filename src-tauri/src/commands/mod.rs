use crate::actions::execute_item;
use crate::domain::{
    AiStatus, CleanupPlan, CleanupReceipt, CreatePlanRequest, ExecutePlanRequest, IntentRequest,
    IntentSuggestion, ReclaimPassport, RestoreRequest, RestoreResult, ScanReport, ScanRequest,
    StorageTimeline, VaultEntry,
};
use crate::insights::{build_reclaim_passports, build_storage_timeline, persist_scan_snapshot};
use crate::intent::{ai_status, interpret_intent};
use crate::planner::{build_plan, verify_plan_hash};
use crate::platform::windows::disk_snapshot;
use crate::receipts::persist_receipt;
use crate::scanner::scan_profile;
use crate::storage::AppState;
use crate::vault::{list_entries, restore_entry};
use chrono::Utc;
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};
use uuid::Uuid;

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ScanRequest,
) -> Result<ScanReport, String> {
    let app_state = state.inner().clone();
    let scan_state = app_state.clone();

    let mut result =
        tauri::async_runtime::spawn_blocking(move || scan_profile(&app, &scan_state, request))
            .await
            .map_err(|error| format!("Scan task failed: {error}"))?
            .map_err(|error| error.to_string())?;

    if let Err(error) = persist_scan_snapshot(&result) {
        result.errors.push(format!(
            "Storage timeline snapshot could not be saved: {error}"
        ));
    }

    *app_state
        .latest_scan
        .lock()
        .map_err(|_| "Scan state is unavailable".to_string())? = Some(result.clone());

    Ok(result)
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) {
    state.cancel_scan.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn get_ai_status() -> AiStatus {
    ai_status()
}

#[tauri::command]
pub async fn interpret_cleanup_intent(
    state: State<'_, AppState>,
    request: IntentRequest,
) -> Result<IntentSuggestion, String> {
    let report = state
        .latest_scan
        .lock()
        .map_err(|_| "Scan state is unavailable".to_string())?
        .clone()
        .ok_or_else(|| "Run a scan before using reclaim-by-intent".to_string())?;

    tauri::async_runtime::spawn_blocking(move || interpret_intent(&report, request))
        .await
        .map_err(|error| format!("Intent task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_cleanup_plan(
    state: State<'_, AppState>,
    request: CreatePlanRequest,
) -> Result<CleanupPlan, String> {
    let report = state
        .latest_scan
        .lock()
        .map_err(|_| "Scan state is unavailable".to_string())?
        .clone()
        .ok_or_else(|| "Run a scan before creating a cleanup plan".to_string())?;

    let plan = build_plan(&report, &request).map_err(|error| error.to_string())?;
    state
        .plans
        .lock()
        .map_err(|_| "Plan storage is unavailable".to_string())?
        .insert(plan.id, plan.clone());

    Ok(plan)
}

#[tauri::command]
pub async fn execute_cleanup_plan(
    state: State<'_, AppState>,
    request: ExecutePlanRequest,
) -> Result<CleanupReceipt, String> {
    let app_state = state.inner().clone();
    let plan = app_state
        .plans
        .lock()
        .map_err(|_| "Plan storage is unavailable".to_string())?
        .get(&request.plan_id)
        .cloned()
        .ok_or_else(|| "Cleanup plan was not found".to_string())?;

    if request.plan_hash != plan.plan_hash
        || !verify_plan_hash(&plan).map_err(|error| error.to_string())?
    {
        return Err("Cleanup plan verification failed; rescan and rebuild the plan".to_string());
    }

    let latest_scan = app_state
        .latest_scan
        .lock()
        .map_err(|_| "Scan state is unavailable".to_string())?
        .clone()
        .ok_or_else(|| "The source scan is unavailable".to_string())?;

    if latest_scan.scan_id != plan.scan_id {
        return Err("The cleanup plan does not match the current scan".to_string());
    }

    let receipt = tauri::async_runtime::spawn_blocking(move || {
        let receipt_id = Uuid::new_v4();
        let started_at = Utc::now();
        let disk_before =
            disk_snapshot(Path::new(&latest_scan.root)).map_err(|error| error.to_string())?;
        let results = plan
            .items
            .iter()
            .map(|item| execute_item(item, receipt_id))
            .collect::<Vec<_>>();
        let disk_after =
            disk_snapshot(Path::new(&latest_scan.root)).map_err(|error| error.to_string())?;
        let vault_entry_ids = results
            .iter()
            .flat_map(|result| result.vault_entry_ids.iter().copied())
            .collect::<Vec<_>>();

        let receipt = CleanupReceipt {
            id: receipt_id,
            plan_id: plan.id,
            plan_hash: plan.plan_hash.clone(),
            started_at,
            completed_at: Utc::now(),
            disk_free_before: disk_before.free_bytes,
            disk_free_after: disk_after.free_bytes,
            actual_reclaimed_bytes: disk_after.free_bytes.saturating_sub(disk_before.free_bytes),
            estimated_reclaim_bytes: plan.estimated_reclaim_bytes,
            results,
            vault_entry_ids,
            protected_summary: vec![
                "Prefetch".into(),
                "Browser profiles".into(),
                "Ollama models".into(),
                "Docker volumes".into(),
                "Android emulators".into(),
                "Windows directories".into(),
                "Project source".into(),
            ],
            rule_set_version: plan.rule_set_version.clone(),
        };

        persist_receipt(&receipt).map_err(|error| {
            format!("Cleanup completed but receipt persistence failed: {error}")
        })?;
        Ok::<CleanupReceipt, String>(receipt)
    })
    .await
    .map_err(|error| format!("Cleanup task failed: {error}"))??;

    app_state
        .receipts
        .lock()
        .map_err(|_| "Receipt storage is unavailable".to_string())?
        .push(receipt.clone());

    Ok(receipt)
}

#[tauri::command]
pub fn list_receipts(state: State<'_, AppState>) -> Result<Vec<CleanupReceipt>, String> {
    Ok(state
        .receipts
        .lock()
        .map_err(|_| "Receipt storage is unavailable".to_string())?
        .clone())
}

#[tauri::command]
pub fn get_storage_timeline() -> Result<StorageTimeline, String> {
    build_storage_timeline().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_reclaim_passports(
    state: State<'_, AppState>,
    scan_id: Uuid,
) -> Result<Vec<ReclaimPassport>, String> {
    let report = state
        .latest_scan
        .lock()
        .map_err(|_| "Scan state is unavailable".to_string())?
        .clone()
        .ok_or_else(|| "Run a scan before requesting reclaim passports".to_string())?;
    if report.scan_id != scan_id {
        return Err("The requested passport scan is no longer current".to_string());
    }
    Ok(build_reclaim_passports(&report))
}

#[tauri::command]
pub fn list_vault_entries() -> Result<Vec<VaultEntry>, String> {
    list_entries().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn restore_vault_entry(request: RestoreRequest) -> Result<RestoreResult, String> {
    restore_entry(request.vault_entry_id).map_err(|error| error.to_string())
}

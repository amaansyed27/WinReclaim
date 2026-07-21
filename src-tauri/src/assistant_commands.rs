use crate::assistant;
use crate::domain::{AnalyzeStorageRequest, StorageAssistantReport, StorageAssistantStatus};
use crate::storage::AppState;
use std::sync::atomic::Ordering;
use tauri::State;

#[tauri::command]
pub fn get_storage_assistant_status(state: State<'_, AppState>) -> StorageAssistantStatus {
    assistant::status(state.assistant_busy.load(Ordering::Relaxed))
}

#[tauri::command]
pub async fn analyze_storage_report(
    state: State<'_, AppState>,
    request: AnalyzeStorageRequest,
) -> Result<StorageAssistantReport, String> {
    let status = assistant::status(false);
    if !status.available {
        return Err("The WinReclaim cloud assistant endpoint is unavailable".to_string());
    }

    let report = state
        .latest_scan
        .lock()
        .map_err(|_| "Scan state is unavailable".to_string())?
        .clone()
        .ok_or_else(|| "Run a scan before using the Storage Assistant".to_string())?;
    if report.scan_id != request.scan_id {
        return Err("The requested scan is no longer current".to_string());
    }

    let busy = state.assistant_busy.clone();
    begin(&busy)?;
    let task = tauri::async_runtime::spawn_blocking(move || assistant::analyze(&report)).await;
    busy.store(false, Ordering::Release);
    task.map_err(|error| format!("Storage Assistant task failed: {error}"))?
        .map_err(|error| error.to_string())
}

fn begin(busy: &std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<(), String> {
    busy.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .map(|_| ())
        .map_err(|_| "The Storage Assistant is already busy".to_string())
}

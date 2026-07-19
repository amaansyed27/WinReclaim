mod actions;
mod commands;
mod domain;
mod insights;
mod intent;
mod planner;
mod platform;
mod receipts;
mod rules;
mod scanner;
mod storage;
mod vault;

use storage::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::start_scan,
            commands::cancel_scan,
            commands::get_ai_status,
            commands::interpret_cleanup_intent,
            commands::create_cleanup_plan,
            commands::execute_cleanup_plan,
            commands::list_receipts,
            commands::get_storage_timeline,
            commands::get_reclaim_passports,
            commands::list_vault_entries,
            commands::restore_vault_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running WinReclaim");
}

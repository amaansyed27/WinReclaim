mod actions;
mod app_data;
mod assistant;
mod assistant_commands;
mod cloud;
mod commands;
mod domain;
mod insights;
mod intent;
mod planner;
mod platform;
mod policy;
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
            app_data::initialize().map_err(|error| std::io::Error::other(error.to_string()))?;
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::start_scan,
            commands::cancel_scan,
            commands::list_storage_drives,
            commands::get_ai_status,
            commands::interpret_cleanup_intent,
            commands::create_cleanup_plan,
            commands::execute_cleanup_plan,
            commands::list_receipts,
            commands::get_storage_timeline,
            commands::get_reclaim_passports,
            commands::list_vault_entries,
            commands::restore_vault_entry,
            commands::get_app_data_summary,
            commands::clear_scan_history,
            commands::clear_cleanup_records,
            commands::reset_app_data,
            assistant_commands::get_storage_assistant_status,
            assistant_commands::analyze_storage_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running WinReclaim");
}

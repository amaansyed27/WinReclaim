mod actions;
mod commands;
mod domain;
mod intent;
mod planner;
mod platform;
mod receipts;
mod rules;
mod scanner;
mod storage;

use storage::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::start_scan,
            commands::cancel_scan,
            commands::get_ai_status,
            commands::interpret_cleanup_intent,
            commands::create_cleanup_plan,
            commands::execute_cleanup_plan,
            commands::list_receipts
        ])
        .run(tauri::generate_context!())
        .expect("error while running WinReclaim");
}

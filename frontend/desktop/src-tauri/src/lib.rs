pub mod commands;
pub mod hive_client;
pub mod models;
pub mod prompts;
pub mod storage;

pub fn run() {
    tauri::Builder::default()
        .manage(commands::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::hive_api::init_hive_client,
            commands::hive_api::hive_api_status,
            commands::hive_api::hive_api_wait_ready,
            commands::hive_api::hive_api_providers,
            commands::hive_api::hive_api_drones,
            commands::hive_api::hive_api_cli_versions,
            commands::hive_api::hive_api_check_cli_version,
            commands::hive_api::hive_api_update_cli,
            commands::hive_api::start_hive_api,
            commands::hive_api::stop_hive_api,
            commands::hive_api::hive_api_process_running,
            commands::hive_monitor::start_hive_monitor,
            commands::hive_monitor::stop_hive_monitor,
            commands::templates::list_templates,
            commands::templates::get_template,
            commands::templates::create_template,
            commands::templates::update_template,
            commands::templates::delete_template,
            commands::templates::clone_template,
            commands::templates::enhance_prompt,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

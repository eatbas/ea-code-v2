use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

use crate::commands::AppState;
use crate::hive_client::health::HealthResponse;
use crate::hive_client::lifecycle::HiveProcess;
use crate::hive_client::providers::{DroneInfo, ProviderInfo};
use crate::hive_client::versions::CliVersionInfo;
use crate::hive_client::HiveClient;

/// Initialise the hive-api client with host/port from settings.
/// Does NOT start the hive-api process (that is managed externally or by the sidecar).
#[tauri::command]
pub async fn init_hive_client(
    host: String,
    port: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = HiveClient::new(&host, port);
    let mut guard = state.hive_client.lock().await;
    *guard = Some(client);
    Ok(())
}

/// Check if hive-api is healthy and drones are booted.
#[tauri::command]
pub async fn hive_api_status(
    state: State<'_, AppState>,
) -> Result<HealthResponse, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    client.check_health().await
}

/// Wait for hive-api to become ready (poll health endpoint).
/// Emits "hive-api:ready" event when ready.
#[tauri::command]
pub async fn hive_api_wait_ready(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<HealthResponse, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    let health = client.wait_until_ready(2000, 30000).await?;
    let _ = app.emit("hive-api:ready", &health);
    Ok(health)
}

/// List available providers from hive-api.
#[tauri::command]
pub async fn hive_api_providers(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderInfo>, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    client.list_providers().await
}

/// List active drones from hive-api.
#[tauri::command]
pub async fn hive_api_drones(
    state: State<'_, AppState>,
) -> Result<Vec<DroneInfo>, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    client.list_drones().await
}

/// Get CLI version information for all providers.
#[tauri::command]
pub async fn hive_api_cli_versions(
    state: State<'_, AppState>,
) -> Result<Vec<CliVersionInfo>, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    client.get_cli_versions().await
}

/// Trigger a CLI version check for a specific provider.
#[tauri::command]
pub async fn hive_api_check_cli_version(
    provider: String,
    state: State<'_, AppState>,
) -> Result<CliVersionInfo, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    client.check_cli_version(&provider).await
}

/// Trigger a CLI update for a specific provider.
#[tauri::command]
pub async fn hive_api_update_cli(
    provider: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    client.update_cli(&provider).await
}

/// Start the hive-api process and initialise the HTTP client.
#[tauri::command]
pub async fn start_hive_api(
    entry_path: String,
    host: String,
    port: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Start the process
    let process = HiveProcess::new(PathBuf::from(&entry_path), port);
    process.start().await?;

    let mut proc_guard = state.hive_process.lock().await;
    *proc_guard = Some(process);

    // Initialise the HTTP client so subsequent commands work
    let client = HiveClient::new(&host, port);
    let mut client_guard = state.hive_client.lock().await;
    *client_guard = Some(client);

    Ok(())
}

/// Stop the hive-api process.
#[tauri::command]
pub async fn stop_hive_api(state: State<'_, AppState>) -> Result<(), String> {
    let mut proc_guard = state.hive_process.lock().await;
    match proc_guard.take() {
        Some(process) => process.stop().await,
        None => Err("hive-api process is not running".to_string()),
    }
}

/// Check if the hive-api process is still running.
#[tauri::command]
pub async fn hive_api_process_running(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut guard = state.hive_process.lock().await;
    match guard.as_mut() {
        Some(process) => Ok(process.is_running().await),
        None => Ok(false),
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::commands::AppState;

/// Start a background health monitor that periodically pings hive-api.
/// Emits `hive-api:disconnected` if the health check fails after being ready,
/// and `hive-api:ready` when connectivity is restored.
///
/// Runs until the app is closed or `stop_hive_monitor` is called.
#[tauri::command]
pub async fn start_hive_monitor(
    app: AppHandle,
    state: State<'_, AppState>,
    poll_interval_secs: Option<u64>,
) -> Result<(), String> {
    let interval = poll_interval_secs.unwrap_or(60);
    let monitor_active = state.monitor_active.clone();

    // Prevent double-start
    if monitor_active.load(Ordering::SeqCst) {
        return Err("Health monitor is already running".to_string());
    }
    monitor_active.store(true, Ordering::SeqCst);

    let hive_client = state.hive_client.clone();
    let monitor_flag = monitor_active.clone();

    tokio::spawn(async move {
        let mut was_healthy = true;

        loop {
            if !monitor_flag.load(Ordering::SeqCst) {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

            if !monitor_flag.load(Ordering::SeqCst) {
                break;
            }

            let is_healthy = {
                let guard = hive_client.lock().await;
                match guard.as_ref() {
                    Some(client) => client.check_health().await.is_ok(),
                    None => false,
                }
            };

            if was_healthy && !is_healthy {
                // Transition: healthy → unhealthy
                let _ = app.emit("hive-api:disconnected", "Health check failed");
            } else if !was_healthy && is_healthy {
                // Transition: unhealthy → healthy (auto-recovery)
                let _ = app.emit("hive-api:reconnected", "Health check restored");
            }

            was_healthy = is_healthy;
        }
    });

    Ok(())
}

/// Stop the background health monitor.
#[tauri::command]
pub async fn stop_hive_monitor(state: State<'_, AppState>) -> Result<(), String> {
    state.monitor_active.store(false, Ordering::SeqCst);
    Ok(())
}

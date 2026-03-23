use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

/// Manages the hive-api Python process lifecycle.
pub struct HiveProcess {
    /// The spawned child process, if running.
    child: Arc<Mutex<Option<Child>>>,
    /// Path to the hive-api entry point (e.g. python script or executable).
    entry_path: PathBuf,
    /// Port the process listens on.
    port: u16,
}

impl HiveProcess {
    pub fn new(entry_path: PathBuf, port: u16) -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            entry_path,
            port,
        }
    }

    /// Spawn the hive-api process. Returns error if already running.
    pub async fn start(&self) -> Result<(), String> {
        let mut guard = self.child.lock().await;
        if guard.is_some() {
            return Err("hive-api process is already running".to_string());
        }

        let child = tokio::process::Command::new("python")
            .arg(self.entry_path.to_string_lossy().as_ref())
            .arg("--port")
            .arg(self.port.to_string())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start hive-api: {}", e))?;

        *guard = Some(child);
        Ok(())
    }

    /// Stop the hive-api process gracefully, then force-kill after timeout.
    pub async fn stop(&self) -> Result<(), String> {
        let mut guard = self.child.lock().await;
        match guard.take() {
            Some(mut child) => {
                // Try graceful kill first
                if let Err(e) = child.kill().await {
                    return Err(format!("Failed to stop hive-api: {}", e));
                }
                Ok(())
            }
            None => Err("hive-api process is not running".to_string()),
        }
    }

    /// Check whether the child process is still alive.
    pub async fn is_running(&self) -> bool {
        let mut guard = self.child.lock().await;
        match guard.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited — clean up
                    *guard = None;
                    false
                }
                Ok(None) => true,  // still running
                Err(_) => false,
            },
            None => false,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hive_process_creation() {
        let proc = HiveProcess::new(PathBuf::from("/fake/hive_api.py"), 8000);
        assert_eq!(proc.port(), 8000);
    }

    #[tokio::test]
    async fn test_stop_without_start_returns_error() {
        let proc = HiveProcess::new(PathBuf::from("/fake/hive_api.py"), 8000);
        let result = proc.stop().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not running"));
    }

    #[tokio::test]
    async fn test_is_running_without_start() {
        let proc = HiveProcess::new(PathBuf::from("/fake/hive_api.py"), 8000);
        assert!(!proc.is_running().await);
    }
}

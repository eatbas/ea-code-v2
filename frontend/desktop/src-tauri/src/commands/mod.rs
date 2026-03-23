pub mod hive_api;
pub mod hive_monitor;
pub mod pipeline;
pub mod templates;

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

use crate::hive_client::lifecycle::HiveProcess;
use crate::hive_client::HiveClient;

/// Shared application state accessible from all Tauri commands.
pub struct AppState {
    /// Global cancellation flag — set to true to stop the current pipeline.
    pub cancel_flag: Arc<AtomicBool>,

    /// Global pause flag — set to true to pause pipeline execution.
    pub pause_flag: Arc<AtomicBool>,

    /// One-shot channels for answering mid-run questions.
    /// Key: question_id, Value: sender to deliver the answer.
    pub question_answers: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,

    /// hive-api HTTP client.
    pub hive_client: Arc<Mutex<Option<HiveClient>>>,

    /// hive-api child process manager.
    pub hive_process: Arc<Mutex<Option<HiveProcess>>>,

    /// Currently active job_id (for cancellation).
    pub active_job_id: Arc<Mutex<Option<String>>>,

    /// Currently active run_id.
    pub active_run_id: Arc<Mutex<Option<String>>>,

    /// Ensures a single active pipeline run at a time.
    pub run_active: Arc<AtomicBool>,

    /// Background health monitor active flag.
    pub monitor_active: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            pause_flag: Arc::new(AtomicBool::new(false)),
            question_answers: Arc::new(Mutex::new(HashMap::new())),
            hive_client: Arc::new(Mutex::new(None)),
            hive_process: Arc::new(Mutex::new(None)),
            active_job_id: Arc::new(Mutex::new(None)),
            active_run_id: Arc::new(Mutex::new(None)),
            run_active: Arc::new(AtomicBool::new(false)),
            monitor_active: Arc::new(AtomicBool::new(false)),
        }
    }
}

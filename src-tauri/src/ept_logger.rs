use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Clone)]
pub struct EPTLogger {
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl EPTLogger {
    pub fn new() -> Self {
        Self {
            app_handle: Arc::new(Mutex::new(None)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_app_handle(&self, app_handle: tauri::AppHandle) {
        if let Ok(mut handle) = self.app_handle.lock() {
            *handle = Some(app_handle);
        }
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();
        let entry = LogEntry {
            level: level.to_string(),
            message: message.to_string(),
            timestamp,
        };

        // Store log entry
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(entry.clone());
        }

        // Emit to frontend
        if let Ok(handle) = self.app_handle.lock() {
            if let Some(app) = handle.as_ref() {
                let _ = app.emit("log-entry", &entry);
            }
        }

        // Also print to console
        println!("[{}] {}: {}", entry.timestamp, level, message);
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn warning(&self, message: &str) {
        self.log("WARNING", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.lock().unwrap().clone()
    }
}

impl Default for EPTLogger {
    fn default() -> Self {
        Self::new()
    }
}


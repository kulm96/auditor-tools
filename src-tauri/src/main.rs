#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Minimal Tauri 2 entrypoint for the auditor-tools shell.
// Integrates File Conversion functionality from the legacy application.

mod file_conversion_adapter;
mod ept_logger;
mod report_model;
mod decompression_engine;
mod process_controller;
mod hashing_service;
mod conversion_engine;
mod llm_export_engine;
mod report_writer;
mod file_scanner;

use file_conversion_adapter::FileConversionResult;
use ept_logger::{EPTLogger, LogEntry};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Builder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub current: usize,
    pub total: usize,
    pub task_category: String,
}

// Global state for the logger
pub struct AppState {
    pub logger: EPTLogger,
    pub app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

#[tauri::command]
fn ping() -> String {
    "auditor-tools shell is running".to_string()
}

#[tauri::command]
fn start_file_conversion(input_path: String, state: tauri::State<'_, AppState>) -> Result<FileConversionResult, String> {
    file_conversion_adapter::start_file_conversion(input_path, &state)
}

#[tauri::command]
fn get_logs(state: tauri::State<'_, AppState>) -> Vec<LogEntry> {
    state.logger.get_logs()
}

fn main() {
    let logger = EPTLogger::new();
    let logger_clone = logger.clone();
    let app_handle = Arc::new(Mutex::new(None));
    let app_handle_clone = app_handle.clone();
    
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            logger: logger_clone,
            app_handle: app_handle_clone,
        })
        .setup(move |app| {
            logger.set_app_handle(app.handle().clone());
            if let Ok(mut handle) = app_handle.lock() {
                *handle = Some(app.handle().clone());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            start_file_conversion,
            get_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running auditor-tools application");
}

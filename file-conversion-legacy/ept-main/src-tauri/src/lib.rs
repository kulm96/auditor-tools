mod ept_logger;
mod report_model;
mod decompression_engine;
mod process_controller;
mod hashing_service;
mod conversion_engine;
mod llm_export_engine;
mod report_writer;
mod file_scanner;

use conversion_engine::ConversionEngine;
use ept_logger::{EPTLogger, LogEntry};
use process_controller::{ProcessController, ProcessingResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub current: usize,
    pub total: usize,
    pub task_category: String,
}

// Global state for the logger and process controller
struct AppState {
    logger: EPTLogger,
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

#[tauri::command]
async fn check_libreoffice(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let engine = ConversionEngine::new(state.logger.clone());
    match engine.find_libreoffice() {
        Ok(_) => Ok(true),
        Err(e) => {
            state.logger.error(&format!("LibreOffice check failed: {}", e));
            Ok(false)
        }
    }
}

#[tauri::command]
fn handle_user_input(path: String) -> Result<String, String> {
    // Normalize and validate path
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    Ok(path)
}

#[tauri::command]
async fn start_processing(
    input_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessingResult, String> {
    let path = PathBuf::from(&input_path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", input_path));
    }
    
    state.logger.info(&format!("Starting processing for: {}", input_path));
    
    let app_handle = {
        if let Ok(handle) = state.app_handle.lock() {
            handle.clone()
        } else {
            return Err("Failed to get app handle".to_string());
        }
    };
    
    let app_handle_clone = app_handle.ok_or("App handle not initialized".to_string())?;
    let mut controller = ProcessController::new(state.logger.clone(), app_handle_clone);
    
    match controller.start_processing(&path) {
        Ok(result) => {
            state.logger.info("Processing completed successfully");
            Ok(result)
        }
        Err(e) => {
            // detailed chain of errors
            let error_chain = e.chain()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            
            let error_msg = format!("Processing failed: {}", error_chain);
            state.logger.error(&error_msg);
            Err(error_msg)
        }
    }
}

/// SECURITY: Validate path before opening to prevent command injection
fn validate_path_for_opening(path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);
    
    // Ensure path exists
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    
    // SECURITY: Canonicalize path to resolve symlinks and normalize
    let canonical = path_buf.canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    
    // SECURITY: Validate path doesn't contain suspicious characters that could be used for injection
    // While Rust's Command::arg() provides protection against shell injection, we validate here as defense-in-depth
    let path_str = canonical.to_string_lossy();
    if path_str.contains('\0') || path_str.contains('\n') || path_str.contains('\r') {
        return Err("Path contains invalid characters".to_string());
    }
    
    Ok(canonical)
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    // SECURITY: Validate and canonicalize path before opening
    let validated_path = validate_path_for_opening(&path)?;
    let path_str = validated_path.to_string_lossy().to_string();
    
    // SECURITY: Use Command::arg() which provides protection against command injection
    // The path is validated and canonicalized, and Command::arg() treats arguments as literal strings
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn open_file(path: String) -> Result<(), String> {
    // SECURITY: Validate and canonicalize path before opening
    let validated_path = validate_path_for_opening(&path)?;
    let path_str = validated_path.to_string_lossy().to_string();
    
    // SECURITY: Use Command::arg() which provides protection against command injection
    // The path is validated and canonicalized, and Command::arg() treats arguments as literal strings
    #[cfg(target_os = "windows")]
    {
        // Use explorer for files on Windows (more secure than cmd /C start)
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
fn quit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
fn get_logs(state: tauri::State<'_, AppState>) -> Vec<LogEntry> {
    state.logger.get_logs()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let logger = EPTLogger::new();
    
    let logger_clone = logger.clone();
    let app_handle = Arc::new(Mutex::new(None));
    let app_handle_clone = app_handle.clone();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            handle_user_input,
            start_processing,
            get_logs,
            open_folder,
            open_file,
            check_libreoffice,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

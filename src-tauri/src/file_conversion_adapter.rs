use crate::process_controller::{ProcessController, ProcessingResult};
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

// Import AppState from main module
use crate::AppState;

/// Result shape returned to the frontend for a File Conversion request.
#[derive(Serialize)]
pub struct FileConversionResult {
    pub status: String,
    pub staging_path: Option<String>,
    pub llm_output_path: Option<String>,
    pub report_path: Option<String>,
}

/// Adapter entrypoint for File Conversion.
///
/// This adapter delegates to the ProcessController which handles the full
/// file conversion pipeline: decompression, scanning, conversion, hashing,
/// and report generation.
pub fn start_file_conversion(
    input_path: String,
    state: &State<'_, AppState>,
) -> Result<FileConversionResult, String> {
    if input_path.trim().is_empty() {
        return Err("Input path must not be empty.".to_string());
    }

    let path = PathBuf::from(&input_path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", input_path));
    }
    
    state.logger.info(&format!("Starting conversion for: {}", input_path));
    
    // Get app handle for ProcessController
    let app_handle_clone = {
        if let Ok(handle) = state.app_handle.lock() {
            handle.clone()
        } else {
            return Err("Failed to get app handle".to_string());
        }
    };
    
    let app_handle_for_controller = app_handle_clone.ok_or("App handle not initialized".to_string())?;
    let mut controller = ProcessController::new(state.logger.clone(), app_handle_for_controller);
    
    match controller.start_processing(&path) {
        Ok(ProcessingResult {
            staging_path,
            llm_output_path,
            report_path,
            ..
        }) => {
            state.logger.info("Conversion request completed.");
            Ok(FileConversionResult {
                status: "completed".to_string(),
                staging_path: Some(staging_path),
                llm_output_path: Some(llm_output_path),
                report_path: Some(report_path),
            })
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



#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Minimal Tauri 2 entrypoint for the auditor-tools shell.
// Integrates a stubbed File Conversion adapter per PRD1; the adapter delegates
// to the legacy File Conversion engine without duplicating its core logic.

mod file_conversion_adapter;

use file_conversion_adapter::FileConversionResult;
use tauri::Builder;

#[tauri::command]
fn ping() -> String {
    "auditor-tools shell is running".to_string()
}

#[tauri::command]
fn start_file_conversion(input_path: String) -> Result<FileConversionResult, String> {
    file_conversion_adapter::start_file_conversion(input_path)
}

fn main() {
    Builder::default()
        .invoke_handler(tauri::generate_handler![
            ping,
            start_file_conversion
        ])
        .run(tauri::generate_context!())
        .expect("error while running auditor-tools application");
}

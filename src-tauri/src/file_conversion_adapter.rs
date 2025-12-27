use serde::Serialize;

/// Result shape returned to the frontend for a File Conversion request.
#[derive(Serialize)]
pub struct FileConversionResult {
    pub status: String,
}

/// Adapter entrypoint for File Conversion.
///
/// PRD1 requirement:
/// - This adapter is responsible for delegating to the existing File Conversion
///   logic from the legacy application without duplicating that logic.
/// - For now, it acts as a thin placeholder that validates input and returns a
///   stub response, and will be extended in a later step to call into the
///   legacy engine.
pub fn start_file_conversion(input_path: String) -> Result<FileConversionResult, String> {
    if input_path.trim().is_empty() {
        return Err("Input path must not be empty.".to_string());
    }

    let result = FileConversionResult {
        status: format!("Conversion requested for: {}", input_path),
    };

    Ok(result)
}



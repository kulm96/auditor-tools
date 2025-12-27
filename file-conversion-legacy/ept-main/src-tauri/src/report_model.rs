use serde::{Deserialize, Serialize};

use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportModel {
    pub file_name: String,
    pub sha512: Option<String>, // None in Phase 1
    pub processed: String, // "Yes" or "No"
    pub skip_reason: Option<String>,
    pub relative_path: String,
    pub file_type: String,
    pub file_size_bytes: u64,
    pub file_size_human: String,
    pub last_modified: String,
    pub created_time: String,
}

impl ReportModel {
    pub fn new(
        file_name: String,
        relative_path: String,
        file_type: String,
        file_size_bytes: u64,
        last_modified: String,
        created_time: String,
    ) -> Self {
        let file_size_human = format_file_size(file_size_bytes);
        Self {
            file_name,
            sha512: None,
            processed: "No".to_string(),
            skip_reason: None,
            relative_path,
            file_type,
            file_size_bytes,
            file_size_human,
            last_modified,
            created_time,
        }
    }

    pub fn is_llm_readable(file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            matches!(
                ext_lower.as_str(),
                "txt" | "md" | "pdf" | "csv" | "json" | "xml" | "html" | "htm" | "log" | "rtf"
            )
        } else {
            false
        }
    }
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}


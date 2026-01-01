use crate::ept_logger::EPTLogger;
use crate::report_model::ReportModel;
use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct FileScanner {
    logger: Option<EPTLogger>,
}

impl FileScanner {
    pub fn new() -> Self {
        Self { logger: None }
    }

    pub fn with_logger(logger: EPTLogger) -> Self {
        Self { logger: Some(logger) }
    }

    /// Check if a file should be skipped (common system files)
    fn should_skip_file(file_name: &str) -> bool {
        file_name.starts_with("~$") 
            || file_name.starts_with("._") 
            || file_name.starts_with(".DS")
            || file_name == "desktop.ini"
            || file_name == "thumbs.db"
            || file_name == ".DS_Store"
    }

    pub fn scan(root_path: &Path) -> Result<Vec<ReportModel>> {
        Self::new().scan_with_logging(root_path)
    }

    pub fn scan_with_logging(&self, root_path: &Path) -> Result<Vec<ReportModel>> {
        let mut entries = Vec::new();
        let mut file_count = 0;
        
        if let Some(ref logger) = self.logger {
            logger.debug(&format!("Scanning directory: {}", root_path.display()));
        }
        
        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path.is_file() {
                // Get file name first to check if we should skip it
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                // Skip common system files
                if Self::should_skip_file(file_name) {
                    continue;
                }
                if let Ok(metadata) = fs::metadata(path) {
                    let file_name = file_name.to_string();
                    
                    let relative_path = path
                        .strip_prefix(root_path)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    
                    let file_type = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let file_size = metadata.len();
                    
                    let last_modified = metadata
                        .modified()
                        .ok()
                        .and_then(|t| {
                            chrono::DateTime::<chrono::Local>::from(t)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                                .into()
                            })
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let created_time = metadata
                        .created()
                        .ok()
                        .and_then(|t| {
                            chrono::DateTime::<chrono::Local>::from(t)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                                .into()
                            })
                        .unwrap_or_else(|| last_modified.clone());
                    
                    let report_entry = ReportModel::new(
                        file_name,
                        relative_path,
                        file_type,
                        file_size,
                        last_modified,
                        created_time,
                    );
                    
                    entries.push(report_entry);
                    file_count += 1;
                    
                    // Log every 100 files for progress feedback
                    if file_count % 100 == 0 {
                        if let Some(ref logger) = self.logger {
                            logger.debug(&format!("Scanned {} files so far...", file_count));
                        }
                    }
                }
            }
        }
        
        if let Some(ref logger) = self.logger {
            logger.info(&format!("File scan complete: found {} files", entries.len()));
        }
        
        Ok(entries)
    }
}


use crate::conversion_engine::ConversionEngine;
use crate::decompression_engine::DecompressionEngine;
use crate::ept_logger::EPTLogger;
use crate::file_scanner::FileScanner;
use crate::hashing_service::HashingService;
use crate::llm_export_engine::LLMExportEngine;
use crate::report_model::ReportModel;
use crate::report_writer::ReportWriter;
use crate::ProgressUpdate;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub entries: Vec<ReportModel>,
    pub staging_path: String,
    pub llm_output_path: String,
    pub report_path: String,
}

pub struct ProcessController {
    logger: EPTLogger,
    decompression_engine: DecompressionEngine,
    report_entries: Vec<ReportModel>,
    app_handle: tauri::AppHandle,
}

impl ProcessController {
    pub fn new(logger: EPTLogger, app_handle: tauri::AppHandle) -> Self {
        let logger_clone = logger.clone();
        let decompression_engine = DecompressionEngine::new(logger);
        Self {
            logger: logger_clone,
            decompression_engine,
            report_entries: Vec::new(),
            app_handle,
        }
    }
    
    fn emit_progress(&self, current: usize, total: usize, task_category: &str) {
        let update = ProgressUpdate {
            current,
            total,
            task_category: task_category.to_string(),
        };
        let _ = self.app_handle.emit("progress-update", &update);
    }

    pub fn start_processing(&mut self, input_path: &Path) -> Result<ProcessingResult> {
        self.logger.info("Starting processing...");
        self.report_entries.clear();
        
        // 1. Prepare Workspace (Expand ZIP or Copy Folder)
        let working_path = self.prepare_workspace(input_path)
            .context("Failed to prepare workspace")?;
        
        // 2. Recursive Decompression
        self.decompress_archives(&working_path)
            .context("Failed during recursive decompression")?;
        
        // 3. Scan Files
        self.scan_files(&working_path)
            .context("Failed to scan files")?;
        
        let total_files = self.report_entries.len();
        self.logger.info(&format!("Found {} files. Starting conversion and hashing...", total_files));
        
        // 4. Process Files (Hash, Convert)
        // Progress updates are handled inside process_file_entries
        self.process_file_entries(&working_path)
            .context("Failed during file processing loop")?;
        
        // 5. Finalize Output (Export, Report)
        let result = self.finalize_output(&working_path, total_files)
            .context("Failed to finalize output")?;
        
        self.logger.info(&format!(
            "Processing complete. {} files processed. Output: {}",
            self.report_entries.len(),
            result.llm_output_path
        ));
        
        Ok(result)
    }

    fn prepare_workspace(&mut self, input_path: &Path) -> Result<PathBuf> {
        if input_path.is_file() {
            if let Some(ext) = input_path.extension().and_then(|e| e.to_str()) {
                if ext.to_lowercase() == "zip" {
                    self.logger.info(&format!("Input is a ZIP file, expanding: {}", input_path.display()));
                    self.emit_progress(0, 1, "Decompressing zip files");
                    return self.decompression_engine.expand_zip_to_folder(input_path)
                        .with_context(|| format!("Failed to expand zip file: {}", input_path.display()));
                }
            }
        } else if input_path.is_dir() {
            self.logger.info(&format!("Input is a folder, copying to staging folder: {}", input_path.display()));
            self.emit_progress(0, 1, "Preparing staging folder");
            
            let folder_name = input_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("folder");
            
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let staging_folder_name = format!("{}__{}", folder_name, timestamp);
            
            let parent_dir = input_path
                .parent()
                .context("Input folder has no parent directory")?;
            
            let staging_path = parent_dir.join(&staging_folder_name);
            
            self.logger.info(&format!("Copying folder {} to staging folder {}", 
                input_path.display(), staging_path.display()));
            
            self.copy_directory_recursive(input_path, &staging_path)
                .with_context(|| format!("Failed to copy directory from {} to {}", input_path.display(), staging_path.display()))?;
            
            return Ok(staging_path);
        }
        
        // Fallback if not zip or dir (should be handled by caller usually)
        Ok(input_path.to_path_buf())
    }

    fn decompress_archives(&mut self, working_path: &Path) -> Result<()> {
        self.logger.info("Starting recursive decompression...");
        self.emit_progress(0, 1, "Decompressing zip files");
        self.decompression_engine.recursive_decompress(working_path)
            .context("Failed to recursively decompress archives")
    }

    fn scan_files(&mut self, working_path: &Path) -> Result<()> {
        self.logger.info("Scanning and cataloging files...");
        self.emit_progress(0, 0, "Scanning files");
        self.report_entries = FileScanner::scan(working_path)
            .context("File scanner failed")?;
        Ok(())
    }

    fn process_file_entries(&mut self, working_path: &Path) -> Result<()> {
        let hashing_service = HashingService::new();
        let conversion_engine = ConversionEngine::new(self.logger.clone());

        // Canonicalize working path for security validation
        let working_path_canonical = working_path.canonicalize()
            .context("Failed to canonicalize working path")?;
        
        // Collect file paths with their original indices to avoid borrowing issues
        // SECURITY: Filter out entries with path traversal attempts, but track original indices
        let file_paths_with_indices: Vec<_> = self.report_entries
            .iter()
            .enumerate()
            .filter_map(|(orig_idx, entry)| {
                // SECURITY: Safely resolve relative paths and validate they stay within working directory
                self.safe_resolve_path(working_path, &working_path_canonical, &entry.relative_path)
                    .map(|path| (orig_idx, path))
            })
            .collect();
        
        // Mark entries that were filtered out due to security issues
        let valid_indices: std::collections::HashSet<usize> = file_paths_with_indices
            .iter()
            .map(|(idx, _)| *idx)
            .collect();
        for (idx, entry) in self.report_entries.iter_mut().enumerate() {
            if !valid_indices.contains(&idx) {
                entry.processed = "No".to_string();
                entry.skip_reason = Some("Path validation failed - potential path traversal".to_string());
            }
        }
        
        let file_paths: Vec<_> = file_paths_with_indices.iter().map(|(_, path)| path).collect();
        
        // Count files that will actually be converted/processed
        // (convertible files OR LLM-readable files)
        let files_to_convert: Vec<usize> = file_paths
            .iter()
            .enumerate()
            .filter_map(|(idx, file_path)| {
                if !file_path.exists() {
                    return None;
                }
                let is_convertible = conversion_engine.is_convertible_file(file_path);
                let is_llm_readable = ReportModel::is_llm_readable(file_path);
                if is_convertible || is_llm_readable {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        
        let conversion_count = files_to_convert.len();
        self.logger.info(&format!(
            "Found {} files to convert/process out of {} total files",
            conversion_count,
            self.report_entries.len()
        ));
        
        if conversion_count > 0 {
            self.emit_progress(0, conversion_count, "Converting Documents");
        }
        
        let mut processed_count = 0;
        for (file_idx, file_path) in file_paths.iter().enumerate() {
            // Get the original index from the mapping
            let orig_idx = file_paths_with_indices[file_idx].0;
            let entry = &mut self.report_entries[orig_idx];
            
            if !file_path.exists() {
                entry.processed = "No".to_string();
                entry.skip_reason = Some("File not found".to_string());
                continue;
            }
            
            // Check if this file needs conversion/processing
            let is_convertible = conversion_engine.is_convertible_file(file_path);
            let is_llm_readable = ReportModel::is_llm_readable(file_path);
            let needs_processing = is_convertible || is_llm_readable;
            
            // Hash the file
            match hashing_service.hash_file_sha512(file_path) {
                Ok(hash) => {
                    entry.sha512 = Some(hash);
                }
                Err(e) => {
                    self.logger.warning(&format!(
                        "Failed to hash {}: {}",
                        file_path.display(),
                        e
                    ));
                    entry.processed = "No".to_string();
                    entry.skip_reason = Some(format!("Hash failed: {}", e));
                    // Only increment progress if this file was supposed to be processed
                    if needs_processing {
                        processed_count += 1;
                        self.emit_progress(processed_count, conversion_count, "Converting Documents");
                    }
                    continue;
                }
            }
            
            // Check conversion
            Self::process_single_file_conversion(
                &self.logger,
                entry, 
                file_path, 
                working_path, 
                &conversion_engine, 
                &hashing_service
            );
            
            // Only increment progress counter for files that were actually processed
            if needs_processing {
                processed_count += 1;
                self.emit_progress(processed_count, conversion_count, "Converting Documents");
            }
        }
        Ok(())
    }

    fn process_single_file_conversion(
        logger: &EPTLogger,
        entry: &mut ReportModel,
        file_path: &Path,
        working_path: &Path,
        conversion_engine: &ConversionEngine,
        hashing_service: &HashingService
    ) {
        let is_convertible = conversion_engine.is_convertible_file(file_path);
        
        if is_convertible {
            match conversion_engine.convert_file(file_path, working_path) {
                Ok(Some(converted_path)) => {
                    logger.info(&format!(
                        "Converted {} to {}",
                        file_path.display(),
                        converted_path.display()
                    ));
                    // Update entry to point to converted file
                    entry.file_name = converted_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| entry.file_name.clone());
                        
                    // Update relative_path
                    if let Ok(relative_converted_path) = converted_path.strip_prefix(working_path) {
                        entry.relative_path = relative_converted_path
                            .to_string_lossy()
                            .to_string();
                    }
                    
                    // Re-hash converted file
                    match hashing_service.hash_file_sha512(&converted_path) {
                        Ok(hash) => {
                            entry.sha512 = Some(hash);
                        }
                        Err(e) => {
                            logger.warning(&format!(
                                "Failed to hash converted file {}: {}",
                                converted_path.display(),
                                e
                            ));
                            entry.sha512 = None;
                        }
                    }
                    entry.processed = "Yes".to_string();
                }
                Ok(None) => {
                    // No conversion needed, mark as processed if LLM-readable
                    entry.processed = "Yes".to_string();
                }
                Err(e) => {
                    logger.error(&format!(
                        "Conversion failed for {}: {}",
                        file_path.display(),
                        e
                    ));
                    entry.processed = "No".to_string();
                    entry.skip_reason = Some(format!("Conversion failed: {}", e));
                }
            }
        } else {
            // Check if file is already LLM-readable
            if ReportModel::is_llm_readable(file_path) {
                entry.processed = "Yes".to_string();
            } else {
                entry.processed = "No".to_string();
                entry.skip_reason = Some("Not LLM-readable and not convertible".to_string());
            }
        }
    }

    fn finalize_output(&self, working_path: &Path, total_files: usize) -> Result<ProcessingResult> {
        // Generate output folder name
        let input_name = working_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("output");
        let llm_folder_name = format!("{}_LLM", input_name);
        
        let parent_dir = working_path
            .parent()
            .context("Working path has no parent directory")?;
        let llm_output_path = parent_dir.join(&llm_folder_name);
        
        // Export LLM-readable files
        self.logger.info("Exporting LLM-readable files...");
        self.emit_progress(total_files, total_files, "Finishing up");
        let llm_export_engine = LLMExportEngine::new(self.logger.clone());
        llm_export_engine.copy_llm_readable_files(
            &self.report_entries,
            working_path,
            &llm_output_path,
        ).context("Failed to export LLM-readable files")?;
        
        // Generate report
        let report_filename = format!("{}_LLM_file-report.xlsx", input_name);
        let report_path = llm_output_path.join(&report_filename);
        
        self.logger.info("Generating report...");
        let report_writer = ReportWriter::new(self.logger.clone());
        report_writer.generate_report(&self.report_entries, &report_path)
            .context("Failed to generate Excel report")?;
        
        // Emit final progress
        self.emit_progress(total_files, total_files, "Complete");
        
        Ok(ProcessingResult {
            entries: self.report_entries.clone(),
            staging_path: working_path.to_string_lossy().to_string(),
            llm_output_path: llm_output_path.to_string_lossy().to_string(),
            report_path: report_path.to_string_lossy().to_string(),
        })
    }

    /// SECURITY: Safely resolve a relative path and ensure it stays within the working directory
    /// Returns None if path traversal is detected
    fn safe_resolve_path(
        &self,
        working_path: &Path,
        working_path_canonical: &Path,
        relative_path: &str,
    ) -> Option<PathBuf> {
        // Remove leading slashes/backslashes
        let sanitized = relative_path.trim_start_matches('/').trim_start_matches('\\');
        
        // Join with working path
        let joined_path = working_path.join(sanitized);
        
        // Try to canonicalize to resolve any symlinks and normalize the path
        match joined_path.canonicalize() {
            Ok(canonical) => {
                // Ensure the canonical path is within the working directory
                if canonical.starts_with(working_path_canonical) {
                    Some(canonical)
                } else {
                    self.logger.warning(&format!(
                        "SECURITY: Blocked path traversal attempt: {} (resolved to: {})",
                        relative_path,
                        canonical.display()
                    ));
                    None
                }
            }
            Err(_) => {
                // If canonicalize fails, check if the path exists and validate manually
                if joined_path.exists() {
                    // For existing paths, do a basic check that it's within working_path
                    // This is less secure than canonicalize but handles edge cases
                    let mut current = joined_path.as_path();
                    let mut depth = 0;
                    const MAX_DEPTH: usize = 100; // Prevent infinite loops
                    
                    while let Some(parent) = current.parent() {
                        if parent == working_path || parent.starts_with(working_path) {
                            return Some(joined_path);
                        }
                        current = parent;
                        depth += 1;
                        if depth > MAX_DEPTH {
                            break;
                        }
                    }
                    
                    self.logger.warning(&format!(
                        "SECURITY: Could not validate path safety: {}",
                        relative_path
                    ));
                    None
                } else {
                    // Path doesn't exist yet, but we'll allow it if it's clearly within working_path
                    if sanitized.contains("..") {
                        self.logger.warning(&format!(
                            "SECURITY: Blocked path with traversal sequence: {}",
                            relative_path
                        ));
                        None
                    } else {
                        Some(joined_path)
                    }
                }
            }
        }
    }

    fn copy_directory_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        // Create destination directory
        fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create destination directory: {}", dst.display()))?;
        
        // Walk through all files and directories in source
        for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
            let src_path = entry.path();
            let relative_path = src_path
                .strip_prefix(src)
                .with_context(|| format!("Failed to get relative path for {}", src_path.display()))?;
            let dst_path = dst.join(relative_path);
            
            if src_path.is_dir() {
                // Create directory in destination
                fs::create_dir_all(&dst_path)
                    .with_context(|| format!("Failed to create directory: {}", dst_path.display()))?;
            } else if src_path.is_file() {
                // Skip common system files
                if let Some(file_name) = src_path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("~$") 
                        || file_name.starts_with("._") 
                        || file_name.starts_with(".DS")
                        || file_name == "desktop.ini"
                        || file_name == "thumbs.db"
                        || file_name == ".DS_Store" {
                        continue;
                    }
                }
                
                // Copy file to destination
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
                }
                fs::copy(src_path, &dst_path)
                    .with_context(|| format!("Failed to copy file from {} to {}", 
                        src_path.display(), dst_path.display()))?;
            }
        }
        
        self.logger.info(&format!("Successfully copied directory from {} to {}", 
            src.display(), dst.display()));
        Ok(())
    }
}

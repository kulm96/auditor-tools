use crate::ept_logger::EPTLogger;
use crate::hashing_service::HashingService;
use crate::report_model::ReportModel;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LLMExportEngine {
    logger: EPTLogger,
    hashing_service: HashingService,
}

impl LLMExportEngine {
    pub fn new(logger: EPTLogger) -> Self {
        let hashing_service = HashingService::new();
        Self {
            logger,
            hashing_service,
        }
    }

    pub fn copy_llm_readable_files(
        &self,
        files: &[ReportModel],
        root_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        self.logger.debug(&format!(
            "Starting LLM export to: {}",
            output_path.display()
        ));

        // Create output directory
        fs::create_dir_all(output_path)
            .with_context(|| format!("Failed to create output directory: {}", output_path.display()))?;

        // Canonicalize root path for security validation
        let root_path_canonical = root_path.canonicalize()
            .context("Failed to canonicalize root path")?;
        
        // Track seen hashes for deduplication
        let mut seen_hashes: HashMap<String, PathBuf> = HashMap::new();
        let mut copied_count = 0;
        let mut skipped_count = 0;

        for file_entry in files {
            // Skip files that weren't processed or were skipped
            if file_entry.processed != "Yes" {
                continue;
            }

            // SECURITY: Safely resolve relative paths and validate they stay within root directory
            let source_path = match self.safe_resolve_path(root_path, &root_path_canonical, &file_entry.relative_path) {
                Some(path) => path,
                None => {
                    self.logger.warning(&format!(
                        "SECURITY: Skipping file with invalid path: {}",
                        file_entry.relative_path
                    ));
                    continue;
                }
            };

            if !source_path.exists() {
                self.logger.warning(&format!(
                    "Source file does not exist: {}",
                    source_path.display()
                ));
                continue;
            }

            // Check if file is LLM-readable or was converted
            if !self.is_llm_readable(&source_path, file_entry) {
                continue;
            }

            // Get hash for deduplication
            let hash = if let Some(ref sha512) = file_entry.sha512 {
                sha512.clone()
            } else {
                // Hash the file if not already hashed
                match self.hashing_service.hash_file_sha512(&source_path) {
                    Ok(h) => h,
                    Err(e) => {
                        self.logger.warning(&format!(
                            "Failed to hash file {}: {}",
                            source_path.display(),
                            e
                        ));
                        continue;
                    }
                }
            };

            // Check for duplicates
            if let Some(existing_path) = seen_hashes.get(&hash) {
                self.logger.info(&format!(
                    "Skipping duplicate (hash {}): {} (already copied as {})",
                    &hash[..16],
                    source_path.display(),
                    existing_path.display()
                ));
                skipped_count += 1;
                continue;
            }

            // Determine output filename
            // For converted files, use the converted filename
            // For others, use original filename
            let output_filename = if file_entry.file_name.contains("__converted") {
                file_entry.file_name.clone()
            } else {
                // Ensure unique filename in flat structure
                let base_name = source_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                // If filename already exists, add a counter
                let mut final_name = base_name.to_string();
                let mut counter = 1;
                while output_path.join(&final_name).exists() {
                    let stem = source_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("file");
                    let ext = source_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if ext.is_empty() {
                        final_name = format!("{}_{}", stem, counter);
                    } else {
                        final_name = format!("{}_{}.{}", stem, counter, ext);
                    }
                    counter += 1;
                }
                final_name
            };

            let dest_path = output_path.join(&output_filename);

            // Copy the file
            match fs::copy(&source_path, &dest_path) {
                Ok(_) => {
                    // Show relative paths in log
                    let source_relative = source_path.strip_prefix(root_path)
                        .unwrap_or(&source_path)
                        .display();
                    let dest_relative = dest_path.strip_prefix(output_path)
                        .unwrap_or(&dest_path)
                        .display();
                    self.logger.debug(&format!(
                        "Copied: {} -> {}",
                        source_relative,
                        dest_relative
                    ));
                    seen_hashes.insert(hash, dest_path.clone());
                    copied_count += 1;
                }
                Err(e) => {
                    self.logger.error(&format!(
                        "Failed to copy {}: {}",
                        source_path.display(),
                        e
                    ));
                }
            }
        }

        self.logger.info(&format!(
            "LLM export complete: {} files copied, {} duplicates skipped",
            copied_count,
            skipped_count
        ));

        Ok(())
    }

    fn is_llm_readable(&self, file_path: &Path, file_entry: &ReportModel) -> bool {
        // Check if file was converted (converted files are always LLM-readable)
        if file_entry.file_name.contains("__converted") {
            return true;
        }

        ReportModel::is_llm_readable(file_path)
    }
    
    /// SECURITY: Safely resolve a relative path and ensure it stays within the root directory
    /// Returns None if path traversal is detected
    fn safe_resolve_path(
        &self,
        root_path: &Path,
        root_path_canonical: &Path,
        relative_path: &str,
    ) -> Option<PathBuf> {
        // Remove leading slashes/backslashes
        let sanitized = relative_path.trim_start_matches('/').trim_start_matches('\\');
        
        // Join with root path
        let joined_path = root_path.join(sanitized);
        
        // Try to canonicalize to resolve any symlinks and normalize the path
        match joined_path.canonicalize() {
            Ok(canonical) => {
                // Ensure the canonical path is within the root directory
                if canonical.starts_with(root_path_canonical) {
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
                    // For existing paths, do a basic check that it's within root_path
                    // This is less secure than canonicalize but handles edge cases
                    let mut current = joined_path.as_path();
                    let mut depth = 0;
                    const MAX_DEPTH: usize = 100; // Prevent infinite loops
                    
                    while let Some(parent) = current.parent() {
                        if parent == root_path || parent.starts_with(root_path) {
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
                    // Path doesn't exist yet, but we'll allow it if it's clearly within root_path
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
}


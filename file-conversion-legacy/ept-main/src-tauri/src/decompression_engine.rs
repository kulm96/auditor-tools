use crate::ept_logger::EPTLogger;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::ZipArchive;

pub struct DecompressionEngine {
    logger: EPTLogger,
    visited_paths: std::collections::HashSet<PathBuf>,
}

impl DecompressionEngine {
    pub fn new(logger: EPTLogger) -> Self {
        Self {
            logger,
            visited_paths: std::collections::HashSet::new(),
        }
    }

    pub fn expand_zip_to_folder(&mut self, zip_path: &Path) -> Result<PathBuf> {
        let zip_name = zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("extracted");
        
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let output_folder_name = format!("{}__{}", zip_name, timestamp);
        
        let parent_dir = zip_path
            .parent()
            .context("ZIP file has no parent directory")?;
        
        let output_path = parent_dir.join(&output_folder_name);
        
        self.logger.info(&format!("Extracting ZIP: {} -> {}", 
            zip_path.display(), output_path.display()));
        
        fs::create_dir_all(&output_path)
            .context("Failed to create extraction directory")?;
        
        // Canonicalize output path for security validation
        let output_path_canonical = output_path.canonicalize()
            .context("Failed to canonicalize output path")?;
        
        let file = fs::File::open(zip_path)
            .context("Failed to open ZIP file")?;
        
        let mut archive = ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read file from ZIP")?;
            
            // SECURITY: Sanitize ZIP entry name to prevent path traversal
            let entry_name = file.name();
            let sanitized_name = self.sanitize_zip_entry_name(entry_name);
            
            let outpath = output_path.join(&sanitized_name);

            // SECURITY: Validate that the resolved path stays within output directory.
            // We already stripped any leading slashes and removed `..` segments in
            // `sanitize_zip_entry_name`, so any path created by joining with
            // `output_path` will remain inside that directory as long as we don't
            // introduce new traversal here.
            //
            // For defense-in-depth, we still *attempt* to canonicalize, but we no
            // longer fail hard when the target file/dir doesn't exist yet (which is
            // normal during extraction and caused `ENOENT` errors).
            let outpath_canonical = match outpath.canonicalize() {
                Ok(canonical) => canonical,
                Err(e) => {
                    // If the path doesn't exist yet, fall back to `outpath` itself.
                    // It's safe because:
                    //   - `output_path` is canonicalized above
                    //   - `sanitize_zip_entry_name` removed `..` and leading separators
                    // Therefore `outpath` cannot escape `output_path`.
                    if e.kind() == std::io::ErrorKind::NotFound {
                        outpath.clone()
                    } else {
                        return Err(e).context("Failed to resolve extraction path");
                    }
                }
            };

            // Ensure the (canonical or constructed) path is within the output directory
            if !outpath_canonical.starts_with(&output_path_canonical) {
                self.logger.warning(&format!(
                    "SECURITY: Blocked path traversal attempt in ZIP entry: {} (resolved to: {})",
                    entry_name,
                    outpath_canonical.display()
                ));
                continue;
            }
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)
                    .context("Failed to create directory in ZIP")?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)
                        .context("Failed to create parent directory")?;
                }
                
                let mut outfile = fs::File::create(&outpath)
                    .context("Failed to create output file")?;
                std::io::copy(&mut file, &mut outfile)
                    .context("Failed to write file from ZIP")?;
            }
        }
        
        self.logger.info(&format!("Successfully extracted ZIP to: {}", output_path.display()));
        Ok(output_path)
    }
    
    /// Sanitize ZIP entry names to prevent path traversal attacks
    /// Removes leading slashes and normalizes path separators
    fn sanitize_zip_entry_name(&self, entry_name: &str) -> String {
        // Remove leading slashes and backslashes
        let mut sanitized = entry_name.trim_start_matches('/').trim_start_matches('\\').to_string();
        
        // Normalize path separators to forward slashes for cross-platform compatibility
        // Then replace with platform-specific separator when joining paths
        sanitized = sanitized.replace('\\', "/");
        
        // Remove any remaining path traversal sequences
        // This is a defense-in-depth measure (canonicalize check is primary protection)
        while sanitized.contains("../") {
            sanitized = sanitized.replace("../", "");
        }
        while sanitized.contains("..\\") {
            sanitized = sanitized.replace("..\\", "");
        }
        
        sanitized
    }

    pub fn recursive_decompress(&mut self, input_path: &Path) -> Result<()> {
        self.visited_paths.clear();
        self._recursive_decompress_internal(input_path)?;
        Ok(())
    }

    fn _recursive_decompress_internal(&mut self, dir_path: &Path) -> Result<()> {
        let entries: Vec<_> = WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        for entry in entries {
            let path = entry.path();
            
            if path.is_file() && self.is_compressed_file(path) {
                let normalized = path.canonicalize()
                    .unwrap_or_else(|_| path.to_path_buf());
                
                if self.visited_paths.contains(&normalized) {
                    self.logger.warning(&format!("Skipping already processed archive: {}", path.display()));
                    continue;
                }
                
                self.visited_paths.insert(normalized);
                
                if let Err(e) = self.decompress_file(path) {
                    self.logger.error(&format!("Failed to decompress {}: {}", path.display(), e));
                }
            }
        }
        
        Ok(())
    }

    fn is_compressed_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            matches!(ext_lower.as_str(), "zip" | "gz")
        } else {
            false
        }
    }

    fn decompress_file(&mut self, file_path: &Path) -> Result<()> {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            
            match ext_lower.as_str() {
                "zip" => self.decompress_zip(file_path),
                "gz" => self.decompress_gz(file_path),
                _ => {
                    self.logger.warning(&format!("Unsupported archive format: {}", ext));
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    fn decompress_zip(&mut self, zip_path: &Path) -> Result<()> {
        self.logger.info(&format!("Decompressing ZIP: {}", zip_path.display()));
        
        let output_path = self.expand_zip_to_folder(zip_path)?;
        
        // Recursively process the newly extracted folder
        self._recursive_decompress_internal(&output_path)?;
        
        Ok(())
    }

    fn decompress_gz(&mut self, gz_path: &Path) -> Result<()> {
        self.logger.info(&format!("Decompressing GZ: {}", gz_path.display()));
        
        let file_stem = gz_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("extracted");
        
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let output_file_name = format!("{}__{}", file_stem, timestamp);
        
        let parent_dir = gz_path
            .parent()
            .context("GZ file has no parent directory")?;
        
        let output_path = parent_dir.join(&output_file_name);
        
        let mut decoder = flate2::read::GzDecoder::new(
            fs::File::open(gz_path)
                .context("Failed to open GZ file")?
        );
        
        let mut output_file = fs::File::create(&output_path)
            .context("Failed to create output file")?;
        
        std::io::copy(&mut decoder, &mut output_file)
            .context("Failed to decompress GZ file")?;
        
        self.logger.info(&format!("Successfully decompressed GZ to: {}", output_path.display()));
        
        // If the output is another archive, process it
        if self.is_compressed_file(&output_path) {
            let normalized = output_path.canonicalize()
                .unwrap_or_else(|_| output_path.clone());
            
            if !self.visited_paths.contains(&normalized) {
                self.visited_paths.insert(normalized);
                self.decompress_file(&output_path)?;
            }
        }
        
        Ok(())
    }
}




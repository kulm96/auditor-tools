use crate::ept_logger::EPTLogger;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use calamine::{open_workbook, Reader, Xlsx, Xls};
use chrono::Local;

pub struct ConversionEngine {
    logger: EPTLogger,
}

impl ConversionEngine {
    pub fn new(logger: EPTLogger) -> Self {
        Self { logger }
    }

    pub fn convert_file(&self, file_path: &Path, _root_path: &Path) -> Result<Option<PathBuf>> {
        if !self.is_convertible_file(file_path) {
            return Ok(None);
        }

        let file_ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Determine output format: xls/xlsx → md, others → PDF
        let output_ext = if matches!(file_ext.as_str(), "xls" | "xlsx") {
            "md"
        } else {
            "pdf"
        };

        // Create output filename: <filename>__converted.<ext>
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("converted");
        
        let output_filename = format!("{}__converted.{}", file_stem, output_ext);
        let output_path = file_path
            .parent()
            .context("File has no parent directory")?
            .join(&output_filename);

        self.logger.info(&format!(
            "Converting {} to {}",
            file_path.display(),
            output_path.display()
        ));

        // Handle XLS/XLSX files separately using calamine
        if output_ext == "md" {
            return self.convert_excel_to_markdown(file_path, &output_path);
        }

        // For other file types, use LibreOffice
        // Find LibreOffice executable
        let libreoffice_cmd = self.find_libreoffice()?;

        let output_dir = output_path
            .parent()
            .context("Output path has no parent")?;

        let mut cmd = Command::new(&libreoffice_cmd);
        cmd.arg("--headless")
            .arg("--convert-to")
            .arg(output_ext)
            .arg("--outdir")
            .arg(output_dir)
            .arg(file_path);

        self.logger.info(&format!(
            "Executing LibreOffice command: {:?} {:?}",
            libreoffice_cmd,
            cmd.get_args().collect::<Vec<_>>()
        ));

        let output = cmd.output()
            .with_context(|| "Failed to execute LibreOffice conversion command".to_string())?;

        let stdout_msg = String::from_utf8_lossy(&output.stdout);
        let stderr_msg = String::from_utf8_lossy(&output.stderr);
        
        self.logger.info(&format!(
            "LibreOffice exit status: {:?}, stdout: {}, stderr: {}",
            output.status.code(),
            stdout_msg,
            stderr_msg
        ));

        if !output.status.success() {
            self.logger.error(&format!(
                "LibreOffice conversion failed for {}: stdout: {}, stderr: {}",
                file_path.display(),
                stdout_msg,
                stderr_msg
            ));
            return Err(anyhow::anyhow!(
                "LibreOffice conversion failed: stdout: {}, stderr: {}",
                stdout_msg,
                stderr_msg
            ));
        }

        // For PDF, check if the file was created with the expected name
        // LibreOffice might use the original filename
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("converted");
        let possible_pdf = output_dir.join(format!("{}.pdf", file_stem));
        self.logger.info(&format!(
            "Checking for PDF output: {} (exists: {}), expected: {} (exists: {})",
            possible_pdf.display(),
            possible_pdf.exists(),
            output_path.display(),
            output_path.exists()
        ));
        
        if possible_pdf.exists() && !output_path.exists() {
            self.logger.info(&format!(
                "Found PDF with original name, renaming {} to {}",
                possible_pdf.display(),
                output_path.display()
            ));
            std::fs::rename(&possible_pdf, &output_path)
                .with_context(|| format!("Failed to rename {} to {}", possible_pdf.display(), output_path.display()))?;
        }

        if output_path.exists() {
            self.logger.info(&format!("Successfully converted to: {}", output_path.display()));
            Ok(Some(output_path))
        } else {
            Err(anyhow::anyhow!("Conversion completed but output file not found"))
        }
    }

    fn convert_excel_to_markdown(&self, file_path: &Path, output_path: &Path) -> Result<Option<PathBuf>> {
        self.logger.info(&format!(
            "Converting Excel file {} to markdown",
            file_path.display()
        ));

        let file_ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Create markdown content
        let mut markdown_content = Vec::new();
        
        // Add header
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        markdown_content.push(format!("# Excel File: {}", file_name));
        markdown_content.push(String::new());
        markdown_content.push(format!(
            "Converted on: {}",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        markdown_content.push(String::new());

        // Process workbook based on file extension
        let sheet_names = if file_ext == "xlsx" {
            self.process_xlsx_workbook(file_path, &mut markdown_content)?
        } else if file_ext == "xls" {
            self.process_xls_workbook(file_path, &mut markdown_content)?
        } else {
            return Err(anyhow::anyhow!("Unsupported Excel file format: {}", file_ext));
        };

        if sheet_names.is_empty() {
            self.logger.warning("Workbook contains no sheets");
            markdown_content.push("*Workbook contains no sheets*".to_string());
        }

        // Write markdown file
        std::fs::write(output_path, markdown_content.join("\n"))
            .with_context(|| format!("Failed to write markdown file: {}", output_path.display()))?;

        self.logger.info(&format!(
            "Successfully converted Excel file to markdown: {} (processed {} sheet(s))",
            output_path.display(),
            sheet_names.len()
        ));

        Ok(Some(output_path.to_path_buf()))
    }

    fn process_xlsx_workbook(&self, file_path: &Path, markdown_content: &mut Vec<String>) -> Result<Vec<String>> {
        let mut workbook: Xlsx<_> = open_workbook(file_path)
            .with_context(|| format!("Failed to open XLSX file: {}", file_path.display()))?;

        let sheet_names = workbook.sheet_names().to_vec();
        
        for sheet_name in &sheet_names {
            if sheet_name == "Conversion Notice" {
                continue;
            }

            self.logger.info(&format!("Processing sheet: {}", sheet_name));

            // Read the sheet into a variable (stored as Vec<Vec<String>>)
            let sheet_data: Vec<Vec<String>> = match workbook.worksheet_range(sheet_name) {
                Ok(range) => {
                    let mut rows = Vec::new();
                    for row in range.rows() {
                        let row_data: Vec<String> = row
                            .iter()
                            .map(|cell| {
                                // Use format! to convert cell to string, with special handling for floats
                                let cell_str = format!("{}", cell);
                                // If it's a float, format it appropriately
                                if let Ok(f) = cell_str.parse::<f64>() {
                                    if f == 0.0 {
                                        "0".to_string()
                                    } else if f.abs() < 0.01 {
                                        format!("{:.6}", f)
                                    } else {
                                        let formatted = format!("{:.2}", f);
                                        formatted.trim_end_matches('0').trim_end_matches('.').to_string()
                                    }
                                } else {
                                    cell_str
                                }
                            })
                            .collect();
                        rows.push(row_data);
                    }
                    rows
                }
                Err(e) => {
                    self.logger.error(&format!("Error reading sheet {}: {}", sheet_name, e));
                    markdown_content.push(format!("## Sheet: {} (Error)", sheet_name));
                    markdown_content.push(String::new());
                    markdown_content.push(format!("*Error processing sheet: {}*", e));
                    markdown_content.push(String::new());
                    continue;
                }
            };

            // Convert sheet data to markdown
            self.sheet_data_to_markdown(sheet_name, sheet_data, markdown_content);
        }

        Ok(sheet_names)
    }

    fn process_xls_workbook(&self, file_path: &Path, markdown_content: &mut Vec<String>) -> Result<Vec<String>> {
        let mut workbook: Xls<_> = open_workbook(file_path)
            .with_context(|| format!("Failed to open XLS file: {}", file_path.display()))?;

        let sheet_names = workbook.sheet_names().to_vec();
        
        for sheet_name in &sheet_names {
            if sheet_name == "Conversion Notice" {
                continue;
            }

            self.logger.info(&format!("Processing sheet: {}", sheet_name));

            // Read the sheet into a variable (stored as Vec<Vec<String>>)
            let sheet_data: Vec<Vec<String>> = match workbook.worksheet_range(sheet_name) {
                Ok(range) => {
                    let mut rows = Vec::new();
                    for row in range.rows() {
                        let row_data: Vec<String> = row
                            .iter()
                            .map(|cell| {
                                // Use format! to convert cell to string, with special handling for floats
                                let cell_str = format!("{}", cell);
                                // If it's a float, format it appropriately
                                if let Ok(f) = cell_str.parse::<f64>() {
                                    if f == 0.0 {
                                        "0".to_string()
                                    } else if f.abs() < 0.01 {
                                        format!("{:.6}", f)
                                    } else {
                                        let formatted = format!("{:.2}", f);
                                        formatted.trim_end_matches('0').trim_end_matches('.').to_string()
                                    }
                                } else {
                                    cell_str
                                }
                            })
                            .collect();
                        rows.push(row_data);
                    }
                    rows
                }
                Err(e) => {
                    self.logger.error(&format!("Error reading sheet {}: {}", sheet_name, e));
                    markdown_content.push(format!("## Sheet: {} (Error)", sheet_name));
                    markdown_content.push(String::new());
                    markdown_content.push(format!("*Error processing sheet: {}*", e));
                    markdown_content.push(String::new());
                    continue;
                }
            };

            // Convert sheet data to markdown
            self.sheet_data_to_markdown(sheet_name, sheet_data, markdown_content);
        }

        Ok(sheet_names)
    }


    fn sheet_data_to_markdown(&self, sheet_name: &str, sheet_data: Vec<Vec<String>>, markdown_content: &mut Vec<String>) {
        // Add sheet header
        markdown_content.push(format!("## Sheet: {}", sheet_name));
        markdown_content.push(String::new());

        // Check if sheet is empty
        if sheet_data.is_empty() || (sheet_data.len() == 1 && sheet_data[0].is_empty()) {
            markdown_content.push("*Sheet is empty*".to_string());
            markdown_content.push(String::new());
            return;
        }

        // Find the maximum number of columns
        let max_cols = sheet_data
            .iter()
            .map(|row| row.len())
            .max()
            .unwrap_or(0);

        if max_cols == 0 {
            markdown_content.push("*Sheet is empty*".to_string());
            markdown_content.push(String::new());
            return;
        }

        // Create header row (use first row if available)
        let headers = if !sheet_data.is_empty() {
            let first_row = &sheet_data[0];
            let mut header_row = first_row.clone();
            while header_row.len() < max_cols {
                header_row.push(String::new());
            }
            header_row.truncate(max_cols);
            header_row
        } else {
            vec![String::new(); max_cols]
        };

        // Escape pipe characters in headers
        let headers: Vec<String> = headers
            .iter()
            .map(|h| h.replace('|', "\\|"))
            .collect();

        // Create markdown table
        markdown_content.push(format!("| {} |", headers.join(" | ")));
        
        // Create separator
        let separator = format!("|{}|", "---|".repeat(max_cols));
        markdown_content.push(separator);

        // Add data rows (skip first row if it was used as header)
        let data_start = if !sheet_data.is_empty() && !sheet_data[0].is_empty() { 1 } else { 0 };
        for row in &sheet_data[data_start..] {
            let mut row_values = row.clone();
            while row_values.len() < max_cols {
                row_values.push(String::new());
            }
            row_values.truncate(max_cols);
            
            // Escape pipe characters
            let row_values: Vec<String> = row_values
                .iter()
                .map(|v| v.replace('|', "\\|"))
                .collect();
            
            markdown_content.push(format!("| {} |", row_values.join(" | ")));
        }

        markdown_content.push(String::new());
    }

    pub fn is_convertible_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            matches!(
                ext_lower.as_str(),
                "doc" | "docx" | "ppt" | "pptx" | "xls" | "xlsx" | "odt" | "ods" | "odp"
            )
        } else {
            false
        }
    }

    pub fn find_libreoffice(&self) -> Result<PathBuf> {
        // First, check EPT_LIBREOFFICE_PATH environment variable
        if let Ok(env_path) = std::env::var("EPT_LIBREOFFICE_PATH") {
            let path = PathBuf::from(&env_path);
            if path.exists() {
                self.logger.info(&format!("Using LibreOffice from EPT_LIBREOFFICE_PATH: {}", env_path));
                return Ok(path);
            } else {
                self.logger.warning(&format!("EPT_LIBREOFFICE_PATH is set to {}, but file does not exist", env_path));
            }
        }

        // Try common LibreOffice executable names and paths
        let candidates = if cfg!(target_os = "windows") {
            vec![
                "soffice.exe",
                "C:\\Program Files\\LibreOffice\\program\\soffice.exe",
                "C:\\Program Files (x86)\\LibreOffice\\program\\soffice.exe",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/Applications/LibreOffice.app/Contents/MacOS/soffice",
                "soffice",
            ]
        } else {
            vec![
                "/usr/bin/soffice",
                "/usr/local/bin/soffice",
                "soffice",
            ]
        };

        for candidate in candidates {
            if let Ok(path) = which::which(candidate) {
                return Ok(path);
            }
        }

        // Try to find it in PATH
        if let Ok(path) = which::which("soffice") {
            return Ok(path);
        }

        Err(anyhow::anyhow!(
            "LibreOffice not found. Please install LibreOffice and ensure 'soffice' is in your PATH."
        ))
    }
}


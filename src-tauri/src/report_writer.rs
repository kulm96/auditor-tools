use crate::ept_logger::EPTLogger;
use crate::report_model::ReportModel;
use anyhow::{Context, Result};
use rust_xlsxwriter::Workbook;
use std::path::Path;

pub struct ReportWriter {
    logger: EPTLogger,
}

impl ReportWriter {
    pub fn new(logger: EPTLogger) -> Self {
        Self { logger }
    }

    pub fn generate_report(&self, entries: &[ReportModel], output_path: &Path) -> Result<()> {
        self.logger.debug(&format!(
            "Generating report: {}",
            output_path.display()
        ));

        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create report directory: {}", parent.display()))?;
        }

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Write headers
        let headers = vec![
            "File Name",
            "Converted File Name",
            "SHA512",
            "Processed",
            "Skip Reason",
            "Relative Path",
            "File Type",
            "File Size (Bytes)",
            "File Size (Human)",
            "Last Modified",
            "Created Time",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string(0, col as u16, header.to_string())
                .with_context(|| format!("Failed to write header: {}", header))?;
        }

        // Write data rows
        for (row, entry) in entries.iter().enumerate() {
            let row_num = (row + 1) as u32;
            
            worksheet
                .write_string(row_num, 0, &entry.original_file_name)
                .with_context(|| "Failed to write original_file_name")?;
            
            let converted_name_str = entry.converted_file_name.as_deref().unwrap_or("");
            worksheet
                .write_string(row_num, 1, converted_name_str)
                .with_context(|| "Failed to write converted_file_name")?;
            
            let sha512_str = entry.sha512.as_deref().unwrap_or("");
            worksheet
                .write_string(row_num, 2, sha512_str)
                .with_context(|| "Failed to write sha512")?;
            
            worksheet
                .write_string(row_num, 3, &entry.processed)
                .with_context(|| "Failed to write processed")?;
            
            let skip_reason_str = entry.skip_reason.as_deref().unwrap_or("");
            worksheet
                .write_string(row_num, 4, skip_reason_str)
                .with_context(|| "Failed to write skip_reason")?;
            
            worksheet
                .write_string(row_num, 5, &entry.original_relative_path)
                .with_context(|| "Failed to write relative_path")?;
            
            worksheet
                .write_string(row_num, 6, &entry.file_type)
                .with_context(|| "Failed to write file_type")?;
            
            worksheet
                .write_number(row_num, 7, entry.file_size_bytes as f64)
                .with_context(|| "Failed to write file_size_bytes")?;
            
            worksheet
                .write_string(row_num, 8, &entry.file_size_human)
                .with_context(|| "Failed to write file_size_human")?;
            
            worksheet
                .write_string(row_num, 9, &entry.last_modified)
                .with_context(|| "Failed to write last_modified")?;
            
            worksheet
                .write_string(row_num, 10, &entry.created_time)
                .with_context(|| "Failed to write created_time")?;
        }

        // Auto-fit columns (approximate)
        for col in 0..headers.len() {
            worksheet.set_column_width(col as u16, 15.0)?;
        }

        // Set specific column widths
        worksheet.set_column_width(0, 30.0)?; // File Name
        worksheet.set_column_width(1, 30.0)?; // Converted File Name
        worksheet.set_column_width(2, 64.0)?; // SHA512
        worksheet.set_column_width(5, 40.0)?; // Relative Path

        // Save the workbook
        workbook
            .save(output_path)
            .with_context(|| format!("Failed to save workbook to: {}", output_path.display()))?;

        self.logger.info(&format!(
            "Report generated successfully: {} ({} entries)",
            output_path.display(),
            entries.len()
        ));

        Ok(())
    }
}


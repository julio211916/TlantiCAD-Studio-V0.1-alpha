//! Excel XLSX writer using rust_xlsxwriter
//!
//! Exports dental data to formatted Excel spreadsheets with:
//! - Auto-width columns, headers, date formatting
//! - Multiple sheets per workbook
//! - Summary/totals rows

use crate::error::ExportError;
use rust_xlsxwriter::*;
use serde_json::Value;
use std::path::Path;

/// Column definition for export
#[derive(Debug, Clone)]
pub struct ExportColumn {
    pub header: String,
    pub field: String,
    pub width: f64,
    pub format_type: ColumnFormat,
}

#[derive(Debug, Clone)]
pub enum ColumnFormat {
    Text,
    Number,
    Currency,
    Date,
    DateTime,
    Boolean,
    Percentage,
}

/// Export JSON data to an Excel file
pub fn export_to_excel(
    path: &Path,
    sheet_name: &str,
    columns: &[ExportColumn],
    data: &[Value],
) -> Result<(), ExportError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(sheet_name).map_err(|e| ExportError::ExcelWrite(e.to_string()))?;

    // Header format
    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin);

    // Currency format
    let currency_fmt = Format::new().set_num_format("$#,##0.00");
    let date_fmt = Format::new().set_num_format("yyyy-mm-dd");
    let pct_fmt = Format::new().set_num_format("0.00%");

    // Write headers
    for (col, column) in columns.iter().enumerate() {
        worksheet
            .write_string_with_format(0, col as u16, &column.header, &header_fmt)
            .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
        worksheet
            .set_column_width(col as u16, column.width)
            .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
    }

    // Write data rows
    for (row_idx, row_data) in data.iter().enumerate() {
        let row = (row_idx + 1) as u32;

        for (col_idx, column) in columns.iter().enumerate() {
            let col = col_idx as u16;
            let value = &row_data[&column.field];

            match column.format_type {
                ColumnFormat::Text => {
                    let text = value.as_str().unwrap_or("");
                    worksheet
                        .write_string(row, col, text)
                        .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
                }
                ColumnFormat::Number => {
                    if let Some(n) = value.as_f64() {
                        worksheet
                            .write_number(row, col, n)
                            .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
                    }
                }
                ColumnFormat::Currency => {
                    if let Some(n) = value.as_f64() {
                        worksheet
                            .write_number_with_format(row, col, n, &currency_fmt)
                            .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
                    }
                }
                ColumnFormat::Date => {
                    let text = value.as_str().unwrap_or("");
                    worksheet
                        .write_string_with_format(row, col, text, &date_fmt)
                        .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
                }
                ColumnFormat::DateTime => {
                    let text = value.as_str().unwrap_or("");
                    worksheet
                        .write_string(row, col, text)
                        .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
                }
                ColumnFormat::Boolean => {
                    let b = value.as_bool().unwrap_or(false);
                    worksheet
                        .write_string(row, col, if b { "Sí" } else { "No" })
                        .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
                }
                ColumnFormat::Percentage => {
                    if let Some(n) = value.as_f64() {
                        worksheet
                            .write_number_with_format(row, col, n, &pct_fmt)
                            .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
                    }
                }
            }
        }
    }

    // Auto-filter
    worksheet
        .autofilter(0, 0, data.len() as u32, (columns.len() - 1) as u16)
        .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;

    // Freeze top row
    worksheet.set_freeze_panes(1, 0).map_err(|e| ExportError::ExcelWrite(e.to_string()))?;

    workbook
        .save(path)
        .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;

    Ok(())
}

/// Export multiple sheets to a single workbook
pub fn export_multi_sheet(
    path: &Path,
    sheets: &[(&str, &[ExportColumn], &[Value])],
) -> Result<(), ExportError> {
    let mut workbook = Workbook::new();

    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin);

    for (sheet_name, columns, data) in sheets {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(*sheet_name).map_err(|e| ExportError::ExcelWrite(e.to_string()))?;

        for (col, column) in columns.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, &column.header, &header_fmt)
                .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
            worksheet
                .set_column_width(col as u16, column.width)
                .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
        }

        for (row_idx, row_data) in data.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            for (col_idx, column) in columns.iter().enumerate() {
                let col = col_idx as u16;
                let val = &row_data[&column.field];
                let text = match val {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => if *b { "Sí" } else { "No" }.to_string(),
                    Value::Null => String::new(),
                    _ => val.to_string(),
                };
                let _ = worksheet.write_string(row, col, &text);
            }
        }

        if !data.is_empty() {
            worksheet
                .autofilter(0, 0, data.len() as u32, (columns.len() - 1) as u16)
                .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;
        }
    }

    workbook
        .save(path)
        .map_err(|e| ExportError::ExcelWrite(e.to_string()))?;

    Ok(())
}

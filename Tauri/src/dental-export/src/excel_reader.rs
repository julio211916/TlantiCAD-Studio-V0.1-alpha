//! Excel XLSX/XLS/ODS reader using calamine
//!
//! Imports dental data from Excel spreadsheets.

use crate::error::ExportError;
use calamine::{open_workbook_auto, Reader, Data};
use serde_json::{json, Value};
use std::path::Path;

/// Read an Excel file and return rows as JSON
pub fn read_excel(path: &Path) -> Result<Vec<Value>, ExportError> {
    let workbook = open_workbook_auto(path)
        .map_err(|e| ExportError::ExcelRead(e.to_string()))?;

    let sheet_names = workbook.sheet_names().to_owned();
    if sheet_names.is_empty() {
        return Err(ExportError::ExcelRead("No sheets found".into()));
    }

    read_excel_sheet(path, &sheet_names[0])
}

/// Read a specific sheet from an Excel file
pub fn read_excel_sheet(path: &Path, sheet_name: &str) -> Result<Vec<Value>, ExportError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| ExportError::ExcelRead(e.to_string()))?;

    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| ExportError::ExcelRead(e.to_string()))?;

    let mut rows_iter = range.rows();

    // First row = headers
    let headers: Vec<String> = match rows_iter.next() {
        Some(header_row) => header_row
            .iter()
            .map(|cell| cell_to_string(cell).to_lowercase().replace(' ', "_"))
            .collect(),
        None => return Ok(Vec::new()),
    };

    // Data rows
    let mut data = Vec::new();
    for row in rows_iter {
        let mut obj = serde_json::Map::new();
        for (i, cell) in row.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                let value = cell_to_json(cell);
                obj.insert(header.clone(), value);
            }
        }
        data.push(Value::Object(obj));
    }

    Ok(data)
}

/// List sheet names in an Excel file
pub fn list_excel_sheets(path: &Path) -> Result<Vec<String>, ExportError> {
    let workbook = open_workbook_auto(path)
        .map_err(|e| ExportError::ExcelRead(e.to_string()))?;

    Ok(workbook.sheet_names().to_vec())
}

/// Get sheet dimensions
pub fn get_sheet_info(path: &Path) -> Result<Vec<SheetInfo>, ExportError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| ExportError::ExcelRead(e.to_string()))?;

    let mut sheets = Vec::new();
    let names = workbook.sheet_names().to_owned();

    for name in &names {
        if let Ok(range) = workbook.worksheet_range(name) {
            let (rows, cols) = range.get_size();
            let headers: Vec<String> = range
                .rows()
                .next()
                .map(|row| row.iter().map(|c| cell_to_string(c)).collect())
                .unwrap_or_default();

            sheets.push(SheetInfo {
                name: name.clone(),
                row_count: rows,
                column_count: cols,
                headers,
            });
        }
    }

    Ok(sheets)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SheetInfo {
    pub name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub headers: Vec<String>,
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format!("{}", f),
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Data::Error(e) => format!("ERROR: {:?}", e),
        Data::DateTime(dt) => format!("{}", dt),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

fn cell_to_json(cell: &Data) -> Value {
    match cell {
        Data::Empty => Value::Null,
        Data::String(s) => json!(s),
        Data::Float(f) => json!(f),
        Data::Int(i) => json!(i),
        Data::Bool(b) => json!(b),
        Data::Error(_) => Value::Null,
        Data::DateTime(dt) => json!(format!("{}", dt)),
        Data::DateTimeIso(s) => json!(s),
        Data::DurationIso(s) => json!(s),
    }
}

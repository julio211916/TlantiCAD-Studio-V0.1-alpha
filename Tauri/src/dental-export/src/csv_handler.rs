//! CSV import/export handler

use crate::error::ExportError;
use serde_json::Value;
use std::path::Path;

/// Export JSON data to a CSV file
pub fn export_to_csv(
    path: &Path,
    headers: &[String],
    fields: &[String],
    data: &[Value],
) -> Result<(), ExportError> {
    let mut writer = csv::Writer::from_path(path)
        .map_err(|e| ExportError::Csv(e.to_string()))?;

    // Write header
    writer
        .write_record(headers)
        .map_err(|e| ExportError::Csv(e.to_string()))?;

    // Write data
    for row in data {
        let record: Vec<String> = fields
            .iter()
            .map(|field| match &row[field] {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                other => other.to_string(),
            })
            .collect();

        writer
            .write_record(&record)
            .map_err(|e| ExportError::Csv(e.to_string()))?;
    }

    writer.flush().map_err(|e| ExportError::Csv(e.to_string()))?;

    Ok(())
}

/// Import a CSV file and return rows as JSON
pub fn import_from_csv(path: &Path) -> Result<Vec<Value>, ExportError> {
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| ExportError::Csv(e.to_string()))?;

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| ExportError::Csv(e.to_string()))?
        .iter()
        .map(|h| h.to_lowercase().replace(' ', "_"))
        .collect();

    let mut data = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| ExportError::Csv(e.to_string()))?;
        let mut obj = serde_json::Map::new();

        for (i, value) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                obj.insert(header.clone(), serde_json::json!(value));
            }
        }
        data.push(Value::Object(obj));
    }

    Ok(data)
}

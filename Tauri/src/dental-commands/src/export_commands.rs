//! Tauri commands for Excel/CSV import-export

use crate::DentalCommandError;
use dental_export::{csv_handler, excel_reader, excel_writer, templates};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Export Commands ──────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub entity: String,       // "patients", "invoices", "payments", etc.
    pub format: String,       // "xlsx" or "csv"
    pub output_path: String,  // Full path to save
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub path: String,
    pub rows_exported: usize,
    pub format: String,
}

#[tauri::command]
pub async fn export_data(
    request: ExportRequest,
) -> Result<ExportResult, DentalCommandError> {
    let path = PathBuf::from(&request.output_path);
    let row_count = request.data.len();

    let columns = match request.entity.as_str() {
        "patients" => templates::patient_columns(),
        "appointments" => templates::appointment_columns(),
        "treatments" => templates::treatment_columns(),
        "invoices" => templates::invoice_columns(),
        "payments" => templates::payment_columns(),
        "inventory" => templates::inventory_columns(),
        "users" => templates::user_columns(),
        _ => return Err(DentalCommandError::Validation(
            format!("Unknown entity: {}. Use: patients, appointments, treatments, invoices, payments, inventory, users", request.entity),
        )),
    };

    match request.format.as_str() {
        "xlsx" => {
            let sheet_name = match request.entity.as_str() {
                "patients" => "Pacientes",
                "appointments" => "Citas",
                "treatments" => "Tratamientos",
                "invoices" => "Facturas",
                "payments" => "Pagos",
                "inventory" => "Inventario",
                "users" => "Usuarios",
                _ => "Datos",
            };

            excel_writer::export_to_excel(&path, sheet_name, &columns, &request.data)
                .map_err(|e| DentalCommandError::Internal(e.to_string()))?;
        }
        "csv" => {
            let headers: Vec<String> = columns.iter().map(|c| c.header.clone()).collect();
            let fields: Vec<String> = columns.iter().map(|c| c.field.clone()).collect();

            csv_handler::export_to_csv(&path, &headers, &fields, &request.data)
                .map_err(|e| DentalCommandError::Internal(e.to_string()))?;
        }
        _ => {
            return Err(DentalCommandError::Validation(
                "Format must be 'xlsx' or 'csv'".into(),
            ));
        }
    }

    Ok(ExportResult {
        path: request.output_path,
        rows_exported: row_count,
        format: request.format,
    })
}

// ── Import Commands ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub rows: Vec<serde_json::Value>,
    pub row_count: usize,
    pub columns: Vec<String>,
    pub format: String,
}

#[tauri::command]
pub async fn import_data(
    path: String,
    sheet_name: Option<String>,
) -> Result<ImportResult, DentalCommandError> {
    let file_path = PathBuf::from(&path);
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "xlsx" | "xls" | "xlsm" | "ods" => {
            let rows = if let Some(sheet) = sheet_name {
                excel_reader::read_excel_sheet(&file_path, &sheet)
            } else {
                excel_reader::read_excel(&file_path)
            }
            .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

            let columns: Vec<String> = rows
                .first()
                .and_then(|row| row.as_object())
                .map(|obj| obj.keys().cloned().collect())
                .unwrap_or_default();

            let row_count = rows.len();

            Ok(ImportResult {
                rows,
                row_count,
                columns,
                format: "excel".into(),
            })
        }
        "csv" => {
            let rows = csv_handler::import_from_csv(&file_path)
                .map_err(|e| DentalCommandError::Internal(e.to_string()))?;

            let columns: Vec<String> = rows
                .first()
                .and_then(|row| row.as_object())
                .map(|obj| obj.keys().cloned().collect())
                .unwrap_or_default();

            let row_count = rows.len();

            Ok(ImportResult {
                rows,
                row_count,
                columns,
                format: "csv".into(),
            })
        }
        _ => Err(DentalCommandError::Validation(
            format!("Unsupported file format: .{}. Use .xlsx, .xls, .csv, or .ods", ext),
        )),
    }
}

#[tauri::command]
pub async fn import_get_sheets(
    path: String,
) -> Result<Vec<dental_export::excel_reader::SheetInfo>, DentalCommandError> {
    let file_path = PathBuf::from(&path);
    excel_reader::get_sheet_info(&file_path)
        .map_err(|e| DentalCommandError::Internal(e.to_string()))
}

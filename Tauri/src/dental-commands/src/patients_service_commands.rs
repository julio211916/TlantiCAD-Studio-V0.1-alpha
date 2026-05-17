//! Patients Service IPC Commands
//!
//! Provides Tauri commands for patient operations with validation:
//! - Create/update patients with validation
//! - Patient search and filtering
//! - Patient statistics

use crate::{DentalCommandError, DentalCommandResult, DentalState};
use dental_core::models::{CreatePatient, Patient, PatientFilters, PatientListItem, UpdatePatient};
use dental_core::Gender;
use patients::PatientService;
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

/// Create a patient with validation
#[tauri::command]
pub async fn patients_create_validated(
    state: State<'_, DentalState>,
    input: CreatePatient,
) -> DentalCommandResult<Patient> {
    let service = PatientService::new(state.db.pool().clone());
    service
        .create(input)
        .map_err(|e| DentalCommandError::Validation(e.to_string()))
}

/// Get a patient by ID
#[tauri::command]
pub async fn patients_get(
    state: State<'_, DentalState>,
    patient_id: Uuid,
) -> DentalCommandResult<Patient> {
    let service = PatientService::new(state.db.pool().clone());
    service
        .get(patient_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Update a patient with validation
#[tauri::command]
pub async fn patients_update_validated(
    state: State<'_, DentalState>,
    patient_id: Uuid,
    input: UpdatePatient,
) -> DentalCommandResult<Patient> {
    let service = PatientService::new(state.db.pool().clone());
    service
        .update(patient_id, input)
        .map_err(|e| DentalCommandError::Validation(e.to_string()))
}

/// Delete a patient
#[tauri::command]
pub async fn patients_delete(
    state: State<'_, DentalState>,
    patient_id: Uuid,
) -> DentalCommandResult<()> {
    let service = PatientService::new(state.db.pool().clone());
    service
        .delete(patient_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// List patients with filters and pagination
#[derive(Debug, Serialize)]
pub struct PaginatedPatients {
    pub items: Vec<PatientListItem>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[tauri::command]
pub async fn patients_list(
    state: State<'_, DentalState>,
    query: Option<String>,
    gender: Option<String>,
    active_only: Option<bool>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> DentalCommandResult<PaginatedPatients> {
    let service = PatientService::new(state.db.pool().clone());
    
    let parsed_gender = gender.and_then(|g| g.parse::<Gender>().ok());
    
    let filters = PatientFilters {
        query,
        gender: parsed_gender,
        active_only,
        ..Default::default()
    };
    
    let p = page.unwrap_or(1);
    let pp = per_page.unwrap_or(20);
    
    let result = service
        .list(filters, p, pp)
        .map_err(|e| DentalCommandError::Database(e.to_string()))?;
    
    Ok(PaginatedPatients {
        items: result.items,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    })
}

/// Search patients
#[tauri::command]
pub async fn patients_search(
    state: State<'_, DentalState>,
    query: String,
    limit: Option<usize>,
) -> DentalCommandResult<Vec<PatientListItem>> {
    let service = PatientService::new(state.db.pool().clone());
    service
        .search(&query, limit.unwrap_or(10))
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Count patients
#[tauri::command]
pub async fn patients_count(
    state: State<'_, DentalState>,
    active_only: Option<bool>,
) -> DentalCommandResult<i64> {
    let service = PatientService::new(state.db.pool().clone());
    service
        .count(active_only.unwrap_or(true))
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

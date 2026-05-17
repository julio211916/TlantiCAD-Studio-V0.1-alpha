//! Patient Tauri commands

use tauri::State;
use uuid::Uuid;

use dental_core::models::{CreatePatient, Patient, PatientFilters, PatientListItem, UpdatePatient};
use dental_database::repositories::{Pagination, PaginatedResult, PatientRepository};

use crate::{CommandResult, DentalCommandError, DentalState};

/// Create a new patient
#[tauri::command]
pub fn patient_create(
    state: State<'_, DentalState>,
    data: CreatePatient,
) -> CommandResult<Patient> {
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.create(data).map_err(|e| e.into())
}

/// Get patient by ID
#[tauri::command]
pub fn patient_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<Patient> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.find_by_id(uuid).map_err(|e| e.into())
}

/// Get patient by patient number
#[tauri::command]
pub fn patient_get_by_number(
    state: State<'_, DentalState>,
    patient_number: String,
) -> CommandResult<Patient> {
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.find_by_patient_number(&patient_number).map_err(|e| e.into())
}

/// Update patient
#[tauri::command]
pub fn patient_update(
    state: State<'_, DentalState>,
    id: String,
    data: UpdatePatient,
) -> CommandResult<Patient> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.update(uuid, data).map_err(|e| e.into())
}

/// Delete patient (soft delete)
#[tauri::command]
pub fn patient_delete(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.delete(uuid).map_err(|e| e.into())
}

/// List patients with pagination
#[tauri::command]
pub fn patient_list(
    state: State<'_, DentalState>,
    page: u32,
    per_page: u32,
    filters: Option<PatientFilters>,
) -> CommandResult<PaginatedResult<PatientListItem>> {
    let pagination = Pagination::new(page, per_page);
    let filters = filters.unwrap_or_default();
    
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.list(filters, pagination).map_err(|e| e.into())
}

/// Search patients
#[tauri::command]
pub fn patient_search(
    state: State<'_, DentalState>,
    query: String,
    limit: Option<usize>,
) -> CommandResult<Vec<PatientListItem>> {
    let limit = limit.unwrap_or(10);
    
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.search(&query, limit).map_err(|e| e.into())
}

/// Count patients
#[tauri::command]
pub fn patient_count(
    state: State<'_, DentalState>,
    active_only: bool,
) -> CommandResult<i64> {
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.count(active_only).map_err(|e| e.into())
}

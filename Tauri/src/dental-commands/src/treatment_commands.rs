//! Treatment Tauri commands

use tauri::State;
use uuid::Uuid;

use dental_core::models::{CreateTreatment, Treatment, TreatmentWithDetails};
use dental_core::TreatmentStatus;
use dental_database::repositories::TreatmentRepository;
use serde::{Deserialize, Serialize};

use crate::{CommandResult, DentalCommandError, DentalState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentStats {
    pub name: String,
    pub count: i64,
    pub revenue: rust_decimal::Decimal,
}

/// Create a new treatment
#[tauri::command]
pub fn treatment_create(
    state: State<'_, DentalState>,
    data: CreateTreatment,
) -> CommandResult<Treatment> {
    let repo = TreatmentRepository::new(state.db.pool().clone());
    repo.create(data).map_err(|e| e.into())
}

/// Get treatment by ID
#[tauri::command]
pub fn treatment_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<Treatment> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid treatment ID".into()))?;
    
    let repo = TreatmentRepository::new(state.db.pool().clone());
    repo.find_by_id(uuid).map_err(|e| e.into())
}

/// Update treatment status
#[tauri::command]
pub fn treatment_update_status(
    state: State<'_, DentalState>,
    id: String,
    status: TreatmentStatus,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid treatment ID".into()))?;
    
    let repo = TreatmentRepository::new(state.db.pool().clone());
    repo.update_status(uuid, status).map_err(|e| e.into())
}

/// Complete treatment
#[tauri::command]
pub fn treatment_complete(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid treatment ID".into()))?;
    
    let repo = TreatmentRepository::new(state.db.pool().clone());
    repo.update_status(uuid, TreatmentStatus::Completed).map_err(|e| e.into())
}

/// Get treatments for a patient
#[tauri::command]
pub fn treatment_list_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<TreatmentWithDetails>> {
    let uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let repo = TreatmentRepository::new(state.db.pool().clone());
    repo.list_by_patient(uuid).map_err(|e| e.into())
}

/// Get treatments for an appointment
#[tauri::command]
pub fn treatment_list_by_appointment(
    state: State<'_, DentalState>,
    appointment_id: String,
) -> CommandResult<Vec<Treatment>> {
    let uuid = Uuid::parse_str(&appointment_id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = TreatmentRepository::new(state.db.pool().clone());
    repo.list_by_appointment(uuid).map_err(|e| e.into())
}

/// Get treatments by patient (alias for frontend compatibility)
#[tauri::command]
pub fn treatment_get_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<TreatmentWithDetails>> {
    treatment_list_by_patient(state, patient_id)
}

/// Get top treatments by revenue
#[tauri::command]
pub fn treatment_top_stats(
    state: State<'_, DentalState>,
    limit: Option<i64>,
) -> CommandResult<Vec<TreatmentStats>> {
    let repo = TreatmentRepository::new(state.db.pool().clone());
    let limit = limit.unwrap_or(6).max(1);

    let results = repo
        .get_top_treatments(limit)
        .map_err(DentalCommandError::from)?;
    Ok(results
        .into_iter()
        .map(|row| TreatmentStats {
            name: row.procedure_name,
            count: row.count,
            revenue: row.revenue,
        })
        .collect())
}

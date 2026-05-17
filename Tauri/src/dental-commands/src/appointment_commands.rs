//! Appointment Tauri commands

use chrono::{DateTime, Utc};
use tauri::State;
use uuid::Uuid;

use dental_core::models::{Appointment, AppointmentListItem, CreateAppointment, UpdateAppointment};
use dental_core::AppointmentStatus;
use dental_database::repositories::AppointmentRepository;

use crate::{CommandResult, DentalCommandError, DentalState};

/// Create a new appointment
#[tauri::command]
pub fn appointment_create(
    state: State<'_, DentalState>,
    data: CreateAppointment,
) -> CommandResult<Appointment> {
    let created_by = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.create(data, created_by).map_err(|e| e.into())
}

/// Get appointment by ID
#[tauri::command]
pub fn appointment_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<Appointment> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.find_by_id(uuid).map_err(|e| e.into())
}

/// Update appointment
#[tauri::command]
pub fn appointment_update(
    state: State<'_, DentalState>,
    id: String,
    data: UpdateAppointment,
) -> CommandResult<Appointment> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.update(uuid, data).map_err(|e| e.into())
}

/// Update appointment status
#[tauri::command]
pub fn appointment_update_status(
    state: State<'_, DentalState>,
    id: String,
    status: AppointmentStatus,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.update_status(uuid, status).map_err(|e| e.into())
}

/// Check in patient
#[tauri::command]
pub fn appointment_check_in(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.update_status(uuid, AppointmentStatus::CheckedIn).map_err(|e| e.into())
}

/// Start appointment
#[tauri::command]
pub fn appointment_start(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.update_status(uuid, AppointmentStatus::InProgress).map_err(|e| e.into())
}

/// Complete appointment
#[tauri::command]
pub fn appointment_complete(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.update_status(uuid, AppointmentStatus::Completed).map_err(|e| e.into())
}

/// Cancel appointment
#[tauri::command]
pub fn appointment_cancel(
    state: State<'_, DentalState>,
    id: String,
    reason: Option<String>,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    
    if reason.is_some() {
        let data = UpdateAppointment {
            cancel_reason: reason,
            status: Some(AppointmentStatus::Cancelled),
            ..Default::default()
        };
        repo.update(uuid, data)?;
    } else {
        repo.update_status(uuid, AppointmentStatus::Cancelled)?;
    }
    
    Ok(())
}

/// Delete appointment
#[tauri::command]
pub fn appointment_delete(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid appointment ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.delete(uuid).map_err(|e| e.into())
}

/// Get appointments by date range
#[tauri::command]
pub fn appointment_list_by_date(
    state: State<'_, DentalState>,
    start: String,
    end: String,
    doctor_id: Option<String>,
) -> CommandResult<Vec<AppointmentListItem>> {
    let start_dt = DateTime::parse_from_rfc3339(&start)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| DentalCommandError::Validation("Invalid start date".into()))?;
    
    let end_dt = DateTime::parse_from_rfc3339(&end)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| DentalCommandError::Validation("Invalid end date".into()))?;
    
    let doctor_uuid = doctor_id
        .map(|id| Uuid::parse_str(&id))
        .transpose()
        .map_err(|_| DentalCommandError::Validation("Invalid doctor ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.list_by_date_range(start_dt, end_dt, doctor_uuid).map_err(|e| e.into())
}

/// Get appointments for a patient
#[tauri::command]
pub fn appointment_list_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<AppointmentListItem>> {
    let uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.list_by_patient(uuid).map_err(|e| e.into())
}

/// Get today's appointments
#[tauri::command]
pub fn appointment_get_today(
    state: State<'_, DentalState>,
    doctor_id: Option<String>,
) -> CommandResult<Vec<AppointmentListItem>> {
    let doctor_uuid = doctor_id
        .map(|id| Uuid::parse_str(&id))
        .transpose()
        .map_err(|_| DentalCommandError::Validation("Invalid doctor ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.get_today(doctor_uuid).map_err(|e| e.into())
}

/// List appointments (alias for frontend compatibility)
/// Uses today's date if no date provided
#[tauri::command]
pub fn appointment_list(
    state: State<'_, DentalState>,
    date: Option<String>,
    doctor_id: Option<String>,
) -> CommandResult<Vec<AppointmentListItem>> {
    let doctor_uuid = doctor_id
        .map(|id| Uuid::parse_str(&id))
        .transpose()
        .map_err(|_| DentalCommandError::Validation("Invalid doctor ID".into()))?;
    
    let repo = AppointmentRepository::new(state.db.pool().clone());
    
    if let Some(date_str) = date {
        // Parse date and create range for that day
        let start_dt = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date_str))
            .or_else(|_| DateTime::parse_from_rfc3339(&date_str))
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| DentalCommandError::Validation("Invalid date format".into()))?;
        
        let end_dt = start_dt + chrono::Duration::days(1);
        repo.list_by_date_range(start_dt, end_dt, doctor_uuid).map_err(|e| e.into())
    } else {
        // Return today's appointments
        repo.get_today(doctor_uuid).map_err(|e| e.into())
    }
}

/// Get appointments by patient (alias for frontend compatibility)
#[tauri::command]
pub fn appointment_get_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<AppointmentListItem>> {
    appointment_list_by_patient(state, patient_id)
}

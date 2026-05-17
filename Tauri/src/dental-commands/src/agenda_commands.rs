//! Agenda Service IPC Commands
//!
//! Provides Tauri commands for scheduling operations:
//! - Daily schedules
//! - Today's appointments
//! - Appointment management

use crate::{DentalCommandError, DentalCommandResult, DentalState};
use agenda::AgendaService;
use chrono::{DateTime, NaiveDate, Utc};
use dental_core::models::{AppointmentListItem, CreateAppointment, RescheduleAppointment, UpdateAppointment};
use dental_core::AppointmentStatus;
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

/// Get appointments for a date range
#[tauri::command]
pub async fn agenda_list_by_date_range(
    state: State<'_, DentalState>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    doctor_id: Option<Uuid>,
) -> DentalCommandResult<Vec<AppointmentListItem>> {
    let service = AgendaService::new(state.db.pool().clone());
    service
        .list_by_date_range(start, end, doctor_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Get today's appointments
#[tauri::command]
pub async fn agenda_get_today(
    state: State<'_, DentalState>,
    doctor_id: Option<Uuid>,
) -> DentalCommandResult<Vec<AppointmentListItem>> {
    let service = AgendaService::new(state.db.pool().clone());
    service
        .today(doctor_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Get appointments by patient
#[tauri::command]
pub async fn agenda_list_by_patient(
    state: State<'_, DentalState>,
    patient_id: Uuid,
) -> DentalCommandResult<Vec<AppointmentListItem>> {
    let service = AgendaService::new(state.db.pool().clone());
    service
        .list_by_patient(patient_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Schedule a new appointment
#[tauri::command]
pub async fn agenda_schedule(
    state: State<'_, DentalState>,
    data: CreateAppointment,
) -> DentalCommandResult<dental_core::Appointment> {
    let user_id = state
        .get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".to_string()))?;
    
    let service = AgendaService::new(state.db.pool().clone());
    service
        .schedule(data, user_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Update an appointment
#[tauri::command]
pub async fn agenda_update(
    state: State<'_, DentalState>,
    appointment_id: Uuid,
    data: UpdateAppointment,
) -> DentalCommandResult<dental_core::Appointment> {
    let service = AgendaService::new(state.db.pool().clone());
    service
        .update(appointment_id, data)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Reschedule an appointment
#[tauri::command]
pub async fn agenda_reschedule(
    state: State<'_, DentalState>,
    appointment_id: Uuid,
    data: RescheduleAppointment,
) -> DentalCommandResult<dental_core::Appointment> {
    let service = AgendaService::new(state.db.pool().clone());
    service
        .reschedule(appointment_id, data)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Cancel an appointment
#[tauri::command]
pub async fn agenda_cancel(
    state: State<'_, DentalState>,
    appointment_id: Uuid,
    reason: Option<String>,
) -> DentalCommandResult<()> {
    let service = AgendaService::new(state.db.pool().clone());
    service
        .cancel(appointment_id, reason)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Update appointment status
#[tauri::command]
pub async fn agenda_set_status(
    state: State<'_, DentalState>,
    appointment_id: Uuid,
    status: AppointmentStatus,
) -> DentalCommandResult<()> {
    let service = AgendaService::new(state.db.pool().clone());
    service
        .set_status(appointment_id, status)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Count appointments by status for a date
#[derive(Debug, Serialize)]
pub struct AppointmentCountByStatus {
    pub scheduled: i32,
    pub confirmed: i32,
    pub checked_in: i32,
    pub in_progress: i32,
    pub completed: i32,
    pub cancelled: i32,
    pub no_show: i32,
}

#[tauri::command]
pub async fn agenda_count_by_date(
    state: State<'_, DentalState>,
    date: NaiveDate,
) -> DentalCommandResult<AppointmentCountByStatus> {
    let service = AgendaService::new(state.db.pool().clone());
    let counts = service
        .count_by_date(date)
        .map_err(|e| DentalCommandError::Database(e.to_string()))?;
    
    Ok(AppointmentCountByStatus {
        scheduled: *counts.get(&AppointmentStatus::Scheduled).unwrap_or(&0),
        confirmed: *counts.get(&AppointmentStatus::Confirmed).unwrap_or(&0),
        checked_in: *counts.get(&AppointmentStatus::CheckedIn).unwrap_or(&0),
        in_progress: *counts.get(&AppointmentStatus::InProgress).unwrap_or(&0),
        completed: *counts.get(&AppointmentStatus::Completed).unwrap_or(&0),
        cancelled: *counts.get(&AppointmentStatus::Cancelled).unwrap_or(&0),
        no_show: *counts.get(&AppointmentStatus::NoShow).unwrap_or(&0),
    })
}

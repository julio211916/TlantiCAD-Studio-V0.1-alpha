//! Clinical Service IPC Commands
//!
//! Provides Tauri commands for clinical note operations:
//! - Create/update clinical notes
//! - List notes by patient/appointment

use crate::{DentalCommandError, DentalCommandResult, DentalState};
use clinical::ClinicalService;
use dental_core::ClinicalNoteType;
use dental_core::models::{ClinicalNote, ClinicalNoteFilters, CreateClinicalNote, UpdateClinicalNote};
use tauri::State;
use uuid::Uuid;

/// Create a new clinical note
#[tauri::command]
pub async fn clinical_note_create(
    state: State<'_, DentalState>,
    patient_id: Uuid,
    appointment_id: Option<Uuid>,
    note_type: ClinicalNoteType,
    content: String,
    attachments: Option<Vec<String>>,
) -> DentalCommandResult<ClinicalNote> {
    let user_id = state
        .get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".to_string()))?;
    
    let create_note = CreateClinicalNote {
        patient_id,
        appointment_id,
        user_id,
        note_type,
        content,
        attachments,
    };
    
    let service = ClinicalService::new(state.db.pool().clone());
    service
        .create_note(create_note)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Get a clinical note by ID
#[tauri::command]
pub async fn clinical_note_get(
    state: State<'_, DentalState>,
    note_id: Uuid,
) -> DentalCommandResult<ClinicalNote> {
    let service = ClinicalService::new(state.db.pool().clone());
    service
        .get_note(note_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// List clinical notes by patient
#[tauri::command]
pub async fn clinical_notes_by_patient(
    state: State<'_, DentalState>,
    patient_id: Uuid,
) -> DentalCommandResult<Vec<ClinicalNote>> {
    let service = ClinicalService::new(state.db.pool().clone());
    service
        .list_notes_by_patient(patient_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// List clinical notes by appointment
#[tauri::command]
pub async fn clinical_notes_by_appointment(
    state: State<'_, DentalState>,
    appointment_id: Uuid,
) -> DentalCommandResult<Vec<ClinicalNote>> {
    let service = ClinicalService::new(state.db.pool().clone());
    service
        .list_notes_by_appointment(appointment_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Update a clinical note
#[tauri::command]
pub async fn clinical_note_update(
    state: State<'_, DentalState>,
    note_id: Uuid,
    note_type: Option<ClinicalNoteType>,
    content: Option<String>,
    attachments: Option<Vec<String>>,
) -> DentalCommandResult<ClinicalNote> {
    let service = ClinicalService::new(state.db.pool().clone());
    
    let update = UpdateClinicalNote {
        note_type,
        content,
        attachments,
    };
    
    service
        .update_note(note_id, update)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Delete a clinical note
#[tauri::command]
pub async fn clinical_note_delete(
    state: State<'_, DentalState>,
    note_id: Uuid,
) -> DentalCommandResult<()> {
    let service = ClinicalService::new(state.db.pool().clone());
    service
        .delete_note(note_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// List clinical notes with filters
#[tauri::command]
pub async fn clinical_notes_list(
    state: State<'_, DentalState>,
    patient_id: Option<Uuid>,
    appointment_id: Option<Uuid>,
    user_id: Option<Uuid>,
    note_type: Option<ClinicalNoteType>,
) -> DentalCommandResult<Vec<ClinicalNote>> {
    let service = ClinicalService::new(state.db.pool().clone());
    let filters = ClinicalNoteFilters {
        patient_id,
        appointment_id,
        user_id,
        note_type,
        date_from: None,
        date_to: None,
    };
    
    service
        .list_notes(filters)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

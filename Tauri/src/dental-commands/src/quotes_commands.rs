//! Quotes (presupuestos) Tauri commands

use tauri::State;
use uuid::Uuid;

use dental_core::models::{CreateQuote, QuoteWithItems};
use dental_database::repositories::QuoteRepository;

use crate::{CommandResult, DentalCommandError, DentalState};

/// Create a new quote
#[tauri::command]
pub fn quote_create(
    state: State<'_, DentalState>,
    data: CreateQuote,
) -> CommandResult<QuoteWithItems> {
    let user_id = state
        .get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;

    let repo = QuoteRepository::new(state.db.pool().clone());
    repo.create(data, user_id).map_err(|e| e.into())
}

/// Get quote by id
#[tauri::command]
pub fn quote_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<QuoteWithItems> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid quote ID".into()))?;

    let repo = QuoteRepository::new(state.db.pool().clone());
    repo.get_with_items(uuid).map_err(|e| e.into())
}

/// List quotes by patient
#[tauri::command]
pub fn quote_list_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<QuoteWithItems>> {
    let uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;

    let repo = QuoteRepository::new(state.db.pool().clone());
    repo.list_by_patient(uuid).map_err(|e| e.into())
}

/// Delete quote
#[tauri::command]
pub fn quote_delete(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid quote ID".into()))?;

    let repo = QuoteRepository::new(state.db.pool().clone());
    repo.delete(uuid).map_err(|e| e.into())
}

//! Invoice Tauri commands

use tauri::State;
use uuid::Uuid;

use dental_core::models::{CreateInvoice, CreatePayment, Invoice, InvoiceItem, InvoiceListItem, Payment};
use dental_database::repositories::InvoiceRepository;

use crate::{CommandResult, DentalCommandError, DentalState};

/// Create a new invoice
#[tauri::command]
pub fn invoice_create(
    state: State<'_, DentalState>,
    data: CreateInvoice,
) -> CommandResult<Invoice> {
    let created_by = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = InvoiceRepository::new(state.db.pool().clone());
    repo.create(data, created_by).map_err(|e| e.into())
}

/// Get invoice by ID
#[tauri::command]
pub fn invoice_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<Invoice> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid invoice ID".into()))?;
    
    let repo = InvoiceRepository::new(state.db.pool().clone());
    repo.find_by_id(uuid).map_err(|e| e.into())
}

/// Get invoice items
#[tauri::command]
pub fn invoice_get_items(
    state: State<'_, DentalState>,
    invoice_id: String,
) -> CommandResult<Vec<InvoiceItem>> {
    let uuid = Uuid::parse_str(&invoice_id)
        .map_err(|_| DentalCommandError::Validation("Invalid invoice ID".into()))?;
    
    let repo = InvoiceRepository::new(state.db.pool().clone());
    repo.get_items(uuid).map_err(|e| e.into())
}

/// Add payment to invoice
#[tauri::command]
pub fn invoice_add_payment(
    state: State<'_, DentalState>,
    data: CreatePayment,
) -> CommandResult<Payment> {
    let received_by = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = InvoiceRepository::new(state.db.pool().clone());
    repo.add_payment(data, received_by).map_err(|e| e.into())
}

/// Get invoices for a patient
#[tauri::command]
pub fn invoice_list_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<InvoiceListItem>> {
    let uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let repo = InvoiceRepository::new(state.db.pool().clone());
    repo.list_by_patient(uuid).map_err(|e| e.into())
}

/// Get invoices by patient (alias for frontend compatibility)
#[tauri::command]
pub fn invoice_get_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<InvoiceListItem>> {
    invoice_list_by_patient(state, patient_id)
}

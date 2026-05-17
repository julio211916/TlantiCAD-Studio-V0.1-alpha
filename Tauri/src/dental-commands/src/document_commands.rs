//! Document Tauri commands

use tauri::State;
use uuid::Uuid;

use dental_core::models::{CreateDocument, CreateDocumentTemplate, Document, DocumentListItem, DocumentTemplate};
use dental_core::DocumentType;
use dental_database::repositories::DocumentRepository;

use crate::{CommandResult, DentalCommandError, DentalState};

/// Create a new document
#[tauri::command]
pub fn document_create(
    state: State<'_, DentalState>,
    data: CreateDocument,
) -> CommandResult<Document> {
    let created_by = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = DocumentRepository::new(state.db.pool().clone());
    repo.create(data, created_by).map_err(|e| e.into())
}

/// Get document by ID
#[tauri::command]
pub fn document_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<Document> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid document ID".into()))?;
    
    let repo = DocumentRepository::new(state.db.pool().clone());
    repo.find_by_id(uuid).map_err(|e| e.into())
}

/// Sign document
#[tauri::command]
pub fn document_sign(
    state: State<'_, DentalState>,
    id: String,
    signature_path: String,
    signed_by: String,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid document ID".into()))?;
    
    let repo = DocumentRepository::new(state.db.pool().clone());
    repo.sign(uuid, signature_path, signed_by).map_err(|e| e.into())
}

/// List documents for a patient
#[tauri::command]
pub fn document_list_by_patient(
    state: State<'_, DentalState>,
    patient_id: String,
) -> CommandResult<Vec<DocumentListItem>> {
    let uuid = Uuid::parse_str(&patient_id)
        .map_err(|_| DentalCommandError::Validation("Invalid patient ID".into()))?;
    
    let repo = DocumentRepository::new(state.db.pool().clone());
    repo.list_by_patient(uuid).map_err(|e| e.into())
}

/// Create document template
#[tauri::command]
pub fn template_create(
    state: State<'_, DentalState>,
    data: CreateDocumentTemplate,
) -> CommandResult<DocumentTemplate> {
    let created_by = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let repo = DocumentRepository::new(state.db.pool().clone());
    repo.create_template(data, created_by).map_err(|e| e.into())
}

/// Get template by ID
#[tauri::command]
pub fn template_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<DocumentTemplate> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid template ID".into()))?;
    
    let repo = DocumentRepository::new(state.db.pool().clone());
    repo.find_template_by_id(uuid).map_err(|e| e.into())
}

/// List templates
#[tauri::command]
pub fn template_list(
    state: State<'_, DentalState>,
    document_type: Option<DocumentType>,
) -> CommandResult<Vec<DocumentTemplate>> {
    let repo = DocumentRepository::new(state.db.pool().clone());
    repo.list_templates(document_type).map_err(|e| e.into())
}

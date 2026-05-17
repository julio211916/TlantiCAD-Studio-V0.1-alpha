//! TlantiStudio Dental - Tauri IPC Commands
//!
//! All Tauri commands for the dental clinic management system.

pub mod auth_commands;
pub mod patient_commands;
pub mod appointment_commands;
pub mod treatment_commands;
pub mod invoice_commands;
pub mod inventory_commands;
pub mod document_commands;
pub mod dashboard_commands;
pub mod odontogram_commands;
pub mod quotes_commands;
pub mod support_ticket_commands;

// Service layer commands
pub mod accounting_commands;
pub mod agenda_commands;
pub mod clinical_commands;
pub mod patients_service_commands;
pub mod pos_commands;

// Imaging, export, system commands
pub mod imaging_commands;
pub mod export_commands;
pub mod system_commands;
pub mod mimesis_commands;

// DICOM networking & PACS
pub mod dicom_network_commands;
pub mod orthanc_commands;

use dental_database::{Database, DatabaseConfig};
use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;

/// Application state
pub struct DentalState {
    pub db: Arc<Database>,
    pub current_user_id: RwLock<Option<uuid::Uuid>>,
    pub current_clinic_id: RwLock<Option<uuid::Uuid>>,
}

impl DentalState {
    pub fn new(db_path: &str) -> Result<Self, DentalCommandError> {
        let config = DatabaseConfig {
            path: db_path.to_string(),
            pool_size: 10,
            create_if_missing: true,
            run_migrations: true,
        };
        
        let db = Database::new(config)
            .map_err(|e| DentalCommandError::Database(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
            current_user_id: RwLock::new(None),
            current_clinic_id: RwLock::new(None),
        })
    }
    
    pub fn set_current_user(&self, user_id: uuid::Uuid) {
        *self.current_user_id.write() = Some(user_id);
    }
    
    pub fn get_current_user(&self) -> Option<uuid::Uuid> {
        *self.current_user_id.read()
    }
    
    pub fn set_current_clinic(&self, clinic_id: uuid::Uuid) {
        *self.current_clinic_id.write() = Some(clinic_id);
    }
}

/// Command error type
#[derive(Error, Debug)]
pub enum DentalCommandError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl serde::Serialize for DentalCommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<dental_database::DbError> for DentalCommandError {
    fn from(e: dental_database::DbError) -> Self {
        match e {
            dental_database::DbError::NotFound(msg) => DentalCommandError::NotFound(msg),
            _ => DentalCommandError::Database(e.to_string()),
        }
    }
}

impl From<dental_core::DentalError> for DentalCommandError {
    fn from(e: dental_core::DentalError) -> Self {
        DentalCommandError::Internal(e.to_string())
    }
}

/// Result type for commands
pub type CommandResult<T> = Result<T, DentalCommandError>;

/// Result type for dental commands (alias)
pub type DentalCommandResult<T> = Result<T, DentalCommandError>;

// Re-export all commands
pub use auth_commands::*;
pub use patient_commands::*;
pub use appointment_commands::*;
pub use treatment_commands::*;
pub use invoice_commands::*;
pub use inventory_commands::*;
pub use document_commands::*;
pub use dashboard_commands::*;
pub use odontogram_commands::*;
pub use quotes_commands::*;
pub use support_ticket_commands::*;

// Service layer commands
pub use accounting_commands::*;
pub use agenda_commands::*;
pub use clinical_commands::*;
pub use patients_service_commands::*;
pub use pos_commands::*;

// Imaging, export, system re-exports
pub use imaging_commands::*;
pub use export_commands::*;
pub use system_commands::*;
pub use mimesis_commands::*;

// DICOM networking & PACS re-exports
pub use dicom_network_commands::*;
pub use orthanc_commands::*;

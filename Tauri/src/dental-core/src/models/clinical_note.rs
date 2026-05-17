//! Clinical note domain models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::ClinicalNoteType;

/// Clinical note entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalNote {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub user_id: Uuid,
    pub note_type: ClinicalNoteType,
    pub content: String,
    pub attachments: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl ClinicalNote {
    pub fn new(
        patient_id: Uuid,
        appointment_id: Option<Uuid>,
        user_id: Uuid,
        note_type: ClinicalNoteType,
        content: String,
        attachments: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            patient_id,
            appointment_id,
            user_id,
            note_type,
            content,
            attachments,
            created_at: Utc::now(),
        }
    }
}

/// Create clinical note DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClinicalNote {
    pub patient_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub user_id: Uuid,
    pub note_type: ClinicalNoteType,
    pub content: String,
    pub attachments: Option<Vec<String>>,
}

/// Update clinical note DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateClinicalNote {
    pub note_type: Option<ClinicalNoteType>,
    pub content: Option<String>,
    pub attachments: Option<Vec<String>>,
}

/// Clinical note filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClinicalNoteFilters {
    pub patient_id: Option<Uuid>,
    pub appointment_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub note_type: Option<ClinicalNoteType>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

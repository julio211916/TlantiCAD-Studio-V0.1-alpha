//! Document and template domain models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::DocumentType;

/// Document entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    
    /// Reference to patient
    pub patient_id: Uuid,
    
    /// Reference to template used (if any)
    pub template_id: Option<Uuid>,
    
    /// Reference to appointment (if related)
    pub appointment_id: Option<Uuid>,
    
    /// Document type
    pub document_type: DocumentType,
    
    /// Title
    pub title: String,
    
    /// Content (markdown, HTML, or plain text)
    pub content: String,
    
    /// File path (for PDFs, images, etc.)
    pub file_path: Option<String>,
    
    /// File mime type
    pub mime_type: Option<String>,
    
    /// File size in bytes
    pub file_size: Option<i64>,
    
    /// Is signed
    pub signed: bool,
    
    /// Signature image path
    pub signature_path: Option<String>,
    
    /// Signature date
    pub signature_date: Option<DateTime<Utc>>,
    
    /// Signed by (patient name or user name)
    pub signed_by: Option<String>,
    
    /// Is visible to patient (via portal)
    pub patient_visible: bool,
    
    /// Notes
    pub notes: Option<String>,
    
    /// Created by user
    pub created_by: Uuid,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn new(
        patient_id: Uuid,
        document_type: DocumentType,
        title: String,
        content: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_id,
            template_id: None,
            appointment_id: None,
            document_type,
            title,
            content,
            file_path: None,
            mime_type: None,
            file_size: None,
            signed: false,
            signature_path: None,
            signature_date: None,
            signed_by: None,
            patient_visible: false,
            notes: None,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Document template entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    pub id: Uuid,
    
    /// Template name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Document type this template is for
    pub document_type: DocumentType,
    
    /// Category for organization
    pub category: Option<String>,
    
    /// Template content (with variable placeholders)
    pub content: String,
    
    /// Available variables (JSON array)
    pub variables: Vec<TemplateVariable>,
    
    /// Header content (for PDFs)
    pub header: Option<String>,
    
    /// Footer content (for PDFs)
    pub footer: Option<String>,
    
    /// CSS styles (for HTML/PDF)
    pub styles: Option<String>,
    
    /// Is active
    pub active: bool,
    
    /// Is system template (cannot be deleted)
    pub is_system: bool,
    
    /// Created by user
    pub created_by: Uuid,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DocumentTemplate {
    pub fn new(
        name: String,
        document_type: DocumentType,
        content: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            document_type,
            category: None,
            content,
            variables: Vec::new(),
            header: None,
            footer: None,
            styles: None,
            active: true,
            is_system: false,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable key (e.g., "patient_name")
    pub key: String,
    
    /// Display label (e.g., "Patient Name")
    pub label: String,
    
    /// Variable type (text, date, number, etc.)
    pub var_type: TemplateVariableType,
    
    /// Default value
    pub default_value: Option<String>,
    
    /// Is required
    pub required: bool,
    
    /// Description/help text
    pub description: Option<String>,
}

/// Template variable types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateVariableType {
    Text,
    Number,
    Date,
    DateTime,
    Boolean,
    Currency,
    Selection,
    MultiLine,
}

/// Consent form - specialized document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentForm {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub document_id: Uuid,
    pub consent_type: String,
    pub procedure_id: Option<Uuid>,
    pub risks_explained: bool,
    pub alternatives_explained: bool,
    pub questions_answered: bool,
    pub patient_signature: Option<String>,
    pub patient_signed_at: Option<DateTime<Utc>>,
    pub witness_name: Option<String>,
    pub witness_signature: Option<String>,
    pub witness_signed_at: Option<DateTime<Utc>>,
    pub doctor_signature: Option<String>,
    pub doctor_signed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Prescription - specialized document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prescription {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub document_id: Uuid,
    pub doctor_id: Uuid,
    pub date: DateTime<Utc>,
    pub medications: Vec<PrescriptionMedication>,
    pub diagnosis: Option<String>,
    pub instructions: Option<String>,
    pub valid_days: i32,
    pub created_at: DateTime<Utc>,
}

/// Prescription medication item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrescriptionMedication {
    pub medication: String,
    pub dosage: String,
    pub frequency: String,
    pub duration: String,
    pub quantity: Option<i32>,
    pub instructions: Option<String>,
}

/// Lab order - specialized document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabOrder {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub document_id: Uuid,
    pub doctor_id: Uuid,
    pub lab_id: Option<Uuid>,
    pub order_number: String,
    pub date: DateTime<Utc>,
    pub items: Vec<LabOrderItem>,
    pub special_instructions: Option<String>,
    pub status: LabOrderStatus,
    pub expected_date: Option<DateTime<Utc>>,
    pub received_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Lab order item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabOrderItem {
    pub description: String,
    pub tooth_number: Option<i32>,
    pub shade: Option<String>,
    pub material: Option<String>,
    pub specifications: Option<String>,
}

/// Lab order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabOrderStatus {
    Draft,
    Sent,
    InProgress,
    Ready,
    Received,
    Cancelled,
}

/// Document list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListItem {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub patient_name: String,
    pub document_type: DocumentType,
    pub title: String,
    pub signed: bool,
    pub created_at: DateTime<Utc>,
}

/// Create document DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocument {
    pub patient_id: Uuid,
    pub template_id: Option<Uuid>,
    pub appointment_id: Option<Uuid>,
    pub document_type: DocumentType,
    pub title: String,
    pub content: String,
    pub patient_visible: Option<bool>,
    pub notes: Option<String>,
}

/// Create template DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentTemplate {
    pub name: String,
    pub description: Option<String>,
    pub document_type: DocumentType,
    pub category: Option<String>,
    pub content: String,
    pub variables: Option<Vec<TemplateVariable>>,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub styles: Option<String>,
}

/// Generate document from template DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateFromTemplate {
    pub patient_id: Uuid,
    pub template_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub title: Option<String>,
    pub variable_values: std::collections::HashMap<String, String>,
    pub generate_pdf: bool,
}

/// Document filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentFilters {
    pub patient_id: Option<Uuid>,
    pub document_type: Option<DocumentType>,
    pub signed: Option<bool>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
}

/// Template filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateFilters {
    pub document_type: Option<DocumentType>,
    pub category: Option<String>,
    pub active_only: Option<bool>,
    pub query: Option<String>,
}

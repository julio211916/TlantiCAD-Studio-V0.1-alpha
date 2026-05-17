//! Error types for TlantiStudio Dental

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DentalError {
    #[error("Patient not found: {0}")]
    PatientNotFound(String),
    
    #[error("Appointment not found: {0}")]
    AppointmentNotFound(String),
    
    #[error("Treatment not found: {0}")]
    TreatmentNotFound(String),
    
    #[error("Product not found: {0}")]
    ProductNotFound(String),
    
    #[error("Invoice not found: {0}")]
    InvoiceNotFound(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Insufficient stock: {product} (available: {available}, requested: {requested})")]
    InsufficientStock {
        product: String,
        available: i32,
        requested: i32,
    },
    
    #[error("Payment error: {0}")]
    PaymentError(String),
    
    #[error("Scheduling conflict: {0}")]
    SchedulingConflict(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl serde::Serialize for DentalError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

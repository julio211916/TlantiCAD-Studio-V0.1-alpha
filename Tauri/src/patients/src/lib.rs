//! Patient services

pub mod error;
pub mod service;

pub use error::{PatientServiceError, PatientServiceResult};
pub use service::PatientService;

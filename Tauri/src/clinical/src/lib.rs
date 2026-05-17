//! Clinical services

pub mod error;
pub mod service;

pub use error::{ClinicalError, ClinicalResult};
pub use service::ClinicalService;

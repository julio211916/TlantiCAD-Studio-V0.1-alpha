//! Accounting services for TlantiStudio Dental

pub mod error;
pub mod service;

pub use error::{AccountingError, AccountingResult};
pub use service::AccountingService;

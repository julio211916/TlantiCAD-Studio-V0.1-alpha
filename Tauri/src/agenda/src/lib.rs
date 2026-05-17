//! Agenda and scheduling services

pub mod error;
pub mod service;

pub use error::{AgendaError, AgendaResult};
pub use service::AgendaService;

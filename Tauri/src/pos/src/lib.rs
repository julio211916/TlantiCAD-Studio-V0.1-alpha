//! Point of Sale services

pub mod error;
pub mod models;
pub mod service;

pub use error::{PosError, PosResult};
pub use models::{SaleItemInput, SalePaymentInput, SaleRequest, SaleResult};
pub use service::PosService;

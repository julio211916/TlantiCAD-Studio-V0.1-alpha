//! Domain models for TlantiStudio Dental

mod patient;
mod appointment;
mod treatment;
pub mod odontogram;
mod product;
mod invoice;
mod document;
mod user;
mod clinic;
mod procedure;
mod clinical_note;
mod quote;

pub use patient::*;
pub use appointment::*;
pub use treatment::*;
pub use odontogram::*;
pub use product::*;
pub use invoice::*;
pub use document::*;
pub use user::*;
pub use clinic::*;
pub use procedure::*;
pub use clinical_note::*;
pub use quote::*;

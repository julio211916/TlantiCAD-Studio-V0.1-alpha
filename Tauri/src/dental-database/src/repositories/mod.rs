//! Database repositories

pub mod patient_repository;
pub mod appointment_repository;
pub mod treatment_repository;
pub mod invoice_repository;
pub mod product_repository;
pub mod document_repository;
pub mod odontogram_repository;
pub mod clinical_note_repository;
pub mod quote_repository;

pub use patient_repository::PatientRepository;
pub use appointment_repository::AppointmentRepository;
pub use treatment_repository::TreatmentRepository;
pub use invoice_repository::InvoiceRepository;
pub use product_repository::ProductRepository;
pub use document_repository::DocumentRepository;
pub use odontogram_repository::{OdontogramRepository, PeriodontogramRepository};
pub use clinical_note_repository::ClinicalNoteRepository;
pub use quote_repository::QuoteRepository;

use crate::DbPool;

/// Repository factory
pub struct Repositories {
    pool: DbPool,
}

impl Repositories {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    pub fn patients(&self) -> PatientRepository {
        PatientRepository::new(self.pool.clone())
    }
    
    pub fn appointments(&self) -> AppointmentRepository {
        AppointmentRepository::new(self.pool.clone())
    }
    
    pub fn treatments(&self) -> TreatmentRepository {
        TreatmentRepository::new(self.pool.clone())
    }
    
    pub fn invoices(&self) -> InvoiceRepository {
        InvoiceRepository::new(self.pool.clone())
    }
    
    pub fn products(&self) -> ProductRepository {
        ProductRepository::new(self.pool.clone())
    }
    
    pub fn documents(&self) -> DocumentRepository {
        DocumentRepository::new(self.pool.clone())
    }

    pub fn quotes(&self) -> QuoteRepository {
        QuoteRepository::new(self.pool.clone())
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
        }
    }
    
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }
    
    pub fn limit(&self) -> u32 {
        self.per_page
    }
}

/// Paginated result
#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: u64, pagination: &Pagination) -> Self {
        let total_pages = ((total as f64) / (pagination.per_page as f64)).ceil() as u32;
        Self {
            items,
            total,
            page: pagination.page,
            per_page: pagination.per_page,
            total_pages,
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

use uuid::Uuid;
use validator::Validate;

use dental_core::models::{
    CreatePatient, Patient, PatientFilters, PatientListItem, UpdatePatient,
};
use dental_database::repositories::{Pagination, PaginatedResult, PatientRepository};
use dental_database::DbPool;

use crate::error::{PatientServiceError, PatientServiceResult};

/// Patient service wrapper
pub struct PatientService {
    pool: DbPool,
}

impl PatientService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, data: CreatePatient) -> PatientServiceResult<Patient> {
        data.validate()
            .map_err(|e| PatientServiceError::Validation(e.to_string()))?;

        let repo = PatientRepository::new(self.pool.clone());
        Ok(repo.create(data)?)
    }

    pub fn get(&self, patient_id: Uuid) -> PatientServiceResult<Patient> {
        let repo = PatientRepository::new(self.pool.clone());
        Ok(repo.find_by_id(patient_id)?)
    }

    pub fn update(&self, patient_id: Uuid, data: UpdatePatient) -> PatientServiceResult<Patient> {
        let repo = PatientRepository::new(self.pool.clone());
        Ok(repo.update(patient_id, data)?)
    }

    pub fn delete(&self, patient_id: Uuid) -> PatientServiceResult<()> {
        let repo = PatientRepository::new(self.pool.clone());
        Ok(repo.delete(patient_id)?)
    }

    pub fn list(&self, filters: PatientFilters, page: u32, per_page: u32) -> PatientServiceResult<PaginatedResult<PatientListItem>> {
        let repo = PatientRepository::new(self.pool.clone());
        let pagination = Pagination::new(page, per_page);
        Ok(repo.list(filters, pagination)?)
    }

    pub fn search(&self, query: &str, limit: usize) -> PatientServiceResult<Vec<PatientListItem>> {
        let repo = PatientRepository::new(self.pool.clone());
        Ok(repo.search(query, limit)?)
    }

    pub fn count(&self, active_only: bool) -> PatientServiceResult<i64> {
        let repo = PatientRepository::new(self.pool.clone());
        Ok(repo.count(active_only)?)
    }
}

use uuid::Uuid;

use dental_core::models::{
    ClinicalNote, ClinicalNoteFilters, CreateClinicalNote, UpdateClinicalNote,
};
use dental_database::repositories::ClinicalNoteRepository;
use dental_database::DbPool;

use crate::error::ClinicalResult;

/// Clinical service for notes and clinical record utilities
pub struct ClinicalService {
    pool: DbPool,
}

impl ClinicalService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_note(&self, data: CreateClinicalNote) -> ClinicalResult<ClinicalNote> {
        let repo = ClinicalNoteRepository::new(self.pool.clone());
        Ok(repo.create(data)?)
    }

    pub fn get_note(&self, note_id: Uuid) -> ClinicalResult<ClinicalNote> {
        let repo = ClinicalNoteRepository::new(self.pool.clone());
        Ok(repo.find_by_id(note_id)?)
    }

    pub fn update_note(&self, note_id: Uuid, data: UpdateClinicalNote) -> ClinicalResult<ClinicalNote> {
        let repo = ClinicalNoteRepository::new(self.pool.clone());
        Ok(repo.update(note_id, data)?)
    }

    pub fn delete_note(&self, note_id: Uuid) -> ClinicalResult<()> {
        let repo = ClinicalNoteRepository::new(self.pool.clone());
        Ok(repo.delete(note_id)?)
    }

    pub fn list_notes(&self, filters: ClinicalNoteFilters) -> ClinicalResult<Vec<ClinicalNote>> {
        let repo = ClinicalNoteRepository::new(self.pool.clone());
        Ok(repo.list(filters)?)
    }

    pub fn list_notes_by_patient(&self, patient_id: Uuid) -> ClinicalResult<Vec<ClinicalNote>> {
        let repo = ClinicalNoteRepository::new(self.pool.clone());
        Ok(repo.list_by_patient(patient_id)?)
    }

    pub fn list_notes_by_appointment(&self, appointment_id: Uuid) -> ClinicalResult<Vec<ClinicalNote>> {
        let repo = ClinicalNoteRepository::new(self.pool.clone());
        Ok(repo.list_by_appointment(appointment_id)?)
    }
}

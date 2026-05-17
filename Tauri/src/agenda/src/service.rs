use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use dental_core::models::{
    Appointment, AppointmentListItem, CreateAppointment, RescheduleAppointment, UpdateAppointment,
};
use dental_core::AppointmentStatus;
use dental_database::AppointmentRepository;
use dental_database::DbPool;

use crate::error::AgendaResult;

/// Agenda service for appointment scheduling and management
pub struct AgendaService {
    pool: DbPool,
}

impl AgendaService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn schedule(&self, data: CreateAppointment, created_by: Uuid) -> AgendaResult<Appointment> {
        let repo = AppointmentRepository::new(self.pool.clone());
        Ok(repo.create(data, created_by)?)
    }

    pub fn update(&self, appointment_id: Uuid, data: UpdateAppointment) -> AgendaResult<Appointment> {
        let repo = AppointmentRepository::new(self.pool.clone());
        Ok(repo.update(appointment_id, data)?)
    }

    pub fn reschedule(&self, appointment_id: Uuid, data: RescheduleAppointment) -> AgendaResult<Appointment> {
        let repo = AppointmentRepository::new(self.pool.clone());
        let update = UpdateAppointment {
            doctor_id: data.new_doctor_id,
            datetime: Some(data.new_datetime),
            chair_number: data.new_chair_number,
            status: Some(AppointmentStatus::Rescheduled),
            cancel_reason: data.reason,
            ..UpdateAppointment::default()
        };
        Ok(repo.update(appointment_id, update)?)
    }

    pub fn cancel(&self, appointment_id: Uuid, reason: Option<String>) -> AgendaResult<()> {
        let repo = AppointmentRepository::new(self.pool.clone());
        let update = UpdateAppointment {
            status: Some(AppointmentStatus::Cancelled),
            cancel_reason: reason,
            ..UpdateAppointment::default()
        };
        repo.update(appointment_id, update)?;
        Ok(())
    }

    pub fn set_status(&self, appointment_id: Uuid, status: AppointmentStatus) -> AgendaResult<()> {
        let repo = AppointmentRepository::new(self.pool.clone());
        repo.update_status(appointment_id, status)?;
        Ok(())
    }

    pub fn delete(&self, appointment_id: Uuid) -> AgendaResult<()> {
        let repo = AppointmentRepository::new(self.pool.clone());
        repo.delete(appointment_id)?;
        Ok(())
    }

    pub fn list_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        doctor_id: Option<Uuid>,
    ) -> AgendaResult<Vec<AppointmentListItem>> {
        let repo = AppointmentRepository::new(self.pool.clone());
        Ok(repo.list_by_date_range(start, end, doctor_id)?)
    }

    pub fn list_by_patient(&self, patient_id: Uuid) -> AgendaResult<Vec<AppointmentListItem>> {
        let repo = AppointmentRepository::new(self.pool.clone());
        Ok(repo.list_by_patient(patient_id)?)
    }

    pub fn today(&self, doctor_id: Option<Uuid>) -> AgendaResult<Vec<AppointmentListItem>> {
        let repo = AppointmentRepository::new(self.pool.clone());
        Ok(repo.get_today(doctor_id)?)
    }

    pub fn count_by_date(&self, date: NaiveDate) -> AgendaResult<std::collections::HashMap<AppointmentStatus, i32>> {
        let repo = AppointmentRepository::new(self.pool.clone());
        Ok(repo.count_by_date(date)?)
    }
}

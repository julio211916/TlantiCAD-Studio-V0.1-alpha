//! Appointment domain model

use chrono::{DateTime, Utc, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::AppointmentStatus;

/// Appointment entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: Uuid,
    
    /// Reference to patient
    pub patient_id: Uuid,
    
    /// Reference to doctor/provider
    pub doctor_id: Uuid,
    
    /// Appointment date and time (start)
    pub datetime: DateTime<Utc>,
    
    /// Duration in minutes
    pub duration_minutes: i32,
    
    /// Chair/room number
    pub chair_number: Option<i32>,
    
    /// Branch/clinic ID (for multi-location)
    pub clinic_id: Option<Uuid>,
    
    /// Status
    pub status: AppointmentStatus,
    
    /// Reason for visit
    pub reason: Option<String>,
    
    /// Procedures planned (comma-separated IDs or JSON)
    pub procedures: Option<String>,
    
    /// Notes for the appointment
    pub notes: Option<String>,
    
    /// Internal notes (not visible to patient)
    pub internal_notes: Option<String>,
    
    /// Confirmation sent
    pub confirmation_sent: bool,
    
    /// Reminder sent
    pub reminder_sent: bool,
    
    /// Patient arrived time
    pub checked_in_at: Option<DateTime<Utc>>,
    
    /// Treatment started time
    pub started_at: Option<DateTime<Utc>>,
    
    /// Treatment completed time
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Cancelled/rescheduled reason
    pub cancel_reason: Option<String>,
    
    /// Reference to recurring appointment group
    pub recurring_group_id: Option<Uuid>,
    
    /// Color code for calendar
    pub color: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Appointment {
    pub fn new(
        patient_id: Uuid,
        doctor_id: Uuid,
        datetime: DateTime<Utc>,
        duration_minutes: i32,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_id,
            doctor_id,
            datetime,
            duration_minutes,
            chair_number: None,
            clinic_id: None,
            status: AppointmentStatus::Scheduled,
            reason: None,
            procedures: None,
            notes: None,
            internal_notes: None,
            confirmation_sent: false,
            reminder_sent: false,
            checked_in_at: None,
            started_at: None,
            completed_at: None,
            cancel_reason: None,
            recurring_group_id: None,
            color: None,
            created_at: now,
            updated_at: now,
            created_by,
        }
    }
    
    pub fn end_time(&self) -> DateTime<Utc> {
        self.datetime + chrono::Duration::minutes(self.duration_minutes as i64)
    }
    
    pub fn is_past(&self) -> bool {
        self.datetime < Utc::now()
    }
    
    pub fn is_today(&self) -> bool {
        self.datetime.date_naive() == Utc::now().date_naive()
    }
}

/// Appointment with patient and doctor details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentDetails {
    pub appointment: Appointment,
    pub patient_name: String,
    pub patient_phone: String,
    pub doctor_name: String,
    pub procedure_names: Vec<String>,
}

/// Appointment list item for calendar views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentListItem {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub patient_name: String,
    pub doctor_id: Uuid,
    pub doctor_name: String,
    pub datetime: DateTime<Utc>,
    pub duration_minutes: i32,
    pub chair_number: Option<i32>,
    pub status: AppointmentStatus,
    pub reason: Option<String>,
    pub color: Option<String>,
}

/// Create appointment DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAppointment {
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub datetime: DateTime<Utc>,
    pub duration_minutes: i32,
    pub chair_number: Option<i32>,
    pub clinic_id: Option<Uuid>,
    pub reason: Option<String>,
    pub procedures: Option<Vec<Uuid>>,
    pub notes: Option<String>,
    pub color: Option<String>,
}

/// Update appointment DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateAppointment {
    pub doctor_id: Option<Uuid>,
    pub datetime: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub chair_number: Option<i32>,
    pub clinic_id: Option<Uuid>,
    pub status: Option<AppointmentStatus>,
    pub reason: Option<String>,
    pub procedures: Option<Vec<Uuid>>,
    pub notes: Option<String>,
    pub internal_notes: Option<String>,
    pub cancel_reason: Option<String>,
    pub color: Option<String>,
}

/// Reschedule appointment DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescheduleAppointment {
    pub new_datetime: DateTime<Utc>,
    pub new_doctor_id: Option<Uuid>,
    pub new_chair_number: Option<i32>,
    pub reason: Option<String>,
}

/// Doctor availability slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilitySlot {
    pub doctor_id: Uuid,
    pub date: chrono::NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub available: bool,
}

/// Doctor schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorSchedule {
    pub doctor_id: Uuid,
    pub day_of_week: u8, // 0 = Sunday, 6 = Saturday
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub slot_duration_minutes: i32,
    pub active: bool,
}

/// Appointment filters for queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppointmentFilters {
    pub patient_id: Option<Uuid>,
    pub doctor_id: Option<Uuid>,
    pub clinic_id: Option<Uuid>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub status: Option<Vec<AppointmentStatus>>,
    pub chair_number: Option<i32>,
}

/// Reminder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    pub appointment_id: Uuid,
    pub channel: crate::enums::ReminderChannel,
    pub send_at: DateTime<Utc>,
    pub message: String,
    pub sent: bool,
    pub sent_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

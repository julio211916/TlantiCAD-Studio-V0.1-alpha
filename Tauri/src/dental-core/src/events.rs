//! Domain events for TlantiStudio Dental

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::{InvoiceStatus, TreatmentStatus};

/// Domain event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DentalEvent {
    // Patient events
    PatientCreated(PatientCreatedEvent),
    PatientUpdated(PatientUpdatedEvent),
    PatientDeactivated(EntityEvent),
    
    // Appointment events
    AppointmentScheduled(AppointmentScheduledEvent),
    AppointmentConfirmed(EntityEvent),
    AppointmentCheckedIn(EntityEvent),
    AppointmentStarted(EntityEvent),
    AppointmentCompleted(EntityEvent),
    AppointmentCancelled(AppointmentCancelledEvent),
    AppointmentRescheduled(AppointmentRescheduledEvent),
    
    // Treatment events
    TreatmentPlanned(TreatmentEvent),
    TreatmentStarted(TreatmentEvent),
    TreatmentCompleted(TreatmentEvent),
    TreatmentPlanApproved(EntityEvent),
    
    // Invoice events
    InvoiceCreated(InvoiceEvent),
    InvoiceUpdated(InvoiceEvent),
    InvoicePaid(InvoiceEvent),
    PaymentReceived(PaymentReceivedEvent),
    
    // Inventory events
    StockLow(StockLowEvent),
    StockMovement(StockMovementEvent),
    
    // Document events
    DocumentCreated(DocumentEvent),
    DocumentSigned(DocumentEvent),
    
    // System events
    UserLoggedIn(UserEvent),
    UserLoggedOut(UserEvent),
    SettingsChanged(SettingsChangedEvent),
}

/// Base entity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEvent {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl EntityEvent {
    pub fn new(entity_id: Uuid, user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id,
            user_id,
            timestamp: Utc::now(),
        }
    }
}

/// Patient created event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientCreatedEvent {
    pub patient_id: Uuid,
    pub patient_number: String,
    pub patient_name: String,
    pub created_by: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Patient updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientUpdatedEvent {
    pub patient_id: Uuid,
    pub fields_changed: Vec<String>,
    pub updated_by: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Appointment scheduled event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentScheduledEvent {
    pub appointment_id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub datetime: DateTime<Utc>,
    pub created_by: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Appointment cancelled event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentCancelledEvent {
    pub appointment_id: Uuid,
    pub patient_id: Uuid,
    pub reason: Option<String>,
    pub cancelled_by: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Appointment rescheduled event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentRescheduledEvent {
    pub appointment_id: Uuid,
    pub patient_id: Uuid,
    pub old_datetime: DateTime<Utc>,
    pub new_datetime: DateTime<Utc>,
    pub reason: Option<String>,
    pub rescheduled_by: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Treatment event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentEvent {
    pub treatment_id: Uuid,
    pub patient_id: Uuid,
    pub procedure_id: Uuid,
    pub tooth_number: Option<i32>,
    pub status: TreatmentStatus,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Invoice event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceEvent {
    pub invoice_id: Uuid,
    pub invoice_number: String,
    pub patient_id: Uuid,
    pub total: rust_decimal::Decimal,
    pub status: InvoiceStatus,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Payment received event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReceivedEvent {
    pub payment_id: Uuid,
    pub invoice_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub payment_method: String,
    pub received_by: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Stock low event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockLowEvent {
    pub product_id: Uuid,
    pub product_name: String,
    pub current_stock: i32,
    pub min_stock: i32,
    pub timestamp: DateTime<Utc>,
}

/// Stock movement event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovementEvent {
    pub movement_id: Uuid,
    pub product_id: Uuid,
    pub movement_type: String,
    pub quantity: i32,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Document event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEvent {
    pub document_id: Uuid,
    pub patient_id: Uuid,
    pub document_type: String,
    pub title: String,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// User event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub user_id: Uuid,
    pub username: String,
    pub action: String,
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Settings changed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsChangedEvent {
    pub setting_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_by: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Event handler trait
pub trait EventHandler {
    fn handle(&self, event: &DentalEvent);
}

/// Event store for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub aggregate_id: Option<Uuid>,
    pub aggregate_type: Option<String>,
    pub user_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}

impl StoredEvent {
    pub fn from_event(event: &DentalEvent, aggregate_id: Option<Uuid>, aggregate_type: Option<String>, user_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type_name(event),
            payload: serde_json::to_string(event).unwrap_or_default(),
            aggregate_id,
            aggregate_type,
            user_id,
            timestamp: Utc::now(),
        }
    }
}

fn event_type_name(event: &DentalEvent) -> String {
    match event {
        DentalEvent::PatientCreated(_) => "patient.created",
        DentalEvent::PatientUpdated(_) => "patient.updated",
        DentalEvent::PatientDeactivated(_) => "patient.deactivated",
        DentalEvent::AppointmentScheduled(_) => "appointment.scheduled",
        DentalEvent::AppointmentConfirmed(_) => "appointment.confirmed",
        DentalEvent::AppointmentCheckedIn(_) => "appointment.checked_in",
        DentalEvent::AppointmentStarted(_) => "appointment.started",
        DentalEvent::AppointmentCompleted(_) => "appointment.completed",
        DentalEvent::AppointmentCancelled(_) => "appointment.cancelled",
        DentalEvent::AppointmentRescheduled(_) => "appointment.rescheduled",
        DentalEvent::TreatmentPlanned(_) => "treatment.planned",
        DentalEvent::TreatmentStarted(_) => "treatment.started",
        DentalEvent::TreatmentCompleted(_) => "treatment.completed",
        DentalEvent::TreatmentPlanApproved(_) => "treatment_plan.approved",
        DentalEvent::InvoiceCreated(_) => "invoice.created",
        DentalEvent::InvoiceUpdated(_) => "invoice.updated",
        DentalEvent::InvoicePaid(_) => "invoice.paid",
        DentalEvent::PaymentReceived(_) => "payment.received",
        DentalEvent::StockLow(_) => "stock.low",
        DentalEvent::StockMovement(_) => "stock.movement",
        DentalEvent::DocumentCreated(_) => "document.created",
        DentalEvent::DocumentSigned(_) => "document.signed",
        DentalEvent::UserLoggedIn(_) => "user.logged_in",
        DentalEvent::UserLoggedOut(_) => "user.logged_out",
        DentalEvent::SettingsChanged(_) => "settings.changed",
    }.to_string()
}

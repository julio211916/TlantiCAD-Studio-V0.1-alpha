//! Clinic/Branch domain model

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Clinic/Branch entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clinic {
    pub id: Uuid,
    
    /// Clinic name
    pub name: String,
    
    /// Legal business name
    pub legal_name: Option<String>,
    
    /// Address
    pub address: String,
    
    /// City
    pub city: String,
    
    /// State/Province
    pub state: String,
    
    /// Postal code
    pub postal_code: String,
    
    /// Country
    pub country: String,
    
    /// Phone number
    pub phone: String,
    
    /// Secondary phone
    pub phone_secondary: Option<String>,
    
    /// Email
    pub email: Option<String>,
    
    /// Website
    pub website: Option<String>,
    
    /// Tax ID (RFC in Mexico)
    pub tax_id: Option<String>,
    
    /// Logo URL
    pub logo_url: Option<String>,
    
    /// Number of chairs/operatories
    pub chair_count: i32,
    
    /// Time zone
    pub timezone: String,
    
    /// Currency code (MXN, USD, etc.)
    pub currency: String,
    
    /// Default tax rate
    pub default_tax_rate: rust_decimal::Decimal,
    
    /// Settings (JSON)
    pub settings: Option<ClinicSettings>,
    
    /// Is headquarters/main branch
    pub is_main: bool,
    
    /// Is active
    pub active: bool,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Clinic {
    pub fn new(name: String, address: String, city: String, phone: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            legal_name: None,
            address,
            city,
            state: String::new(),
            postal_code: String::new(),
            country: "Mexico".to_string(),
            phone,
            phone_secondary: None,
            email: None,
            website: None,
            tax_id: None,
            logo_url: None,
            chair_count: 1,
            timezone: "America/Mexico_City".to_string(),
            currency: "MXN".to_string(),
            default_tax_rate: rust_decimal::Decimal::from(16),
            settings: None,
            is_main: false,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Clinic settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClinicSettings {
    /// Appointment slot duration in minutes
    pub slot_duration: i32,
    
    /// Allow online booking
    pub online_booking: bool,
    
    /// Send appointment reminders
    pub send_reminders: bool,
    
    /// Reminder hours before appointment
    pub reminder_hours: Vec<i32>,
    
    /// Invoice series prefix
    pub invoice_prefix: Option<String>,
    
    /// Invoice number counter
    pub invoice_counter: i64,
    
    /// Patient number prefix
    pub patient_prefix: Option<String>,
    
    /// Patient number counter
    pub patient_counter: i64,
    
    /// Receipt footer text
    pub receipt_footer: Option<String>,
    
    /// Working days (0 = Sunday, 6 = Saturday)
    pub working_days: Vec<u8>,
    
    /// Opening time
    pub open_time: Option<String>,
    
    /// Closing time
    pub close_time: Option<String>,
    
    /// Lunch break start
    pub lunch_start: Option<String>,
    
    /// Lunch break end
    pub lunch_end: Option<String>,
}

/// Clinic schedule (working hours)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicSchedule {
    pub clinic_id: Uuid,
    pub day_of_week: u8,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_open: bool,
}

/// Holiday/closure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicClosure {
    pub id: Uuid,
    pub clinic_id: Uuid,
    pub date: chrono::NaiveDate,
    pub reason: String,
    pub is_recurring: bool,
}

/// Clinic list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicListItem {
    pub id: Uuid,
    pub name: String,
    pub city: String,
    pub phone: String,
    pub chair_count: i32,
    pub is_main: bool,
    pub active: bool,
}

/// Create clinic DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClinic {
    pub name: String,
    pub legal_name: Option<String>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: Option<String>,
    pub phone: String,
    pub phone_secondary: Option<String>,
    pub email: Option<String>,
    pub tax_id: Option<String>,
    pub chair_count: i32,
    pub timezone: Option<String>,
    pub currency: Option<String>,
    pub default_tax_rate: Option<rust_decimal::Decimal>,
}

/// Update clinic DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateClinic {
    pub name: Option<String>,
    pub legal_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub phone_secondary: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub tax_id: Option<String>,
    pub logo_url: Option<String>,
    pub chair_count: Option<i32>,
    pub timezone: Option<String>,
    pub currency: Option<String>,
    pub default_tax_rate: Option<rust_decimal::Decimal>,
    pub settings: Option<ClinicSettings>,
    pub active: Option<bool>,
}

/// Clinic stats/dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicStats {
    pub clinic_id: Uuid,
    pub clinic_name: String,
    pub total_patients: i64,
    pub active_patients: i64,
    pub appointments_today: i64,
    pub appointments_week: i64,
    pub revenue_today: rust_decimal::Decimal,
    pub revenue_month: rust_decimal::Decimal,
    pub pending_payments: rust_decimal::Decimal,
    pub low_stock_items: i64,
}

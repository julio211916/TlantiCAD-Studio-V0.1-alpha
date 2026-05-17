//! Dashboard Tauri commands

use rust_decimal::Decimal;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use tauri::State;

use dental_database::repositories::{AppointmentRepository, InvoiceRepository, PatientRepository, ProductRepository};

use crate::{CommandResult, DentalState};

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_patients: i64,
    pub active_patients: i64,
    pub appointments_today: i32,
    pub appointments_pending: i32,
    pub revenue_today: Decimal,
    pub revenue_month: Decimal,
    pub pending_payments: Decimal,
    pub low_stock_count: i32,
}

/// Get dashboard statistics
#[tauri::command]
pub fn dashboard_get_stats(
    state: State<'_, DentalState>,
) -> CommandResult<DashboardStats> {
    let patient_repo = PatientRepository::new(state.db.pool().clone());
    let appointment_repo = AppointmentRepository::new(state.db.pool().clone());
    let product_repo = ProductRepository::new(state.db.pool().clone());
    let invoice_repo = InvoiceRepository::new(state.db.pool().clone());

    let now = chrono::Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

    let month_start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let next_month = if now.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
    };
    let month_end = next_month
        .pred_opt()
        .unwrap()
        .and_hms_opt(23, 59, 59)
        .unwrap();

    let revenue_today = invoice_repo
        .sum_payments_in_range(
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(today_start, chrono::Utc),
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(today_end, chrono::Utc),
        )
        .unwrap_or(Decimal::ZERO);

    let revenue_month = invoice_repo
        .sum_payments_in_range(
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(month_start, chrono::Utc),
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(month_end, chrono::Utc),
        )
        .unwrap_or(Decimal::ZERO);

    let pending_payments = invoice_repo.sum_pending_balance().unwrap_or(Decimal::ZERO);
    
    let total_patients = patient_repo.count(false).unwrap_or(0);
    let active_patients = patient_repo.count(true).unwrap_or(0);
    
    let today_appointments = appointment_repo.get_today(None).unwrap_or_default();
    let appointments_today = today_appointments.len() as i32;
    let appointments_pending = today_appointments.iter()
        .filter(|a| a.status == dental_core::AppointmentStatus::Scheduled || a.status == dental_core::AppointmentStatus::Confirmed)
        .count() as i32;
    
    let low_stock_alerts = product_repo.get_low_stock_alerts().unwrap_or_default();
    let low_stock_count = low_stock_alerts.len() as i32;
    
    Ok(DashboardStats {
        total_patients,
        active_patients,
        appointments_today,
        appointments_pending,
        revenue_today,
        revenue_month,
        pending_payments,
        low_stock_count,
    })
}

/// Get today's appointments for dashboard
#[tauri::command]
pub fn dashboard_get_today_appointments(
    state: State<'_, DentalState>,
) -> CommandResult<Vec<dental_core::models::AppointmentListItem>> {
    let repo = AppointmentRepository::new(state.db.pool().clone());
    repo.get_today(None).map_err(|e| e.into())
}

/// Get low stock alerts for dashboard
#[tauri::command]
pub fn dashboard_get_low_stock(
    state: State<'_, DentalState>,
) -> CommandResult<Vec<dental_core::models::LowStockAlert>> {
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.get_low_stock_alerts().map_err(|e| e.into())
}

/// Quick search across patients
#[tauri::command]
pub fn dashboard_quick_search(
    state: State<'_, DentalState>,
    query: String,
) -> CommandResult<Vec<dental_core::models::PatientListItem>> {
    let repo = PatientRepository::new(state.db.pool().clone());
    repo.search(&query, 5).map_err(|e| e.into())
}

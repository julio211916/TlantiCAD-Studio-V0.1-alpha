//! Accounting Service IPC Commands
//!
//! Provides Tauri commands for accounting operations:
//! - Daily cash summaries
//! - Payment history
//! - Patient balances

use crate::{DentalCommandError, DentalCommandResult, DentalState};
use accounting::AccountingService;
use chrono::NaiveDate;
use dental_core::models::{DailyCashSummary, PatientBalance, PaymentFilters};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

/// Get daily cash summary for a specific date
#[tauri::command]
pub async fn accounting_get_daily_summary(
    state: State<'_, DentalState>,
    date: NaiveDate,
) -> DentalCommandResult<DailyCashSummary> {
    let service = AccountingService::new(state.db.pool().clone());
    service
        .daily_cash_summary(date)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// Get patient balance
#[tauri::command]
pub async fn accounting_get_patient_balance(
    state: State<'_, DentalState>,
    patient_id: Uuid,
) -> DentalCommandResult<PatientBalance> {
    let service = AccountingService::new(state.db.pool().clone());
    service
        .patient_balance(patient_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))
}

/// List payments with filters
#[tauri::command]
pub async fn accounting_list_payments(
    state: State<'_, DentalState>,
    invoice_id: Option<Uuid>,
    payment_method: Option<String>,
    received_by: Option<Uuid>,
) -> DentalCommandResult<Vec<PaymentResponse>> {
    let service = AccountingService::new(state.db.pool().clone());
    
    let method = payment_method.and_then(|m| m.parse().ok());
    
    let filters = PaymentFilters {
        invoice_id,
        payment_method: method,
        date_from: None,
        date_to: None,
        received_by,
    };
    
    let payments = service
        .list_payments(filters)
        .map_err(|e| DentalCommandError::Database(e.to_string()))?;
    
    Ok(payments
        .into_iter()
        .map(|p| PaymentResponse {
            id: p.id,
            invoice_id: p.invoice_id,
            amount: p.amount.to_string().parse().unwrap_or(0.0),
            payment_method: p.payment_method.to_string(),
            reference: p.reference,
            date: p.date.to_rfc3339(),
            notes: p.notes,
            is_refund: p.is_refund,
        })
        .collect())
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub amount: f64,
    pub payment_method: String,
    pub reference: Option<String>,
    pub date: String,
    pub notes: Option<String>,
    pub is_refund: bool,
}

/// Get cash summary for date range
#[tauri::command]
pub async fn accounting_get_range_summary(
    state: State<'_, DentalState>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> DentalCommandResult<Vec<DailyCashSummary>> {
    let service = AccountingService::new(state.db.pool().clone());
    let mut summaries = Vec::new();
    
    let mut current = start_date;
    while current <= end_date {
        let summary = service
            .daily_cash_summary(current)
            .map_err(|e| DentalCommandError::Database(e.to_string()))?;
        summaries.push(summary);
        current = current.succ_opt().unwrap_or(current);
    }
    
    Ok(summaries)
}

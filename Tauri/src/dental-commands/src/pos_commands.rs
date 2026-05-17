//! POS Service IPC Commands
//!
//! Provides Tauri commands for Point of Sale operations:
//! - Create sales with stock management
//! - Cart management

use crate::{DentalCommandError, DentalCommandResult, DentalState};
use pos::{PosService, SaleItemInput, SalePaymentInput, SaleRequest};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

/// Cart item input for IPC
#[derive(Debug, Deserialize)]
pub struct CartItemInput {
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Option<f64>,
    pub discount: Option<f64>,
    pub description: Option<String>,
}

/// Payment input for IPC
#[derive(Debug, Deserialize)]
pub struct PaymentInput {
    pub amount: f64,
    pub payment_method: String,
    pub reference: Option<String>,
    pub authorization_code: Option<String>,
    pub notes: Option<String>,
}

/// Sale result for IPC
#[derive(Debug, Serialize)]
pub struct SaleResultResponse {
    pub invoice_id: Uuid,
    pub invoice_number: String,
    pub item_count: usize,
    pub payment_id: Option<Uuid>,
}

/// Create a POS sale with automatic stock decrement and invoice creation
#[tauri::command]
pub async fn pos_create_sale(
    state: State<'_, DentalState>,
    patient_id: Uuid,
    clinic_id: Uuid,
    items: Vec<CartItemInput>,
    payment: Option<PaymentInput>,
    notes: Option<String>,
) -> DentalCommandResult<SaleResultResponse> {
    let user_id = state
        .get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".to_string()))?;
    
    // Convert cart items
    let sale_items: Vec<SaleItemInput> = items
        .into_iter()
        .map(|item| SaleItemInput {
            product_id: item.product_id,
            quantity: item.quantity,
            unit_price: item.unit_price.map(|p| Decimal::try_from(p).unwrap_or(Decimal::ZERO)),
            discount: item.discount.map(|d| Decimal::try_from(d).unwrap_or(Decimal::ZERO)),
            description: item.description,
        })
        .collect();
    
    // Convert payment
    let sale_payment = payment.map(|p| SalePaymentInput {
        amount: Decimal::try_from(p.amount).unwrap_or(Decimal::ZERO),
        payment_method: p.payment_method.parse().unwrap_or(dental_core::PaymentMethod::Cash),
        reference: p.reference,
        authorization_code: p.authorization_code,
        notes: p.notes,
    });
    
    let request = SaleRequest {
        patient_id,
        clinic_id: Some(clinic_id),
        items: sale_items,
        payment: sale_payment,
        notes,
    };
    
    let service = PosService::new(state.db.pool().clone());
    let result = service
        .create_sale(request, user_id)
        .map_err(|e| DentalCommandError::Database(e.to_string()))?;
    
    Ok(SaleResultResponse {
        invoice_id: result.invoice_id,
        invoice_number: result.invoice_number,
        item_count: result.item_count,
        payment_id: result.payment_id,
    })
}

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use dental_core::PaymentMethod;

/// Sale item input for POS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItemInput {
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub description: Option<String>,
}

/// Payment input for POS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalePaymentInput {
    pub amount: Decimal,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
    pub authorization_code: Option<String>,
    pub notes: Option<String>,
}

/// POS sale request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleRequest {
    pub patient_id: Uuid,
    pub clinic_id: Option<Uuid>,
    pub items: Vec<SaleItemInput>,
    pub payment: Option<SalePaymentInput>,
    pub notes: Option<String>,
}

/// POS sale result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleResult {
    pub invoice_id: Uuid,
    pub invoice_number: String,
    pub item_count: usize,
    pub payment_id: Option<Uuid>,
}

//! Quote (presupuesto) domain models

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub created_by: Uuid,
    pub notes: Option<String>,
    pub total: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteItem {
    pub id: Uuid,
    pub quote_id: Uuid,
    pub description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuoteItem {
    pub description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuote {
    pub patient_id: Uuid,
    pub notes: Option<String>,
    pub items: Vec<CreateQuoteItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteWithItems {
    pub quote: Quote,
    pub items: Vec<QuoteItem>,
}

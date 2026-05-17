//! Invoice and payment domain models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::{InvoiceStatus, PaymentMethod};

/// Invoice entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    
    /// Invoice number (e.g., INV-2024-00001)
    pub invoice_number: String,
    
    /// Reference to patient
    pub patient_id: Uuid,
    
    /// Clinic/Branch ID
    pub clinic_id: Option<Uuid>,
    
    /// Invoice date
    pub date: DateTime<Utc>,
    
    /// Due date
    pub due_date: Option<NaiveDate>,
    
    /// Status
    pub status: InvoiceStatus,
    
    /// Subtotal (before tax and discount)
    pub subtotal: Decimal,
    
    /// Tax amount
    pub tax: Decimal,
    
    /// Tax rate percentage
    pub tax_rate: Decimal,
    
    /// Discount amount
    pub discount: Decimal,
    
    /// Total amount
    pub total: Decimal,
    
    /// Amount paid
    pub amount_paid: Decimal,
    
    /// Balance due
    pub balance: Decimal,
    
    /// Notes visible to patient
    pub notes: Option<String>,
    
    /// Internal notes
    pub internal_notes: Option<String>,
    
    /// Electronic invoice (CFDI) UUID
    pub cfdi_uuid: Option<String>,
    
    /// Electronic invoice status
    pub cfdi_status: Option<String>,
    
    /// Created by user
    pub created_by: Uuid,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invoice {
    pub fn new(patient_id: Uuid, invoice_number: String, created_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            invoice_number,
            patient_id,
            clinic_id: None,
            date: now,
            due_date: None,
            status: InvoiceStatus::Draft,
            subtotal: Decimal::ZERO,
            tax: Decimal::ZERO,
            tax_rate: Decimal::from(16), // Default 16% IVA
            discount: Decimal::ZERO,
            total: Decimal::ZERO,
            amount_paid: Decimal::ZERO,
            balance: Decimal::ZERO,
            notes: None,
            internal_notes: None,
            cfdi_uuid: None,
            cfdi_status: None,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn calculate_totals(&mut self, items: &[InvoiceItem]) {
        self.subtotal = items.iter().map(|i| i.total).sum();
        self.tax = self.subtotal * (self.tax_rate / Decimal::from(100));
        self.total = self.subtotal + self.tax - self.discount;
        self.balance = self.total - self.amount_paid;
        
        // Update status based on payment
        if self.amount_paid >= self.total {
            self.status = InvoiceStatus::Paid;
        } else if self.amount_paid > Decimal::ZERO {
            self.status = InvoiceStatus::PartiallyPaid;
        }
    }
    
    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            self.status != InvoiceStatus::Paid && Utc::now().date_naive() > due
        } else {
            false
        }
    }
}

/// Invoice item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub id: Uuid,
    
    /// Invoice reference
    pub invoice_id: Uuid,
    
    /// Treatment reference (optional)
    pub treatment_id: Option<Uuid>,
    
    /// Product reference (optional)
    pub product_id: Option<Uuid>,
    
    /// Description
    pub description: String,
    
    /// Quantity
    pub quantity: i32,
    
    /// Unit price
    pub unit_price: Decimal,
    
    /// Discount amount
    pub discount: Decimal,
    
    /// Total (quantity * unit_price - discount)
    pub total: Decimal,
}

impl InvoiceItem {
    pub fn new(invoice_id: Uuid, description: String, quantity: i32, unit_price: Decimal) -> Self {
        let total = Decimal::from(quantity) * unit_price;
        Self {
            id: Uuid::new_v4(),
            invoice_id,
            treatment_id: None,
            product_id: None,
            description,
            quantity,
            unit_price,
            discount: Decimal::ZERO,
            total,
        }
    }
    
    pub fn calculate_total(&mut self) {
        self.total = (Decimal::from(self.quantity) * self.unit_price) - self.discount;
    }
}

/// Payment entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    
    /// Invoice reference
    pub invoice_id: Uuid,
    
    /// Payment amount
    pub amount: Decimal,
    
    /// Payment method
    pub payment_method: PaymentMethod,
    
    /// Reference number (card/transfer)
    pub reference: Option<String>,
    
    /// Payment date
    pub date: DateTime<Utc>,
    
    /// Authorization code (for cards)
    pub authorization_code: Option<String>,
    
    /// Terminal ID (for card payments)
    pub terminal_id: Option<String>,
    
    /// Notes
    pub notes: Option<String>,
    
    /// Is refund
    pub is_refund: bool,
    
    /// Refund reason
    pub refund_reason: Option<String>,
    
    /// Received by user
    pub received_by: Uuid,
    
    pub created_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(invoice_id: Uuid, amount: Decimal, payment_method: PaymentMethod, received_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            invoice_id,
            amount,
            payment_method,
            reference: None,
            date: Utc::now(),
            authorization_code: None,
            terminal_id: None,
            notes: None,
            is_refund: false,
            refund_reason: None,
            received_by,
            created_at: Utc::now(),
        }
    }
}

/// Invoice with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceDetails {
    pub invoice: Invoice,
    pub items: Vec<InvoiceItem>,
    pub payments: Vec<Payment>,
    pub patient_name: String,
    pub clinic_name: Option<String>,
}

/// Invoice list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListItem {
    pub id: Uuid,
    pub invoice_number: String,
    pub patient_id: Uuid,
    pub patient_name: String,
    pub date: DateTime<Utc>,
    pub total: Decimal,
    pub balance: Decimal,
    pub status: InvoiceStatus,
    pub is_overdue: bool,
}

/// Create invoice DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoice {
    pub patient_id: Uuid,
    pub clinic_id: Option<Uuid>,
    pub due_date: Option<NaiveDate>,
    pub tax_rate: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub notes: Option<String>,
    pub items: Vec<CreateInvoiceItem>,
}

/// Create invoice item DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceItem {
    pub treatment_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount: Option<Decimal>,
}

/// Create payment DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePayment {
    pub invoice_id: Uuid,
    pub amount: Decimal,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
    pub authorization_code: Option<String>,
    pub notes: Option<String>,
}

/// Invoice filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvoiceFilters {
    pub patient_id: Option<Uuid>,
    pub clinic_id: Option<Uuid>,
    pub status: Option<Vec<InvoiceStatus>>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub overdue_only: Option<bool>,
    pub has_balance: Option<bool>,
}

/// Payment filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaymentFilters {
    pub invoice_id: Option<Uuid>,
    pub payment_method: Option<PaymentMethod>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub received_by: Option<Uuid>,
}

/// Daily cash summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCashSummary {
    pub date: NaiveDate,
    pub total_invoiced: Decimal,
    pub total_received: Decimal,
    pub total_cash: Decimal,
    pub total_card: Decimal,
    pub total_transfer: Decimal,
    pub total_other: Decimal,
    pub invoice_count: i32,
    pub payment_count: i32,
}

/// Patient balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientBalance {
    pub patient_id: Uuid,
    pub patient_name: String,
    pub total_invoiced: Decimal,
    pub total_paid: Decimal,
    pub balance: Decimal,
    pub overdue_amount: Decimal,
    pub oldest_invoice_date: Option<DateTime<Utc>>,
}

/// Receipt for printing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub invoice_number: String,
    pub date: DateTime<Utc>,
    pub patient_name: String,
    pub items: Vec<ReceiptItem>,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub payment_method: PaymentMethod,
    pub payment_amount: Decimal,
    pub change: Decimal,
    pub clinic_name: String,
    pub clinic_address: String,
    pub clinic_phone: String,
}

/// Receipt item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptItem {
    pub description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub total: Decimal,
}

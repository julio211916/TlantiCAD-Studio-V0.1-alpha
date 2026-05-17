//! Product and inventory domain models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::{ProductCategory, ProductUnit, StockMovementType};

/// Product entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    
    /// SKU (Stock Keeping Unit)
    pub sku: String,
    
    /// Barcode (optional)
    pub barcode: Option<String>,
    
    /// Product name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Category
    pub category: ProductCategory,
    
    /// Unit of measure
    pub unit: ProductUnit,
    
    /// Current stock quantity
    pub current_stock: i32,
    
    /// Minimum stock level (for alerts)
    pub min_stock: i32,
    
    /// Maximum stock level
    pub max_stock: Option<i32>,
    
    /// Reorder point
    pub reorder_point: Option<i32>,
    
    /// Unit cost (purchase price)
    pub unit_cost: Decimal,
    
    /// Unit price (sale price)
    pub unit_price: Decimal,
    
    /// Default supplier
    pub supplier_id: Option<Uuid>,
    
    /// Storage location
    pub location: Option<String>,
    
    /// Brand
    pub brand: Option<String>,
    
    /// Manufacturer
    pub manufacturer: Option<String>,
    
    /// Is taxable
    pub taxable: bool,
    
    /// Tax rate percentage
    pub tax_rate: Option<Decimal>,
    
    /// Is active
    pub active: bool,
    
    /// Image URL
    pub image_url: Option<String>,
    
    /// Notes
    pub notes: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Product {
    pub fn new(sku: String, name: String, category: ProductCategory, unit: ProductUnit) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            sku,
            barcode: None,
            name,
            description: None,
            category,
            unit,
            current_stock: 0,
            min_stock: 0,
            max_stock: None,
            reorder_point: None,
            unit_cost: Decimal::ZERO,
            unit_price: Decimal::ZERO,
            supplier_id: None,
            location: None,
            brand: None,
            manufacturer: None,
            taxable: true,
            tax_rate: None,
            active: true,
            image_url: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn is_low_stock(&self) -> bool {
        self.current_stock <= self.min_stock
    }
    
    pub fn needs_reorder(&self) -> bool {
        match self.reorder_point {
            Some(point) => self.current_stock <= point,
            None => self.is_low_stock(),
        }
    }
    
    pub fn profit_margin(&self) -> Decimal {
        if self.unit_cost > Decimal::ZERO {
            ((self.unit_price - self.unit_cost) / self.unit_cost) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }
}

/// Product with supplier info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductWithSupplier {
    pub product: Product,
    pub supplier_name: Option<String>,
}

/// Product list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductListItem {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub category: ProductCategory,
    pub current_stock: i32,
    pub min_stock: i32,
    pub unit_cost: Decimal,
    pub unit_price: Decimal,
    pub is_low_stock: bool,
    pub active: bool,
}

/// Supplier entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: Uuid,
    
    /// Company name
    pub name: String,
    
    /// Contact person name
    pub contact_name: Option<String>,
    
    /// Phone
    pub phone: String,
    
    /// Secondary phone
    pub phone_secondary: Option<String>,
    
    /// Email
    pub email: Option<String>,
    
    /// Address
    pub address: Option<String>,
    
    /// City
    pub city: Option<String>,
    
    /// Tax ID (RFC in Mexico)
    pub tax_id: Option<String>,
    
    /// Website
    pub website: Option<String>,
    
    /// Payment terms (days)
    pub payment_terms: Option<i32>,
    
    /// Notes
    pub notes: Option<String>,
    
    /// Is active
    pub active: bool,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Supplier {
    pub fn new(name: String, phone: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            contact_name: None,
            phone,
            phone_secondary: None,
            email: None,
            address: None,
            city: None,
            tax_id: None,
            website: None,
            payment_terms: None,
            notes: None,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Stock movement entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
    pub id: Uuid,
    
    /// Product reference
    pub product_id: Uuid,
    
    /// Movement type
    pub movement_type: StockMovementType,
    
    /// Quantity (positive for IN, negative for OUT)
    pub quantity: i32,
    
    /// Unit cost at time of movement
    pub unit_cost: Decimal,
    
    /// Total value
    pub total_value: Decimal,
    
    /// Reference document (PO number, invoice, etc.)
    pub reference: Option<String>,
    
    /// Related appointment or invoice
    pub related_id: Option<Uuid>,
    
    /// Batch/Lot number
    pub batch_number: Option<String>,
    
    /// Expiration date
    pub expiration_date: Option<NaiveDate>,
    
    /// Notes
    pub notes: Option<String>,
    
    /// User who made the movement
    pub user_id: Uuid,
    
    pub created_at: DateTime<Utc>,
}

impl StockMovement {
    pub fn new(
        product_id: Uuid,
        movement_type: StockMovementType,
        quantity: i32,
        unit_cost: Decimal,
        user_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            product_id,
            movement_type,
            quantity,
            unit_cost,
            total_value: unit_cost * Decimal::from(quantity.abs()),
            reference: None,
            related_id: None,
            batch_number: None,
            expiration_date: None,
            notes: None,
            user_id,
            created_at: Utc::now(),
        }
    }
}

/// Purchase order entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: Uuid,
    
    /// PO number
    pub order_number: String,
    
    /// Supplier reference
    pub supplier_id: Uuid,
    
    /// Status
    pub status: PurchaseOrderStatus,
    
    /// Order date
    pub order_date: DateTime<Utc>,
    
    /// Expected delivery date
    pub expected_date: Option<NaiveDate>,
    
    /// Received date
    pub received_date: Option<DateTime<Utc>>,
    
    /// Subtotal
    pub subtotal: Decimal,
    
    /// Tax
    pub tax: Decimal,
    
    /// Total
    pub total: Decimal,
    
    /// Notes
    pub notes: Option<String>,
    
    /// Created by user
    pub created_by: Uuid,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Purchase order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseOrderStatus {
    Draft,
    Submitted,
    Approved,
    Ordered,
    PartiallyReceived,
    Received,
    Cancelled,
}

/// Purchase order item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItem {
    pub id: Uuid,
    pub purchase_order_id: Uuid,
    pub product_id: Uuid,
    pub quantity_ordered: i32,
    pub quantity_received: i32,
    pub unit_cost: Decimal,
    pub total: Decimal,
}

/// Create product DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProduct {
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub category: ProductCategory,
    pub unit: ProductUnit,
    pub min_stock: i32,
    pub max_stock: Option<i32>,
    pub reorder_point: Option<i32>,
    pub unit_cost: Decimal,
    pub unit_price: Decimal,
    pub supplier_id: Option<Uuid>,
    pub location: Option<String>,
    pub brand: Option<String>,
    pub taxable: bool,
    pub tax_rate: Option<Decimal>,
}

/// Update product DTO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProduct {
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<ProductCategory>,
    pub unit: Option<ProductUnit>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,
    pub reorder_point: Option<i32>,
    pub unit_cost: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub supplier_id: Option<Uuid>,
    pub location: Option<String>,
    pub brand: Option<String>,
    pub taxable: Option<bool>,
    pub tax_rate: Option<Decimal>,
    pub active: Option<bool>,
}

/// Create stock movement DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockMovement {
    pub product_id: Uuid,
    pub movement_type: StockMovementType,
    pub quantity: i32,
    pub unit_cost: Option<Decimal>,
    pub reference: Option<String>,
    pub related_id: Option<Uuid>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

/// Product filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductFilters {
    pub query: Option<String>,
    pub sku: Option<String>,
    pub category: Option<ProductCategory>,
    pub supplier_id: Option<Uuid>,
    pub low_stock_only: Option<bool>,
    pub active_only: Option<bool>,
}

/// Stock movement filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StockMovementFilters {
    pub product_id: Option<Uuid>,
    pub movement_type: Option<StockMovementType>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
}

/// Low stock alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowStockAlert {
    pub product_id: Uuid,
    pub product_name: String,
    pub sku: String,
    pub current_stock: i32,
    pub min_stock: i32,
    pub reorder_point: Option<i32>,
    pub supplier_name: Option<String>,
}

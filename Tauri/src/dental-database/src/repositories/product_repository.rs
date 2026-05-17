//! Product/Inventory repository

use chrono::{DateTime, Utc};
use rusqlite::params;
use rust_decimal::Decimal;
use uuid::Uuid;

use dental_core::models::{Product, CreateProduct, UpdateProduct, ProductListItem, StockMovement, CreateStockMovement, LowStockAlert};
use dental_core::{ProductCategory, ProductUnit, StockMovementType};

use crate::{DbError, DbPool, DbResult};

/// Product repository
pub struct ProductRepository {
    pool: DbPool,
}

impl ProductRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new product
    pub fn create(&self, data: CreateProduct) -> DbResult<Product> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        conn.execute(
            r#"
            INSERT INTO products (
                id, sku, barcode, name, description, category, unit,
                current_stock, min_stock, max_stock, reorder_point,
                unit_cost, unit_price, supplier_id, location, brand,
                taxable, tax_rate, active, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21
            )
            "#,
            params![
                id.to_string(),
                data.sku,
                data.barcode,
                data.name,
                data.description,
                data.category.to_string(),
                data.unit.to_string(),
                0,
                data.min_stock,
                data.max_stock,
                data.reorder_point,
                data.unit_cost.to_string(),
                data.unit_price.to_string(),
                data.supplier_id.map(|s| s.to_string()),
                data.location,
                data.brand,
                data.taxable,
                data.tax_rate.map(|t| t.to_string()),
                true,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;
        
        self.find_by_id(id)
    }
    
    /// Find product by ID
    pub fn find_by_id(&self, id: Uuid) -> DbResult<Product> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            "SELECT * FROM products WHERE id = ?1",
            [id.to_string()],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Product {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Find product by SKU
    pub fn find_by_sku(&self, sku: &str) -> DbResult<Product> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            "SELECT * FROM products WHERE sku = ?1",
            [sku],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Product SKU {}", sku)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Update product stock
    pub fn update_stock(&self, id: Uuid, quantity_change: i32, movement: CreateStockMovement, user_id: Uuid) -> DbResult<()> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        
        // Get current stock
        let product = self.find_by_id(id)?;
        let new_stock = product.current_stock + quantity_change;
        
        if new_stock < 0 {
            return Err(DbError::QueryError("Insufficient stock".to_string()));
        }
        
        // Update stock
        conn.execute(
            "UPDATE products SET current_stock = ?2, updated_at = ?3 WHERE id = ?1",
            params![id.to_string(), new_stock, now.to_rfc3339()],
        )?;
        
        // Record movement
        let movement_id = Uuid::new_v4();
        let unit_cost = movement.unit_cost.unwrap_or(product.unit_cost);
        let total_value = unit_cost * Decimal::from(quantity_change.abs());
        
        conn.execute(
            r#"
            INSERT INTO stock_movements (
                id, product_id, movement_type, quantity, unit_cost, total_value,
                reference, related_id, batch_number, expiration_date, notes, user_id, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                movement_id.to_string(),
                id.to_string(),
                movement.movement_type.to_string(),
                quantity_change,
                unit_cost.to_string(),
                total_value.to_string(),
                movement.reference,
                movement.related_id.map(|r| r.to_string()),
                movement.batch_number,
                movement.expiration_date.map(|d| d.to_string()),
                movement.notes,
                user_id.to_string(),
                now.to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Get low stock alerts
    pub fn get_low_stock_alerts(&self) -> DbResult<Vec<LowStockAlert>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT p.id, p.name, p.sku, p.current_stock, p.min_stock, p.reorder_point, s.name
            FROM products p
            LEFT JOIN suppliers s ON s.id = p.supplier_id
            WHERE p.active = 1 AND p.current_stock <= p.min_stock
            ORDER BY p.current_stock
            "#
        )?;
        
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(LowStockAlert {
                product_id: Uuid::parse_str(&id).unwrap_or_default(),
                product_name: row.get(1)?,
                sku: row.get(2)?,
                current_stock: row.get(3)?,
                min_stock: row.get(4)?,
                reorder_point: row.get(5)?,
                supplier_name: row.get(6)?,
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    /// List products
    pub fn list(&self, active_only: bool) -> DbResult<Vec<ProductListItem>> {
        let conn = self.pool.get()?;
        
        let sql = if active_only {
            "SELECT id, sku, name, category, current_stock, min_stock, unit_cost, unit_price, active FROM products WHERE active = 1 ORDER BY name"
        } else {
            "SELECT id, sku, name, category, current_stock, min_stock, unit_cost, unit_price, active FROM products ORDER BY name"
        };
        
        let mut stmt = conn.prepare(sql)?;
        
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let category_str: String = row.get(3)?;
            let unit_cost_str: String = row.get(6)?;
            let unit_price_str: String = row.get(7)?;
            let current_stock: i32 = row.get(4)?;
            let min_stock: i32 = row.get(5)?;
            
            Ok(ProductListItem {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                sku: row.get(1)?,
                name: row.get(2)?,
                category: category_str.parse().unwrap_or(ProductCategory::Other),
                current_stock,
                min_stock,
                unit_cost: unit_cost_str.parse().unwrap_or(Decimal::ZERO),
                unit_price: unit_price_str.parse().unwrap_or(Decimal::ZERO),
                is_low_stock: current_stock <= min_stock,
                active: row.get(8)?,
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Update product details
    pub fn update(&self, id: Uuid, data: UpdateProduct) -> DbResult<Product> {
        let conn = self.pool.get()?;
        let now = Utc::now();

        let existing = self.find_by_id(id)?;

        let barcode = data.barcode.or(existing.barcode);
        let name = data.name.unwrap_or(existing.name);
        let description = data.description.or(existing.description);
        let category = data.category.unwrap_or(existing.category);
        let unit = data.unit.unwrap_or(existing.unit);
        let min_stock = data.min_stock.unwrap_or(existing.min_stock);
        let max_stock = data.max_stock.or(existing.max_stock);
        let reorder_point = data.reorder_point.or(existing.reorder_point);
        let unit_cost = data.unit_cost.unwrap_or(existing.unit_cost);
        let unit_price = data.unit_price.unwrap_or(existing.unit_price);
        let supplier_id = data.supplier_id.or(existing.supplier_id);
        let location = data.location.or(existing.location);
        let brand = data.brand.or(existing.brand);
        let taxable = data.taxable.unwrap_or(existing.taxable);
        let tax_rate = data.tax_rate.or(existing.tax_rate);
        let active = data.active.unwrap_or(existing.active);

        conn.execute(
            r#"
            UPDATE products
            SET barcode = ?2,
                name = ?3,
                description = ?4,
                category = ?5,
                unit = ?6,
                min_stock = ?7,
                max_stock = ?8,
                reorder_point = ?9,
                unit_cost = ?10,
                unit_price = ?11,
                supplier_id = ?12,
                location = ?13,
                brand = ?14,
                taxable = ?15,
                tax_rate = ?16,
                active = ?17,
                updated_at = ?18
            WHERE id = ?1
            "#,
            params![
                id.to_string(),
                barcode,
                name,
                description,
                category.to_string(),
                unit.to_string(),
                min_stock,
                max_stock,
                reorder_point,
                unit_cost.to_string(),
                unit_price.to_string(),
                supplier_id.map(|s| s.to_string()),
                location,
                brand,
                taxable,
                tax_rate.map(|t| t.to_string()),
                active,
                now.to_rfc3339(),
            ],
        )?;

        self.find_by_id(id)
    }

    /// Activate or deactivate a product
    pub fn set_active(&self, id: Uuid, active: bool) -> DbResult<Product> {
        let conn = self.pool.get()?;
        let now = Utc::now();

        conn.execute(
            "UPDATE products SET active = ?2, updated_at = ?3 WHERE id = ?1",
            params![id.to_string(), active, now.to_rfc3339()],
        )?;

        self.find_by_id(id)
    }
    
    /// Get stock movements for a product
    pub fn get_movements(&self, product_id: Uuid) -> DbResult<Vec<StockMovement>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM stock_movements WHERE product_id = ?1 ORDER BY created_at DESC LIMIT 100"
        )?;
        
        let rows = stmt.query_map([product_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let pid: String = row.get(1)?;
            let movement_type_str: String = row.get(2)?;
            let unit_cost_str: String = row.get(4)?;
            let total_value_str: String = row.get(5)?;
            let user_id: String = row.get(11)?;
            let created_at_str: String = row.get(12)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            Ok(StockMovement {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                product_id: Uuid::parse_str(&pid).unwrap_or_default(),
                movement_type: movement_type_str.parse().unwrap_or(StockMovementType::Adjustment),
                quantity: row.get(3)?,
                unit_cost: unit_cost_str.parse().unwrap_or(Decimal::ZERO),
                total_value: total_value_str.parse().unwrap_or(Decimal::ZERO),
                reference: row.get(6)?,
                related_id: row.get::<_, Option<String>>(7)?.and_then(|s| Uuid::parse_str(&s).ok()),
                batch_number: row.get(8)?,
                expiration_date: None,
                notes: row.get(10)?,
                user_id: Uuid::parse_str(&user_id).unwrap_or_default(),
                created_at,
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    fn map_row(&self, row: &rusqlite::Row) -> Result<Product, rusqlite::Error> {
        let id: String = row.get(0)?;
        let category_str: String = row.get(5)?;
        let unit_str: String = row.get(6)?;
        let unit_cost_str: String = row.get(11)?;
        let unit_price_str: String = row.get(12)?;
        let created_at_str: String = row.get(21)?;
        let updated_at_str: String = row.get(22)?;
        
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        Ok(Product {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            sku: row.get(1)?,
            barcode: row.get(2)?,
            name: row.get(3)?,
            description: row.get(4)?,
            category: category_str.parse().unwrap_or(ProductCategory::Other),
            unit: unit_str.parse().unwrap_or(ProductUnit::Unit),
            current_stock: row.get(7)?,
            min_stock: row.get(8)?,
            max_stock: row.get(9)?,
            reorder_point: row.get(10)?,
            unit_cost: unit_cost_str.parse().unwrap_or(Decimal::ZERO),
            unit_price: unit_price_str.parse().unwrap_or(Decimal::ZERO),
            supplier_id: row.get::<_, Option<String>>(13)?.and_then(|s| Uuid::parse_str(&s).ok()),
            location: row.get(14)?,
            brand: row.get(15)?,
            manufacturer: None,
            taxable: row.get(17)?,
            tax_rate: row.get::<_, Option<String>>(18)?.and_then(|s| s.parse().ok()),
            active: row.get(19)?,
            image_url: None,
            notes: row.get(20)?,
            created_at,
            updated_at,
        })
    }
}

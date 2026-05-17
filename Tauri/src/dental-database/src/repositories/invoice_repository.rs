//! Invoice repository

use chrono::{DateTime, Utc};
use rusqlite::params;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use uuid::Uuid;

use dental_core::models::{Invoice, InvoiceItem, Payment, CreateInvoice, CreatePayment, InvoiceListItem};
use dental_core::InvoiceStatus;

use crate::{DbError, DbPool, DbResult};

/// Invoice repository
pub struct InvoiceRepository {
    pool: DbPool,
}

impl InvoiceRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new invoice
    pub fn create(&self, data: CreateInvoice, created_by: Uuid) -> DbResult<Invoice> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        // Generate invoice number
        let invoice_number = self.generate_invoice_number(&conn)?;
        
        // Calculate totals
        let mut subtotal = Decimal::ZERO;
        for item in &data.items {
            let item_total = Decimal::from(item.quantity) * item.unit_price - item.discount.unwrap_or(Decimal::ZERO);
            subtotal += item_total;
        }
        
        let tax_rate = data.tax_rate.unwrap_or(Decimal::from(16));
        let tax = subtotal * (tax_rate / Decimal::from(100));
        let discount = data.discount.unwrap_or(Decimal::ZERO);
        let total = subtotal + tax - discount;
        
        conn.execute(
            r#"
            INSERT INTO invoices (
                id, invoice_number, patient_id, clinic_id, date, due_date,
                status, subtotal, tax, tax_rate, discount, total,
                amount_paid, balance, notes, created_by, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
            )
            "#,
            params![
                id.to_string(),
                invoice_number,
                data.patient_id.to_string(),
                data.clinic_id.map(|c| c.to_string()),
                now.to_rfc3339(),
                data.due_date.map(|d| d.to_string()),
                "pending",
                subtotal.to_string(),
                tax.to_string(),
                tax_rate.to_string(),
                discount.to_string(),
                total.to_string(),
                "0",
                total.to_string(),
                data.notes,
                created_by.to_string(),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;
        
        // Insert items
        for item_data in data.items {
            let item_id = Uuid::new_v4();
            let item_discount = item_data.discount.unwrap_or(Decimal::ZERO);
            let item_total = Decimal::from(item_data.quantity) * item_data.unit_price - item_discount;
            
            conn.execute(
                r#"
                INSERT INTO invoice_items (id, invoice_id, treatment_id, product_id, description, quantity, unit_price, discount, total)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    item_id.to_string(),
                    id.to_string(),
                    item_data.treatment_id.map(|t| t.to_string()),
                    item_data.product_id.map(|p| p.to_string()),
                    item_data.description,
                    item_data.quantity,
                    item_data.unit_price.to_string(),
                    item_discount.to_string(),
                    item_total.to_string(),
                ],
            )?;
        }
        
        self.find_by_id(id)
    }
    
    /// Find invoice by ID
    pub fn find_by_id(&self, id: Uuid) -> DbResult<Invoice> {
        let conn = self.pool.get()?;
        
        conn.query_row(
            r#"
            SELECT id, invoice_number, patient_id, clinic_id, date, due_date,
                   status, subtotal, tax, tax_rate, discount, total,
                   amount_paid, balance, notes, internal_notes,
                   cfdi_uuid, cfdi_status, created_by, created_at, updated_at
            FROM invoices
            WHERE id = ?1
            "#,
            [id.to_string()],
            |row| self.map_row(row),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Invoice {}", id)),
            _ => DbError::QueryError(e.to_string()),
        })
    }
    
    /// Add payment to invoice
    pub fn add_payment(&self, data: CreatePayment, received_by: Uuid) -> DbResult<Payment> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        // Insert payment
        conn.execute(
            r#"
            INSERT INTO payments (id, invoice_id, amount, payment_method, reference, date, authorization_code, notes, received_by, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                id.to_string(),
                data.invoice_id.to_string(),
                data.amount.to_string(),
                data.payment_method.to_string(),
                data.reference,
                now.to_rfc3339(),
                data.authorization_code,
                data.notes,
                received_by.to_string(),
                now.to_rfc3339(),
            ],
        )?;
        
        // Update invoice balance
        let invoice = self.find_by_id(data.invoice_id)?;
        let new_amount_paid = invoice.amount_paid + data.amount;
        let new_balance = invoice.total - new_amount_paid;
        let new_status = if new_balance <= Decimal::ZERO {
            InvoiceStatus::Paid
        } else {
            InvoiceStatus::PartiallyPaid
        };
        
        conn.execute(
            "UPDATE invoices SET amount_paid = ?2, balance = ?3, status = ?4, updated_at = ?5 WHERE id = ?1",
            params![
                data.invoice_id.to_string(),
                new_amount_paid.to_string(),
                new_balance.to_string(),
                new_status.to_string(),
                now.to_rfc3339(),
            ],
        )?;
        
        Ok(Payment::new(data.invoice_id, data.amount, data.payment_method, received_by))
    }
    
    /// Get invoice items
    pub fn get_items(&self, invoice_id: Uuid) -> DbResult<Vec<InvoiceItem>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, invoice_id, treatment_id, product_id, description, quantity, unit_price, discount, total FROM invoice_items WHERE invoice_id = ?1"
        )?;
        
        let rows = stmt.query_map([invoice_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let inv_id: String = row.get(1)?;
            let unit_price_str: String = row.get(6)?;
            let discount_str: String = row.get(7)?;
            let total_str: String = row.get(8)?;
            
            Ok(InvoiceItem {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                invoice_id: Uuid::parse_str(&inv_id).unwrap_or_default(),
                treatment_id: row.get::<_, Option<String>>(2)?.and_then(|s| Uuid::parse_str(&s).ok()),
                product_id: row.get::<_, Option<String>>(3)?.and_then(|s| Uuid::parse_str(&s).ok()),
                description: row.get(4)?,
                quantity: row.get(5)?,
                unit_price: unit_price_str.parse().unwrap_or(Decimal::ZERO),
                discount: discount_str.parse().unwrap_or(Decimal::ZERO),
                total: total_str.parse().unwrap_or(Decimal::ZERO),
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
    
    /// List invoices by patient
    pub fn list_by_patient(&self, patient_id: Uuid) -> DbResult<Vec<InvoiceListItem>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT i.id, i.invoice_number, i.patient_id, p.first_name || ' ' || p.last_name,
                   i.date, i.total, i.balance, i.status, i.due_date
            FROM invoices i
            JOIN patients p ON p.id = i.patient_id
            WHERE i.patient_id = ?1
            ORDER BY i.date DESC
            "#
        )?;
        
        let rows = stmt.query_map([patient_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let pid: String = row.get(2)?;
            let date_str: String = row.get(4)?;
            let total_str: String = row.get(5)?;
            let balance_str: String = row.get(6)?;
            let status_str: String = row.get(7)?;
            let due_date: Option<String> = row.get(8)?;
            
            let date = DateTime::parse_from_rfc3339(&date_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let is_overdue = if let Some(due) = due_date {
                chrono::NaiveDate::parse_from_str(&due, "%Y-%m-%d")
                    .map(|d| Utc::now().date_naive() > d)
                    .unwrap_or(false)
            } else {
                false
            };
            
            Ok(InvoiceListItem {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                invoice_number: row.get(1)?,
                patient_id: Uuid::parse_str(&pid).unwrap_or_default(),
                patient_name: row.get(3)?,
                date,
                total: total_str.parse().unwrap_or(Decimal::ZERO),
                balance: balance_str.parse().unwrap_or(Decimal::ZERO),
                status: status_str.parse().unwrap_or(InvoiceStatus::Pending),
                is_overdue,
            })
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Sum payments for a date range
    pub fn sum_payments_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Decimal> {
        let conn = self.pool.get()?;
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();

        let total: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(CAST(amount AS REAL)), 0) FROM payments WHERE date >= ?1 AND date <= ?2",
                params![start_str, end_str],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        Ok(Decimal::from_f64(total).unwrap_or(Decimal::ZERO))
    }

    /// Sum pending balances across invoices
    pub fn sum_pending_balance(&self) -> DbResult<Decimal> {
        let conn = self.pool.get()?;
        let total: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(CAST(balance AS REAL)), 0) FROM invoices WHERE balance > 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        Ok(Decimal::from_f64(total).unwrap_or(Decimal::ZERO))
    }
    
    fn generate_invoice_number(&self, conn: &rusqlite::Connection) -> DbResult<String> {
        let prefix: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'invoice.prefix'",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "INV".to_string());
        
        let counter: i64 = conn.query_row(
            "SELECT value FROM settings WHERE key = 'invoice.counter'",
            [],
            |row| {
                let val: String = row.get(0)?;
                Ok(val.parse::<i64>().unwrap_or(1))
            },
        ).unwrap_or(1);
        
        let year = chrono::Utc::now().format("%Y");
        let invoice_number = format!("{}-{}-{:05}", prefix, year, counter);
        
        conn.execute(
            "UPDATE settings SET value = ?1, updated_at = ?2 WHERE key = 'invoice.counter'",
            params![(counter + 1).to_string(), Utc::now().to_rfc3339()],
        )?;
        
        Ok(invoice_number)
    }
    
    fn map_row(&self, row: &rusqlite::Row) -> Result<Invoice, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(2)?;
        let date_str: String = row.get(4)?;
        let status_str: String = row.get(6)?;
        let created_by: String = row.get(18)?;
        let created_at_str: String = row.get(19)?;
        let updated_at_str: String = row.get(20)?;
        
        let date = DateTime::parse_from_rfc3339(&date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        fn parse_decimal(s: &str) -> Decimal {
            s.parse().unwrap_or(Decimal::ZERO)
        }
        
        Ok(Invoice {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            invoice_number: row.get(1)?,
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            clinic_id: row.get::<_, Option<String>>(3)?.and_then(|s| Uuid::parse_str(&s).ok()),
            date,
            due_date: None,
            status: status_str.parse().unwrap_or(InvoiceStatus::Pending),
            subtotal: parse_decimal(&row.get::<_, String>(7)?),
            tax: parse_decimal(&row.get::<_, String>(8)?),
            tax_rate: parse_decimal(&row.get::<_, String>(9)?),
            discount: parse_decimal(&row.get::<_, String>(10)?),
            total: parse_decimal(&row.get::<_, String>(11)?),
            amount_paid: parse_decimal(&row.get::<_, String>(12)?),
            balance: parse_decimal(&row.get::<_, String>(13)?),
            notes: row.get(14)?,
            internal_notes: row.get(15)?,
            cfdi_uuid: row.get(16)?,
            cfdi_status: row.get(17)?,
            created_by: Uuid::parse_str(&created_by).unwrap_or_default(),
            created_at,
            updated_at,
        })
    }
}

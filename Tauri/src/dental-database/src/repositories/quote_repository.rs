//! Quote repository

use chrono::{DateTime, Utc};
use rusqlite::params;
use rust_decimal::Decimal;
use uuid::Uuid;

use dental_core::models::{CreateQuote, Quote, QuoteItem, QuoteWithItems};

use crate::{DbError, DbPool, DbResult};

/// Quote repository
pub struct QuoteRepository {
    pool: DbPool,
}

impl QuoteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Create a new quote
    pub fn create(&self, data: CreateQuote, created_by: Uuid) -> DbResult<QuoteWithItems> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let quote_id = Uuid::new_v4();

        let mut total = Decimal::ZERO;
        for item in &data.items {
            let item_total = Decimal::from(item.quantity) * item.unit_price;
            total += item_total;
        }

        conn.execute(
            r#"
            INSERT INTO quotes (id, patient_id, created_by, notes, total, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                quote_id.to_string(),
                data.patient_id.to_string(),
                created_by.to_string(),
                data.notes,
                total.to_string(),
                now.to_rfc3339(),
            ],
        )?;

        for item in data.items {
            let item_id = Uuid::new_v4();
            let item_total = Decimal::from(item.quantity) * item.unit_price;

            conn.execute(
                r#"
                INSERT INTO quote_items (id, quote_id, description, quantity, unit_price, total)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![
                    item_id.to_string(),
                    quote_id.to_string(),
                    item.description,
                    item.quantity,
                    item.unit_price.to_string(),
                    item_total.to_string(),
                ],
            )?;
        }

        self.get_with_items(quote_id)
    }

    /// Get quote by id with items
    pub fn get_with_items(&self, id: Uuid) -> DbResult<QuoteWithItems> {
        let conn = self.pool.get()?;

        let quote = conn
            .query_row(
                r#"
                SELECT id, patient_id, created_by, notes, total, created_at
                FROM quotes
                WHERE id = ?1
                "#,
                [id.to_string()],
                |row| self.map_quote_row(row),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Quote {}", id)),
                _ => DbError::QueryError(e.to_string()),
            })?;

        let items = self.list_items(id)?;
        Ok(QuoteWithItems { quote, items })
    }

    /// List quotes for a patient
    pub fn list_by_patient(&self, patient_id: Uuid) -> DbResult<Vec<QuoteWithItems>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, patient_id, created_by, notes, total, created_at
            FROM quotes
            WHERE patient_id = ?1
            ORDER BY created_at DESC
            "#,
        )?;

        let rows = stmt.query_map([patient_id.to_string()], |row| self.map_quote_row(row))?;
        let mut quotes = Vec::new();

        for row in rows {
            if let Ok(quote) = row {
                let items = self.list_items(quote.id)?;
                quotes.push(QuoteWithItems { quote, items });
            }
        }

        Ok(quotes)
    }

    /// Delete quote
    pub fn delete(&self, id: Uuid) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM quotes WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn list_items(&self, quote_id: Uuid) -> DbResult<Vec<QuoteItem>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, quote_id, description, quantity, unit_price, total
            FROM quote_items
            WHERE quote_id = ?1
            ORDER BY id
            "#,
        )?;

        let rows = stmt.query_map([quote_id.to_string()], |row| self.map_item_row(row))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    fn map_quote_row(&self, row: &rusqlite::Row) -> Result<Quote, rusqlite::Error> {
        let id: String = row.get(0)?;
        let patient_id: String = row.get(1)?;
        let created_by: String = row.get(2)?;
        let total_str: String = row.get(4)?;
        let created_at_str: String = row.get(5)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Quote {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            patient_id: Uuid::parse_str(&patient_id).unwrap_or_default(),
            created_by: Uuid::parse_str(&created_by).unwrap_or_default(),
            notes: row.get(3)?,
            total: total_str.parse().unwrap_or(Decimal::ZERO),
            created_at,
        })
    }

    fn map_item_row(&self, row: &rusqlite::Row) -> Result<QuoteItem, rusqlite::Error> {
        let id: String = row.get(0)?;
        let quote_id: String = row.get(1)?;
        let unit_price_str: String = row.get(4)?;
        let total_str: String = row.get(5)?;

        Ok(QuoteItem {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            quote_id: Uuid::parse_str(&quote_id).unwrap_or_default(),
            description: row.get(2)?,
            quantity: row.get(3)?,
            unit_price: unit_price_str.parse().unwrap_or(Decimal::ZERO),
            total: total_str.parse().unwrap_or(Decimal::ZERO),
        })
    }
}

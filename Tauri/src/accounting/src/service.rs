use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use dental_core::models::{
    CreateInvoice, CreatePayment, DailyCashSummary, Invoice, InvoiceItem, InvoiceListItem,
    PatientBalance, Payment, PaymentFilters,
};
use dental_core::PaymentMethod;
use dental_database::{DbPool, InvoiceRepository};
use dental_database::rusqlite::{params_from_iter, types::Value};

use crate::error::{AccountingError, AccountingResult};

/// Accounting service for invoices, payments, and summaries
pub struct AccountingService {
    pool: DbPool,
}

impl AccountingService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_invoice(&self, data: CreateInvoice, created_by: Uuid) -> AccountingResult<Invoice> {
        let repo = InvoiceRepository::new(self.pool.clone());
        Ok(repo.create(data, created_by)?)
    }

    pub fn get_invoice(&self, invoice_id: Uuid) -> AccountingResult<Invoice> {
        let repo = InvoiceRepository::new(self.pool.clone());
        Ok(repo.find_by_id(invoice_id)?)
    }

    pub fn get_invoice_items(&self, invoice_id: Uuid) -> AccountingResult<Vec<InvoiceItem>> {
        let repo = InvoiceRepository::new(self.pool.clone());
        Ok(repo.get_items(invoice_id)?)
    }

    pub fn list_invoices_by_patient(&self, patient_id: Uuid) -> AccountingResult<Vec<InvoiceListItem>> {
        let repo = InvoiceRepository::new(self.pool.clone());
        Ok(repo.list_by_patient(patient_id)?)
    }

    pub fn add_payment(&self, data: CreatePayment, received_by: Uuid) -> AccountingResult<Payment> {
        let repo = InvoiceRepository::new(self.pool.clone());
        Ok(repo.add_payment(data, received_by)?)
    }

    pub fn list_payments(&self, filters: PaymentFilters) -> AccountingResult<Vec<Payment>> {
        let conn = self.pool.get()?;

        let mut conditions: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        if let Some(invoice_id) = filters.invoice_id {
            conditions.push("invoice_id = ?".to_string());
            values.push(Value::from(invoice_id.to_string()));
        }

        if let Some(method) = filters.payment_method {
            conditions.push("payment_method = ?".to_string());
            values.push(Value::from(method.to_string()));
        }

        if let Some(date_from) = filters.date_from {
            conditions.push("date >= ?".to_string());
            values.push(Value::from(date_from.to_rfc3339()));
        }

        if let Some(date_to) = filters.date_to {
            conditions.push("date <= ?".to_string());
            values.push(Value::from(date_to.to_rfc3339()));
        }

        if let Some(received_by) = filters.received_by {
            conditions.push("received_by = ?".to_string());
            values.push(Value::from(received_by.to_string()));
        }

        let where_sql = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT id, invoice_id, amount, payment_method, reference, date,
                   authorization_code, terminal_id, notes, is_refund, refund_reason,
                   received_by, created_at
            FROM payments
            {}
            ORDER BY date DESC
            "#,
            where_sql
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(values), |row| {
            let id: String = row.get(0)?;
            let invoice_id: String = row.get(1)?;
            let amount_str: String = row.get(2)?;
            let method_str: String = row.get(3)?;
            let date_str: String = row.get(5)?;
            let received_by: String = row.get(11)?;
            let created_at_str: String = row.get(12)?;

            let date = DateTime::parse_from_rfc3339(&date_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Payment {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                invoice_id: Uuid::parse_str(&invoice_id).unwrap_or_default(),
                amount: amount_str.parse().unwrap_or(Decimal::ZERO),
                payment_method: method_str.parse().unwrap_or(PaymentMethod::Other),
                reference: row.get(4)?,
                date,
                authorization_code: row.get(6)?,
                terminal_id: row.get(7)?,
                notes: row.get(8)?,
                is_refund: row.get::<_, i32>(9)? != 0,
                refund_reason: row.get(10)?,
                received_by: Uuid::parse_str(&received_by).unwrap_or_default(),
                created_at,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn daily_cash_summary(&self, date: NaiveDate) -> AccountingResult<DailyCashSummary> {
        let conn = self.pool.get()?;
        let date_str = date.to_string();

        let total_invoiced: Decimal = conn
            .query_row(
                "SELECT COALESCE(SUM(total), '0') FROM invoices WHERE date(date) = ?1",
                [date_str.clone()],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(Decimal::ZERO);

        let invoice_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM invoices WHERE date(date) = ?1",
                [date_str.clone()],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let payments = self.list_payments(PaymentFilters {
            date_from: Some(DateTime::<Utc>::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap(), Utc)),
            date_to: Some(DateTime::<Utc>::from_naive_utc_and_offset(date.and_hms_opt(23, 59, 59).unwrap(), Utc)),
            ..PaymentFilters::default()
        })?;

        let mut total_received = Decimal::ZERO;
        let mut total_cash = Decimal::ZERO;
        let mut total_card = Decimal::ZERO;
        let mut total_transfer = Decimal::ZERO;
        let mut total_other = Decimal::ZERO;

        for payment in &payments {
            total_received += payment.amount;
            match payment.payment_method {
                PaymentMethod::Cash => total_cash += payment.amount,
                PaymentMethod::CreditCard | PaymentMethod::DebitCard => total_card += payment.amount,
                PaymentMethod::BankTransfer => total_transfer += payment.amount,
                _ => total_other += payment.amount,
            }
        }

        Ok(DailyCashSummary {
            date,
            total_invoiced,
            total_received,
            total_cash,
            total_card,
            total_transfer,
            total_other,
            invoice_count,
            payment_count: payments.len() as i32,
        })
    }

    pub fn patient_balance(&self, patient_id: Uuid) -> AccountingResult<PatientBalance> {
        let conn = self.pool.get()?;

        let patient_name: String = conn
            .query_row(
                "SELECT first_name || ' ' || last_name FROM patients WHERE id = ?1",
                [patient_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| AccountingError::Database(e.to_string()))?;

        let invoices = self.list_invoices_by_patient(patient_id)?;

        let mut total_invoiced = Decimal::ZERO;
        let mut total_paid = Decimal::ZERO;
        let mut balance = Decimal::ZERO;
        let mut overdue_amount = Decimal::ZERO;
        let mut oldest_invoice_date: Option<DateTime<Utc>> = None;

        for invoice in &invoices {
            total_invoiced += invoice.total;
            balance += invoice.balance;
            total_paid += invoice.total - invoice.balance;

            if invoice.is_overdue {
                overdue_amount += invoice.balance;
            }

            if oldest_invoice_date.map(|d| invoice.date < d).unwrap_or(true) {
                oldest_invoice_date = Some(invoice.date);
            }
        }

        Ok(PatientBalance {
            patient_id,
            patient_name,
            total_invoiced,
            total_paid,
            balance,
            overdue_amount,
            oldest_invoice_date,
        })
    }
}

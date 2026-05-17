use rust_decimal::Decimal;
use uuid::Uuid;

use dental_core::models::{CreateInvoice, CreateInvoiceItem, CreatePayment, CreateStockMovement};
use dental_core::{PaymentMethod, StockMovementType};
use dental_database::{DbPool, InvoiceRepository, ProductRepository};

use crate::error::{PosError, PosResult};
use crate::models::{SaleItemInput, SalePaymentInput, SaleRequest, SaleResult};

/// Point of sale service
pub struct PosService {
    pool: DbPool,
}

impl PosService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_sale(&self, request: SaleRequest, created_by: Uuid) -> PosResult<SaleResult> {
        if request.items.is_empty() {
            return Err(PosError::Validation("At least one sale item is required".to_string()));
        }

        let product_repo = ProductRepository::new(self.pool.clone());
        let invoice_repo = InvoiceRepository::new(self.pool.clone());

        let mut items: Vec<CreateInvoiceItem> = Vec::new();

        for item in &request.items {
            if item.quantity <= 0 {
                return Err(PosError::Validation("Item quantity must be positive".to_string()));
            }

            let product = product_repo.find_by_id(item.product_id)?;
            let unit_price = item.unit_price.unwrap_or(product.unit_price);
            let description = item
                .description
                .clone()
                .unwrap_or_else(|| product.name.clone());

            items.push(CreateInvoiceItem {
                treatment_id: None,
                product_id: Some(product.id),
                description,
                quantity: item.quantity,
                unit_price,
                discount: item.discount,
            });
        }

        let invoice = invoice_repo.create(
            CreateInvoice {
                patient_id: request.patient_id,
                clinic_id: request.clinic_id,
                due_date: None,
                tax_rate: None,
                discount: None,
                notes: request.notes,
                items: items.clone(),
            },
            created_by,
        )?;

        for item in request.items {
            let product = product_repo.find_by_id(item.product_id)?;
            let movement = CreateStockMovement {
                product_id: product.id,
                movement_type: StockMovementType::Sale,
                quantity: -item.quantity,
                unit_cost: Some(product.unit_cost),
                reference: Some(invoice.invoice_number.clone()),
                related_id: Some(invoice.id),
                batch_number: None,
                expiration_date: None,
                notes: Some("POS sale".to_string()),
            };

            product_repo.update_stock(product.id, -item.quantity, movement, created_by)?;
        }

        let payment_id = if let Some(payment) = request.payment {
            Some(self.apply_payment(invoice.id, payment, created_by)?.id)
        } else {
            None
        };

        Ok(SaleResult {
            invoice_id: invoice.id,
            invoice_number: invoice.invoice_number,
            item_count: items.len(),
            payment_id,
        })
    }

    fn apply_payment(
        &self,
        invoice_id: Uuid,
        payment: SalePaymentInput,
        received_by: Uuid,
    ) -> PosResult<dental_core::models::Payment> {
        let invoice_repo = InvoiceRepository::new(self.pool.clone());

        let create_payment = CreatePayment {
            invoice_id,
            amount: payment.amount,
            payment_method: payment.payment_method,
            reference: payment.reference,
            authorization_code: payment.authorization_code,
            notes: payment.notes,
        };

        Ok(invoice_repo.add_payment(create_payment, received_by)?)
    }
}

impl Default for SalePaymentInput {
    fn default() -> Self {
        Self {
            amount: Decimal::ZERO,
            payment_method: PaymentMethod::Cash,
            reference: None,
            authorization_code: None,
            notes: None,
        }
    }
}

impl From<SaleItemInput> for CreateInvoiceItem {
    fn from(item: SaleItemInput) -> Self {
        CreateInvoiceItem {
            treatment_id: None,
            product_id: Some(item.product_id),
            description: item.description.unwrap_or_default(),
            quantity: item.quantity,
            unit_price: item.unit_price.unwrap_or(Decimal::ZERO),
            discount: item.discount,
        }
    }
}

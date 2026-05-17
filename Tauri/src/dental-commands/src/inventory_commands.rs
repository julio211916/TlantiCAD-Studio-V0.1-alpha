//! Inventory Tauri commands

use tauri::State;
use uuid::Uuid;

use dental_core::models::{CreateProduct, CreateStockMovement, LowStockAlert, Product, ProductListItem, StockMovement, UpdateProduct};
use dental_database::repositories::ProductRepository;

use crate::{CommandResult, DentalCommandError, DentalState};

/// Create a new product
#[tauri::command]
pub fn product_create(
    state: State<'_, DentalState>,
    data: CreateProduct,
) -> CommandResult<Product> {
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.create(data).map_err(|e| e.into())
}

/// Get product by ID
#[tauri::command]
pub fn product_get(
    state: State<'_, DentalState>,
    id: String,
) -> CommandResult<Product> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid product ID".into()))?;
    
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.find_by_id(uuid).map_err(|e| e.into())
}

/// Get product by SKU
#[tauri::command]
pub fn product_get_by_sku(
    state: State<'_, DentalState>,
    sku: String,
) -> CommandResult<Product> {
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.find_by_sku(&sku).map_err(|e| e.into())
}

/// Update product details
#[tauri::command]
pub fn product_update(
    state: State<'_, DentalState>,
    id: String,
    data: UpdateProduct,
) -> CommandResult<Product> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid product ID".into()))?;

    let repo = ProductRepository::new(state.db.pool().clone());
    repo.update(uuid, data).map_err(|e| e.into())
}

/// Activate or deactivate product
#[tauri::command]
pub fn product_set_active(
    state: State<'_, DentalState>,
    id: String,
    active: bool,
) -> CommandResult<Product> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DentalCommandError::Validation("Invalid product ID".into()))?;

    let repo = ProductRepository::new(state.db.pool().clone());
    repo.set_active(uuid, active).map_err(|e| e.into())
}

/// List products
#[tauri::command]
pub fn product_list(
    state: State<'_, DentalState>,
    active_only: bool,
) -> CommandResult<Vec<ProductListItem>> {
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.list(active_only).map_err(|e| e.into())
}

/// Update stock (add movement)
#[tauri::command]
pub fn stock_update(
    state: State<'_, DentalState>,
    product_id: String,
    quantity_change: i32,
    movement: CreateStockMovement,
) -> CommandResult<()> {
    let user_id = state.get_current_user()
        .ok_or_else(|| DentalCommandError::PermissionDenied("Not logged in".into()))?;
    
    let uuid = Uuid::parse_str(&product_id)
        .map_err(|_| DentalCommandError::Validation("Invalid product ID".into()))?;
    
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.update_stock(uuid, quantity_change, movement, user_id).map_err(|e| e.into())
}

/// Get low stock alerts
#[tauri::command]
pub fn stock_get_low_alerts(
    state: State<'_, DentalState>,
) -> CommandResult<Vec<LowStockAlert>> {
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.get_low_stock_alerts().map_err(|e| e.into())
}

/// Get stock movements for a product
#[tauri::command]
pub fn stock_get_movements(
    state: State<'_, DentalState>,
    product_id: String,
) -> CommandResult<Vec<StockMovement>> {
    let uuid = Uuid::parse_str(&product_id)
        .map_err(|_| DentalCommandError::Validation("Invalid product ID".into()))?;
    
    let repo = ProductRepository::new(state.db.pool().clone());
    repo.get_movements(uuid).map_err(|e| e.into())
}

// ============ Aliases for frontend compatibility ============

/// List all inventory products (alias)
#[tauri::command]
pub fn inventory_list(
    state: State<'_, DentalState>,
) -> CommandResult<Vec<ProductListItem>> {
    product_list(state, true)
}

/// Get low stock alerts (alias)
#[tauri::command]
pub fn inventory_get_low_stock(
    state: State<'_, DentalState>,
) -> CommandResult<Vec<LowStockAlert>> {
    stock_get_low_alerts(state)
}

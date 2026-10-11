//! The inventory's error type. Provided.
//!
//! Callers match on these variants to decide what to do (show a message, return HTTP 409,
//! retry...). They never need to look inside a `sqlx::Error`, which is why the repository
//! translates database errors it understands into these domain errors.

/// Everything that can go wrong in the inventory.
#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("a product with SKU {0:?} already exists")]
    DuplicateSku(String),
    #[error("a warehouse named {0:?} already exists")]
    DuplicateWarehouse(String),
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("quantity must be positive, got {0}")]
    InvalidQuantity(i64),
    #[error("not enough stock: {available} available, {requested} requested")]
    InsufficientStock { available: i64, requested: i64 },
    #[error("can't transfer stock to the warehouse it's already in")]
    SameWarehouse,
    /// Anything we don't translate: the connection broke, the disk is full...
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub type Result<T, E = InventoryError> = std::result::Result<T, E>;

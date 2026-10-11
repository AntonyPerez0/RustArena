//! The repository trait. Provided.
//!
//! The rest of a program talks to storage only through this trait. That keeps SQL in one
//! file, lets you swap SQLite for PostgreSQL later by writing a second implementation, and
//! lets tests of business logic use a fake in memory.
//!
//! The methods are written as `fn ... -> impl Future<Output = ...> + Send` rather than
//! `async fn` so that the futures are guaranteed to be `Send` (movable between threads),
//! which web frameworks like axum require. In the implementation you still write
//! plain `async fn`.

use std::future::Future;

use crate::error::Result;
use crate::models::{NewProduct, Product, StockLevel, Warehouse};

pub trait InventoryRepo {
    /// Stores a new product and returns it with its id.
    /// A SKU that's already taken is `InventoryError::DuplicateSku(sku)`.
    fn create_product(&self, new: &NewProduct) -> impl Future<Output = Result<Product>> + Send;

    /// The product with this SKU, or `None`.
    fn product_by_sku(&self, sku: &str) -> impl Future<Output = Result<Option<Product>>> + Send;

    /// Every product, sorted by SKU.
    fn list_products(&self) -> impl Future<Output = Result<Vec<Product>>> + Send;

    /// Stores a new warehouse. A name that's already taken is
    /// `InventoryError::DuplicateWarehouse(name)`.
    fn create_warehouse(&self, name: &str) -> impl Future<Output = Result<Warehouse>> + Send;

    /// The warehouse with this name, or `None`.
    fn warehouse_by_name(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<Option<Warehouse>>> + Send;

    /// Adds `quantity` (must be positive, else `InvalidQuantity`) units of a product to a
    /// warehouse and returns the new quantity there. An unknown product or warehouse id is
    /// `NotFound`.
    fn add_stock(
        &self,
        warehouse_id: i64,
        product_id: i64,
        quantity: i64,
    ) -> impl Future<Output = Result<i64>> + Send;

    /// How many units of the product the warehouse holds (0 if it has never held any).
    fn stock_of(
        &self,
        warehouse_id: i64,
        product_id: i64,
    ) -> impl Future<Output = Result<i64>> + Send;

    /// Moves `quantity` units of a product from one warehouse to another, all or nothing.
    ///
    /// Errors: `InvalidQuantity` (not positive), `SameWarehouse`,
    /// `InsufficientStock { available, requested }` (the source holds too few), and
    /// `NotFound` (unknown destination warehouse). After any error, no stock has moved.
    fn transfer(
        &self,
        product_id: i64,
        from_warehouse: i64,
        to_warehouse: i64,
        quantity: i64,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Every non-zero stock level, sorted by warehouse name, then SKU.
    fn stock_report(&self) -> impl Future<Output = Result<Vec<StockLevel>>> + Send;
}

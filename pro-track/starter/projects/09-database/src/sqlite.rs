//! The SQLite implementation of [`InventoryRepo`]. This is the file you write.
//!
//! Use runtime-checked queries: `sqlx::query`, `sqlx::query_as::<_, T>` and
//! `sqlx::query_scalar`, with `?` placeholders and `.bind(value)` for every value.
//! Never build SQL with `format!`: binding is what keeps you safe from SQL injection.

use sqlx::SqlitePool;

use crate::error::Result;
use crate::models::{NewProduct, Product, StockLevel, Warehouse};
use crate::repo::InventoryRepo;

/// Inventory stored in a SQLite database.
#[derive(Clone)]
pub struct SqliteInventory {
    pool: SqlitePool,
}

impl SqliteInventory {
    /// Wraps a pool that [`crate::db::connect`] opened (so the schema is up to date).
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// The underlying pool, for the odd query the trait doesn't cover (tests use it).
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

impl InventoryRepo for SqliteInventory {
    async fn create_product(&self, new: &NewProduct) -> Result<Product> {
        // TODO: INSERT ... RETURNING id, sku, name, price_cents, with query_as.
        // A unique-constraint violation must become InventoryError::DuplicateSku.
        let _ = new;
        todo!("create_product")
    }

    async fn product_by_sku(&self, sku: &str) -> Result<Option<Product>> {
        let _ = sku;
        todo!("product_by_sku")
    }

    async fn list_products(&self) -> Result<Vec<Product>> {
        todo!("list_products")
    }

    async fn create_warehouse(&self, name: &str) -> Result<Warehouse> {
        // TODO: needs the warehouses table, which you create in a new migration.
        let _ = name;
        todo!("create_warehouse")
    }

    async fn warehouse_by_name(&self, name: &str) -> Result<Option<Warehouse>> {
        let _ = name;
        todo!("warehouse_by_name")
    }

    async fn add_stock(&self, warehouse_id: i64, product_id: i64, quantity: i64) -> Result<i64> {
        // TODO: an "upsert": INSERT ... ON CONFLICT (warehouse_id, product_id) DO UPDATE ...
        let _ = (warehouse_id, product_id, quantity);
        todo!("add_stock")
    }

    async fn stock_of(&self, warehouse_id: i64, product_id: i64) -> Result<i64> {
        let _ = (warehouse_id, product_id);
        todo!("stock_of")
    }

    async fn transfer(
        &self,
        product_id: i64,
        from_warehouse: i64,
        to_warehouse: i64,
        quantity: i64,
    ) -> Result<()> {
        // TODO: one transaction: self.pool.begin(), queries on &mut *tx, tx.commit().
        let _ = (product_id, from_warehouse, to_warehouse, quantity);
        todo!("transfer")
    }

    async fn stock_report(&self) -> Result<Vec<StockLevel>> {
        // TODO: a JOIN of stock, warehouses and products.
        todo!("stock_report")
    }
}

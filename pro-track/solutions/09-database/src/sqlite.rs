//! The SQLite implementation of [`InventoryRepo`]. This is the file you write.
//!
//! Use runtime-checked queries: `sqlx::query`, `sqlx::query_as::<_, T>` and
//! `sqlx::query_scalar`, with `?` placeholders and `.bind(value)` for every value.
//! Never build SQL with `format!`: binding is what keeps you safe from SQL injection.

use sqlx::SqlitePool;

use crate::error::{InventoryError, Result};
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

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.is_unique_violation())
}

fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.is_foreign_key_violation())
}

/// Translates "that id doesn't exist" (a foreign key violation) into a domain error.
fn missing(what: &'static str) -> impl FnOnce(sqlx::Error) -> InventoryError {
    move |err| {
        if is_foreign_key_violation(&err) {
            InventoryError::NotFound(what)
        } else {
            InventoryError::Database(err)
        }
    }
}

fn positive(quantity: i64) -> Result<()> {
    if quantity > 0 {
        Ok(())
    } else {
        Err(InventoryError::InvalidQuantity(quantity))
    }
}

const ADD_STOCK: &str = "INSERT INTO stock (warehouse_id, product_id, quantity) VALUES (?, ?, ?)
     ON CONFLICT (warehouse_id, product_id) DO UPDATE SET quantity = quantity + excluded.quantity
     RETURNING quantity";

impl InventoryRepo for SqliteInventory {
    async fn create_product(&self, new: &NewProduct) -> Result<Product> {
        sqlx::query_as::<_, Product>(
            "INSERT INTO products (sku, name, price_cents) VALUES (?, ?, ?)
             RETURNING id, sku, name, price_cents",
        )
        .bind(&new.sku)
        .bind(&new.name)
        .bind(new.price_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            if is_unique_violation(&err) {
                InventoryError::DuplicateSku(new.sku.clone())
            } else {
                err.into()
            }
        })
    }

    async fn product_by_sku(&self, sku: &str) -> Result<Option<Product>> {
        let product = sqlx::query_as::<_, Product>(
            "SELECT id, sku, name, price_cents FROM products WHERE sku = ?",
        )
        .bind(sku)
        .fetch_optional(&self.pool)
        .await?;
        Ok(product)
    }

    async fn list_products(&self) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            "SELECT id, sku, name, price_cents FROM products ORDER BY sku",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(products)
    }

    async fn create_warehouse(&self, name: &str) -> Result<Warehouse> {
        sqlx::query_as::<_, Warehouse>(
            "INSERT INTO warehouses (name) VALUES (?) RETURNING id, name",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            if is_unique_violation(&err) {
                InventoryError::DuplicateWarehouse(name.to_string())
            } else {
                err.into()
            }
        })
    }

    async fn warehouse_by_name(&self, name: &str) -> Result<Option<Warehouse>> {
        let warehouse =
            sqlx::query_as::<_, Warehouse>("SELECT id, name FROM warehouses WHERE name = ?")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;
        Ok(warehouse)
    }

    async fn add_stock(&self, warehouse_id: i64, product_id: i64, quantity: i64) -> Result<i64> {
        positive(quantity)?;
        sqlx::query_scalar(ADD_STOCK)
            .bind(warehouse_id)
            .bind(product_id)
            .bind(quantity)
            .fetch_one(&self.pool)
            .await
            .map_err(missing("product or warehouse"))
    }

    async fn stock_of(&self, warehouse_id: i64, product_id: i64) -> Result<i64> {
        let quantity: Option<i64> = sqlx::query_scalar(
            "SELECT quantity FROM stock WHERE warehouse_id = ? AND product_id = ?",
        )
        .bind(warehouse_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(quantity.unwrap_or(0))
    }

    async fn transfer(
        &self,
        product_id: i64,
        from_warehouse: i64,
        to_warehouse: i64,
        quantity: i64,
    ) -> Result<()> {
        positive(quantity)?;
        if from_warehouse == to_warehouse {
            return Err(InventoryError::SameWarehouse);
        }

        // Everything between begin() and commit() happens all together or not at all.
        // If we return early (with `?` or an error), `tx` is dropped and rolled back.
        let mut tx = self.pool.begin().await?;

        // Take the stock out only if there's enough: the check and the update are one
        // statement, so nothing can sneak in between them.
        let taken = sqlx::query(
            "UPDATE stock SET quantity = quantity - ?
             WHERE warehouse_id = ? AND product_id = ? AND quantity >= ?",
        )
        .bind(quantity)
        .bind(from_warehouse)
        .bind(product_id)
        .bind(quantity)
        .execute(&mut *tx)
        .await?;

        if taken.rows_affected() == 0 {
            let available: Option<i64> = sqlx::query_scalar(
                "SELECT quantity FROM stock WHERE warehouse_id = ? AND product_id = ?",
            )
            .bind(from_warehouse)
            .bind(product_id)
            .fetch_optional(&mut *tx)
            .await?;
            return Err(InventoryError::InsufficientStock {
                available: available.unwrap_or(0),
                requested: quantity,
            });
        }

        sqlx::query_scalar::<_, i64>(ADD_STOCK)
            .bind(to_warehouse)
            .bind(product_id)
            .bind(quantity)
            .fetch_one(&mut *tx)
            .await
            .map_err(missing("warehouse"))?;

        tx.commit().await?;
        Ok(())
    }

    async fn stock_report(&self) -> Result<Vec<StockLevel>> {
        let levels = sqlx::query_as::<_, StockLevel>(
            "SELECT w.name AS warehouse, p.sku AS sku, s.quantity AS quantity
             FROM stock s
             JOIN warehouses w ON w.id = s.warehouse_id
             JOIN products p ON p.id = s.product_id
             WHERE s.quantity > 0
             ORDER BY w.name, p.sku",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(levels)
    }
}

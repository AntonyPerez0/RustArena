//! A small warehouse inventory stored in SQLite.
//!
//! - [`db`] opens a connection pool and runs the migrations in `migrations/`.
//! - [`InventoryRepo`] is the repository trait: everything the rest of a program may do
//!   with stored inventory, with no SQL in sight.
//! - [`SqliteInventory`] implements it with SQL. That's the file you write.

pub mod db;
pub mod error;
pub mod models;
pub mod repo;
pub mod sqlite;

pub use error::{InventoryError, Result};
pub use models::{NewProduct, Product, StockLevel, Warehouse};
pub use repo::InventoryRepo;
pub use sqlite::SqliteInventory;

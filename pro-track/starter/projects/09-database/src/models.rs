//! Plain data types. Provided.
//!
//! `#[derive(FromRow)]` lets `sqlx::query_as::<_, Product>(...)` build a `Product` from a
//! row, matching columns to fields by name. The SQL must select every field, with the same
//! names (use `AS` when they differ).

use sqlx::FromRow;

/// A product as stored, with the id the database gave it.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Product {
    pub id: i64,
    pub sku: String,
    pub name: String,
    pub price_cents: i64,
}

/// What a caller supplies to create a product. There's no id yet: the database assigns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProduct {
    pub sku: String,
    pub name: String,
    pub price_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Warehouse {
    pub id: i64,
    pub name: String,
}

/// One line of the stock report: how many of a product one warehouse holds.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct StockLevel {
    pub warehouse: String,
    pub sku: String,
    pub quantity: i64,
}

//! A tiny command-line front end for the inventory. Provided: it only works once
//! `src/sqlite.rs` and your migration do.
//!
//!   cargo run -- add-product KB-01 "Mechanical keyboard" 4999
//!   cargo run -- add-warehouse Lyon
//!   cargo run -- receive Lyon KB-01 25
//!   cargo run -- transfer KB-01 Lyon Paris 10
//!   cargo run -- report
//!
//! The data lives in `inventory.db` (or wherever DATABASE_URL points), so it's still
//! there the next time you run it.

use std::process::ExitCode;

use inventory::{InventoryError, InventoryRepo, NewProduct, SqliteInventory, db};

const USAGE: &str = "usage:
  inventory add-product <sku> <name> <price_cents>
  inventory add-warehouse <name>
  inventory receive <warehouse> <sku> <quantity>
  inventory transfer <sku> <from-warehouse> <to-warehouse> <quantity>
  inventory products
  inventory report";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:inventory.db".to_string());
    match run(&url, &args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn run(url: &str, args: &[String]) -> Result<(), String> {
    let pool = db::connect(url)
        .await
        .map_err(|e| format!("can't open {url}: {e}"))?;
    let repo = SqliteInventory::new(pool);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["add-product", sku, name, price] => {
            let new = NewProduct {
                sku: sku.to_string(),
                name: name.to_string(),
                price_cents: number(price)?,
            };
            let product = repo.create_product(&new).await.map_err(text)?;
            println!("added product {} (id {})", product.sku, product.id);
        }
        ["add-warehouse", name] => {
            let warehouse = repo.create_warehouse(name).await.map_err(text)?;
            println!("added warehouse {} (id {})", warehouse.name, warehouse.id);
        }
        ["receive", warehouse, sku, quantity] => {
            let w = warehouse_id(&repo, warehouse).await?;
            let p = product_id(&repo, sku).await?;
            let now = repo
                .add_stock(w, p, number(quantity)?)
                .await
                .map_err(text)?;
            println!("{warehouse} now holds {now} x {sku}");
        }
        ["transfer", sku, from, to, quantity] => {
            let p = product_id(&repo, sku).await?;
            let (f, t) = (
                warehouse_id(&repo, from).await?,
                warehouse_id(&repo, to).await?,
            );
            repo.transfer(p, f, t, number(quantity)?)
                .await
                .map_err(text)?;
            println!("moved {quantity} x {sku} from {from} to {to}");
        }
        ["products"] => {
            for p in repo.list_products().await.map_err(text)? {
                println!("{}\t{}\t{}", p.sku, p.name, p.price_cents);
            }
        }
        ["report"] => {
            for line in repo.stock_report().await.map_err(text)? {
                println!("{}\t{}\t{}", line.warehouse, line.sku, line.quantity);
            }
        }
        _ => return Err(USAGE.to_string()),
    }
    Ok(())
}

async fn product_id(repo: &SqliteInventory, sku: &str) -> Result<i64, String> {
    match repo.product_by_sku(sku).await.map_err(text)? {
        Some(p) => Ok(p.id),
        None => Err(format!("no product with SKU {sku}")),
    }
}

async fn warehouse_id(repo: &SqliteInventory, name: &str) -> Result<i64, String> {
    match repo.warehouse_by_name(name).await.map_err(text)? {
        Some(w) => Ok(w.id),
        None => Err(format!("no warehouse named {name}")),
    }
}

fn number(s: &str) -> Result<i64, String> {
    s.parse()
        .map_err(|_| format!("{s:?} is not a whole number"))
}

fn text(e: InventoryError) -> String {
    e.to_string()
}

//! The inventory's contract, tested against a fresh in-memory database per test.
//! Provided: don't change this file (the grader checks that). Make it pass.
//!
//!   cargo test                      # everything
//!   cargo test transfer             # only tests whose name contains "transfer"

use inventory::{
    InventoryError, InventoryRepo, NewProduct, Product, SqliteInventory, StockLevel, Warehouse, db,
};

/// A brand new database with every migration applied.
async fn repo() -> SqliteInventory {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect and migrate");
    SqliteInventory::new(pool)
}

fn product(sku: &str, name: &str, price_cents: i64) -> NewProduct {
    NewProduct {
        sku: sku.to_string(),
        name: name.to_string(),
        price_cents,
    }
}

/// Code that only needs "an inventory" takes `&impl InventoryRepo`, not the SQLite type.
async fn seed(repo: &impl InventoryRepo) -> (Product, Warehouse, Warehouse) {
    let keyboard = repo
        .create_product(&product("KB-01", "Keyboard", 4999))
        .await
        .unwrap();
    let lyon = repo.create_warehouse("Lyon").await.unwrap();
    let paris = repo.create_warehouse("Paris").await.unwrap();
    (keyboard, lyon, paris)
}

#[tokio::test]
async fn migrations_create_every_table() {
    let repo = repo().await;
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(repo.pool())
            .await
            .unwrap();
    for table in ["products", "warehouses", "stock"] {
        assert!(
            tables.iter().any(|t| t == table),
            "no {table} table in {tables:?}"
        );
    }
    let applied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(repo.pool())
        .await
        .unwrap();
    assert!(
        applied >= 2,
        "expected your new migration to be applied too"
    );
}

#[tokio::test]
async fn create_and_find_a_product() {
    let repo = repo().await;
    let created = repo
        .create_product(&product("KB-01", "Keyboard", 4999))
        .await
        .unwrap();
    assert!(created.id > 0);
    assert_eq!(created.sku, "KB-01");
    assert_eq!(created.name, "Keyboard");
    assert_eq!(created.price_cents, 4999);

    let found = repo.product_by_sku("KB-01").await.unwrap();
    assert_eq!(found, Some(created));
    assert_eq!(repo.product_by_sku("NOPE").await.unwrap(), None);
}

#[tokio::test]
async fn a_duplicate_sku_is_a_domain_error() {
    let repo = repo().await;
    repo.create_product(&product("KB-01", "Keyboard", 4999))
        .await
        .unwrap();
    let err = repo
        .create_product(&product("KB-01", "Another keyboard", 1))
        .await
        .unwrap_err();
    assert!(
        matches!(err, InventoryError::DuplicateSku(ref sku) if sku == "KB-01"),
        "expected DuplicateSku(\"KB-01\"), got {err:?}"
    );
}

#[tokio::test]
async fn products_are_listed_by_sku() {
    let repo = repo().await;
    for (sku, name) in [
        ("MS-02", "Mouse"),
        ("KB-01", "Keyboard"),
        ("SC-07", "Screen"),
    ] {
        repo.create_product(&product(sku, name, 100)).await.unwrap();
    }
    let skus: Vec<String> = repo
        .list_products()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.sku)
        .collect();
    assert_eq!(skus, ["KB-01", "MS-02", "SC-07"]);
}

#[tokio::test]
async fn warehouses_have_unique_names() {
    let repo = repo().await;
    let lyon = repo.create_warehouse("Lyon").await.unwrap();
    assert!(lyon.id > 0);
    assert_eq!(lyon.name, "Lyon");
    assert_eq!(repo.warehouse_by_name("Lyon").await.unwrap(), Some(lyon));
    assert_eq!(repo.warehouse_by_name("Oslo").await.unwrap(), None);

    let err = repo.create_warehouse("Lyon").await.unwrap_err();
    assert!(
        matches!(err, InventoryError::DuplicateWarehouse(ref name) if name == "Lyon"),
        "expected DuplicateWarehouse(\"Lyon\"), got {err:?}"
    );
}

#[tokio::test]
async fn adding_stock_accumulates() {
    let repo = repo().await;
    let (keyboard, lyon, paris) = seed(&repo).await;
    assert_eq!(repo.stock_of(lyon.id, keyboard.id).await.unwrap(), 0);
    assert_eq!(repo.add_stock(lyon.id, keyboard.id, 5).await.unwrap(), 5);
    assert_eq!(repo.add_stock(lyon.id, keyboard.id, 7).await.unwrap(), 12);
    assert_eq!(repo.stock_of(lyon.id, keyboard.id).await.unwrap(), 12);
    assert_eq!(repo.stock_of(paris.id, keyboard.id).await.unwrap(), 0);
}

#[tokio::test]
async fn adding_stock_needs_a_positive_quantity() {
    let repo = repo().await;
    let (keyboard, lyon, _) = seed(&repo).await;
    for bad in [0, -3] {
        let err = repo.add_stock(lyon.id, keyboard.id, bad).await.unwrap_err();
        assert!(
            matches!(err, InventoryError::InvalidQuantity(q) if q == bad),
            "got {err:?}"
        );
    }
}

#[tokio::test]
async fn adding_stock_of_unknown_things_is_not_found() {
    let repo = repo().await;
    let (keyboard, lyon, _) = seed(&repo).await;
    let err = repo.add_stock(lyon.id, 9999, 1).await.unwrap_err();
    assert!(
        matches!(err, InventoryError::NotFound(_)),
        "unknown product: got {err:?}"
    );
    let err = repo.add_stock(9999, keyboard.id, 1).await.unwrap_err();
    assert!(
        matches!(err, InventoryError::NotFound(_)),
        "unknown warehouse: got {err:?}"
    );
}

#[tokio::test]
async fn transfer_moves_stock() {
    let repo = repo().await;
    let (keyboard, lyon, paris) = seed(&repo).await;
    repo.add_stock(lyon.id, keyboard.id, 10).await.unwrap();
    repo.transfer(keyboard.id, lyon.id, paris.id, 4)
        .await
        .unwrap();
    assert_eq!(repo.stock_of(lyon.id, keyboard.id).await.unwrap(), 6);
    assert_eq!(repo.stock_of(paris.id, keyboard.id).await.unwrap(), 4);
    repo.transfer(keyboard.id, lyon.id, paris.id, 6)
        .await
        .unwrap();
    assert_eq!(repo.stock_of(lyon.id, keyboard.id).await.unwrap(), 0);
    assert_eq!(repo.stock_of(paris.id, keyboard.id).await.unwrap(), 10);
}

#[tokio::test]
async fn transfer_validates_its_input() {
    let repo = repo().await;
    let (keyboard, lyon, paris) = seed(&repo).await;
    repo.add_stock(lyon.id, keyboard.id, 10).await.unwrap();
    let err = repo
        .transfer(keyboard.id, lyon.id, paris.id, 0)
        .await
        .unwrap_err();
    assert!(
        matches!(err, InventoryError::InvalidQuantity(0)),
        "got {err:?}"
    );
    let err = repo
        .transfer(keyboard.id, lyon.id, lyon.id, 1)
        .await
        .unwrap_err();
    assert!(matches!(err, InventoryError::SameWarehouse), "got {err:?}");
}

#[tokio::test]
async fn transfer_without_enough_stock_changes_nothing() {
    let repo = repo().await;
    let (keyboard, lyon, paris) = seed(&repo).await;
    repo.add_stock(lyon.id, keyboard.id, 3).await.unwrap();
    let err = repo
        .transfer(keyboard.id, lyon.id, paris.id, 5)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            InventoryError::InsufficientStock {
                available: 3,
                requested: 5
            }
        ),
        "got {err:?}"
    );
    assert_eq!(repo.stock_of(lyon.id, keyboard.id).await.unwrap(), 3);
    assert_eq!(repo.stock_of(paris.id, keyboard.id).await.unwrap(), 0);

    // Paris has never held any keyboards at all.
    let err = repo
        .transfer(keyboard.id, paris.id, lyon.id, 1)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            InventoryError::InsufficientStock {
                available: 0,
                requested: 1
            }
        ),
        "got {err:?}"
    );
}

#[tokio::test]
async fn transfer_is_all_or_nothing() {
    let repo = repo().await;
    let (keyboard, lyon, _) = seed(&repo).await;
    repo.add_stock(lyon.id, keyboard.id, 10).await.unwrap();
    // Warehouse 9999 doesn't exist, so the second half of the transfer fails.
    // The first half (taking 4 out of Lyon) must be rolled back with it.
    let err = repo
        .transfer(keyboard.id, lyon.id, 9999, 4)
        .await
        .unwrap_err();
    assert!(matches!(err, InventoryError::NotFound(_)), "got {err:?}");
    assert_eq!(
        repo.stock_of(lyon.id, keyboard.id).await.unwrap(),
        10,
        "stock vanished"
    );
}

#[tokio::test]
async fn concurrent_transfers_never_oversell() {
    let repo = repo().await;
    let (keyboard, lyon, paris) = seed(&repo).await;
    repo.add_stock(lyon.id, keyboard.id, 10).await.unwrap();
    // 25 transfers of 1 race each other; only 10 can succeed.
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..25 {
        let repo = repo.clone();
        tasks.spawn(async move { repo.transfer(keyboard.id, lyon.id, paris.id, 1).await });
    }
    let mut succeeded = 0;
    while let Some(result) = tasks.join_next().await {
        match result.unwrap() {
            Ok(()) => succeeded += 1,
            Err(InventoryError::InsufficientStock { .. }) => {}
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }
    assert_eq!(succeeded, 10);
    assert_eq!(repo.stock_of(lyon.id, keyboard.id).await.unwrap(), 0);
    assert_eq!(repo.stock_of(paris.id, keyboard.id).await.unwrap(), 10);
}

#[tokio::test]
async fn the_report_joins_names_and_skips_empty_shelves() {
    let repo = repo().await;
    let (keyboard, lyon, paris) = seed(&repo).await;
    let mouse = repo
        .create_product(&product("MS-02", "Mouse", 1999))
        .await
        .unwrap();
    repo.add_stock(paris.id, mouse.id, 2).await.unwrap();
    repo.add_stock(lyon.id, mouse.id, 8).await.unwrap();
    repo.add_stock(lyon.id, keyboard.id, 5).await.unwrap();
    repo.transfer(keyboard.id, lyon.id, paris.id, 5)
        .await
        .unwrap(); // Lyon: 0 keyboards

    let level = |warehouse: &str, sku: &str, quantity| StockLevel {
        warehouse: warehouse.to_string(),
        sku: sku.to_string(),
        quantity,
    };
    assert_eq!(
        repo.stock_report().await.unwrap(),
        [
            level("Lyon", "MS-02", 8),
            level("Paris", "KB-01", 5),
            level("Paris", "MS-02", 2)
        ]
    );
}

#[tokio::test]
async fn the_schema_itself_forbids_negative_stock() {
    let repo = repo().await;
    let (keyboard, lyon, _) = seed(&repo).await;
    repo.add_stock(lyon.id, keyboard.id, 1).await.unwrap();
    // Even a buggy query (or someone in the sqlite3 shell) can't store a negative quantity.
    let err = sqlx::query("UPDATE stock SET quantity = -1")
        .execute(repo.pool())
        .await
        .unwrap_err();
    assert!(
        matches!(err, sqlx::Error::Database(ref e) if e.is_check_violation()),
        "expected a CHECK constraint violation, got {err:?}"
    );
}

#[tokio::test]
async fn data_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let url = format!("sqlite://{}", dir.path().join("inventory.db").display());

    let pool = db::connect(&url).await.unwrap();
    let repo = SqliteInventory::new(pool.clone());
    let (keyboard, lyon, _) = seed(&repo).await;
    repo.add_stock(lyon.id, keyboard.id, 42).await.unwrap();
    pool.close().await;

    // Connecting again runs the migrations again: already-applied ones must be skipped.
    let repo = SqliteInventory::new(db::connect(&url).await.unwrap());
    assert_eq!(repo.stock_of(lyon.id, keyboard.id).await.unwrap(), 42);
    assert_eq!(repo.product_by_sku("KB-01").await.unwrap(), Some(keyboard));
}

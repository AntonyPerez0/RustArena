# 09 · Persistence with SQLx and SQLite

Until now your programs forgot everything when they stopped. Real services keep their data in a database, and the code that talks to it is some of the most important code a team owns: a bug there loses or corrupts data, and that can't be fixed by redeploying.

You'll finish a small warehouse **inventory**: products, warehouses, and how much stock each warehouse holds, stored in SQLite with **SQLx**. The schema, the error type and the repository trait are written; you write a migration and the SQL behind the trait.

**You'll practice:** migrations, parameterized queries, mapping rows to structs, transactions, translating database errors into domain errors, and the repository pattern.

## Background

### SQLite and SQLx

**SQLite** is a whole SQL database in a library: the data is one file, there's no server to run, and it's the most deployed database in the world (every phone has dozens). It's a great fit for tools, small services and tests. **SQLx** is the most popular async SQL toolkit for Rust. It speaks SQLite, PostgreSQL and MySQL with the same API, so what you learn here carries over to a Postgres job.

SQLx can check queries at compile time with `sqlx::query!`, but that needs a live database (or saved query data) while compiling. This project uses the **runtime-checked** functions instead, which compile anywhere, including CI:

```rust
let product = sqlx::query_as::<_, Product>(
    "SELECT id, sku, name, price_cents FROM products WHERE sku = ?",
)
.bind(sku)                       // fills the ?: never format! values into SQL
.fetch_optional(&self.pool)      // Option<Product>; also fetch_one, fetch_all
.await?;
```

- `query` runs a statement (`.execute()` returns how many rows changed), `query_as::<_, T>` builds a `T` that derives `FromRow`, and `query_scalar` returns a single column (`COUNT(*)`, a quantity...).
- `.bind()` sends values separately from the SQL text. Building SQL with `format!("... WHERE sku = '{sku}'")` is how SQL injection happens; binding makes it impossible.
- SQLite supports `INSERT ... RETURNING id, ...`, so an insert can hand back the stored row in one round trip.

### Migrations

The schema changes over a project's life, and every copy of the database (your laptop, CI, production) has to follow the same steps. A **migration** is one numbered SQL file. `sqlx::migrate!("./migrations")` embeds them all into the program, and `MIGRATOR.run(&pool)` (in `src/db.rs`) applies those the database hasn't seen yet, recording each one with a checksum in the `_sqlx_migrations` table.

Two rules teams live by:

1. **Never edit a migration that has shipped.** Databases that already applied it won't run it again, and SQLx refuses to start when a checksum changed. To change the schema, add a new file.
2. **Let the database enforce the rules it can.** `NOT NULL`, `UNIQUE`, `REFERENCES` (foreign keys) and `CHECK` constraints protect the data even from buggy code or someone typing in the `sqlite3` shell.

`build.rs` tells Cargo to recompile when `migrations/` changes, because the files are read at compile time.

### Turning database errors into domain errors

When an insert breaks a `UNIQUE` constraint, SQLx returns `sqlx::Error::Database(e)` where `e.is_unique_violation()` is true (there's also `is_foreign_key_violation()` and `is_check_violation()`). Callers shouldn't have to know that, let alone parse error messages. The repository translates it into `InventoryError::DuplicateSku(sku)`, which a web handler can turn into HTTP 409 without knowing SQL exists.

Notice that the database does the checking: "look first, then insert" has a race (two requests can both look, see nothing, and both insert). The constraint can't be raced.

### Transactions

Moving stock is two writes: take from one warehouse, add to another. If the program crashes or the second write fails between them, stock disappears. A **transaction** makes them one unit:

```rust
let mut tx = self.pool.begin().await?;
sqlx::query("UPDATE ...").bind(...).execute(&mut *tx).await?;   // note &mut *tx
sqlx::query("INSERT ...").bind(...).execute(&mut *tx).await?;
tx.commit().await?;   // dropping tx without commit (an early return, a `?`) rolls back
```

Checking stock and then updating it in two statements has the same race as above. Do the check inside the update: `UPDATE stock SET quantity = quantity - ? WHERE ... AND quantity >= ?`, then look at `rows_affected()`.

### The repository pattern

`src/repo.rs` defines `InventoryRepo`, a trait listing everything the program may do with stored inventory. The rest of the code depends on the trait, never on SQL. That keeps the SQL in one file, lets you add a PostgreSQL implementation later, and lets you test business logic with a fake. The provided tests call it through `&impl InventoryRepo`.

### One gotcha: in-memory databases

The tests use `sqlite::memory:`, a fresh database per test that never touches the disk. But each connection to `:memory:` gets its **own** empty database, so `db::connect` limits the pool to one connection that's never closed. Read `src/db.rs` to see how.

## Your tasks

1. Read `src/models.rs`, `src/error.rs`, `src/repo.rs` and `tests/inventory.rs`. The tests are the spec.
2. Write `migrations/0002_create_warehouses_and_stock.sql`:
   - `warehouses`: an integer primary key `id` and a unique, non-null `name`.
   - `stock`: `warehouse_id` and `product_id` referencing the other tables, a `quantity` that can never be negative (`CHECK`), and the pair `(warehouse_id, product_id)` as the primary key.
3. In `src/sqlite.rs`, implement the products and warehouses methods. Map unique violations to `DuplicateSku` and `DuplicateWarehouse`.
4. Implement `add_stock` as an upsert: `INSERT ... ON CONFLICT (warehouse_id, product_id) DO UPDATE SET quantity = quantity + excluded.quantity RETURNING quantity`. A foreign key violation means the product or warehouse doesn't exist: `NotFound`.
5. Implement `transfer` in one transaction, and `stock_report` as a `JOIN` (`SELECT w.name AS warehouse, ...` so the column names match `StockLevel`).
6. Try the CLI, then look inside the file yourself:

```bash
cargo test
cargo run -- add-product KB-01 "Mechanical keyboard" 4999
cargo run -- add-warehouse Lyon
cargo run -- receive Lyon KB-01 25
cargo run -- report
sqlite3 inventory.db '.schema stock' 'SELECT * FROM _sqlx_migrations;'
bash ../../tools/grade.sh 09-database
```

`*.db` files are in `.gitignore`: databases don't belong in Git.

## What the grader checks

- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean.
- `tests/inventory.rs` and `migrations/0001_create_products.sql` are unchanged, and you added a migration.
- Every test passes.
- The CLI, run against a fresh database file, stores and reports the right stock, and the `sqlite3` shell finds it in the file.

## Hints

- `transfer_is_all_or_nothing` sends stock to a warehouse that doesn't exist. Without a transaction, the stock leaves Lyon and never arrives.
- When the conditional `UPDATE` changes no rows, query the available quantity (still inside the transaction) to build `InsufficientStock { available, requested }`. A missing row means 0.
- A helper `fn is_unique_violation(err: &sqlx::Error) -> bool` keeps the `map_err` closures short.
- If a migration fails to apply, the test output shows SQLite's message. Try the SQL in `sqlite3 :memory:` first.

## Going further

- Install `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features sqlite`) and use `sqlx migrate add` and `sqlx migrate run`.
- Add a `place_order` method that takes several lines (SKU and quantity) and reserves all of them or none.
- Write an `InMemoryInventory` that implements the trait with a `Mutex<HashMap>`, and run the same tests against both.
- Port it to PostgreSQL with `sqlx::PgPool`: what changes in the SQL (`$1` placeholders, `SERIAL`/`IDENTITY`)?
- Try `sqlx::query!` with offline mode (`cargo sqlx prepare`) and compare the experience.

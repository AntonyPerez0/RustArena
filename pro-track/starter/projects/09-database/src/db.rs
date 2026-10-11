//! Opening the database. Provided: read it, you don't need to change it.

use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

/// Every file in `migrations/`, embedded into the program at compile time.
/// Running it applies the migrations a database hasn't seen yet, in order, and records
/// them in the `_sqlx_migrations` table.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Connects to `url` (for example `sqlite:inventory.db` or `sqlite::memory:`),
/// creates the database file if needed, and brings the schema up to date.
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    // SQLite ignores foreign keys unless each connection switches them on.
    // sqlx does that by default; saying it here documents that we rely on it.
    let mut options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let in_memory = url.contains(":memory:") || url.contains("mode=memory");
    let pool = if in_memory {
        // An in-memory database belongs to ONE connection: a second connection would
        // open a different, empty database. So the pool keeps exactly one connection
        // and never closes it.
        SqlitePoolOptions::new()
            .max_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options)
            .await?
    } else {
        // Write-ahead logging lets readers keep reading while a write is in progress.
        options = options.journal_mode(SqliteJournalMode::Wal);
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?
    };

    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

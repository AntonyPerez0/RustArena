//! Opening the database: the same code as project 09. Provided.

use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

/// Every file in `migrations/`, embedded into the program at compile time.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Connects to `url` (for example `sqlite:shortener.db` or `sqlite::memory:`), creates the
/// database file if needed, and brings the schema up to date.
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let mut options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let in_memory = url.contains(":memory:") || url.contains("mode=memory");
    let pool = if in_memory {
        // An in-memory database belongs to ONE connection, so the pool keeps exactly one
        // connection and never closes it.
        SqlitePoolOptions::new()
            .max_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options)
            .await?
    } else {
        options = options.journal_mode(SqliteJournalMode::Wal);
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?
    };

    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

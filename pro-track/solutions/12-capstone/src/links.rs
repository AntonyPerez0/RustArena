//! Links: validation rules, code generation, and the SQL.

use rand::Rng;
use rand::distr::Alphanumeric;
use sqlx::{FromRow, SqlitePool};

use crate::error::AppError;

/// Length of generated codes. 62^7 is about 3.5 trillion possibilities.
pub const CODE_LEN: usize = 7;
const MAX_URL_LEN: usize = 2048;
/// Codes that would clash with the API's own paths.
const RESERVED: &[&str] = &["api", "health"];

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Link {
    pub code: String,
    pub url: String,
    pub hits: i64,
    pub created_at: String,
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::Invalid(message.into())
}

/// Checks that `input` is an absolute http(s) URL with a host, and returns it normalized.
pub fn validate_url(input: &str) -> Result<String, AppError> {
    let input = input.trim();
    if input.len() > MAX_URL_LEN {
        return Err(invalid(format!(
            "url is longer than {MAX_URL_LEN} characters"
        )));
    }
    let url = url::Url::parse(input).map_err(|err| invalid(format!("url is not valid: {err}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(invalid("url must start with http:// or https://"));
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(invalid("url must have a host"));
    }
    Ok(url.into())
}

/// Custom codes: 4 to 32 ASCII letters, digits, `-` or `_`, and not a reserved word.
pub fn validate_code(code: &str) -> Result<(), AppError> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    if !(4..=32).contains(&code.len()) || !code.chars().all(allowed) {
        return Err(invalid("code must be 4 to 32 letters, digits, '-' or '_'"));
    }
    if RESERVED.contains(&code.to_ascii_lowercase().as_str()) {
        return Err(invalid(format!("code {code:?} is reserved")));
    }
    Ok(())
}

/// A random code of `CODE_LEN` letters and digits.
pub fn random_code() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(CODE_LEN)
        .map(char::from)
        .collect()
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.is_unique_violation())
}

/// Stores a new link. A code that's taken is `AppError::CodeTaken`.
pub async fn insert(pool: &SqlitePool, code: &str, url: &str) -> Result<Link, AppError> {
    sqlx::query_as::<_, Link>(
        "INSERT INTO links (code, url) VALUES (?, ?) RETURNING code, url, hits, created_at",
    )
    .bind(code)
    .bind(url)
    .fetch_one(pool)
    .await
    .map_err(|err| {
        if is_unique_violation(&err) {
            AppError::CodeTaken(code.to_string())
        } else {
            err.into()
        }
    })
}

/// Stores a new link under a fresh random code, trying again on the (rare) collision.
pub async fn insert_with_random_code(pool: &SqlitePool, url: &str) -> Result<Link, AppError> {
    let mut attempts = 0;
    loop {
        match insert(pool, &random_code(), url).await {
            Err(AppError::CodeTaken(_)) if attempts < 5 => attempts += 1,
            other => return other,
        }
    }
}

pub async fn find(pool: &SqlitePool, code: &str) -> Result<Option<Link>, AppError> {
    let link =
        sqlx::query_as::<_, Link>("SELECT code, url, hits, created_at FROM links WHERE code = ?")
            .bind(code)
            .fetch_optional(pool)
            .await?;
    Ok(link)
}

/// Counts a visit and returns where to send the visitor, in one statement.
pub async fn record_hit(pool: &SqlitePool, code: &str) -> Result<Option<String>, AppError> {
    let url = sqlx::query_scalar("UPDATE links SET hits = hits + 1 WHERE code = ? RETURNING url")
        .bind(code)
        .fetch_optional(pool)
        .await?;
    Ok(url)
}

/// Deletes a link; `false` if there was none.
pub async fn delete(pool: &SqlitePool, code: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM links WHERE code = ?")
        .bind(code)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

//! Links: validation rules, code generation, and the SQL.
//!
//! TODO:
//! - a `Link` struct deriving `sqlx::FromRow`
//! - `validate_url` (absolute http or https URL with a host, at most 2048 characters) and
//!   `validate_code` (4 to 32 ASCII letters, digits, '-' or '_'; not "api" or "health")
//! - `random_code`: 7 random letters and digits (`rand::distr::Alphanumeric`)
//! - insert (a unique violation means the code is taken), find, record_hit, delete

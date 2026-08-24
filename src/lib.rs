#![warn(missing_docs)]
//! # Agrum
//!
//! Agrum is a database access layers designed to let developpers:
//!  1. to write the SQL they want so they can directly populate their aggregates from the database
//!  2. to focus on the business value of the queries
//!  3. to test the queries they write
//!
//! This library is still in early development stage, means it is **not production
//! ready**. If you are looking for a mature solution, have a look at
//! [Elephantry](https://elephantry.github.io/)

mod condition;
mod connection;
mod projection;
mod query;
mod query_book;
mod structure;

/// Derive [`SqlEntity`] and [`Structured`] from a struct's fields.
///
/// Field names are used as column names. SQL types are inferred from the Rust
/// field types (`Uuid` → `uuid`, `String` → `text`, and so on).
pub use agrum_derive::SqlEntity;
pub use condition::*;
pub use connection::*;
pub use projection::*;
pub use query::*;
pub use query_book::*;
pub use structure::*;

// Re-exports used by `#[derive(SqlEntity)]` so generated code can name
// `tokio_postgres::Row` via `::agrum` without a direct tokio-postgres dependency.
// It is kept out of rustdoc; it is not meant for people to import.
#[doc(hidden)]
#[allow(missing_docs)]
pub mod __private {
    pub use tokio_postgres::Row;
}

type Result<T> = anyhow::Result<T>;

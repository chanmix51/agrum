use std::{error::Error, fmt::Display};

use postgres::Row;

use crate::Projection;

/// Error raised during entity hydration process.
#[derive(Debug)]
pub enum HydrationError {
    /// Data could not be parsed or cast in the expected structure.
    InvalidData(String),

    /// Error while fetching data from the database.
    FetchFailed {
        error: postgres::Error,
        field_index: usize,
    },
}

impl Display for HydrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData(msg) => write!(f, "Invalid data error: «{}»", msg),
            Self::FetchFailed { error, field_index } => write!(
                f,
                "Fail to fetch data for field index {}, message: «{}».",
                field_index, error
            ),
        }
    }
}

impl Error for HydrationError {}

/// Database entity, this trait defined how entities are hydrated from database
/// data.
pub trait Entity {
    /// create a new Entity from database data in a result row.
    fn hydrate(row: Row) -> Result<Self, HydrationError>
    where
        Self: Sized;

    /// Create an instance of the [Projection] required to fetch this Entity.
    fn make_projection(&self) -> Projection;
}

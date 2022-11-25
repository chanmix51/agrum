use std::{error::Error, fmt::Display};

use postgres::Row;

use crate::Structure;

/// Error raised during entity hydration process.
#[derive(Debug)]
pub enum HydrationError {
    /// Data could not be parsed or cast in the expected structure.
    InvalidData(String),

    /// Error while fetching data from the database.
    FieldFetchFailed {
        error: postgres::Error,
        field_index: usize,
    },

    /// Error while fetching the Row from the database.
    RowFetchFailed(postgres::Error),
}

impl Display for HydrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData(msg) => write!(f, "Invalid data error: «{}»", msg),
            Self::FieldFetchFailed { error, field_index } => write!(
                f,
                "Fail to fetch data for field index {}, message: «{}».",
                field_index, error
            ),
            Self::RowFetchFailed(e) => write!(f, "Fail to fetch the row, message «{}».", e),
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

    /// Create an instance of the [Structure] required to fetch this Entity.
    fn get_structure() -> Structure;
}

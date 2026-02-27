use std::error::Error;
use std::fmt::Display;

use tokio_postgres::{Row, error::Error as PgError};

use crate::Projection;

/// SQL field structure.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructureField {
    /// Name of the field.
    name: String,

    /// SQL type of the field.
    sql_type: String,
}

impl StructureField {
    /// Create a new structure field.
    pub fn new(name: &str, sql_type: &str) -> Self {
        Self {
            name: name.to_string(),
            sql_type: sql_type.to_string(),
        }
    }

    /// Dump the structure field as a tuple of name and SQL type.
    pub fn dump(&self) -> (&str, &str) {
        (&self.name, &self.sql_type)
    }
}
/// Structure of a SQL tuple.
#[derive(Debug, Clone, Default)]
pub struct Structure {
    fields: Vec<StructureField>,
}

impl Structure {
    /// Create a new instance of Structure from a slice of tuples.
    pub fn new(field_definitions: &[(&str, &str)]) -> Self {
        let mut fields: Vec<StructureField> = Vec::new();

        for (name, sql_type) in field_definitions {
            fields.push(StructureField::new(name, sql_type));
        }

        Self { fields }
    }

    /// Set a field in the structure.
    pub fn set_field(&mut self, name: &str, sql_type: &str) -> &mut Self {
        let name = name.to_string();
        let sql_type = sql_type.to_string();

        let definition = StructureField { name, sql_type };
        self.fields.push(definition);

        self
    }

    /// Get the fields of the structure.
    pub fn get_fields(&self) -> &Vec<StructureField> {
        &self.fields
    }

    /// Get the names of the fields in the structure.
    pub fn get_names(&self) -> Vec<&str> {
        let names: Vec<&str> = self.fields.iter().map(|f| f.name.as_str()).collect();

        names
    }
}

/// A trait to mark types that are structured.
/// A structured type is a type that has a structure.
/// The structure is a list of fields with their names and SQL types.
pub trait Structured {
    /// Get the structure of the type.
    fn get_structure() -> Structure;
}

/// Error raised during entity hydration process.
#[derive(Debug)]
pub enum HydrationError {
    /// Data could not be parsed or cast in the expected structure.
    InvalidData(String),

    /// Error while fetching data from the database.
    FieldFetchFailed {
        /// Error while fetching the data.
        error: PgError,
        /// Index of the field that failed to fetch.
        field_index: usize,
    },

    /// Error while fetching the Row from the database.
    RowFetchFailed(PgError),
}

impl Display for HydrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData(msg) => write!(f, "Invalid data error: «{msg}»"),
            Self::FieldFetchFailed { error, field_index } => write!(
                f,
                "Fail to fetch data for field index {field_index}, message: «{error}»."
            ),
            Self::RowFetchFailed(e) => write!(f, "Fail to fetch the row, message «{e}»."),
        }
    }
}

impl Error for HydrationError {}

/// A trait to mark types that are SQL entities.
/// An SQL entity is a type that has a structure, a projection and a hydration function.
pub trait SqlEntity: Structured + Sized {
    /// Get the projection of the entity.
    fn get_projection() -> Projection<Self>;

    /// Hydrate the entity from a row.
    fn hydrate(row: &Row) -> Result<Self, HydrationError>;
}

#[cfg(test)]
mod tests {

    use super::*;

    fn get_structure() -> Structure {
        Structure::new(&[("a_field", "a_type"), ("another_field", "another_type")])
    }

    #[test]
    fn use_structure() {
        let structure = get_structure();

        assert_eq!(
            &[
                StructureField {
                    name: "a_field".to_string(),
                    sql_type: "a_type".to_string()
                },
                StructureField {
                    name: "another_field".to_string(),
                    sql_type: "another_type".to_string()
                }
            ]
            .to_vec(),
            structure.get_fields()
        );
    }

    #[test]
    fn get_names() {
        let structure = get_structure();
        assert_eq!(vec!["a_field", "another_field"], structure.get_names());
    }
}

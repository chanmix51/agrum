use crate::structure::Structure;

/// This represent a SQL data source. It can be a table, a SQL function, a query
/// etc.
pub trait Source {
    /// Return the textual definition of the SQL source by example the fully
    /// qualified name of a table, a SQL query etc.
    fn get_definition(&self) -> String;

    /// Return the structure of the tuple provided by the source.
    fn get_structure(&self) -> Structure;
}

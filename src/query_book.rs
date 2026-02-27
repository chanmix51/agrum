use std::{collections::HashMap, iter::repeat_n};

use crate::{SqlEntity, SqlQuery, ToSqlAny, WhereCondition};

/// A trait to mark types that are query books.
/// Query books are responsible of building the queries that will be sent to the
/// database server. This is the place where SQL templates are defined and
/// populated with the parameters. By essence, a QueryBook takes its data from a SQL source
/// and defines the SQL computation that will return the data.
///
/// The example shown therafter could be brought by the `ReadQueryBook` trait
/// and is presented here for the sake of example.
///
/// # Examples
/// ```rust
/// use std::marker::PhantomData;
/// use uuid::Uuid;
/// use agrum::{QueryBook, SqlEntity, SqlQuery, ToSqlAny, WhereCondition};
///
/// #[derive(Default)]
/// struct CompanyQueryBook<T: SqlEntity> {
///     _phantom: PhantomData<T>,
/// }
///
/// impl<T: SqlEntity> QueryBook<T> for CompanyQueryBook<T> {
///     fn get_sql_source(&self) -> &'static str {
///         "some_schema.company"
///     }
/// }
///
/// impl<T: SqlEntity> CompanyQueryBook<T> {
///     fn get_by_id<'a>(&self, company_id: &'a Uuid) -> SqlQuery<'a, T> {
///         self.select(WhereCondition::new("company_id = $?", vec![company_id]))
///     }
///
///     fn get_sql_definition(&self) -> &'static str {
///         "select {:projection:} from {:source:} where {:condition:}"
///     }
///
///     fn select<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T> {
///         let mut query = SqlQuery::new(self.get_sql_definition());
///         let (conditions, parameters) = conditions.expand();
///         query
///             .set_variable("projection", &T::get_projection().to_string())
///             .set_variable("source", self.get_sql_source())
///             .set_variable("condition", &conditions.to_string())
///             .set_parameters(parameters);
///         query
///     }
/// }
/// ```
pub trait QueryBook<T: SqlEntity> {
    /// Return the definition of the SQL data source.
    /// It could be a table name or a view name or a values list or function or
    /// even a sub-query.
    fn get_sql_source(&self) -> &'static str;
}

/// A trait that marks QueryBooks that perform simple `select {:projection:}
/// from {:source:} where {:condition:}` queries.
pub trait ReadQueryBook<T: SqlEntity>: QueryBook<T> {
    /// Return the definition of the SQL query.
    /// It could be a select statement or an insert statement or a update statement or a delete statement.
    fn get_sql_definition(&self) -> &'static str {
        "select {:projection:} from {:source:} where {:condition:}"
    }

    /// Create a new select query with the given conditions.
    /// The query will be built using the definition returned by the `get_sql_definition` method.
    /// The projection will be the projection of the entity returned by the `get_projection` method.
    /// The source will be the source returned by the `get_sql_source` method.
    fn select<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T> {
        let mut query = SqlQuery::new(self.get_sql_definition());
        let (conditions, parameters) = conditions.expand();
        query
            .set_variable("projection", &T::get_projection().to_string())
            .set_variable("source", self.get_sql_source())
            .set_variable("condition", &conditions.to_string())
            .set_parameters(parameters);

        query
    }
}

/// A trait that marks QueryBooks that perform `delete from {:source:} where
/// {:condition:} returning {:projection:}` queries.  These queries actually
/// return the deleted entities.
pub trait DeleteQueryBook<T: SqlEntity>: QueryBook<T> {
    /// Definition of the delete query.
    fn get_sql_definition(&self) -> &'static str {
        "delete from {:source:} where {:condition:} returning {:projection:}"
    }

    /// Create a new delete query with the given conditions.
    /// The query will be built using the definition returned by the `get_sql_definition` method.
    /// The source will be the source returned by the `get_sql_source` method.
    /// The conditions will be the conditions passed to the method.
    /// The projection will be the projection of the entity returned by the `get_projection` method.
    fn delete<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T> {
        let mut query = SqlQuery::new(self.get_sql_definition());
        let (conditions, parameters) = conditions.expand();
        query
            .set_variable("source", self.get_sql_source())
            .set_variable("condition", &conditions.to_string())
            .set_variable("projection", &T::get_projection().to_string())
            .set_parameters(parameters);
        query
    }
}

/// A trait that marks QueryBooks that perform simple aSQL update queries
/// These queries actually return the updated entities.
pub trait UpdateQueryBook<T: SqlEntity>: QueryBook<T> {
    /// Definition of the delete query.
    fn get_sql_definition(&self) -> &'static str {
        "update {:source:} set {:updates:} where {:condition:} returning {:projection:}"
    }

    /// Create a new update query with the given updates and conditions.
    /// The query will be built using the definition returned by the `get_sql_definition` method.
    /// The source will be the source returned by the `get_sql_source` method.
    /// The updates will be the updates passed to the method.
    /// The conditions will be the conditions passed to the method.
    /// The projection will be the projection of the entity returned by the `get_projection` method.
    fn update<'a>(
        &self,
        updates: HashMap<&'a str, &'a dyn ToSqlAny>,
        conditions: WhereCondition<'a>,
    ) -> SqlQuery<'a, T> {
        let (condition_sql, condition_params) = conditions.expand();
        let mut updates_fragments = Vec::with_capacity(updates.len());
        let mut params: Vec<&'a dyn ToSqlAny> =
            Vec::with_capacity(updates.len() + condition_params.len());

        for (column, value) in updates {
            updates_fragments.push(format!("{column} = $?"));
            params.push(value);
        }
        let updates_sql = updates_fragments.join(", ");

        let mut query = SqlQuery::new(self.get_sql_definition());
        query
            .set_variable("source", self.get_sql_source())
            .set_variable("updates", &updates_sql)
            .set_variable("condition", &condition_sql)
            .set_variable("projection", &T::get_projection().to_string())
            .set_parameters(params)
            .append_parameters(condition_params);

        query
    }
}

/// A trait that marks QueryBooks that perform SQL insert queries.
/// These queries actually return the inserted entities.
pub trait InsertQueryBook<T: SqlEntity>: QueryBook<T> {
    /// Definition of the insert query.
    fn get_sql_definition(&self) -> &'static str {
        "insert into {:source:} ({:structure:}) values ({:values:}) returning {:projection:}"
    }

    /// Create a new insert query with the given values.
    /// The query will be built using the definition returned by the `get_sql_definition` method.
    /// The source will be the source returned by the `get_sql_source` method.
    /// The structure will be the structure of the entity returned by the `get_structure` method.
    /// The values will be the values passed to the method.
    /// The projection will be the projection of the entity returned by the `get_projection` method.
    fn insert<'a>(&self, values: HashMap<&'a str, &'a dyn ToSqlAny>) -> SqlQuery<'a, T> {
        // Build column list and parameter list following the entity structure
        let structure = <T as crate::Structured>::get_structure();

        let mut columns: Vec<&str> = Vec::new();
        let mut params: Vec<&'a dyn ToSqlAny> = Vec::new();

        for name in structure.get_names() {
            if let Some(value) = values.get(name) {
                columns.push(name);
                params.push(*value);
            }
        }

        let columns_sql = columns.join(", ");
        let values_sql = repeat_n("$?", columns.len()).collect::<Vec<_>>().join(", ");

        let mut query = SqlQuery::new(self.get_sql_definition());
        query
            .set_variable("source", self.get_sql_source())
            .set_variable("structure", &columns_sql)
            .set_variable("values", &values_sql)
            .set_variable("projection", &T::get_projection().to_string())
            .set_parameters(params);

        query
    }
}

#[cfg(test)]
mod tests {
    use std::{any::Any, collections::HashMap, marker::PhantomData};

    use crate::{Projection, Structure, Structured};

    use super::*;

    struct Entity {
        _id: u32,
        _name: String,
        _score: i32,
        _is_active: bool,
    }

    impl SqlEntity for Entity {
        fn get_projection() -> Projection<Self> {
            Projection::new("entity_table")
        }

        fn hydrate(row: &tokio_postgres::Row) -> Result<Self, crate::HydrationError> {
            Ok(Entity {
                _id: row.get("id"),
                _name: row.get("name"),
                _score: row.get("score"),
                _is_active: row.get("is_active"),
            })
        }
    }
    impl Structured for Entity {
        fn get_structure() -> Structure {
            Structure::new(&[
                ("id", "integer"),
                ("name", "text"),
                ("score", "integer"),
                ("is_active", "bool"),
            ])
        }
    }

    struct EntityQueryBook {
        _phantom: PhantomData<Entity>,
    }

    impl Default for EntityQueryBook {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }
    impl QueryBook<Entity> for EntityQueryBook {
        fn get_sql_source(&self) -> &'static str {
            "some_schema.entity_table"
        }
    }

    impl ReadQueryBook<Entity> for EntityQueryBook {}

    impl UpdateQueryBook<Entity> for EntityQueryBook {}

    impl DeleteQueryBook<Entity> for EntityQueryBook {}

    impl InsertQueryBook<Entity> for EntityQueryBook {}

    #[test]
    fn test_select() {
        let query = EntityQueryBook::default().select(WhereCondition::new("id = $?", vec![&1_u32]));
        assert_eq!(
            query.to_string(),
            "select entity_table.id as id, entity_table.name as name, entity_table.score as score, entity_table.is_active as is_active from some_schema.entity_table where id = $1"
        );
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 1);
        let parameter: &u32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_u32);
    }

    #[test]
    fn test_update() {
        let updates = HashMap::from([("name", &"test_name" as &dyn ToSqlAny)]);
        let query = EntityQueryBook::default()
            .update(updates, WhereCondition::new("id = $?", vec![&1_u32]));
        assert_eq!(
            query.to_string(),
            "update some_schema.entity_table set name = $1 where id = $2 returning entity_table.id as id, entity_table.name as name, entity_table.score as score, entity_table.is_active as is_active"
        );
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 2);
        let parameter: &&str = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &"test_name");
        let parameter: &u32 = (parameters[1] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_u32);
    }

    #[test]
    fn test_delete() {
        let query = EntityQueryBook::default().delete(WhereCondition::new("id = $?", vec![&1_u32]));
        assert_eq!(
            query.to_string(),
            "delete from some_schema.entity_table where id = $1 returning entity_table.id as id, entity_table.name as name, entity_table.score as score, entity_table.is_active as is_active"
        );
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 1);
        let parameter: &u32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_u32);
    }

    #[test]
    fn test_insert() {
        let query = EntityQueryBook::default().insert(HashMap::from([
            ("name", &"test_name" as &dyn ToSqlAny),
            ("score", &42_i32 as &dyn ToSqlAny),
            ("is_active", &true as &dyn ToSqlAny),
        ]));
        assert_eq!(
            query.to_string(),
            "insert into some_schema.entity_table (name, score, is_active) values ($1, $2, $3) returning entity_table.id as id, entity_table.name as name, entity_table.score as score, entity_table.is_active as is_active"
        );
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 3);

        let parameter: &&str = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &"test_name");

        let parameter: &i32 = (parameters[1] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &42_i32);

        let parameter: &bool = (parameters[2] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &true);
    }
}

use std::{collections::HashMap, fmt::Display, marker::PhantomData};

use crate::{SqlEntity, ToSqlAny, WhereCondition};

pub struct SqlQuery<'a, T: SqlEntity> {
    query: String,
    parameters: Vec<&'a dyn ToSqlAny>,
    variables: HashMap<&'a str, String>,
    _phantom: PhantomData<T>,
}

impl<'a, T: SqlEntity> SqlQuery<'a, T> {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            parameters: Vec::new(),
            variables: [("projection", T::get_projection().expand())].into(),
            _phantom: PhantomData,
        }
    }

    /// Set a variable in the query. This variable will be replaced by its value
    /// in the query.
    pub fn set_variable(&mut self, name: &'a str, value: &str) -> &mut Self {
        self.variables.insert(name, value.to_string());
        self
    }

    /// Add a parameter to the query. This parameter will be replaced by its
    /// value in the query. The parameter will be expanded in the `$?` placeholder.
    pub fn add_parameter(&mut self, parameter: &'a dyn ToSqlAny) -> &mut Self {
        self.parameters.push(parameter);
        self
    }

    /// Set the parameters of the query.
    pub fn set_parameters(&mut self, parameters: Vec<&'a dyn ToSqlAny>) -> &mut Self {
        self.parameters = parameters;
        self
    }

    /// Return the variables of the query.
    pub fn get_variables(&self) -> &HashMap<&'a str, String> {
        &self.variables
    }

    /// Return the parameters of the query. This method is mostly intended for
    /// testing purposes since the parameters are cloned.
    pub fn get_parameters(&self) -> Vec<&'a dyn ToSqlAny> {
        self.parameters.clone()
    }

    /// Return the query and the parameters to be sent to the server.
    /// This consumes the query instance.
    pub fn expand(self) -> (String, Vec<&'a dyn ToSqlAny>) {
        let query = self.to_string();
        let parameters = self.parameters;
        (query, parameters)
    }
}

impl<'a, T: SqlEntity> Display for SqlQuery<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut query = self.query.clone();
        for (name, value) in &self.variables {
            query = query.replace(&format!("{{:{name}:}}"), value);
        }
        write!(f, "{}", query)
    }
}

pub trait QueryBook<T: SqlEntity> {
    /// Return the definition of the SQL data source.
    /// It could be a table name or a view name or a values list or function or
    /// even a sub-query.
    fn get_sql_source(&self) -> &'static str;
}
pub trait ReadQueryBook<T: SqlEntity>: QueryBook<T> {
    /// Return the definition of the SQL query.
    /// It could be a select statement or an insert statement or a update statement or a delete statement.
    fn get_sql_definition(&self) -> &'static str {
        "select {:projection:} from {:source:} where {:condition:}"
    }

    fn select<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T> {
        let mut query = SqlQuery::new(self.get_sql_definition());
        let (conditions, parameters) = conditions.expand();
        query
            .set_variable("projection", &T::get_projection().expand())
            .set_variable("source", self.get_sql_source())
            .set_variable("condition", &conditions.to_string())
            .set_parameters(parameters);

        query
    }
}
#[cfg(test)]
mod tests {
    use std::any::Any;
    use tokio_postgres::Row;

    use crate::{HydrationError, Projection, SqlEntity, Structure, Structured};

    use super::*;

    #[allow(unused)]
    struct TestSqlEntity {
        id: i64,
        name: String,
    }

    impl SqlEntity for TestSqlEntity {
        fn get_projection() -> Projection<TestSqlEntity> {
            Projection::<TestSqlEntity>::default()
        }

        fn hydrate(row: &Row) -> Result<Self, HydrationError> {
            Ok(TestSqlEntity {
                id: row.get("id"),
                name: row.get("name"),
            })
        }
    }

    impl Structured for TestSqlEntity {
        fn get_structure() -> Structure {
            Structure::new(&[("id", "int"), ("name", "text")])
        }
    }

    #[test]
    fn test_to_string() {
        let query = {
            let mut query = SqlQuery::<TestSqlEntity>::new(
                "SELECT {:projection:} FROM my_table WHERE {:condition:}",
            );
            query
                .set_variable("projection", &TestSqlEntity::get_projection().expand())
                .set_variable("condition", "1 = $1")
                .add_parameter(&1_i32);
            query
        };
        let parameters = query.get_parameters();
        let parameter: &i32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        let variables = query.get_variables();
        assert_eq!(variables["condition"], "1 = $1");
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameter, &1_i32);
        assert_eq!(variables.len(), 2);
        assert_eq!(variables["projection"], "id as id, name as name");
        let result = query.to_string();
        assert_eq!(
            &result,
            "SELECT id as id, name as name FROM my_table WHERE 1 = $1"
        );
    }
}

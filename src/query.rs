use std::{collections::HashMap, fmt::Display, marker::PhantomData};

use crate::{SqlEntity, ToSqlAny};

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
            variables: [("projection", T::get_projection().to_string())].into(),
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

    /// Append a vec of parameters to the query.
    pub fn append_parameters(&mut self, parameters: Vec<&'a dyn ToSqlAny>) -> &mut Self {
        self.parameters.extend(parameters);
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
        let mut param_index = 1;
        //
        // Replace parameters placeholders by numerated parameters.
        loop {
            if !query.contains("$?") {
                break;
            }
            query = query.replacen("$?", &format!("${param_index}"), 1);
            param_index += 1;
        }

        write!(f, "{}", query)
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use tokio_postgres::Row;

    use crate::{HydrationError, Projection, SqlEntity, Structure, Structured, params};

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
    fn test_set_variable() {
        let mut query = SqlQuery::<TestSqlEntity>::new("one: {:one:}; two: {:two:}");
        query.set_variable("one", "ein");
        query.set_variable("two", "zwei");
        let (query, _parameters) = query.expand();
        assert_eq!(query, "one: ein; two: zwei");
    }

    #[test]
    fn test_add_parameter() {
        let mut query = SqlQuery::<TestSqlEntity>::new("whatever");
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 0);

        query.add_parameter(&1_i32);
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 1);
        let parameter: &i32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_i32);

        query.add_parameter(&true);
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 2);
        let parameter: &i32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_i32);
        let parameter: &bool = (parameters[1] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &true);
    }

    #[test]
    fn test_append_parameters() {
        let mut query = SqlQuery::<TestSqlEntity>::new("whatever");
        query
            .add_parameter(&1_i32)
            .append_parameters(params![2_i32, true]);
        let parameters = query.get_parameters();
        println!("parameters: {:?}", parameters);
        assert_eq!(parameters.len(), 3);
        let parameter: &i32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_i32);
        let parameter: &i32 = (parameters[1] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &2_i32);
        let parameter: &bool = (parameters[2] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &true);
    }

    #[test]
    fn test_set_parameters() {
        let mut query = SqlQuery::<TestSqlEntity>::new("whatever");
        query.set_parameters(params![1_i32, 2_i32, true]);
        let parameters = query.get_parameters();
        assert_eq!(parameters.len(), 3);
        let parameter: &i32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_i32);
        let parameter: &i32 = (parameters[1] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &2_i32);
        let parameter: &bool = (parameters[2] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &true);
    }

    #[test]
    fn test_to_string() {
        let query = {
            let mut query = SqlQuery::<TestSqlEntity>::new(
                "projection: {:projection:} condition: {:condition:}",
            );
            query
                .set_variable("projection", &TestSqlEntity::get_projection().to_string())
                .set_variable("condition", "1 = $?")
                .set_parameters(params![1_i32]);
            query
        };
        let parameters = query.get_parameters();
        let parameter: &i32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        let variables = query.get_variables();
        assert_eq!(variables["condition"], "1 = $?");
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameter, &1_i32);
        assert_eq!(variables.len(), 2);
        assert_eq!(variables["projection"], "id as id, name as name");
        let result = query.to_string();
        assert_eq!(
            &result,
            "projection: id as id, name as name condition: 1 = $1"
        );
    }

    #[test]
    fn test_to_string_with_multiple_parameters() {
        let mut query = SqlQuery::<TestSqlEntity>::new("VALUES ($?, $?, $?)");
        query.set_parameters(params![1_i32, 2_i32, 3_i32]);
        let (query, parameters) = query.expand();
        assert_eq!(query, "VALUES ($1, $2, $3)");
        assert_eq!(parameters.len(), 3);
        let parameter: &i32 = (parameters[0] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &1_i32);
        let parameter: &i32 = (parameters[1] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &2_i32);
        let parameter: &i32 = (parameters[2] as &dyn Any).downcast_ref().unwrap();
        assert_eq!(parameter, &3_i32);
    }
}

use crate::{SqlEntity, SqlQuery, WhereCondition};

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
            .set_variable("projection", &T::get_projection().to_string())
            .set_variable("source", self.get_sql_source())
            .set_variable("condition", &conditions.to_string())
            .set_parameters(parameters);

        query
    }
}

pub trait DeleteQueryBook<T: SqlEntity>: QueryBook<T> {
    fn get_sql_definition(&self) -> &'static str {
        "delete from {:source:} where {:condition:} returning {:projection:}"
    }

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

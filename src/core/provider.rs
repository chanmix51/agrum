use std::error::Error;
use std::marker::PhantomData;

use tokio_postgres::Client as PgClient;

use super::{SqlEntity, WhereCondition};

/// Whatever that aims to be a database data source (query, table, function
/// etc.) This has to be the SQL definition as it will be interpreted by
/// Postgres.
pub trait SqlDefinition {
    /// SQL that is sent to Postgres (parameters shall be marked as `$?`)
    fn expand(&self, condition: &str) -> String;
}

/// A Provider is a structure that holds the connection and the pipeworks to
/// actually perform queries.
pub struct Provider<'client, T>
where
    T: SqlEntity + Send,
{
    pg_client: &'client PgClient,
    definition: Box<dyn SqlDefinition>,
    _phantom: PhantomData<T>,
}

/// A Provider uses an entity associated Projection to issue SQL queries and
/// return an iterator over results.
impl<'client, T> Provider<'client, T>
where
    T: SqlEntity + Send,
{
    /// constructor
    pub fn new(pg_client: &'client PgClient, definition: Box<dyn SqlDefinition>) -> Self {
        Self {
            pg_client,
            definition,
            _phantom: PhantomData,
        }
    }

    /// Return the SQL definition of this Provider.
    pub fn get_definition(&self) -> &dyn SqlDefinition {
        self.definition.as_ref()
    }

    /// Launch a SQL statement to fetch the associated entities.
    pub async fn find(
        &'client self,
        condition: WhereCondition<'_>,
    ) -> Result<Vec<T>, Box<dyn Error>> {
        let (expression, parameters) = condition.expand();
        let sql = self.definition.expand(&expression);
        let mut items: Vec<T> = Vec::new();

        println!("SQL = “{sql}”.");

        for row in self.pg_client.query(&sql, parameters.as_slice()).await? {
            items.push(T::hydrate(row)?);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {}

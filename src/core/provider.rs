use std::error::Error;

use async_trait::async_trait;
use tokio_postgres::Client as PgClient;

use super::{SqlEntity, WhereCondition};

/// Whatever that aims to be a database data source (query, table, function
/// etc.) This has to be the SQL definition as it will be interpreted by
/// Postgres.
pub trait SqlDefinition: Sync + Send {
    /// SQL that is sent to Postgres (parameters shall be marked as `$?`)
    fn expand(&self, condition: &str) -> String;
}

/// A Provider uses an entity associated Projection to issue SQL queries and
/// return an iterator over results.
#[async_trait]
pub trait Provider<T>
where
    T: SqlEntity + Send,
{
    /// Return the SQL definition of this Provider.
    fn get_definition(&self) -> &dyn SqlDefinition;

    /// Launch a SQL statement to fetch the associated entities.
    async fn fetch(
        &self,
        client: &PgClient,
        condition: WhereCondition<'_>,
    ) -> Result<Vec<T>, Box<dyn Error>> {
        let (expression, parameters) = condition.expand();
        let sql = self.get_definition().expand(&expression);
        let mut items: Vec<T> = Vec::new();

        for row in client.query(&sql, parameters.as_slice()).await? {
            items.push(T::hydrate(row)?);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {}

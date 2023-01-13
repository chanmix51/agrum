use std::error::Error;
use std::marker::PhantomData;

use tokio_postgres::{types::ToSql, Client as PgClient};

use super::{SqlDefinition, SqlEntity};

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
    pub fn get_definition(&self, condition: &str) -> &Box<dyn SqlDefinition> {
        &self.definition
    }

    /// Launch a SQL statement to fetch the associated entities.
    pub async fn find(
        &'client self,
        condition: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, Box<dyn Error>> {
        let sql = self.definition.expand(condition);
        let mut items: Vec<T> = Vec::new();

        for row in self.pg_client.query(&sql, params).await? {
            items.push(T::hydrate(row)?);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {}

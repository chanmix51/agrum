use std::marker::PhantomData;

use tokio_postgres::{types::ToSql, Client};

use crate::StdResult;

use super::{SqlEntity, WhereCondition};

pub type SqlQueryWithParameters<'a> = (String, Vec<&'a (dyn ToSql + Sync)>);

/// Whatever that aims to be a database data source (query, table, function
/// etc.) This has to be the SQL definition as it will be interpreted by
/// Postgres.
pub trait SqlDefinition: Sync + Send {
    /// SQL that is sent to Postgres (parameters shall be marked as `$?`)
    fn expand<'a>(&self, condition: WhereCondition<'a>) -> SqlQueryWithParameters<'a>;
}

/// A ProviderBuilder provides an easy way to build entity providers. It has ownership over the
/// connetion so use it only when you can give it the connection.
pub struct ProviderBuilder {
    client: Client,
}

impl ProviderBuilder {
    /// Create a new builder instance. This takes the ownership over the Postgres client.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Return a borrow of the internal Postgres client.
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// Execute a query without returned values
    pub async fn execute(&self, query: &str) -> StdResult<()> {
        let _ = self.client.execute(query, &[]).await?;

        Ok(())
    }

    /// Create a new Provider
    pub fn build_provider<T>(&self, definition: Box<dyn SqlDefinition>) -> Provider<'_, T>
    where
        T: SqlEntity,
    {
        Provider {
            client: &self.client,
            definition,
            _entity_type: PhantomData,
        }
    }
}

/// A Provider uses an entity associated Projection to issue SQL queries and
/// return an iterator over results.
pub struct Provider<'client, T>
where
    T: SqlEntity,
{
    client: &'client Client,
    definition: Box<dyn SqlDefinition>,
    _entity_type: PhantomData<T>,
}

impl<'client, T> Provider<'client, T>
where
    T: SqlEntity,
{
    /// Constructor
    pub fn new(client: &'client Client, definition: Box<dyn SqlDefinition>) -> Self {
        Self {
            client,
            definition,
            _entity_type: PhantomData,
        }
    }

    /// Launch a SQL statement to fetch the associated entities.
    pub async fn fetch(&self, condition: WhereCondition<'_>) -> StdResult<Vec<T>> {
        let (sql, parameters) = self.definition.expand(condition);
        let mut items: Vec<T> = Vec::new();

        for row in self.client.query(&sql, parameters.as_slice()).await? {
            items.push(T::hydrate(row)?);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {}

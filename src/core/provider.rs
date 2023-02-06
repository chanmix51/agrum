use std::marker::PhantomData;
use std::{error::Error, fmt::Write};

use async_trait::async_trait;
use tokio_postgres::Client as PgClient;

use super::{SqlEntity, WhereCondition};

/// Whatever that aims to be a database data source (query, table, function
/// etc.) This has to be the SQL definition as it will be interpreted by
/// Postgres.
pub trait SqlDefinition: Send + Sync {
    /// SQL that is sent to Postgres (parameters shall be ?)
    fn expand(&self, condition: String) -> String;
}

#[async_trait]
pub trait Provider<T: SqlEntity + Send + Sync>: Send + Sync {
    fn get_definition(&self) -> &dyn SqlDefinition;
    async fn find(
        &self,
        condition: WhereCondition<'_>,
    ) -> Result<Vec<T>, Box<dyn Error + Sync + Send>>;
}

/// A Provider is a structure that holds the connection and the pipeworks to
/// actually perform queries.
pub struct DbProvider<'client, T>
where
    T: SqlEntity + Send + Sync,
{
    pg_client: &'client PgClient,
    definition: Box<dyn SqlDefinition>,
    _phantom: PhantomData<T>,
}

/// A Provider uses an entity associated Projection to issue SQL queries and
/// return an iterator over results.
impl<'client, T> DbProvider<'client, T>
where
    T: SqlEntity + Send + Sync,
{
    /// constructor
    pub fn new(pg_client: &'client PgClient, definition: Box<dyn SqlDefinition>) -> Self {
        Self {
            pg_client,
            definition,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<'client, T> Provider<T> for DbProvider<'client, T>
where
    T: SqlEntity + Send + Sync,
{
    /// Return the SQL definition of this Provider.
    fn get_definition(&self) -> &dyn SqlDefinition {
        self.definition.as_ref()
    }

    /// Launch a SQL statement to fetch the associated entities.
    async fn find(
        &self,
        condition: WhereCondition<'_>,
    ) -> Result<Vec<T>, Box<dyn Error + Sync + Send>> {
        let (expression, parameters) = condition.expand();
        let sql = self.get_definition().expand(expression);
        let mut items: Vec<T> = Vec::new();

        println!("SQL = “{sql}”.");

        for row in self.pg_client.query(&sql, parameters.as_slice()).await? {
            items.push(T::hydrate(row)?);
        }

        Ok(items)
    }
}

/// FakeProvider is meant for testing.
struct FakeProvider<'buffer, T>
where
    T: SqlEntity + Send + Sync,
{
    buffer: &'buffer mut (dyn Write + Sync + Send),
    definition: Box<dyn SqlDefinition>,
    expection: Vec<T>,
}

impl<'buffer, T> FakeProvider<'buffer, T>
where
    T: SqlEntity + Send + Sync,
{
    /// Instantiate new FakeProvider.
    pub fn new(
        buffer: &'buffer mut (dyn Write + Sync + Send),
        definition: Box<dyn SqlDefinition>,
    ) -> Self {
        Self {
            buffer,
            definition,
            expection: Vec::new(),
        }
    }

    /// Set what will be returned by this provider when queried.
    pub fn set_expectation(&mut self, expectation: Vec<T>) {
        self.expection = expectation;
    }
}

#[async_trait]
impl<'buffer, T> Provider<T> for FakeProvider<'buffer, T>
where
    T: SqlEntity + Send + Sync,
{
    /// Return the SQL definition of this Provider.
    fn get_definition(&self) -> &dyn SqlDefinition {
        self.definition.as_ref()
    }

    /// Launch a SQL statement to fetch the associated entities.
    async fn find(
        &self,
        condition: WhereCondition<'_>,
    ) -> Result<Vec<T>, Box<dyn Error + Sync + Send>> {
        let (expression, _parameters) = condition.expand();
        let sql = self.get_definition().expand(expression);
        self.buffer.write_str(&sql)?;

        Ok(self.expection.clone())
    }
}

#[cfg(test)]
mod tests {}

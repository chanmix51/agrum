use std::{collections::HashMap, error::Error, marker::PhantomData};

use postgres::{fallible_iterator::FallibleIterator, Client, RowIter};

use super::{Entity, HydrationError, Projection};

pub type SourceAliases = HashMap<String, String>;

/// Iterator over query results that yield entities.
pub struct EntityIterator<'client, T> {
    iterator: RowIter<'client>,
    phantom: PhantomData<T>,
}

impl<'client, T> EntityIterator<'client, T> {
    /// Instanciate new iterator.
    pub fn new(iterator: RowIter<'client>) -> Self {
        Self {
            iterator,
            phantom: PhantomData,
        }
    }
}

impl<'client, T> FallibleIterator for EntityIterator<'client, T>
where
    T: Entity,
{
    type Item = T;
    type Error = HydrationError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.iterator
            .next()
            .map_err(|e| HydrationError::RowFetchFailed(e))?
            .map(|row| T::hydrate(row))
            .transpose()
    }
}

/// A Provider uses an entity associated Projection to issue SQL queries and
/// return an iterator over results.
pub trait Provider<'client> {
    /// Entity returned by the Provider.
    type Entity;

    /// Return the SQL definition of this Provider.
    fn get_definition(&self) -> String;

    /// Launch a SQL statement to fetch the associated entities.
    fn find(
        &'client self,
        condition: &str,
        params: &Vec<&str>,
    ) -> Result<EntityIterator<'client, Self::Entity>, Box<dyn Error>> {
        let sql = self.get_definition();
        let pg_client = self.get_client();
        let iterator = pg_client.query_raw(&sql, params)?;

        Ok(EntityIterator::new(iterator))
    }

    /// Share the database client instance.
    fn get_client(&'client self) -> &'client mut Client;

    /// Share the provider's [Projection].
    fn get_projection(&self) -> &Projection;
}

#[cfg(test)]
mod tests {}

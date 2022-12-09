use std::{collections::HashMap, error::Error, marker::PhantomData, pin::Pin};

use async_trait::async_trait;
use futures::StreamExt;
use futures_util::{stream::IntoStream, Stream, TryStream, TryStreamExt};
use tokio_postgres::{Client as PgClient, RowStream};

use super::{HydrationError, Projection, SqlEntity};

pub type SourceAliases = HashMap<String, String>;

/// A Provider uses an entity associated Projection to issue SQL queries and
/// return an iterator over results.
pub trait Provider<T>
where
    T: SqlEntity,
{
    /// Return the SQL definition of this Provider.
    fn get_definition(&self) -> String;

    /// Launch a SQL statement to fetch the associated entities.
    fn find(
        &self,
        condition: &str,
        params: &Vec<&str>,
    ) -> Result<Box<dyn Stream<Item = T>>, Box<dyn Error>>;
}

#[cfg(test)]
mod tests {}

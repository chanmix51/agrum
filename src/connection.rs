use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{Result, SqlEntity, SqlQuery};
use futures_core::Stream;
use tokio_postgres::{RowStream, Transaction as TokioTransaction, types::ToSql};

pub struct EntityStream<T: SqlEntity> {
    stream: Pin<Box<RowStream>>,
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> EntityStream<T> {
    pub fn new(stream: RowStream) -> Self {
        Self {
            stream: Box::pin(stream),
            _phantom: PhantomData,
        }
    }
}

impl<T: SqlEntity> Stream for EntityStream<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Safe: we only project to the `stream` field and call as_mut() on Pin<Box<RowStream>>;
        // we do not move or unpin any part of Self.
        let stream = unsafe { self.get_unchecked_mut().stream.as_mut() };

        match stream.poll_next(cx) {
            Poll::Ready(Some(result)) => {
                let item: Result<T> = result
                    .map_err(anyhow::Error::from)
                    .and_then(|row| T::hydrate(&row).map_err(anyhow::Error::from));
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct Transaction<'a> {
    transaction: TokioTransaction<'a>,
}

impl<'a> Transaction<'a> {
    pub async fn start(transaction: TokioTransaction<'a>) -> Self {
        Self { transaction }
    }

    pub async fn commit(self) -> Result<()> {
        self.transaction.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        self.transaction.rollback().await?;
        Ok(())
    }

    pub async fn query<E: SqlEntity>(&self, query: SqlQuery<'a, E>) -> Result<EntityStream<E>> {
        let (statement, parameters) = query.expand();
        let parameters: Vec<&dyn ToSql> = parameters.into_iter().map(|p| p as &dyn ToSql).collect();
        let stream = self.transaction.query_raw(&statement, parameters).await?;
        Ok(EntityStream::new(stream))
    }
}

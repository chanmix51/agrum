use std::error::Error;

use ::futures::pin_mut;
use agrum::{Projection, SqlEntity, Structure};
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::{self};
use tokio_postgres::{Client, NoTls, Row};

#[derive(Debug, Clone, PartialEq)]
struct WhateverEntity {
    entity_id: u32,
    content: String,
    has_thing: bool,
    something: Option<i64>,
}

impl SqlEntity for WhateverEntity {
    fn hydrate(row: Row) -> Result<Self, agrum::HydrationError>
    where
        Self: Sized,
    {
        Ok(Self {
            entity_id: row.get("entity_id"),
            content: row.get("content"),
            has_thing: row.get("has_thing"),
            something: row.get("something"),
        })
    }

    fn get_structure() -> Structure {
        let mut structure = Structure::new();
        structure
            .set_field("entity_id", "int")
            .set_field("content", "text")
            .set_field("has_thing", "bool")
            .set_field("something", "int");

        structure
    }
}

struct WhateverProvider<'client> {
    structure: Structure,
    projection: Projection,
    pg_client: &'client Client,
}

impl<'client> WhateverProvider<'client> {
    pub fn new(pg_client: &'client Client) -> Self {
        let structure = WhateverEntity::get_structure();
        let projection = Projection::from_structure(structure.clone(), "whatever");

        Self {
            structure,
            projection,
            pg_client,
        }
    }
}
/*
#[async_trait]
impl<'client> Provider for WhateverProvider<'client> {
    type Entity = WhateverEntity;

    fn get_definition(&self) -> String {
        todo!()
    }

    async fn find(
        &self,
        condition: &str,
        params: &Vec<&str>,
    ) -> Result<EntityStream<Self::Entity>, Box<dyn Error>> {
        todo!()
    }
}
*/

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client.query_raw("SELECT $1::int as entity_id, $2::text as content, true::bool as has_thing, null::int as something", &[&"1", &"hello world"])
        .await?
        .map(|res| WhateverEntity::hydrate(res.unwrap()));
    pin_mut!(rows);
    let entity = rows.next().await.expect("there must be a result").unwrap();

    assert_eq!(
        WhateverEntity {
            entity_id: 1,
            content: "hello world".to_string(),
            has_thing: true,
            something: None
        },
        entity
    );
    Ok(())
}

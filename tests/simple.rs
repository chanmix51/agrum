use std::error::Error;

use agrum::{
    core::{
        HydrationError, Projection, Provider, SourceAliases, SqlDefinition, SqlEntity, Structure,
        Structured, WhereCondition,
    },
    params,
};
use tokio_postgres::{Client, NoTls, Row};

#[derive(Debug, Clone, PartialEq)]
pub struct WhateverEntity {
    entity_id: i32,
    content: String,
    has_thing: bool,
    something: Option<i32>,
}

impl Structured for WhateverEntity {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("thing_id", "int"),
            ("content", "text"),
            ("has_thing", "bool"),
            ("maybe", "int"),
        ])
    }
}

impl SqlEntity for WhateverEntity {
    fn hydrate(row: Row) -> Result<Self, HydrationError>
    where
        Self: Sized,
    {
        Ok(Self {
            entity_id: row.get("thing_id"),
            content: row.get("content"),
            has_thing: row.get("has_thing"),
            something: row.get("maybe"),
        })
    }
}

struct WhateverSqlDefinition {
    projection: Projection<WhateverEntity>,
    source_aliases: SourceAliases,
}

impl WhateverSqlDefinition {
    pub fn new(projection: Projection<WhateverEntity>) -> Self {
        Self {
            projection,
            source_aliases: SourceAliases::new(&[("thing", "whatever")]),
        }
    }
}

impl SqlDefinition for WhateverSqlDefinition {
    fn expand(&self, condition: &str) -> String {
        let projection = self.projection.expand(&self.source_aliases);

        format!("select {projection} from (values (1, 'whatever', true, null), (2, 'something else', false, 1)) whatever (thing_id, content, has_thing, maybe) where {condition}")
    }
}

pub struct WhateverEntityProvider<'client> {
    client: &'client Client,
    definition: WhateverSqlDefinition,
}

impl<'client> Provider<WhateverEntity> for WhateverEntityProvider<'client> {
    fn get_definition(&self) -> &dyn SqlDefinition {
        &self.definition
    }
}

impl<'client> WhateverEntityProvider<'client> {
    pub fn new(client: &'client Client, projection: Projection<WhateverEntity>) -> Self {
        Self {
            client,
            definition: WhateverSqlDefinition::new(projection),
        }
    }

    pub async fn fetch_all(&self) -> Result<Vec<WhateverEntity>, Box<dyn Error>> {
        self.fetch(self.client, WhereCondition::default()).await
    }

    pub async fn fetch_by_id(
        &self,
        thing_id: i32,
    ) -> Result<Option<WhateverEntity>, Box<dyn Error>> {
        let condition = WhereCondition::new("thing_id = $?", params![thing_id]);
        let entity = self.fetch(self.client, condition).await?.pop();

        Ok(entity)
    }
}

async fn get_client() -> Client {
    let config = std::fs::read_to_string("tests/config.txt").unwrap();
    let (client, connection) = tokio_postgres::connect(&config, NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
}

#[tokio::test]
async fn provider_no_filter() {
    // Connect to the database.
    let client = get_client().await;
    let provider = WhateverEntityProvider::new(&client, Projection::<WhateverEntity>::default());

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    let entities = provider.fetch_all().await.unwrap();

    assert_eq!(
        vec![
            WhateverEntity {
                entity_id: 1,
                content: "whatever".to_string(),
                has_thing: true,
                something: None,
            },
            WhateverEntity {
                entity_id: 2,
                content: "something else".to_string(),
                has_thing: false,
                something: Some(1),
            },
        ],
        entities
    );
}

#[tokio::test]
async fn provider_with_filter() {
    // Connect to the database.
    let client = get_client().await;
    let provider = WhateverEntityProvider::new(&client, Projection::<WhateverEntity>::default());

    let entity = provider
        .fetch_by_id(1)
        .await
        .unwrap()
        .expect("there should be a thing with ID=1");

    assert_eq!(
        WhateverEntity {
            entity_id: 1,
            content: "whatever".to_string(),
            has_thing: true,
            something: None,
        },
        entity
    );
}

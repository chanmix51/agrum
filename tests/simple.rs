mod utils;

use utils::get_config;

use std::error::Error;

use agrum::{
    core::{
        HydrationError, Projection, Provider, ProviderBuilder, SourceAliases, SqlDefinition,
        SqlEntity, Structure, Structured, WhereCondition,
    },
    params,
};
use tokio_postgres::{Client, NoTls, Row};

/// Entity that will be hydrated from query results.
#[derive(Debug, Clone, PartialEq)]
pub struct WhateverEntity {
    entity_id: i32,
    content: String,
    has_thing: bool,
    something: Option<i32>,
}

/// Database representation of this entity.
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

/// How to create instances from query results.
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

/// A simple query that maps the table to the entity.
/// It uses a managed projection that pops out [WhateverEntity] instances.
struct WhateverSqlDefinition {
    projection: Projection<WhateverEntity>,
    source_aliases: SourceAliases,
}

impl WhateverSqlDefinition {
    /// Constructor
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

        format!(
            r#"
select {projection}
from (values (1, 'whatever', true, null), (2, 'something else', false, 1)) whatever (thing_id, content, has_thing, maybe)
where {condition}"#
        )
    }
}

/// Repositories are a good place to perform several queries through several providers. It manages
/// conditions and transactions.
struct WhateverEntityRepository<'client> {
    provider: Provider<'client, WhateverEntity>,
}

impl<'client> WhateverEntityRepository<'client> {
    pub fn new(provider: Provider<'client, WhateverEntity>) -> Self {
        Self { provider }
    }

    pub async fn fetch_all(&self) -> Result<Vec<WhateverEntity>, Box<dyn Error + Sync + Send>> {
        self.provider.fetch(WhereCondition::default()).await
    }

    pub async fn fetch_by_id(
        &self,
        thing_id: i32,
    ) -> Result<Option<WhateverEntity>, Box<dyn Error + Sync + Send>> {
        let condition = WhereCondition::new("thing_id = $?", params![thing_id]);
        let entity = self.provider.fetch(condition).await?.pop();

        Ok(entity)
    }
}

/// This function is used by different test cases; it creates a connection and return a client.
async fn get_client() -> Client {
    let config = get_config(vec![]).unwrap();
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
    // This example uses the provider builder to create the internal provider used by the
    // repository. Types are implied by the definitions.
    let provider_builder = ProviderBuilder::new(get_client().await);
    let sql_definition = Box::new(WhateverSqlDefinition::new(Projection::default()));
    let repository = WhateverEntityRepository::new(provider_builder.build_provider(sql_definition));

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    let entities = repository.fetch_all().await.unwrap();

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
    // In this example, the provider is used directly in the code without the provider_builder.
    let client = get_client().await;
    let sql_definition = Box::new(WhateverSqlDefinition::new(Projection::default()));
    let provider = Provider::new(&client, sql_definition);
    let repository = WhateverEntityRepository::new(provider);

    let entity = repository
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

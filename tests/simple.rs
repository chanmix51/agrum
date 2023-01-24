use agrum::core::{
    HydrationError, Projection, Provider, SourceAliases, SqlDefinition, SqlEntity, Structure,
    WhereCondition,
};
use tokio::{self};
use tokio_postgres::{Client, NoTls, Row};

#[derive(Debug, Clone, PartialEq)]
struct WhateverEntity {
    entity_id: i32,
    content: String,
    has_thing: bool,
    something: Option<i32>,
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

    fn get_structure() -> Structure {
        let mut structure = Structure::default();
        structure
            .set_field("thing_id", "int")
            .set_field("content", "text")
            .set_field("has_thing", "bool")
            .set_field("maybe", "int");

        structure
    }
}

struct WhateverSqlDefinition {
    projection: Projection,
}

impl WhateverSqlDefinition {
    pub fn new() -> Self {
        let projection = Projection::from_structure(WhateverEntity::get_structure(), "main");

        Self { projection }
    }
}

impl SqlDefinition for WhateverSqlDefinition {
    fn expand(&self, condition: String) -> String {
        let projection = self
            .projection
            .expand(&SourceAliases::new(vec![("main", "whatever")]));

        format!("select {projection} from (values (1, 'whatever', true, null), (2, 'something else', false, 1)) whatever (thing_id, content, has_thing, maybe) where {condition}")
    }
}

async fn get_client() -> Client {
    let config = "host=postgres.lxc user=greg";
    let (client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();
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
    let provider: Provider<WhateverEntity> =
        Provider::new(&client, Box::new(WhateverSqlDefinition::new()));

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    let rows = provider.find(WhereCondition::default()).await.unwrap();

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
        rows
    );
}

#[tokio::test]
async fn provider_with_filter() {
    // Connect to the database.
    let client = get_client().await;
    let provider: Provider<WhateverEntity> =
        Provider::new(&client, Box::new(WhateverSqlDefinition::new()));

    let rows = provider
        .find(WhereCondition::where_in(
            "thing_id",
            vec![&1_i32, &12_i32, &15_i32],
        ))
        .await
        .unwrap();

    assert_eq!(
        vec![WhateverEntity {
            entity_id: 1,
            content: "whatever".to_string(),
            has_thing: true,
            something: None,
        },],
        rows
    );
}

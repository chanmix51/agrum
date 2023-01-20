use std::error::Error;

use agrum::{
    Projection, Provider, SourceAliases, SqlDefinition, SqlEntity, Structure, WhereCondition,
};
use tokio::{self};
use tokio_postgres::{NoTls, Row};

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

struct WhateverSqlDefinition {
    definition: String,
}

impl WhateverSqlDefinition {
    pub fn new() -> Self {
        let projection = Projection::from_structure(WhateverEntity::get_structure(), "main")
            .expand(&SourceAliases::new(vec![("main", "whatever")]));
        let sql = format!("select {projection} from (values (1, 'whatever', true, null), (2, 'something else', false, 1)) whatever (thing_id, something, is_thing, maybe)");

        Self { definition: sql }
    }
}

impl SqlDefinition for WhateverSqlDefinition {
    fn expand(&self, condition: &WhereCondition) -> String {
        self.definition.clone()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
    let provider: Provider<WhateverEntity> =
        Provider::new(&client, Box::new(WhateverSqlDefinition::new()));

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = provider.find("", &[]).await?;

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

    Ok(())
}

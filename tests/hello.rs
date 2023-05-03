mod utils;

use utils::get_config;

use agrum::core::{
    HydrationError, Provider, SqlDefinition, SqlEntity, Structure, Structured, WhereCondition,
};
use tokio_postgres::{NoTls, Row};

/// Structure that will own the data from Postgres.
#[derive(Debug, PartialEq)]
struct DbMessage {
    message: String,
}

/// Description of the database representation of this entity data.
impl Structured for DbMessage {
    fn get_structure() -> Structure {
        Structure::new(&[("message", "text")])
    }
}

/// Description of how to hydrate the entity from the DB data.
impl SqlEntity for DbMessage {
    fn hydrate(row: Row) -> Result<Self, HydrationError> {
        Ok(Self {
            message: row.get("message"),
        })
    }
}

/// SQL query, there is no need for a managed projection in this example.
#[derive(Debug, Default)]
struct HelloWorldDbMessageDefinition;

impl SqlDefinition for HelloWorldDbMessageDefinition {
    fn expand(&self, _condition: &str) -> String {
        format!(r#"select 'hello world' as message"#)
    }
}

#[tokio::test]
async fn hello_world() {
    let config = get_config(vec![]).unwrap();
    let (client, connection) = tokio_postgres::connect(&config, NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let provider: Provider<DbMessage> =
        Provider::new(&client, Box::new(HelloWorldDbMessageDefinition));

    let mut messages = provider.fetch(WhereCondition::default()).await.unwrap();

    assert_eq!(
        DbMessage {
            message: "hello world".to_string()
        },
        messages.pop().unwrap()
    );
}

use super::get_config;

use tokio_postgres::{Client, NoTls};

pub async fn get_client() -> Client {
    let config = get_config(Vec::new()).unwrap();
    let (client, connection) = tokio_postgres::connect(&config, NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
}

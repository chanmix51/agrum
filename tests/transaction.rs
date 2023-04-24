use agrum::core::{ProviderBuilder, Transaction};

use tokio_postgres::{Client, NoTls};

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
async fn transaction_commit() {
    // This example shows how to perform transactions
    let provider_builder = ProviderBuilder::new(get_client().await);
    let transaction = Transaction::default();
    provider_builder
        .execute(&transaction.start())
        .await
        .unwrap();
    provider_builder
        .execute(&transaction.set_savepoint("whatever"))
        .await
        .unwrap();
    provider_builder
        .execute(&transaction.rollback_to_savepoint("whatever"))
        .await
        .unwrap();
    provider_builder
        .execute(&transaction.commit())
        .await
        .unwrap();
}

#[tokio::test]
async fn transaction_rollback() {
    let provider_builder = ProviderBuilder::new(get_client().await);
    let transaction = Transaction::default();
    provider_builder
        .execute(&transaction.start())
        .await
        .unwrap();
    provider_builder
        .execute(&transaction.set_savepoint("whatever"))
        .await
        .unwrap();
    provider_builder
        .execute(&transaction.rollback())
        .await
        .unwrap();
}

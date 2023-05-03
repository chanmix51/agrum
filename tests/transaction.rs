mod utils;

use utils::get_config;

use agrum::core::{Transaction, TransactionStatus, TransactionToken};

use tokio_postgres::{Client, NoTls};

async fn get_client() -> Client {
    let config = get_config(Vec::new()).unwrap();
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
    let client = get_client().await;
    let mut transaction = Transaction::new(&client, TransactionToken::default());
    assert_eq!(TransactionStatus::Unstarted, transaction.get_status());

    transaction.start().await.unwrap();
    assert_eq!(TransactionStatus::Started, transaction.get_status());

    transaction.set_savepoint("whatever").await.unwrap();
    assert_eq!(TransactionStatus::Started, transaction.get_status());

    transaction.rollback_to_savepoint("whatever").await.unwrap();
    assert_eq!(TransactionStatus::Started, transaction.get_status());

    transaction.release_savepoint("whatever").await.unwrap();
    assert_eq!(TransactionStatus::Started, transaction.get_status());

    transaction.commit().await.unwrap();
    assert_eq!(TransactionStatus::Committed, transaction.get_status());
}

#[tokio::test]
async fn transaction_rollback() {
    let client = get_client().await;
    let mut transaction = Transaction::new(&client, TransactionToken::default());

    transaction.start().await.unwrap();
    transaction.rollback().await.unwrap();

    assert_eq!(TransactionStatus::Aborted, transaction.get_status());
}

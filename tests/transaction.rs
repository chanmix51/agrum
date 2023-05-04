mod utils;

use utils::get_client;

use agrum::core::{Transaction, TransactionStatus, TransactionToken};

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

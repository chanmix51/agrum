use std::fmt::Display;

/// PostgreSQL transaction [isolation
/// levels](https://www.postgresql.org/docs/current/transaction-iso.html).
pub enum IsolationLevel {
    /// Cannot read data from uncommitted transactions.
    ReadCommitted,

    /// Data in this transaction are not altered by other committed transactions.
    RepeatableRead,

    /// Data consistency is guaranteed like transactions are ran one after the other.
    Serializable,
}

impl Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadCommitted => write!(f, "read committed"),
            Self::RepeatableRead => write!(f, "repeatable read"),
            Self::Serializable => write!(f, "serializable"),
        }
    }
}

/// PostgreSQL [transaction
/// type](https://www.postgresql.org/docs/current/sql-set-transaction.html).
pub enum TransactionType {
    /// Read-only transaction
    ReadOnly,

    /// Read-write transaction
    ReadWrite,
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "read only"),
            Self::ReadWrite => write!(f, "read write"),
        }
    }
}

pub struct Transaction {
    isolation_level: IsolationLevel,
    transaction_type: TransactionType,
}

impl Default for Transaction {
    fn default() -> Self {
        Self::new(IsolationLevel::ReadCommitted, TransactionType::ReadWrite)
    }
}

impl Transaction {
    /// Constructor
    pub fn new(isolation_level: IsolationLevel, transaction_type: TransactionType) -> Self {
        Self {
            isolation_level,
            transaction_type,
        }
    }

    /// Shortcut to build a repeatable-read read/write transaction
    pub fn repeatable_read() -> Self {
        Self::new(IsolationLevel::RepeatableRead, TransactionType::ReadWrite)
    }

    /// Shortcut to build a serializable read/write transaction
    pub fn serializable() -> Self {
        Self::new(IsolationLevel::Serializable, TransactionType::ReadWrite)
    }

    /// Start a new transaction
    pub fn start(&self) -> String {
        format!(
            "start transaction isolation level {} {}",
            self.isolation_level, self.transaction_type
        )
    }

    /// Commit a transaction
    pub fn commit(&self) -> String {
        "commit".to_string()
    }

    /// Rollback a transaction
    pub fn rollback(&self) -> String {
        "rollback".to_string()
    }

    /// Rollback a transaction to the given savepoint
    pub fn rollback_to_savepoint(&self, savepoint: &str) -> String {
        format!("rollback to savepoint {savepoint}")
    }

    /// Set a savepoint
    pub fn set_savepoint(&self, savepoint: &str) -> String {
        format!("savepoint {savepoint}")
    }

    /// Release the given savepoint
    pub fn release_savepoint(&self, savepoint: &str) -> String {
        format!("release savepoint {savepoint}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_transaction_read_commited_read_write() {
        let transaction = Transaction::default();
        let query = transaction.start();

        assert_eq!(
            "start transaction isolation level read committed read write".to_string(),
            query
        );
    }

    #[test]
    fn start_transaction_read_commited_read_only() {
        let transaction =
            Transaction::new(IsolationLevel::ReadCommitted, TransactionType::ReadOnly);
        let query = transaction.start();

        assert_eq!(
            "start transaction isolation level read committed read only".to_string(),
            query
        );
    }

    #[test]
    fn start_transaction_repeatable_read_read_write() {
        let transaction = Transaction::repeatable_read();
        let query = transaction.start();

        assert_eq!(
            "start transaction isolation level repeatable read read write".to_string(),
            query
        );
    }

    #[test]
    fn start_transaction_repeatable_read_read_only() {
        let transaction =
            Transaction::new(IsolationLevel::RepeatableRead, TransactionType::ReadOnly);
        let query = transaction.start();

        assert_eq!(
            "start transaction isolation level repeatable read read only".to_string(),
            query
        );
    }

    #[test]
    fn start_transaction_serializable_read_write() {
        let transaction = Transaction::serializable();
        let query = transaction.start();

        assert_eq!(
            "start transaction isolation level serializable read write".to_string(),
            query
        );
    }

    #[test]
    fn start_transaction_serializable_read_only() {
        let transaction = Transaction::new(IsolationLevel::Serializable, TransactionType::ReadOnly);
        let query = transaction.start();

        assert_eq!(
            "start transaction isolation level serializable read only".to_string(),
            query
        );
    }

    #[test]
    fn rollback_transaction() {
        let transaction = Transaction::default();

        assert_eq!("rollback".to_string(), transaction.rollback());
    }

    #[test]
    fn rollback_to_savepoint() {
        let transaction = Transaction::default();

        assert_eq!(
            "rollback to savepoint whatever".to_string(),
            transaction.rollback_to_savepoint("whatever")
        );
    }

    #[test]
    fn set_savepoint() {
        let transaction = Transaction::default();

        assert_eq!(
            "savepoint whatever".to_string(),
            transaction.set_savepoint("whatever")
        );
    }

    #[test]
    fn release_savepoint() {
        let transaction = Transaction::default();

        assert_eq!(
            "release savepoint whatever".to_string(),
            transaction.release_savepoint("whatever")
        );
    }

    #[test]
    fn commit_transaction() {
        let transaction = Transaction::default();

        assert_eq!("commit".to_string(), transaction.commit());
    }
}

pub type Result<T> = std::result::Result<T, ProcessTransactionError>;

#[derive(thiserror::Error, Debug)]
pub enum ProcessTransactionError {
    #[error("Client is locked")]
    ClientLocked,
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),
    #[error("Transaction already exists")]
    DuplicateTransaction,
    #[error("Transaction does not exist")]
    TransactionNotFound,
    #[error("Unauthorized access to transaction")]
    Unauthorized,
    #[error("Client not found")]
    ClientNotFound,
    #[error("Arithmetic overflow occurred")]
    Overflow,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid operation: {0}")]
    InvalidOperation(&'static str),
}

pub type Result<T> = std::result::Result<T, ProcessTransactionError>;

#[derive(thiserror::Error, Debug)]
pub enum ProcessTransactionError {
    #[error("Client is locked")]
    ClientLocked,
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),
    #[error("Transaction already exists")]
    TransactionAlreadyExists,
    #[error("Overflowed bounds of monetary amount")]
    Overflow,
    #[error("Insufficient funds")]
    InsufficientFunds,
}

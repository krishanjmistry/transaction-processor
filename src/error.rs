pub type Result<T> = std::result::Result<T, ProcessTransactionError>;

#[derive(thiserror::Error, Debug)]
pub enum ProcessTransactionError {
    #[error("Client is locked")]
    ClientLocked,
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),
    #[error("Transaction already exists")]
    TransactionAlreadyExists,
    #[error("Transaction does not exist")]
    TransactionDoesNotExist,
    #[error("Client not found")]
    ClientNotFound,
    #[error("Overflowed bounds of monetary amount")]
    Overflow,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Claim state error: {0}")]
    ClaimStateError(&'static str),
}

pub type Result<T> = std::result::Result<T, ProcessTransactionError>;

#[derive(thiserror::Error, Debug)]
pub enum ProcessTransactionError {
    #[error("Client is locked")]
    ClientLocked,
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),
}

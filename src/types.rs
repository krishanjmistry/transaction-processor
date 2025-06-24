use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientId(u16);

#[derive(Debug, Deserialize)]
pub struct TransactionId(u32);

pub type MonetaryAmount = Decimal;

#[derive(Debug)]
pub struct TransactionRequest {
    client: ClientId,
    transaction: TransactionId,
    request: TransactionType,
}

impl TransactionRequest {
    pub fn new(client: ClientId, transaction: TransactionId, request: TransactionType) -> Self {
        Self {
            client,
            transaction,
            request,
        }
    }
}

#[derive(Debug)]
pub enum TransactionType {
    Deposit(MonetaryAmount),
    Withdrawal(MonetaryAmount),
    Claim(ClaimType),
}

#[derive(Debug)]
pub enum ClaimType {
    Dispute,
    Resolve,
    Chargeback,
}

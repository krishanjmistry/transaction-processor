use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct ClientId(u16);

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn client(&self) -> ClientId {
        self.client
    }
    pub fn transaction(&self) -> TransactionId {
        self.transaction
    }
    pub fn request(&self) -> TransactionType {
        self.request
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionType {
    Monetary(MonetaryTransaction),
    Claim(ClaimType),
}

#[derive(Debug, Clone, Copy)]
pub enum MonetaryTransaction {
    Deposit(MonetaryAmount),
    Withdrawal(MonetaryAmount),
}

#[derive(Debug, Clone, Copy)]
pub enum ClaimType {
    Dispute,
    Resolve,
    Chargeback,
}

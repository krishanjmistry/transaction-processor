use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClientId(u16);

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(u32);

pub type MonetaryAmount = Decimal;

#[derive(Debug)]
pub struct TransactionRequest {
    pub client: ClientId,
    pub transaction: TransactionId,
    pub request_type: RequestType,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestType {
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

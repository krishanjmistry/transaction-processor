use std::collections::HashMap;

use crate::{
    TransactionRequest,
    types::{ClientId, MonetaryAmount, TransactionId},
};

pub struct Exchange {
    clients: HashMap<ClientId, Client>,
    transactions: HashMap<TransactionId, TransactionRequest>,
}

struct Client {
    available: MonetaryAmount,
    held: MonetaryAmount,
    locked: bool,
    transactions: HashMap<TransactionId, TransactionInformation>,
}

struct TransactionInformation {
    request: TransactionRequest,
    // We only need to keep track when there is a dispute or chargeback
    // When a claim is resolved, we can go back to the None state
    claim: Option<ClaimState>,
}

enum ClaimState {
    Disputed,
    Chargebacked,
}

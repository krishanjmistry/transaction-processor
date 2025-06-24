use std::collections::HashMap;

use crate::{
    TransactionRequest,
    error::{ProcessTransactionError, Result},
    types::{ClientId, MonetaryAmount, MonetaryTransaction, TransactionId, TransactionType},
};

pub struct Exchange {
    clients: HashMap<ClientId, Client>,
    transactions: HashMap<TransactionId, ClientId>,
}

impl Exchange {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, request: TransactionRequest) -> Result<()> {
        match request.request() {
            TransactionType::Monetary(transaction) => {
                if self.transactions.contains_key(&request.transaction()) {
                    return Err(ProcessTransactionError::TransactionAlreadyExists);
                }

                let client = self
                    .clients
                    .entry(request.client())
                    .or_insert(Client::new());

                client.process_monetary_request(request.transaction(), transaction)?;

                self.transactions
                    .insert(request.transaction(), request.client());

                Ok(())
            }
            TransactionType::Claim(claim_type) => {
                unimplemented!()
            }
        }
    }

    pub fn get_clients(&self) -> &HashMap<ClientId, Client> {
        &self.clients
    }
}

pub struct Client {
    available: MonetaryAmount,
    held: MonetaryAmount,
    locked: bool,
    transactions: HashMap<TransactionId, TransactionInformation>,
}

impl Client {
    fn new() -> Self {
        Self {
            available: MonetaryAmount::ZERO,
            held: MonetaryAmount::ZERO,
            locked: false,
            transactions: HashMap::new(),
        }
    }

    pub fn available(&self) -> MonetaryAmount {
        self.available
    }

    pub fn held(&self) -> MonetaryAmount {
        self.held
    }

    pub fn total(&self) -> MonetaryAmount {
        self.available + self.held
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    fn process_monetary_request(
        &mut self,
        transaction_id: TransactionId,
        transaction: MonetaryTransaction,
    ) -> Result<()> {
        if self.locked {
            return Err(ProcessTransactionError::ClientLocked);
        }

        match transaction {
            MonetaryTransaction::Deposit(amount) => {
                self.available = self
                    .available
                    .checked_add(amount)
                    .ok_or(ProcessTransactionError::Overflow)?;
            }
            MonetaryTransaction::Withdrawal(amount) => {
                if self.available < amount {
                    return Err(ProcessTransactionError::InsufficientFunds);
                }
                self.available = self
                    .available
                    .checked_sub(amount)
                    .ok_or(ProcessTransactionError::Overflow)?;
            }
        }

        self.transactions.insert(
            transaction_id,
            TransactionInformation {
                request: transaction,
                claim: None,
            },
        );
        Ok(())
    }
}

struct TransactionInformation {
    request: MonetaryTransaction,
    // We only need to keep track when there is a dispute or chargeback
    // When a claim is resolved, we can go back to the None state
    claim: Option<ClaimState>,
}

enum ClaimState {
    Disputed,
    Chargebacked,
}

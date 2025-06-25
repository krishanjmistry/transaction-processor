use std::collections::HashMap;

use crate::{
    TransactionRequest,
    error::{ProcessTransactionError, Result},
    types::{ClaimType, ClientId, MonetaryAmount, MonetaryTransaction, RequestType, TransactionId},
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
        match request.request_type {
            RequestType::Monetary(transaction) => {
                if self.transactions.contains_key(&request.transaction) {
                    return Err(ProcessTransactionError::DuplicateTransaction);
                }

                let client = self.clients.entry(request.client).or_insert(Client::new());

                client.process_monetary_request(request.transaction, transaction)?;

                self.transactions
                    .insert(request.transaction, request.client);

                Ok(())
            }
            RequestType::Claim(claim_type) => {
                let transaction_owner = self
                    .transactions
                    .get(&request.transaction)
                    .copied()
                    .ok_or(ProcessTransactionError::TransactionNotFound)?;

                if transaction_owner != request.client {
                    return Err(ProcessTransactionError::Unauthorized);
                }

                let client = self
                    .clients
                    .get_mut(&request.client)
                    .ok_or(ProcessTransactionError::ClientNotFound)?;

                client.process_claim(request.transaction, claim_type)?;

                Ok(())
            }
        }
    }

    pub fn get_clients(&self) -> &HashMap<ClientId, Client> {
        &self.clients
    }
}

pub struct Client {
    pub available: MonetaryAmount,
    pub held: MonetaryAmount,
    pub locked: bool,
    transactions: HashMap<TransactionId, TransactionInformation>,
}

struct TransactionInformation {
    request: MonetaryTransaction,
    /// We only need to keep track when there is a dispute or chargeback.
    /// When a claim is un-disputed or resolved, we can go back to the None state
    claim: Option<ClaimState>,
}

enum ClaimState {
    Disputed,
    Chargebacked,
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

    fn process_claim(
        &mut self,
        transaction_id: TransactionId,
        claim_type: ClaimType,
    ) -> Result<()> {
        if self.locked {
            return Err(ProcessTransactionError::ClientLocked);
        }

        let transaction_info = self
            .transactions
            .get_mut(&transaction_id)
            .ok_or(ProcessTransactionError::TransactionNotFound)?;

        match claim_type {
            ClaimType::Dispute => {
                if transaction_info.claim.is_some() {
                    return Err(ProcessTransactionError::InvalidOperation(
                        "Transaction already disputed",
                    ));
                }
                match transaction_info.request {
                    MonetaryTransaction::Deposit(amount) => {
                        self.held += amount;
                        self.available -= amount;
                    }
                    MonetaryTransaction::Withdrawal(_) => {
                        // No change to available funds, just mark as disputed
                    }
                };
                transaction_info.claim = Some(ClaimState::Disputed);
            }
            ClaimType::Resolve => {
                if let Some(ClaimState::Disputed) = transaction_info.claim {
                    match transaction_info.request {
                        MonetaryTransaction::Deposit(amount) => {
                            self.held -= amount;
                            self.available += amount;
                        }
                        MonetaryTransaction::Withdrawal(_) => {
                            // No change to available funds, just mark as disputed
                        }
                    };
                    transaction_info.claim = None;
                } else {
                    return Err(ProcessTransactionError::InvalidOperation(
                        "No dispute to resolve",
                    ));
                }
            }
            ClaimType::Chargeback => {
                if let Some(ClaimState::Disputed) = transaction_info.claim {
                    match transaction_info.request {
                        MonetaryTransaction::Deposit(amount) => {
                            self.held -= amount;
                        }
                        MonetaryTransaction::Withdrawal(amount) => {
                            self.available += amount;
                        }
                    };
                    transaction_info.claim = Some(ClaimState::Chargebacked);
                    self.locked = true;
                } else {
                    return Err(ProcessTransactionError::InvalidOperation(
                        "No dispute to chargeback",
                    ));
                }
            }
        }

        Ok(())
    }
}

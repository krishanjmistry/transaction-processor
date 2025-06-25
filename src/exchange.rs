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
                            // No change to available funds as we'd only just marked as disputed
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::{Decimal, dec};

    #[test]
    fn test_deposit_increases_available_balance() {
        let mut client = Client::new();
        let transaction_id = TransactionId(1);
        let deposit_amount = dec!(1.234);

        let result = client
            .process_monetary_request(transaction_id, MonetaryTransaction::Deposit(deposit_amount));

        assert!(result.is_ok());
        assert_eq!(client.available, deposit_amount);
        assert_eq!(client.held, Decimal::ZERO);
        assert!(!client.locked);
        assert!(client.transactions.contains_key(&transaction_id));
    }

    #[test]
    fn test_multiple_deposits_accumulate() {
        let mut client = Client::new();
        let first_deposit = dec!(0.5000);
        let second_deposit = dec!(0.7500);

        client
            .process_monetary_request(
                TransactionId(1),
                MonetaryTransaction::Deposit(first_deposit),
            )
            .unwrap();

        client
            .process_monetary_request(
                TransactionId(2),
                MonetaryTransaction::Deposit(second_deposit),
            )
            .unwrap();

        assert_eq!(client.available, first_deposit + second_deposit);
        assert_eq!(client.transactions.len(), 2);
    }

    #[test]
    fn test_withdrawal_decreases_available_balance() {
        let mut client: Client = Client::new();
        let initial_deposit = dec!(1.0000);
        let withdrawal_amount = dec!(0.3000);

        client
            .process_monetary_request(
                TransactionId(1),
                MonetaryTransaction::Deposit(initial_deposit),
            )
            .unwrap();

        client
            .process_monetary_request(
                TransactionId(2),
                MonetaryTransaction::Withdrawal(withdrawal_amount),
            )
            .unwrap();

        assert_eq!(client.available, initial_deposit - withdrawal_amount);
        assert_eq!(client.transactions.len(), 2);
    }

    #[test]
    fn test_withdrawal_with_insufficient_funds_fails() {
        let mut client = Client::new();
        client.available = dec!(0.5);

        let withdrawal_request = client
            .process_monetary_request(TransactionId(2), MonetaryTransaction::Withdrawal(dec!(1.0)));

        assert!(matches!(
            withdrawal_request.unwrap_err(),
            ProcessTransactionError::InsufficientFunds
        ));
    }

    #[test]
    fn test_locked_client_cannot_process_transactions() {
        let mut client = Client::new();
        client.locked = true;

        let deposit_result = client
            .process_monetary_request(TransactionId(1), MonetaryTransaction::Deposit(dec!(10.00)));
        assert!(matches!(
            deposit_result.unwrap_err(),
            ProcessTransactionError::ClientLocked
        ));

        let withdrawal_result = client.process_monetary_request(
            TransactionId(2),
            MonetaryTransaction::Withdrawal(dec!(10.00)),
        );

        assert!(matches!(
            withdrawal_result.unwrap_err(),
            ProcessTransactionError::ClientLocked
        ));
        assert_eq!(client.available, Decimal::ZERO);
        assert!(client.transactions.is_empty());
    }

    #[test]
    fn test_zero_amount_deposit_and_withdrawal() {
        let mut client = Client::new();
        let deposit_result = client.process_monetary_request(
            TransactionId(1),
            MonetaryTransaction::Deposit(Decimal::ZERO),
        );

        assert!(deposit_result.is_ok());
        assert_eq!(client.available, Decimal::ZERO);

        let withdrawal_result = client.process_monetary_request(
            TransactionId(2),
            MonetaryTransaction::Withdrawal(Decimal::ZERO),
        );

        assert!(withdrawal_result.is_ok());
        assert_eq!(client.available, Decimal::ZERO);
    }
}

#[cfg(test)]
mod claim_tests {
    use super::*;
    use rust_decimal::{Decimal, dec};

    const TRANSACTION_ID: TransactionId = TransactionId(1);
    const DEPOSIT_AMOUNT: MonetaryAmount = dec!(100.00);

    fn create_client_with_deposit_transaction() -> Client {
        let mut client = Client::new();
        client
            .process_monetary_request(TRANSACTION_ID, MonetaryTransaction::Deposit(DEPOSIT_AMOUNT))
            .expect("Failed to process deposit");
        client
    }

    #[test]
    fn test_dispute_transaction() {
        let mut client = create_client_with_deposit_transaction();

        let dispute_result = client.process_claim(TRANSACTION_ID, ClaimType::Dispute);
        assert!(dispute_result.is_ok());
        assert_eq!(client.held, DEPOSIT_AMOUNT);
        assert_eq!(client.available, Decimal::ZERO);
    }

    #[test]
    fn test_resolve_disputed_transaction() {
        let mut client = create_client_with_deposit_transaction();

        client
            .process_claim(TRANSACTION_ID, ClaimType::Dispute)
            .unwrap();
        assert_eq!(client.held, DEPOSIT_AMOUNT);
        assert_eq!(client.available, Decimal::ZERO);

        let resolve_result = client.process_claim(TRANSACTION_ID, ClaimType::Resolve);
        assert!(resolve_result.is_ok());
        assert_eq!(client.held, Decimal::ZERO);
        assert_eq!(client.available, DEPOSIT_AMOUNT);
    }

    #[test]
    fn test_chargeback_disputed_transaction() {
        let mut client = create_client_with_deposit_transaction();

        client
            .process_claim(TRANSACTION_ID, ClaimType::Dispute)
            .unwrap();
        assert_eq!(client.held, DEPOSIT_AMOUNT);
        assert_eq!(client.available, Decimal::ZERO);
        assert!(!client.locked);

        let chargeback_result = client.process_claim(TRANSACTION_ID, ClaimType::Chargeback);
        assert!(chargeback_result.is_ok());
        assert_eq!(client.held, Decimal::ZERO);
        assert_eq!(client.available, Decimal::ZERO);
        assert!(client.locked);
    }

    #[test]
    fn test_resolve_or_chargeback_without_dispute_fails() {
        let mut client = create_client_with_deposit_transaction();

        let resolve_result = client.process_claim(TRANSACTION_ID, ClaimType::Resolve);
        assert!(matches!(
            resolve_result.unwrap_err(),
            ProcessTransactionError::InvalidOperation(_)
        ));

        let chargeback_result = client.process_claim(TRANSACTION_ID, ClaimType::Chargeback);
        assert!(matches!(
            chargeback_result.unwrap_err(),
            ProcessTransactionError::InvalidOperation(_)
        ));
    }

    #[test]
    fn test_dispute_on_disputed_transaction_fails() {
        let mut client = create_client_with_deposit_transaction();

        client
            .process_claim(TRANSACTION_ID, ClaimType::Dispute)
            .unwrap();

        let dispute_again_result = client.process_claim(TRANSACTION_ID, ClaimType::Dispute);
        assert!(matches!(
            dispute_again_result.unwrap_err(),
            ProcessTransactionError::InvalidOperation(_)
        ));
    }

    #[test]
    fn test_dispute_on_resolved_transaction() {
        let mut client = create_client_with_deposit_transaction();

        client
            .process_claim(TRANSACTION_ID, ClaimType::Dispute)
            .unwrap();
        client
            .process_claim(TRANSACTION_ID, ClaimType::Resolve)
            .unwrap();

        client
            .process_claim(TRANSACTION_ID, ClaimType::Dispute)
            .unwrap();
        assert_eq!(client.held, DEPOSIT_AMOUNT);
        assert_eq!(client.available, Decimal::ZERO);
    }

    #[test]
    fn test_chargeback_on_resolved_or_undisputed_transaction_fails() {
        let mut client = create_client_with_deposit_transaction();

        let chargeback_result = client.process_claim(TRANSACTION_ID, ClaimType::Chargeback);
        assert!(matches!(
            chargeback_result.unwrap_err(),
            ProcessTransactionError::InvalidOperation(_)
        ));

        client
            .process_claim(TRANSACTION_ID, ClaimType::Dispute)
            .unwrap();
        client
            .process_claim(TRANSACTION_ID, ClaimType::Resolve)
            .unwrap();

        let chargeback_result = client.process_claim(TRANSACTION_ID, ClaimType::Chargeback);
        assert!(matches!(
            chargeback_result.unwrap_err(),
            ProcessTransactionError::InvalidOperation(_)
        ));
    }

    #[test]
    fn test_claim_on_chargebacked_transaction_fails() {
        let mut client = create_client_with_deposit_transaction();
        client
            .process_claim(TRANSACTION_ID, ClaimType::Dispute)
            .unwrap();
        client
            .process_claim(TRANSACTION_ID, ClaimType::Chargeback)
            .unwrap();

        let chargeback_again_result = client.process_claim(TRANSACTION_ID, ClaimType::Chargeback);
        assert!(matches!(
            chargeback_again_result.unwrap_err(),
            ProcessTransactionError::ClientLocked
        ));

        let resolve_result = client.process_claim(TRANSACTION_ID, ClaimType::Resolve);
        assert!(matches!(
            resolve_result.unwrap_err(),
            ProcessTransactionError::ClientLocked
        ));

        let dispute_result = client.process_claim(TRANSACTION_ID, ClaimType::Dispute);
        assert!(matches!(
            dispute_result.unwrap_err(),
            ProcessTransactionError::ClientLocked
        ));
    }
}

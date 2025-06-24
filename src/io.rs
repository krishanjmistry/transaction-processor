use serde::{Deserialize, Serialize};

use crate::{
    error::{ProcessTransactionError, Result},
    types::{
        ClaimType, ClientId, MonetaryAmount, MonetaryTransaction, TransactionId,
        TransactionRequest, TransactionType,
    },
};

// Unfortunately, we can't use internally tagged enums on CSVs
// Hence, we can't directly deserialize into a type that better represents the fact that only a deposit and withdrawal require the amount field.
// https://github.com/BurntSushi/rust-csv/pull/231
#[derive(Debug, Deserialize)]
pub struct CsvRecord {
    #[serde(rename = "type")]
    transaction_type: CsvTransactionType,
    client: ClientId,
    #[serde(rename = "tx")]
    transaction: TransactionId,
    amount: Option<MonetaryAmount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CsvTransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

fn validate_amount(amount: Option<MonetaryAmount>) -> Result<MonetaryAmount> {
    let amount = amount.ok_or(ProcessTransactionError::InvalidData(
        "Amount is required for this transaction",
    ))?;

    // TODO: check case when amount is 0
    if amount.is_sign_positive() {
        Ok(amount.round_dp(4))
    } else {
        Err(ProcessTransactionError::InvalidData(
            "Amount must be positive",
        ))
    }
}

impl TryFrom<CsvRecord> for TransactionRequest {
    type Error = ProcessTransactionError;

    fn try_from(record: CsvRecord) -> std::result::Result<Self, Self::Error> {
        let request = match record.transaction_type {
            CsvTransactionType::Deposit => TransactionType::Monetary(MonetaryTransaction::Deposit(
                validate_amount(record.amount)?,
            )),
            CsvTransactionType::Withdrawal => TransactionType::Monetary(
                MonetaryTransaction::Withdrawal(validate_amount(record.amount)?),
            ),
            CsvTransactionType::Dispute => TransactionType::Claim(ClaimType::Dispute),
            CsvTransactionType::Resolve => TransactionType::Claim(ClaimType::Resolve),
            CsvTransactionType::Chargeback => TransactionType::Claim(ClaimType::Chargeback),
        };

        Ok(TransactionRequest::new(
            record.client,
            record.transaction,
            request,
        ))
    }
}

#[derive(Debug, Serialize)]
pub struct OutputCsvRecord {
    #[serde(rename = "client")]
    pub client_id: ClientId,
    pub available: MonetaryAmount,
    pub held: MonetaryAmount,
    pub total: MonetaryAmount,
    pub locked: bool,
}

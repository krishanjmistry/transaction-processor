use serde::{Deserialize, Serialize};

use crate::{
    error::{ProcessTransactionError, Result},
    types::{
        ClaimType, ClientId, MonetaryAmount, MonetaryTransaction, RequestType, TransactionId,
        TransactionRequest,
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
            CsvTransactionType::Deposit => RequestType::Monetary(MonetaryTransaction::Deposit(
                validate_amount(record.amount)?,
            )),
            CsvTransactionType::Withdrawal => RequestType::Monetary(
                MonetaryTransaction::Withdrawal(validate_amount(record.amount)?),
            ),
            CsvTransactionType::Dispute => RequestType::Claim(ClaimType::Dispute),
            CsvTransactionType::Resolve => RequestType::Claim(ClaimType::Resolve),
            CsvTransactionType::Chargeback => RequestType::Claim(ClaimType::Chargeback),
        };

        Ok(TransactionRequest {
            client: record.client,
            transaction: record.transaction,
            request_type: request,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::{Decimal, dec};

    #[test]
    fn test_validate_amount() {
        // Standard valid decimal
        assert_eq!(
            validate_amount(Some(dec!(100.1234))).unwrap(),
            dec!(100.1234)
        );
        // Valid decimal with more than 4 decimal places has rounding applied using banker's rounding
        assert_eq!(
            validate_amount(Some(dec!(100.12345))).unwrap(),
            dec!(100.1234)
        );
        assert_eq!(
            validate_amount(Some(dec!(100.12343))).unwrap(),
            dec!(100.1234)
        );
        assert_eq!(
            validate_amount(Some(dec!(100.12346))).unwrap(),
            dec!(100.1235)
        );
        assert_eq!(validate_amount(Some(dec!(0.00001))).unwrap(), dec!(0.0000));
        // Zero is accepted as valid
        assert_eq!(validate_amount(Some(dec!(0.0))).unwrap(), Decimal::ZERO);
        // Negative zero is treated as zero
        assert_eq!(validate_amount(Some(dec!(-0))).unwrap(), Decimal::ZERO);
        assert!(validate_amount(Some(dec!(-100.0))).is_err());
        assert!(validate_amount(None).is_err());
    }
}

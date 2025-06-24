use csv::ReaderBuilder;
use serde::Deserialize;

use crate::{
    error::ProcessTransactionError,
    types::{
        ClaimType, ClientId, MonetaryAmount, TransactionId, TransactionRequest, TransactionType,
    },
};

mod error;
mod exchange;
mod types;

fn main() {
    if std::env::args().len() != 2 {
        eprintln!("Usage: cargo run -- /path/to/file.csv");
        std::process::exit(1);
    }

    let file_path = std::env::args().nth(1).expect("No file path provided");

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_path(file_path)
        .expect("Failed to create CSV reader");

    for record in rdr.deserialize() {
        let record: CsvRecord = record.expect("Failed to read record");
        println!("{:?}", record);

        match TransactionRequest::try_from(record) {
            Ok(request) => println!("Parsed request: {:?}", request),
            Err(e) => eprintln!("Error parsing record: {}", e),
        }
    }
}

// Unfortunately, we can't use internally tagged enums on CSVs
// Hence, we can't directly deserialize into a type that better represents the fact that only a deposit and withdrawal require the amount field.
// https://github.com/BurntSushi/rust-csv/pull/231
#[derive(Debug, Deserialize)]
struct CsvRecord {
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

impl TryFrom<CsvRecord> for TransactionRequest {
    type Error = ProcessTransactionError;

    fn try_from(record: CsvRecord) -> Result<Self, Self::Error> {
        let request = match record.transaction_type {
            CsvTransactionType::Deposit => TransactionType::Deposit(
                record
                    .amount
                    .ok_or(ProcessTransactionError::InvalidData(
                        "Deposit requires an amount",
                    ))?
                    .round_dp(4),
            ),
            CsvTransactionType::Withdrawal => TransactionType::Withdrawal(
                record
                    .amount
                    .ok_or(ProcessTransactionError::InvalidData(
                        "Withdrawal requires an amount",
                    ))?
                    .round_dp(4),
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

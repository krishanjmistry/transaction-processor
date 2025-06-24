use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::error::ProcessTransactionError;

mod error;

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

#[derive(Debug, Deserialize)]
struct ClientId(u16);

#[derive(Debug, Deserialize)]
struct TransactionId(u32);

type MonetaryAmount = Decimal;

#[derive(Debug)]
struct TransactionRequest {
    client: ClientId,
    transaction: TransactionId,
    request: TransactionType,
}

#[derive(Debug)]
enum TransactionType {
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

impl TryFrom<CsvRecord> for TransactionRequest {
    type Error = ProcessTransactionError;

    fn try_from(record: CsvRecord) -> Result<Self, Self::Error> {
        let request = match record.transaction_type {
            CsvTransactionType::Deposit => TransactionType::Deposit(record.amount.ok_or(
                ProcessTransactionError::InvalidData("Deposit requires an amount"),
            )?),
            CsvTransactionType::Withdrawal => TransactionType::Withdrawal(record.amount.ok_or(
                ProcessTransactionError::InvalidData("Withdrawal requires an amount"),
            )?),
            CsvTransactionType::Dispute => TransactionType::Claim(ClaimType::Dispute),
            CsvTransactionType::Resolve => TransactionType::Claim(ClaimType::Resolve),
            CsvTransactionType::Chargeback => TransactionType::Claim(ClaimType::Chargeback),
        };

        Ok(TransactionRequest {
            client: record.client,
            transaction: record.transaction,
            request,
        })
    }
}

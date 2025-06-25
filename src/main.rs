use csv::{ReaderBuilder, WriterBuilder};

use crate::{
    exchange::Exchange,
    io::{CsvRecord, OutputCsvRecord},
    types::TransactionRequest,
};

mod error;
mod exchange;
mod io;
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

    let mut exchange = Exchange::new();

    for record in rdr.deserialize() {
        let record: CsvRecord = record.expect("Failed to read record");

        let transaction_request = TransactionRequest::try_from(record)
            .expect("Failed to convert record to TransactionRequest");

        exchange
            .process_transaction(transaction_request)
            .unwrap_or_else(|e| {
                eprintln!("Error processing transaction: {}", e);
            });
    }

    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(std::io::stdout());

    let clients = exchange.get_clients();

    clients.iter().for_each(|(client_id, client)| {
        let output_record = OutputCsvRecord {
            client_id: *client_id,
            available: client.available,
            held: client.held,
            total: client.available + client.held,
            locked: client.locked,
        };
        wtr.serialize(output_record)
            .expect("Failed to write record");
    });
}

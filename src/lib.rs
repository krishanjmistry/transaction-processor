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

pub fn process<R: std::io::Read, W: std::io::Write>(rdr: R, wtr: W) {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(rdr);

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

    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(wtr);

    let clients = exchange.get_clients();

    // Sort clients by client_id for deterministic output
    let mut sorted_clients = clients.iter().collect::<Vec<_>>();
    sorted_clients.sort_by_key(|(client_id, _)| *client_id);

    for (client_id, client) in sorted_clients {
        let output_record = OutputCsvRecord {
            client_id: *client_id,
            available: client.available,
            held: client.held,
            total: client.available + client.held,
            locked: client.locked,
        };
        wtr.serialize(output_record)
            .expect("Failed to write record");
    }
}

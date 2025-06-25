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

    // Sort clients by client_id for deterministic output.
    // Could have instead used a BTreeMap in the exchange to maintain a sorted map but that reduces performance.
    // Also could use the `indexmap` crate for a map that maintains insertion order.
    let mut sorted_clients = clients.iter().collect::<Vec<_>>();
    sorted_clients.sort_by_key(|(client_id, _)| *client_id);

    // Ensure headers are written even if no records exist
    if sorted_clients.is_empty() {
        wtr.write_record(["client", "available", "held", "total", "locked"])
            .expect("Failed to write headers");
    } else {
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

    wtr.flush().expect("Failed to flush CSV writer");
}

use csv::ReaderBuilder;

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

    for record in rdr.records() {
        let record = record.expect("Failed to read record");
        println!("{:?}", record);
    }
}
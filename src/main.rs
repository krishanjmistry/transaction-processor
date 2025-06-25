use transaction_processor::process;

fn main() {
    if std::env::args().len() != 2 {
        eprintln!("Usage: cargo run -- /path/to/file.csv");
        std::process::exit(1);
    }

    let file_path = std::env::args().nth(1).expect("No file path provided");
    let reader = std::fs::File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path));

    process(reader, std::io::stdout());
}

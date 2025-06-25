use std::fs::File;
use transaction_processor::process;

fn test_handler(file_name: &str) {
    let input_file =
        File::open(format!("tests/input/{}.csv", file_name)).expect("Failed to open input file");
    let mut output = Vec::new();

    process(input_file, &mut output);

    let output_str = String::from_utf8(output).expect("Invalid UTF-8 output");

    let expected_output_str = std::fs::read_to_string(format!("tests/output/{}.csv", file_name))
        .expect("Failed to read expected output file");

    assert_eq!(output_str, expected_output_str);
}

#[test]
fn test_single_client_transactions() {
    test_handler("single_client");
}

#[test]
fn test_multiple_client() {
    test_handler("multiple_client");
}

#[test]
fn test_client_created_on_withdrawal() {
    test_handler("client_created_on_withdrawal");
}

#[test]
fn test_dispute_chargeback() {
    test_handler("dispute_chargeback");
}

#[test]
fn test_dispute() {
    test_handler("dispute");
}

#[test]
fn test_dispute_resolve() {
    test_handler("dispute_resolve");
}

#[test]
fn test_empty_input() {
    test_handler("empty_transactions");
}

#[test]
fn test_no_deposits_or_withdrawals() {
    test_handler("no_deposits_or_withdrawals");
}

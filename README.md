# A mini transaction processor

Implementation of a mini transaction processor

## Usage

Can be built via:

```shell
cargo build
```

Executable via:

```shell
cargo run -- <path/to/file.csv>
```

## Design

The exchange maintains two databases which are implemented as Rust standard library hashmaps.

1. Clients - for every client, store the state of their account
2. Transactions - a ledger of all identifiers for valid transactions that have occurred. Transaction ID's are globally unique

In a real world system which is distributed, I'd expect the transactions database to act like a distributed lock that is held whilst a monetary transaction is being made.

### Error handling
Errors in the exchange are handled by propagating back up to the client application in lib.rs where they are printed to STDERR. The exchange itself should be `panic` free with errors being recoverable.

## Assumptions

- A deposit or withdrawal is the only way a new client can be created
- In the case of a withdrawal for the first transaction, we should register the client into the system but then error on withdrawal as there aren't any funds. Think of it like a registration form when you sign up for a service.
- A claim (dispute/resolve/chargeback) is handled differently for deposits and withdrawals (see below table)

| claim      | deposit                                                                 | withdrawal            |
| ---------- | ----------------------------------------------------------------------- | --------------------- |
| dispute    | ring-fence funds into held (allowed to turn available balance negative) | nothing               |
| resolve    | release held money to available funds                                   | nothing               |
| chargeback | exchange takes hold of held funds                                       | exchange credits user |

- The only time a client's available balance is allowed to turn negative is in the case of a dispute

## Things I've learnt

Banker's rounding: The `rust_decimal` crate rounds numbers by default using this strategy. https://docs.rs/rust_decimal/1.37.2/rust_decimal/struct.Decimal.html#method.round_dp

## Things that could be improved

- Error enum should be reviewed and potentially simplified
- Currently, whilst the CSV is streamed, every transaction on the exchange is sequential. It should be possible to allow different clients to operate independently, with some thought on how to avoid contention checking the transaction id.
- Better type/handling in the exchange for a monetary value given in a deposit/withdrawal to ensure it is positive. Current validation only occurs during deserialization.
- The writing logic in the process handler in lib.rs could be separated out into separate functionality 

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

The exchange maintains two databases (implemented as hashmaps).

1. Clients - for every client, store the state of their account
2. Transactions - a ledger of all identifiers for valid transactions that have occurred. Transaction ID's are globally unique

## Assumptions

- A deposit or withdrawal is the only way a new client can be created
- In the case of a withdrawal for the first transaction, we should register the client into the system but then error on withdrawal as there aren't any funds
- A claim (dispute/resolve/chargeback) is handled differently for deposits and withdrawals (see below table)

| claim      | deposit                                                                 | withdrawal            |
| ---------- | ----------------------------------------------------------------------- | --------------------- |
| dispute    | ring-fence funds into held (allowed to turn available balance negative) | nothing               |
| resolve    | release held money to available funds                                   | nothing               |
| chargeback | exchange takes hold of held funds                                       | exchange credits user |

- The only time a client's available balance is allowed to turn negative is in the case of a dispute

## Things I've learnt

Banker's rounding: The `rust_decimal` crate rounds numbers by default using this strategy. https://docs.rs/rust_decimal/1.37.2/rust_decimal/struct.Decimal.html#method.round_dp

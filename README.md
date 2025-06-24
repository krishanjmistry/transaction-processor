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

## Assumptions

- A deposit or withdrawal is the only way a new client can be created
  - In the case of a withdrawal for the first transaction, we should register the client into the system but then error on withdrawal as there aren't any funds

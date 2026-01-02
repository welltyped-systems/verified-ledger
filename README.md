# Verified Ledger: Model-Based Conformance Testing Architecture

[<img src="https://welltyped.systems/img/badge.png" alt="Welltyped Systems Badge" width="150" href/>](https://welltyped.systems)

This repo contains a minimal reference architecture for model-based conformance testing of critical systems. It includes a formally verified Lean 4 model of a simple ledger system, an intentionally buggy Rust implementation, and a differential fuzzing harness that tests the Rust code against the Lean model.

## Differential testing for correctness and security

Our approach to auditing correctness and security at [Welltyped Systems](https://welltyped.systems) is based on formal verification. We develop
a simplified formal model of the target system/subsystem in a proof assistant language (current Lean 4) and prove that the model satisfies the desired invariants. From this, we construct a fuzzing / property-based testing framework that differentially tests the target system against the formally verified model, identifying any divergences. By integrating these tests into CI/CD pipelines, we can continuously monitor the correctness of the implementation as it evolves.

## Ledger model

The model is a simple ledger with accounts identified by strings and balances represented as `UInt64`. It supports three operations:

- `deposit(account: String, amount: UInt64)`: Increases the balance of `account` by `amount`.
- `withdraw(account: String, amount: UInt64)`: Decreases the balance of `account` by `amount` if sufficient funds exist; otherwise, it fails.
- `transfer(from: String, to: String, amount: UInt64)`: Moves `amount` from `from` to `to` if `from` has sufficient funds; otherwise, it fails.

## Repo structure

- `lean/`: Contains the Lean 4 model, FFI bindings, and proofs.
- `src/`: Contains the Rust implementation of the ledger and the differential fuzzing harness.

## Intentional Rust bugs

- `withdraw` rejects withdrawals that exactly equal the balance (uses `<=` instead of `<`).
- `transfer` withdraws and then deposits back into the sender account (never credits the recipient).

## Lean proofs

The executable model can be found in `lean/VerifiedLedger/Model.lean`, and the
FFI entrypoints used by Rust are in `lean/VerifiedLedger/FFI.lean`. The
specification and proofs live in `lean/VerifiedLedger/Proofs.lean`, with the following theorems/lemmas proved:

- Deposit:
  - `apply_deposit_balance_self`: The target account increases by `amount`.
  - `apply_deposit_balance_other`: All other accounts are unchanged.
- Withdraw:
  - `apply_withdraw_fail`: If the balance is insufficient, `apply` returns an error tagged with the account name.
  - `apply_withdraw_ok_balance_self`: With sufficient funds, the account decreases by `amount`.
  - `apply_withdraw_ok_balance_other`: With sufficient funds, all other accounts are unchanged.
- Transfer:
  - `apply_transfer_fail`: If the sender lacks funds, `apply` returns an error tagged with the sender.
  - `apply_transfer_ok_balance_from`: On success, the sender decreases by `amount` (when `from != to`).
  - `apply_transfer_ok_balance_to`: On success, the recipient increases by `amount` (when `from != to`).
  - `apply_transfer_ok_balance_other`: On success, all unrelated accounts are unchanged.
  - `apply_transfer_ok_same_account`: A self-transfer with sufficient funds is a no-op on that account’s balance.

All arithmetic is over `UInt64`, so the proofs reflect `UInt64` wrap semantics.

## Running the tests

To run the differential fuzz tests, ensure you have Rust, Lean 4, and a C compiler installed on your system. Then, execute the following command in the terminal:

```
cargo run
```

The Rust build will invoke `lake` to compile the Lean model to C and link it into the
binary, so make sure `lake` is on your PATH (it is installed via `elan`).

### CLI options

```
cargo run -- fuzz --cases 50 --steps 200 --max-amount 50
cargo run -- replay --seed 12345 --steps 200
```

### Build the Lean model standalone

The Lean model and proofs are built implicitly by the Rust build process, but you can also build it manually:

```
cd lean
lake build VerifiedLedger
```

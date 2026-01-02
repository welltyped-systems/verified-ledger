# Verified Ledger: Differential Testing with a Formally Verified Model

<img src="https://welltyped.systems/img/badge.png" alt="Welltyped Systems Badge" width="150"/>
A sample project demonstrating the usage of formally verified models as oracles for differential testing, focusing on a simple ledger system implemented in Rust with a Lean 4 model.

## Differential testing for correctness and security

Our approach to auditing correctness and security at [Welltyped Systems](https://welltyped.systems) is based on formal verification. We develop
a simplified formal model of the target system/subsystem in a proof assistant language (current Lean 4) and prove that the model satisfies the desired invariants. From this, we construct a fuzzing / property-based testing framework that differentially tests the target system against the formally verified model, identifying any divergences. By integrating these tests into CI/CD pipelines, we can continuously monitor the correctness of the implementation as it evolves.

## What this repo contains

Whilst this repo is a toy example, it illustrates the core components of differential testing with a formally verified model. It includes:

- `/lean`: A simple Lean 4 model of a ledger system, alongisde proofs of key invariants.
- `/src/ledger.rs`: An intentionally buggy Rust implementation of the same ledger system.
- `/src/fuzz.rs`: A differential fuzzing harness that generates deterministically seeded pseudorandom sequences of ledger operations, and applies them to both the Lean model and the Rust implementation.

In case of divergence, the harness reports the sequence of operations and states that led to the mismatch, enabling easy reproduction and debugging.

### Intentional Rust bugs

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

### Build the Lean model

The Lean model and proofs are built implicitly by the Rust build process, but you can also build it manually:

```
cd lean
lake build VerifiedLedger
```

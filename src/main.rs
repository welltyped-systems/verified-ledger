mod fuzz;
mod lean_model;
mod ledger;
mod op;

use std::env;
use std::process;

use crate::fuzz::{seed_from_time, FuzzConfig};

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next();

    match command.as_deref() {
        None | Some("fuzz") => run_fuzz(args, false),
        Some("replay") => run_fuzz(args, true),
        Some("help") | Some("-h") | Some("--help") => {
            print_help();
        }
        Some(other) => {
            eprintln!("unknown command: {}", other);
            print_help();
            process::exit(2);
        }
    }
}

fn run_fuzz(args: impl Iterator<Item = String>, replay: bool) {
    let seed = seed_from_time();
    let config = FuzzConfig::new(seed);

    let mut config = match parse_flags(args, config) {
        Ok(config) => config,
        Err(ParseOutcome::Help) => {
            print_help();
            return;
        }
        Err(ParseOutcome::Error(message)) => {
            eprintln!("{}", message);
            print_help();
            process::exit(2);
        }
    };

    if replay {
        config.cases = 1;
    }

    println!(
        "\nrunning fuzz: seed={}, cases={}, steps={}, max_amount={}",
        config.seed, config.cases, config.steps, config.max_amount
    );

    match fuzz::run(&config) {
        Ok(()) => {
            println!("no mismatches found");
        }
        Err(failure) => {
            eprintln!("{}", failure);
            process::exit(1);
        }
    }
}

enum ParseOutcome {
    Help,
    Error(String),
}

fn parse_flags(
    mut args: impl Iterator<Item = String>,
    mut config: FuzzConfig,
) -> Result<FuzzConfig, ParseOutcome> {
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| ParseOutcome::Error("missing value for --seed".to_string()))?;
                config.seed = parse_u64("--seed", &value)?;
            }
            "--cases" => {
                let value = args
                    .next()
                    .ok_or_else(|| ParseOutcome::Error("missing value for --cases".to_string()))?;
                config.cases = parse_u64("--cases", &value)?;
            }
            "--steps" => {
                let value = args
                    .next()
                    .ok_or_else(|| ParseOutcome::Error("missing value for --steps".to_string()))?;
                config.steps = parse_usize("--steps", &value)?;
            }
            "--max-amount" => {
                let value = args.next().ok_or_else(|| {
                    ParseOutcome::Error("missing value for --max-amount".to_string())
                })?;
                config.max_amount = parse_u64("--max-amount", &value)?;
            }
            "--help" | "-h" => {
                return Err(ParseOutcome::Help);
            }
            other => {
                return Err(ParseOutcome::Error(format!("unknown flag: {}", other)));
            }
        }
    }

    Ok(config)
}

fn parse_u64(flag: &str, value: &str) -> Result<u64, ParseOutcome> {
    value
        .parse::<u64>()
        .map_err(|_| ParseOutcome::Error(format!("invalid value for {}: {}", flag, value)))
}

fn parse_usize(flag: &str, value: &str) -> Result<usize, ParseOutcome> {
    value
        .parse::<usize>()
        .map_err(|_| ParseOutcome::Error(format!("invalid value for {}: {}", flag, value)))
}

fn print_help() {
    println!("verified-ledger: differential fuzzing harness");
    println!("");
    println!("Usage:");
    println!("  cargo run -- fuzz [--seed N] [--cases N] [--steps N] [--max-amount N]");
    println!("  cargo run -- replay --seed N [--steps N] [--max-amount N]");
    println!("");
    println!("Defaults:");
    println!("  --seed       random from system time");
    println!("  --cases      50");
    println!("  --steps      200");
    println!("  --max-amount 50");
}

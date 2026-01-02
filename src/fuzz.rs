use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lean_model::LeanLedger;
use crate::ledger::Ledger;
use crate::op::{ApplyError, Op};

#[derive(Clone, Debug)]
pub struct FuzzConfig {
    pub seed: u64,
    pub cases: u64,
    pub steps: usize,
    pub max_amount: u64,
}

impl FuzzConfig {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            cases: 50,
            steps: 200,
            max_amount: 50,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FuzzFailure {
    pub seed: u64,
    pub case_index: u64,
    pub step_index: usize,
    pub op: Op,
    pub model_result: Result<(), ApplyError>,
    pub impl_result: Result<(), ApplyError>,
    pub model_pre_snapshot: Vec<(String, u64)>,
    pub impl_pre_snapshot: Vec<(String, u64)>,
    pub history: Vec<StepRecord>,
}

impl fmt::Display for FuzzFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\nmismatch detected (case={}, step={}, seed={})",
            self.case_index, self.step_index, self.seed
        )?;
        writeln!(f, "----------------------------------------")?;
        writeln!(f, "op            : {}", self.op)?;
        match &self.model_result {
            Ok(()) => writeln!(f, "model         : {:?}", self.model_pre_snapshot)?,
            Err(e) => writeln!(f, "model         : Err({:?})", e)?,
        }
        match &self.impl_result {
            Ok(()) => writeln!(f, "impl          : {:?}", self.impl_pre_snapshot)?,
            Err(e) => writeln!(f, "impl          : Err({:?})", e)?,
        }
        writeln!(
            f,
            "replay        : cargo run -- replay --seed {} --steps {}",
            self.seed,
            self.history.len()
        )?;
        writeln!(f, "history:")?;
        if let Some(_first) = self.history.first() {
            writeln!(f, "  0: <initial>")?;
            writeln!(f, "     model: {:?}", self.model_pre_snapshot)?;
            writeln!(f, "     impl : {:?}", self.impl_pre_snapshot)?;
        }
        for (i, step) in self.history.iter().enumerate() {
            let index = i + 1;
            writeln!(f, "  {}: {}", index, step.op)?;
            match &step.model_result {
                Ok(()) => writeln!(f, "     model: {:?}", step.model_snapshot)?,
                Err(e) => writeln!(f, "     model: Err({:?})", e)?,
            }
            match &step.impl_result {
                Ok(()) => writeln!(f, "     impl : {:?}", step.impl_snapshot)?,
                Err(e) => writeln!(f, "     impl : Err({:?})", e)?,
            }
        }
        Ok(())
    }
}

pub fn seed_from_time() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs() ^ ((now.subsec_nanos() as u64) << 32)
}

pub fn run(config: &FuzzConfig) -> Result<(), FuzzFailure> {
    for case_index in 0..config.cases {
        let case_seed = config.seed.wrapping_add(case_index);
        if let Err(failure) = run_case(config, case_index, case_seed) {
            return Err(failure);
        }
    }
    Ok(())
}

fn run_case(config: &FuzzConfig, case_index: u64, seed: u64) -> Result<(), FuzzFailure> {
    let (mut init_rng, mut rng) = make_rng_streams(seed);
    let accounts = default_accounts();

    let mut model = LeanLedger::new();
    let mut ledger = Ledger::new();
    let mut history = Vec::with_capacity(config.steps);

    seed_initial_balances(
        &mut model,
        &mut ledger,
        &accounts,
        config.max_amount,
        &mut init_rng,
    );

    for step_index in 0..config.steps {
        let model_pre_snapshot = model.snapshot(&accounts);
        let impl_pre_snapshot = ledger.snapshot();

        let op = random_op(&mut rng, &accounts, config.max_amount);

        let model_result = model.apply(&op);
        let impl_result = ledger.apply(&op);

        let model_snapshot = model.snapshot(&accounts);
        let impl_snapshot = ledger.snapshot();

        history.push(StepRecord {
            op: op.clone(),
            model_result: model_result.clone(),
            impl_result: impl_result.clone(),
            model_snapshot: model_snapshot.clone(),
            impl_snapshot: impl_snapshot.clone(),
        });

        let results_match = model_result == impl_result;
        let state_match = model_snapshot == impl_snapshot;

        if !results_match || !state_match {
            return Err(FuzzFailure {
                seed,
                case_index,
                step_index,
                op,
                model_result,
                impl_result,
                model_pre_snapshot,
                impl_pre_snapshot,
                history,
            });
        }
    }

    Ok(())
}

fn random_op(rng: &mut XorShift64, accounts: &[String], max_amount: u64) -> Op {
    let roll = rng.next_u64() % 100;
    let amount = 1 + rng.next_u64() % max_amount.max(1);

    if roll < 45 {
        let account = pick_account(rng, accounts).clone();
        Op::Deposit { account, amount }
    } else if roll < 75 {
        let account = pick_account(rng, accounts).clone();
        Op::Withdraw { account, amount }
    } else {
        let from = pick_account(rng, accounts).clone();
        let mut to = pick_account(rng, accounts).clone();
        if from == to {
            to = pick_different_account(rng, accounts, &from).clone();
        }
        Op::Transfer { from, to, amount }
    }
}

#[derive(Clone, Debug)]
pub struct StepRecord {
    pub op: Op,
    pub model_result: Result<(), ApplyError>,
    pub impl_result: Result<(), ApplyError>,
    pub model_snapshot: Vec<(String, u64)>,
    pub impl_snapshot: Vec<(String, u64)>,
}

fn pick_account<'a>(rng: &mut XorShift64, accounts: &'a [String]) -> &'a String {
    let index = (rng.next_u64() % accounts.len() as u64) as usize;
    &accounts[index]
}

fn pick_different_account<'a>(
    rng: &mut XorShift64,
    accounts: &'a [String],
    current: &str,
) -> &'a String {
    for _ in 0..accounts.len() {
        let candidate = pick_account(rng, accounts);
        if candidate != current {
            return candidate;
        }
    }
    &accounts[0]
}

// xorshift prng
#[derive(Clone, Debug)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let seed = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

fn make_rng_streams(seed: u64) -> (XorShift64, XorShift64) {
    //separate streams for balance and ops, need to decorrelate
    let init_seed = mix_seed(seed, 0x53a9_e5b1_6f1d_6b29);
    let op_seed = mix_seed(seed, 0xa5a3_98d7_612c_e4b5);
    (XorShift64::new(init_seed), XorShift64::new(op_seed))
}

fn mix_seed(seed: u64, stream: u64) -> u64 {
    let mut z = seed.wrapping_add(stream.wrapping_mul(0x9e3779b97f4a7c15));
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn seed_initial_balances(
    model: &mut LeanLedger,
    ledger: &mut Ledger,
    accounts: &[String],
    max_amount: u64,
    rng: &mut XorShift64,
) {
    let bound = max_amount.max(1);
    for account in accounts {
        let amount = rng.next_u64() % bound.saturating_add(1);
        if amount == 0 {
            continue;
        }
        let op = Op::Deposit {
            account: account.clone(),
            amount,
        };
        model
            .apply(&op)
            .expect("Lean model seeding deposit should not fail");
        ledger
            .apply(&op)
            .expect("ledger seeding deposit should not fail");
    }
}

fn default_accounts() -> Vec<String> {
    vec![
        "alice".to_string(),
        "bob".to_string(),
        "carol".to_string(),
        "dave".to_string(),
        "erin".to_string(),
    ]
}

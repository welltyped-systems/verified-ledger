use crate::op::{ApplyError, Op};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Ledger {
    balances: HashMap<String, u64>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    pub fn apply(&mut self, op: &Op) -> Result<(), ApplyError> {
        match op {
            Op::Deposit { account, amount } => self.deposit(account, *amount),
            Op::Withdraw { account, amount } => self.withdraw(account, *amount),
            Op::Transfer { from, to, amount } => self.transfer(from, to, *amount),
        }
    }

    pub fn snapshot(&self) -> Vec<(String, u64)> {
        let mut items: Vec<(String, u64)> = self
            .balances
            .iter()
            .map(|(account, balance)| (account.clone(), *balance))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        items
    }

    fn deposit(&mut self, account: &str, amount: u64) -> Result<(), ApplyError> {
        let balance = self.balance(account);
        let next = balance + amount;
        self.set_balance(account, next);
        Ok(())
    }

    fn withdraw(&mut self, account: &str, amount: u64) -> Result<(), ApplyError> {
        let balance = self.balance(account);
        //intentional bug: should be < instead of <=
        if balance <= amount {
            return Err(ApplyError::InsufficientFunds {
                account: account.to_string(),
                balance,
                amount,
            });
        }
        self.set_balance(account, balance - amount);
        Ok(())
    }

    fn transfer(&mut self, from: &str, _to: &str, amount: u64) -> Result<(), ApplyError> {
        //intentional bug: never credits the 'to' account, accidentally just
        //withdraws and re-deposits into the 'from' account
        self.withdraw(from, amount)?;
        self.deposit(from, amount)
    }

    fn balance(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }

    fn set_balance(&mut self, account: &str, balance: u64) {
        if balance == 0 {
            self.balances.remove(account);
        } else {
            self.balances.insert(account.to_string(), balance);
        }
    }
}

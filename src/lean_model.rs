use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::sync::Once;

use crate::op::{ApplyError, Op};

extern "C" {
    fn ledger_lean_initialize();
    fn ledger_lean_state_new() -> *mut c_void;
    fn ledger_lean_state_dec(state: *mut c_void);
    fn ledger_lean_apply_deposit(
        state: *mut c_void,
        account: *const c_char,
        amount: u64,
        ok: *mut u8,
    ) -> *mut c_void;
    fn ledger_lean_apply_withdraw(
        state: *mut c_void,
        account: *const c_char,
        amount: u64,
        ok: *mut u8,
    ) -> *mut c_void;
    fn ledger_lean_apply_transfer(
        state: *mut c_void,
        from_account: *const c_char,
        to_account: *const c_char,
        amount: u64,
        ok: *mut u8,
    ) -> *mut c_void;
    fn ledger_lean_balance(state: *mut c_void, account: *const c_char) -> u64;
}

static INIT: Once = Once::new();

fn ensure_initialized() {
    INIT.call_once(|| unsafe {
        ledger_lean_initialize();
    });
}

pub struct LeanLedger {
    state: *mut c_void,
}

impl LeanLedger {
    pub fn new() -> Self {
        ensure_initialized();
        let state = unsafe { ledger_lean_state_new() };
        Self { state }
    }

    pub fn apply(&mut self, op: &Op) -> Result<(), ApplyError> {
        let mut ok = 0u8;
        let new_state = match op {
            Op::Deposit { account, amount } => {
                let account = CString::new(account.as_str()).expect("account contains NUL");
                unsafe { ledger_lean_apply_deposit(self.state, account.as_ptr(), *amount, &mut ok) }
            }
            Op::Withdraw { account, amount } => {
                let account = CString::new(account.as_str()).expect("account contains NUL");
                unsafe {
                    ledger_lean_apply_withdraw(self.state, account.as_ptr(), *amount, &mut ok)
                }
            }
            Op::Transfer { from, to, amount } => {
                let from_account = CString::new(from.as_str()).expect("from contains NUL");
                let to_account = CString::new(to.as_str()).expect("to contains NUL");
                unsafe {
                    ledger_lean_apply_transfer(
                        self.state,
                        from_account.as_ptr(),
                        to_account.as_ptr(),
                        *amount,
                        &mut ok,
                    )
                }
            }
        };

        self.state = new_state;

        if ok == 1 {
            Ok(())
        } else {
            match op {
                Op::Deposit { account, amount } => Err(ApplyError::InsufficientFunds {
                    account: account.clone(),
                    balance: self.balance(account),
                    amount: *amount,
                }),
                Op::Withdraw { account, amount } => Err(ApplyError::InsufficientFunds {
                    account: account.clone(),
                    balance: self.balance(account),
                    amount: *amount,
                }),
                Op::Transfer { from, amount, .. } => Err(ApplyError::InsufficientFunds {
                    account: from.clone(),
                    balance: self.balance(from),
                    amount: *amount,
                }),
            }
        }
    }

    pub fn snapshot(&self, accounts: &[String]) -> Vec<(String, u64)> {
        let mut items: Vec<(String, u64)> = accounts
            .iter()
            .filter_map(|account| {
                let balance = self.balance(account);
                if balance == 0 {
                    None
                } else {
                    Some((account.clone(), balance))
                }
            })
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        items
    }

    pub fn balance(&self, account: &str) -> u64 {
        let account = CString::new(account).expect("account contains NUL");
        unsafe { ledger_lean_balance(self.state, account.as_ptr()) }
    }
}

impl Drop for LeanLedger {
    fn drop(&mut self) {
        unsafe {
            ledger_lean_state_dec(self.state);
        }
    }
}

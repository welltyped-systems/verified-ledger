use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Deposit {
        account: String,
        amount: u64,
    },
    Withdraw {
        account: String,
        amount: u64,
    },
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    InsufficientFunds {
        account: String,
        balance: u64,
        amount: u64,
    },
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplyError::InsufficientFunds {
                account,
                balance,
                amount,
            } => write!(
                f,
                "insufficient funds: account={}, balance={}, amount={}",
                account, balance, amount
            ),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Deposit { account, amount } => {
                write!(f, "deposit({}, {})", account, amount)
            }
            Op::Withdraw { account, amount } => {
                write!(f, "withdraw({}, {})", account, amount)
            }
            Op::Transfer { from, to, amount } => {
                write!(f, "transfer({}, {}, {})", from, to, amount)
            }
        }
    }
}

--
-- Contains FFI functions for interacting with the model from the harness
--

import VerifiedLedger.Model

namespace VerifiedLedger

structure ApplyResult where
  ok : Bool
  state : State
  deriving Repr

def applyOp (s : State) (op : Op) : ApplyResult :=
  match apply s op with
  | .ok s' => { ok := true, state := s' }
  | .error _ => { ok := false, state := s }

@[export verified_ledger_empty]
def ffiEmpty : State :=
  empty

@[export verified_ledger_apply_deposit]
def ffiApplyDeposit (s : State) (account : String) (amount : UInt64) : ApplyResult :=
  applyOp s (Op.deposit account amount)

@[export verified_ledger_apply_withdraw]
def ffiApplyWithdraw (s : State) (account : String) (amount : UInt64) : ApplyResult :=
  applyOp s (Op.withdraw account amount)

@[export verified_ledger_apply_transfer]
def ffiApplyTransfer (s : State) (fromAccount : String) (toAccount : String) (amount : UInt64) : ApplyResult :=
  applyOp s (Op.transfer fromAccount toAccount amount)

@[export verified_ledger_balance]
def ffiBalance (s : State) (account : String) : UInt64 :=
  balance s account

end VerifiedLedger

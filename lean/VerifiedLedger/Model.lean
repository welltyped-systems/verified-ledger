--
-- Verified Ledger Model
--

import Std

namespace VerifiedLedger

inductive Op where
  | deposit (account : String) (amount : UInt64)
  | withdraw (account : String) (amount : UInt64)
  | transfer (fromAccount : String) (toAccount : String) (amount : UInt64)
  deriving Repr, DecidableEq

structure State where
  balances : List (String × UInt64)
  deriving Repr

def empty : State :=
  { balances := [] }

def balance (s : State) (account : String) : UInt64 :=
  match s.balances.find? (fun entry => entry.1 == account) with
  | some entry => entry.2
  | none => 0

def setBalance (s : State) (account : String) (amount : UInt64) : State :=
  let filtered := s.balances.filter (fun entry => entry.1 != account)
  if amount = 0 then
    { balances := filtered }
  else
    { balances := (account, amount) :: filtered }

def apply (s : State) (op : Op) : Except String State :=
  match op with
  | Op.deposit account amount =>
      let current := balance s account
      Except.ok (setBalance s account (current + amount))
  | Op.withdraw account amount =>
      let current := balance s account
      if current < amount then
        Except.error s!"insufficient funds: {account}"
      else
        Except.ok (setBalance s account (current - amount))
  | Op.transfer fromAccount toAccount amount =>
      let current := balance s fromAccount
      if current < amount then
        Except.error s!"insufficient funds: {fromAccount}"
      else
        let s' := setBalance s fromAccount (current - amount)
        let toBalance := balance s' toAccount
        Except.ok (setBalance s' toAccount (toBalance + amount))

def applyAll (s : State) (ops : List Op) : Except String State :=
  ops.foldl
    (fun acc op => acc.bind (fun state => apply state op))
    (Except.ok s)

end VerifiedLedger

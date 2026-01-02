--
-- Spec and proofs for the verified ledger model
--

import VerifiedLedger.Model

namespace VerifiedLedger

open Classical

-- helper lemmas for balance and list filtering

-- filtering by a predicate that removes all matches for account results in no match
theorem balance_filter_self (balances : List (String × UInt64)) (account : String) :
    balance { balances := balances.filter (fun entry => entry.1 != account) } account = 0 := by
  unfold balance
  have hnone :
      (balances.filter (fun entry => entry.1 != account)).find? (fun entry => entry.1 == account) = none := by
    apply (List.find?_eq_none).2
    intro entry hmem
    have hneq : entry.1 != account := (List.mem_filter.mp hmem).2
    have hne : entry.1 ≠ account := by
      simpa [bne_iff_ne] using hneq
    simpa [beq_iff_eq] using hne
  simp [hnone]

-- filtering by a predicate that keeps all matches for p does not change find
theorem find_filter_preserve {α : Type _} (p r : α → Bool) (l : List α)
    (hkeep : ∀ a, p a = true → r a = true) :
    List.find? p (List.filter r l) = List.find? p l := by
  induction l with
  | nil => simp
  | cons x xs ih =>
      cases hpx : p x
      · cases hrx : r x <;> simp [List.filter, hpx, hrx, ih]
      · have hrx : r x = true := hkeep x hpx
        simp [List.filter, hpx, hrx]

-- filtering out an account does not change the balance of other accounts
theorem balance_filter_other (balances : List (String × UInt64)) (account other : String)
    (h : other ≠ account) :
    balance { balances := balances.filter (fun entry => entry.1 != account) } other
      = balance { balances := balances } other := by
  have hkeep :
      ∀ entry : String × UInt64, (entry.1 == other) = true → (entry.1 != account) = true := by
    intro entry hEq
    have hEqBool : entry.1 == other := by simpa using hEq
    have hEqProp : entry.1 = other := (beq_iff_eq).1 hEqBool
    have hne : entry.1 ≠ account := by
      intro hAcc
      apply h
      simpa [hEqProp] using hAcc
    have hbool : entry.1 != account := (bne_iff_ne).2 hne
    simpa using hbool
  have hfind :=
      find_filter_preserve (p := fun entry => entry.1 == other)
        (r := fun entry => entry.1 != account) balances hkeep
  unfold balance
  simp [hfind]

theorem balance_setBalance_self (s : State) (account : String) (amount : UInt64) :
    balance (setBalance s account amount) account = amount := by
  by_cases h : amount = 0
  · simpa [setBalance, h] using (balance_filter_self s.balances account)
  · simp [balance, setBalance, h]

theorem balance_setBalance_other (s : State) (account other : String) (amount : UInt64)
    (h : other ≠ account) :
    balance (setBalance s account amount) other = balance s other := by
  by_cases hAmount : amount = 0
  · simpa [setBalance, hAmount] using (balance_filter_other s.balances account other h)
  · have hbeq : (account == other) = false := (beq_eq_false_iff_ne).2 (Ne.symm h)
    simpa [balance, setBalance, hAmount, hbeq] using
      (balance_filter_other s.balances account other h)

-- the main theorems

theorem apply_deposit_balance_self (s : State) (account : String) (amount : UInt64) :
    (apply s (Op.deposit account amount)).map (fun s' => balance s' account)
      = .ok (balance s account + amount) := by
  simp [apply, balance_setBalance_self, Except.map]

theorem apply_deposit_balance_other (s : State) (account other : String) (amount : UInt64)
    (h : other ≠ account) :
    (apply s (Op.deposit account amount)).map (fun s' => balance s' other)
      = .ok (balance s other) := by
  simp [apply, balance_setBalance_other, h, Except.map]

theorem apply_withdraw_fail (s : State) (account : String) (amount : UInt64)
    (h : balance s account < amount) :
    apply s (Op.withdraw account amount) = .error s!"insufficient funds: {account}" := by
  simp [apply, h]

theorem apply_withdraw_ok_balance_self (s : State) (account : String) (amount : UInt64)
    (h : ¬ balance s account < amount) :
    (apply s (Op.withdraw account amount)).map (fun s' => balance s' account)
      = .ok (balance s account - amount) := by
  simp [apply, h, balance_setBalance_self, Except.map]

theorem apply_withdraw_ok_balance_other (s : State) (account other : String) (amount : UInt64)
    (h : ¬ balance s account < amount) (hne : other ≠ account) :
    (apply s (Op.withdraw account amount)).map (fun s' => balance s' other)
      = .ok (balance s other) := by
  simp [apply, h, balance_setBalance_other, hne, Except.map]

theorem apply_transfer_fail (s : State) (fromAccount toAccount : String) (amount : UInt64)
    (h : balance s fromAccount < amount) :
    apply s (Op.transfer fromAccount toAccount amount) = .error s!"insufficient funds: {fromAccount}" := by
  simp [apply, h]

theorem apply_transfer_ok_balance_from (s : State) (fromAccount toAccount : String) (amount : UInt64)
    (h : ¬ balance s fromAccount < amount) (hneq : fromAccount ≠ toAccount) :
    (apply s (Op.transfer fromAccount toAccount amount)).map (fun s' => balance s' fromAccount)
      = .ok (balance s fromAccount - amount) := by
  simp [apply, h, balance_setBalance_other, hneq, balance_setBalance_self, Except.map]

theorem apply_transfer_ok_balance_to (s : State) (fromAccount toAccount : String) (amount : UInt64)
    (h : ¬ balance s fromAccount < amount) (hneq : fromAccount ≠ toAccount) :
    (apply s (Op.transfer fromAccount toAccount amount)).map (fun s' => balance s' toAccount)
      = .ok (balance s toAccount + amount) := by
  have hpre :
      balance (setBalance s fromAccount (balance s fromAccount - amount)) toAccount
        = balance s toAccount := by
    simpa using
      (balance_setBalance_other s fromAccount toAccount (balance s fromAccount - amount) (Ne.symm hneq))
  simp [apply, h, balance_setBalance_self, hpre, Except.map]

theorem apply_transfer_ok_balance_other (s : State) (fromAccount toAccount other : String)
    (amount : UInt64) (h : ¬ balance s fromAccount < amount)
    (hfrom : other ≠ fromAccount) (hto : other ≠ toAccount) :
    (apply s (Op.transfer fromAccount toAccount amount)).map (fun s' => balance s' other)
      = .ok (balance s other) := by
  have hpre :
      balance (setBalance s fromAccount (balance s fromAccount - amount)) other
        = balance s other := by
    simpa using
      (balance_setBalance_other s fromAccount other (balance s fromAccount - amount) hfrom)
  simp [apply, h, balance_setBalance_other, hto, hpre, Except.map]

theorem apply_transfer_ok_same_account (s : State) (account : String) (amount : UInt64)
    (h : ¬ balance s account < amount) :
    (apply s (Op.transfer account account amount)).map (fun s' => balance s' account)
      = .ok (balance s account) := by
  simp [apply, h, balance_setBalance_self, Except.map]

end VerifiedLedger

import Lake
open Lake DSL

package "verified-ledger" where
  moreLeanArgs := #["-DwarningAsError=false"]
  defaultTargets := #[`VerifiedLedger]

lean_lib "VerifiedLedger" where
  srcDir := "."

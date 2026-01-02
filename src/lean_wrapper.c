#include <lean/lean.h>
#include <stdint.h>
#include <stdlib.h>

extern lean_object *verified_ledger_empty;
extern lean_object *verified_ledger_apply_deposit(lean_object *, lean_object *,
                                                  uint64_t);
extern lean_object *verified_ledger_apply_withdraw(lean_object *, lean_object *,
                                                   uint64_t);
extern lean_object *verified_ledger_apply_transfer(lean_object *, lean_object *,
                                                   lean_object *, uint64_t);
extern uint64_t verified_ledger_balance(lean_object *, lean_object *);
extern lean_object *initialize_VerifiedLedger_FFI(uint8_t builtin);
extern void lean_initialize_runtime_module(void);
extern char **lean_setup_args(int argc, char **argv);

static int g_initialized = 0;

static void ledger_lean_abort(lean_object *error) {
  lean_io_result_show_error(error);
  abort();
}

void ledger_lean_initialize(void) {
  if (g_initialized) {
    return;
  }
  g_initialized = 1;

  char *args[] = {"verified-ledger", NULL};
  lean_setup_args(1, args);
  lean_initialize_runtime_module();
  lean_set_panic_messages(false);
  lean_object *res = initialize_VerifiedLedger_FFI(1);
  lean_set_panic_messages(true);

  if (lean_io_result_is_error(res)) {
    ledger_lean_abort(res);
  }

  lean_dec_ref(res);
  lean_io_mark_end_initialization();
  lean_init_task_manager();
}

void *ledger_lean_state_new(void) {
  lean_inc(verified_ledger_empty);
  return verified_ledger_empty;
}

void ledger_lean_state_dec(void *state) {
  if (state != NULL) {
    lean_dec_ref((lean_object *)state);
  }
}

static void *ledger_lean_apply_result(lean_object *result, uint8_t *ok) {
  uint8_t ok_value = lean_ctor_get_uint8(result, sizeof(void *) * 1);
  lean_object *state = lean_ctor_get(result, 0);

  lean_inc(state);
  lean_dec_ref(result);

  if (ok != NULL) {
    *ok = ok_value;
  }
  return state;
}

void *ledger_lean_apply_deposit(void *state, const char *account,
                                uint64_t amount, uint8_t *ok) {
  lean_object *account_obj = lean_mk_string(account);
  lean_object *result =
      verified_ledger_apply_deposit((lean_object *)state, account_obj, amount);
  return ledger_lean_apply_result(result, ok);
}

void *ledger_lean_apply_withdraw(void *state, const char *account,
                                 uint64_t amount, uint8_t *ok) {
  lean_object *account_obj = lean_mk_string(account);
  lean_object *result =
      verified_ledger_apply_withdraw((lean_object *)state, account_obj, amount);
  return ledger_lean_apply_result(result, ok);
}

void *ledger_lean_apply_transfer(void *state, const char *from_account,
                                 const char *to_account, uint64_t amount,
                                 uint8_t *ok) {
  lean_object *from_obj = lean_mk_string(from_account);
  lean_object *to_obj = lean_mk_string(to_account);
  lean_object *result = verified_ledger_apply_transfer(
      (lean_object *)state, from_obj, to_obj, amount);
  return ledger_lean_apply_result(result, ok);
}

uint64_t ledger_lean_balance(void *state, const char *account) {
  lean_inc((lean_object *)state);
  lean_object *account_obj = lean_mk_string(account);
  uint64_t result = verified_ledger_balance((lean_object *)state, account_obj);
  return result;
}

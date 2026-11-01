//! What: Raw bytes and generated records through the owner lock, transaction owner and
//!       `/proc/<pid>/stat` readers.
//! Why: Owner records are files any process can write, and a stat line carries a command name
//!      the process chose; liveness decisions come from both.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGeneratedOwnerLock(data); checkOwnerLockText(decode(data)); checkTransactionOwner(data); checkStat(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::owners::{
    check_generated_owner_lock, check_owner_lock_text, check_stat, check_transaction_owner,
};
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_generated_owner_lock(data);
    check_owner_lock_text(String::from_utf8_lossy(data).as_ref());
    check_transaction_owner(data);
    check_stat(data);
});

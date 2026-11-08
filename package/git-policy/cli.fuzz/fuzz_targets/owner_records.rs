//! What:
//! Raw bytes and generated owners through the copy owner token,
//! the journal and temporary file name,
//! transaction owner and
//!       `/proc/<pid>/stat` readers.
//! Why:
//! A copy's owner is written into file names,
//! owner records are files any process can write,
//! and a stat line carries a command name
//!      the process chose;
//! liveness decisions come from all of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGeneratedCopyOwnerToken(data); checkCopyOwnerToken(decode(data)); checkOwnerFileNames(decode(data)); checkTransactionOwner(data); checkStat(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::owners::{
    check_copy_owner_token, check_generated_copy_owner_token, check_owner_file_names, check_stat,
    check_transaction_owner,
};
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_generated_copy_owner_token(data);
    let text: String = String::from_utf8_lossy(data).into_owned();
    check_copy_owner_token(text.as_str());
    check_owner_file_names(text.as_str());
    check_transaction_owner(data);
    check_stat(data);
});

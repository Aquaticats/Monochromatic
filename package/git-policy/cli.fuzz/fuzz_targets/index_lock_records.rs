//! What: Raw bytes and generated text through the PID file reader and the stat-line reader.
//! Why: The index-lock evidence decides between waiting without limit and retiring a lock from
//!      these two readers; what they accept must mean one thing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkPidFileText(data); checkLinuxStat(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's index-lock checks.
use git_policy_cli_fuzz::index_lock::{check_linux_stat, check_pid_file_text};
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_pid_file_text(data);
    check_linux_stat(data);
});

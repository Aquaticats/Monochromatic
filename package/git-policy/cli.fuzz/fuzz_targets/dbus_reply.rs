//! What:
//!  Raw and generated D-Bus replies through the session bus client's reply reader.
//! Why:
//!  The bytes come from a socket another process writes,
//!  so the reader must answer every input without panicking or reading past the message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkRawReply(data); checkGeneratedReply(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared checks.
use git_policy_cli_fuzz::dbus_reply::{check_generated_reply, check_raw_reply};
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_raw_reply(data);
    check_generated_reply(data);
});

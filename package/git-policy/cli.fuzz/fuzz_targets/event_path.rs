//! What: Pathname bytes through the event form, its base64 and rendered event lines.
//! Why: Git allows any bytes in a name; an event must carry the readable text and, for a
//!      name that is not UTF-8, the exact bytes, without breaking its JSON line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => checkEventPath(data));
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared checks.
use git_policy_cli_fuzz::markdown::check_event_path;
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_event_path(data);
});

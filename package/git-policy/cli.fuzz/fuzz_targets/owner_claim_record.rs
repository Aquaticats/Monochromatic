//! What:
//! Generated and hand-edited text of a single-owner claim file,
//! through the decoder that decides whether a command may work on an item.
//! Why:
//! A claim file is text any process can write.
//! The decoder is private to the command that writes claims,
//! so this target reaches it through `decode`,
//! and reading a claim wrong either stops a command for nothing or lets two commands share one item.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkGeneratedClaimText(data); checkClaimText(textOf(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's claim-record checks.
use git_policy_cli_fuzz::owner_claim_record::{check_claim_text, check_generated_claim_text};
use libfuzzer_sys::fuzz_target;

// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_generated_claim_text(data);
    let text: String = String::from_utf8_lossy(data).into_owned();
    check_claim_text(text.as_str());
});

//! What:
//!  Raw and generated `git ls-files --stage -z` listings through the listing parser
//!       and the staged delta a `git add` prediction computes from two of them.
//! Why:
//!  Listing pathnames are chosen by whoever writes the repository and may hold tabs,
//!      spaces and bytes that are not UTF-8;
//!  the delta decides which files the content
//!      policies read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkStageRecords(data); checkGeneratedListings(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::content::{check_generated_listings, check_stage_records};
use libfuzzer_sys::fuzz_target;

// Run both views of every input: the bytes as a listing, and listings built from them.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    check_stage_records(data);
    check_generated_listings(data);
});

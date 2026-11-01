//! What:
//!  Coverage-guided entry for the Rust sidecar test rule.
//! Why:
//!  libFuzzer mutates arbitrary source and a bounded structured grammar together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Register checkRustStyle with the fuzz runtime.
//! ```
#![no_main]

/// Import the existing entry-point generator rather than hand-writing unsafe foreign-function glue.
use libfuzzer_sys::fuzz_target;
/// Import the same assertions exercised by the sidecar's ordinary tests.
use monochromatic_lint_fuzz::rust_style::check_rust_style;

// This is the macro's input grammar, not a Rust closure expression:
// libfuzzer-sys 0.4.13 src/lib.rs:247-295 expands it to named __libfuzzer_sys_run.
fuzz_target!(|data: &[u8]| -> () {
    check_rust_style(data);
});

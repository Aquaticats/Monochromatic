//! What:
//!  Coverage-guided semantic explicit-type policy checks.
//! Why:
//!  Generated valid programs and arbitrary source overlays exercise the same production session.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Register the named semantic property checker with libFuzzer.
//! ```
#![no_main]

/// Import the entry generator that expands to a named fuzz function.
use libfuzzer_sys::fuzz_target;
/// Import the property checker also used by ordinary generator controls.
use monochromatic_lint_fuzz::explicit_types::check_explicit_types;

// libfuzzer-sys 0.4.13 expands this input grammar into named __libfuzzer_sys_run, not an authored closure.
fuzz_target!(|data: &[u8]| -> () {
    check_explicit_types(data);
});

//! What:
//!  ASAN coverage-guided entry for native Markdown rules and original-source edits.
//! Why:
//!  The shared assertions run in unit controls and in a resource-bounded,
//!  mount-free fuzz process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Register checkMarkdown with the existing fuzz runtime.
//! ```
#![no_main]

/// Import the inspected runtime entry generator and shared property assertions.
use libfuzzer_sys::fuzz_target;
use monochromatic_lint_fuzz::markdown::check_markdown;

// This is macro input grammar expanding to a named entry function, not an authored Rust closure.
fuzz_target!(|data: &[u8]| -> () {
    check_markdown(data);
});

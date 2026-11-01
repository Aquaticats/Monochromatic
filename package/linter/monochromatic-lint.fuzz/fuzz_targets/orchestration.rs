//! What:
//!  ASAN coverage-guided entry for the executable's per-source path.
//! Why:
//!  Host rules,
//!  always-on processors,
//!  nested doc tests,
//!  host projection and the bounded fix
//! loop run together here exactly as the executable runs them for one file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Register checkOrchestration with the existing fuzz runtime.
//! ```
#![no_main]

/// Import the inspected runtime entry generator and shared property assertions.
use libfuzzer_sys::fuzz_target;
use monochromatic_lint_fuzz::orchestration::check_orchestration;

// This is macro input grammar expanding to a named entry function, not an authored Rust closure.
fuzz_target!(|data: &[u8]| -> () {
    check_orchestration(data);
});

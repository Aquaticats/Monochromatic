//! What:
//!  Process-isolated controls for configured logging filters and the startup fallback.
//! Why:
//!  Replacing the startup filter with its default must be observable rather than survive mutation testing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Run the same startup filter in child processes with valid, missing and invalid RUST_LOG values.
//! ```

/// Import the production startup filter and ordinary subprocess interface.
use super::logging_filter;
use std::process::Command;

/// Run only when the parent has selected this exact child probe and its expected filter.
#[test]
fn logging_filter_probe() {
    // Result's Ok variant contains the parent-owned fixture expectation; ordinary enumeration returns immediately.
    let Ok(expected) = std::env::var("SCANNER_LOGGING_FILTER_EXPECTED") else { return; };
    // Display renders only fixed fixture directives here, independently of subscriber initialization.
    assert_eq!(logging_filter().to_string(), expected);
}

/// Missing/invalid configuration preserves the info fallback;
///  valid configuration preserves the selected directives.
#[test]
fn configured_filters_and_fallback_are_observable() {
    // Option<&str> represents absent or borrowed environment text, rather than copying each literal into String.
    // None means remove RUST_LOG; Some contains the exact test-only directive.
    let cases: [(Option<&str>, &str); 4] = [
        (None, "info"),
        (Some("warn"), "warn"),
        (Some("forbidden_strings=trace"), "forbidden_strings=trace"),
        (Some("[invalid"), "info"),
    ];
    for (configuration, expected) in cases {
        // current_exe returns an owned native pathname to this test binary, not the installed scanner.
        let mut command: Command = Command::new(std::env::current_exe().expect("unit-test executable"));
        command.args(["--exact", "logging_tests::logging_filter_probe", "--nocapture"])
            .env("SCANNER_LOGGING_FILTER_EXPECTED", expected).env_remove("RUST_LOG");
        // Borrowed directive text is copied into the child environment without changing the parent process.
        if let Some(value) = configuration { command.env("RUST_LOG", value); }
        let output = command.output().expect("isolated logging-filter probe");
        // Borrow output bytes for UTF-8 display only after the child finished; these are fixed synthetic directives.
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
}

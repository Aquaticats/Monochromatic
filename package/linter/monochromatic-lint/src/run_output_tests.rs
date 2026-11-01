//! What:
//!  Consumer-level diagnostic stream and exit-status controls.
//! Why:
//!  Hidden findings still count,
//!  and fixed source must remain byte-identical outside its requested edits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Render real findings and inspect stdout, stderr, and status independently.
//! ```

/// Import actual output routing and diagnostic construction.
use super::{OutputOptions, RunOutput, run_output};
use crate::diagnostic::{Diagnostic, Severity, Span};

/// Construct one real finding with a known source position.
fn finding(severity: Severity) -> Diagnostic {
    return Diagnostic::new(
        "rust/max-lines",
        severity,
        String::from("message"),
        String::from("input.rs"),
        Span {
            offset: 0,
            length: 1,
            line: 1,
            column: 1,
        },
    );
}

/// Clean ordinary runs emit nothing;
///  ordinary findings use only JSONL stdout.
#[test]
fn clean_and_ordinary_output_keep_the_jsonl_contract() {
    let clean: RunOutput = run_output(&[], None, OutputOptions::default()).expect("clean");
    assert_eq!(clean.stdout, "");
    assert_eq!(clean.stderr, "");
    assert_eq!(clean.exit_code, 0);
    let ordinary: RunOutput =
        run_output(&[finding(Severity::Error)], None, OutputOptions::default()).expect("finding");
    assert!(ordinary.stdout.starts_with("{"));
    assert!(ordinary.stdout.ends_with('\n'));
    assert_eq!(ordinary.stderr, "");
    assert_eq!(ordinary.exit_code, 1);
}

/// The stdin-fix exception keeps original source bytes separate from JSONL diagnostic bytes.
#[test]
fn stdin_fix_routes_source_and_findings_separately() {
    let source: &str = "\u{feff}🚀\r\n";
    let result: RunOutput = run_output(
        &[finding(Severity::Warn)],
        Some(source),
        OutputOptions::default(),
    )
    .expect("stdin fix");
    assert_eq!(result.stdout, source);
    assert!(result.stderr.starts_with("{"));
    assert_eq!(result.exit_code, 0);
    let silent: OutputOptions = OutputOptions {
        silent: true,
        ..OutputOptions::default()
    };
    let hidden: RunOutput =
        run_output(&[finding(Severity::Error)], Some(source), silent).expect("silent fix");
    assert_eq!(hidden.stdout, source);
    assert_eq!(hidden.stderr, "");
    assert_eq!(hidden.exit_code, 1);
}

/// Hidden warnings still exceed the requested warning budget.
#[test]
fn display_filtering_does_not_change_accounting() {
    let options: OutputOptions = OutputOptions {
        quiet: true,
        max_warnings: Some(0),
        silent: false,
    };
    let exceeded: RunOutput =
        run_output(&[finding(Severity::Warn)], None, options).expect("warning limit");
    assert_eq!(exceeded.stdout, "");
    assert_eq!(exceeded.exit_code, 1);
    let accepted: OutputOptions = OutputOptions {
        max_warnings: Some(1),
        ..options
    };
    assert_eq!(
        run_output(&[finding(Severity::Warn)], None, accepted)
            .expect("limit equality")
            .exit_code,
        0
    );
}

/// A clean stdin-fix run still emits its source,
///  and hidden warnings still count against their budget.
#[test]
fn clean_stdin_and_silent_warning_limits_preserve_status() {
    let source: &str = "unchanged\r\n";
    let clean: RunOutput =
        run_output(&[], Some(source), OutputOptions::default()).expect("clean stdin");
    assert_eq!(clean.stdout, source);
    assert_eq!(clean.stderr, "");
    assert_eq!(clean.exit_code, 0);
    let options: OutputOptions = OutputOptions {
        silent: true,
        max_warnings: Some(0),
        ..OutputOptions::default()
    };
    let hidden: RunOutput =
        run_output(&[finding(Severity::Warn)], None, options).expect("hidden warning");
    assert_eq!(hidden.stdout, "");
    assert_eq!(hidden.exit_code, 1);
    let mut incomplete: Diagnostic = finding(Severity::Warn);
    incomplete.processing_failure = true;
    assert_eq!(
        run_output(&[incomplete], None, options)
            .expect("incomplete warning")
            .exit_code,
        2
    );
}

/// Unavailable semantic coverage is a processing failure even when output is silent.
#[test]
fn incomplete_processing_has_precedence_over_policy_failure() {
    let mut failure: Diagnostic = finding(Severity::Error);
    failure.processing_failure = true;
    let options: OutputOptions = OutputOptions {
        silent: true,
        max_warnings: Some(0),
        ..OutputOptions::default()
    };
    let result: RunOutput =
        run_output(&[failure, finding(Severity::Warn)], None, options).expect("processing failure");
    assert_eq!(result.stdout, "");
    assert_eq!(result.stderr, "");
    assert_eq!(result.exit_code, 2);
}

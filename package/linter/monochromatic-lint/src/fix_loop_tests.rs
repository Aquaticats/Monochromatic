//! What:
//!  Fix-loop convergence,
//!  cycle,
//!  budget and failure controls.
//! Why:
//!  Passing single-pass edit tests does not prove reparsing or final diagnostic freshness.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Record checker snapshots and drive real grouped edits through the production loop.
//! ```

/// Import the actual loop and its existing result models.
use super::{FixStop, SourceChecker, fix_source};
use crate::diagnostic::{Diagnostic, Severity, Span};
use crate::edits::{Edit, Fix, FixError};

/// Deterministic fixture behavior,
///  not executable repository configuration.
enum Mode {
    Settle,
    Cycle,
    Grow,
    Noop,
    Invalid,
    Processing,
    Fail,
}

/// Named checker state records every source it was actually asked to inspect.
struct Checker {
    /// Selected test behavior.
    mode: Mode,
    /// Exact ordered inputs to check.
    seen: Vec<String>,
    /// Optional check call that fails after earlier provisional work.
    fail_at: Option<usize>,
    /// Optional check call that reports incomplete processing rather than returning an error.
    processing_at: Option<usize>,
}

impl SourceChecker for Checker {
    /// Produce a fix derived from the current source,
    ///  never an offset cached from another pass.
    fn check(&mut self, source: &str) -> Result<Vec<Diagnostic>, FixError> {
        self.seen.push(String::from(source));
        if self.fail_at == Some(self.seen.len()) {
            return Err(FixError {
                message: String::from("late fixture check failed"),
            });
        }
        let replacement: String = match self.mode {
            Mode::Settle => {
                if source == "é" {
                    String::from("🚀")
                } else if source == "🚀" {
                    String::from("done")
                } else {
                    return Ok(Vec::new());
                }
            }
            Mode::Cycle => {
                if source == "a" {
                    String::from("b")
                } else {
                    String::from("a")
                }
            }
            Mode::Grow => format!("{source}x"),
            Mode::Noop | Mode::Processing => String::from(source),
            Mode::Invalid => String::from("invalid"),
            Mode::Fail => {
                return Err(FixError {
                    message: String::from("fixture check failed"),
                });
            }
        };
        let mut finding: Diagnostic = Diagnostic::new(
            "fixture/change",
            Severity::Error,
            format!("current: {source}"),
            String::from("fixture.md"),
            Span {
                offset: 0,
                length: source.len(),
                line: 1,
                column: 1,
            },
        );
        finding.processing_failure =
            matches!(self.mode, Mode::Processing) || self.processing_at == Some(self.seen.len());
        let end: usize = if matches!(self.mode, Mode::Invalid) {
            source.len() + 1
        } else {
            source.len()
        };
        finding.fix = Some(Fix {
            edits: vec![Edit {
                start: 0,
                end,
                replacement,
            }],
        });
        return Ok(vec![finding]);
    }
}

/// Construct a fresh named checker with no retained previous snapshots.
fn checker(mode: Mode) -> Checker {
    return Checker {
        mode,
        seen: Vec::<String>::new(),
        fail_at: None,
        processing_at: None,
    };
}

/// Unicode replacement lengths change between passes,
///  and the returned diagnostics are from the final source.
#[test]
fn reparses_each_changed_snapshot_and_checks_once_more() {
    let mut state: Checker = checker(Mode::Settle);
    let result = fix_source("é", &mut state).expect("settles");
    assert_eq!(result.source, "done");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.changed_passes, 2);
    assert_eq!(result.stop, FixStop::Unchanged);
    assert_eq!(state.seen, ["é", "🚀", "done", "done"]);
}

/// Circular fixes stop before exhausting the budget and retain a stable cycle anchor.
#[test]
fn cycles_stop_at_the_repeated_snapshot() {
    let mut state: Checker = checker(Mode::Cycle);
    let result = fix_source("a", &mut state).expect("cycle stops");
    assert_eq!(result.source, "a");
    assert_eq!(result.stop, FixStop::Cycle);
    assert_eq!(result.changed_passes, 2);
    assert_eq!(result.diagnostics[0].message, "current: a");
    assert_eq!(state.seen, ["a", "b", "a"]);
}

/// Nonconvergent growth stops at the agreed pass limit,
///  keeping fresh remaining findings.
#[test]
fn budget_limits_changed_passes_not_the_final_check() {
    let mut state: Checker = checker(Mode::Grow);
    let result = fix_source("a", &mut state).expect("bounded");
    assert_eq!(result.source, "axxxxxxxxxx");
    assert_eq!(result.stop, FixStop::Limit);
    assert_eq!(result.changed_passes, 10);
    assert_eq!(state.seen.len(), 11);
    assert_eq!(result.diagnostics[0].message, "current: axxxxxxxxxx");
}

/// A no-op fix does not create another edit pass.
#[test]
fn unchanged_source_stops_even_with_a_remaining_fix() {
    let mut state: Checker = checker(Mode::Noop);
    let result = fix_source("same", &mut state).expect("no-op");
    assert_eq!(result.source, "same");
    assert_eq!(result.stop, FixStop::Unchanged);
    assert_eq!(result.changed_passes, 0);
    assert_eq!(state.seen, ["same", "same"]);
}

/// Failures after an edit or during the mandatory final check cannot publish provisional output.
#[test]
fn late_failures_do_not_escape_as_accepted_fixed_source() {
    for mode in [Mode::Settle, Mode::Noop] {
        let mut state: Checker = checker(mode);
        state.fail_at = Some(2);
        assert!(fix_source("é", &mut state).is_err());
        assert_eq!(state.seen.len(), 2);
    }
    let mut incomplete: Checker = checker(Mode::Noop);
    incomplete.processing_at = Some(2);
    let error: FixError = fix_source("same", &mut incomplete).expect_err("incomplete final check");
    assert!(error.message.contains("fixture.md"));
    assert!(error.message.contains("fixture/change"));
    assert_eq!(incomplete.seen, ["same", "same"]);
}

/// Invalid edit plans and incomplete checking never return a provisional source for publication.
#[test]
fn edit_and_processing_failures_leave_the_callers_input_untouched() {
    let original: String = String::from("original");
    for mode in [Mode::Invalid, Mode::Processing, Mode::Fail] {
        let mut state: Checker = checker(mode);
        assert!(fix_source(original.as_str(), &mut state).is_err());
        assert_eq!(original, "original");
    }
}

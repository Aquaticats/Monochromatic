//! What:
//!  Controls of the answer the comparison builds:
//!  unknown outranks a finding,
//!  a finding outranks a pass,
//!  and every line a person acts on is printed.
//! Why:
//!  A comparison that could not read the whole configuration must never report
//!  a pass over it,
//!  and a finding nobody prints a correction for leaves the drift in place,
//!  so the precedence and the line shape are proved on built answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(coverageAnswer([finding], 'unproved', 'pass').outcome.kind).toBe('unknown');
//! ```

use super::{CoverageAnswer, CoverageFinding};
use crate::doctor_zfs_route::CheckOutcome;

/// One finding with its correction.
fn a_finding() -> CoverageFinding {
    return CoverageFinding::new("recursion", "the ancestor does not recurse", Some(String::from("change it to yes")));
}

/// No finding and no unproved fact is a pass carrying its detail.
#[test]
fn no_finding_and_no_unproved_fact_is_a_pass() {
    let answer: CoverageAnswer = CoverageAnswer::new(Vec::new(), None, String::from("everything holds"));
    assert!(matches!(answer.outcome, CheckOutcome::Pass { ref detail } if detail == "everything holds"));
    assert_eq!(answer.lines(), Vec::<String>::new());
}

/// Findings are a finding whose detail names every field,
/// and each finding and its correction are printed.
#[test]
fn findings_are_a_finding_naming_every_field_and_every_line() {
    let answer: CoverageAnswer = CoverageAnswer::new(vec![ a_finding() ], None, String::new());
    assert!(matches!(answer.outcome, CheckOutcome::Finding { ref field, ref detail } if field == "recursion" && detail.contains("recursion") && detail.contains("1 finding(s)")));
    assert_eq!(answer.lines(), vec![String::from("finding: recursion: the ancestor does not recurse"), String::from("correction: change it to yes")]);
}

/// A finding without a correction prints its line and no correction line.
#[test]
fn a_finding_without_a_correction_prints_no_correction_line() {
    let answer: CoverageAnswer = CoverageAnswer::new(vec![CoverageFinding::new("exclusion-dataset", "the dataset is absent", None)], None, String::new());
    assert_eq!(answer.lines(), vec![String::from("finding: exclusion-dataset: the dataset is absent")]);
}

/// An unproved fact outranks a finding and is the answer's line,
/// and the findings are printed below it.
#[test]
fn an_unproved_fact_outranks_a_finding_and_still_prints_it() {
    let answer: CoverageAnswer = CoverageAnswer::new(vec![ a_finding() ], Some(String::from("the file could not be read")), String::new());
    assert!(matches!(answer.outcome, CheckOutcome::Unknown { ref reason } if reason == "the file could not be read"));
    assert_eq!(answer.unproved.as_deref(), Some("the file could not be read"));
    assert_eq!(answer.lines()[0], "finding: recursion: the ancestor does not recurse");
    assert_eq!(answer.lines()[2], "unknown: the file could not be read");
}

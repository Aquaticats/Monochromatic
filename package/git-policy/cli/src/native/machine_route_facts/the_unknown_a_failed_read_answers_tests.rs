//! What:
//!  The controls of the line one failed read answers:
//!  one check,
//!  one sentence,
//!  and an unknown state no check can read as a pass.
//! Why:
//!  A check over an empty list passes without proving anything,
//!  so the line a failed read answers must be an unknown naming the reason and
//!  the check must stay the findable handle it is in every other line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theUnknownsOf(['store-entries'], 'EACCES')[0]).toEqual({ check: 'store-entries', outcome: { kind: 'unknown', reason: 'EACCES' } });
//! ```

/// The two builders a control reads.
use super::{
    the_unknown_of,
    the_unknowns_of,
};
/// The checks and report lines they build.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
};

/// What:
///  One check's line is an unknown that carries the read's own sentence and
///  prints under the check's stable name.
/// Why:
///  A person reads which check could not be answered and why,
///  and the name is what a log search of another release still finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theUnknownOf(Check.StoreEntries, 'the folder could not be listed').line()).toBe('zfs: store-entries: unknown: the folder could not be listed');
/// ```
#[test]
fn one_failed_read_answers_an_unknown_under_the_checks_name() {
    let report: CheckReport = the_unknown_of(
        Check::StoreEntries,
        "the folder could not be listed (EACCES)",
    );
    assert!(report.unknown(), "{}", report.line());
    assert_eq!(
        report.line(),
        "zfs: store-entries: unknown: the folder could not be listed (EACCES)"
    );
}

/// What:
///  Every check of one read answers its own unknown,
///  in the order the read's checks were named.
/// Why:
///  A read that feeds four checks leaves all four unproved,
///  and a control that named three of them would let the fourth print a pass
///  over nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theUnknownsOf([Check.CheckoutMount, Check.NestedReachability], 'no table')).toHaveLength(2);
/// ```
#[test]
fn every_check_of_one_failed_read_answers_an_unknown() {
    let reports: Vec<CheckReport> = the_unknowns_of(
        &[Check::CheckoutMount, Check::NestedReachability],
        "the mount table could not be read",
    );
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].check, Check::CheckoutMount);
    assert_eq!(reports[1].check, Check::NestedReachability);
    for report in &reports {
        assert_eq!(
            report.outcome,
            CheckOutcome::Unknown {
                reason: String::from("the mount table could not be read"),
            }
        );
    }
}

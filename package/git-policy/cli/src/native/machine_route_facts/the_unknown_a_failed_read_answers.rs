//! What:
//!  The report lines one failed read answers:
//!  one unknown per check that read it,
//!  carrying the read's own reason.
//! Why:
//!  Plan step 18 reports a check it cannot read or prove as unknown
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints"),
//!  and some of the facts a check reads are lists:
//!  the caller's mount table,
//!  the container's children,
//!  a folder's listing.
//!  A list that could not be read is empty,
//!  and a check over an empty list passes without proving anything,
//!  so a failed read answers its own unknown here instead,
//!  in the words of the read that failed,
//!  and no check prints a pass it did not prove.
//!  The checks' own facts still hold whatever the other reads answered,
//!  so one unreadable folder costs the checks that read it and no others.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unread = theUnknownsOf([Check.StoreEntries, Check.StagingForms], 'the store folder could not be listed (EACCES)');
//! // [{ check: 'store-entries', outcome: { kind: 'unknown', reason: '…' } }, …]
//! ```

/// One check's stable name,
/// and the report line a check answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
    unknown,
};

/// What:
///  The unknown one check answers when the read its facts come from failed.
/// Why:
///  One call per check keeps the reason in one place,
///  so two checks that read the same list say the same words about why it is
///  unread,
///  and the check keeps its own name in the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = theUnknownOf(Check.CheckoutMount, 'the table could not be read');
/// ```
pub(crate) fn the_unknown_of(
    check: Check,
    why: &str,
) -> CheckReport {
    return CheckReport::new(check, unknown(why));
}

/// What:
///  The unknowns every check over one unreadable read answers,
///  in the order the checks are named.
/// Why:
///  One read feeds several checks,
///  and each of them answers its own unknown rather than a pass over an empty
///  list,
///  so a caller names the whole set once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = theUnknownsOf([Check.StoreEntries, Check.StagingForms], why);
/// ```
pub(crate) fn the_unknowns_of(
    checks: &[Check],
    why: &str,
) -> Vec<CheckReport> {
    // What:     `iter().map(…).collect()` builds one report per check, which is the list `route_checks` prints.
    // Why:      The caller pushes the whole set at once, so no check of the read is forgotten.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return checks.map((check) => theUnknownOf(check, why));
    // ```
    return checks
        .iter()
        .map(|check| return the_unknown_of(*check, why))
        .collect();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_unknown_a_failed_read_answers_tests.rs"]
mod tests;

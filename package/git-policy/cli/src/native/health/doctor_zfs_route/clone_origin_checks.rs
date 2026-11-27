//! What:
//!  The origin each living clone names,
//!  and the state of its deferred-destroy mark:
//!  the origin dataset and snapshot the add recorded,
//!  whether the pool still holds that snapshot,
//!  whether somebody marked it for ZFS's own deferred destroy,
//!  and what a person can do about a mark.
//! Why:
//!  The design asks the report to print "each living clone's recorded origin"
//!  and to report "the deferred-origin check"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  Step 15 built that check,
//! and its facts are the measured ones this line
//!  rests on:
//!  the mark cannot be cleared (F8),
//!  and a marked snapshot cannot be destroyed while any clone of it exists
//!  (F18),
//!  so destroying the last clone reaps the marked snapshot in the same
//!  transaction and takes the worktree's files with it
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Reading the mark" and "How a person finishes a refused removal").
//!  The mark is one property of the origin snapshot's own object-stats reply,
//!  which is the read step 15 exported,
//!  so this line reuses that read and states neither a second property nor a
//!  second rule.
//!  A reply without the property is unknown and never "unmarked",
//!  because a check that cannot read its one field has not checked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkCloneOrigin({ clone, recorded, live }); // pass | finding('origin'|'defer_destroy') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use super::zfs_checks::Reading;
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    DEFER_DESTROY_PROPERTY,
    MARK_OFF,
    OriginMark,
};

/// What:
///  The origin dataset and snapshot one add recorded for its clone.
/// Why:
///  Both names belong to the origin,
///  and a line that printed only the snapshot would leave a person unable to
///  find it on the pool without the clone's own record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedOrigin = { dataset: string; snapshot: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedOrigin {
    /// The dataset the origin snapshot was taken on.
    pub dataset: String,
    /// The origin snapshot's full name.
    pub snapshot: String,
}

/// What:
///  One clone's recorded origin and what a read of that snapshot answered.
/// Why:
///  The record says which snapshot the clone was made from and the pool says
///  whether it stands and whether it is marked,
///  so both travel together and a control writes the state it judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneOriginFacts = { clone; recorded?: RecordedOrigin; live?: Reading<OriginMark> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneOriginFacts {
    /// The clone dataset this line is about.
    pub clone: String,
    /// The origin the add recorded,
    /// and nothing while the record names none.
    pub recorded: Option<RecordedOrigin>,
    /// What a read of the origin snapshot answered,
    /// and nothing when there was no origin snapshot to read.
    pub live: Option<Reading<OriginMark>>,
}

/// What:
///  Whether the clone's origin stands,
///  what its mark says,
///  and what a person can do about a marked one.
/// Why:
///  The mark is the one fact that decides whether a removal of this clone would
///  destroy history another program asked to keep,
///  and the ways a person finishes are the design's own:
///  leave it,
///  or destroy the clone outside the wrapper and accept that the marked
///  snapshot goes with it.
///  Neither way is a command this release has,
///  so the line names both acts and no wrapper command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCloneOrigin(facts); // pass | finding('origin'|'defer_destroy') | unknown
/// ```
pub fn check_clone_origin(facts: &CloneOriginFacts) -> CheckOutcome {
    let clone: &str = facts.clone.as_str();
    let Some(recorded) = &facts.recorded else {
        return pass(format!(
            "the record of `{clone}` names no origin yet, which is where an add that has not snapshotted the checkout stands"
        ));
    };
    let named: String = format!(
        "`{clone}` was cloned from `{}` on the dataset `{}`",
        recorded.snapshot, recorded.dataset
    );
    let Some(live) = &facts.live else {
        return unknown(format!(
            "{named}, and no read of that snapshot was handed in, so its state is unproved"
        ));
    };
    return match live {
        Reading::Missing {
            why,
        } => finding(
            "origin",
            format!(
                "{named}, and the pool holds no such snapshot ({why}), so the lineage this clone was made by cannot be compared with anything; a promoted or destroyed origin is the add's and the removal's problem, and this report changes nothing"
            ),
        ),
        Reading::Unreadable {
            why,
        } => unknown(format!(
            "{named}, and that snapshot could not be read ({why}), so whether it is marked is unproved"
        )),
        Reading::Read(OriginMark::Unmarked) => pass(format!(
            "{named}, the pool holds it, and `{DEFER_DESTROY_PROPERTY}` reads {MARK_OFF}, so no program asked ZFS to reap it; while this clone names it as its origin the pool refuses to destroy it, and this report changes neither"
        )),
        Reading::Read(OriginMark::Unreadable {
            said,
        }) => unknown(format!(
            "{named}, and {said}, so whether another program asked ZFS to reap it is unproved"
        )),
        Reading::Read(OriginMark::Marked {
            clones,
            holds,
        }) => finding(
            "defer_destroy",
            format!(
                "{named}, and another program marked that snapshot for deferred destroy (`{DEFER_DESTROY_PROPERTY}` reads 1): {clones} clone(s) of it stand and {} user hold(s). The mark cannot be cleared, and a marked snapshot cannot be destroyed while any clone of it exists, so destroying this last clone reaps the marked snapshot in the same transaction and takes the worktree's files with it. A removal of this clone refuses before its first delete for exactly this reason. A person either leaves the mark and the clone as they are, or takes the destruction outside the wrapper, knowing that the marked snapshot goes with the worktree's files; this report changes nothing.",
                holds_text(holds)
            ),
        ),
    };
}

/// What:
///  How a refusal words the hold count of a marked snapshot.
/// Why:
///  Zero holds is part of the condition under which the next destroy reaps the
///  marked snapshot,
///  so a count that could not be read must never print as zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = holdsText(holds); // '0' | '2' | 'a count that could not be read (...)'
/// ```
fn holds_text(
    holds: &crate::worktree_removal_deferred_origin_check::origin_mark::UserHolds
) -> String {
    use crate::worktree_removal_deferred_origin_check::origin_mark::UserHolds;
    return match holds {
        UserHolds::Counted(count) => count.to_string(),
        UserHolds::Unreadable {
            said,
        } => format!("a count that could not be read ({said})"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_origin_checks_tests.rs"]
mod tests;

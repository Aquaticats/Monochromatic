//! What:
//!  The nudge a command asks for on behalf of a waiting clone:
//!  the gap after which one attempt is due,
//!  the claim whose link decides which command of a burst owns it,
//!  which is the whole of the command's part:
//!  the helper that writes the claim's nonce is step 22's,
//!  with the helper's own mode and the gaps below.
//! Why:
//!  A nudge is one change of the clone's sweep nonce,
//!  and the command must not wait for the gate,
//!  the device or the write,
//!  so the command claims an attempt and a detached helper pays the route
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "How often,
//! and the burst" and V17).
//!  The gap doubles from the first to the longest and stops there,
//!  and a request that nothing can answer is nudged for as long as it stands,
//!  which the project accepted with its cost
//!  ("A further answer:
//! how long a lost request keeps being nudged").
//!  This file is compiled only under the `zfs-lost-event-nudge` feature,
//!  which is off:
//!  the first gap,
//!  the longest gap and the helper's run are step 22's,
//!  and `the_gaps` answers nothing until they are measured.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gaps = theGaps(); if (!gaps) return;
//! if (!isDue(attempt, written, now, gaps)) return;
//! const claim = claimTheNextAttempt(folder, entry); // one link winner
//! ```

use crate::foundation::iso_time::now_iso;
use crate::worktree_clone_mount_protocol::unanswered_request_entry::{
    RequestEntry,
    RequestOutcome,
};
use crate::worktree_clone_mount_protocol::unanswered_request_names::{
    EntryName,
    one_past_the_highest,
};
use crate::worktree_clone_mount_protocol::unanswered_requests_folder::UnansweredRequests;
use std::time::{
    Duration,
    SystemTime,
};

/// What:
///  The two gaps one entry's attempts are spaced by,
///  and the delay after which a waiting command nudges for itself.
/// Why:
///   No number ships from here:
///  measurement M8 sizes all three,
///  because they decide what a command pays and what one nudge costs root
///  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
///  "The measurement owed").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gaps = { first: Duration; longest: Duration; waitDelay: Duration };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TheGaps {
    /// The gap after the first entry,
    /// `G0`.
    pub(crate) first: Duration,
    /// The gap no later attempt waits longer than,
    /// `Gmax`.
    pub(crate) longest: Duration,
    /// The delay after which a waiting command nudges for itself,
    /// `D`.
    pub(crate) wait_delay: Duration,
}

/// What:
///  The gaps this build ships,
///  which is none:
///  step 22 answers them from M8 with the measurement beside each number.
/// Why:
///  No number is invented to fill a gap,
///  and a switch with no numbers must nudge nothing,
///  so the one place the numbers live answers nothing until they are measured
///  (rule `LQM`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theGaps = () => undefined; // step 22: { first, longest, waitDelay }
/// ```
pub(crate) fn the_gaps() -> Option<TheGaps> {
    return None;
}

/// What:
///  How long after one attempt the next is due:
///  `G0 * 2^(n - 1)`,
///  capped at `Gmax`.
/// Why:
///   The first nudge comes `G0` after the entry was written,
///  the second twice as far,
///  and from the attempt that reaches `Gmax` on one nudge per `Gmax` for as
///  long as the request is unanswered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gap = (n, gaps) => min(gaps.first * 2 ** (n - 1), gaps.longest);
/// ```
pub(crate) fn the_gap_before(
    attempt: u32,
    gaps: &TheGaps,
) -> Duration {
    // What: the doubling stops at the longest gap, so the shift cannot overflow either.
    // Why:  an attempt number is bounded by the names present, and a gap that had doubled past `Gmax`
    //       would be capped anyway.
    let doublings: u32 = attempt.saturating_sub(1).min(31);
    return gaps
        .first
        .saturating_mul(1u32 << doublings)
        .min(gaps.longest);
}

/// Whether one entry's attempt has waited its gap.
pub(crate) fn is_due(
    attempt: u32,
    written: SystemTime,
    now: SystemTime,
    gaps: &TheGaps,
) -> bool {
    let waited: Duration = now.duration_since(written).unwrap_or(Duration::ZERO);
    return waited >= the_gap_before(attempt, gaps);
}

/// What:
///  Take the clone's next attempt for this command,
///  or answer nothing because another command of the burst took it.
/// Why:
///   The link of one name decides the one owner,
///  so exactly one command of a burst writes and spawns,
///  and the attempt's number is one past the highest name present,
///  empty or readable,
///  which keeps a claim possible under a crash's empty file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = claimTheNextAttempt(folder, entry); // undefined when another command won
/// ```
pub(crate) fn claim_the_next_attempt(
    folder: &UnansweredRequests,
    entry: &RequestEntry,
) -> Option<EntryName> {
    let present: Vec<u32> = folder.attempts_of(entry.guid);
    let claim: EntryName = EntryName {
        guid: entry.guid,
        attempt: one_past_the_highest(present.as_slice()),
    };
    let claimed: RequestEntry = RequestEntry {
        outcome: RequestOutcome::Claimed,
        written_at: now_iso(),
        ..entry.clone()
    };
    return match folder.publish_a_new_attempt(claim, &claimed) {
        Ok(true) => Some(claim),
        Ok(false) | Err(_) => None,
    };
}

/// What:
///  Ask for one nudge of an entry the receipt does not answer,
///  when its attempt has waited its gap.
/// Why:
///   The gap and the claim are the whole of the command's part:
///  everything else is the helper's,
///  and a command that lost the link writes nothing more and prints nothing.
///  A build whose gaps are not measured asks for nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gaps = theGaps(); if (!gaps) return;
/// if (!isDue(...)) return; claimTheNextAttempt(folder, entry);
/// ```
pub(crate) fn ask_for_a_nudge(
    folder: &UnansweredRequests,
    name: EntryName,
    entry: &RequestEntry,
) -> Option<EntryName> {
    let gaps: TheGaps = the_gaps()?;
    let Ok(facts) = std::fs::metadata(folder.path().join(name.file_name())) else {
        return None;
    };
    let written: SystemTime = facts.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    if !is_due(name.attempt, written, SystemTime::now(), &gaps) {
        return None;
    }
    // What: the helper that writes this claim's nudge is started at step 22, with the gaps above.
    // Why:  its mode is a word of the one executable, and the file that dispatches a wrapper's arguments is
    //       being changed by the round for issue 6641 now, so the mode and the spawn land together with the
    //       numbers rather than ahead of them.
    return claim_the_next_attempt(folder, entry);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unanswered_requests_nudge_tests.rs"]
mod tests;

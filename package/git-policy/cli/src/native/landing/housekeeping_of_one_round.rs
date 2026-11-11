//! What:
//!  How long one round of the service's after-commit work may go on starting housekeeping,
//!  and what is left of a round's housekeeping when that is spent.
//! Why:
//!   A round drains one repository's records in the order:
//! the person's pushes,
//! the upkeep,
//! the prewarms,
//! the subvolume deletions,
//! the trashes,
//! the reviews.
//! A trash of one removed worktree is hundreds of thousands of removals
//!  (`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`),
//! and a service that logged 15 of them spent 1 h 37 min in one round without making the
//!  person's push of `refs/heads/main`
//! (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
//! A bound makes a round say "not this round" and leave the rest for a later one,
//! where it is named
//!  in the journal,
//! so nothing is lost and nothing waits for an hour.
//! The bound decides what a round *starts*:
//! an item that has started runs to its end,
//! because a removal that stopped
//!  halfway would need a resume the trash does not keep,
//! and the one place of the blocking pool this round leaves free
//! (`places_a_round_may_take`) keeps the steps that answer a
//!  client running while that item lasts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slice = new HousekeepingOfOneRound(HOUSEKEEPING_OF_ONE_ROUND); for (const item of trashes) { if (slice.spent()) left.push(item); else await empty(item); }
//! ```

use std::time::{Duration, Instant};

/// How long one round of the after-commit work may spend starting housekeeping:
/// thirty seconds.
/// Measured and reasoned:
/// the longest answer wait of a commit in the service's journal of 2026-11-03 was 308,267 ms and the longest landing 3,194 ms,
/// and a person's push that arrives while a round works waits for that round to end
///  (`service_server_work_loop_awaiting.rs`);
/// thirty seconds is six of the client's five-second windows
/// (`HEARTBEAT_SILENCE`) and about ten of the slowest landing,
/// so a round that is ending its housekeeping is
///  shorter than the wait a client already tolerates.
/// It is not a bound on one item:
/// a removal already begun is never interrupted (see the module description),
/// so the figure buys the order of the work and not the length of one step.
/// The coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const HOUSEKEEPING_OF_ONE_ROUND: Duration = Duration::from_secs(30);

/// How soon a round that left housekeeping for later asks to be woken again:
/// five seconds.
/// It is the client's own silence window (`HEARTBEAT_SILENCE`),
/// which is as long as this service lets anything wait without a word,
/// and it keeps the trash of a removed worktree moving while no
///  commit arrives to wake the service.
pub const LOOK_AGAIN_AFTER_DEFERRING: Duration = Duration::from_secs(5);

/// What:
///  One round's housekeeping slice:
/// when it began and how long it may go on starting housekeeping.
/// Why:
///   The question the drain asks is only "may I start this one now",
/// and the answer is the time the honour ran out,
/// so no lock,
/// no counter and no thread is needed for it,
/// and the drain's one task owns it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HousekeepingOfOneRound { constructor(private bound: Duration) {} spent() { return this.began.elapsed() >= this.bound; } }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct HousekeepingOfOneRound {
    /// When this round began its housekeeping.
    began: Instant,
    /// How long it may go on starting housekeeping.
    bound: Duration,
}

impl HousekeepingOfOneRound {
    /// A slice that begins now and lasts `bound`.
    pub fn starting_now(bound: Duration) -> Self {
        return Self { began: Instant::now(), bound };
    }

    /// Whether this round may still start housekeeping work.
    pub fn may_start(&self) -> bool {
        return self.began.elapsed() < self.bound;
    }

    /// Whether this round has spent its share of housekeeping.
    pub fn spent(&self) -> bool {
        return !self.may_start();
    }

    /// How long this round has spent on housekeeping so far,
    /// in milliseconds,
    /// for the line that names what was left.
    pub fn spent_ms(&self) -> u128 {
        return self.began.elapsed().as_millis();
    }

    /// How long this round was allowed.
    pub fn bound(&self) -> Duration {
        return self.bound;
    }
}

/// Housekeeping controls stay out of the release executable.
#[cfg(test)]
#[path = "housekeeping_of_one_round_tests.rs"]
mod tests;

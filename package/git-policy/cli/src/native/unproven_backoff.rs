//! What:
//!  How long a command or the commit service looks again at an `index.lock` whose owner cannot be proven alive or dead,
//!  before it stops waiting and says what it found.
//! Why:
//!  A lock whose owner is proven alive is waited for as long as the owner holds it, with no bound at all, because the owner
//!  will let go (the owner's rule "You should (almost) never be waiting." makes the wait short in practice, and "Commits
//!  should never fail." forbids ending it on a clock). A lock nobody can be shown to hold may be a killed program's
//!  leftover that never goes, so it gets this short backoff and then a diagnostic that lists the evidence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const UNPROVEN_OWNER_BACKOFF_MS = 1000;
//! ```

use std::time::Duration;

/// How long an owner that cannot be proven alive or dead is waited for.
///
/// Measured on 2026-11-03 on this host, in this repository's layout (20,739 tracked files, a 2.9 MB index), with the
///  `core.lockfilePid` setting off and 20 runs of each of `git status`, `git add -A`, `git reset -q` and
///  `git update-index --refresh` on a copy of the index: the lock was held for at most 100 ms (`add -A`; medians 26 to 53
///  ms; `status` at most 55 ms; `reset` at most 71 ms; `update-index` at most 77 ms). One second is ten times the longest
///  hold of any of them, so Git's own short holders are always waited out, and an owner that holds longer than that without
///  proof is a long-running command or a leftover, which the diagnostic names. The record's default is the same number
///  (`doc/decision/cli-git-concurrent-commits.md`, "Locks": `indexLock.unprovenOwnerTimeoutMs`, default 1000).
///
/// The service's journal and the wrapper's timing log hold no measurement of a foreign holder to set this against: none of
///  the 1,335 landings the journal still held on 2026-11-03 recorded finding the lock held (`lock_looks` absent), and none
///  of the 1,090 records of `.git/cli-git-commit-timing.jsonl` has a waiting notice (`waitingNotices` zero).
pub const UNPROVEN_OWNER_BACKOFF: Duration = Duration::from_millis(1000);

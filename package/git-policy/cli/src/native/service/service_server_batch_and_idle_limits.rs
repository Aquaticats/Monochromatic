//! What:
//!  The numbers one service run is held to:
//!  how many requests it lands as one batch,
//!  the largest batch whose cost still grows as its size does,
//!  how long it waits with nothing to do before it exits,
//!  and how often it asks whether its own worktree is still there.
//! Why:
//!   Each figure here has the measurement or the owner's decision that made it what it is,
//!   and each is obeyed somewhere else:
//!   the batch cap by the loop in `service_server.rs`,
//!   the idle exit and the worktree check by the settings beside this file.
//!  A reader who asks "how long does the service wait" finds the number and its evidence in one place,
//!  and the setting a control shortens in `service_server_serve_settings.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const MAX_BATCH = 256; const IDLE_EXIT = 3 * 60 * 60 * 1000; const OWN_WORKTREE_CHECK = 60 * 1000;
//! ```
use std::time::Duration;

/// The largest batch whose landing still costs about what its size predicts: 256.
/// Measured with the service binary on throwaway
///  repositories,
/// 2026-11-03,
/// bursts of simultaneous commit requests (`MAX_BATCH` raised to 4096 for the probe,
/// three rounds
///  alternating over 64,
/// 128,
/// 256,
/// 512 and 1024 clients,
/// other sessions running,
/// a debug build):
/// a landing took 71 to 76 ms for
///  62 to 63 requests,
/// 82 to 85 ms for 126,
/// 104 to 109 ms for 253 to 255,
/// 193 to 204 ms for 509 to 511 (once 2,002 ms,
/// a host stall)
///  and 525 to 547 ms for 1,021 to 1,023.
/// The cost of each added request is 0.16 ms up to 254 requests,
/// 0.36 ms from there to
///  510,
/// and 0.66 ms to 1,022,
/// so the cost per request has not yet doubled at 256.
/// The time a client waits for a burst of 128 was
///  176 to 206 ms in one landing and 243 to 249 ms in three at the old cap of 64.
pub const BATCH_COST_KNEE: usize = 256;

/// The most requests landed as one batch:
/// `BATCH_COST_KNEE`.
/// A batch is answered all at once and holds the index lock while it
///  lands,
/// so the cap bounds both.
/// The prototype's burst reached 64,
/// and production never reached more than one request in a
///  batch (1,302 answered requests,
/// 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z),
/// so the cap bites only in a burst.
/// Option B of the
///  batch size question,
/// picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups").
pub const MAX_BATCH: usize = BATCH_COST_KNEE;

/// How long the service waits with nothing to do before it exits;
/// the manager's socket stays,
/// so the next connection starts it
///  again.
/// Bazel's server does the same with `--max_idle_secs`,
/// whose default is three hours,
/// which is this value's precedent.
///  Measured cost of exceeding it:
/// of 1,051 commits in the commit timing log (2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z),
/// the 10
///  that followed an idle gap of more than three hours reached the service in at most 70 ms (`reachMs`) after a start wait of at
///  most 47 ms (`startWaitMs`);
/// 110 idle exits happened in that window.
/// The 28 running services held 212 MiB together (7.9 MiB each,
///  `systemctl --user show --property=MemoryCurrent`).
/// A shorter exit would trade at most that start for that memory;
/// the owner's
///  bound is that a commit should not take 5 s longer than native Git (`SPEC.md`).
/// Kept at three hours as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const IDLE_EXIT: Duration = Duration::from_secs(3 * 60 * 60);

/// How often a serving service asks whether the worktree whose units are its own is still there.
/// Why:
///   A removal Git ran with the wrapper replaced and a worktree deleted without Git leave the socket unit listening,
/// and the service that serves it is already running:
/// no start happens for a gone worktree,
/// so nothing would ever ask,
/// and the socket would listen for that directory until a person remembered the sweep.
/// At this interval the socket stops at most this long after the removal,
/// and the price is one wake of the landing thread and two stat calls per service per interval:
/// one `symlink_metadata` on the Git directory and one `is_dir` on the place that holds it
/// (`service_units_of_removed_worktrees::worktree_is_gone`),
/// measured on this host at 11.4 microseconds for the pair (five rounds of a million,
/// 11.32 to 11.74),
/// so 355 services cost 0.07 milliseconds a second between them.
/// A minute rather than an hour because the leftover is a listening socket,
/// and a minute rather than a second
/// because the loop's own wake is what it costs;
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const OWN_WORKTREE_CHECK: Duration = Duration::from_secs(60);

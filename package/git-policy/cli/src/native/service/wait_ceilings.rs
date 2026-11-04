//! What:
//!  The longest the wrapper waits for each thing it waits for without a person at the keyboard, with the measurement or the
//!  decision beside each number. A write in place that is not safe to run twice (the branch write) has no ceiling at all: the
//!  owner, 2026-11-04, "Never cut a branch write (Recommended)".
//! Why:
//!  The owner, 2026-11-03: "nothing should ever wait forever or indefinitely". Every wait on the commit path ends at one of these,
//!  the request is withdrawn so it never lands later, the person's index and worktree stay as they were, and the command exits
//!  non-zero naming the blocker (`doc/planning/cli-git-no-unbounded-waits.md`). The data was read on 2026-11-03 from this
//!  repository's commit timing log (1,329 records) and the commit service's journal (1,335 landings).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const ANSWER_CEILING_MS = 20 * 60 * 1000;
//! ```

use std::time::Duration;

/// The longest a client waits for the commit service's answer to a request, heartbeats or not.
///  Measured: the longest answer wait in the timing log was 308,267 ms (99th percentile 50,344 ms) and the longest whole commit
///  750,604 ms. Twenty minutes is 3.9 times the longest answer wait and 1.6 times the longest whole commit.
pub const ANSWER_CEILING: Duration = Duration::from_secs(20 * 60);

/// The longest a client waits for the service to start when it can ask the service manager how the start goes.
///  Measured: the slowest of 52 cold starts took 1,041 ms (`startWaitMs`, `reachMs`). One minute is 58 times that.
pub const START_CEILING: Duration = Duration::from_secs(60);

/// The longest a landing, a merge or an index reconciliation waits for a foreign program's `index.lock`.
///  Unmeasured: no log holds a wait behind a foreign holder (no `lock_looks` in the service journal, no waiting notice in the
///  timing log). Ten minutes is below `ANSWER_CEILING`, so the service ends its own wait and answers before the client gives up;
///  a person's `git commit` with its editor open for longer than this makes ours stop with the error.
pub const LOCK_CEILING: Duration = Duration::from_secs(10 * 60);

/// The longest a forwarded `git add`, `rm` or `mv` waits for the commit service's hold on the index, and the longest its whole
///  series of reruns lasts. Measured: the longest of 715 logged service holds was 119,721 ms (disk sync of the new index). Ten
///  minutes is five times that.
pub const SERVICE_HOLD_CEILING: Duration = Duration::from_secs(10 * 60);

/// The longest a client waits for the service to answer a withdrawal of its request. The service answers it as soon as the landing
///  it is in the middle of ends, which `LOCK_CEILING` and the step bounds limit; the number is the longest observed landing
///  (journal `landing_ms` 119,836 ms) with the same margin of five.
pub const WITHDRAW_CEILING: Duration = Duration::from_secs(10 * 60);

/// The longest a connected push may show no progress before it is ended and left to the service's retry plan.
///  Unmeasured, labelled as the other quiet windows are: no log holds a quiet gap of a connected push. It is twice the longest
///  whole push in the journal (72.4 s of 794 pushes), which includes the connected time.
pub const CONNECTED_PUSH_QUIET_WINDOW: Duration = Duration::from_secs(150);

/// The longest a test marker compiled into the release executable waits for its release file; the marker is armed only by an
///  environment variable. Five minutes is the longest any test may run (`.config/nextest.toml`: a test is stopped after
///  `slow-timeout` 60 s times `terminate-after` 5), so the marker never outlives the control that armed it.
pub const PHASE_PATIENCE: Duration = Duration::from_secs(5 * 60);

/// The longest a quick Git query step (`rev-parse`, `for-each-ref`, `cat-file`, `write-tree`, ...) runs before the wrapper ends it,
///  even while it shows signs of work. Measured: the longest read step of the service's landings was 9,663 ms (`reads_ms`, 1,651
///  landings); two minutes is 12 times that. It is below `ANSWER_CEILING`, so the service answers first.
pub const PLUMBING_STEP_CEILING: Duration = Duration::from_secs(2 * 60);

/// The longest a bulk Git step that is not a write in place (`fast-import`, `pack-objects`, `add` and `reset` on a private index,
///  the commit Git builds in a scratch repository, ...) runs before the wrapper ends it, even while it shows signs of work. Measured: the longest client-side preparation of the timing log was
///  250,280 ms (`prepareMs`, 1,400 records) and the longest build 183,296 ms; the service's object step took at most 1,358 ms
///  (`objects_ms`). Ten minutes is 2.4 times the longest preparation and below `ANSWER_CEILING`.
pub const OBJECT_STEP_CEILING: Duration = Duration::from_secs(10 * 60);

/// How long a write that the wrapper never ends (`transaction_git.rs`, `WRITES_IN_PLACE`: the branch write above all) runs before
///  the commit service tells the clients queued behind it what it is stuck in, and before it lets a queued client be withdrawn
///  while the landing thread is inside it (`service_busy.rs`). This is a notice threshold, not a bound: nothing ends the write.
///  Measured: the longest of 2,093 reference updates in the service journal (`answered` lines, `ref_ms`, 2026-11-02T04:01:38-05:00
///  to 2026-11-04T09:11:09-05:00) took 3,438 ms (99th percentile 47 ms); one minute is 17 times the longest.
pub const NEVER_ENDED_STEP_SLOW_AFTER: Duration = Duration::from_secs(60);

//! What:
//!  The longest the wrapper waits for each thing it waits for without a person at the keyboard,
//! with the measurement or the
//!  decision beside each number.
//! A step that cannot be run twice (the branch write above all) has no ceiling at all:
//! the
//!  owner,
//! 2026-11-04,
//! "Never cut a branch write (Recommended)".
//! Why:
//!  The owner,
//! 2026-11-03:
//! "nothing should ever wait forever or indefinitely".
//! Every wait on the commit path ends at one of these,
//!  the request is withdrawn so it never lands later,
//! the person's index and worktree stay as they were,
//! and the command exits
//!  non-zero naming the blocker (`doc/planning/cli-git-no-unbounded-waits.md`).
//! The data was read on 2026-11-03 from this
//!  repository's commit timing log (1,329 records) and the commit service's journal (1,335 landings).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const ANSWER_CEILING_MS = 20 * 60 * 1000;
//! ```

use std::time::Duration;

/// The longest a client waits for the commit service's answer to a request,
/// heartbeats or not.
///  Measured:
/// the longest answer wait in the timing log was 308,267 ms (99th percentile 50,344 ms) and the longest whole commit
///  750,604 ms. Twenty minutes is 3.9 times the longest answer wait and 1.6 times the longest whole commit.
pub const ANSWER_CEILING: Duration = Duration::from_secs(20 * 60);

/// The longest a client waits for the service to start when it can ask the service manager how the start goes.
///  Measured:
/// the slowest of 52 cold starts took 1,041 ms (`startWaitMs`,
/// `reachMs`).
/// One minute is 58 times that.
pub const START_CEILING: Duration = Duration::from_secs(60);

/// The longest the look at the running processes (`landing/earlier_service_process_look.rs`) takes before the adoption of an
///  earlier helper's journal counts the answer as "a service may be running".
/// A read of another process's command line waits
///  for that process's address-space lock,
/// which a process stuck in the kernel holds without end.
///  Measured on 2026-11-05 on this host:
/// 899 processes,
/// one `cat` of each `/proc/<pid>/cmdline` from a shell loop,
/// 0.64 s in all
///  (the look itself starts no process,
/// so it is faster).
/// Ten seconds is 15 times that and well below `START_CEILING`.
pub const PROCESS_LOOK_CEILING: Duration = Duration::from_secs(10);

/// The longest a landing,
/// a merge or an index reconciliation waits for a foreign program's `index.lock`.
///  Unmeasured:
/// no log holds a wait behind a foreign holder (no `lock_looks` in the service journal,
/// no waiting notice in the
///  timing log).
/// Ten minutes is below `ANSWER_CEILING`,
/// so the service ends its own wait and answers before the client gives up;
///  a person's `git commit` with its editor open for longer than this makes ours stop with the error.
pub const LOCK_CEILING: Duration = Duration::from_secs(10 * 60);

/// The longest a forwarded `git add`,
/// `rm` or `mv` waits for the commit service's hold on the index,
/// and the longest its whole
///  series of reruns lasts.
/// Measured:
/// the longest of 715 logged service holds was 119,721 ms (disk sync of the new index).
/// Ten
///  minutes is five times that.
pub const SERVICE_HOLD_CEILING: Duration = Duration::from_secs(10 * 60);

/// The longest a client waits for the service to answer a withdrawal of its request.
/// The service answers it as soon as the landing
///  it is in the middle of ends,
/// which `LOCK_CEILING` and the step bounds limit;
/// the number is the longest observed landing
///  (journal `landing_ms` 119,836 ms) with the same margin of five.
pub const WITHDRAW_CEILING: Duration = Duration::from_secs(10 * 60);

/// The longest a connected push may show no progress before it is ended and left to the service's retry plan.
///  Unmeasured,
/// labelled as the other quiet windows are:
/// no log holds a quiet gap of a connected push.
/// It is twice the longest
///  whole push in the journal (72.4 s of 794 pushes),
/// which includes the connected time.
pub const CONNECTED_PUSH_QUIET_WINDOW: Duration = Duration::from_secs(150);

/// The longest a test marker compiled into the release executable waits for its release file;
/// the marker is armed only by an
///  environment variable.
/// Five minutes is the longest any test may run (`.config/nextest.toml`:
/// a test is stopped after
///  `slow-timeout` 60 s times `terminate-after` 5),
/// so the marker never outlives the control that armed it.
pub const PHASE_PATIENCE: Duration = Duration::from_secs(5 * 60);

/// The longest a quick Git query step (`rev-parse`,
/// `for-each-ref`,
/// `cat-file`,
/// `write-tree`,
/// ...) runs before the wrapper ends it,
///  even while it shows signs of work.
/// Measured:
/// the longest read step of the service's landings was 9,663 ms (`reads_ms`,
/// 1,651
///  landings);
/// two minutes is 12 times that.
/// It is below `ANSWER_CEILING`,
/// so the service answers first.
pub const PLUMBING_STEP_CEILING: Duration = Duration::from_secs(2 * 60);

/// The longest a bulk Git step that can be run twice (`fast-import`,
/// `pack-objects`,
/// `add` and `reset` on a private index,
///  the commit Git builds in a scratch repository,
/// ...) runs before the wrapper ends it,
/// even while it shows signs of work.
/// Measured:
/// the longest client-side preparation of the timing log was
///  250,280 ms (`prepareMs`,
/// 1,400 records) and the longest build 183,296 ms;
/// the service's object step took at most 1,358 ms
///  (`objects_ms`).
/// Ten minutes is 2.4 times the longest preparation and below `ANSWER_CEILING`.
pub const OBJECT_STEP_CEILING: Duration = Duration::from_secs(10 * 60);

/// How long a write that the wrapper never ends (`transaction_git.rs`,
/// `may_be_ended`:
/// the branch write above all) runs before
///  the commit service tells the clients queued behind it what it is stuck in,
/// and before it lets a queued client be withdrawn
///  while the landing thread is inside it (`service_busy.rs`).
/// This is a notice threshold,
/// not a bound:
/// nothing ends the write.
///  Measured:
/// the longest of 2,093 reference updates in the service journal (`answered` lines,
/// `ref_ms`,
/// 2026-11-02T04:01:38-05:00
///  to 2026-11-04T09:11:09-05:00) took 3,438 ms (99th percentile 47 ms);
/// one minute is 17 times the longest.
pub const NEVER_ENDED_STEP_SLOW_AFTER: Duration = Duration::from_secs(60);

/// The longest one package-manager tool of a worktree repair (`pnpm install`,
/// `cargo metadata`,
/// `cargo clean`,
/// `mpm-shadow install`)
///  runs,
/// whatever it shows,
/// before the wrapper ends it with its whole process group.
/// The quiet window of 600 seconds
///  (`package_repair.rs`) ends a tool that shows nothing;
/// this ends one that keeps showing something (owner,
/// 2026-11-04,
/// "Silence
///  plus a ceiling (Recommended)":
/// "Keep ending on silence,
/// and add a generous overall ceiling per tool,
/// measured from real runs,
///  after which it's ended with an error naming the tool.
/// Con:
/// an extremely slow but working run on a huge repository can hit the
///  ceiling").
///  Measured:
/// the add timing log of this repository (79 adds,
/// 2026-11-03T08:55:46Z to 2026-11-04T13:53:17Z;
/// 69 of them hold the
///  per-tool `work` figures):
/// the longest `pnpm install` stage took 53.1 s,
/// the longest `mpm-shadow install` stage 30.0 s,
/// and
///  the longest whole Cargo stage 211.2 s (it holds every `cargo metadata` and `cargo clean` of an add,
/// 34 workspaces at most,
/// so
///  it bounds any one of them).
/// Margin:
/// the smallest whole number of minutes that is at least five times the longest,
/// the factor the
///  service-side ceilings use (`SERVICE_HOLD_CEILING`),
/// so 5 times 211.2 s is 1,056 s and 18 minutes is 1,080 s.
/// The sample is one
///  repository over two days with warm package stores,
/// no cold `pnpm install` of a large dependency graph.
pub const REPAIR_TOOL_CEILING: Duration = Duration::from_secs(18 * 60);

/// The longest the Markdown linter checks one file,
/// whatever it shows,
/// before the wrapper stops waiting for it (owner,
/// 2026-11-04,
///  "Silence plus a ceiling (Recommended)").
/// The linter runs on a thread of the wrapper,
/// which cannot be stopped from outside,
/// so
///  the ceiling is the moment the wrapper abandons the thread,
/// as the quiet window is.
///  Measured:
/// the linter program on the four largest Markdown files of this repository (188 kB to 748 kB,
/// five runs each,
/// through
///  standard input,
/// host load average 3.7 of 28 threads) took 8 to 43 ms;
/// the whole policy gate of a commit,
/// which holds every
///  lint of it,
/// took at most 38,551 ms over 1,838 commits (99th percentile 943 ms).
/// Margin:
/// the smallest whole number of minutes at
///  least five times the longest gate,
/// so 192.8 s is 4 minutes.
/// The program was timed as a separate process,
/// not as the linked
///  thread the wrapper runs;
/// the gate figure is the one that covers the thread.
pub const LINT_CEILING: Duration = Duration::from_secs(4 * 60);

/// The longest the standards-review call to the decisions endpoint runs in all,
/// whatever it shows,
/// before it is ended (owner,
///  2026-11-04,
/// "Silence plus a ceiling (Recommended)").
/// The idle windows of the call stay as they are (10 s in the service,
/// 30 s
///  for the script's default).
///  Measured:
/// the review log's wall-clock cost of 1,598 calls to the endpoint (2026-10-31T01:38:57Z to 2026-11-04T13:57:13Z) had a
///  median of 0.9 s,
/// a 99th percentile of 2.6 s and a longest of 9.8 s.
/// Margin:
/// the smallest whole number of minutes at least five
///  times the longest,
/// so 49 s is 1 minute.
pub const REVIEW_CALL_CEILING: Duration = Duration::from_secs(60);

/// The longest the whole standards review of one landed commit (its Git reads,
/// the call,
/// the write of the entry) runs before the
///  service stops waiting for it (owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)").
/// The review runs on a thread that
///  cannot be stopped from outside,
/// so the service stops waiting and the thread ends with the process.
///  Measured:
/// the service journal's `review_ms` of 1,764 reviews (2026-11-01T12:19:57-05:00 to 2026-11-04T09:01:52-05:00) had a
///  median of 668 ms,
/// a 99th percentile of 2,351 ms and a longest of 18,208 ms. Margin:
/// the smallest whole number of minutes at
///  least five times the longest,
/// so 91 s is 2 minutes.
pub const REVIEW_CEILING: Duration = Duration::from_secs(2 * 60);

/// How much longer a command whose own write is still running waits,
/// after its own bound,
/// before it reports that the result is not
///  known:
/// the client of the commit service whose request is inside the stuck branch write,
/// and a command that started a write in
///  place that cannot be run twice and has outlasted its class's ceiling.
/// The write is left running,
/// never ended.
///  Owner,
/// 2026-11-04,
/// commit protocol revision 12,
/// "Short extra wait (Recommended)":
/// "the command waits a few more seconds,
/// a
///  constant to be measured,
/// for the write it started;
/// if the disk is still stalled it exits non-zero saying the result is not
///  known,
/// naming `git log -1` and the side ref".
/// Owner,
/// 2026-11-04,
/// "7 s (Recommended)":
/// twice the longest measured branch write.
///  Measured:
/// the longest of 2,093 reference updates in the service journal (`answered` lines,
/// `ref_ms`,
/// 2026-11-02T04:01:38-05:00
///  to 2026-11-04T09:11:09-05:00) took 3,438 ms (99th percentile 47 ms);
/// twice that is 6.9 s,
/// 7 s.
pub const SHORT_EXTRA_WAIT: Duration = Duration::from_secs(7);

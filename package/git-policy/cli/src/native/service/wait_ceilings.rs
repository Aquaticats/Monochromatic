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

/// The longest a commit that moved under its own build is built again on the new tip,
/// counted from the first build.
/// It replaces a count of five builds,
/// which ended a commit on 2026-11-08 (release 0.6.1-alpha.3,
/// 270,232 ms in all,
/// five attempts,
/// three of them answered `branch-moved` while other agents landed on the same branch) with the person's change left uncommitted,
/// though nothing conflicted:
/// each slow build gave the branch time to move again.
/// A rebuild is one signature and no lock is held while it runs,
/// so the number of rebuilds is not what has to be bounded,
/// the time is.
/// The bound is `ANSWER_CEILING`:
/// the longest whole commit in the timing log of 2026-11-03 was 750,604 ms (1,329 records),
/// and 20 minutes is 1.6 times that,
/// so a commit that landed in the log never reaches it.
/// Every rebuild says so on standard error with the time so far and this limit.
pub const MOVED_BRANCH_REBUILD_CEILING: Duration = ANSWER_CEILING;

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

/// The ten minutes the foreign-owner patience of another PID namespace is derived from (`process/pid_namespace.rs`:
///  `FOREIGN_OWNER_PATIENCE`,
/// twice this).
/// No wait takes it any more:
/// the landing,
/// the merge,
/// the settling of earlier batches and the
///  index reconciliation each take their own measured ceiling above.
/// It is unmeasured and stays only so that patience keeps its
///  twenty minutes,
/// which another change owns.
pub const LOCK_CEILING: Duration = Duration::from_secs(10 * 60);

/// The longest wait for a foreign program's `index.lock` that ended because the program let go,
/// in the commit service's journal.
///  Measured on 2026-11-07 over the `lock_wait_ms` of the 3,634 `answered` lines of the journal since 2026-11-01:
/// ten landings waited
///  more than a second and ended because the holder let go,
/// at 1,034,
/// 1,110,
/// 1,230,
/// 1,389,
/// 1,516,
/// 1,915,
/// 2,208,
/// 2,483,
/// 5,629 and 8,000 ms;
///  the other waits over a second were one lock that never went (five refusals at 600,090 to 600,718 ms,
/// and one 497,803 ms wait that
///  ended when a person removed the lock).
/// The sample is ten waits on one host.
pub const LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT: Duration = Duration::from_millis(8_000);

/// How many times the longest measured wait a ceiling is,
/// the margin every ceiling of this file uses.
const CEILING_MARGIN: u32 = 5;

/// The smallest whole number of minutes that is at least `wait`.
const fn whole_minutes_at_least(wait: Duration) -> Duration {
    const MINUTE_MILLISECONDS: u128 = 60_000;
    let minutes: u128 = wait.as_millis().div_ceil(MINUTE_MILLISECONDS);
    return Duration::from_secs((minutes * MINUTE_MILLISECONDS / 1000) as u64);
}

/// The longest a landing waits for a foreign program's `index.lock`,
/// and so the longest a commit waits behind one:
/// the longest
///  self-released wait with a margin of five,
/// in whole minutes (40 s,
/// so one minute).
/// It replaces the ten minutes of `LOCK_CEILING`,
///  which was unmeasured and let four queued commits wait 678 to 1,229 s each on 2026-11-07.
/// After a wait this long the commit is
///  withdrawn,
/// kept on a side ref,
/// and the person is told the holder's number,
/// program and start time;
/// the commits queued behind the
///  same lock are refused at once (`landing/service_index_refused_lock.rs`).
/// Limit:
/// ten waits on one host,
/// and a person's own
///  `git commit` with an editor open longer than this makes the wrapper's commit stop where native Git would have stopped at once.
pub const LANDING_LOCK_CEILING: Duration = whole_minutes_at_least(Duration::from_millis(
    LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT.as_millis() as u64 * CEILING_MARGIN as u64,
));

/// The longest whole build of a commit (`buildMs`) in this repository's commit timing log.
/// The build holds the scratch repository's `git commit` step and everything else the client does while it builds,
/// so it can only overstate that step.
/// Measured on 2026-11-08 over the 1,041 records of the log written from 2026-11-05T03:13:38.961Z to 2026-11-08T03:23:41.536Z
/// (median 104 ms,
/// seven above 60 s):
/// the longest was 406,024 ms,
/// of release 0.6.0 at 2026-11-07T18:34:22.063Z.
/// The log's earlier generation (1,112 records,
/// 2026-11-03T18:48:29.620Z to 2026-11-07T15:22:17.144Z,
/// release 0.5.27 to 0.6.0) peaked at 26,729 ms.
/// The 14 records from release 0.6.1-alpha.2 that name each Git step put the `git commit` step at 31 to 1,002 ms,
/// and the owner's signing probe on 2026-11-07 signed a commit in 16 ms against 9 ms unsigned,
/// so neither signing nor hooks (the step runs with every hook off) explains the long builds,
/// and the records that hold them name no step.
/// The ceiling is looser than the step's own measurement for that reason,
/// so it stops no commit that landed in the log.
pub const SLOWEST_MEASURED_COMMIT_BUILD: Duration = Duration::from_millis(406_024);

/// How many times the slowest measured build the ceiling of the scratch `git commit` is (owner,
/// 2026-11-07,
/// "limit unless editor":
/// "a ceiling of three times the slowest measured signing time").
/// The owner's wording names signing,
/// and signing measured about 5 ms of a step that took 16 ms,
/// so three times that would stop a commit that took a second under load;
/// the coordinator's pick is to apply the owner's factor to the whole step's measured time,
/// as `SLOWEST_MEASURED_COMMIT_BUILD` bounds it from above (open to the owner's veto).
pub const SCRATCH_COMMIT_STEP_MARGIN: u32 = 3;

/// The longest the scratch repository's `git commit` runs when the message came with the command (`-m`,
/// `-F`,
/// `-C`),
/// before the wrapper ends it with everything below it:
/// the slowest measured build times the owner's factor,
/// in whole minutes (1,218.07 s,
/// so 21 minutes).
/// With an editor open the step has no ceiling,
/// only the waiting notice:
/// the person is the wait,
/// and a message typed into an editor would be lost by a stop.
/// A stop loses nothing:
/// the step runs in a scratch repository on a private copy of the index,
/// so the person's index,
/// files and branch are as they were,
/// and the message is on the command line to run again.
/// Limit:
/// the figure bounds the whole build,
/// not the step,
/// because the log that holds the long builds names no step;
/// two weeks of step records would tighten it.
pub const SCRATCH_COMMIT_STEP_CEILING: Duration = whole_minutes_at_least(Duration::from_millis(
    SLOWEST_MEASURED_COMMIT_BUILD.as_millis() as u64 * SCRATCH_COMMIT_STEP_MARGIN as u64,
));

/// The longest the settling of an earlier service's open batches took in the commit service's journal:
/// the `settle_ms` of the
///  `answered` lines since 2026-11-01 (3,630 lines,
/// 150 of them above zero,
/// 99th percentile 20 ms,
/// longest 172 ms).
/// The settle runs before
///  each batch and waits for a foreign `index.lock` only when an earlier batch's landing meets one;
/// no settle in that journal waited
///  longer than 172 ms for anything.
pub const LONGEST_SETTLE_WAIT: Duration = Duration::from_millis(172);

/// The longest the settling of an earlier service's batches waits for a foreign `index.lock`:
/// five times `LONGEST_SETTLE_WAIT`,
///  in whole minutes (0.86 s,
/// so one minute).
/// The sample is the settle's whole time,
/// which holds any wait it made,
/// so it can only
///  overstate the wait;
/// no settle in the journal waited on a foreign lock for any time worth naming.
pub const SETTLE_LOCK_CEILING: Duration = whole_minutes_at_least(Duration::from_millis(LONGEST_SETTLE_WAIT.as_millis() as u64 * CEILING_MARGIN as u64));

/// The longest a merge waits for a foreign `index.lock` before it changes anything:
/// the ceiling of the other foreign-holder wait
///  that was measured,
/// the landing's (five times 8 s,
/// one minute).
/// Unmeasured on its own:
/// a merge's wait is on the person's terminal
///  and no log keeps its length,
/// so the landing's population of holders (another program's `git`,
/// an editor-held commit) is the
///  sample.
/// A person's own `git commit` with an editor open longer than this makes the merge stop where native Git would have
///  failed at once.
pub const MERGE_LOCK_CEILING: Duration = LANDING_LOCK_CEILING;

/// The longest the index reconciliation of a commit whose corrections removed every change waits for a foreign `index.lock`:
/// the
///  landing's ceiling,
/// for the same reason as the merge's;
/// the reconciliation runs in the client and logs no duration.
pub const RECONCILE_LOCK_CEILING: Duration = LANDING_LOCK_CEILING;

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
/// ...) runs before the wrapper ends it,
/// even while it shows signs of work.
/// The commit Git builds in a scratch repository is not one of them:
/// it runs with the terminal attached and has its own ceiling,
/// `SCRATCH_COMMIT_STEP_CEILING`.
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

/// The ceilings are derived and checked as controls in the test module.
#[cfg(test)]
#[path = "wait_ceilings_tests.rs"]
mod tests;

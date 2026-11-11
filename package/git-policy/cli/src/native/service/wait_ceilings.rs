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

/// The longest a command waits for the greeting of a service it only has to tell about work whose record it has
/// already written:
/// the trash an add sets aside,
/// the subvolume a remove renamed out of the way,
/// and a push record.
///  Why:
///   The record is durable and every later command of the repository wakes the same service,
/// so an answer that does not come costs one later wake and never the work;
/// this wait is what a person waits for nothing,
/// and the owner's rule is that no command waits on a service it does not need (`AGENTS.md`,
/// "Agents should never wait on each other on commits or merges").
/// A command whose next step depends on the answer (a relink or a prewarm the command must otherwise run itself)
/// keeps the longer `START_WAIT`.
///  Measured:
/// the slowest of 52 cold starts greeted in 1,041 ms (`START_WAIT`),
/// and a service that is already running greets in well under a millisecond,
/// so two seconds is twice the slowest start this repository has recorded.
pub const HAND_OVER_GREETING_WAIT: Duration = Duration::from_secs(2);

/// How long a commit lets a service the manager reports as running stay silent before it asks the manager to restart
/// it.
///  Why:
///  A service greets a connection before it restores anything
/// (`service_server_greets_before_the_journal_is_restored_tests.rs`),
/// so a unit the manager calls `active` that has greeted nothing is not starting:
/// it is stopped inside its own process (a frozen control group,
/// a debugger,
/// a suspended machine,
/// a kernel wait
/// nothing ends),
/// and no start brings it back while the manager believes it runs.
/// Restarting is the one repair of that state,
/// and the commit is what needs the answer.
///  Measured:
/// the slowest of 52 cold starts greeted in 1,041 ms (`START_WAIT`),
/// so three seconds is nearly three times the slowest start,
/// and it is under the owner's 5,000 ms bound on what a commit may add to native Git
/// (`native:perf`,
/// "commits vs native git should not take 5s longer").
pub const SERVICE_GREETING_SILENCE: Duration = Duration::from_secs(3);

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
/// Read again on 2026-11-10 as the look itself makes it,
/// with the host's load average at 56 of 8 threads:
/// 40 samples of the socket table,
/// the process table and one read of every command line of the user's 617 to 625 processes
/// (1,336 processes of all users were listed),
/// 23.0 ms at the median,
/// 43.0 ms at the 90th percentile and 54.1 ms at the slowest;
/// the command-line reads took 16.9 ms at the median and 35.7 ms at the slowest,
/// and the socket table 3.5 to 6.2 ms.
/// The slowest read of one process took 6.4 ms.
/// Ten seconds is 185 times the slowest whole look read here:
/// it is the bound a read that waits for a process stuck in the kernel needs,
/// never what an ordinary look costs.
/// What does not end at this bound is a command line the look cannot prove:
/// `landing/earlier_service_process_look.rs` reads up to the kernel's own ceiling on a command line,
/// and a process it may not read at all holds the look back until that process ends
/// (`doc/troubleshooting/cli-git-settle-look-could-not-prove-a-process.md`).
pub const PROCESS_LOOK_CEILING: Duration = Duration::from_secs(10);

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
/// Read again on 2026-11-08T18:43-05:00 over the journal's 4,386 `answered` lines
/// (both the pre-rename `cli-git-service` identifier and `monochromatic-git-policy-cli-service`):
/// 71 landings waited at all for the holder's place,
/// and the longest that ended when the lock was released is still 8,000 ms
/// (2026-11-02T23:59:30-05:00;
/// the line names no holder,
/// so it says only that something let go).
/// The whole acquisition (`lock_wait_ms`) reaches 11,490 ms
/// (2026-11-08T18:35:52-05:00)
/// because it also holds the wrapper's own staging of a cold index:
/// 9,467 of those ms.
/// Five times either is under 60 s,
/// so the one minute derived below holds on both readings.
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
/// It replaces an unmeasured ten minutes,
///  which let four queued commits wait 678 to 1,229 s each on 2026-11-07.
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
/// Read again on 2026-11-08T18:43-05:00 (see `LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT`):
/// of the journal's 4,386 landings,
/// 20 waited more than a second on the index.
/// Fourteen of them landed
/// (the largest 11,490 ms counted with the wrapper's own staging of a cold index,
/// 8,000 ms at the holder's place),
/// one answered `branch-moved`,
/// one more landed only when a person removed the lock (497,482 ms at the place),
/// and five were refused at 600,090 to 600,718 ms by the lock that never went.
/// The ceiling is 5.2 times the largest landed wait and 7.5 times the largest place wait.
pub const LANDING_LOCK_CEILING: Duration = whole_minutes_at_least(Duration::from_millis(
    LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT.as_millis() as u64 * CEILING_MARGIN as u64,
));

/// The longest a landing waits for a reference lock that refused its branch write.
///  Why:
///   Git refuses a reference update at once when a lock file stands,
///  and a live holder lets go within its own step:
///  the longest reference step of the service's journal took 3,438 ms
///  (`stale_ref_lock_judge.rs`,
///  `LONGEST_LIVE_REF_STEP`),
///  and a lock younger than twice that is not even judged,
///  because a live Git may still be filling it (`stale_ref_lock_judge.rs`,
///  `MARGIN`).
///  Waiting exactly that margin lets a live holder finish and lets the lock be judged at the end of the wait,
///  so the commit lands instead of failing,
///  and a lock nothing proves gone is named with the command that clears it.
///  It replaces no wait:
///  the write was refused at once before,
///  and the commit failed after three refusals read as a moved branch,
///  which took 17 to 28 seconds and said nothing true
///  (2026-11-09,
///  a killed `git update-ref --stdin` holding `refs/heads/main.lock`).
pub const REFERENCE_LOCK_WAIT: Duration = crate::stale_ref_lock_judge::MARGIN;

/// The slowest `git commit` step of the scratch repository in this repository's commit timing log,
/// one figure per record,
/// of the records that do not run past the 16 s the owner chose.
/// Measured on 2026-11-09 from both generations of the timing log
/// (`monochromatic-git-policy-cli-commit-timing.jsonl` and its one rotated generation,
/// 958 records,
/// 2026-11-08T03:29:55Z to 2026-11-09T04:33:32Z,
/// releases 0.6.1-alpha.2 to 0.6.1-alpha.5,
/// 907 of them naming each Git step):
/// twelve records hold a step past the 3,006 ms ceiling this replaces,
/// the three slowest hold 26,256,
/// 36,651 and 171,383 ms,
/// and the slowest of the rest holds 5,444 ms.
/// The owner was shown the same figures over the 886 records the log held then and answered
/// (2026-11-08,
/// "About 16 seconds"):
/// "About 16s,
/// and investigate what happened for those 3 commits.
/// If we can't figure it out,
/// then we have a defect and should log more."
/// The 5,444 ms figure is a step the 3,006 ms ceiling ended,
/// so it holds the stop's own cost (2.4 s of it),
/// and the next reading of an unstopped population may be larger.
/// The owner's signing probe on 2026-11-07 signed a commit in 16 ms against 9 ms unsigned,
/// and the step runs with every hook off,
/// so neither signing nor hooks is what makes a step long when nothing competes.
/// Whole builds that took 11 to 406 s on 2026-11-07 are in the log too,
/// but they name no step;
/// a retry's private index without stat data made the scratch commit hash the whole work tree,
/// and that is fixed (`initialize_retry_index`).
pub const SLOWEST_MEASURED_COMMIT_STEP: Duration = Duration::from_millis(5_444);

/// How many times the slowest measured step the ceiling of the scratch `git commit` is (owner,
/// 2026-11-07,
/// "limit unless editor":
/// "a ceiling of three times the slowest measured signing time"),
/// applied to the step's own record (owner,
/// 2026-11-08,
/// "About 3 s now":
/// "Three times the slowest step record (1,002 ms)";
/// and again,
/// "About 16 seconds":
/// "three times the slowest ordinary step").
pub const SCRATCH_COMMIT_STEP_MARGIN: u32 = 3;

/// The longest the scratch repository's `git commit` runs when the message came with the command (`-m`,
/// `-F`,
/// `-C`),
/// before the wrapper ends it with everything below it:
/// three times the slowest step record,
/// 3 x 5,444 ms = 16,332 ms,
/// which the owner chose as "about 16 s" (2026-11-08,
/// "About 16 seconds").
/// The owner chose it knowing its price,
/// which the option named:
/// on the log of 2026-11-08 it still refuses the three commits whose step ran 26,256,
/// 36,651 and 171,383 ms,
/// two of which landed.
/// With an editor open the step has no ceiling,
/// only the waiting notice:
/// the person is the wait,
/// and a message typed into an editor would be lost by a stop.
/// A stop loses nothing:
/// the step runs in a scratch repository on a private copy of the index,
/// so the person's index,
/// files and branch are as they were,
/// the message is on the command line,
/// and the same command can be run again.
/// `CLI_GIT_SCRATCH_COMMIT_CEILING_SECONDS` sets another ceiling in whole seconds.
pub const SCRATCH_COMMIT_STEP_CEILING: Duration = Duration::from_millis(SLOWEST_MEASURED_COMMIT_STEP.as_millis() as u64 * SCRATCH_COMMIT_STEP_MARGIN as u64);

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
/// Read again on 2026-11-08T18:43-05:00 over 4,386 `answered` lines:
/// 4,214 settles are zero and 172 are above it,
/// the 99th percentile is 17 ms and the longest is still 172 ms.
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
/// The commit timing log holds no settle at all (the settling is the management command
/// `service settle-earlier-batches`),
/// so the journal stays its only sample.
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
///  Checked against the commit timing log on 2026-11-08 (1,332 records,
/// 2026-11-05T03:13:38Z to 2026-11-08T20:59:20Z):
/// the log holds no record of a merge at
///  all,
/// though 278 merge commits were made in that window,
/// because the record is written by the commit command alone.
/// In the borrowed
///  population the longest wait that ended because the holder let go was under 23,783 ms of answer time,
/// under two fifths of
///  this ceiling,
/// and the ten minutes this replaces is 25 times that.
/// A person's own `git commit` with an editor open longer than this makes the merge stop where native Git would have
///  failed at once.
/// Read again on 2026-11-08T18:43-05:00 over 1,608 records
/// (2026-11-05T03:13:38Z to 2026-11-08T23:49:56Z):
/// still no record of a merge,
/// so the merge's wait keeps the landing's population as its sample,
/// whose 8,000 ms place wait is an eighth of this ceiling.
pub const MERGE_LOCK_CEILING: Duration = LANDING_LOCK_CEILING;

/// The longest the index reconciliation of a commit whose corrections removed every change waits for a foreign `index.lock`:
/// the
///  landing's ceiling,
/// for the same reason as the merge's;
/// the reconciliation runs in the client and logs no duration.
///  Checked against the commit timing log on 2026-11-08 (1,332 records):
/// no record names the `git reset` this step runs,
/// so the
///  reconciliation did not run in that window;
/// the preparation phase it belongs to took 21,878 ms at the 99th percentile and 230,525 ms at
///  most,
/// and that phase holds it and everything else a policy does.
/// Read again on 2026-11-08T18:43-05:00:
/// none of the 1,608 records names a `git reset` step,
/// so the reconciliation has not run in the log yet and keeps the landing's population as its sample.
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
/// which the landing's ceiling and the step bounds limit;
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

/// The longest the after-commit work waits for one step that reads or writes state which is not a Git command:
/// the plan of the pushes waiting for a new try,
/// the applied changes of a round,
/// the note of a repository to look at when the service starts again,
/// and the common directory that names a round's lane.
/// The owner's rule is "nothing should ever wait forever or indefinitely",
/// and such a step can wait in the kernel with no end on a filesystem that stopped answering,
/// so the wait is bounded and the work is planned again by the next round.
/// No measurement fixes the value:
/// it is the wait a client's own start is granted (`START_CEILING`),
/// because a state step that outlasts it is one the round after it serves better.
pub const AFTER_COMMIT_STATE_CEILING: Duration = START_CEILING;

/// The longest a landing's own state step waits:
/// the read of a worktree's `HEAD` file and of the `commondir` that proves the directory is that
///  worktree's,
/// which `landing/service_head.rs` reads as a job of the blocking pool.
/// The owner's rule is "nothing should ever wait forever or indefinitely",
/// and such a read waits in the kernel on a mount that stopped answering,
/// so the wait is bounded and a read that gives no answer is reported through Git's own answer.
/// No measurement fixes the value:
/// it is `START_CEILING`,
/// the wait a state step of the after-commit work is granted (`AFTER_COMMIT_STATE_CEILING`),
/// because both are the same kind of read and a step that outlasts either is one the answer Git
///  gives serves better.
pub const LANDING_STATE_CEILING: Duration = START_CEILING;

/// The longest the route decision reads the file system for:
///  the walk up from the start directory and the files it reads,
///  which `landing/service_route.rs` reads as a job of the blocking pool.
/// The owner's rule is "nothing should ever wait forever or indefinitely",
/// and such a walk waits in the kernel on a mount that stopped answering,
/// so the wait is bounded and a walk that gives no answer is reported through Git's own answer.
/// No measurement fixes the value:
/// it is `START_CEILING`,
/// the wait a landing's own state step is granted (`LANDING_STATE_CEILING`),
/// because both are the same kind of read and a step that outlasts either is one the answer Git
///  gives serves better.
pub const ROUTE_STATE_CEILING: Duration = START_CEILING;

/// How long one package-manager tool of a worktree repair may run before the command says which tool it waits for,
/// where,
/// and until when.
///  Why:
///  A tool that hangs showed nothing at all:
/// an add whose `pnpm` never ended printed its last line at two seconds and nothing for the next 613 s,
/// though the quiet window that ends it and the ceiling above that were both running.
/// The line is a notice threshold,
/// not a bound:
/// nothing ends at it,
/// and the quiet window (600 s) and `REPAIR_TOOL_CEILING` (18 minutes) stay what ends the tool.
///  Measured:
/// the longest `pnpm install` stage of the add timing log took 53.1 s and the longest `mpm-shadow install` stage 30.0 s
/// (`REPAIR_TOOL_CEILING`),
/// so a tool still running after two seconds is one the person is waiting for and should hear about;
/// two seconds is twice the slowest of 52 cold service starts,
/// under the owner's 5,000 ms bound on what a command may add (2026-11-09).
pub const REPAIR_TOOL_NOTICE_AFTER: Duration = Duration::from_secs(2);

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

/// The longest one step of the cheap-worktree ladder that owns the checkout's conversion filters (`git read-tree`)
/// runs before the wrapper ends it with a `SIGKILL` to Git,
/// which leaves the filters Git started to run,
/// as a Git the person kills does.
/// It is `OBJECT_STEP_CEILING`,
/// the class of a bulk Git step that can be run twice,
/// because such a step writes only the wrapper's own stage:
/// a stop leaves the snapshot partly updated,
/// the rung ends,
/// and Git's own checkout or the next add does the work again.
/// Measured:
/// the add timing log of this repository
/// (`.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl`,
/// 338 records):
/// the step that holds `git read-tree -m -u` (`applyDifference`) took at most 2,560 ms over 81 adds,
/// and the one that holds `git read-tree --reset -u`,
/// the `clean` and the proof (`restoreDirty`),
/// at most 24,989 ms over 229.
/// Ten minutes is 24 times the longer of the two,
/// and it is the ceiling of every other bulk step of the wrapper.
pub const LADDER_READ_TREE_CEILING: Duration = OBJECT_STEP_CEILING;

/// The longest a whole checkout the wrapper runs itself (the fallback `git reset --hard` of a refused rung)
/// runs before the
///  wrapper ends it with a `SIGKILL` to Git,
/// which leaves the helpers Git started to run,
/// as a Git the person kills does.
/// Measured:
/// the add timing log of this repository (338 records):
/// the longest whole `git worktree add`,
/// which writes a whole checkout,
/// took 229,386 ms.
/// Margin:
/// the smallest whole number of minutes that is at least five times that (1,146,930 ms),
/// the factor the ceilings above use,
/// so 20 minutes.
/// A stopped checkout loses nothing of the person's:
/// it writes the new
///  worktree,
/// whose files are the ones Git's own checkout had just written,
/// and the source is never written.
/// The report of an interrupted `git worktree add` then names the worktree,
/// the index lock a stopped
///  Git leaves (when it stands),
/// and the two commands that finish or remove it.
pub const LADDER_CHECKOUT_CEILING: Duration = whole_minutes_at_least(Duration::from_millis(229_386 * CEILING_MARGIN as u64));

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

/// How long a client keeps looking for the commit a request it already sent made,
/// after the answer was lost,
/// before it says the result is not known.
///  The look reads the service's journal for the request's own batch and the branch it moved,
/// so it turns a lost answer into the
///  commit that was made instead of a re-run the person has to make,
/// and the words that say nothing is known stay for the case where it finds nothing.
///  Measured:
/// the audit of service restarts and version skew (2026-11-09,
///  `~/temp/agent/sonnet-audits/service-restart-and-version-skew/final.md`)
/// saw the honest "not known whether the commit was made" after 5 to 9 s of client wait,
/// with the commit landing seconds later,
///  and a restarted service's recovery of a killed batch took 4 to 21 s (8.3 s in one run).
/// Ten seconds is twice the owner's 5 s bound on what a commit may add to native Git,
/// the derivation
///  `WAITING_NOTICE_AFTER` uses,
/// and it covers the 5 to 9 s the client used to give up inside.
/// The look waits only while an open batch names the request,
/// so a request nothing is landing costs one look and no wait.
pub const LOOKING_FOR_A_LANDED_COMMIT: Duration = Duration::from_secs(10);

/// How long the record of a batch that landed is kept after it was written,
/// because a client of that batch may still read it or send its request again.
/// Why:
///   A client whose answer was lost reads the finished record to learn which commit its request became
/// (`service_commit_looking_for_what_landed.rs`),
/// and it sends the same request again to be told it already landed,
/// so the record must outlive every client that can still be inside those two steps.
/// A client's whole wait for an answer ends at `ANSWER_CEILING`,
/// counted from its first send and longer than any batch takes to land,
/// and it looks for `LOOKING_FOR_A_LANDED_COMMIT` after that;
/// the `SHORT_EXTRA_WAIT` inside the ceiling is what a client inside a stuck write waits on top of it.
/// A record older than the sum cannot be read by any client of it,
/// and the cleanup that removes a removed worktree's journal may then take it.
pub const A_FINISHED_RECORD_IS_READ: Duration = ANSWER_CEILING.saturating_add(SHORT_EXTRA_WAIT).saturating_add(LOOKING_FOR_A_LANDED_COMMIT);

/// How long a command that stopped an instance's service waits for the manager to report its processes gone
/// before it leaves the instance's journal to a later sweep.
/// Why:
///   A stop call returns when the job is queued (`systemd_dbus.rs`),
/// so the journal may still be written while the report says the stop was taken,
/// and removing it then can delete a record the service published in between
/// or make its next write fail.
///  Measured:
/// nothing measures how long a service with nothing in flight takes to end after a stop;
/// the service's own bound on settling a stop is `SETTLE_CEILING` (tens of seconds),
/// which a worktree removal must not wait for.
/// Two seconds is a pick under this brief,
/// and a service that outlasts it keeps its journal and the files that name it,
/// which the next sweep removes once the manager reports it gone.
/// Open to the owner's veto.
pub const SERVICE_GONE_AFTER_A_STOP: Duration = Duration::from_secs(2);

/// How long a commit waits before the wrapper says where it stands:
/// the place the service reported,
/// or that the service has not started on it,
/// and the words of a write the command's own request is inside.
/// A notice threshold,
/// not a bound:
/// nothing ends at it,
/// and nothing waits for it,
/// because the client reads its own clock each time the service sends anything.
///  Measured:
/// the answer wait of this repository's commit timing log (both generations,
/// 919 records,
/// 2026-11-09T06:56:21Z to 2026-11-09T14:59:05Z,
/// release 0.6.1-alpha.5,
/// 53 worktrees,
/// 909 of them stating one) has a median of 125 ms,
/// a 95th percentile of 5,459 ms and a 99th percentile of 19,840 ms;
/// 26 reach 10,000 ms (2.9%),
/// 19 reach 15,000 (2.1%) and 10 reach 20,000 (1.1%).
/// The two commits of the incident this notice answers waited 31,066 and 31,124 ms with nothing said about either.
/// Ten seconds is twice the owner's 5 s bound on what a commit may add to native Git,
/// so a commit that finishes quickly says nothing,
/// and 97 of 100 commits say nothing.
/// A branch write is what the words of a write inside the service are about,
/// and it is shorter than that by far:
/// the 6,703 landings this host's service journal recorded from 2026-11-01 to 2026-11-11 took 5 ms at the median,
/// 601 ms at the 99th percentile and 4,785 ms at the 99.9th,
/// and 6 of them reached 5,000 ms,
/// so a commit whose write is a normal one lands before this wait is over and says nothing of it.
pub const WAITING_NOTICE_AFTER: Duration = Duration::from_secs(10);

/// The environment variable that sets how long a commit waits before the wrapper says where it stands,
/// in whole
///  seconds.
/// Why:
///   A control that checks what a healthy commit prints must not depend on how slowly the host runs one,
/// so it sets
///  this past its own bound,
/// as `CLI_GIT_WAIT_NOTICE_SECONDS` does for the holder's lines.
pub const WAITING_NOTICE_VARIABLE: &str = "CLI_GIT_STANDING_NOTICE_SECONDS";

/// How long a commit waits before the wrapper says where it stands:
/// the setting when it holds a positive number of seconds,
/// `WAITING_NOTICE_AFTER` otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitingNoticeAfter() // Duration
/// ```
pub fn waiting_notice_after() -> Duration {
    return notice_after_of(std::env::var(WAITING_NOTICE_VARIABLE).ok().as_deref());
}

/// What:
///  The wait a setting of `CLI_GIT_STANDING_NOTICE_SECONDS` stands for.
/// Why:
///   A setting that is missing,
/// is not a number or is zero leaves the shipped wait,
/// so a misspelling never silences the telling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noticeAfterOf('30') // 30 s
/// ```
pub fn notice_after_of(setting: Option<&str>) -> Duration {
    let seconds: Option<u64> = match setting {
        Some(text) => text.trim().parse::<u64>().ok(),
        None => None,
    };
    return match seconds {
        Some(found) if found > 0 => Duration::from_secs(found),
        _ => WAITING_NOTICE_AFTER,
    };
}
/// The longest the optional configuration warning of a commit may take to reach the stream it is written to.
///  The warning is written on the commit's preparation path before the request is submitted,
/// and a stream whose reader
///  has stopped draining waits in the kernel until it resumes.
/// Owner,
/// 2026-11-03:
/// "nothing should ever wait forever or indefinitely",
/// with the commit delta the
///  owner allows at 5,000 ms (`native:perf`,
/// "commits vs native git should not take 5s longer"),
/// so this value leaves
///  4,750 ms of that slack untouched.
///  Measured on this host,
/// 2026-11-09:
/// writing one line of 1 KiB to a pipe whose reader drains it took
///  0.006 ms at the median and 3.6 ms at the slowest of 200 writes,
/// so 250 ms is about 39,000 times the median and 69 times
///  the slowest of them.
/// A write that is still waiting at this value is abandoned and logged;
/// the commit goes on,
///  because the line
///  is optional beside the command's own result.
pub const WARNING_WRITE_CEILING: Duration = Duration::from_millis(250);

/// The longest a read of the health report's own file system facts may take:
/// the hook directories it scans,
/// the shim
///  files it compares,
/// and the checkout's `statfs` and mount table.
///  Those calls wait in the kernel on a mount whose server has stopped answering,
/// and the owner's rule is
///  "nothing should ever wait forever or indefinitely".
/// Doctor runs on request,
/// never on the commit path,
/// so its
///  ceiling may be generous;
/// what it must not do is wait with no end.
///  Measured on this host,
/// 2026-11-09:
/// listing 50 entries of a hooks directory and statting each took 0.36 ms at the
///  median and 0.79 ms at the slowest of 200 scans,
/// so 5 s is about 14,000 times the median and 6,300 times the
///  slowest of them.
pub const REPORT_SCAN_CEILING: Duration = Duration::from_secs(5);

/// The longest a `git worktree add` waits for the snapshot the commit service makes ahead of it.
///  The ceiling covers the whole wait,
/// not one look:
/// the add asks the service for a snapshot,
///  looks for what it published,
/// and ends the snapshot rung when this passes so that Git's own checkout makes the
///  worktree.
///  Measured on this volume,
/// the 473 adds of this repository's own add timing log to
///  2026-11-11T01:53:18.713Z spent 5,413.0 ms in the snapshot ioctl at the median,
///  15,238.4 ms at the
///  ninetieth percentile and 39,828.5 ms at the worst (402 of them recorded that step),
/// so 1.5 times the
///  worst measured call is about an hour of this volume's worst traffic,
/// and it is the owner's own bound for an add:
///  "60s is fine for adding worktrees".
/// A wait that reaches it costs the add Git's own checkout of the tree,
/// which
///  is what a worktree add does on any filesystem that cannot snapshot,
/// and the command says why it took that route.
pub const SNAPSHOT_AHEAD_CEILING: Duration = Duration::from_secs(60);

/// The ceilings are derived and checked as controls in the test module.
#[cfg(test)]
#[path = "wait_ceilings_tests.rs"]
mod tests;

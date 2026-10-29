//! Controls for the index-lock wait with a fake clock and scripted holders:
//!  the backoff schedule,
//! the caps on a proven holder's polls,
//!  the unproven budget,
//!  and the read failures.

use super::*;
use crate::index_lock_evidence::{LockMetadata, PidFile, PidOwner};
use crate::index_lock_holders::{Holder, HolderEvidence};
use std::path::PathBuf;
use std::time::Duration;

/// A fake wait:
///  scripted presence and evidence,
///  a clock moved by sleeps and gathers,
///  and the record
/// of everything the wait did.
struct FakeEffects {
    /// Presence answers,
    ///  consumed in order;
    ///  the last one repeats.
    presence: Vec<bool>,
    /// Evidence answers,
    ///  consumed in order;
    ///  the last one repeats.
    evidence: Vec<Option<Evidence>>,
    /// Milliseconds one gather adds to the clock.
    gather_cost: f64,
    /// Failure text the presence read returns,
    ///  when set.
    presence_failure: Option<String>,
    /// Failure text the evidence read returns,
    ///  when set.
    gather_failure: Option<String>,
    /// The clock.
    clock: f64,
    /// Every sleep,
    ///  in order.
    sleeps: Vec<u64>,
    /// Every line reported.
    reports: Vec<String>,
    /// How many presence reads happened.
    presence_reads: usize,
    /// How many evidence reads happened.
    gathers: usize,
    /// When the wait repeats its notice.
    cadence: Cadence,
}

impl FakeEffects {
    /// A fake that answers `presence` and `evidence` in order.
    fn new(presence: Vec<bool>, evidence: Vec<Option<Evidence>>) -> Self {
        return FakeEffects {
            presence,
            evidence,
            gather_cost: 0.0,
            presence_failure: None,
            gather_failure: None,
            clock: 0.0,
            sleeps: Vec::new(),
            reports: Vec::new(),
            presence_reads: 0,
            gathers: 0,
            cadence: Cadence::standard(),
        };
    }
}

impl WaitEffects for FakeEffects {
    fn lock_present(&mut self) -> Result<bool, String> {
        if let Some(reason) = &self.presence_failure {
            return Err(reason.clone());
        }
        let index: usize = self.presence_reads.min(self.presence.len() - 1);
        self.presence_reads += 1;
        return Ok(self.presence[index]);
    }

    fn gather(&mut self) -> Result<Option<Evidence>, String> {
        if let Some(reason) = &self.gather_failure {
            return Err(reason.clone());
        }
        self.clock += self.gather_cost;
        let index: usize = self.gathers.min(self.evidence.len() - 1);
        self.gathers += 1;
        return Ok(self.evidence[index].clone());
    }

    fn sleep(&mut self, ms: u64) {
        // A wait that lost its bound fails here instead of looping for good.
        assert!(self.sleeps.len() < 100_000, "the wait never ended");
        self.clock += ms as f64;
        self.sleeps.push(ms);
    }

    fn random(&mut self) -> f64 {
        return 0.5;
    }

    fn now_ms(&mut self) -> f64 {
        return self.clock;
    }

    fn report(&mut self, line: &str) {
        self.reports.push(String::from(line));
    }

    fn cadence(&mut self) -> Cadence {
        return self.cadence;
    }
}

/// Evidence with a PID file naming PID 42 in the given owner state,
///  and no holder scan.
fn pid_evidence(owner: PidOwner) -> Evidence {
    return Evidence {
        lock_path: PathBuf::from("/repo/index.lock"),
        lock: LockMetadata {
            device: 1,
            inode: 2,
            ctime_ms: 1000,
        },
        pid_file: PidFile::Owner(42, owner),
        holders: None,
    };
}

/// Evidence whose holder scan found PID 55 holding the lock open.
fn descriptor_evidence() -> Evidence {
    return Evidence {
        lock_path: PathBuf::from("/repo/index.lock"),
        lock: LockMetadata {
            device: 1,
            inode: 2,
            ctime_ms: 1000,
        },
        pid_file: PidFile::Absent,
        holders: Some(HolderEvidence {
            method: "proc-fd",
            holders: vec![Holder {
                pid: 55,
                command: Some(String::from("git")),
            }],
            partial: Vec::new(),
        }),
    };
}

/// Evidence nothing proves either way:
///  no PID file and no holder.
fn free_evidence() -> Evidence {
    return Evidence {
        lock_path: PathBuf::from("/repo/index.lock"),
        lock: LockMetadata {
            device: 1,
            inode: 2,
            ctime_ms: 1000,
        },
        pid_file: PidFile::Absent,
        holders: Some(HolderEvidence {
            method: "proc-fd",
            holders: Vec::new(),
            partial: Vec::new(),
        }),
    };
}

/// Whether every value is at most `cap`.
fn all_within(values: &[u64], cap: u64) -> bool {
    for value in values {
        if *value > cap {
            return false;
        }
    }
    return true;
}

/// A run of `count` presence answers that say the lock is held,
///  then one that says released.
fn held_then_released(count: usize) -> Vec<bool> {
    let mut answers: Vec<bool> = vec![true; count];
    answers.push(false);
    return answers;
}

/// Git's formula:
///  `(750 + random%500) * multiplier / 1000`,
///  rounded down.
#[test]
fn jittered_waits_follow_gits_formula() {
    let state: BackoffState = BackoffState { step: 3, multiplier: 9 };
    assert_eq!(jittered_wait(state, 0.5), 9);
    assert_eq!(jittered_wait(state, 0.0), 6);
    assert_eq!(jittered_wait(state, 0.999), 11);
}

/// The multiplier grows by `2n + 1` (so it is the square of the step) and stops at the cap.
#[test]
fn backoff_squares_then_stops_at_the_cap() {
    let first: BackoffState = next_backoff(INITIAL_BACKOFF);
    assert_eq!(first, BackoffState { step: 2, multiplier: 4 });
    let second: BackoffState = next_backoff(first);
    assert_eq!(second, BackoffState { step: 3, multiplier: 9 });
    let near_cap: BackoffState = BackoffState { step: 31, multiplier: 961 };
    assert_eq!(next_backoff(near_cap), BackoffState { step: 31, multiplier: BACKOFF_MAX_MULTIPLIER });
    // A step that lands on the cap exactly is still a step; only one that would pass it is held back.
    let lands_on_cap: BackoffState = BackoffState { step: 5, multiplier: 989 };
    assert_eq!(next_backoff(lands_on_cap), BackoffState { step: 6, multiplier: BACKOFF_MAX_MULTIPLIER });
    let at_cap: BackoffState = BackoffState { step: 31, multiplier: BACKOFF_MAX_MULTIPLIER };
    assert_eq!(next_backoff(at_cap), at_cap);
}

/// A lock already gone returns at once,
///  with no evidence read and no sleep.
#[test]
fn a_released_lock_returns_at_once() {
    let mut effects: FakeEffects = FakeEffects::new(vec![false], vec![None]);
    assert_eq!(wait_for_index_lock(&mut effects, 100, "did nothing"), Ok(()));
    assert!(effects.sleeps.is_empty());
    assert_eq!(effects.gathers, 0);
}

/// A live PID-file owner is waited for beyond the unproven budget,
///  reported once,
///  and polled no slower than 100 ms.
#[test]
fn a_proven_pid_file_owner_is_waited_for_beyond_the_unproven_budget() {
    let mut effects: FakeEffects = FakeEffects::new(
        held_then_released(12),
        vec![Some(pid_evidence(PidOwner::StartedBeforeLock(500.0)))],
    );
    assert_eq!(wait_for_index_lock(&mut effects, 1, "did nothing"), Ok(()));
    assert_eq!(effects.sleeps.len(), 12);
    assert!(all_within(effects.sleeps.as_slice(), PROVEN_BY_PID_FILE_POLL_CAP_MS));
    assert_eq!(effects.sleeps.last(), Some(&PROVEN_BY_PID_FILE_POLL_CAP_MS));
    assert_eq!(effects.reports.len(), 1);
    assert!(effects.reports[0].starts_with("cli-git: waiting for PID 42, which holds /repo/index.lock"));
}

/// A descriptor-proven holder is polled up to its own,
///  longer,
///  cap.
#[test]
fn a_descriptor_holder_has_its_own_longer_poll_cap() {
    let mut effects: FakeEffects = FakeEffects::new(held_then_released(12), vec![Some(descriptor_evidence())]);
    assert_eq!(wait_for_index_lock(&mut effects, 1, "did nothing"), Ok(()));
    assert!(all_within(effects.sleeps.as_slice(), PROVEN_BY_DESCRIPTOR_POLL_CAP_MS));
    assert_eq!(effects.sleeps.last(), Some(&144));
    assert!(effects.reports[0].contains("(proven by an open descriptor)"));
}

/// A dead owner is retried on the schedule and fails once the sleeps and attempts spend the budget.
#[test]
fn a_dead_owner_fails_after_the_budget() {
    let mut effects: FakeEffects = FakeEffects::new(
        vec![true],
        vec![Some(pid_evidence(PidOwner::Missing))],
    );
    let failure: WaitFailure = wait_for_index_lock(&mut effects, 100, "did not run git add")
        .expect_err("a dead owner must fail once the budget is spent");
    match failure {
        WaitFailure::UnprovenOwner {
            verdict,
            timeout_ms,
            consequence,
            ..
        } => {
            assert_eq!(verdict, Verdict::Dead(42));
            assert_eq!(timeout_ms, 100);
            assert_eq!(consequence, "did not run git add");
        }
        other => panic!("expected the unproven owner failure, got {other:?}"),
    }
    assert!(effects.sleeps.iter().sum::<u64>() >= 100);
    assert!(effects.reports.is_empty());
}

/// An owner nothing proves alive fails on the same budget,
///  with the verdict that says so.
#[test]
fn an_evidence_free_owner_fails_on_the_same_budget() {
    let mut effects: FakeEffects = FakeEffects::new(vec![true], vec![Some(free_evidence())]);
    let failure: WaitFailure = wait_for_index_lock(&mut effects, 50, "landed nothing")
        .expect_err("an evidence-free owner must fail");
    assert!(matches!(
        failure,
        WaitFailure::UnprovenOwner {
            verdict: Verdict::EvidenceFree,
            ..
        }
    ));
}

/// Time spent gathering evidence counts toward the budget,
///  so one slow scan can exhaust it without a sleep.
#[test]
fn gathering_evidence_counts_toward_the_budget() {
    let mut effects: FakeEffects = FakeEffects::new(vec![true], vec![Some(free_evidence())]);
    effects.gather_cost = 200.0;
    let failure: WaitFailure = wait_for_index_lock(&mut effects, 100, "landed nothing")
        .expect_err("the scan alone spends the budget");
    assert!(matches!(failure, WaitFailure::UnprovenOwner { .. }));
    assert!(effects.sleeps.is_empty());
}

/// A lock released between the attempt and the evidence read is retried without a sleep.
#[test]
fn a_lock_released_before_the_evidence_read_is_retried() {
    let mut effects: FakeEffects = FakeEffects::new(vec![true, false], vec![None]);
    assert_eq!(wait_for_index_lock(&mut effects, 100, "did nothing"), Ok(()));
    assert_eq!(effects.gathers, 1);
    assert!(effects.sleeps.is_empty());
}

/// An unreadable lock or evidence stops the wait with its reason,
///  never a guessed verdict.
#[test]
fn unreadable_state_stops_the_wait_with_its_reason() {
    let mut presence: FakeEffects = FakeEffects::new(vec![true], vec![None]);
    presence.presence_failure = Some(String::from("denied"));
    assert_eq!(
        wait_for_index_lock(&mut presence, 100, "did nothing"),
        Err(WaitFailure::Read(String::from("denied")))
    );
    let mut evidence: FakeEffects = FakeEffects::new(vec![true], vec![None]);
    evidence.gather_failure = Some(String::from("no proc"));
    assert_eq!(
        wait_for_index_lock(&mut evidence, 100, "did nothing"),
        Err(WaitFailure::Read(String::from("no proc")))
    );
}

/// The failure names the lock,
///  the owner state,
///  the budget,
///  what was not done,
///  and the evidence.
#[test]
fn the_unproven_owner_message_names_each_fact() {
    let evidence: Evidence = pid_evidence(PidOwner::Missing);
    let verdict: Verdict = classify_index_lock(&evidence);
    let message: String = unproven_owner_message(&evidence, &verdict, 100, "did not run git add");
    assert!(message.starts_with("Another process left /repo/index.lock, whose owner PID 42 no longer runs;"));
    assert!(message.contains("after 100 ms (indexLock.unprovenOwnerTimeoutMs)"));
    assert!(message.contains("cli-git left the lock in place and did not run git add."));
    assert!(message.contains("Evidence: /repo/index.lock (device 1, inode 2"));
    let unproven: String = unproven_owner_message(&free_evidence(), &Verdict::EvidenceFree, 5, "x");
    assert!(unproven.contains("whose owner cli-git could not prove alive"));
}

/// A proven-alive owner that never lets go ends the wait at the live limit,
///  naming the owner, the waited time and the limit,
///  after one holder line and with polls no slower than the cap.
#[test]
fn a_proven_owner_past_the_live_limit_ends_the_wait() {
    let mut effects: FakeEffects = FakeEffects::new(
        vec![true],
        vec![Some(pid_evidence(PidOwner::StartedBeforeLock(500.0)))],
    );
    let failure: WaitFailure = wait_for_index_lock_within(&mut effects, 1, 1_000, "landed nothing")
        .expect_err("a live owner that never releases ends the wait");
    match failure {
        WaitFailure::LiveHolderTimedOut {
            lock_path,
            pid,
            command,
            waited_ms,
            limit_ms,
            consequence,
        } => {
            assert_eq!(lock_path, PathBuf::from("/repo/index.lock"));
            assert_eq!((pid, command), (42, None));
            assert!((1_000..1_100).contains(&waited_ms), "{waited_ms}");
            assert_eq!(limit_ms, 1_000);
            assert_eq!(consequence, "landed nothing");
        }
        other => panic!("expected the live owner timeout, got {other:?}"),
    }
    assert!(all_within(effects.sleeps.as_slice(), PROVEN_BY_PID_FILE_POLL_CAP_MS));
    assert!(effects.sleeps.iter().sum::<u64>() >= 1_000);
    assert_eq!(effects.reports.len(), 1);
}

/// A descriptor-proven owner ends the wait the same way and keeps its command name.
#[test]
fn a_descriptor_owner_past_the_live_limit_ends_the_wait_with_its_command() {
    let mut effects: FakeEffects = FakeEffects::new(vec![true], vec![Some(descriptor_evidence())]);
    let failure: WaitFailure = wait_for_index_lock_within(&mut effects, 1, 2_000, "did nothing")
        .expect_err("a live owner that never releases ends the wait");
    match failure {
        WaitFailure::LiveHolderTimedOut { pid, command, waited_ms, .. } => {
            assert_eq!(pid, 55);
            assert_eq!(command.as_deref(), Some("git"));
            assert!((2_000..2_500).contains(&waited_ms), "{waited_ms}");
        }
        other => panic!("expected the live owner timeout, got {other:?}"),
    }
    assert!(all_within(effects.sleeps.as_slice(), PROVEN_BY_DESCRIPTOR_POLL_CAP_MS));
}

/// An owner that releases before the live limit is not an error.
#[test]
fn an_owner_that_releases_before_the_live_limit_is_not_an_error() {
    let mut effects: FakeEffects = FakeEffects::new(
        held_then_released(3),
        vec![Some(pid_evidence(PidOwner::StartedBeforeLock(500.0)))],
    );
    assert_eq!(wait_for_index_lock_within(&mut effects, 1, 1_000, "did nothing"), Ok(()));
}

/// The timeout text names the owner and command, the lock, the whole seconds waited, the setting and the consequence.
#[test]
fn the_live_owner_timeout_is_described() {
    let with_command: String = live_holder_timeout_message(
        std::path::Path::new("/repo/index.lock"),
        55,
        Some("git"),
        61_900,
        "landed nothing",
    );
    assert_eq!(
        with_command,
        "Gave up after waiting 61 seconds for PID 55 (git), which still holds /repo/index.lock; \
         CLI_GIT_LOCK_WAIT_SECONDS sets how long cli-git waits. cli-git left the lock in place and landed nothing. \
         Look at that process, or raise the limit."
    );
    let anonymous: String =
        live_holder_timeout_message(std::path::Path::new("/x"), 7, None, 5_000, "did nothing");
    assert!(anonymous.starts_with("Gave up after waiting 5 seconds for PID 7, which still holds /x;"), "{anonymous}");
}

/// A live holder is announced at the start and named again at every interval, with the time waited and the limit;
///  the wait on the fake clock ends at the limit.
#[test]
fn a_live_holder_is_named_again_at_every_interval() {
    let mut effects: FakeEffects = FakeEffects::new(
        vec![true],
        vec![Some(pid_evidence(PidOwner::StartedBeforeLock(500.0)))],
    );
    let failure: WaitFailure = wait_for_index_lock_within(&mut effects, 1, 25_000, "did nothing")
        .expect_err("a live owner outlasts the limit");
    assert!(matches!(failure, WaitFailure::LiveHolderTimedOut { pid: 42, .. }), "{failure:?}");
    assert_eq!(effects.reports.len(), 3, "{:?}", effects.reports);
    assert!(
        effects.reports[0].ends_with("giving up after 25 seconds (CLI_GIT_LOCK_WAIT_SECONDS changes this), leaving the lock in place.\n"),
        "{:?}",
        effects.reports
    );
    assert!(effects.reports[1].starts_with("cli-git: still waiting for PID 42, which holds /repo/index.lock"), "{:?}", effects.reports);
    assert!(effects.reports[1].contains("after 10 seconds; giving up after 25 seconds"), "{:?}", effects.reports);
    assert!(effects.reports[2].contains("after 20 seconds; giving up after 25 seconds"), "{:?}", effects.reports);
}

/// The interval comes from the effects' cadence, so a shorter one repeats the line sooner.
#[test]
fn a_shorter_cadence_repeats_the_holder_line_sooner() {
    let mut effects: FakeEffects = FakeEffects::new(
        vec![true],
        vec![Some(pid_evidence(PidOwner::StartedBeforeLock(500.0)))],
    );
    effects.cadence = Cadence::from_interval(Duration::from_secs(2));
    let _failure: WaitFailure = wait_for_index_lock_within(&mut effects, 1, 7_000, "did nothing")
        .expect_err("a live owner outlasts the limit");
    // The start, then 2 s, 4 s and 6 s.
    assert_eq!(effects.reports.len(), 4, "{:?}", effects.reports);
}

/// A wait behind a dead owner's lock says so after the grace and at every interval, and names the budget.
#[test]
fn a_wait_behind_a_dead_owners_lock_says_so_while_it_retries() {
    let mut effects: FakeEffects = FakeEffects::new(vec![true], vec![Some(pid_evidence(PidOwner::Missing))]);
    let _failure: WaitFailure = wait_for_index_lock(&mut effects, 25_000, "did nothing")
        .expect_err("a dead owner outlasts the budget");
    assert_eq!(effects.reports.len(), 3, "{:?}", effects.reports);
    assert!(effects.reports[0].starts_with("cli-git: still waiting for /repo/index.lock, whose owner PID 42 no longer runs, after 1 seconds;"), "{:?}", effects.reports);
    assert!(effects.reports.iter().all(|line| { return line.contains("retrying until 25000 ms (indexLock.unprovenOwnerTimeoutMs)"); }), "{:?}", effects.reports);
}

/// A wait behind a lock nothing proves either way names that, and prints nothing inside the grace.
#[test]
fn a_wait_behind_an_evidence_free_lock_says_nothing_proves_it_alive() {
    let mut effects: FakeEffects = FakeEffects::new(vec![true], vec![Some(free_evidence())]);
    let _failure: WaitFailure = wait_for_index_lock(&mut effects, 12_000, "did nothing")
        .expect_err("an evidence-free owner outlasts the budget");
    assert_eq!(effects.reports.len(), 2, "{:?}", effects.reports);
    assert!(effects.reports[0].contains("whose owner cli-git cannot prove alive"), "{:?}", effects.reports);
    let mut quick: FakeEffects = FakeEffects::new(vec![true], vec![Some(free_evidence())]);
    let _short: WaitFailure = wait_for_index_lock(&mut quick, 400, "did nothing").expect_err("short budget");
    assert!(quick.reports.is_empty(), "{:?}", quick.reports);
}

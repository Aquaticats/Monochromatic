//! Controls for the renewal of claim leases:
//! what a renewal changes,
//! what it leaves alone,
//! what a settlement reaches,
//! what a settlement that finds the record gone says,
//! and that
//! the thread stops.

use crate::work_item_recovery::requeue_expired;
use super::*;
use crate::service_claim_renewal_running::{ClaimRenewal, Control, DrivenRenewal, RENEWALS_MADE, RENEWAL_STOP_CEILING, TASKS_ENDED, TASKS_STARTED};
use crate::service_work::MAX_TRIES;
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, DONE, FAILED, PENDING, RECOVERING, WorkItem, WorkKind, claim_next_of, enqueue, find_claim, state_directory};
use std::path::Path;

/// The bound that ends a control which waits for a renewal thread to run:
/// a backstop for a thread that never runs,
/// never a host that runs it slowly,
/// because every interval of a control is milliseconds.
const PASS_PATIENCE: Duration = Duration::from_secs(20);

/// How many passes a control watches a renewal thread make.
///
/// The interval of every such control is milliseconds and a pass renames each held claim once,
/// so one pass would be a single look;
/// two show a loop that went round the claims twice.
const PASSES_WATCHED: usize = 2;

/// Waits until the renewal thread has made `count` more passes than `before`,
/// which the renewal itself counts,
/// and gives whether it did within `PASS_PATIENCE`.
fn wait_for_passes(before: usize, count: usize) -> bool {
    let began: std::time::Instant = std::time::Instant::now();
    while RENEWALS_MADE.load(std::sync::atomic::Ordering::SeqCst) < before.saturating_add(count) {
        if began.elapsed() >= PASS_PATIENCE {
            return false;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    return true;
}

/// The number of renewal passes the running renewals of this process have made.
fn passes_made() -> usize {
    return RENEWALS_MADE.load(std::sync::atomic::Ordering::SeqCst);
}

const OID: &str = "0123456789abcdef0123456789abcdef01234567";
const LEASE: Duration = Duration::from_secs(900);
const BOOT: &str = "0123456789abcdef0123456789abcdef";

/// The clocks at the start of every test.
fn clock() -> ClaimTime {
    return ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000));
}

/// A claimed push item in a fresh work directory,
/// claimed at `stamp`;
/// the claim is returned with the directories.
fn claimed_item(name: &str, stamp: &ClaimTime) -> (PathBuf, PathBuf, Claimed) {
    return claimed_item_after_tries(name, stamp, 0);
}

///  A claimed push item whose record says it has already used `tries` attempts.
fn claimed_item_after_tries(name: &str, stamp: &ClaimTime, tries: u32) -> (PathBuf, PathBuf, Claimed) {
    let root: PathBuf = fixture(name);
    let work_dir: PathBuf = root.join("work");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: root.to_path_buf(),
        git_dir: root.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: OID.to_owned(),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries,
    };
    enqueue(work_dir.as_path(), &item).expect("enqueue");
    let claimed: Claimed = claim_next_of(work_dir.as_path(), stamp, &[WorkKind::Push]).expect("claim").expect("one claim");
    return (root, work_dir, claimed);
}

/// The names in `claimed`.
fn claimed_names(work_dir: &Path) -> Vec<String> {
    return std::fs::read_dir(state_directory(work_dir, CLAIMED)).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
}

/// A renewed claim is not taken over at the end of the lease it started with,
/// and the same claim unrenewed is.
#[test]
fn a_renewed_claim_outlives_the_lease_it_started_with() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-outlives", &started);
    let almost_over: ClaimTime = started.advanced(LEASE - Duration::from_secs(1));
    let renewed: String = renew_claim(work_dir.as_path(), claimed.name.as_str(), &almost_over).expect("the claim is still held");
    assert_eq!(claimed_names(work_dir.as_path()), vec![renewed], "the claim is one file, renamed");
    assert_eq!(requeue_expired(work_dir.as_path(), &started.advanced(LEASE + Duration::from_secs(1)), LEASE, MAX_TRIES), 0, "renewed, so still held");
    assert_eq!(requeue_expired(work_dir.as_path(), &almost_over.advanced(LEASE + Duration::from_secs(1)), LEASE, MAX_TRIES), 1, "a holder that stopped renewing loses it");
    remove(root.as_path());
}

/// A renewal finds nothing to renew once the claim has left `claimed`,
/// and never creates a claim.
#[test]
fn renewing_never_brings_back_a_settled_claim() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-settled", &started);
    std::fs::remove_file(state_directory(work_dir.as_path(), CLAIMED).join(claimed.name.as_str())).expect("settled");
    assert_eq!(renew_claim(work_dir.as_path(), claimed.name.as_str(), &started.advanced(Duration::from_secs(5))), None, "nothing is left to renew");
    assert!(claimed_names(work_dir.as_path()).is_empty(), "a renewal does not create the claim");
    remove(root.as_path());
}

/// The held set follows the claims taken and settled,
/// and a settlement moves the claim under the name it has now.
#[test]
fn the_held_set_follows_what_is_taken_and_settled() {
    let (root, work_dir, claimed) = claimed_item("renewal-held", &clock());
    let (held, mut renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    assert_eq!(held.snapshot(), vec![claimed.id.clone()]);
    assert!(held.is_held(claimed.id.as_str()));
    renewer.renew(&clock().advanced(Duration::from_secs(30)));
    let renamed: String = renewer.names().into_iter().next().expect("held");
    assert_ne!(renamed, claimed.name, "the renewed name follows the renewal");
    assert_eq!(claimed_names(work_dir.as_path()), vec![renamed], "and so does the file");
    held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect("settled under the renewed name");
    assert!(held.snapshot().is_empty());
    assert!(find_claim(work_dir.as_path(), claimed.id.as_str()).is_none());
    assert!(state_directory(work_dir.as_path(), DONE).join(claimed.id.as_str()).exists());
    remove(root.as_path());
}

/// A claim another instance took over is found gone by the next renewal,
/// is no longer held,
/// and its settlement says so and
/// moves nothing:
/// not the other instance's new claim of the same item either.
#[test]
fn a_claim_taken_over_is_found_gone_and_settles_nothing_of_the_new_claim() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-taken-over", &started);
    let (held, mut renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    // Another instance finds the lease over, returns the item to pending and claims it.
    let later: ClaimTime = started.advanced(LEASE * 2);
    assert_eq!(requeue_expired(work_dir.as_path(), &later, LEASE, MAX_TRIES), 1);
    let successor: Claimed = claim_next_of(work_dir.as_path(), &later, &[WorkKind::Push]).expect("claim").expect("requeued item");
    renewer.renew(&later);
    assert!(!held.is_held(claimed.id.as_str()), "the renewal found the claim gone");
    assert_eq!(claimed_names(work_dir.as_path()), vec![successor.name.clone()], "the renewal did not touch the successor's claim");
    let refused: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a taken-over claim settles nothing");
    assert!(refused.contains("taken over"), "{refused}");
    assert_eq!(claimed_names(work_dir.as_path()), vec![successor.name], "the successor's claim stays where it is");
    assert!(!state_directory(work_dir.as_path(), DONE).join(claimed.id.as_str()).exists());
    remove(root.as_path());
}

/// A holder whose renewal has not run since another instance took its claim and claimed the item again finds,
/// by the
/// generation in the file name,
/// that the file in `claimed` is not its own:
/// it is not held,
/// and its settlement moves nothing.
#[test]
fn a_claim_claimed_again_by_another_instance_is_not_this_holders_though_its_id_is_in_claimed() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-claimed-again", &started);
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let later: ClaimTime = started.advanced(LEASE * 2);
    assert_eq!(requeue_expired(work_dir.as_path(), &later, LEASE, MAX_TRIES), 1);
    let successor: Claimed = claim_next_of(work_dir.as_path(), &later, &[WorkKind::Push]).expect("claim").expect("requeued item");
    assert_eq!(successor.id, claimed.id, "the same item, claimed again");
    assert!(!held.is_held_on_disk(work_dir.as_path(), claimed.id.as_str()), "the file of that id in claimed is the other instance's");
    held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("nothing of the other instance's claim is settled");
    assert_eq!(claimed_names(work_dir.as_path()), vec![successor.name], "the successor's claim stays where it is");
    remove(root.as_path());
}

/// The renewal thread renames the held claims by itself,
/// and is gone when the renewal is dropped:
/// the drop waits for the task to end,
/// so the task's own ending is the witness that nothing can rename afterwards.
///  The names are read after that ending and never across it:
/// a task that made one more pass before it read the stop was renewing correctly,
/// and
///  a name taken before the drop would have compared the claim against a pass that was still allowed to happen.
#[test]
fn a_running_renewal_renews_the_held_claims_until_its_stop_is_awaited() {
    let (root, work_dir, claimed) = claimed_item("renewal-thread", &clock());
    let (held, renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let renewal: ClaimRenewal = ClaimRenewal::start(renewer, Duration::from_millis(5));
    // The bound only ends a failing run: a passing run leaves the loop as soon as the thread has renewed the claim.
    let bound: std::time::Instant = std::time::Instant::now() + PASS_PATIENCE;
    while claimed_names(work_dir.as_path()) == vec![claimed.name.clone()] && std::time::Instant::now() < bound {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_ne!(claimed_names(work_dir.as_path()), vec![claimed.name.clone()], "the thread renewed the held claim");
    // The stop says whether it waited for the task or only for its ceiling, and the task says it returned, so no clock is
    //  asked whether anything can rename after this line.
    let ended_before: usize = TASKS_ENDED.load(std::sync::atomic::Ordering::SeqCst);
    let stopped = crate::service_timing::capture::entries_of(|| {
        crate::lazy_runtime::run_on_runtime(renewal.stop_awaiting()).expect("the runtime runs the stop");
    });
    assert!(stopped.is_empty(), "the renewal's task ended with the stop, so nothing can rename after it: {stopped:?}");
    assert_eq!(TASKS_ENDED.load(std::sync::atomic::Ordering::SeqCst), ended_before.saturating_add(1), "the task returned before the stop did");
    // What the returned task left: one claim, under a renewal's name. A pass it made before it read the stop is its own work
    //  and not a rename after it, so no name is compared with one taken before the drop.
    let after: Vec<String> = claimed_names(work_dir.as_path());
    assert_eq!(after.len(), 1, "the drop left one claim: {after:?}");
    assert_ne!(after, vec![claimed.name.clone()], "the claim the task left is a renewal's: {after:?}");
    remove(root.as_path());
}

/// A claim the renewal does not hold is left to its lease,
/// so a dead holder's claim still expires.
/// The renewal's own count of its passes is what shows the thread ran;
/// the claim's name is what shows it was left alone.
#[test]
fn a_claim_that_is_not_held_is_not_renewed() {
    let (root, work_dir, claimed) = claimed_item("renewal-not-held", &clock());
    let (held, renewer) = held_claims(work_dir.as_path());
    let before: usize = passes_made();
    let renewal: ClaimRenewal = ClaimRenewal::start(renewer, Duration::from_millis(5));
    // The held set names another claim; the thread runs many intervals and must leave this one alone.
    let other: Claimed = Claimed { id: String::from("push-another-claim"), name: String::from("push-another-claim~00~x"), item: claimed.item.clone() };
    held.hold(&other);
    assert!(wait_for_passes(before, PASSES_WATCHED), "the renewal thread ran {PASSES_WATCHED} passes, so leaving the claim alone is its own doing");
    drop(renewal);
    assert_eq!(claimed_names(work_dir.as_path()), vec![claimed.name], "an unheld claim keeps its name");
    assert!(!held.is_held("push-another-claim"), "a held claim that is not in the directory is found gone");
    remove(root.as_path());
}

/// A settlement finds its claim by id and generation,
/// so a renewal that renamed the file since the claim was taken does not
/// make the settlement miss it,
/// and no lock stands between the two.
#[test]
fn a_settlement_after_renewals_finds_the_claim_under_whatever_name_it_has() {
    let (root, work_dir, claimed) = claimed_item("renewal-settle-after", &clock());
    let (held, mut renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    for third in 1..=RENEWALS_PER_LEASE {
        renewer.renew(&clock().advanced(Duration::from_secs(u64::from(third))));
    }
    assert_ne!(claimed_names(work_dir.as_path()), vec![claimed.name.clone()], "the file was renamed under the drain");
    held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect("settled under the renewed name, found by its generation");
    assert!(state_directory(work_dir.as_path(), DONE).join(claimed.id.as_str()).exists());
    assert!(claimed_names(work_dir.as_path()).is_empty());
    remove(root.as_path());
}

/// A drain and its renewal thread work at once without either waiting:
/// settlements race renewals driven as fast as the thread
/// answers,
/// and every claim is settled exactly once,
/// none lost and none left in `claimed`.
#[test]
fn settlements_racing_renewals_lose_no_claim() {
    let root: PathBuf = fixture("renewal-race");
    let work_dir: PathBuf = root.join("work");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: root.to_path_buf(),
        git_dir: root.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: OID.to_owned(),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    let (held, renewer) = held_claims(work_dir.as_path());
    let driver: DrivenRenewal = DrivenRenewal::default();
    let renewal: ClaimRenewal = ClaimRenewal::driven(renewer, &driver);
    let mut ids: Vec<String> = Vec::new();
    for number in 0..RACED_CLAIMS {
        let queued: WorkItem = WorkItem { oid: format!("{number:040x}"), ..item.clone() };
        enqueue(work_dir.as_path(), &queued).expect("enqueue");
        let claimed: Claimed = claim_next_of(work_dir.as_path(), &clock(), &[WorkKind::Push]).expect("claim").expect("one claim");
        held.hold(&claimed);
        ids.push(claimed.id);
    }
    let stop: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let renewing = scope.spawn(|| {
            let mut round: u64 = 0;
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                round += 1;
                let _ = driver.renew(&clock().advanced(Duration::from_secs(round)));
            }
        });
        let letting_go: StopRenewing<'_> = StopRenewing(&stop);
        for id in &ids {
            held.settle(work_dir.as_path(), id.as_str(), DONE).expect("settled while renewals run");
        }
        drop(letting_go);
        renewing.join().expect("the renewals finished");
    });
    drop(renewal);
    assert!(claimed_names(work_dir.as_path()).is_empty(), "no claim was left in claimed");
    for id in &ids {
        assert!(state_directory(work_dir.as_path(), DONE).join(id.as_str()).exists(), "{id} was settled");
    }
    remove(root.as_path());
}

/// How many claims the race holds at once.
const RACED_CLAIMS: usize = 40;

/// What:
///  The flag that ends the renewal thread of a control.
/// Why:
///   The flag goes down when the control lets go and also when the control ends however it ends,
/// so a settlement that fails
///  cannot leave the scope joining a thread that renews without end.
struct StopRenewing<'flag>(&'flag std::sync::atomic::AtomicBool);

impl Drop for StopRenewing<'_> {
    /// End the renewal thread.
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// The number of threads this process has,
/// read from its status.
fn threads_of_this_process() -> usize {
    let status: String = std::fs::read_to_string("/proc/self/status").expect("the status of this process");
    let line: &str = status.lines().find(|line| return line.starts_with("Threads:")).expect("a Threads line");
    return line.trim_start_matches("Threads:").trim().parse().expect("a thread count");
}

/// How many renewals the control below keeps running at once.
const RENEWALS_RUNNING: usize = 16;

/// Waits until `count` renewal tasks have begun beyond `before`,
/// which the tasks themselves say,
/// and gives whether they did within `PASS_PATIENCE`.
fn wait_for_starts(before: usize, count: usize) -> bool {
    let began: std::time::Instant = std::time::Instant::now();
    while TASKS_STARTED.load(std::sync::atomic::Ordering::SeqCst) < before.saturating_add(count) {
        if began.elapsed() >= PASS_PATIENCE {
            return false;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    return true;
}

/// Renewals run as tasks of the process's runtime:
/// many running at once add far fewer threads than renewals,
/// where a thread each added one thread each.
/// The tasks say when they have begun,
/// so the count of threads is read of renewals that really run and not of renewals the runtime has not started yet.
#[test]
fn renewals_running_at_once_start_no_thread_each() {
    // The runtime's own workers are started by the first use, and are not the renewals' threads.
    crate::lazy_runtime::run_on_runtime(async {}).expect("the runtime starts");
    let before: usize = threads_of_this_process();
    let started_before: usize = TASKS_STARTED.load(std::sync::atomic::Ordering::SeqCst);
    let mut running: Vec<ClaimRenewal> = Vec::new();
    let mut roots: Vec<PathBuf> = Vec::new();
    for number in 0..RENEWALS_RUNNING {
        let root: PathBuf = fixture(format!("renewal-no-thread-{number}").as_str());
        let (_held, renewer) = held_claims(root.join("work").as_path());
        running.push(ClaimRenewal::start(renewer, LEASE));
        roots.push(root);
    }
    assert!(wait_for_starts(started_before, RENEWALS_RUNNING), "{RENEWALS_RUNNING} renewals began");
    let growth: usize = threads_of_this_process().saturating_sub(before);
    drop(running);
    for root in &roots {
        remove(root.as_path());
    }
    assert!(growth < RENEWALS_RUNNING / 2, "{RENEWALS_RUNNING} renewals added {growth} threads: a renewal must be a task");
}

/// How long the control below keeps the blocking pool busy.
const POOL_BUSY_FOR: Duration = Duration::from_secs(3);

/// How long the control below lets dropping a renewal wait.
const SHORT_STOP_CEILING: Duration = Duration::from_millis(300);

/// Stopping a renewal whose task cannot end (every thread of the blocking pool is busy,
/// so the renames wait for one) returns at the ceiling and not when the pool frees:
/// the wait is bounded,
/// and the claim is left to expire by its lease.
/// What shows the pool had not freed is the count of its finished jobs,
/// not a clock:
/// the one time in it is the ceiling itself,
/// which is the behaviour under test.
#[test]
fn stopping_a_renewal_whose_task_cannot_end_returns_at_the_ceiling() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-stop-bounded", &started);
    let (held, renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let driver: DrivenRenewal = DrivenRenewal::default();
    let renewal: ClaimRenewal = ClaimRenewal::driven_with_stop_ceiling(renewer, &driver, SHORT_STOP_CEILING);
    // Every thread the blocking pool may have is kept busy, then the task is asked to renew: its job waits for a thread.
    let workers: usize = match crate::lazy_runtime::built_flavor() {
        Some(crate::lazy_runtime::BuiltFlavor::MultiThread { workers }) => workers,
        _ => 1,
    };
    let jobs: usize = workers * 2 + 2;
    let finished: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let busy = {
        let counting = std::sync::Arc::clone(&finished);
        crate::lazy_runtime::spawn_on_runtime(async move {
            for _ in 0..jobs {
                let each = std::sync::Arc::clone(&counting);
                drop(tokio::task::spawn_blocking(move || {
                    std::thread::sleep(POOL_BUSY_FOR);
                    each.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }));
            }
        })
    };
    assert!(busy.is_some(), "the runtime has workers");
    std::thread::sleep(Duration::from_millis(100));
    let _asked = renewal.control.send(Control::Renew(clock(), None));
    let began: std::time::Instant = std::time::Instant::now();
    crate::lazy_runtime::run_on_runtime(renewal.stop_awaiting()).expect("the runtime runs the stop");
    let waited: Duration = began.elapsed();
    let done: usize = finished.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(done, 0, "the stop waited for the pool to free instead of for the ceiling: {done} of {jobs} jobs had finished after {waited:?}");
    assert!(waited >= SHORT_STOP_CEILING, "the stop did not wait for the task at all: {waited:?}");
    drop(held);
    remove(root.as_path());
}

/// Dropping a renewal never waits for its task,
/// so a caller that is itself a task of the runtime holds no worker while the task it stopped reads its stop:
/// the drop returns in a moment,
/// and the task is left to end on its own,
/// with the claims expiring by their lease if it cannot.
/// Why the pool is kept busy:
/// it is what a waiting drop would wait for,
/// so the control can tell a drop that waits from one that only tells the task to stop.
#[test]
fn dropping_a_renewal_leaves_its_task_to_end_without_waiting_for_it() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-drop-does-not-wait", &started);
    let (held, renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let driver: DrivenRenewal = DrivenRenewal::default();
    let renewal: ClaimRenewal = ClaimRenewal::driven_with_stop_ceiling(renewer, &driver, SHORT_STOP_CEILING);
    // Every thread the blocking pool may have is kept busy, then the task is asked to renew: its job waits for a thread.
    let workers: usize = match crate::lazy_runtime::built_flavor() {
        Some(crate::lazy_runtime::BuiltFlavor::MultiThread { workers }) => workers,
        _ => 1,
    };
    let jobs: usize = workers * 2 + 2;
    let busy = {
        crate::lazy_runtime::spawn_on_runtime(async move {
            for _ in 0..jobs {
                drop(tokio::task::spawn_blocking(move || {
                    std::thread::sleep(POOL_BUSY_FOR);
                }));
            }
        })
    };
    assert!(busy.is_some(), "the runtime has workers");
    std::thread::sleep(Duration::from_millis(100));
    let _asked = renewal.control.send(Control::Renew(clock(), None));
    let ended_before: usize = TASKS_ENDED.load(std::sync::atomic::Ordering::SeqCst);
    let began: std::time::Instant = std::time::Instant::now();
    // The drop runs inside a task, which is where a drain that is a task would drop one.
    crate::lazy_runtime::run_on_runtime(async move {
        drop(renewal);
    })
    .expect("the runtime runs the drop");
    let dropped: Duration = began.elapsed();
    assert!(dropped < SHORT_STOP_CEILING, "the drop held the thread it ran on until the ceiling: {dropped:?}");
    assert_eq!(
        TASKS_ENDED.load(std::sync::atomic::Ordering::SeqCst),
        ended_before,
        "the drop waited for the task to end, which is what the two stop faces are for"
    );
    drop(held);
    remove(root.as_path());
}

/// Measurement,
/// not a control:
/// what one renewal of one claim costs here.
/// Why it is no control:
///  its verdict would be the host's speed;
/// the ceiling it compares against is derived from a measurement of a
///  check and a rename (`worst_delay_seen_on_this_host.rs`),
/// and a loaded host fails any wall bound short of that
///  measurement.
/// Run it by name with `--run-ignored all --no-capture`.
#[test]
#[ignore = "a measurement of the disk, not a control: its bar would be the host's speed"]
fn a_renewal_takes_far_less_than_its_stop_ceiling() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-time", &started);
    let began: std::time::Instant = std::time::Instant::now();
    let renewed = renew_claim(work_dir.as_path(), claimed.name.as_str(), &started.advanced(Duration::from_secs(5)));
    let took: Duration = began.elapsed();
    assert!(renewed.is_some());
    assert!(took * 1000 < RENEWAL_STOP_CEILING, "one renewal took {took:?}, a thousandth of the ceiling is the control's bar");
    remove(root.as_path());
}

/// What:
///  The words a settlement wrote when its move found nothing,
///  checked against the one fact the control arranged and against the guess the words must never carry.
/// Why:
///  Every order that leaves a record somewhere else has its own sentence,
/// and a sentence that says "probably" says nothing a reader can act on:
/// the control holds the order,
/// so the words have to name it.
fn assert_the_words_say(said: &str, expected: &[&str]) {
    for fact in expected {
        assert!(said.contains(fact), "the words do not say {fact:?}: {said}");
    }
    assert!(!said.contains("probably"), "the words guess instead of saying what the record's own places show: {said}");
}

/// A settlement that finds another instance's claim of the same item in `claimed` says which order happened:
/// the lease ran out and the item was claimed again,
/// so its work is run again by the instance that holds it now.
///  The other instance's claim is the only one,
/// and nothing of this holder's claim was moved,
/// so the record has one owner at a time.
#[test]
fn a_move_that_finds_another_instance_claim_says_who_holds_the_record_now() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("move-finds-a-successor", &started);
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    // Another instance finds the lease over and claims the item again; this holder's renewal has not run, which is the holder
    //  whose renewal is the one that is late.
    let later: ClaimTime = started.advanced(LEASE * 2);
    assert_eq!(requeue_expired(work_dir.as_path(), &later, LEASE, MAX_TRIES), 1, "the lease ran out");
    let successor: Claimed = claim_next_of(work_dir.as_path(), &later, &[WorkKind::Push]).expect("claim").expect("the item is queued again");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a claim taken over settles nothing");
    assert_the_words_say(
        said.as_str(),
        &["claimed it again", successor.name.as_str(), "run again there", "nothing of it was moved here"],
    );
    assert_eq!(claimed_names(work_dir.as_path()), vec![successor.name.clone()], "the other instance's claim is the only one");
    assert!(!state_directory(work_dir.as_path(), DONE).join(claimed.id.as_str()).exists(), "nothing was settled for either claim");
    remove(root.as_path());
}

/// A settlement that finds the item queued again says so:
/// the claim was taken back when its lease ran out and the item waits in `pending`,
/// so the next look that claims it runs its work again.
#[test]
fn a_move_that_finds_the_record_queued_again_says_its_work_runs_again() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("move-finds-the-queue", &started);
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let later: ClaimTime = started.advanced(LEASE * 2);
    assert_eq!(requeue_expired(work_dir.as_path(), &later, LEASE, MAX_TRIES), 1, "the lease ran out and the item went back to the queue");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a claim taken back settles nothing");
    assert_the_words_say(said.as_str(), &["left it queued as pending", "run again", "nothing of it was moved here"]);
    assert!(state_directory(work_dir.as_path(), PENDING).join(claimed.id.as_str()).exists(), "the record waits in pending");
    remove(root.as_path());
}

/// A settlement that finds the record given up on says so,
/// and that nothing looks at it again:
/// the recovery of the claim reached the last try and moved it to `failed`,
/// where no round reads it.
#[test]
fn a_move_that_finds_the_record_given_up_on_says_nothing_runs_its_work() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("move-finds-given-up", &started);
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let later: ClaimTime = started.advanced(LEASE * 2);
    // One try is the whole allowance this control gives, so the recovery that takes the claim back gives up on it.
    assert_eq!(requeue_expired(work_dir.as_path(), &later, LEASE, 1), 1, "the claim was taken back and given up on");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a claim given up on settles nothing");
    assert_the_words_say(said.as_str(), &["given up on", "no later look runs its work"]);
    assert!(state_directory(work_dir.as_path(), FAILED).join(claimed.id.as_str()).exists(), "the record is in failed");
    remove(root.as_path());
}

/// A settlement that finds a take in flight says so:
/// another instance is moving the record's file out of `claimed` into a move of its own,
/// which is the state its take or its recovery leaves when it ends at any instruction between the two renames.
#[test]
fn a_move_that_finds_a_take_in_flight_says_which_moves_can_end_there() {
    let (root, work_dir, claimed) = claimed_item("move-finds-a-take", &clock());
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let recovering: PathBuf = state_directory(work_dir.as_path(), RECOVERING);
    // The exact file a take moves: a recovery's first rename leaves the claim's file in `recovering` under its own name.
    std::fs::rename(state_directory(work_dir.as_path(), CLAIMED).join(claimed.name.as_str()), recovering.join(claimed.name.as_str()))
        .expect("the claim is in recovering");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a claim a recovery holds settles nothing");
    assert_the_words_say(said.as_str(), &[RECOVERING, CLAIMED, PENDING, FAILED, "tries", "which of the three happened"]);
    assert!(!said.contains("left it queued as"), "a record a recovery is moving is not reported as one the queue already holds: {said}");
    remove(root.as_path());
}

/// A settlement that finds no work directory says the record went with it,
/// which is what a repository removed under the service leaves:
/// no state of the record can be read,
/// so nothing says the work was done and nothing will read the record again.
#[test]
fn a_move_that_finds_no_work_directory_says_the_record_went_with_it() {
    let (root, work_dir, claimed) = claimed_item("move-finds-no-work", &clock());
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    std::fs::remove_dir_all(work_dir.as_path()).expect("the work directory is gone");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a record whose directory is gone settles nothing");
    assert_the_words_say(said.as_str(), &["work directory", "is not there", "went with it", "recorded nowhere"]);
    remove(root.as_path());
}

/// A settlement that finds no `claimed` directory says so,
/// and that the record cannot be read rather than that another instance took it.
#[test]
fn a_move_that_finds_no_claimed_directory_says_which_directory_is_missing() {
    let (root, work_dir, claimed) = claimed_item("move-finds-no-claimed", &clock());
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    std::fs::remove_dir_all(state_directory(work_dir.as_path(), CLAIMED)).expect("the claimed directory is gone");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a record whose claim directory is gone settles nothing");
    assert_the_words_say(said.as_str(), &["claimed directory", "is not there", "recorded nowhere"]);
    remove(root.as_path());
}

/// A settlement whose move fails while the claim is still `claimed` says nothing moved,
/// and when its work is run again:
/// the record's own file is where it was,
/// only the place it was to move into was missing.
#[test]
fn a_move_that_finds_no_done_directory_says_the_record_stays_claimed() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("move-finds-no-done", &started);
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    // A work directory whose `done` directory is gone is a move that cannot land, with every other part of the record intact.
    std::fs::remove_dir(state_directory(work_dir.as_path(), DONE)).expect("the empty done directory");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a move into a directory that is not there fails");
    assert_the_words_say(said.as_str(), &["still in claimed/", "nothing of the record moved here", "when its lease runs out"]);
    let claimed_after: Vec<String> = claimed_names(work_dir.as_path());
    assert_eq!(claimed_after.len(), 1, "the claim the settlement could not move is kept: {claimed_after:?}");
    remove(root.as_path());
}

/// A settlement that finds no file of the record anywhere says that every place was looked at and the record is in none of them,
/// which is the one way a record leaves every state directory:
/// a removal nothing of the claim protocol made.
#[test]
fn a_move_that_finds_no_file_of_the_record_says_something_else_removed_it() {
    let (root, work_dir, claimed) = claimed_item("move-finds-no-file", &clock());
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    std::fs::remove_file(state_directory(work_dir.as_path(), CLAIMED).join(claimed.name.as_str())).expect("the claim's file is removed");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a claim whose file is gone settles nothing");
    assert_the_words_say(said.as_str(), &["no file of", "something other than a move of this queue", "every one of those places was looked at"]);
    remove(root.as_path());
}

/// What:
///  A state directory made unsearchable for reading while it stays reachable by name,
///  and put back when this value ends.
/// Why:
///  A directory whose read permission is gone answers a listing with "Permission denied"
/// and still lets a name inside it be looked at,
/// which is the accident a settlement must not read as a record somebody removed.
/// The mode is put back however the control ends, so a failing control leaves no fixture behind that the next one trips over.
struct Unreadable<'directory>(&'directory Path);

impl Unreadable<'_> {
    /// Take away the read permission of `directory`, keeping the search permission.
    fn of(directory: &Path) -> Unreadable<'_> {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o111)).expect("the directory can be searched but not listed");
        return Unreadable(directory);
    }
}

impl Drop for Unreadable<'_> {
    /// Give the read permission back.
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(self.0, std::fs::Permissions::from_mode(0o755)).expect("the directory can be listed again");
    }
}

/// A settlement that finds another instance's recovery of the item in `recovering` says both endings that recovery has:
/// the item waits queued again, or it is given up on,
/// and a record whose tries are all used is the second.
/// The first control of the words named only the queueing, so this one holds the last try of an item,
/// where the recovery that took it goes on to `failed` and nothing runs the work again.
#[test]
fn a_move_that_finds_a_take_in_flight_says_both_endings_of_the_recovery() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item_after_tries("move-recovering-last-try", &started, MAX_TRIES - 1);
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    // The take a recovery makes: the expired claim is renamed into `recovering`, where no reader of `claimed` looks.
    std::fs::rename(
        state_directory(work_dir.as_path(), CLAIMED).join(claimed.name.as_str()),
        state_directory(work_dir.as_path(), RECOVERING).join(claimed.name.as_str()),
    )
    .expect("the claim is in recovering");
    let said: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a claim a recovery holds settles nothing");
    // The recovery finishes with the second writer: this record has used its last try, so it is given up on.
    assert_eq!(requeue_expired(work_dir.as_path(), &started.advanced(LEASE * 2), LEASE, MAX_TRIES), 1, "the recovery finishes");
    let failed: bool = state_directory(work_dir.as_path(), FAILED).join(claimed.id.as_str()).exists();
    let queued_again: bool = state_directory(work_dir.as_path(), PENDING).join(claimed.id.as_str()).exists();
    remove(root.as_path());
    assert!(failed, "the last try of the item goes to failed");
    assert!(!queued_again, "the last try of the item is not queued again");
    assert!(said.contains(PENDING) && said.contains(FAILED) && said.contains(CLAIMED), "the words name all three endings: {said}");
    assert!(said.contains("tries"), "the words say what decides which ending it is: {said}");
    assert!(said.contains("which of the three happened"), "the words say how to tell which happened: {said}");
    assert!(
        !said.contains("run again") && !said.contains("queue it again"),
        "the record went to failed with nothing running it, but the settlement promised a run: {said}"
    );
    assert!(!said.contains("probably"), "the words say what the record's places show instead of guessing: {said}");
}

/// A settlement that cannot list `claimed` says that the directory could not be read and what the error was,
/// and never that a record was removed:
/// the claim's file is still there,
/// only the look at it failed.
#[test]
fn a_move_that_finds_an_unreadable_claimed_directory_says_the_look_failed() {
    let (root, work_dir, claimed) = claimed_item("move-unreadable-claimed", &clock());
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    assert!(held.is_held_on_disk(work_dir.as_path(), claimed.id.as_str()), "the claim is listed while the directory can be read");
    let claimed_directory: PathBuf = state_directory(work_dir.as_path(), CLAIMED);
    let said: String;
    let still_there: bool;
    {
        let _unreadable: Unreadable<'_> = Unreadable::of(claimed_directory.as_path());
        said = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("an unlistable claim directory settles nothing");
        // The name is still reachable inside a directory that may be searched but not listed.
        still_there = claimed_directory.join(claimed.name.as_str()).exists();
    }
    remove(root.as_path());
    assert!(still_there, "the failed moves and listings removed the claim");
    assert!(said.contains("Permission denied"), "the words keep the error the listing gave: {said}");
    assert!(said.contains(CLAIMED) && said.contains("could not be read"), "the words name the directory that could not be read: {said}");
    assert!(said.contains("could not be found out"), "the words say what could not be found out: {said}");
    assert!(
        !said.contains("something other than the queue moved or removed it") && !said.contains("no file of"),
        "the claim was still there, but the words said a record was removed: {said}"
    );
}

/// What:
///  The sentence a settlement writes when another instance's take has staged a claim of the record in `recovering`,
///  and the one it writes when the take publishes that claim in `claimed`.
/// Why:
///  A take stages a fresh claim in `recovering` for the time of one stamp and one rename,
/// and that claim ends in `claimed`, where the instance that made it runs the work:
/// a settlement that names only the two endings a recovery has names neither the real one nor what a person should expect.
fn a_staged_take_before_and_after_it_publishes(name: &str) -> (String, String) {
    let (root, work_dir, claimed) = claimed_item(name, &clock());
    let (held, _renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let staged: String = format!("{}~0123456789abcdef0123456789abcdef~{}", claimed.id, ClaimTime::read().encode());
    let claimed_directory: PathBuf = state_directory(work_dir.as_path(), CLAIMED);
    std::fs::rename(claimed_directory.join(claimed.name.as_str()), state_directory(work_dir.as_path(), RECOVERING).join(staged.as_str()))
        .expect("the take stages its claim");
    let before: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("the claim is not in claimed");
    std::fs::rename(state_directory(work_dir.as_path(), RECOVERING).join(staged.as_str()), claimed_directory.join(staged.as_str()))
        .expect("the take publishes its claim");
    // The same instance looks again, with a held claim of its own, after the take has published.
    let (after_settling, _its_renewer): (HeldClaims, Renewer) = held_claims(work_dir.as_path());
    after_settling.hold(&claimed);
    let after: String = after_settling.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("the claim is not this instance's");
    remove(root.as_path());
    return (before, after);
}

/// A claim another instance's take has staged in `recovering` is told as a claim that is published in `claimed`,
/// where that instance runs the work,
/// and not as a recovery with two endings only.
#[test]
fn a_move_that_finds_a_staged_take_says_the_claim_is_published_there() {
    let (before, _after) = a_staged_take_before_and_after_it_publishes("move-staged-take");
    assert!(before.contains(CLAIMED), "the words name `claimed` as where the record goes: {before}");
    assert!(before.contains("runs the work there"), "the words say the work runs in that instance's claim: {before}");
    assert!(!before.contains("two endings"), "the words say a move has two endings only, but a take's ends in claimed: {before}");
    assert!(!before.contains("probably"), "the words guess: {before}");
}

/// A claim another instance's take has published in `claimed` is said to be that instance's claim:
/// its ending has happened, and the words say so rather than naming a move still under way.
#[test]
fn a_move_that_finds_the_takes_published_claim_says_it_is_claimed_again() {
    let (_before, after) = a_staged_take_before_and_after_it_publishes("move-take-published");
    assert!(after.contains("claimed it again"), "the words say the record is claimed again: {after}");
    assert!(!after.contains("taking the claim"), "the take has finished, but the words say it is under way: {after}");
}

/// A `claimed` directory that is not there, or that a plain file has taken the place of, is an absence the filesystem
/// answered:
/// the look says the record went with it,
/// sends nobody to permissions,
/// and the claim is released because nothing is left to keep.
#[test]
fn a_look_at_a_removed_or_replaced_claimed_directory_says_the_record_went_with_it() {
    for replaced in [false, true] {
        let (root, work_dir, claimed) = claimed_item(if replaced { "look-claimed-replaced" } else { "look-work-removed" }, &clock());
        let (held, _renewer) = held_claims(work_dir.as_path());
        held.hold(&claimed);
        if replaced {
            std::fs::remove_dir_all(state_directory(work_dir.as_path(), CLAIMED)).expect("the claimed directory is removed");
            std::fs::write(state_directory(work_dir.as_path(), CLAIMED), b"a file where the directory was").expect("a file at its name");
        } else {
            std::fs::remove_dir_all(work_dir.as_path()).expect("the work directory is removed under the service");
        }
        let refused: String = held.is_held_on_disk_or_the_error(work_dir.as_path(), claimed.id.as_str()).expect_err("there is no claim to find");
        let entries: Vec<crate::service_timing::capture::Captured> = crate::service_timing::capture::entries_of(|| {
            let mine: Vec<Claimed> = crate::service_work_handlers_replanning_and_acceptance::still_held(&held, work_dir.as_path(), vec![claimed.clone()]);
            assert!(mine.is_empty(), "no work runs on a record that went away (replaced={replaced})");
        });
        let kept: bool = held.is_held(claimed.id.as_str());
        remove(root.as_path());
        assert!(refused.contains("went with it"), "the words say the record went with the directory (replaced={replaced}): {refused}");
        assert!(!refused.contains("permissions"), "the words send a person to permissions for a directory that is not there (replaced={replaced}): {refused}");
        assert!(!kept, "a claim with nothing left to keep was kept (replaced={replaced})");
        assert!(
            entries.iter().any(|entry| { return entry.message.contains("went with it"); }),
            "the drain says the record went with the directory (replaced={replaced}): {entries:?}"
        );
    }
}

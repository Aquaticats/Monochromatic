//! Controls for the renewal of claim leases:
//! what a renewal changes,
//! what it leaves alone,
//! what a settlement reaches,
//! and that
//! the thread stops.

use crate::work_item_recovery::requeue_expired;
use super::*;
use crate::service_work::MAX_TRIES;
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, DONE, WorkItem, WorkKind, claim_next_of, enqueue, find_claim, state_directory};
use std::path::Path;

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
    let root: PathBuf = fixture(name);
    let work_dir: PathBuf = root.join("work");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: root.display().to_string(),
        git_dir: root.display().to_string(),
        branch_ref: String::from("refs/heads/main"),
        oid: OID.to_owned(),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
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
/// and is gone when the renewal is dropped.
#[test]
fn a_running_renewal_renews_the_held_claims_until_it_is_dropped() {
    let (root, work_dir, claimed) = claimed_item("renewal-thread", &clock());
    let (held, renewer) = held_claims(work_dir.as_path());
    held.hold(&claimed);
    let renewal: ClaimRenewal = ClaimRenewal::start(renewer, Duration::from_millis(5));
    // The bound only ends a failing run: a passing run leaves the loop as soon as the thread has renewed the claim.
    let bound: std::time::Instant = std::time::Instant::now() + Duration::from_secs(20);
    while claimed_names(work_dir.as_path()) == vec![claimed.name.clone()] && std::time::Instant::now() < bound {
        std::thread::sleep(Duration::from_millis(5));
    }
    let renewed: Vec<String> = claimed_names(work_dir.as_path());
    assert_ne!(renewed, vec![claimed.name.clone()], "the thread renewed the held claim");
    drop(renewal);
    // Dropping joins the thread, so nothing can rename after this line.
    let after_drop: Vec<String> = claimed_names(work_dir.as_path());
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(claimed_names(work_dir.as_path()), after_drop, "a dropped renewal renames nothing");
    remove(root.as_path());
}

/// A claim the renewal does not hold is left to its lease,
/// so a dead holder's claim still expires.
#[test]
fn a_claim_that_is_not_held_is_not_renewed() {
    let (root, work_dir, claimed) = claimed_item("renewal-not-held", &clock());
    let (held, renewer) = held_claims(work_dir.as_path());
    let renewal: ClaimRenewal = ClaimRenewal::start(renewer, Duration::from_millis(5));
    // The held set names another claim; the thread runs for many intervals and must leave this one alone.
    let other: Claimed = Claimed { id: String::from("push-another-claim"), name: String::from("push-another-claim~00~x"), item: claimed.item.clone() };
    held.hold(&other);
    std::thread::sleep(Duration::from_millis(50));
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
        repository: root.display().to_string(),
        git_dir: root.display().to_string(),
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
        for id in &ids {
            held.settle(work_dir.as_path(), id.as_str(), DONE).expect("settled while renewals run");
        }
        stop.store(true, std::sync::atomic::Ordering::SeqCst);
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

/// The number of threads this process has,
/// read from its status.
fn threads_of_this_process() -> usize {
    let status: String = std::fs::read_to_string("/proc/self/status").expect("the status of this process");
    let line: &str = status.lines().find(|line| return line.starts_with("Threads:")).expect("a Threads line");
    return line.trim_start_matches("Threads:").trim().parse().expect("a thread count");
}

/// How many renewals the control below keeps running at once.
const RENEWALS_RUNNING: usize = 16;

/// Renewals run as tasks of the process's runtime:
/// many running at once add far fewer threads than renewals,
/// where a thread each added one thread each.
#[test]
fn renewals_running_at_once_start_no_thread_each() {
    // The runtime's own workers are started by the first use, and are not the renewals' threads.
    crate::lazy_runtime::run_on_runtime(async {}).expect("the runtime starts");
    let before: usize = threads_of_this_process();
    let mut running: Vec<ClaimRenewal> = Vec::new();
    let mut roots: Vec<PathBuf> = Vec::new();
    for number in 0..RENEWALS_RUNNING {
        let root: PathBuf = fixture(format!("renewal-no-thread-{number}").as_str());
        let (_held, renewer) = held_claims(root.join("work").as_path());
        running.push(ClaimRenewal::start(renewer, LEASE));
        roots.push(root);
    }
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

/// Dropping a renewal whose task cannot end (every thread of the blocking pool is busy,
/// so the renames wait for one) returns at the ceiling and not when the pool frees:
/// the wait is bounded,
/// and the claim is left to expire by its lease.
#[test]
fn dropping_a_renewal_whose_task_cannot_end_returns_at_the_ceiling() {
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
    let busy = crate::lazy_runtime::spawn_on_runtime(async move {
        for _ in 0..workers * 2 + 2 {
            drop(tokio::task::spawn_blocking(|| { std::thread::sleep(POOL_BUSY_FOR); }));
        }
    });
    assert!(busy.is_some(), "the runtime has workers");
    std::thread::sleep(Duration::from_millis(100));
    let _asked = renewal.control.send(Control::Renew(clock(), None));
    let began: std::time::Instant = std::time::Instant::now();
    drop(renewal);
    let waited: Duration = began.elapsed();
    assert!(waited < POOL_BUSY_FOR / 2, "dropping waited {waited:?}, which is until the pool freed and not the ceiling");
    assert!(waited >= SHORT_STOP_CEILING, "dropping did not wait for the task at all: {waited:?}");
    drop(held);
    remove(root.as_path());
}

/// A renewal of a claim takes far less than its ceiling:
/// the rename is a few system calls.
#[test]
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

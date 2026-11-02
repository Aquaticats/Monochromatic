//! Controls for the renewal of claim leases: what a renewal stamps, what it leaves alone, and that it stops.

use super::*;
use crate::test_support::{fixture, remove};
use crate::service_work::MAX_TRIES;
use crate::work_item::{CLAIMED, WorkItem, WorkKind, claim_next_of, enqueue, requeue_expired, state_directory};
use std::path::Path;

const OID: &str = "0123456789abcdef0123456789abcdef01234567";
const LEASE: Duration = Duration::from_secs(900);

/// A claimed push item in a fresh work directory, stamped at `stamp`; the claim's id is returned with the directory.
fn claimed_item(name: &str, stamp: SystemTime) -> (PathBuf, PathBuf, String) {
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
    let claimed = claim_next_of(work_dir.as_path(), stamp, &[WorkKind::Push]).expect("claim").expect("one claim");
    return (root, work_dir, claimed.id);
}

/// The stamp of a claim.
fn stamp_of(work_dir: &Path, id: &str) -> SystemTime {
    return std::fs::metadata(state_directory(work_dir, CLAIMED).join(id)).and_then(|meta| return meta.modified()).expect("stamp");
}

/// A renewed claim is not taken over at the end of the lease it started with, and the same claim unrenewed is.
#[test]
fn a_renewed_claim_outlives_the_lease_it_started_with() {
    let started: SystemTime = SystemTime::now();
    let (root, work_dir, id) = claimed_item("renewal-outlives", started);
    let almost_over: SystemTime = started + LEASE - Duration::from_secs(1);
    assert!(renew_claim(work_dir.as_path(), id.as_str(), almost_over), "the claim is still held");
    let past_first_lease: SystemTime = started + LEASE + Duration::from_secs(1);
    assert_eq!(requeue_expired(work_dir.as_path(), past_first_lease, LEASE, MAX_TRIES), 0, "renewed, so still held");
    let past_renewed_lease: SystemTime = almost_over + LEASE + Duration::from_secs(1);
    assert_eq!(requeue_expired(work_dir.as_path(), past_renewed_lease, LEASE, MAX_TRIES), 1, "a holder that stopped renewing loses it");
    remove(root.as_path());
}

/// A renewal stamps the time it is given, and finds nothing to stamp once the claim has left `claimed`.
#[test]
fn renewing_a_claim_stamps_the_given_time_and_never_brings_back_a_settled_one() {
    let started: SystemTime = SystemTime::now() - Duration::from_secs(3600);
    let (root, work_dir, id) = claimed_item("renewal-stamp", started);
    let later: SystemTime = started + Duration::from_secs(1234);
    assert!(renew_claim(work_dir.as_path(), id.as_str(), later));
    assert_eq!(stamp_of(work_dir.as_path(), id.as_str()), later);
    std::fs::remove_file(state_directory(work_dir.as_path(), CLAIMED).join(id.as_str())).expect("settled");
    assert!(!renew_claim(work_dir.as_path(), id.as_str(), later), "nothing is left to renew");
    assert!(!state_directory(work_dir.as_path(), CLAIMED).join(id.as_str()).exists(), "a renewal does not create the claim");
    remove(root.as_path());
}

/// The held set follows the claims taken and settled.
#[test]
fn the_held_set_follows_what_is_taken_and_settled() {
    let held: HeldClaims = HeldClaims::default();
    held.hold("push-a");
    held.hold("push-b");
    assert_eq!(held.snapshot(), vec!["push-a", "push-b"]);
    held.release("push-a");
    assert_eq!(held.snapshot(), vec!["push-b"]);
}

/// The renewal thread stamps the held claims by itself, and is gone when the renewal is dropped.
#[test]
fn a_running_renewal_stamps_the_held_claims_until_it_is_dropped() {
    let started: SystemTime = SystemTime::now() - Duration::from_secs(3600);
    let (root, work_dir, id) = claimed_item("renewal-thread", started);
    let held: HeldClaims = HeldClaims::default();
    held.hold(id.as_str());
    let renewal: ClaimRenewal = ClaimRenewal::start(work_dir.clone(), held, Duration::from_millis(5));
    // The bound only ends a failing run: a passing run leaves the loop as soon as the thread has stamped the claim.
    let bound: std::time::Instant = std::time::Instant::now() + Duration::from_secs(20);
    while stamp_of(work_dir.as_path(), id.as_str()) <= started && std::time::Instant::now() < bound {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(stamp_of(work_dir.as_path(), id.as_str()) > started, "the thread renewed the held claim");
    drop(renewal);
    // Dropping joins the thread, so nothing can stamp after this line.
    let old: SystemTime = started + Duration::from_secs(1);
    assert!(renew_claim(work_dir.as_path(), id.as_str(), old));
    assert_eq!(stamp_of(work_dir.as_path(), id.as_str()), old, "a dropped renewal stamps nothing");
    remove(root.as_path());
}

/// A claim the renewal does not hold is left to its lease, so a dead holder's claim still expires.
#[test]
fn a_claim_that_is_not_held_is_not_renewed() {
    let started: SystemTime = SystemTime::now() - Duration::from_secs(3600);
    let (root, work_dir, id) = claimed_item("renewal-not-held", started);
    let held: HeldClaims = HeldClaims::default();
    let renewal: ClaimRenewal = ClaimRenewal::start(work_dir.clone(), held.clone(), Duration::from_millis(5));
    // The held set names another claim; the thread runs for many intervals and must leave this one alone.
    held.hold("push-another-claim");
    std::thread::sleep(Duration::from_millis(50));
    drop(renewal);
    assert_eq!(stamp_of(work_dir.as_path(), id.as_str()), started, "an unheld claim keeps its stamp");
    remove(root.as_path());
}

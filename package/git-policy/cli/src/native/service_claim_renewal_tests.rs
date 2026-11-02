//! Controls for the renewal of claim leases: what a renewal changes, what it leaves alone, what a settlement reaches, and that
//! the thread stops.

use super::*;
use crate::service_work::MAX_TRIES;
use crate::test_support::{fixture, remove};
use crate::work_item::{CLAIMED, DONE, WorkItem, WorkKind, claim_next_of, enqueue, find_claim, requeue_expired, state_directory};
use std::path::Path;

const OID: &str = "0123456789abcdef0123456789abcdef01234567";
const LEASE: Duration = Duration::from_secs(900);
const BOOT: &str = "0123456789abcdef0123456789abcdef";

/// The clocks at the start of every test.
fn clock() -> ClaimTime {
    return ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000));
}

/// A claimed push item in a fresh work directory, claimed at `stamp`; the claim is returned with the directories.
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

/// A renewed claim is not taken over at the end of the lease it started with, and the same claim unrenewed is.
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

/// A renewal finds nothing to renew once the claim has left `claimed`, and never creates a claim.
#[test]
fn renewing_never_brings_back_a_settled_claim() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-settled", &started);
    std::fs::remove_file(state_directory(work_dir.as_path(), CLAIMED).join(claimed.name.as_str())).expect("settled");
    assert_eq!(renew_claim(work_dir.as_path(), claimed.name.as_str(), &started.advanced(Duration::from_secs(5))), None, "nothing is left to renew");
    assert!(claimed_names(work_dir.as_path()).is_empty(), "a renewal does not create the claim");
    remove(root.as_path());
}

/// The held set follows the claims taken and settled, and a settlement moves the claim under the name it has now.
#[test]
fn the_held_set_follows_what_is_taken_and_settled() {
    let (root, work_dir, claimed) = claimed_item("renewal-held", &clock());
    let held: HeldClaims = HeldClaims::default();
    held.hold(&claimed);
    assert_eq!(held.snapshot(), vec![claimed.id.clone()]);
    assert!(held.is_held(claimed.id.as_str()));
    held.renew(work_dir.as_path(), &clock().advanced(Duration::from_secs(30)));
    let renamed: String = held.name_of(claimed.id.as_str()).expect("held");
    assert_ne!(renamed, claimed.name, "the held name follows the renewal");
    assert_eq!(claimed_names(work_dir.as_path()), vec![renamed], "and so does the file");
    held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect("settled under the renewed name");
    assert!(held.snapshot().is_empty());
    assert!(find_claim(work_dir.as_path(), claimed.id.as_str()).is_none());
    assert!(state_directory(work_dir.as_path(), DONE).join(claimed.id.as_str()).exists());
    remove(root.as_path());
}

/// A claim another instance took over is found gone by the next renewal, is no longer held, and its settlement says so and
/// moves nothing: not the other instance's new claim of the same item either.
#[test]
fn a_claim_taken_over_is_found_gone_and_settles_nothing_of_the_new_claim() {
    let started: ClaimTime = clock();
    let (root, work_dir, claimed) = claimed_item("renewal-taken-over", &started);
    let held: HeldClaims = HeldClaims::default();
    held.hold(&claimed);
    // Another instance finds the lease over, returns the item to pending and claims it.
    let later: ClaimTime = started.advanced(LEASE * 2);
    assert_eq!(requeue_expired(work_dir.as_path(), &later, LEASE, MAX_TRIES), 1);
    let successor: Claimed = claim_next_of(work_dir.as_path(), &later, &[WorkKind::Push]).expect("claim").expect("requeued item");
    held.renew(work_dir.as_path(), &later);
    assert!(!held.is_held(claimed.id.as_str()), "the renewal found the claim gone");
    assert_eq!(claimed_names(work_dir.as_path()), vec![successor.name.clone()], "the renewal did not touch the successor's claim");
    let refused: String = held.settle(work_dir.as_path(), claimed.id.as_str(), DONE).expect_err("a taken-over claim settles nothing");
    assert!(refused.contains("taken over"), "{refused}");
    assert_eq!(claimed_names(work_dir.as_path()), vec![successor.name], "the successor's claim stays where it is");
    assert!(!state_directory(work_dir.as_path(), DONE).join(claimed.id.as_str()).exists());
    remove(root.as_path());
}

/// The renewal thread renames the held claims by itself, and is gone when the renewal is dropped.
#[test]
fn a_running_renewal_renews_the_held_claims_until_it_is_dropped() {
    let (root, work_dir, claimed) = claimed_item("renewal-thread", &clock());
    let held: HeldClaims = HeldClaims::default();
    held.hold(&claimed);
    let renewal: ClaimRenewal = ClaimRenewal::start(work_dir.clone(), held.clone(), Duration::from_millis(5));
    // The bound only ends a failing run: a passing run leaves the loop as soon as the thread has renewed the claim.
    let bound: std::time::Instant = std::time::Instant::now() + Duration::from_secs(20);
    while held.name_of(claimed.id.as_str()).as_deref() == Some(claimed.name.as_str()) && std::time::Instant::now() < bound {
        std::thread::sleep(Duration::from_millis(5));
    }
    let renewed: String = held.name_of(claimed.id.as_str()).expect("held");
    assert_ne!(renewed, claimed.name, "the thread renewed the held claim");
    drop(renewal);
    // Dropping joins the thread, so nothing can rename after this line.
    let after_drop: String = held.name_of(claimed.id.as_str()).expect("held");
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(held.name_of(claimed.id.as_str()), Some(after_drop.clone()), "a dropped renewal renames nothing");
    assert_eq!(claimed_names(work_dir.as_path()), vec![after_drop]);
    remove(root.as_path());
}

/// A claim the renewal does not hold is left to its lease, so a dead holder's claim still expires.
#[test]
fn a_claim_that_is_not_held_is_not_renewed() {
    let (root, work_dir, claimed) = claimed_item("renewal-not-held", &clock());
    let held: HeldClaims = HeldClaims::default();
    let renewal: ClaimRenewal = ClaimRenewal::start(work_dir.clone(), held.clone(), Duration::from_millis(5));
    // The held set names another claim; the thread runs for many intervals and must leave this one alone.
    let other: Claimed = Claimed { id: String::from("push-another-claim"), name: String::from("push-another-claim~00~x"), item: claimed.item.clone() };
    held.hold(&other);
    std::thread::sleep(Duration::from_millis(50));
    drop(renewal);
    assert_eq!(claimed_names(work_dir.as_path()), vec![claimed.name], "an unheld claim keeps its name");
    assert!(!held.is_held("push-another-claim"), "a held claim that is not in the directory is found gone");
    remove(root.as_path());
}

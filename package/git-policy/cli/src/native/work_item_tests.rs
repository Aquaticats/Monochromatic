//! Controls for the work-item record: idempotent enqueue, exclusive claims, lease expiry, and refusal of bad records.

use super::*;
use crate::test_support::{fixture, remove};

/// The lease the tests use.
const LEASE: Duration = Duration::from_secs(60);

/// The attempts after which an item is given up on.
const MAX_TRIES: u32 = 3;

/// An item for `oid` of `kind`.
fn item(kind: WorkKind, oid: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: String::from("/repo with space"),
        git_dir: String::from("/repo with space/.git"),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from(oid),
        parent: String::new(),
        created: String::from("2026-11-01T00:00:00.000Z"),
        tries: 0,
    };
}

/// The names in a state directory.
fn names(work: &Path, state: &str) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(state_directory(work, state)).expect("listing").flatten().map(|entry| return entry.file_name().to_string_lossy().into_owned()).collect();
    found.sort();
    return found;
}

/// A record decodes to the item it encoded, one line, with an id derived from kind and object.
#[test]
fn a_record_round_trips() {
    for kind in [WorkKind::Review, WorkKind::Push, WorkKind::Maintenance] {
        let original: WorkItem = item(kind, "abc");
        let text: String = original.encode();
        assert_eq!(text.matches('\n').count(), 1);
        assert_eq!(WorkItem::decode(text.as_str()), Ok(original.clone()));
        assert_eq!(original.id(), format!("{}-abc", kind.name()));
        assert_eq!(WorkKind::from_name(kind.name()), Some(kind));
    }
}

/// Records that are not exactly the format are refused with a reason.
#[test]
fn bad_records_are_refused() {
    let good: String = item(WorkKind::Review, "abc").encode();
    for broken in [
        String::from("not json"),
        good.replace("\"v\":1", "\"v\":2"),
        good.replace("\"kind\":\"review\"", "\"kind\":\"deploy\""),
        good.replace("\"id\":\"review-abc\"", "\"id\":\"review-other\""),
        good.replace("\"oid\":\"abc\"", ""),
        good.replace("\"tries\":0", "\"tries\":-1"),
    ] {
        assert!(WorkItem::decode(broken.as_str()).is_err(), "{broken}");
    }
}

/// The same landing enqueued twice, or after it was claimed or finished, adds nothing.
#[test]
fn enqueue_is_idempotent_across_states() {
    let directory: PathBuf = fixture("work-enqueue");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let first: WorkItem = item(WorkKind::Review, "aaa");
    assert_eq!(enqueue(work.as_path(), &first), Ok(Enqueued::Added));
    assert_eq!(enqueue(work.as_path(), &first), Ok(Enqueued::Known));
    assert_eq!(enqueue(work.as_path(), &item(WorkKind::Push, "aaa")), Ok(Enqueued::Added), "another kind is another item");
    let claimed: Claimed = claim_next(work.as_path(), SystemTime::now()).expect("claim").expect("an item");
    assert_eq!(enqueue(work.as_path(), &claimed.item), Ok(Enqueued::Known));
    settle(work.as_path(), &claimed, DONE).expect("settle");
    assert_eq!(enqueue(work.as_path(), &claimed.item), Ok(Enqueued::Known));
    assert_eq!(names(work.as_path(), DONE), vec![claimed.id.clone()]);
    remove(directory.as_path());
}

/// Claims go oldest first, each item to one claimant, and an empty queue gives none.
#[test]
fn claims_are_oldest_first_and_exclusive() {
    let directory: PathBuf = fixture("work-claim");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let base: SystemTime = SystemTime::now();
    for (index, oid) in ["ccc", "aaa", "bbb"].into_iter().enumerate() {
        enqueue(work.as_path(), &item(WorkKind::Review, oid)).expect("enqueue");
        let file = std::fs::File::options().append(true).open(state_directory(work.as_path(), PENDING).join(format!("review-{oid}"))).expect("open");
        file.set_modified(base + Duration::from_secs(u64::try_from(index).expect("small"))).expect("age");
    }
    let order: Vec<String> = (0..3).map(|_| return claim_next(work.as_path(), base).expect("claim").expect("item").item.oid).collect();
    assert_eq!(order, vec!["ccc", "aaa", "bbb"]);
    assert_eq!(claim_next(work.as_path(), base).expect("claim"), None);
    assert!(names(work.as_path(), PENDING).is_empty());
    remove(directory.as_path());
}

/// A record that does not decode goes to `failed` and does not block the item behind it.
#[test]
fn an_undecodable_record_does_not_block_the_queue() {
    let directory: PathBuf = fixture("work-undecodable");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Review, "good")).expect("enqueue");
    std::fs::write(state_directory(work.as_path(), PENDING).join("review-bad"), "garbage").expect("plant");
    let pending: std::fs::File = std::fs::File::options().append(true).open(state_directory(work.as_path(), PENDING).join("review-bad")).expect("open");
    pending.set_modified(SystemTime::UNIX_EPOCH).expect("oldest");
    let claimed: Claimed = claim_next(work.as_path(), SystemTime::now()).expect("claim").expect("the good item");
    assert_eq!(claimed.item.oid, "good");
    assert_eq!(names(work.as_path(), FAILED), vec![String::from("review-bad")]);
    remove(directory.as_path());
}

/// A claim whose lease ran out returns to pending with one more try, then to failed at the limit; a live lease stays.
#[test]
fn expired_leases_requeue_and_then_fail() {
    let directory: PathBuf = fixture("work-lease");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Review, "lll")).expect("enqueue");
    let mut now: SystemTime = SystemTime::now();
    let held: Claimed = claim_next(work.as_path(), now).expect("claim").expect("item");
    assert_eq!(requeue_expired(work.as_path(), now + LEASE, LEASE, MAX_TRIES), 0, "a lease at its limit still holds");
    assert_eq!(names(work.as_path(), CLAIMED), vec![held.id.clone()]);
    for attempt in 1..=MAX_TRIES {
        now += LEASE * 2;
        assert_eq!(requeue_expired(work.as_path(), now, LEASE, MAX_TRIES), 1);
        if attempt < MAX_TRIES {
            let again: Claimed = claim_next(work.as_path(), now).expect("claim").expect("requeued item");
            assert_eq!(again.item.tries, attempt);
        }
    }
    assert_eq!(names(work.as_path(), FAILED), vec![held.id]);
    assert!(names(work.as_path(), PENDING).is_empty() && names(work.as_path(), CLAIMED).is_empty());
    remove(directory.as_path());
}

/// Every claim returns to pending whatever its age, with one more try each time, and to failed at the limit.
#[test]
fn every_claim_is_requeued_whatever_its_age_and_fails_at_the_limit() {
    let directory: PathBuf = fixture("work-requeue-claimed");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Review, "ccc")).expect("enqueue");
    let now: SystemTime = SystemTime::now();
    let held: Claimed = claim_next(work.as_path(), now).expect("claim").expect("item");
    for attempt in 1..=MAX_TRIES {
        assert_eq!(requeue_claimed(work.as_path(), MAX_TRIES), 1, "a claim stamped this instant is still returned");
        if attempt < MAX_TRIES {
            let again: Claimed = claim_next(work.as_path(), now).expect("claim").expect("requeued item");
            assert_eq!(again.item.tries, attempt);
        }
    }
    assert_eq!(names(work.as_path(), FAILED), vec![held.id]);
    assert_eq!(requeue_claimed(work.as_path(), MAX_TRIES), 0, "nothing is left to return");
    remove(directory.as_path());
}

/// The oldest pending item's age is what the wrapper's notice is based on; an empty queue has none.
#[test]
fn the_oldest_pending_age_is_measured_from_the_oldest_item() {
    let directory: PathBuf = fixture("work-age");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let now: SystemTime = SystemTime::now();
    assert_eq!(oldest_pending_age(work.as_path(), now), None);
    for (oid, age) in [("young", 5_u64), ("old", 500)] {
        enqueue(work.as_path(), &item(WorkKind::Review, oid)).expect("enqueue");
        let file = std::fs::File::options().append(true).open(state_directory(work.as_path(), PENDING).join(format!("review-{oid}"))).expect("open");
        file.set_modified(now - Duration::from_secs(age)).expect("age");
    }
    assert_eq!(oldest_pending_age(work.as_path(), now), Some(Duration::from_secs(500)));
    remove(directory.as_path());
}

/// Two instances that both saw an expired claim: the first returns it to pending and claims it again, then the second, still
///  acting on what it saw, must leave the new claim alone. The seam is the `due` closure, which runs between a recovery's
///  look at the claims and its take.
#[test]
fn a_recovery_that_looked_earlier_leaves_a_claim_acquired_since_alone() {
    let directory: PathBuf = fixture("work-recovery-race");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "rrr")).expect("enqueue");
    let now: SystemTime = SystemTime::now();
    let dead: Claimed = claim_next(work.as_path(), now).expect("claim").expect("item");
    let first_look: std::cell::Cell<bool> = std::cell::Cell::new(true);
    let successor: std::cell::RefCell<Option<Claimed>> = std::cell::RefCell::new(None);
    let moved: usize = requeue_where(work.as_path(), MAX_TRIES, &|_started| {
        if first_look.replace(false) {
            // The other instance's whole recovery and its new claim, which it also renews.
            assert_eq!(requeue_expired(work.as_path(), now + LEASE * 2, LEASE, MAX_TRIES), 1);
            let again: Claimed = claim_next(work.as_path(), now + LEASE * 2).expect("claim").expect("requeued item");
            assert_eq!(again.item.tries, 1);
            *successor.borrow_mut() = Some(again);
        }
        return true;
    });
    assert_eq!(moved, 0, "the claim the second instance saw is gone; it took nothing");
    assert_eq!(names(work.as_path(), CLAIMED), vec![dead.id.clone()], "the successor's claim is still held");
    assert!(names(work.as_path(), PENDING).is_empty(), "nothing was handed out a second time");
    assert!(successor.borrow().is_some());
    remove(directory.as_path());
}

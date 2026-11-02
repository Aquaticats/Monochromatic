//! Controls for the work-item record: idempotent enqueue, exclusive claims, lease expiry on the right clock, recovery that
//! cannot take another instance's claim, and refusal of bad records.

use super::*;
use crate::test_support::{fixture, remove};

/// The lease the tests use.
const LEASE: Duration = Duration::from_secs(60);

/// A boot ID, and another boot's.
const BOOT: &str = "0123456789abcdef0123456789abcdef";
const OTHER_BOOT: &str = "fedcba9876543210fedcba9876543210";

/// The clocks at the start of every test: one boot, a monotonic clock that has run for a while, and a wall clock.
fn clock() -> ClaimTime {
    return ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000));
}

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
    let claimed: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("an item");
    assert_eq!(enqueue(work.as_path(), &claimed.item), Ok(Enqueued::Known), "a claimed item is known by its id, not its file name");
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
    let order: Vec<String> = (0..3).map(|_| return claim_next(work.as_path(), &clock()).expect("claim").expect("item").item.oid).collect();
    assert_eq!(order, vec!["ccc", "aaa", "bbb"]);
    assert_eq!(claim_next(work.as_path(), &clock()).expect("claim"), None);
    assert!(names(work.as_path(), PENDING).is_empty());
    remove(directory.as_path());
}

/// A claim is one file whose name has the id, a generation of its own and the clock reading, and no two claims of the same id
/// share a generation.
#[test]
fn a_claim_is_named_by_its_id_its_generation_and_its_clock_reading() {
    let directory: PathBuf = fixture("work-claim-name");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Review, "nnn")).expect("enqueue");
    let first: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    assert_eq!(names(work.as_path(), CLAIMED), vec![first.name.clone()]);
    assert_eq!(id_of_entry(first.name.as_str()), first.id);
    let (token, stamp) = parse_claim_name(first.name.as_str()).expect("a claim of this release");
    assert_eq!(stamp, clock());
    assert_eq!(first.name, claim_file_name(first.id.as_str(), token, &clock()));
    assert_eq!(requeue_expired(work.as_path(), &clock().advanced(LEASE * 2), LEASE, MAX_TRIES), 1);
    let second: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("requeued item");
    assert_ne!(parse_claim_name(second.name.as_str()).map(|(generation, _)| return generation), Some(token), "a new claim is a new generation");
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
    let claimed: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("the good item");
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
    let mut now: ClaimTime = clock();
    let held: Claimed = claim_next(work.as_path(), &now).expect("claim").expect("item");
    assert_eq!(requeue_expired(work.as_path(), &now.advanced(LEASE), LEASE, MAX_TRIES), 0, "a lease at its limit still holds");
    assert_eq!(names(work.as_path(), CLAIMED), vec![held.name.clone()]);
    for attempt in 1..=MAX_TRIES {
        now = now.advanced(LEASE * 2);
        assert_eq!(requeue_expired(work.as_path(), &now, LEASE, MAX_TRIES), 1);
        if attempt < MAX_TRIES {
            let again: Claimed = claim_next(work.as_path(), &now).expect("claim").expect("requeued item");
            assert_eq!(again.item.tries, attempt);
        }
    }
    assert_eq!(names(work.as_path(), FAILED), vec![held.id]);
    assert!(names(work.as_path(), PENDING).is_empty() && names(work.as_path(), CLAIMED).is_empty() && names(work.as_path(), RECOVERING).is_empty());
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
///  acting on what it saw, must leave the new claim alone. The seam is the `expired` closure, which runs between a recovery's
///  look at the claims and its take.
#[test]
fn a_recovery_that_looked_earlier_leaves_a_claim_acquired_since_alone() {
    let directory: PathBuf = fixture("work-recovery-race");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "rrr")).expect("enqueue");
    let now: ClaimTime = clock();
    let dead: Claimed = claim_next(work.as_path(), &now).expect("claim").expect("item");
    let later: ClaimTime = now.advanced(LEASE * 2);
    let first_look: std::cell::Cell<bool> = std::cell::Cell::new(true);
    let successor: std::cell::RefCell<Option<Claimed>> = std::cell::RefCell::new(None);
    let moved: usize = requeue_where(work.as_path(), &later, MAX_TRIES, &|_stamp| {
        if first_look.replace(false) {
            // The other instance's whole recovery and its new claim.
            assert_eq!(requeue_expired(work.as_path(), &later, LEASE, MAX_TRIES), 1);
            let again: Claimed = claim_next(work.as_path(), &later).expect("claim").expect("requeued item");
            assert_eq!(again.item.tries, 1);
            *successor.borrow_mut() = Some(again);
        }
        return true;
    });
    assert_eq!(moved, 0, "the claim the second instance saw is gone; it took nothing");
    let held: Claimed = successor.borrow().clone().expect("the successor claimed the item");
    assert_eq!(names(work.as_path(), CLAIMED), vec![held.name.clone()], "the successor's claim is still held");
    assert_ne!(held.name, dead.name);
    assert!(names(work.as_path(), PENDING).is_empty() && names(work.as_path(), RECOVERING).is_empty(), "nothing was handed out a second time");
    // The first holder, which the successor replaced, can neither renew nor settle the successor's claim.
    assert_eq!(renew_claim(work.as_path(), dead.name.as_str(), &later), None);
    assert!(settle(work.as_path(), &dead, DONE).is_err());
    assert_eq!(names(work.as_path(), CLAIMED), vec![held.name.clone()]);
    assert!(names(work.as_path(), DONE).is_empty());
    settle(work.as_path(), &held, DONE).expect("the successor settles its own claim");
    remove(directory.as_path());
}

/// A holder that renews between a recovery's look and its take keeps the claim: the renewal changed the name the recovery
/// means to take.
#[test]
fn a_renewal_between_the_look_and_the_take_keeps_the_claim() {
    let directory: PathBuf = fixture("work-renewal-race");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "sss")).expect("enqueue");
    let now: ClaimTime = clock();
    let held: Claimed = claim_next(work.as_path(), &now).expect("claim").expect("item");
    let later: ClaimTime = now.advanced(LEASE * 2);
    let renewed: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
    let moved: usize = requeue_where(work.as_path(), &later, MAX_TRIES, &|_stamp| {
        *renewed.borrow_mut() = renew_claim(work.as_path(), held.name.as_str(), &later);
        return true;
    });
    assert_eq!(moved, 0);
    let name: String = renewed.borrow().clone().expect("the holder renewed");
    assert_eq!(names(work.as_path(), CLAIMED), vec![name.clone()]);
    assert_eq!(parse_claim_name(name.as_str()).map(|(_, stamp)| return stamp), Some(later.clone()), "the renewal carries the new reading");
    assert_eq!(requeue_expired(work.as_path(), &later.advanced(LEASE), LEASE, MAX_TRIES), 0, "a renewed claim is held for another lease");
    assert_eq!(requeue_expired(work.as_path(), &later.advanced(LEASE * 2), LEASE, MAX_TRIES), 1, "and no longer than that");
    remove(directory.as_path());
}

/// A step of the wall clock forward must not make a healthy holder look dead, and a step back must not keep a dead holder's
/// claim alive: within one boot only the monotonic clock counts.
#[test]
fn a_wall_clock_step_neither_expires_a_live_holder_nor_keeps_a_dead_one() {
    let directory: PathBuf = fixture("work-clock-step");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "ttt")).expect("enqueue");
    let start: ClaimTime = clock();
    let held: Claimed = claim_next(work.as_path(), &start).expect("claim").expect("item");
    // The holder renews a minute in, then the wall clock jumps an hour forward; the next renewal is not due for a minute.
    let renewed_at: ClaimTime = start.advanced(Duration::from_secs(60));
    let name: String = renew_claim(work.as_path(), held.name.as_str(), &renewed_at).expect("renewed");
    let observer: ClaimTime = renewed_at.advanced(Duration::from_secs(30)).wall_stepped_forward(Duration::from_secs(3600));
    assert_eq!(requeue_expired(work.as_path(), &observer, LEASE, MAX_TRIES), 0, "thirty seconds after a renewal is live whatever the wall clock says");
    assert_eq!(names(work.as_path(), CLAIMED), vec![name]);
    // The holder dies. An hour later by the monotonic clock, the wall clock has been set back a day.
    let late: ClaimTime = renewed_at.advanced(Duration::from_secs(3600)).wall_stepped_back(Duration::from_secs(24 * 3600));
    assert_eq!(requeue_expired(work.as_path(), &late, LEASE, MAX_TRIES), 1, "a dead holder's claim runs out whatever the wall clock says");
    remove(directory.as_path());
}

/// Across boots the monotonic readings cannot be compared, so the wall clock is the fallback: a claim of an earlier boot
/// runs out by wall time.
#[test]
fn a_claim_of_an_earlier_boot_runs_out_by_the_wall_clock() {
    let directory: PathBuf = fixture("work-clock-boot");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "bbb")).expect("enqueue");
    let before_reboot: ClaimTime = clock();
    claim_next(work.as_path(), &before_reboot).expect("claim").expect("item");
    // The new boot's monotonic clock is far behind the old one's reading, which would make the claim look as young as new.
    let soon: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 10));
    assert_eq!(requeue_expired(work.as_path(), &soon, LEASE, MAX_TRIES), 0, "ten seconds of wall time is within the lease");
    let after: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 61));
    assert_eq!(requeue_expired(work.as_path(), &after, LEASE, MAX_TRIES), 1, "a minute of wall time is not");
    remove(directory.as_path());
}

/// How long until the first claim runs out is the shortest time left on any claim, and nothing when none is held.
#[test]
fn the_next_expiry_is_the_shortest_time_left() {
    let directory: PathBuf = fixture("work-next-expiry");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let start: ClaimTime = clock();
    assert_eq!(next_claim_expiry(work.as_path(), &start, LEASE), None);
    enqueue(work.as_path(), &item(WorkKind::Push, "one")).expect("enqueue");
    claim_next(work.as_path(), &start).expect("claim").expect("item");
    enqueue(work.as_path(), &item(WorkKind::Push, "two")).expect("enqueue");
    claim_next(work.as_path(), &start.advanced(Duration::from_secs(20))).expect("claim").expect("item");
    assert_eq!(next_claim_expiry(work.as_path(), &start.advanced(Duration::from_secs(30)), LEASE), Some(Duration::from_secs(30)));
    assert_eq!(next_claim_expiry(work.as_path(), &start.advanced(LEASE * 3), LEASE), Some(Duration::ZERO), "an expired claim has no time left");
    remove(directory.as_path());
}

/// A claim by an earlier release is named by its id alone and aged by its file's modification time on the wall clock; it
/// is recovered like any other, and renewing it gives it a generation of this release.
#[test]
fn a_claim_by_an_earlier_release_is_aged_by_its_modification_time() {
    let directory: PathBuf = fixture("work-legacy-claim");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let legacy: WorkItem = item(WorkKind::Push, "old");
    enqueue(work.as_path(), &legacy).expect("enqueue");
    let id: String = legacy.id();
    std::fs::rename(state_directory(work.as_path(), PENDING).join(id.as_str()), state_directory(work.as_path(), CLAIMED).join(id.as_str())).expect("claimed the old way");
    let stamped: SystemTime = SystemTime::now();
    std::fs::File::options().append(true).open(state_directory(work.as_path(), CLAIMED).join(id.as_str())).and_then(|file| return file.set_modified(stamped)).expect("stamp");
    assert_eq!(enqueue(work.as_path(), &legacy), Ok(Enqueued::Known));
    let seen: ClaimTime = ClaimTime::from_wall(stamped);
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(LEASE), LEASE, MAX_TRIES), 0);
    let renewed: String = renew_claim(work.as_path(), id.as_str(), &seen.advanced(LEASE)).expect("renewed");
    assert_ne!(renewed, id);
    assert_eq!(id_of_entry(renewed.as_str()), id);
    assert!(parse_claim_name(renewed.as_str()).is_some());
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(LEASE * 3), LEASE, MAX_TRIES), 1);
    assert_eq!(names(work.as_path(), PENDING), vec![id]);
    remove(directory.as_path());
}

/// A recovery killed after it took a claim leaves the item in `recovering`, exactly once; that entry runs out like a claim
/// and is then returned, and while it is young it is left alone.
#[test]
fn a_recovery_killed_after_the_take_is_finished_by_the_next_one() {
    let directory: PathBuf = fixture("work-recovery-killed");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "kkk")).expect("enqueue");
    let start: ClaimTime = clock();
    let held: Claimed = claim_next(work.as_path(), &start).expect("claim").expect("item");
    // What a recovery that was killed right after its take leaves: the claim, renamed, in `recovering`.
    create_private_directory_all(state_directory(work.as_path(), RECOVERING).as_path()).expect("directory");
    let taken_at: ClaimTime = start.advanced(LEASE * 2);
    let taken: String = claim_file_name(held.id.as_str(), "0123456789abcdef0123456789abcdef", &taken_at);
    std::fs::rename(state_directory(work.as_path(), CLAIMED).join(held.name.as_str()), state_directory(work.as_path(), RECOVERING).join(taken.as_str())).expect("taken");
    assert_eq!(enqueue(work.as_path(), &held.item), Ok(Enqueued::Known), "an item being recovered is known");
    assert_eq!(requeue_expired(work.as_path(), &taken_at.advanced(Duration::from_secs(1)), LEASE, MAX_TRIES), 0, "a recovery in progress is left alone");
    assert_eq!(requeue_expired(work.as_path(), &taken_at.advanced(LEASE * 2), LEASE, MAX_TRIES), 1);
    assert_eq!(names(work.as_path(), PENDING), vec![held.id.clone()]);
    assert!(names(work.as_path(), RECOVERING).is_empty() && names(work.as_path(), CLAIMED).is_empty());
    let again: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    assert_eq!(again.item.tries, 1);
    remove(directory.as_path());
}

/// A temporary file of a recovery that was killed is removed once it is a day old, and not before.
#[test]
fn a_stray_recovery_temporary_file_is_removed_when_old() {
    let directory: PathBuf = fixture("work-stray-temporary");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    create_private_directory_all(state_directory(work.as_path(), RECOVERING).as_path()).expect("directory");
    let stray: PathBuf = state_directory(work.as_path(), RECOVERING).join("push-abc~0123~x.retry");
    std::fs::write(stray.as_path(), "partial").expect("stray");
    let modified: SystemTime = SystemTime::now();
    std::fs::File::options().append(true).open(stray.as_path()).and_then(|file| return file.set_modified(modified)).expect("stamp");
    let seen: ClaimTime = ClaimTime::from_wall(modified);
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(Duration::from_secs(3600)), LEASE, MAX_TRIES), 0);
    assert!(stray.exists(), "an hour old is young");
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(STRAY_TEMPORARY_AGE + Duration::from_secs(1)), LEASE, MAX_TRIES), 0);
    assert!(!stray.exists());
    remove(directory.as_path());
}

/// A claim whose record cannot be read once taken is failed in that pass, so it is not taken again every lease and does not keep
/// a service waiting; the directory stands in for a file that cannot be read.
#[test]
fn a_claim_that_cannot_be_read_is_failed_in_one_pass() {
    let directory: PathBuf = fixture("work-unreadable-claim");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "uuu")).expect("enqueue");
    create_private_directory_all(state_directory(work.as_path(), CLAIMED).as_path()).expect("directory");
    let unreadable: String = claim_file_name("push-unreadable", "0123456789abcdef0123456789abcdef", &clock());
    std::fs::create_dir(state_directory(work.as_path(), CLAIMED).join(unreadable)).expect("a claim that is not a readable file");
    assert_eq!(requeue_expired(work.as_path(), &clock().advanced(LEASE * 2), LEASE, MAX_TRIES), 1);
    assert_eq!(names(work.as_path(), FAILED), vec![String::from("push-unreadable")]);
    assert!(names(work.as_path(), CLAIMED).is_empty() && names(work.as_path(), RECOVERING).is_empty());
    assert_eq!(next_claim_expiry(work.as_path(), &clock().advanced(LEASE * 2), LEASE), None, "nothing is left to wait for");
    remove(directory.as_path());
}

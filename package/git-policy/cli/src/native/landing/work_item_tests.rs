//! Controls for the work-item record:
//! idempotent enqueue,
//! exclusive claims,
//! lease expiry on the right clock,
//! recovery that
//! cannot take another instance's claim,
//! and refusal of bad records.

use crate::work_item_recovery::{Recovered, next_claim_expiry, requeue_expired, requeue_expired_reporting};
use crate::work_item_state_directories_and_claim_names::{NAME_SEPARATOR, take_listings};
use super::*;
use crate::test_support::{fixture, remove};
use crate::work_item_recovery::{ClaimExpiry, STRAY_TEMPORARY_AGE, TEMPORARY_SUFFIX, requeue_with};

/// A function-like value with the right shape is an expiry test in these controls:
/// the shipped caller writes a small struct.
impl<Test: Fn(&ClaimTime) -> bool> ClaimExpiry for Test {
    fn has_expired(&self, stamp: &ClaimTime) -> bool {
        return self(stamp);
    }
}

/// The lease the tests use.
const LEASE: Duration = Duration::from_secs(60);

/// A boot ID,
/// and another boot's.
const BOOT: &str = "0123456789abcdef0123456789abcdef";
const OTHER_BOOT: &str = "fedcba9876543210fedcba9876543210";

/// A machine ID,
/// and another machine's.
const MACHINE: &str = "00112233445566778899aabbccddeeff";
const OTHER_MACHINE: &str = "ffeeddccbbaa99887766554433221100";

/// The longest a file name may be on the filesystems the work directory is kept on.
const FILE_NAME_MAX: usize = 255;

/// The clocks at the start of every test:
/// one boot,
/// a monotonic clock that has run for a while,
/// and a wall clock.
fn clock() -> ClaimTime {
    return ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000)).on_machine(MACHINE);
}

/// The attempts after which an item is given up on.
const MAX_TRIES: u32 = 3;

/// An item for `oid` of `kind`.
fn item(kind: WorkKind, oid: &str) -> WorkItem {
    return WorkItem {
        kind,
        repository: PathBuf::from("/repo with space"),
        git_dir: PathBuf::from("/repo with space/.git"),
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

/// A record decodes to the item it encoded,
/// one line,
/// with an id derived from kind and object.
#[test]
fn a_record_round_trips() {
    for kind in [WorkKind::Review, WorkKind::Push, WorkKind::Maintenance] {
        let original: WorkItem = item(kind, "abc");
        let text: String = original.encode();
        assert_eq!(text.matches('\n').count(), 1);
        assert_eq!(WorkItem::decode(text.as_str()), Ok(original.clone()));
        let expected: String = match kind {
            WorkKind::Push => format!("{}-abc-{}", kind.name(), crate::work_item_ids_and_names::branch_digest("refs/heads/main")),
            other => format!("{}-abc", other.name()),
        };
        assert_eq!(original.id(), expected);
        assert_eq!(WorkKind::from_name(kind.name()), Some(kind));
    }
}

/// What:
///  A record keeps the directories of a repository whose path is not UTF-8,
/// byte for byte.
/// Why:
///   The record is text,
/// and the service compares the Git directory it reads back with the directory it drains,
/// so a path that text cannot spell would refuse every item of such a repository.
#[test]
fn a_record_keeps_the_bytes_of_a_path_that_is_not_text() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("work-item-non-utf8");
    let worktree: PathBuf = root.join(OsStr::from_bytes(b"w\xffrktree"));
    let git_dir: PathBuf = root.join(OsStr::from_bytes(b"c\xffmmon"));
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let original: WorkItem = WorkItem { repository: worktree.clone(), git_dir: git_dir.clone(), ..item(WorkKind::Push, "abc") };
    let read: WorkItem = WorkItem::decode(original.encode().as_str()).expect("the record decodes");
    assert_eq!(read.git_dir, git_dir, "the Git directory is the one the wrapper named");
    assert_eq!(read.repository, worktree, "the worktree is the one the wrapper named");
    assert_eq!(read, original);
    remove(root.as_path());
}

/// What:
///  A record written by a release that spelled a directory as plain text is still read,
/// and that text is the directory.
/// Why:
///   A service that starts across a pending item of an upgrade must run it,
/// not fail it,
/// so the form the earlier release wrote has to decode.
#[test]
fn a_record_of_plain_text_directories_is_still_read() {
    // The one line the release before the Hexadecimal field wrote for the same item.
    let earlier: &str = "{\"v\":1,\"tries\":0,\"kind\":\"push\",\"id\":\"push-abc\",\"repository\":\"/srv/repo\",\"gitDir\":\"/srv/repo/.git\",\"ref\":\"refs/heads/main\",\"oid\":\"abc\",\"parent\":\"\",\"created\":\"2026-11-01T00:00:00.000Z\"}\n";
    let read: WorkItem = WorkItem::decode(earlier).expect("an earlier release's record decodes");
    assert_eq!(read.git_dir, PathBuf::from("/srv/repo/.git"));
    assert_eq!(read.repository, PathBuf::from("/srv/repo"));
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

/// A record of a push on `branch` for `oid`.
fn push_of(branch: &str, oid: &str) -> WorkItem {
    return WorkItem { branch_ref: String::from(branch), ..item(WorkKind::Push, oid) };
}

/// Two branches can hold one commit,
/// and each must reach its own remote:
/// the record of the second branch is an item of its own,
/// not one the queue already knew.
#[test]
fn two_branches_at_one_commit_are_both_queued() {
    let directory: PathBuf = fixture("work-two-branches-one-commit");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let oid: &str = "abc";
    let topic: WorkItem = push_of("refs/heads/topic", oid);
    let release: WorkItem = push_of("refs/heads/release", oid);
    assert_ne!(topic.id(), release.id(), "one commit on two branches is two records");
    assert_eq!(enqueue(work.as_path(), &topic), Ok(Enqueued::Added));
    let first: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("the topic item");
    settle(work.as_path(), &first, DONE).expect("settle");
    assert_eq!(enqueue(work.as_path(), &release), Ok(Enqueued::Added), "a record that names another branch does not stand for this one");
    let second: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("the release item");
    assert_eq!(second.item.branch_ref, String::from("refs/heads/release"));
    remove(directory.as_path());
}

/// A pending record an earlier release wrote is named by its kind and object alone,
/// and this release still reads and runs it.
#[test]
fn a_pending_record_of_an_earlier_release_is_claimed() {
    let directory: PathBuf = fixture("work-earlier-release-name");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let earlier: &str = "{\"v\":1,\"tries\":0,\"kind\":\"push\",\"id\":\"push-abc\",\"repository\":\"/repo\",\"gitDir\":\"/repo/.git\",\"ref\":\"refs/heads/main\",\"oid\":\"abc\",\"parent\":\"\",\"created\":\"2026-11-01T00:00:00.000Z\"}\n";
    for state in [PENDING, CLAIMED] {
        std::fs::create_dir_all(state_directory(work.as_path(), state)).expect("a state directory");
    }
    std::fs::write(state_directory(work.as_path(), PENDING).join("push-abc"), earlier).expect("a record an earlier release left");
    let claimed: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("the earlier release's item");
    assert_eq!(claimed.item.branch_ref, String::from("refs/heads/main"));
    assert_eq!(claimed.item.oid, "abc");
    remove(directory.as_path());
}

/// The same landing enqueued twice,
/// or after it was claimed or finished,
/// adds nothing.
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

/// The listings one producer's round of `count` items costs.
fn listings_of_an_enqueue_round(count: usize) -> usize {
    let directory: PathBuf = fixture(format!("work-enqueue-listings-{count}").as_str());
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let items: Vec<WorkItem> = (0..count).map(|n| { return item(WorkKind::Push, format!("{n:040x}").as_str()); }).collect();
    let _nothing_yet: usize = take_listings();
    let answers: Vec<Enqueued> = enqueue_all(work.as_path(), items.as_slice()).expect("the producer's round");
    let listings: usize = take_listings();
    assert_eq!(answers, vec![Enqueued::Added; items.len()], "every item of the round is new");
    assert_eq!(names(work.as_path(), PENDING).len(), items.len(), "the round of {count} items queued every one of them");
    remove(directory.as_path());
    return listings;
}

/// A producer of many items walks the work queue once per round and not once per item (`RG2`):
/// the number of directory listings it costs does not grow with the number of items it queues.
#[test]
fn a_producer_of_many_items_lists_the_state_directories_once() {
    let measured: Vec<(usize, usize)> = [4, 24, 96].iter().map(|count| { return (*count, listings_of_an_enqueue_round(*count)); }).collect();
    let first: usize = measured[0].1;
    for (_count, listings) in &measured {
        assert_eq!(*listings, first, "the listings of a producer's round do not grow with its items: {measured:?}");
    }
}

/// Claims go oldest first,
/// each item to one claimant,
/// and an empty queue gives none.
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

/// A claim is one file whose name has the id,
/// a generation of its own and the clock reading,
/// and no two claims of the same id
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

/// A claim whose lease ran out returns to pending with one more try,
/// then to failed at the limit;
/// a live lease stays.
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

/// The oldest pending item's age is what the wrapper's notice is based on;
/// an empty queue has none.
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

/// Two instances that both saw an expired claim:
/// the first returns it to pending and claims it again,
/// then the second,
/// still
///  acting on what it saw,
/// must leave the new claim alone.
/// The seam is the `expired` closure,
/// which runs between a recovery's
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
    let moved: usize = requeue_with(work.as_path(), &later, MAX_TRIES, &|_stamp: &ClaimTime| {
        if first_look.replace(false) {
            // The other instance's whole recovery and its new claim.
            assert_eq!(requeue_expired(work.as_path(), &later, LEASE, MAX_TRIES), 1);
            let again: Claimed = claim_next(work.as_path(), &later).expect("claim").expect("requeued item");
            assert_eq!(again.item.tries, 1);
            *successor.borrow_mut() = Some(again);
        }
        return true;
    })
    .moved;
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

/// A holder that renews between a recovery's look and its take keeps the claim:
/// the renewal changed the name the recovery
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
    let moved: usize = requeue_with(work.as_path(), &later, MAX_TRIES, &|_stamp: &ClaimTime| {
        *renewed.borrow_mut() = renew_claim(work.as_path(), held.name.as_str(), &later);
        return true;
    })
    .moved;
    assert_eq!(moved, 0);
    let name: String = renewed.borrow().clone().expect("the holder renewed");
    assert_eq!(names(work.as_path(), CLAIMED), vec![name.clone()]);
    assert_eq!(parse_claim_name(name.as_str()).map(|(_, stamp)| return stamp), Some(later.clone()), "the renewal carries the new reading");
    assert_eq!(requeue_expired(work.as_path(), &later.advanced(LEASE), LEASE, MAX_TRIES), 0, "a renewed claim is held for another lease");
    assert_eq!(requeue_expired(work.as_path(), &later.advanced(LEASE * 2), LEASE, MAX_TRIES), 1, "and no longer than that");
    remove(directory.as_path());
}

/// A step of the wall clock forward must not make a healthy holder look dead,
/// and a step back must not keep a dead holder's
/// claim alive:
/// within one boot only the monotonic clock counts.
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

/// A claim file name carries its holder's boot ID.
/// When the judge's boot differs,
/// the holder of the earlier boot is dead,
/// so the claim has run out at once,
/// however little wall time has passed and whichever way the wall clock was set.
#[test]
fn a_claim_of_an_earlier_boot_has_run_out_at_once() {
    let directory: PathBuf = fixture("work-clock-boot");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "bbb")).expect("enqueue");
    let before_reboot: ClaimTime = clock();
    claim_next(work.as_path(), &before_reboot).expect("claim").expect("item");
    // The new boot's wall clock was set back at boot, so it reads earlier than the claim's own stamp.
    let set_back: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 - 3600)).on_machine(MACHINE);
    assert_eq!(next_claim_expiry(work.as_path(), &set_back, LEASE), Some(Duration::ZERO), "nothing is left to wait for");
    assert_eq!(requeue_expired(work.as_path(), &set_back, LEASE, MAX_TRIES), 1, "a clock set back at boot does not strand it");
    remove(directory.as_path());
}

/// A claim of another machine (a second host on a shared filesystem) is not of an earlier boot,
/// whatever the boot IDs say:
/// it
/// runs out by its lease on the wall clock,
/// and not before.
#[test]
fn a_claim_of_another_machine_runs_out_by_the_wall_clock() {
    let directory: PathBuf = fixture("work-clock-other-machine");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "eee")).expect("enqueue");
    claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    let soon: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 10)).on_machine(OTHER_MACHINE);
    assert_eq!(next_claim_expiry(work.as_path(), &soon, LEASE), Some(LEASE - Duration::from_secs(10)), "the lease is what is left");
    assert_eq!(requeue_expired(work.as_path(), &soon, LEASE, MAX_TRIES), 0, "ten seconds of wall time is within the lease");
    let after: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 61)).on_machine(OTHER_MACHINE);
    assert_eq!(requeue_expired(work.as_path(), &after, LEASE, MAX_TRIES), 1, "a minute of wall time is not");
    remove(directory.as_path());
}

/// A claim an earlier release named (three-part reading,
/// no machine ID) is still read,
/// and is judged by its lease even by a
/// judge of another boot:
/// an old claim left by a reboot waits out its lease once.
#[test]
fn a_claim_in_the_old_file_name_form_runs_out_by_the_wall_clock() {
    let directory: PathBuf = fixture("work-clock-old-form");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "fff")).expect("enqueue");
    let old_reading: ClaimTime = ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(1_700_000_000));
    let claimed: Claimed = claim_next(work.as_path(), &old_reading).expect("claim").expect("item");
    assert_eq!(claimed.name.matches(NAME_SEPARATOR).count(), 4, "id, generation and the three parts of the old reading");
    let soon: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 10)).on_machine(MACHINE);
    assert_eq!(requeue_expired(work.as_path(), &soon, LEASE, MAX_TRIES), 0, "a reboot is not known, so the lease holds");
    let after: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 61)).on_machine(MACHINE);
    assert_eq!(requeue_expired(work.as_path(), &after, LEASE, MAX_TRIES), 1, "and it runs out once the lease is over");
    remove(directory.as_path());
}

/// The longest claim file name the code can make,
/// with the longest kind,
/// a SHA-256 object name,
/// and a reading whose
/// numbers fill 64 bits,
/// fits a file name,
/// with the suffix of a recovery's temporary file too.
#[test]
fn the_longest_claim_file_name_fits_a_file_name() {
    let longest_kind: usize = [WorkKind::Review, WorkKind::Push, WorkKind::Maintenance].into_iter().map(|kind| { return kind.name().len(); }).max().expect("kinds");
    let oid: String = "a".repeat(64);
    let id: String = item(WorkKind::Maintenance, oid.as_str()).id();
    assert_eq!(id.len(), longest_kind + 1 + oid.len());
    let widest: ClaimTime = ClaimTime::from_parts(BOOT, Duration::from_millis(u64::MAX), Duration::from_secs(u64::MAX)).on_machine(MACHINE);
    let name: String = claim_file_name(id.as_str(), "0123456789abcdef0123456789abcdef", &widest);
    assert!(name.len() + TEMPORARY_SUFFIX.len() <= FILE_NAME_MAX, "{} bytes: {name}", name.len());
    assert!(parse_claim_name(name.as_str()).is_some_and(|(_, stamp)| { return stamp == widest; }), "and it reads back whole");
}

/// A legacy claim has no machine ID,
/// so a judge whose wall clock was set back at boot has nothing to measure it by until that
/// clock passes the claim's own timestamp:
/// it recovers at the old timestamp plus the lease,
/// not one lease after the restart.
#[test]
fn a_legacy_claim_waits_for_the_wall_clock_to_pass_its_own_timestamp() {
    let directory: PathBuf = fixture("work-clock-legacy-set-back");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "abc")).expect("enqueue");
    let claimed_at: u64 = 1_700_000_000;
    let legacy: ClaimTime = ClaimTime::from_parts(BOOT, Duration::from_secs(5000), Duration::from_secs(claimed_at));
    claim_next(work.as_path(), &legacy).expect("claim").expect("item");
    let set_back: u64 = 3600;
    let lease_over_by_restart: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(claimed_at - set_back + LEASE.as_secs() + 1)).on_machine(MACHINE);
    assert_eq!(requeue_expired(work.as_path(), &lease_over_by_restart, LEASE, MAX_TRIES), 0, "one lease after the restart is not enough");
    let past_the_stamp: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(claimed_at + LEASE.as_secs() + 1)).on_machine(MACHINE);
    assert_eq!(requeue_expired(work.as_path(), &past_the_stamp, LEASE, MAX_TRIES), 1, "the old timestamp plus the lease is");
    remove(directory.as_path());
}

/// The modification time of `path`.
fn modified_of(path: &Path) -> SystemTime {
    return std::fs::metadata(path).expect("metadata").modified().expect("modification time");
}

/// Sets the modification time of `path` to an hour ago,
/// which is how long an item waited in the queue.
fn age_an_hour(path: &Path) {
    let file: std::fs::File = std::fs::File::open(path).expect("open");
    file.set_modified(SystemTime::now() - Duration::from_secs(3600)).expect("set the time");
}

/// A release that cannot read the reading in a claim's name judges the claim by its file's modification time,
/// which a rename
/// keeps.
/// The claim and every renewal must leave that time at the moment they happened,
/// or such a reader takes a live claim.
#[test]
fn a_claim_and_a_renewal_leave_the_modification_time_at_their_own_moment() {
    let directory: PathBuf = fixture("work-clock-modified");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "abd")).expect("enqueue");
    age_an_hour(state_directory(work.as_path(), PENDING).join(item(WorkKind::Push, "abd").id()).as_path());
    let before_claim: SystemTime = SystemTime::now();
    let claimed: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    let claimed_path: PathBuf = state_directory(work.as_path(), CLAIMED).join(claimed.name.as_str());
    assert!(modified_of(claimed_path.as_path()) >= before_claim, "the claim is as old as the claim");
    age_an_hour(claimed_path.as_path());
    let before_renewal: SystemTime = SystemTime::now();
    let renewed: String = renew_claim(work.as_path(), claimed.name.as_str(), &clock().advanced(LEASE / 2)).expect("renewal");
    let renewed_path: PathBuf = state_directory(work.as_path(), CLAIMED).join(renewed.as_str());
    assert!(modified_of(renewed_path.as_path()) >= before_renewal, "the renewal is as old as the renewal");
    remove(directory.as_path());
}

/// An id is a kind and an object name of at most 64 letters and digits;
/// any other entry of `pending` is moved to `failed`
/// and never claimed,
/// since its claim's name could fit a file name while the name of the temporary file of its recovery did not.
#[test]
fn an_entry_whose_id_is_not_a_kind_and_an_object_name_is_failed_not_claimed() {
    let directory: PathBuf = fixture("work-malformed-id");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let longest_name: String = "a".repeat(64);
    let malformed: Vec<String> = vec!["a".repeat(120), "a".repeat(65), String::from("a.b"), String::from("ab~cd"), String::new()];
    for oid in &malformed {
        enqueue(work.as_path(), &item(WorkKind::Maintenance, oid.as_str())).expect("enqueue");
    }
    enqueue(work.as_path(), &item(WorkKind::Maintenance, longest_name.as_str())).expect("enqueue");
    let claimed: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("the well-formed entry");
    assert_eq!(claimed.id, item(WorkKind::Maintenance, longest_name.as_str()).id());
    assert_eq!(claim_next(work.as_path(), &clock()).expect("claim"), None, "nothing else is claimable");
    assert_eq!(names(work.as_path(), PENDING), Vec::<String>::new());
    assert_eq!(names(work.as_path(), FAILED).len(), malformed.len(), "every malformed entry is failed");
    // Even long after the lease, recovery has only the one well-formed claim to take, and it completes.
    assert_eq!(requeue_expired(work.as_path(), &clock().advanced(LEASE * 2), LEASE, MAX_TRIES), 1);
    assert_eq!(names(work.as_path(), RECOVERING), Vec::<String>::new());
    remove(directory.as_path());
}

/// A judge with no boot ID cannot tell the claim's boot from its own,
/// so the wall clock is what is left:
/// the claim runs out
/// by wall time,
/// and not before.
#[test]
fn a_claim_judged_without_a_boot_runs_out_by_the_wall_clock() {
    let directory: PathBuf = fixture("work-clock-no-boot");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "ccc")).expect("enqueue");
    claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    let soon: ClaimTime = ClaimTime::from_parts("", Duration::ZERO, Duration::from_secs(1_700_000_000 + 10));
    assert_eq!(requeue_expired(work.as_path(), &soon, LEASE, MAX_TRIES), 0, "ten seconds of wall time is within the lease");
    let after: ClaimTime = ClaimTime::from_parts("", Duration::ZERO, Duration::from_secs(1_700_000_000 + 61));
    assert_eq!(requeue_expired(work.as_path(), &after, LEASE, MAX_TRIES), 1, "a minute of wall time is not");
    remove(directory.as_path());
}

/// A claim stamped with no boot ID (a host without one) is judged by the wall clock even by a judge that has a boot ID.
#[test]
fn a_claim_stamped_without_a_boot_runs_out_by_the_wall_clock() {
    let directory: PathBuf = fixture("work-clock-claim-no-boot");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "ddd")).expect("enqueue");
    let unbooted: ClaimTime = ClaimTime::from_parts("", Duration::ZERO, Duration::from_secs(1_700_000_000));
    claim_next(work.as_path(), &unbooted).expect("claim").expect("item");
    let soon: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 10));
    assert_eq!(requeue_expired(work.as_path(), &soon, LEASE, MAX_TRIES), 0, "ten seconds of wall time is within the lease");
    let after: ClaimTime = ClaimTime::from_parts(OTHER_BOOT, Duration::from_secs(5), Duration::from_secs(1_700_000_000 + 61));
    assert_eq!(requeue_expired(work.as_path(), &after, LEASE, MAX_TRIES), 1, "a minute of wall time is not");
    remove(directory.as_path());
}

/// How long until the first claim runs out is the shortest time left on any claim,
/// and nothing when none is held.
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

/// A claim by an earlier release is named by its id alone and aged by its file's modification time on the wall clock;
/// it
/// is recovered like any other,
/// and renewing it gives it a generation of this release.
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

/// A recovery killed after it took a claim leaves the item in `recovering`,
/// exactly once;
/// that entry runs out like a claim
/// and is then returned,
/// and while it is young it is left alone.
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

/// A day,
/// which is how old a killed recovery's temporary file must be before it is removed.
const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// A temporary file of a killed recovery in `recovering`,
/// stamped now,
/// and the clock reading of that stamp.
fn stray_temporary(work: &Path) -> (PathBuf, ClaimTime) {
    create_private_directory_all(state_directory(work, RECOVERING).as_path()).expect("directory");
    let stray: PathBuf = state_directory(work, RECOVERING).join("push-abc~0123~x.retry");
    std::fs::write(stray.as_path(), "partial").expect("stray");
    let modified: SystemTime = SystemTime::now();
    std::fs::File::options().append(true).open(stray.as_path()).and_then(|file| return file.set_modified(modified)).expect("stamp");
    return (stray, ClaimTime::from_wall(modified));
}

/// A temporary file of a recovery that was killed is removed once it is a day old,
/// and not before.
#[test]
fn a_stray_recovery_temporary_file_is_removed_when_old() {
    let directory: PathBuf = fixture("work-stray-temporary");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let (stray, seen): (PathBuf, ClaimTime) = stray_temporary(work.as_path());
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(Duration::from_secs(3600)), LEASE, MAX_TRIES), 0);
    assert!(stray.exists(), "an hour old is young");
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(STRAY_TEMPORARY_AGE + Duration::from_secs(1)), LEASE, MAX_TRIES), 0);
    assert!(!stray.exists());
    remove(directory.as_path());
}

/// A temporary file that is a minute short of a day old is still young:
/// the age is a day,
/// not the hour or so a mistyped
/// product of its factors would make it,
/// which would remove a file a slow recovery may still be using.
#[test]
fn a_stray_recovery_temporary_file_a_minute_short_of_a_day_is_kept() {
    let directory: PathBuf = fixture("work-stray-temporary-young");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let (stray, seen): (PathBuf, ClaimTime) = stray_temporary(work.as_path());
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(A_DAY - Duration::from_secs(60)), LEASE, MAX_TRIES), 0);
    assert!(stray.exists(), "a minute short of a day is young");
    remove(directory.as_path());
}

/// A claim whose record cannot be read once taken is failed in that pass,
/// so it is not taken again every lease and does not keep
/// a service waiting;
/// the directory stands in for a file that cannot be read.
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

/// A recovery that moves an item to `failed` for want of tries gives the item back,
/// so the caller can tell the person;
/// a
/// recovery that returns it to `pending` gives nothing.
#[test]
fn a_recovery_reports_the_items_it_gives_up_on() {
    let directory: PathBuf = fixture("work-given-up");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let last_try: WorkItem = WorkItem { tries: MAX_TRIES - 1, ..item(WorkKind::Push, "ggg") };
    let spare_try: WorkItem = item(WorkKind::Push, "hhh");
    enqueue(work.as_path(), &last_try).expect("enqueue");
    enqueue(work.as_path(), &spare_try).expect("enqueue");
    claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    let recovered: Recovered = requeue_expired_reporting(work.as_path(), &clock().advanced(LEASE * 2), LEASE, MAX_TRIES);
    assert_eq!(recovered.moved, 2);
    assert_eq!(recovered.given_up.iter().map(WorkItem::id).collect::<Vec<String>>(), vec![last_try.id()]);
    assert_eq!(names(work.as_path(), FAILED), vec![last_try.id()]);
    assert_eq!(names(work.as_path(), PENDING), vec![spare_try.id()]);
    remove(directory.as_path());
}

/// A pending record that cannot be written is an error that names the item,
/// never an item that is "known":
/// a landing that
/// believed its review or push was already queued would lose the work without a trace.
/// The object name is one character
/// longer than a file name may be,
/// so the write fails for a reason other than the file existing.
#[test]
fn a_record_that_cannot_be_written_is_an_error_and_not_known() {
    let directory: PathBuf = fixture("work-unwritable");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let too_long: WorkItem = item(WorkKind::Push, "a".repeat(FILE_NAME_MAX + 1).as_str());
    let outcome: Result<Enqueued, String> = enqueue(work.as_path(), &too_long);
    assert!(matches!(&outcome, Err(message) if message.contains("writing the work item")), "{outcome:?}");
    assert!(names(work.as_path(), PENDING).is_empty(), "nothing is queued");
    remove(directory.as_path());
}

/// How long ago the controls say a record was queued.
const QUEUED_AGO: Duration = Duration::from_secs(2 * 60 * 60);

/// A stamp that fails the way a file another user owns does.
fn refused_stamp(_path: &Path) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
}

/// A stamp that sets the time and then reports a failure,
/// as a claim does that is stamped and then cannot be published.
fn stamped_then_failed(path: &Path) -> std::io::Result<()> {
    stamp_modification_time(path)?;
    return Err(std::io::Error::other("the stamp was set, then the claim failed"));
}

/// Sets the modification time of `path`.
fn set_modified(path: &Path, when: SystemTime) {
    std::fs::File::options().append(true).open(path).and_then(|file| return file.set_modified(when)).expect("stamp");
}

/// The modification time of the pending record `id`.
fn queued_at(work: &Path, id: &str) -> SystemTime {
    return std::fs::metadata(state_directory(work, PENDING).join(id)).and_then(|meta| return meta.modified()).expect("pending record");
}

/// A record queued `QUEUED_AGO` ago;
/// returns its id and the time it was stamped with.
fn queue_aged(work: &Path, oid: &str) -> (String, SystemTime) {
    let waiting: WorkItem = item(WorkKind::Push, oid);
    enqueue(work, &waiting).expect("enqueue");
    let long_ago: SystemTime = SystemTime::now() - QUEUED_AGO;
    set_modified(state_directory(work, PENDING).join(waiting.id()).as_path(), long_ago);
    return (waiting.id(), long_ago);
}

/// A claim whose stamp is refused is not taken:
/// an older release would read the stale modification time as an expired lease
/// and take a live holder's claim.
/// The record stays queued with the age it had.
#[test]
fn a_claim_whose_stamp_fails_is_not_taken_and_keeps_its_age() {
    let directory: PathBuf = fixture("work-claim-stamp-refused");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let (id, long_ago): (String, SystemTime) = queue_aged(work.as_path(), "stamp");
    assert_eq!(claim_next_stamped(work.as_path(), &clock(), &[WorkKind::Push], &refused_stamp), Ok(None));
    assert_eq!(names(work.as_path(), PENDING), vec![id.clone()]);
    assert!(names(work.as_path(), CLAIMED).is_empty() && names(work.as_path(), RECOVERING).is_empty());
    assert_eq!(queued_at(work.as_path(), id.as_str()), long_ago);
    let claimed: Claimed = claim_next_stamped(work.as_path(), &clock(), &[WorkKind::Push], &stamp_modification_time).expect("claim").expect("a stamp that works takes the claim");
    assert_eq!(claimed.id, id);
    remove(directory.as_path());
}

/// A claim that was stamped and could not be published puts the record back with the age it was queued at,
/// so a queue that
/// cannot be drained does not look new to the backlog notice.
#[test]
fn a_claim_that_cannot_be_published_keeps_the_age_of_the_queued_record() {
    let directory: PathBuf = fixture("work-claim-unpublishable");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let (id, long_ago): (String, SystemTime) = queue_aged(work.as_path(), "publish");
    let claimed_directory: PathBuf = state_directory(work.as_path(), CLAIMED);
    std::fs::remove_dir(claimed_directory.as_path()).expect("empty claimed directory");
    std::fs::write(claimed_directory.as_path(), "a file where the directory was").expect("a file in its place");
    assert_eq!(claim_next(work.as_path(), &clock()), Ok(None));
    assert_eq!(names(work.as_path(), PENDING), vec![id.clone()]);
    assert!(names(work.as_path(), RECOVERING).is_empty(), "nothing is left staged");
    assert_eq!(queued_at(work.as_path(), id.as_str()), long_ago, "the record keeps the time it was queued");
    let age: Duration = oldest_pending_age(work.as_path(), SystemTime::now()).expect("one record is pending");
    assert!(age >= QUEUED_AGO, "{age:?}");
    remove(directory.as_path());
}

/// The same holds when the stamp itself set the time before the claim failed.
#[test]
fn a_stamp_set_before_a_failure_is_undone() {
    let directory: PathBuf = fixture("work-claim-stamp-undone");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let (id, long_ago): (String, SystemTime) = queue_aged(work.as_path(), "undone");
    assert_eq!(claim_next_stamped(work.as_path(), &clock(), &[WorkKind::Push], &stamped_then_failed), Ok(None));
    assert_eq!(names(work.as_path(), PENDING), vec![id.clone()]);
    assert_eq!(queued_at(work.as_path(), id.as_str()), long_ago);
    remove(directory.as_path());
}

/// The real stamp sets a file's time to now and reports a file it cannot reach.
#[test]
fn the_stamp_sets_the_time_and_reports_a_file_it_cannot_reach() {
    let directory: PathBuf = fixture("work-stamp");
    let file: PathBuf = directory.join("record");
    std::fs::write(file.as_path(), "x").expect("record");
    set_modified(file.as_path(), SystemTime::now() - QUEUED_AGO);
    let before: SystemTime = SystemTime::now();
    stamp_modification_time(file.as_path()).expect("stamped");
    assert!(std::fs::metadata(file.as_path()).and_then(|meta| return meta.modified()).expect("time") >= before);
    assert_eq!(stamp_modification_time(directory.join("absent").as_path()).map_err(|error| return error.kind()), Err(std::io::ErrorKind::NotFound));
    remove(directory.as_path());
}

/// A renewal whose stamp fails is no renewal:
/// the claim keeps its name,
/// so the holder learns it must not rely on its lease.
#[test]
fn a_renewal_whose_stamp_fails_renews_nothing() {
    let directory: PathBuf = fixture("work-renew-stamp-refused");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "renew")).expect("enqueue");
    let held: Claimed = claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    assert_eq!(renew_claim_stamped(work.as_path(), held.name.as_str(), &clock().advanced(LEASE), &refused_stamp), None);
    assert_eq!(names(work.as_path(), CLAIMED), vec![held.name.clone()]);
    assert!(renew_claim_stamped(work.as_path(), held.name.as_str(), &clock().advanced(LEASE), &stamp_modification_time).is_some());
    remove(directory.as_path());
}

/// A queued file whose record is of other work than its name says is never claimed:
/// it would be run and then settled as done
/// under the wrong id,
/// and the work its name stands for would never run.
/// It is moved to `failed`.
#[test]
fn a_queued_file_holding_other_work_than_its_name_is_not_claimed() {
    let directory: PathBuf = fixture("work-claim-mismatch");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "seed")).expect("enqueue");
    claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    let named: WorkItem = item(WorkKind::Push, "aaa");
    let holding: WorkItem = item(WorkKind::Push, "bbb");
    std::fs::write(state_directory(work.as_path(), PENDING).join(named.id()), holding.encode()).expect("a mismatched record");
    assert_eq!(claim_next(work.as_path(), &clock()), Ok(None));
    assert_eq!(names(work.as_path(), FAILED), vec![named.id()]);
    assert!(names(work.as_path(), PENDING).is_empty() && names(work.as_path(), DONE).is_empty());
    remove(directory.as_path());
}

/// A recovered claim whose record is of other work than its name says is failed,
/// not returned to `pending` to be run.
#[test]
fn a_recovered_claim_holding_other_work_than_its_name_is_failed() {
    let directory: PathBuf = fixture("work-recover-mismatch");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "seed")).expect("enqueue");
    let named: WorkItem = item(WorkKind::Push, "aaa");
    let claim: PathBuf = state_directory(work.as_path(), CLAIMED).join(named.id());
    std::fs::write(claim.as_path(), item(WorkKind::Push, "bbb").encode()).expect("a mismatched claim by an earlier release");
    let stamped: SystemTime = SystemTime::now();
    set_modified(claim.as_path(), stamped);
    let seen: ClaimTime = ClaimTime::from_wall(stamped);
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(LEASE * 2), LEASE, MAX_TRIES), 1);
    assert_eq!(names(work.as_path(), FAILED), vec![named.id()]);
    assert_eq!(names(work.as_path(), PENDING), vec![item(WorkKind::Push, "seed").id()]);
    remove(directory.as_path());
}

/// A claim by an earlier release whose id could not make this release's names is failed by its own name,
/// in the pass that finds
/// it,
/// instead of being taken and left in `recovering` by every later pass.
#[test]
fn an_old_claim_with_an_id_that_would_not_fit_is_failed_in_one_pass() {
    let directory: PathBuf = fixture("work-recover-long-id");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "seed")).expect("enqueue");
    let long: WorkItem = item(WorkKind::Maintenance, "a".repeat(MAX_OBJECT_NAME_LENGTH * 2).as_str());
    let claim: PathBuf = state_directory(work.as_path(), CLAIMED).join(long.id());
    std::fs::write(claim.as_path(), long.encode()).expect("an old claim");
    let stamped: SystemTime = SystemTime::now();
    set_modified(claim.as_path(), stamped);
    let seen: ClaimTime = ClaimTime::from_wall(stamped);
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(LEASE * 2), LEASE, MAX_TRIES), 1);
    assert_eq!(names(work.as_path(), FAILED), vec![long.id()]);
    assert!(!state_directory(work.as_path(), RECOVERING).exists(), "no recovery was started for it");
    assert!(names(work.as_path(), CLAIMED).is_empty());
    assert_eq!(requeue_expired(work.as_path(), &seen.advanced(LEASE * 4), LEASE, MAX_TRIES), 0, "a second pass has nothing left");
    remove(directory.as_path());
}

/// Standard error restored to the descriptor it was.
struct RestoredStandardError(std::os::fd::RawFd);

impl Drop for RestoredStandardError {
    fn drop(&mut self) {
        // SAFETY:
        // `self.0` is the duplicate of descriptor 2 made for this guard,
        // and nothing else closes it.
        unsafe {
            libc::dup2(self.0, 2);
            libc::close(self.0);
        }
    }
}

/// A note about a quarantined entry never stops the consumer when standard error fails (a full log filesystem,
/// a journal
/// restarting):
/// the entry is moved and the later work is still claimed.
/// Descriptor 2 is `/dev/full` for this test,
/// which
/// nextest runs in a process of its own.
#[test]
fn a_failing_standard_error_does_not_stop_the_queue() {
    use std::os::fd::AsRawFd;
    let directory: PathBuf = fixture("work-stderr-full");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    enqueue(work.as_path(), &item(WorkKind::Push, "good")).expect("enqueue");
    let bad: String = format!("push-{}", "x".repeat(MAX_OBJECT_NAME_LENGTH + 1));
    std::fs::write(state_directory(work.as_path(), PENDING).join(bad.as_str()), "{}").expect("an entry that is not a kind and an object name");
    set_modified(state_directory(work.as_path(), PENDING).join(bad.as_str()).as_path(), SystemTime::now() - QUEUED_AGO);
    let full: std::fs::File = std::fs::File::options().write(true).open("/dev/full").expect("/dev/full");
    // SAFETY:
    // duplicating descriptor 2 has no preconditions.
    let saved: std::os::fd::RawFd = unsafe { libc::dup(2) };
    assert!(saved >= 0, "descriptor 2 is open");
    let _restored: RestoredStandardError = RestoredStandardError(saved);
    // SAFETY:
    // both descriptors are open;
    // the guard puts descriptor 2 back.
    assert!(unsafe { libc::dup2(full.as_raw_fd(), 2) } >= 0);
    let claimed: Option<Claimed> = claim_next(work.as_path(), &clock()).expect("claim");
    assert_eq!(claimed.map(|held| return held.id), Some(item(WorkKind::Push, "good").id()));
    assert_eq!(names(work.as_path(), FAILED), vec![bad]);
    remove(directory.as_path());
}

/// The notice of a push the service gave up on is made before the record moves to `failed`:
/// nothing scans `failed`,
/// so a kill between the two would abandon the branch in silence and no later command would ever learn of it.
#[test]
fn the_notice_of_a_given_up_push_is_made_before_the_record_is_failed() {
    let directory: PathBuf = fixture("work-given-up-notice-first");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let last_try: WorkItem = WorkItem { tries: MAX_TRIES - 1, ..item(WorkKind::Push, "jjj") };
    enqueue(work.as_path(), &last_try).expect("enqueue");
    claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    let failed_path: PathBuf = state_directory(work.as_path(), FAILED).join(last_try.id());
    let told: std::cell::RefCell<Vec<bool>> = std::cell::RefCell::new(Vec::new());
    struct Watching<'a> {
        failed_path: &'a Path,
        id: &'a str,
        told: &'a std::cell::RefCell<Vec<bool>>,
    }
    impl crate::work_item_recovery::GivenUpNotice for Watching<'_> {
        fn tell(&self, given_up: &WorkItem) -> bool {
            assert_eq!(given_up.id(), self.id);
            self.told.borrow_mut().push(self.failed_path.exists());
            return true;
        }
    }
    let wanted: String = last_try.id();
    let watching: Watching<'_> = Watching { failed_path: failed_path.as_path(), id: wanted.as_str(), told: &told };
    let recovered: Recovered = crate::work_item_recovery::requeue_expired_reporting_telling(work.as_path(), &clock().advanced(LEASE * 2), LEASE, MAX_TRIES, &watching);
    assert_eq!(recovered.given_up.iter().map(WorkItem::id).collect::<Vec<String>>(), vec![last_try.id()], "the pass gave up on the push");
    assert_eq!(*told.borrow(), vec![false], "the person's notice was made while the record was still this instance's, before it moved to `failed`");
    assert!(failed_path.exists(), "and the record then moved to `failed`");
    remove(directory.as_path());
}

/// What:
///  A push whose notice cannot be written is not settled:
/// it stays recoverable,
/// and a later pass that can write the notice
///  gives up on it then.
/// Why:
///   Nothing scans `failed`,
/// so a record there is the person's last word about the branch;
/// settling it while the
///  notice could not be written abandons the push in silence.
#[test]
fn a_given_up_push_whose_notice_cannot_be_written_stays_recoverable() {
    let directory: PathBuf = fixture("work-given-up-unwritten");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let last_try: WorkItem = WorkItem { tries: MAX_TRIES - 1, ..item(WorkKind::Push, "kkk") };
    enqueue(work.as_path(), &last_try).expect("enqueue");
    claim_next(work.as_path(), &clock()).expect("claim").expect("item");
    struct Refusing;
    impl crate::work_item_recovery::GivenUpNotice for Refusing {
        fn tell(&self, _given_up: &WorkItem) -> bool {
            return false;
        }
    }
    let refused: Recovered = crate::work_item_recovery::requeue_expired_reporting_telling(work.as_path(), &clock().advanced(LEASE * 2), LEASE, MAX_TRIES, &Refusing);
    assert_eq!(refused.moved, 0, "nothing is settled while the person is not told");
    assert!(refused.given_up.is_empty(), "and nothing is reported as given up");
    assert!(names(work.as_path(), FAILED).is_empty(), "the push is not abandoned in silence");
    assert_eq!(names(work.as_path(), RECOVERING).len(), 1, "it stays this instance's, to be settled once the notice is written");
    struct Telling;
    impl crate::work_item_recovery::GivenUpNotice for Telling {
        fn tell(&self, _given_up: &WorkItem) -> bool {
            return true;
        }
    }
    let settled: Recovered = crate::work_item_recovery::requeue_expired_reporting_telling(work.as_path(), &clock().advanced(LEASE * 4), LEASE, MAX_TRIES, &Telling);
    assert_eq!(settled.moved, 1, "a pass that can tell the person settles it");
    assert_eq!(names(work.as_path(), FAILED), vec![last_try.id()]);
    assert!(names(work.as_path(), RECOVERING).is_empty());
    remove(directory.as_path());
}

/// What:
///  A push whose held record stands is not queued again.
/// Why:
///   The held record is the queue's statement that this push waits for its gate,
/// so a `pending` record written beside it runs the push before the gate answered,
/// which is the mirror image of a hold over an approved push:
/// both leave two records for one commit and one of them runs too early.
/// The producer's round is what writes it, on a replay after a restart or beside a hold another command wrote.
#[test]
fn a_push_whose_held_record_stands_is_not_queued_again() {
    let directory: PathBuf = fixture("work-enqueue-held");
    let work: PathBuf = directory.join(WORK_DIRECTORY);
    let held: WorkItem = item(WorkKind::Push, "aaa");
    assert_eq!(hold(work.as_path(), &held, "4321"), Ok(()), "the push waits for its gate");
    assert_eq!(enqueue(work.as_path(), &held), Ok(Enqueued::Known), "the queue already has this push, held");
    assert_eq!(enqueue_all(work.as_path(), std::slice::from_ref(&held)), Ok(vec![Enqueued::Known]), "a round of it answers the same");
    assert_eq!(names(work.as_path(), PENDING), Vec::<String>::new(), "nothing is queued beside the held record");
    assert_eq!(names(work.as_path(), HELD), vec![held.id()], "the held record is the queue's statement of the wait");
    remove(directory.as_path());
}

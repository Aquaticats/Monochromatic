//! Controls for the claim chain: where the file system gives no `flock`, a stale lock is still removed by exactly one reclaimer,
//!  a live lock is never removed, and a dead claimer never blocks anyone.

use super::*;
use crate::service_fault::hooks;
use crate::service_index_claim::{CLAIM_INFIX, claim_key, claim_path};
use crate::service_index_marker::{LOCK_MARKER, marker_of_file, own_identity};
use crate::service_index_placement::rename_noreplace;
use crate::test_support::{fixture, remove};

/// A `flock` that the file system refuses (no lock manager, `ENOLCK`): the case the claim chain is for.
pub(super) fn no_flock(_: &std::fs::File) -> std::io::Result<()> {
    return Err(std::io::Error::from_raw_os_error(libc::ENOLCK));
}

/// A rename that does not replace, which refuses the names of claims and nothing else.
pub(super) fn rename_unless_claim(from: &Path, to: &Path) -> std::io::Result<()> {
    if to.to_string_lossy().contains(CLAIM_INFIX) {
        return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
    }
    return rename_noreplace(from, to);
}

/// A hard link that refuses the names of claims and nothing else.
pub(super) fn link_unless_claim(from: &Path, to: &Path) -> std::io::Result<()> {
    if to.to_string_lossy().contains(CLAIM_INFIX) {
        return Err(std::io::Error::from_raw_os_error(libc::EPERM));
    }
    return std::fs::hard_link(from, to);
}

/// A killed process's marker for batch `seq`: a holder that is not any running process.
fn gone_marker(seq: u64) -> String {
    let mut child = std::process::Command::new("true").spawn().expect("a short process");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reaped");
    return format!("{LOCK_MARKER}{seq} {pid} gone\n");
}

/// This process's marker, which names a holder that runs.
fn live_marker() -> String {
    return marker_text(0, &own_identity).expect("this process has an identity");
}

fn index_in(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let index: PathBuf = root.join("index");
    std::fs::write(&index, "old index").expect("index");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, gone_marker(5)).expect("stale lock");
    return (root, index, lock);
}

/// The claim at `slot` for the file now at `lock`.
fn claim_at(lock: &Path, slot: usize) -> PathBuf {
    let file = std::fs::File::open(lock).expect("the stale lock");
    let marked = marker_of_file(&file).expect("a marker");
    let key: String = claim_key(&file, &marked).expect("its identity");
    return claim_path(lock, key.as_str(), slot);
}

fn names_in(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    names.sort();
    return names;
}

/// Allowed: where the `flock` is refused, a stale lock is removed, and nothing is left beside the index.
#[test]
fn a_stale_lock_is_removed_where_flock_is_not_available() {
    let (root, _index, lock) = index_in("claim-plain");
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Removed);
    assert!(!lock.exists());
    assert_eq!(names_in(&root), vec![String::from("index")], "no claim and no staging file is left");
    remove(root.as_path());
}

/// Allowed: a landing that meets a stale lock where the `flock` is refused takes the lock in the same acquisition, at once.
#[test]
fn a_landing_takes_a_stale_lock_where_flock_is_not_available_without_waiting() {
    let (root, index, _lock) = index_in("claim-landing");
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 20;
    let began: std::time::Instant = std::time::Instant::now();
    let taken = acquire_within(&platform, &index, 9, ceiling, &|_| {}).expect("the stale lock is reclaimed and the lock is taken");
    assert!(began.elapsed() < ceiling, "the acquisition waited for its ceiling");
    drop(taken);
    assert_eq!(names_in(&root), vec![String::from("index")], "no claim and no lock is left");
    remove(root.as_path());
}

/// Refused: while one reclaimer holds the chain, a second reclaimer of the same stale lock backs off and removes nothing, so it
///  cannot remove what a landing placed after the first; the first still removes the stale lock and clears its claims.
#[test]
fn a_second_reclaimer_without_flock_backs_off_while_the_first_holds() {
    let (root, _index, lock) = index_in("claim-two-reclaimers");
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    let second: std::rc::Rc<std::cell::RefCell<Option<Reclaimed>>> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let (hook_lock, hook_second) = (lock.clone(), std::rc::Rc::clone(&second));
    hooks::set("reclaim-held", std::rc::Rc::new(move || {
        let other: Platform = Platform { flock: &no_flock, ..SYSTEM };
        *hook_second.borrow_mut() = Some(reclaim(&other, &hook_lock));
    }));
    let first: Reclaimed = reclaim(&platform, &lock);
    assert_eq!(*second.borrow(), Some(Reclaimed::Busy), "the second reclaimer backs off");
    assert_eq!(first, Reclaimed::Removed, "the first reclaimer removes the stale lock");
    assert_eq!(names_in(&root), vec![String::from("index")], "the claims of both are gone");
    remove(root.as_path());
}

/// Allowed: claims of makers that are gone (a killed reclaimer's) stand in the way of nothing; the reclaimer passes them over,
///  removes the stale lock, and clears them after.
#[test]
fn a_dead_claimers_claims_are_passed_over_and_cleared() {
    let (root, _index, lock) = index_in("claim-dead");
    std::fs::write(claim_at(&lock, 0), gone_marker(0)).expect("a dead claimer's claim");
    std::fs::write(claim_at(&lock, 1), gone_marker(0)).expect("another dead claimer's claim");
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Removed);
    assert!(!lock.exists());
    assert_eq!(names_in(&root), vec![String::from("index")], "the dead claims are cleared with the own one");
    remove(root.as_path());
}

/// Allowed: a file at a claim's name that holds no whole marker (empty, cut, not text) is no running claimer's, since a placed
///  claim is whole; it is passed over like a dead claimer's.
#[test]
fn a_claim_without_a_whole_marker_is_passed_over() {
    let (root, _index, lock) = index_in("claim-torn");
    std::fs::write(claim_at(&lock, 0), b"").expect("an empty claim");
    std::fs::write(claim_at(&lock, 1), &gone_marker(0).as_bytes()[..10]).expect("a cut claim");
    std::fs::write(claim_at(&lock, 2), [0xff_u8, 0xfe, 0x00]).expect("a claim that is not text");
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Removed);
    assert_eq!(names_in(&root), vec![String::from("index")]);
    remove(root.as_path());
}

/// Refused: a claim whose maker runs is ahead, so the reclaimer backs off, removes neither the lock nor that claim, and leaves
///  no claim of its own.
#[test]
fn a_running_claimer_ahead_makes_the_reclaimer_back_off() {
    let (root, _index, lock) = index_in("claim-live-ahead");
    let ahead: PathBuf = claim_at(&lock, 0);
    std::fs::write(&ahead, live_marker()).expect("a running claimer's claim");
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Busy);
    assert!(lock.exists(), "the stale lock stays");
    assert_eq!(std::fs::read_to_string(&ahead).expect("the claim stands"), live_marker());
    assert_eq!(names_in(&root).len(), 3, "the index, the lock, the claim ahead: {:?}", names_in(&root));
    remove(root.as_path());
}

/// Refused: a reclaimer that read the slot below as gone long ago, and finds when it looks again that another reclaimer placed a
///  running claim there, backs off with its own claim removed and the running claim untouched.
#[test]
fn a_slot_below_that_turns_live_after_the_reclaimer_looked_makes_it_back_off() {
    let (root, _index, lock) = index_in("claim-turned-live");
    let below: PathBuf = claim_at(&lock, 0);
    std::fs::write(&below, gone_marker(0)).expect("a dead claimer's claim");
    let (hook_below, done) = (below.clone(), std::rc::Rc::new(std::cell::Cell::new(false)));
    hooks::set("reclaim-claim-placed", std::rc::Rc::new(move || {
        if !done.replace(true) {
            // The dead claim was cleared by a reclaimer that finished, and a new reclaimer placed its running claim there.
            std::fs::remove_file(&hook_below).expect("the dead claim is cleared");
            std::fs::write(&hook_below, live_marker()).expect("the new reclaimer's claim");
        }
    }));
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Busy);
    assert!(lock.exists(), "the stale lock stays");
    assert_eq!(std::fs::read_to_string(&below).expect("the running claim stands"), live_marker());
    assert_eq!(names_in(&root).len(), 3, "the index, the lock, the running claim: {:?}", names_in(&root));
    remove(root.as_path());
}

/// Refused: a slot below that is missing when the reclaimer looks again (its claimer finished and cleared it) is no proof that
///  nobody holds, because a new reclaimer may place there next and hold at once; the reclaimer backs off and asks again.
#[test]
fn a_slot_below_that_vanishes_after_the_reclaimer_looked_makes_it_back_off() {
    let (root, _index, lock) = index_in("claim-vanished");
    let below: PathBuf = claim_at(&lock, 0);
    std::fs::write(&below, gone_marker(0)).expect("a dead claimer's claim");
    let (hook_below, done) = (below.clone(), std::rc::Rc::new(std::cell::Cell::new(false)));
    hooks::set("reclaim-claim-placed", std::rc::Rc::new(move || {
        if !done.replace(true) {
            std::fs::remove_file(&hook_below).expect("the dead claim is cleared");
        }
    }));
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Busy);
    assert!(lock.exists(), "the stale lock stays");
    assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "the own claim is removed");
    // The next attempt, with nothing in the way, holds at the lowest slot and removes the lock.
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Removed);
    remove(root.as_path());
}

/// Refused: a reclaimer that held the chain, and finds at its check that the name now refers to another file (the stale lock was
///  removed by someone else and a live landing placed its own), removes nothing, and clears its claims.
#[test]
fn a_replaced_name_without_flock_removes_nothing() {
    let (root, index, lock) = index_in("claim-replaced");
    let live_cell: std::rc::Rc<std::cell::RefCell<Option<IndexLock>>> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let (hook_lock, hook_index, hook_live) = (lock.clone(), index.clone(), std::rc::Rc::clone(&live_cell));
    hooks::set("reclaim-held", std::rc::Rc::new(move || {
        // A reclaimer on a file system that gives it the hold removed the stale lock; its landing places a live lock.
        std::fs::remove_file(&hook_lock).expect("the other reclaimer removes the stale lock");
        *hook_live.borrow_mut() = Some(try_acquire(&hook_index, 8).expect("the live landing places its lock"));
    }));
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Changed);
    assert_eq!(std::fs::read_to_string(&lock).expect("the live lock stands"), marker_text(8, &own_identity).expect("an identity"));
    let live = live_cell.borrow_mut().take().expect("the live lock");
    drop(live);
    assert_eq!(names_in(&root), vec![String::from("index")], "the live lock was released and no claim is left");
    remove(root.as_path());
}

/// Refused: a stale lock that cannot be unlinked stays, the reclaimer removes only its own claim, and a dead claimer's claim
///  below stays where it was.
#[test]
fn a_removal_that_fails_clears_only_the_own_claim() {
    let (root, _index, lock) = index_in("claim-stuck");
    let dead: PathBuf = claim_at(&lock, 0);
    std::fs::write(&dead, gone_marker(0)).expect("a dead claimer's claim");
    let refusing = |_: &Path| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(libc::EPERM)); };
    let platform: Platform = Platform { flock: &no_flock, unlink: &refusing, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Stuck);
    assert!(lock.exists());
    assert!(dead.exists(), "a claim of another maker stays while the stale lock stands");
    assert_eq!(names_in(&root).len(), 3, "the index, the lock, the dead claim: {:?}", names_in(&root));
    remove(root.as_path());
}

/// Refused: where neither the `flock` nor a claim can be had, the stale lock stays and the caller is told so (`Unguarded`).
#[test]
fn neither_flock_nor_claim_leaves_the_lock_and_says_so() {
    let (root, _index, lock) = index_in("claim-none");
    let platform: Platform = Platform { flock: &no_flock, rename: &rename_unless_claim, link: &link_unless_claim, ..SYSTEM };
    let outcome: Reclaimed = reclaim(&platform, &lock);
    assert!(matches!(outcome, Reclaimed::Unguarded(_)), "{outcome:?}");
    assert!(lock.exists());
    assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "no claim and no staging file is left");
    remove(root.as_path());
}

/// Refused: an error of the `flock` that says "try again" (not enough memory for the lock record) is no sign that the file
///  system has none, so the reclaimer backs off and does not take the claim path beside a reclaimer that holds the `flock`.
#[test]
fn a_flock_that_fails_for_want_of_memory_is_a_back_off_not_a_fallback() {
    let (root, _index, lock) = index_in("claim-enomem");
    let transient = |_: &std::fs::File| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(libc::ENOMEM)); };
    let platform: Platform = Platform { flock: &transient, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Busy);
    assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "no claim was placed");
    remove(root.as_path());
}

/// What the refusals that mean "this file system gives no `flock`" are: what the file systems that define the operation return.
const NO_FLOCK_ERRNOS: [i32; 5] = [libc::ENOLCK, libc::EOPNOTSUPP, libc::ENOSYS, libc::EINVAL, libc::EBADF];

/// Allowed: each refusal that means "no `flock` here" takes the claim path and removes the stale lock.
#[test]
fn every_refusal_that_means_no_flock_takes_the_claim_path() {
    for errno in NO_FLOCK_ERRNOS {
        let (root, _index, lock) = index_in("claim-errnos");
        let refusing = move |_: &std::fs::File| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(errno)); };
        let platform: Platform = Platform { flock: &refusing, ..SYSTEM };
        assert_eq!(reclaim(&platform, &lock), Reclaimed::Removed, "errno {errno}");
        remove(root.as_path());
    }
}

/// Refused: a `flock` that fails in a way that does not say the file system has none (an I/O error from a lock manager) may be
///  failing for one call while another reclaimer holds the `flock`; the claim path beside it would not keep the two apart, so the
///  stale lock stays and the caller is told (`Unguarded`).
#[test]
fn a_flock_that_fails_in_an_unrecognised_way_leaves_the_lock_and_says_so() {
    let (root, _index, lock) = index_in("claim-eio");
    let failing = |_: &std::fs::File| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(libc::EIO)); };
    let platform: Platform = Platform { flock: &failing, ..SYSTEM };
    let outcome: Reclaimed = reclaim(&platform, &lock);
    assert!(matches!(outcome, Reclaimed::Unguarded(_)), "{outcome:?}");
    assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "no claim was placed");
    remove(root.as_path());
}

/// Allowed: a placement whose answer is an error although it took effect (a network file system can lose the answer of a link
///  or rename) is this attempt's own claim, found by its text; it is held and removed, not left as a running claimer's claim.
#[test]
fn a_placement_whose_answer_was_lost_is_still_this_attempts_claim() {
    let (root, _index, lock) = index_in("claim-lost-answer");
    let lost = |from: &Path, to: &Path| -> std::io::Result<()> {
        rename_noreplace(from, to)?;
        return Err(std::io::Error::from_raw_os_error(libc::EIO));
    };
    let platform: Platform = Platform { flock: &no_flock, rename: &lost, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Removed);
    assert_eq!(names_in(&root), vec![String::from("index")], "no claim is left standing as a running claimer's");
    remove(root.as_path());
}

/// Allowed: every reclaimer derives the same claim name from the marker of the stale lock, whatever the device and inode it
///  reached the file by (two mounts of one remote tree give one file different device numbers).
#[test]
fn the_claim_name_comes_from_the_marker_and_not_from_the_files_device_and_inode() {
    let root: PathBuf = fixture("claim-key");
    let text: String = gone_marker(5);
    let (first, second) = (root.join("first"), root.join("second"));
    std::fs::write(&first, &text).expect("a lock");
    std::fs::write(&second, &text).expect("the same lock seen by another mount");
    let key_of = |path: &Path| -> String {
        let file = std::fs::File::open(path).expect("the lock");
        let marked = marker_of_file(&file).expect("a marker");
        return claim_key(&file, &marked).expect("a key");
    };
    assert_eq!(key_of(&first), key_of(&second));
    remove(root.as_path());
}

/// Allowed: a claim is readable by every account whatever the umask of its maker, because a reclaimer that cannot read a claim
///  takes its maker to run.
#[test]
fn a_claim_is_readable_by_other_accounts_whatever_the_makers_umask() {
    let (root, _index, lock) = index_in("claim-mode");
    let own: PathBuf = claim_at(&lock, 0);
    let seen = std::rc::Rc::new(std::cell::Cell::new(0_u32));
    let (hook_seen, hook_own) = (std::rc::Rc::clone(&seen), own.clone());
    hooks::set("reclaim-claim-placed", std::rc::Rc::new(move || {
        hook_seen.set(std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(&hook_own).expect("the claim").permissions()));
    }));
    // SAFETY: `umask` takes a mode only; each control runs in its own process under nextest.
    let before: libc::mode_t = unsafe { libc::umask(0o077) };
    let platform: Platform = Platform { flock: &no_flock, ..SYSTEM };
    let outcome: Reclaimed = reclaim(&platform, &lock);
    // SAFETY: as above.
    unsafe { libc::umask(before) };
    assert_eq!(outcome, Reclaimed::Removed);
    let readable: u32 = 0o444;
    assert_eq!(seen.get() & readable, readable, "the claim's mode was {:o}", seen.get());
    remove(root.as_path());
}

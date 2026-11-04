//! Controls for the claim chain, which every reclaim of a stale lock goes through on every file system: exactly one reclaimer
//!  removes a stale lock, a live lock is never removed, and a dead claimer never blocks anyone.

use super::*;
use crate::service_fault::hooks;
use crate::service_index_claim::{CLAIM_INFIX, claim_key, claim_path};
use crate::service_index_marker::{LOCK_MARKER, Marked, marker_of_file, own_identity};
use crate::service_index_placement::rename_noreplace;
use crate::test_support::{fixture, remove};

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
pub(super) fn gone_marker(seq: u64) -> String {
    let mut child = std::process::Command::new("true").spawn().expect("a short process");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reaped");
    return format!("{LOCK_MARKER}{seq} {pid} gone\n");
}

/// This process's marker, which names a holder that runs.
pub(super) fn live_marker() -> String {
    return marker_text(0, &own_identity).expect("this process has an identity");
}

pub(super) fn index_in(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let index: PathBuf = root.join("index");
    std::fs::write(&index, "old index").expect("index");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, gone_marker(5)).expect("stale lock");
    return (root, index, lock);
}

/// The claim at `slot` for the file now at `lock`.
pub(super) fn claim_at(lock: &Path, slot: usize) -> PathBuf {
    let file = std::fs::File::open(lock).expect("the stale lock");
    let marked = marker_of_file(&file).expect("a marker");
    let key: String = claim_key(&file, &marked).expect("its identity");
    return claim_path(lock, key.as_str(), slot);
}

pub(super) fn names_in(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    names.sort();
    return names;
}

/// Allowed: a stale lock is removed under a claim, whatever the file system gives, and nothing is left beside the index. The
///  claim stands while the reclaimer holds, which shows the `flock` is not what keeps reclaimers apart.
#[test]
fn a_stale_lock_is_removed_under_a_claim_and_leaves_nothing() {
    let (root, _index, lock) = index_in("claim-plain");
    let claim: PathBuf = claim_at(&lock, 0);
    let standing = std::rc::Rc::new(std::cell::Cell::new(false));
    let (hook_claim, hook_standing) = (claim, std::rc::Rc::clone(&standing));
    hooks::set("reclaim-held", std::rc::Rc::new(move || { hook_standing.set(hook_claim.exists()); }));
    let platform: Platform = SYSTEM;
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Removed);
    assert!(standing.get(), "the reclaimer held a claim while it removed the lock");
    assert!(!lock.exists());
    assert_eq!(names_in(&root), vec![String::from("index")], "no claim and no staging file is left");
    remove(root.as_path());
}

/// Allowed: a `flock` held on the stale lock by another descriptor stops nothing, because no reclaimer takes the `flock`: the
///  claim chain is the one mechanism, so a reclaimer that took a claim and one that took a `flock` can never be mixed.
#[test]
fn a_flock_on_the_stale_lock_does_not_stop_the_reclaim() {
    let (root, _index, lock) = index_in("claim-flock-ignored");
    let holding = std::fs::File::open(&lock).expect("another descriptor");
    // SAFETY: `flock` takes an open descriptor, which `holding` keeps open for the call.
    let taken: i32 = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&holding), libc::LOCK_EX | libc::LOCK_NB) };
    assert_eq!(taken, 0, "the control plays a holder of the flock");
    assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Removed);
    assert!(!lock.exists());
    drop(holding);
    remove(root.as_path());
}

/// Allowed: a landing that meets a stale lock takes the lock in the same acquisition, at once.
#[test]
fn a_landing_takes_a_stale_lock_under_a_claim_without_waiting() {
    let (root, index, _lock) = index_in("claim-landing");
    let platform: Platform = SYSTEM;
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
fn a_second_reclaimer_backs_off_while_the_first_holds() {
    let (root, _index, lock) = index_in("claim-two-reclaimers");
    let platform: Platform = SYSTEM;
    let second: std::rc::Rc<std::cell::RefCell<Option<Reclaimed>>> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let (hook_lock, hook_second) = (lock.clone(), std::rc::Rc::clone(&second));
    hooks::set("reclaim-held", std::rc::Rc::new(move || {
        let other: Platform = SYSTEM;
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
    let platform: Platform = SYSTEM;
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
    let platform: Platform = SYSTEM;
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
    let platform: Platform = SYSTEM;
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
    let platform: Platform = SYSTEM;
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
    let platform: Platform = SYSTEM;
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
fn a_replaced_name_removes_nothing() {
    let (root, index, lock) = index_in("claim-replaced");
    let live_cell: std::rc::Rc<std::cell::RefCell<Option<IndexLock>>> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let (hook_lock, hook_index, hook_live) = (lock.clone(), index.clone(), std::rc::Rc::clone(&live_cell));
    hooks::set("reclaim-held", std::rc::Rc::new(move || {
        // A reclaimer on a file system that gives it the hold removed the stale lock; its landing places a live lock.
        std::fs::remove_file(&hook_lock).expect("the other reclaimer removes the stale lock");
        *hook_live.borrow_mut() = Some(try_acquire(&hook_index, 8).expect("the live landing places its lock"));
    }));
    let platform: Platform = SYSTEM;
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
    let platform: Platform = Platform { unlink: &refusing, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Stuck);
    assert!(lock.exists());
    assert!(dead.exists(), "a claim of another maker stays while the stale lock stands");
    assert_eq!(names_in(&root).len(), 3, "the index, the lock, the dead claim: {:?}", names_in(&root));
    remove(root.as_path());
}

/// Refused: where no claim can be placed, the stale lock stays and the caller is told so (`Unguarded`).
#[test]
fn no_claim_can_be_placed_leaves_the_lock_and_says_so() {
    let (root, _index, lock) = index_in("claim-none");
    let platform: Platform = Platform { rename: &rename_unless_claim, link: &link_unless_claim, ..SYSTEM };
    let outcome: Reclaimed = reclaim(&platform, &lock);
    assert!(matches!(outcome, Reclaimed::Unguarded(_)), "{outcome:?}");
    assert!(lock.exists());
    assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "no claim and no staging file is left");
    remove(root.as_path());
}

/// Refused: a claim whose placement is interrupted or finds no memory is a reclaimer that is not to go on now: it backs off, leaves
///  the stale lock, and asks again later, instead of telling the person to remove the lock by hand.
#[test]
fn a_claim_placement_that_is_interrupted_or_short_of_memory_is_a_back_off() {
    for errno in TRANSIENT_ERRNOS {
        let (root, _index, lock) = index_in("claim-transient");
        let failing = move |_: &Path, _: &Path| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(errno)); };
        let platform: Platform = Platform { rename: &failing, ..SYSTEM };
        assert_eq!(reclaim(&platform, &lock), Reclaimed::Busy, "errno {errno}");
        assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "no claim was placed");
        remove(root.as_path());
    }
}

/// The errors of a placement that say "not now" and not "never".
const TRANSIENT_ERRNOS: [i32; 2] = [libc::EINTR, libc::ENOMEM];

/// Allowed: a placement whose answer is an error although it took effect (a network file system can lose the answer of a link
///  or rename) is this attempt's own claim, found by its text; it is held and removed, not left as a running claimer's claim.
#[test]
fn a_placement_whose_answer_was_lost_is_still_this_attempts_claim() {
    let (root, _index, lock) = index_in("claim-lost-answer");
    let lost = |from: &Path, to: &Path| -> std::io::Result<()> {
        rename_noreplace(from, to)?;
        return Err(std::io::Error::from_raw_os_error(libc::EIO));
    };
    let platform: Platform = Platform { rename: &lost, ..SYSTEM };
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
    let platform: Platform = SYSTEM;
    let outcome: Reclaimed = reclaim(&platform, &lock);
    // SAFETY: as above.
    unsafe { libc::umask(before) };
    assert_eq!(outcome, Reclaimed::Removed);
    let readable: u32 = 0o444;
    assert_eq!(seen.get() & readable, readable, "the claim's mode was {:o}", seen.get());
    remove(root.as_path());
}

/// The rename of the system that counts the times a claim's name was found taken, and tells `first_refusal` the first time.
fn counting_rename<'a>(refused: &'a std::cell::Cell<usize>, first_refusal: &'a std::sync::mpsc::Sender<()>) -> impl Fn(&Path, &Path) -> std::io::Result<()> + 'a {
    return move |from: &Path, to: &Path| -> std::io::Result<()> {
        let answer = rename_noreplace(from, to);
        if to.to_string_lossy().contains(CLAIM_INFIX) && answer.as_ref().is_err_and(|error| { return error.kind() == std::io::ErrorKind::AlreadyExists; }) {
            refused.set(refused.get() + 1);
            if refused.get() == 1 {
                let _told = first_refusal.send(());
            }
        }
        return answer;
    };
}

/// The thread that plays another reclaimer: it places a running claim, says so, and keeps it until told to let go (or until
///  `give_up`, so a failed control cannot hang).
fn spawn_other_claimer(claim: PathBuf, placed: std::sync::mpsc::Sender<()>, let_go: std::sync::mpsc::Receiver<()>, give_up: std::time::Duration) -> std::thread::JoinHandle<()> {
    return std::thread::spawn(move || {
        std::fs::write(&claim, live_marker()).expect("the other reclaimer's claim");
        placed.send(()).expect("the control waits for the claim");
        let _told = let_go.recv_timeout(give_up);
        let _removed = std::fs::remove_file(&claim);
    });
}

/// A landing that meets a stale lock another reclaimer holds a claim on neither removes it nor fails at once: it looks again at
///  the poll pace, which the count of refusals shows, and ends at its ceiling; the stale lock stays for the reclaimer that holds it.
#[test]
fn a_landing_waits_for_the_reclaimer_that_holds_a_claim() {
    let (root, index, lock) = index_in("claim-landing-busy");
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 20;
    let (placed_tx, placed_rx) = std::sync::mpsc::channel::<()>();
    let (let_go_tx, let_go_rx) = std::sync::mpsc::channel::<()>();
    let other = spawn_other_claimer(claim_at(&lock, 0), placed_tx, let_go_rx, ceiling * 4);
    placed_rx.recv().expect("the claim is placed");
    let refused = std::cell::Cell::new(0_usize);
    let (first_tx, _first_rx) = std::sync::mpsc::channel::<()>();
    let counting = counting_rename(&refused, &first_tx);
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    let began: std::time::Instant = std::time::Instant::now();
    let failure = acquire_within(&platform, &index, 9, ceiling, &|_| {}).expect_err("the lock is not taken while the other reclaimer holds a claim");
    assert_eq!(failure.code, FailureCode::IndexInUse, "the wait ends at its ceiling, as for any held lock");
    assert!(began.elapsed() >= ceiling, "the wait ended before its ceiling");
    assert!(refused.get() > 1, "the landing looked again while the claim stood: {} refusals", refused.get());
    assert!(lock.exists(), "the stale lock stays for the reclaimer that holds the claim");
    let_go_tx.send(()).expect("the other reclaimer is told to let go");
    other.join().expect("the other reclaimer ends");
    remove(root.as_path());
}

/// A landing that meets a stale lock another reclaimer holds a claim on waits in the same acquisition and takes the lock once
///  the claim is gone without the lock having been unlinked: the end of a claim leaves no event on the lock's name, so the wait
///  looks again at the poll pace. The other reclaimer lets go only after the landing was refused, so the control meets the busy case.
#[test]
fn a_landing_takes_the_stale_lock_when_the_other_reclaimers_claim_is_dropped() {
    let (root, index, lock) = index_in("claim-landing-wakes");
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 20;
    let (placed_tx, placed_rx) = std::sync::mpsc::channel::<()>();
    let (first_tx, first_rx) = std::sync::mpsc::channel::<()>();
    let other = spawn_other_claimer(claim_at(&lock, 0), placed_tx, first_rx, ceiling * 4);
    placed_rx.recv().expect("the claim is placed");
    let refused = std::cell::Cell::new(0_usize);
    let counting = counting_rename(&refused, &first_tx);
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    let holders = std::cell::RefCell::new(Vec::<String>::new());
    let began: std::time::Instant = std::time::Instant::now();
    let taken = acquire_within(&platform, &index, 9, ceiling, &|announce| {
        if let Announce::Holder(who) = announce {
            holders.borrow_mut().push(who);
        }
    })
    .expect("the stale lock is reclaimed in the same acquisition once the claim is dropped");
    assert!(refused.get() >= 1, "the acquisition met the other reclaimer's claim");
    assert!(began.elapsed() < ceiling, "the acquisition took the whole ceiling");
    assert!(holders.borrow().is_empty(), "a claimed stale lock is no dead holder to give up on: {:?}", holders.borrow());
    drop(taken);
    other.join().expect("the other reclaimer ends");
    remove(root.as_path());
}

/// The longest file name Linux gives a directory entry.
const NAME_LIMIT: usize = 255;

/// Allowed: the claim file of a real identity (start tick, boot, namespace and machine) beside a lock with a long name stays within
///  the limit of a file name, so a claim can be placed beside the lock of an alternate index with a long name.
#[test]
fn a_claim_name_for_a_real_identity_fits_beside_a_long_lock_name() {
    let identity: String = own_identity().expect("an identity");
    let marked = Marked { seq: u64::MAX, holder: Some((i64::from(i32::MAX), identity)) };
    let root: PathBuf = fixture("claim-long-name");
    let file_name: String = "i".repeat(40) + ".lock";
    let lock: PathBuf = root.join(file_name);
    std::fs::write(&lock, "x").expect("a file");
    let key: String = claim_key(&std::fs::File::open(&lock).expect("open"), &marked).expect("a key");
    let claim: PathBuf = claim_path(&lock, key.as_str(), usize::MAX);
    let length: usize = claim.file_name().expect("a name").len();
    assert!(length <= NAME_LIMIT, "the claim name is {length} bytes: {claim:?}");
    remove(root.as_path());
}

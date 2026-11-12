//! Controls for the claim chain,
//! which every reclaim of a stale lock goes through on every file system:
//! exactly one reclaimer
//!  removes a stale lock,
//! a live lock is never removed,
//! and a dead claimer never blocks anyone.

use super::*;
use crate::service_fault::hooks;
use crate::service_index_claim::{CLAIM_INFIX, claim_key, claim_path};
use crate::service_index_marker::{LOCK_MARKER, Marked, marker_of_file, own_identity};
use crate::service_index_placement::rename_noreplace;
use crate::test_support::{fixture, remove};

/// A rename that does not replace,
/// which refuses the names of claims and nothing else.
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

/// A killed process's marker for batch `seq`:
/// a holder that is not any running process.
pub(super) fn gone_marker(seq: u64) -> String {
    let mut child = std::process::Command::new("true").spawn().expect("a short process");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reaped");
    // The format of this release: the claim chain reclaims no other.
    return format!("{LOCK_MARKER}{}{seq} {pid} gone\n", crate::service_index_marker::MARKER_VERSION);
}

/// This process's marker,
/// which names a holder that runs.
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
    let key: String = claim_key(&marked);
    return claim_path(lock, key.as_str(), slot);
}

pub(super) fn names_in(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    names.sort();
    return names;
}

/// Allowed:
/// a stale lock is removed under a claim,
/// whatever the file system gives,
/// and nothing is left beside the index.
/// The
///  claim stands while the reclaimer holds,
/// which shows the `flock` is not what keeps reclaimers apart.
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

/// Allowed:
/// a `flock` held on the stale lock by another descriptor stops nothing,
/// because no reclaimer takes the `flock`:
/// the
///  claim chain is the one mechanism,
/// so a reclaimer that took a claim and one that took a `flock` can never be mixed.
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

/// Allowed:
/// a landing that meets a stale lock takes the lock in the same acquisition,
/// at once.
#[test]
fn a_landing_takes_a_stale_lock_under_a_claim_without_waiting() {
    let (root, index, _lock) = index_in("claim-landing");
    let platform: Platform = SYSTEM;
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 20;
    // What the person would have been told while it waited: nothing, because the lock is taken at once.
    // A wall-clock budget cannot carry this claim: one btrfs transaction commit from another session held every test of a
    //  wave for tens of seconds on 2026-11-08, and a test's own fixture writes then cost more than any such budget.
    let announced: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
    let taken = acquire_within(&platform, &index, 9, ceiling, &|announce| {
        if let Announce::Waiting(text) = announce {
            announced.borrow_mut().push(text);
        }
    })
    .expect("the stale lock is reclaimed and the lock is taken");
    assert!(announced.borrow().is_empty(), "the stale lock was taken without waiting for a holder: {:?}", announced.borrow());
    drop(taken);
    assert_eq!(names_in(&root), vec![String::from("index")], "no claim and no lock is left");
    remove(root.as_path());
}

/// Refused:
/// while one reclaimer holds the chain,
/// a second reclaimer of the same stale lock backs off and removes nothing,
/// so it
///  cannot remove what a landing placed after the first;
/// the first still removes the stale lock and clears its claims.
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

/// Allowed:
/// claims of makers that are gone (a killed reclaimer's) stand in the way of nothing;
/// the reclaimer passes them over,
///  removes the stale lock,
/// and clears them after.
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

/// Allowed:
/// a file at a claim's name that holds no whole marker (empty,
/// cut,
/// not text) is no running claimer's,
/// since a placed
///  claim is whole;
/// it is passed over like a dead claimer's.
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

/// Refused:
/// a claim whose maker runs is ahead,
/// so the reclaimer backs off,
/// removes neither the lock nor that claim,
/// and leaves
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

/// Refused:
/// a reclaimer that read the slot below as gone long ago,
/// and finds when it looks again that another reclaimer placed a
///  running claim there,
/// backs off with its own claim removed and the running claim untouched.
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

/// Refused:
/// a slot below that is missing when the reclaimer looks again (its claimer finished and cleared it) is no proof that
///  nobody holds,
/// because a new reclaimer may place there next and hold at once;
/// the reclaimer backs off and asks again.
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

/// Refused:
/// a reclaimer that held the chain,
/// and finds at its check that the name now refers to another file (the stale lock was
///  removed by someone else and a live landing placed its own),
/// removes nothing,
/// and clears its claims.
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

/// Refused:
/// a stale lock that cannot be unlinked stays,
/// the reclaimer removes only its own claim,
/// and a dead claimer's claim
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

/// Refused:
/// where no claim can be placed,
/// the stale lock stays and the caller is told so (`Unguarded`).
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

/// Refused:
/// a claim whose placement is interrupted or finds no memory is a reclaimer that is not to go on now:
/// it backs off,
/// leaves
///  the stale lock,
/// and asks again later,
/// instead of telling the person to remove the lock by hand.
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

/// Allowed:
/// a placement whose answer is an error although it took effect (a network file system can lose the answer of a link
///  or rename) is this attempt's own claim,
/// found by its text;
/// it is held and removed,
/// not left as a running claimer's claim.
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

/// Allowed:
/// every reclaimer derives the same claim name from the marker of the stale lock,
/// whatever the device and inode it
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
        return claim_key(&marked);
    };
    assert_eq!(key_of(&first), key_of(&second));
    remove(root.as_path());
}

/// Allowed:
/// a claim is readable by every account whatever the umask of its maker,
/// because a reclaimer that cannot read a claim
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

/// The rename of the system that counts the times a claim's name was found taken,
/// and tells `first_refusal` the first time.
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

/// How long the helper that plays another reclaimer holds its claim before it gives up,
/// so a failed control cannot hang.
/// Far beyond any file system stall this suite has met:
/// a btrfs transaction commit from another session held every test of a wave for tens of seconds on 2026-11-08,
/// and a helper that gave up in a few seconds released its claim before the control under test could meet it.
const CLAIMER_GIVE_UP: std::time::Duration = std::time::Duration::from_secs(120);

/// The thread that plays another reclaimer:
/// it places a running claim,
/// says so,
/// and keeps it until told to let go (or until
///  `give_up`,
/// so a failed control cannot hang).
fn spawn_other_claimer(claim: PathBuf, placed: std::sync::mpsc::Sender<()>, let_go: std::sync::mpsc::Receiver<()>, give_up: std::time::Duration) -> std::thread::JoinHandle<()> {
    return std::thread::spawn(move || {
        std::fs::write(&claim, live_marker()).expect("the other reclaimer's claim");
        placed.send(()).expect("the control waits for the claim");
        let _told = let_go.recv_timeout(give_up);
        let _removed = std::fs::remove_file(&claim);
    });
}

/// What:
///  A landing that meets a stale lock another reclaimer holds a claim on neither removes it nor fails at once:
///  it ends at its ceiling,
///  and the stale lock stays for the reclaimer that holds it.
/// Why:
///   The count of refusals this control used to require (more than one) was a clock in disguise:
/// how many attempts fit in the ceiling depends on how long each attempt takes,
/// and on 2026-11-09 this host stalled one attempt for ten seconds (the same two controls,
/// alone,
/// took 0.3 to 19.6 s over
/// six runs),
/// so the count fell to one and the control failed while the landing behaved as designed.
/// That the wait looks again at the poll pace is what `a_landing_takes_the_stale_lock_when_the_other_reclaimers_claim_is_dropped`
/// controls,
/// through `WaitNotes::polled`,
/// which no clock decides.
#[test]
fn a_landing_waits_for_the_reclaimer_that_holds_a_claim() {
    let (root, index, lock) = index_in("claim-landing-busy");
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 20;
    let (placed_tx, placed_rx) = std::sync::mpsc::channel::<()>();
    let (let_go_tx, let_go_rx) = std::sync::mpsc::channel::<()>();
    let other = spawn_other_claimer(claim_at(&lock, 0), placed_tx, let_go_rx, CLAIMER_GIVE_UP);
    placed_rx.recv().expect("the claim is placed");
    let refused = std::cell::Cell::new(0_usize);
    let (first_tx, _first_rx) = std::sync::mpsc::channel::<()>();
    let counting = counting_rename(&refused, &first_tx);
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    let began: std::time::Instant = std::time::Instant::now();
    let failure = acquire_within(&platform, &index, 9, ceiling, &|_| {}).expect_err("the lock is not taken while the other reclaimer holds a claim");
    assert_eq!(failure.code, FailureCode::IndexInUse, "the wait ends at its ceiling, as for any held lock");
    assert!(began.elapsed() >= ceiling, "the wait ended before its ceiling");
    assert!(refused.get() >= 1, "the landing met the reclaimer's claim: {} refusals", refused.get());
    assert!(lock.exists(), "the stale lock stays for the reclaimer that holds the claim");
    let_go_tx.send(()).expect("the other reclaimer is told to let go");
    other.join().expect("the other reclaimer ends");
    remove(root.as_path());
}

/// What:
///  A landing that meets a stale lock another reclaimer holds a claim on waits in the same acquisition and takes the lock once
///  the claim is gone without the lock having been unlinked:
///  the end of a claim leaves no event on the lock's name,
///  so the wait looks again at the poll pace.
/// The other reclaimer lets go only after the landing was refused,
/// so the control meets the busy case.
/// Why:
///   The poll pace was asserted through a one-second ceiling,
/// which a stalled attempt eats:
/// on 2026-11-09 two of these controls failed at 11.268 s and 11.271 s in a run of 1,269 tests while the same two,
/// alone,
/// took 0.3 to 19.6 s over six runs on this host.
/// The wait now says in `WaitNotes::polled` that it looked again at the poll pace while a claim stood,
/// and the ceiling is the product's own (`LANDING_LOCK_CEILING`),
/// so a stall no longer decides the verdict and a wait that blocked for the notice interval is still caught.
#[test]
fn a_landing_takes_the_stale_lock_when_the_other_reclaimers_claim_is_dropped() {
    let (root, index, lock) = index_in("claim-landing-wakes");
    let ceiling: std::time::Duration = crate::wait_ceilings::LANDING_LOCK_CEILING;
    let (placed_tx, placed_rx) = std::sync::mpsc::channel::<()>();
    let (first_tx, first_rx) = std::sync::mpsc::channel::<()>();
    let other = spawn_other_claimer(claim_at(&lock, 0), placed_tx, first_rx, CLAIMER_GIVE_UP);
    placed_rx.recv().expect("the claim is placed");
    let refused = std::cell::Cell::new(0_usize);
    let counting = counting_rename(&refused, &first_tx);
    let platform: Platform = Platform { rename: &counting, ..SYSTEM };
    let holders = std::cell::RefCell::new(Vec::<String>::new());
    let mut notes: crate::service_index_found::WaitNotes = crate::service_index_found::WaitNotes::default();
    let announce = |note: Announce| {
        if let Announce::Holder(who) = note {
            holders.borrow_mut().push(who);
        }
    };
    crate::service_index_attempts::forget_waits_chosen();
    let taken = acquire_placing(&platform, &index, 9, ceiling, &announce, &mut notes, None, &mut crate::service_index_lock::TheLockWaitsMemory::default())
    .expect("the stale lock is reclaimed in the same acquisition once the claim is dropped");
    assert!(refused.get() >= 1, "the acquisition met the other reclaimer's claim");
    assert!(notes.polled, "the wait looked again at the poll pace while another reclaimer held a claim, as nothing wakes it when the claim goes");
    // The span the wait asked for, which the clock cannot say: a thread the scheduler holds makes a wait that asked for the poll
    //  pace look slow and one held just after a sleep makes a wait that asked for the notice look quick.
    let chosen: Vec<Duration> = crate::service_index_attempts::waits_chosen_by_this_thread();
    assert!(!chosen.is_empty(), "the acquisition waited for the claim instead of taking the lock at once");
    assert!(
        chosen[0] <= crate::service_index_watch::POLL_PAUSE,
        "the wait that followed the refusal meeting the claim asked for {:?}: a claim's end leaves no event on the lock's name, so the wait looks again at the poll pace ({} ms), not at the notice interval ({:?})",
        chosen[0],
        crate::service_index_watch::POLL_PAUSE.as_millis(),
        notice_every()
    );
    assert!(holders.borrow().is_empty(), "a claimed stale lock is no dead holder to give up on: {:?}", holders.borrow());
    drop(taken);
    other.join().expect("the other reclaimer ends");
    remove(root.as_path());
}

/// The wait that follows a refusal which met another reclaimer's claim is the poll pace,
/// and only a watched wait is the notice interval.
#[test]
fn a_wait_while_a_claim_stands_asks_for_the_poll_pace() {
    let notice: Duration = notice_every();
    let ceiling: Duration = crate::wait_ceilings::LANDING_LOCK_CEILING;
    assert!(notice > crate::service_index_watch::POLL_PAUSE, "the notice interval and the poll pace are told apart: {notice:?}");
    // No watch can end a claim, so the wait that met one looks again at the poll pace.
    assert_eq!(crate::service_index_attempts::until_next_wake(notice, ceiling, false), crate::service_index_watch::POLL_PAUSE);
    // A watched wait is the notice itself, which is what makes the cap above the figure that decides.
    assert_eq!(crate::service_index_attempts::until_next_wake(notice, ceiling, true), notice);
    // The ceiling bounds both.
    let short: Duration = crate::service_index_watch::POLL_PAUSE / 2;
    assert_eq!(crate::service_index_attempts::until_next_wake(notice, short, false), short);
    assert_eq!(crate::service_index_attempts::until_next_wake(notice, short, true), short);
}

/// The longest file name Linux gives a directory entry.
const NAME_LIMIT: usize = 255;

/// Allowed:
/// the claim file of a real identity (start tick,
/// boot,
/// namespace and machine) beside a lock with a long name stays within
///  the limit of a file name,
/// so a claim can be placed beside the lock of an alternate index with a long name.
#[test]
fn a_claim_name_for_a_real_identity_fits_beside_a_long_lock_name() {
    let identity: String = own_identity().expect("an identity");
    let marked = Marked { seq: u64::MAX, holder: Some((i64::from(i32::MAX), identity)), current: true };
    let root: PathBuf = fixture("claim-long-name");
    let file_name: String = "i".repeat(40) + ".lock";
    let lock: PathBuf = root.join(file_name);
    std::fs::write(&lock, "x").expect("a file");
    let key: String = claim_key(&marked);
    let claim: PathBuf = claim_path(&lock, key.as_str(), usize::MAX);
    let length: usize = claim.file_name().expect("a name").len();
    assert!(length <= NAME_LIMIT, "the claim name is {length} bytes: {claim:?}");
    remove(root.as_path());
}

/// Allowed:
/// the claim name of a marker that names no process (a release before the holder field wrote it) comes from the marker
///  too,
/// since two mounts of one remote tree give one file different device numbers there as well.
#[test]
fn the_claim_name_of_a_marker_without_a_holder_does_not_depend_on_the_file() {
    let root: PathBuf = fixture("claim-key-holderless");
    let (first, second) = (root.join("first"), root.join("second"));
    std::fs::write(&first, format!("{LOCK_MARKER}5\n")).expect("a lock");
    std::fs::write(&second, format!("{LOCK_MARKER}5\n")).expect("the same lock seen by another mount");
    let key_of = |path: &Path| -> String {
        let file = std::fs::File::open(path).expect("the lock");
        let marked = marker_of_file(&file).expect("a marker");
        assert!(marked.holder.is_none());
        return claim_key(&marked);
    };
    assert_eq!(key_of(&first), key_of(&second));
    remove(root.as_path());
}

/// Allowed:
/// a claim is placed,
/// and the stale lock removed,
/// beside a lock whose own name is far longer than any identity,
/// because the
///  claim's name is a fixed-length digest and not the lock's name with the whole identity appended.
#[test]
fn a_claim_is_placed_beside_a_lock_with_a_very_long_name() {
    crate::claim_time::test_machine::inject("0123456789abcdef0123456789abcdef");
    let root: PathBuf = fixture("claim-very-long-name");
    let lock: PathBuf = root.join("i".repeat(LONG_LOCK_NAME) + ".lock");
    // A gone process with a whole identity (boot, namespace and machine), so that the identity is as long as a real one.
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    let boot: &str = crate::claim_time::current_boot_id();
    let text: String = gone_marker(5).replacen("gone", &format!("gone#{boot}#{scope}#0123456789abcdef0123456789abcdef"), 1);
    std::fs::write(&lock, text).expect("a stale lock");
    let seen = std::rc::Rc::new(std::cell::Cell::new(0_usize));
    let (hook_lock, hook_seen) = (lock.clone(), std::rc::Rc::clone(&seen));
    hooks::set("reclaim-held", std::rc::Rc::new(move || { hook_seen.set(claim_at(&hook_lock, 0).file_name().map_or(0, |name| { return name.len(); })); }));
    assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Removed);
    assert!(seen.get() > 0 && seen.get() <= NAME_LIMIT, "the claim name is {} bytes", seen.get());
    assert_eq!(std::fs::read_dir(&root).expect("listing").count(), 0, "nothing is left");
    remove(root.as_path());
}

/// The length of a lock's name before `.lock`:
/// the staging file of a lock adds about a hundred bytes to its name,
/// so this leaves it
///  within the limit while the earlier claim name (the lock's name,
/// the identity and a slot) is not.
const LONG_LOCK_NAME: usize = 150;

/// A marker for a process of another PID namespace on this boot:
/// its maker cannot be judged here,
/// and it is not another machine's.
fn claim_text_of_an_unjudgeable_maker() -> String {
    let boot: &str = crate::claim_time::current_boot_id();
    return format!("{LOCK_MARKER}{}0 4242 linux:1#{boot}#1\n", crate::service_index_marker::MARKER_VERSION);
}

/// Refused and told:
/// a claim whose maker cannot be judged (another PID namespace) is neither passed over nor removed,
/// and it is
///  not passing contention:
/// the answer names the claim,
/// so the person can remove it.
#[test]
fn a_claim_that_cannot_be_judged_is_named_and_not_taken_for_contention() {
    let (root, _index, lock) = index_in("claim-unjudged");
    let claim: PathBuf = claim_at(&lock, 0);
    let text: String = claim_text_of_an_unjudgeable_maker();
    std::fs::write(&claim, &text).expect("a claim of a maker that cannot be judged");
    match reclaim(&SYSTEM, &lock) {
        Reclaimed::Unjudged { file, marked } => {
            assert_eq!(file, claim);
            assert_eq!(marked.map(|found| { return found.seq; }), Some(0));
        }
        other => panic!("expected the claim to be named: {other:?}"),
    }
    assert!(lock.exists(), "the stale lock stays");
    assert_eq!(std::fs::read_to_string(&claim).expect("the claim stands"), text);
    remove(root.as_path());
}

/// The words for a claim that cannot be judged name the file and the maker and give the removal.
#[test]
fn the_words_for_a_claim_that_cannot_be_judged_give_the_removal() {
    let marked = parse_marker(claim_text_of_an_unjudgeable_maker().as_str()).expect("a marker");
    let words: String = unjudged_words(Path::new("/repo/it's here/cli-reclaim-x-0"), Some(&marked));
    for needed in ["/repo/it's here/cli-reclaim-x-0", "process 4242", "cannot", "rm -- '/repo/it'\\''s here/cli-reclaim-x-0'"] {
        assert!(words.contains(needed), "missing {needed}: {words}");
    }
}

/// A landing that meets such a claim waits to its ceiling and names the claim in its notice.
#[test]
fn a_landing_names_the_claim_that_cannot_be_judged() {
    let (root, index, lock) = index_in("claim-unjudged-landing");
    let claim: PathBuf = claim_at(&lock, 0);
    std::fs::write(&claim, claim_text_of_an_unjudgeable_maker()).expect("a claim");
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 4;
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let failure = acquire_within(&SYSTEM, &index, 9, ceiling, &|announce| {
        if let Announce::Waiting(line) = announce {
            said.borrow_mut().push(line);
        }
    })
    .expect_err("the lock is not taken");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    let lines: String = said.borrow().join("\n");
    assert!(lines.contains(claim.to_string_lossy().as_ref()) && lines.contains("rm --"), "{lines}");
    remove(root.as_path());
}

/// Allowed:
/// a placement whose answer was lost (it took effect,
/// `EINTR`) and whose claim could not be read back to be recognised
///  (`EACCES` at that moment) is not forgotten:
/// the next attempt of this thread finds its own abandoned claim by its text and
///  removes it,
/// instead of meeting a live-looking claim of its own process for ever.
#[test]
fn an_abandoned_claim_of_this_thread_is_resolved_by_its_next_attempt() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, _index, lock) = index_in("claim-abandoned");
        let claim: PathBuf = claim_at(&lock, 0);
        let lost = |from: &Path, to: &Path| -> std::io::Result<()> {
            rename_noreplace(from, to)?;
            return Err(std::io::Error::from_raw_os_error(libc::EINTR));
        };
        let (hook_claim, done) = (claim.clone(), std::rc::Rc::new(std::cell::Cell::new(false)));
        hooks::set("reclaim-claim-unread", std::rc::Rc::new(move || {
            if !done.replace(true) {
                // The claim cannot be read at this moment.
                std::fs::set_permissions(&hook_claim, std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("the claim is unreadable");
            }
        }));
        let first: Platform = Platform { rename: &lost, ..SYSTEM };
        assert_eq!(reclaim(&first, &lock), Reclaimed::Busy, "the placement's answer is lost and the claim cannot be recognised");
        assert!(claim.exists(), "the claim took effect");
        std::fs::set_permissions(&claim, std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("readable again");
        assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Removed, "the next attempt resolves its own abandoned claim and goes on");
        assert_eq!(names_in(&root), vec![String::from("index")]);
        remove(root.as_path());
    });
}

/// Refused:
/// a stale lock written in the format of an earlier release is left,
/// because that release's reclaimers (a `flock`) and
///  this release's (claims) cannot keep each other apart;
/// the answer carries the file and words that say so.
#[test]
fn a_stale_lock_of_an_earlier_releases_format_is_left_and_named() {
    let (root, _index, lock) = index_in("claim-legacy");
    let legacy: String = gone_marker(5).replacen(crate::service_index_marker::MARKER_VERSION, "", 1);
    std::fs::write(&lock, &legacy).expect("a legacy lock");
    match reclaim(&SYSTEM, &lock) {
        Reclaimed::Foreign { file, marked } => {
            assert_eq!(file, lock);
            assert!(!marked.current);
            let words: String = foreign_words(file.as_path(), &marked);
            assert!(words.contains("earlier release") && words.contains("rm --"), "{words}");
        }
        other => panic!("expected the legacy lock to be left: {other:?}"),
    }
    assert_eq!(std::fs::read_to_string(&lock).expect("the lock stands"), legacy);
    assert_eq!(names_in(&root), vec![String::from("index"), String::from("index.lock")], "no claim was placed");
    remove(root.as_path());
}

/// Refused:
/// a marker of this release is not one an earlier release's reader reads as a marker at all:
/// it reads the lock as a lock it
///  does not know,
/// which it leaves,
/// so an earlier release's reclaimers never act on a lock of this release.
#[test]
fn an_earlier_releases_reader_finds_no_marker_in_a_marker_of_this_release() {
    let text: String = marker_text(7, &own_identity).expect("a marker");
    // The earlier release's reader: the prefix, then a batch number, then a holder.
    let after_prefix: &str = text.strip_prefix(LOCK_MARKER).expect("the prefix");
    let first_word: &str = after_prefix.split(' ').next().expect("a word");
    assert!(first_word.parse::<u64>().is_err(), "an earlier release would read {first_word:?} as a batch number");
}

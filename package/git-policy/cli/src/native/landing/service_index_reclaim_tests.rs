//! Controls for the removal of a stale lock: it takes away only the file it found stale, and a lock another program placed in
//!  its place is never removed.

use super::*;
use crate::service_fault::hooks;
use crate::service_index_marker::LOCK_MARKER;
use crate::test_support::{fixture, remove};

/// A killed service's lock for batch `seq`: a marker whose holder is not any running process.
fn stale_lock_text(seq: u64) -> String {
    let mut child = std::process::Command::new("true").spawn().expect("a short process");
    let pid: i64 = i64::from(child.id());
    child.wait().expect("reaped");
    return format!("{LOCK_MARKER}{seq} {pid} gone\n");
}

fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let index: PathBuf = root.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

/// A stale lock is removed, and nothing is left beside the index.
#[test]
fn a_stale_lock_is_removed_and_leaves_no_file() {
    let (root, index) = index_in("reclaim-plain");
    std::fs::write(lock_path_of(&index), stale_lock_text(5)).expect("stale lock");
    remove_stale(&index, 5);
    assert!(!lock_path_of(&index).exists());
    let names: Vec<String> = std::fs::read_dir(&root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(names, vec![String::from("index")], "no taken-aside file is left");
    remove(root.as_path());
}

/// Another program that places its own lock after the stale one was found, and before it was removed, keeps its lock: the file
///  the removal takes is looked at again, and a file that is not the stale one is put back where it was.
#[test]
fn a_lock_placed_after_the_stale_one_was_found_is_not_removed() {
    let (root, index) = index_in("reclaim-replaced");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let (hook_lock, hook_root) = (lock.clone(), root.clone());
    hooks::set("reclaim-judged", std::rc::Rc::new(move || {
        // The stale lock goes and a foreign program's lock takes its name, as Git's own `index.lock` would.
        std::fs::remove_file(&hook_lock).expect("the stale lock is removed by someone else");
        std::fs::write(hook_root.join("staged-foreign"), "a foreign program's lock").expect("staged");
        std::fs::rename(hook_root.join("staged-foreign"), &hook_lock).expect("placed");
    }));
    remove_stale(&index, 5);
    assert_eq!(std::fs::read_to_string(&lock).expect("the foreign lock stands"), "a foreign program's lock");
    let names: Vec<String> = std::fs::read_dir(&root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(names.len(), 2, "only the index and the lock: {names:?}");
    remove(root.as_path());
}

/// A lock that is not the service's, or whose holder runs, is never taken aside.
#[test]
fn a_foreign_lock_and_a_running_holders_lock_stay() {
    let (root, index) = index_in("reclaim-foreign");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, "foreign").expect("foreign");
    remove_stale(&index, 5);
    assert_eq!(std::fs::read_to_string(&lock).expect("stands"), "foreign");
    remove(root.as_path());
}

/// What:
///  A reclaimer that is working on a live lock another instance placed (it judged the stale lock that stood there before, then the
///  name was reclaimed and taken) leaves that lock under its name for the whole time, so a third program (Git) cannot create the
///  lock.
/// Why:
///   This is the race two services reclaiming one physical index (an alternate index shared by two worktrees) could run: the
///  earlier reclaim took the live lock aside by a rename, and Git created `index.lock` while it was off its name, which left the
///  live holder's lock stranded and the batch open. The first version of this control failed on that code (`dd37f0c462`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = tryAcquire(index, 7); hook('reclaim-opened', () => gitCreatesLock()); reclaim(lock);
/// expect(gitCreatedLock).toBe(false); expect(read(lock)).toBe(liveMarker);
/// ```
#[test]
fn a_live_lock_found_by_a_late_reclaimer_is_never_open_to_a_third_program() {
    let (root, index) = index_in("reclaim-late-reclaimer");
    let lock: PathBuf = lock_path_of(&index);
    // The other instance's live lock, the one the late reclaimer's earlier look at the stale lock did not see.
    let live = try_acquire(&index, 7).expect("the live holder places its lock");
    let live_marker: String = std::fs::read_to_string(&lock).expect("the live marker");
    let third_placed = std::rc::Rc::new(std::cell::Cell::new(false));
    let (hook_lock, hook_placed) = (lock.clone(), std::rc::Rc::clone(&third_placed));
    hooks::set("reclaim-opened", std::rc::Rc::new(move || {
        // Git takes `index.lock` by exclusive creation, as it does for every command that writes the index.
        let created = std::fs::OpenOptions::new().write(true).create_new(true).open(&hook_lock);
        hook_placed.set(created.is_ok());
    }));
    let outcome: Reclaimed = reclaim(&SYSTEM, &lock);
    assert_eq!(outcome, Reclaimed::Kept, "a lock whose holder runs is not removed");
    assert!(!third_placed.get(), "a third program placed its lock while the reclaimer worked on the live holder's lock");
    assert_eq!(std::fs::read_to_string(&lock).expect("a lock stands"), live_marker, "the live holder's lock stands under its name");
    let names: Vec<String> = std::fs::read_dir(&root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(names.len(), 2, "only the index and the lock: {names:?}");
    drop(live);
    assert!(!lock.exists(), "the live holder still owns its lock and releases it");
    remove(root.as_path());
}

/// Refused: a reclaimer that judged a stale lock, and finds at its check that the name now refers to another file (the stale
///  lock was reclaimed and a live landing placed its own), removes nothing, moves nothing, and says the name changed. Allowed:
///  the same reclaimer on a name that still refers to the file it judged removes it (`a_stale_lock_is_removed_and_leaves_no_file`).
#[test]
fn a_reclaimer_whose_judged_file_was_replaced_removes_nothing() {
    let (root, index) = index_in("reclaim-replaced-inode");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let live_cell: std::rc::Rc<std::cell::RefCell<Option<IndexLock>>> = std::rc::Rc::new(std::cell::RefCell::new(None));
    let (hook_lock, hook_index, hook_live) = (lock.clone(), index.clone(), std::rc::Rc::clone(&live_cell));
    hooks::set("reclaim-judged", std::rc::Rc::new(move || {
        // The other reclaimer removes the stale lock it judged too, and its landing places a live lock.
        std::fs::remove_file(&hook_lock).expect("the other reclaimer removes the stale lock");
        *hook_live.borrow_mut() = Some(try_acquire(&hook_index, 8).expect("the live landing places its lock"));
    }));
    let outcome: Reclaimed = reclaim(&SYSTEM, &lock);
    assert_eq!(outcome, Reclaimed::Changed);
    assert_eq!(std::fs::read_to_string(&lock).expect("the live lock stands"), marker_text(8, &crate::service_index_marker::own_identity).expect("this process has an identity"));
    let live = live_cell.borrow_mut().take().expect("the live lock");
    drop(live);
    assert!(!lock.exists(), "the live holder still owns its lock and releases it");
    remove(root.as_path());
}

/// Refused: while one reclaimer holds a stale lock to remove it, a second reclaimer of the same lock backs off and removes
///  nothing, so it cannot remove what a landing placed after the first; the first still removes the stale lock.
#[test]
fn a_second_reclaimer_of_one_stale_lock_backs_off_while_the_first_holds_it() {
    let (root, index) = index_in("reclaim-two-reclaimers");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let second = std::rc::Rc::new(std::cell::Cell::new(None));
    let (hook_lock, hook_second) = (lock.clone(), std::rc::Rc::clone(&second));
    hooks::set("reclaim-held", std::rc::Rc::new(move || {
        // The second reclaimer reaches the same file while the first holds it, and reaches no point of the hold itself.
        hook_second.set(Some(reclaim(&SYSTEM, &hook_lock)));
    }));
    let first: Reclaimed = reclaim(&SYSTEM, &lock);
    assert_eq!(second.get(), Some(Reclaimed::Busy), "the second reclaimer backs off");
    assert_eq!(first, Reclaimed::Removed, "the first reclaimer removes the stale lock");
    assert!(!lock.exists());
    remove(root.as_path());
}

/// Allowed after the refusal: once the first reclaimer is done, a reclaimer takes the next stale lock at the same name.
#[test]
fn a_reclaimer_that_backed_off_can_reclaim_once_the_hold_is_dropped() {
    let (root, index) = index_in("reclaim-after-busy");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let holding = std::fs::File::open(&lock).expect("the first reclaimer's descriptor");
    // SAFETY: `flock` takes an open descriptor, which `holding` keeps open for the call.
    let taken: i32 = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&holding), libc::LOCK_EX | libc::LOCK_NB) };
    assert_eq!(taken, 0, "the control plays the first reclaimer");
    assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Busy);
    assert!(lock.exists(), "a reclaimer that backs off removes nothing");
    drop(holding);
    assert_eq!(reclaim(&SYSTEM, &lock), Reclaimed::Removed);
    assert!(!lock.exists());
    remove(root.as_path());
}

/// A landing that meets a stale lock another reclaimer holds neither removes it nor fails at once: it waits as for any held lock
///  and ends at its ceiling, and the stale lock stays for the reclaimer that holds it.
#[test]
fn a_landing_waits_for_the_reclaimer_that_holds_a_stale_lock() {
    let (root, index) = index_in("reclaim-landing-busy");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let holding = std::fs::File::open(&lock).expect("the other reclaimer's descriptor");
    // SAFETY: `flock` takes an open descriptor, which `holding` keeps open for the call.
    let taken: i32 = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&holding), libc::LOCK_EX | libc::LOCK_NB) };
    assert_eq!(taken, 0, "the control plays the other reclaimer");
    let ceiling: std::time::Duration = std::time::Duration::from_millis(200);
    let failure = acquire_within(&SYSTEM, &index, 9, ceiling, &|_| {}).expect_err("the lock is not taken while the other reclaimer holds it");
    assert_eq!(failure.code, FailureCode::IndexInUse, "the wait ends at its ceiling, as for any held lock");
    assert!(lock.exists(), "the stale lock stays for the reclaimer that holds it");
    assert!(try_acquire(&index, 9).is_none(), "a restart's single attempt leaves the batch open");
    drop(holding);
    let reclaimed = acquire_within(&SYSTEM, &index, 9, ceiling, &|_| {}).expect("the stale lock is reclaimed once nobody holds it");
    drop(reclaimed);
    remove(root.as_path());
}

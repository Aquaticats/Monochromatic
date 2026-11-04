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

/// The hold the platform reports as taken by another reclaimer is a back-off whatever the file system is.
#[test]
fn a_hold_reported_as_taken_by_another_reclaimer_is_a_back_off() {
    let (root, index) = index_in("reclaim-platform-busy");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let held = |_: &std::fs::File| -> std::io::Result<()> { return Err(std::io::Error::from(std::io::ErrorKind::WouldBlock)); };
    let platform: Platform = Platform { flock: &held, ..SYSTEM };
    assert_eq!(reclaim(&platform, &lock), Reclaimed::Busy);
    assert!(lock.exists());
    remove(root.as_path());
}

/// A `flock` that fails for any reason but a hold by another reclaimer (no `flock` on the file system, `ENOLCK`) never has the
///  stale lock removed: the unlink that follows the name check is not safe without the hold, because a second reclaimer without
///  it could remove the stale lock and a live landing could place its own in between, and the unlink would remove that.
#[test]
fn a_failed_flock_never_has_the_stale_lock_removed() {
    let (root, index) = index_in("reclaim-no-flock");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let tried = std::cell::Cell::new(0_usize);
    let refusing = |_: &std::fs::File| -> std::io::Result<()> {
        tried.set(tried.get() + 1);
        return Err(std::io::Error::from_raw_os_error(libc::ENOLCK));
    };
    let platform: Platform = Platform { flock: &refusing, ..SYSTEM };
    let outcome: Reclaimed = reclaim(&platform, &lock);
    assert_eq!(outcome, Reclaimed::Unguarded, "the stale lock is left, and the caller is told why");
    assert_eq!(tried.get(), 1, "the hold was asked for once");
    assert!(lock.exists(), "the stale lock stays");
    remove(root.as_path());
}

/// A landing on a file system with no `flock` treats the stale lock as a held lock: it waits to its ceiling, withdraws the
///  commit with the index in use, leaves the lock where it is, and tells the person which lock it is, whose batch and process
///  left it, and how to remove it.
#[test]
fn a_landing_with_no_flock_waits_to_its_ceiling_and_names_the_stale_lock() {
    let (root, index) = index_in("reclaim-landing-no-flock");
    let lock: PathBuf = lock_path_of(&index);
    let text: String = stale_lock_text(5);
    std::fs::write(&lock, &text).expect("stale lock");
    let refusing = |_: &std::fs::File| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(libc::ENOLCK)); };
    let platform: Platform = Platform { flock: &refusing, ..SYSTEM };
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 4;
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let began: std::time::Instant = std::time::Instant::now();
    let failure = acquire_within(&platform, &index, 9, ceiling, &|announce| {
        if let Announce::Waiting(line) = announce {
            said.borrow_mut().push(line);
        }
    })
    .expect_err("the lock is not taken");
    assert_eq!(failure.code, FailureCode::IndexInUse);
    assert!(began.elapsed() >= ceiling, "the wait ended before its ceiling");
    assert_eq!(std::fs::read_to_string(&lock).expect("the stale lock stays"), text);
    let lines: String = said.borrow().join("\n");
    assert!(lines.contains(lock.to_string_lossy().as_ref()), "the notice names the lock: {lines}");
    assert!(lines.contains("remove it"), "the notice says how to remove it: {lines}");
    assert!(lines.contains("batch 5"), "the notice names the batch that left it: {lines}");
    remove(root.as_path());
}

/// A landing that meets a stale lock another reclaimer holds waits in the same acquisition and takes the lock once the hold is
///  dropped without the file being unlinked: the hold leaves no event in the directory, so the wait looks again at the poll pace.
#[test]
fn a_landing_takes_the_stale_lock_when_the_other_reclaimers_hold_is_dropped() {
    let (root, index) = index_in("reclaim-landing-wakes");
    let lock: PathBuf = lock_path_of(&index);
    std::fs::write(&lock, stale_lock_text(5)).expect("stale lock");
    let hold: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 3;
    let ceiling: std::time::Duration = hold * 40;
    let (taken_tx, taken_rx) = std::sync::mpsc::channel::<()>();
    let hook_lock: PathBuf = lock.clone();
    let reclaimer = std::thread::spawn(move || {
        let holding = std::fs::File::open(&hook_lock).expect("the other reclaimer's descriptor");
        // SAFETY: `flock` takes an open descriptor, which `holding` keeps open for the call.
        let taken: i32 = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&holding), libc::LOCK_EX | libc::LOCK_NB) };
        assert_eq!(taken, 0, "the control plays the other reclaimer");
        taken_tx.send(()).expect("the landing is waiting for the hold");
        std::thread::sleep(hold);
        drop(holding);
    });
    taken_rx.recv().expect("the hold is taken");
    let holders = std::cell::RefCell::new(Vec::<String>::new());
    let began: std::time::Instant = std::time::Instant::now();
    let taken = acquire_within(&SYSTEM, &index, 9, ceiling, &|announce| {
        if let Announce::Holder(who) = announce {
            holders.borrow_mut().push(who);
        }
    })
    .expect("the stale lock is reclaimed in the same acquisition once the hold is dropped");
    assert!(began.elapsed() < ceiling, "the acquisition took the whole ceiling");
    assert!(holders.borrow().is_empty(), "a held stale lock is no dead holder to give up on: {:?}", holders.borrow());
    drop(taken);
    reclaimer.join().expect("the other reclaimer ends");
    remove(root.as_path());
}

/// The old sequence took a live lock aside by a rename, so the lock's name was empty for a while and Git could take it. The
///  directory is watched while a reclaimer works on a live lock, and no event may name the lock: no rename, no removal, no
///  creation. The scheduling is the kernel's own, so the control needs no hook inside the reclaimer and fails against the old
///  sequence behind the same call.
#[test]
fn a_reclaimer_that_finds_a_live_lock_leaves_no_event_on_the_lock_name() {
    let (root, index) = index_in("reclaim-live-no-event");
    let lock: PathBuf = lock_path_of(&index);
    let live = try_acquire(&index, 7).expect("the live holder places its lock");
    // SAFETY: `inotify_init1` takes flags only.
    let watch: i32 = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
    assert!(watch >= 0, "an inotify descriptor");
    let directory = std::ffi::CString::new(std::os::unix::ffi::OsStrExt::as_bytes(root.as_os_str())).expect("a path");
    let events: u32 = libc::IN_MOVED_FROM | libc::IN_MOVED_TO | libc::IN_DELETE | libc::IN_CREATE;
    // SAFETY: both arguments outlive the call.
    let added: i32 = unsafe { libc::inotify_add_watch(watch, directory.as_ptr(), events) };
    assert!(added >= 0, "a watch on the directory");
    let _outcome = reclaim(&SYSTEM, &lock);
    let mut buffer: Vec<u8> = vec![0; 4096];
    // SAFETY: the buffer is writable for its whole length.
    let read: isize = unsafe { libc::read(watch, buffer.as_mut_ptr().cast(), buffer.len()) };
    // SAFETY: the descriptor was opened above and is closed once.
    unsafe { libc::close(watch) };
    let seen: String = if read > 0 { String::from_utf8_lossy(&buffer[..read as usize]).into_owned() } else { String::new() };
    assert!(!seen.contains("index.lock"), "the live lock's name saw events: {seen:?}");
    assert_eq!(std::fs::read_to_string(&lock).expect("the live lock stands"), marker_text(7, &crate::service_index_marker::own_identity).expect("this process has an identity"));
    drop(live);
    remove(root.as_path());
}

/// The notice for a stale lock that is not removed describes the file the reclaimer judged, not whatever stands at the name when the
///  notice is written: when the stale lock is replaced by a live landing's lock in between, the notice names the stale lock's
///  process, and never calls the live lock's process gone.
#[test]
fn the_notice_names_the_judged_stale_lock_and_not_a_live_replacement() {
    let (root, index) = index_in("reclaim-notice-replaced");
    let lock: PathBuf = lock_path_of(&index);
    let stale: String = stale_lock_text(5);
    let stale_marked: Marked = parse_marker(stale.as_str()).expect("a marker");
    let stale_pid: i64 = stale_marked.holder.as_ref().expect("a holder").0;
    std::fs::write(&lock, &stale).expect("stale lock");
    let live_cell: std::cell::RefCell<Option<IndexLock>> = std::cell::RefCell::new(None);
    // The hold fails once, and the stale lock is reclaimed by another instance and replaced by a live landing's lock meanwhile.
    let replacing = |_: &std::fs::File| -> std::io::Result<()> {
        if live_cell.borrow().is_none() {
            std::fs::remove_file(&lock).expect("the other reclaimer removes the stale lock");
            *live_cell.borrow_mut() = Some(try_acquire(&index, 8).expect("the live landing places its lock"));
        }
        return Err(std::io::Error::from_raw_os_error(libc::ENOLCK));
    };
    let platform: Platform = Platform { flock: &replacing, ..SYSTEM };
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let ceiling: std::time::Duration = crate::service_index_watch::POLL_PAUSE * 4;
    let _failure = acquire_within(&platform, &index, 9, ceiling, &|announce| {
        if let Announce::Waiting(line) = announce {
            said.borrow_mut().push(line);
        }
    })
    .expect_err("the live lock is held to the ceiling");
    let first: String = said.borrow().first().cloned().expect("a notice");
    assert!(first.contains(&format!("process {stale_pid}")), "the notice names the judged stale lock's process: {first}");
    assert!(!first.contains(&format!("process {}", std::process::id())), "the notice calls the live lock's process gone: {first}");
    drop(live_cell.borrow_mut().take());
    remove(root.as_path());
}

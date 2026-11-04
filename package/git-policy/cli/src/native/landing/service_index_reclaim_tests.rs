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
    hooks::set("reclaim-checked", std::rc::Rc::new(move || {
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
///  A reclaimer that checked a lock as stale, and reaches its rename only after another instance has replaced that lock with its
///  own live one, takes the live lock aside; a third program (Git) places its own lock during the moments the live one is off
///  its name.
/// Why:
///   This is the race two services reclaiming one physical index (an alternate index shared by two worktrees) can run:
///  `remove_stale`, `try_acquire` and the wait of a landing each read the marker and then rename by name, which is two steps.
///  The live holder then finds another file under its name, so it installs nothing and its batch stays open; and the third
///  program and the live holder both believed they hold the lock for as long as the stranded lock lasts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = tryAcquire(index, 7); hook('reclaim-aside', () => gitCreatesLock()); reclaim(lock);
/// expect(gitCreatedLock).toBe(false); expect(read(lock)).toBe(liveMarker);
/// ```
#[test]
#[ignore = "red: shows the late-reclaimer race, which no guard closes yet; run with --run-ignored all"]
fn a_live_lock_taken_aside_by_a_late_reclaimer_is_never_open_to_a_third_program() {
    let (root, index) = index_in("reclaim-late-reclaimer");
    let lock: PathBuf = lock_path_of(&index);
    // The other instance's live lock, the one the late reclaimer's earlier look at the stale lock did not see.
    let live = try_acquire(&index, 7).expect("the live holder places its lock");
    let live_marker: String = std::fs::read_to_string(&lock).expect("the live marker");
    let third_placed = std::rc::Rc::new(std::cell::Cell::new(false));
    let (hook_lock, hook_placed) = (lock.clone(), std::rc::Rc::clone(&third_placed));
    hooks::set("reclaim-aside", std::rc::Rc::new(move || {
        // Git takes `index.lock` by exclusive creation, as it does for every command that writes the index.
        let created = std::fs::OpenOptions::new().write(true).create_new(true).open(&hook_lock);
        hook_placed.set(created.is_ok());
    }));
    let removed: bool = reclaim(&SYSTEM, &lock);
    assert!(!removed, "a lock whose holder runs is not removed");
    assert!(!third_placed.get(), "a third program placed its lock while the live holder's lock was off its name");
    assert_eq!(std::fs::read_to_string(&lock).expect("a lock stands"), live_marker, "the live holder's lock is back under its name");
    let names: Vec<String> = std::fs::read_dir(&root).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(names.len(), 2, "only the index and the lock: {names:?}");
    drop(live);
    remove(root.as_path());
}

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

//! Controls for taking the index lock of a landing:
//! the lock names the batch that took it,
//! and the marker of the batch after that one begins flushing while this landing holds the lock,
//! so the next commit finds it on the disk instead of waiting for a flush this landing started after it ended.

use super::*;
use crate::service_index_lock::{lock_path_of, marker_text};
use crate::service_index_marker::own_identity;
use crate::service_index_prestage;
use crate::test_support::{fixture, remove};
use std::sync::Mutex;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The controls run one at a time:
/// the marker made ahead is kept in one slot per landing thread,
/// and the slot is the process's.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The names in the index's directory other than the index and its lock.
fn leftovers(index: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(index.parent().expect("directory"))
        .expect("listing")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name != "index" && name != "index.lock"; })
        .collect();
    names.sort();
    return names;
}

/// The request a landing takes the lock for:
/// nothing in it is read once the lock is taken,
/// so only the index and the scratch tree name real places.
fn request_of(index: &Path, scratch: &Path, git_dir: &Path) -> CommitRequest {
    return CommitRequest {
        id: String::from("control"),
        repository: git_dir.parent().expect("repository").to_path_buf(),
        common_dir: git_dir.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        reference: String::from("refs/heads/control"),
        base: String::from("0".repeat(40)),
        changes: Vec::new(),
        message: String::from("control"),
        author: None,
        committer: String::from("Control <control@example.invalid> 0 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: index.to_path_buf(),
        scratch: scratch.to_path_buf(),
        captured: 0,
    };
}

/// A context whose Git is never started here:
/// taking the lock runs no child.
fn context_of(git_dir: &Path) -> GitContext {
    return GitContext { real_git: PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(git_dir.to_path_buf()) };
}

/// The marker made ahead is written by a job of the blocking pool,
/// so the file appears a moment after the landing asks for it:
/// waits for that file while the landing still holds the lock.
fn until_the_next_batchs_marker_is_being_made(index: &Path) -> Vec<String> {
    let began: Instant = Instant::now();
    loop {
        let staged: Vec<String> = leftovers(index);
        if !staged.is_empty() {
            return staged;
        }
        assert!(began.elapsed() < Duration::from_secs(60), "no marker of the batch after this one was started while the landing held the lock");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The marker of the batch after the one that took the lock begins flushing while this landing holds the lock:
/// its file is there before the landing ends,
/// and its text is that batch's.
#[test]
fn the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let root: PathBuf = fixture("take-index-lock-starts-next-batchs-marker");
        let git_dir: PathBuf = root.join("git-dir");
        std::fs::create_dir(&git_dir).expect("directory");
        let index: PathBuf = git_dir.join("index");
        std::fs::write(&index, "old index").expect("index");
        let scratch: PathBuf = root.join("scratch");
        std::fs::create_dir(&scratch).expect("scratch");
        let journal: PathBuf = root.join("journal");
        std::fs::create_dir(&journal).expect("journal");
        let first: CommitRequest = request_of(index.as_path(), scratch.as_path(), git_dir.as_path());
        let mut timing: LandTiming = LandTiming::default();
        let held: IndexLock = take_index_lock(&context_of(git_dir.as_path()), journal.as_path(), Duration::from_secs(30), &first, 7, &|_| {}, &mut timing, "new", "scratch-ref").expect("the lock is taken");
        // The lock is the batch's own marker, and the next batch's marker is being made beside it while this one is held.
        assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("the lock's text"), marker_text(7, &own_identity).expect("text"));
        let staged: Vec<String> = until_the_next_batchs_marker_is_being_made(index.as_path());
        assert_eq!(staged.len(), 1, "one file at most is made ahead: {staged:?}");
        assert_eq!(std::fs::read_to_string(git_dir.join(&staged[0])).expect("the staged marker's text"), marker_text(8, &own_identity).expect("text"));
        drop(held);
        service_index_prestage::discard();
        remove(root.as_path());
    });
}

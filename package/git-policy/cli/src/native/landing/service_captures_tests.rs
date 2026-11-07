//! Controls for the memory of landed captures:
//! what is kept,
//! what is forgotten,
//! and that worktrees stay apart.

use super::*;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

/// Entries come back oldest first,
/// an unknown stamp is not kept,
/// and one worktree's entries are not another's.
#[test]
fn entries_are_kept_per_worktree_and_an_unknown_stamp_is_not_kept() {
    let common: PathBuf = PathBuf::from("/captures/one/repo-dir");
    let first: PathBuf = PathBuf::from("/captures/one");
    let second: PathBuf = PathBuf::from("/captures/two");
    let mut memory: CaptureMemory = CaptureMemory::default();
    memory.record_landed(common.as_path(), first.as_path(), COMMIT, 10);
    memory.record_landed(common.as_path(), first.as_path(), COMMIT, 0);
    memory.record_landed(common.as_path(), first.as_path(), COMMIT, 20);
    assert_eq!(memory.remembered(common.as_path(), first.as_path()).iter().map(|entry| { return entry.captured; }).collect::<Vec<u64>>(), vec![10, 20]);
    assert!(memory.remembered(common.as_path(), second.as_path()).is_empty());
}

/// The oldest entries go first once the bound is passed.
#[test]
fn the_oldest_entries_are_forgotten_first() {
    let common: PathBuf = PathBuf::from("/captures/bound/repo-dir");
    let worktree: PathBuf = PathBuf::from("/captures/bound");
    let extra: u64 = 3;
    let mut memory: CaptureMemory = CaptureMemory::default();
    for stamp in 1..=(u64::try_from(REMEMBERED).unwrap_or(0) + extra) {
        memory.record_landed(common.as_path(), worktree.as_path(), COMMIT, stamp);
    }
    let kept: Vec<LandedCapture> = memory.remembered(common.as_path(), worktree.as_path());
    assert_eq!(kept.len(), REMEMBERED);
    assert_eq!(kept.first().map(|entry| { return entry.captured; }), Some(extra + 1));
}

/// Two memories are two values:
/// nothing is shared between them,
/// which is what a table behind a lock in the process would have
/// done to two services (or two tests) that run in one process.
#[test]
fn a_memory_is_a_value_and_not_shared_with_another() {
    let common: PathBuf = PathBuf::from("/captures/apart/repo-dir");
    let worktree: PathBuf = PathBuf::from("/captures/apart");
    let mut first: CaptureMemory = CaptureMemory::default();
    let second: CaptureMemory = CaptureMemory::default();
    first.record_landed(common.as_path(), worktree.as_path(), COMMIT, 1);
    assert_eq!(first.remembered(common.as_path(), worktree.as_path()).len(), 1);
    assert!(second.remembered(common.as_path(), worktree.as_path()).is_empty());
}

//! What:
//! Sizes of directory trees by the disk space the files occupy,
//! with hard links counted once and a deadline.
//! Why:
//! Caps and reports are about bytes on disk.
//! A walk of a build tree can run for minutes on this machine,
//! so it obeys a deadline and says when it stopped early.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function treeBytes(path: string, seen: Set<string>, deadline: Deadline): { bytes: number; complete: boolean }
//! ```

/// Directory reading and metadata.
use std::{collections::HashSet, fs, os::unix::fs::MetadataExt, path::Path};

/// Device and inode,
/// the deadline and the block arithmetic.
use crate::lean_identity::{Deadline, Identity};
/// Imports the names this file uses from `crate::lean_removal`.
use crate::lean_removal::allocated_bytes_of;

/// How many entries a walk reads between looks at the clock.
const ENTRIES_BETWEEN_CLOCK_READS: usize = 512;

/// The result of one size walk.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Measured {
    /// Disk bytes of the entries read.
    pub bytes: u64,
    /// Whether the whole tree was read before the deadline.
    pub complete: bool,
}

/// What:
/// `pub fn add_tree_bytes(path: &Path, seen: &mut HashSet<Identity>, deadline: &Deadline) -> Measured` adds the disk bytes of a
/// tree to a running total,
/// counting each hard-linked file once across every call that shares `seen`.
/// Why:
/// Units that hard-link the same artifact must not count it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addTreeBytes(path: string, seen: Set<string>, deadline: Deadline): Measured
/// ```
pub fn add_tree_bytes(path: &Path, seen: &mut HashSet<Identity>, deadline: &Deadline) -> Measured {
    let mut total: Measured = Measured { bytes: 0, complete: true };
    let mut stack: Vec<std::path::PathBuf> = vec![path.to_path_buf()];
    let mut counter: usize = 0;
    while let Some(current) = stack.pop() {
        counter += 1;
        if counter.is_multiple_of(ENTRIES_BETWEEN_CLOCK_READS) && deadline.passed() {
            total.complete = false;
            return total;
        }
        let Ok(metadata) = fs::symlink_metadata(&current) else { continue };
        if metadata.is_dir() {
            total.bytes += allocated_bytes_of(&metadata);
            if let Ok(children) = fs::read_dir(&current) {
                for child in children.flatten() {
                    stack.push(child.path());
                }
            }
            continue;
        }
        if metadata.nlink() <= 1 || seen.insert(Identity { device: metadata.dev(), inode: metadata.ino() }) {
            total.bytes += allocated_bytes_of(&metadata);
        }
    }
    return total;
}

/// What:
/// `pub fn tree_bytes(path: &Path, deadline: &Deadline) -> Measured` is the disk bytes of one tree.
/// Why:
/// The common case of `add_tree_bytes` with a fresh set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const treeBytes = (path: string, deadline: Deadline): Measured => addTreeBytes(path, new Set(), deadline);
/// ```
pub fn tree_bytes(path: &Path, deadline: &Deadline) -> Measured {
    let mut seen: HashSet<Identity> = HashSet::new();
    return add_tree_bytes(path, &mut seen, deadline);
}

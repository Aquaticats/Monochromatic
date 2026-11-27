//! What:
//! Sizes of directory trees,
//! with hard links counted once and a deadline.
//! Why:
//! Caps and reports are about bytes on disk,
//! so the walk asks its caller what a byte of one entry is (`ByteCount`):
//! the tools count the blocks the disk holds,
//! and a control that sizes what it wrote a moment ago counts the length,
//! which no file system has to settle first.
//! A walk of a build tree can run for minutes on this machine,
//! so it obeys a deadline and says when it stopped early.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function treeBytes(path: string, seen: Set<string>, deadline: Deadline): { bytes: number; complete: boolean }
//! ```

/// Directory reading and metadata.
use std::{
    collections::HashSet,
    fs,
    os::unix::fs::MetadataExt,
    path::Path,
};

/// Device and inode,
/// and the deadline.
use crate::lean_identity::{
    Deadline,
    Identity,
};

/// How many entries a walk reads between looks at the clock.
const ENTRIES_BETWEEN_CLOCK_READS: usize = 512;

/// What:
/// `pub type ByteCount = fn(&fs::Metadata) -> u64` is how a size walk counts the bytes of one entry:
/// a plain function from an entry's metadata to a number of bytes.
/// Why:
/// What counts as a byte depends on what the caller asks.
/// The tool's caps and reports count the bytes the disk holds (`crate::lean_removal::allocated_bytes_of`).
/// A control that sizes what it wrote a moment ago counts the bytes the entry's length says it holds,
/// because a file system may report no blocks for a fresh file until it accounts them,
/// and on this host's ZFS that is one 512-byte block until the pool's transaction group commits.
/// The siblings a reader might expect are a trait with one method (a TS interface every rule would implement)
/// and an enum naming the rules (which fixes the set at compile time);
/// a plain function is the one form TS's `(metadata: Stats) => number` maps onto directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ByteCount = (metadata: Stats) => number;
/// ```
pub type ByteCount = fn(&fs::Metadata) -> u64;

/// The result of one size walk.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Measured {
    /// The bytes of the entries read,
    /// by the rule the caller set.
    pub bytes: u64,
    /// Whether the whole tree was read before the deadline.
    pub complete: bool,
}

/// What:
/// `pub fn add_tree_bytes(path: &Path, seen: &mut HashSet<Identity>, deadline: &Deadline, count_bytes: ByteCount) -> Measured`
/// adds the bytes of a tree to a running total,
/// counting each hard-linked file once across every call that shares `seen`.
/// Why:
/// Units that hard-link the same artifact must not count it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addTreeBytes(path: string, seen: Set<string>, deadline: Deadline, countBytes: ByteCount): Measured
/// ```
pub fn add_tree_bytes(
    path: &Path,
    seen: &mut HashSet<Identity>,
    deadline: &Deadline,
    count_bytes: ByteCount,
) -> Measured {
    let mut total: Measured = Measured {
        bytes: 0,
        complete: true,
    };
    let mut stack: Vec<std::path::PathBuf> = vec![path.to_path_buf()];
    let mut counter: usize = 0;
    while let Some(current) = stack.pop() {
        counter += 1;
        if counter.is_multiple_of(ENTRIES_BETWEEN_CLOCK_READS) && deadline.passed() {
            total.complete = false;
            return total;
        }
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            continue;
        };
        if metadata.is_dir() {
            total.bytes += count_bytes(&metadata);
            if let Ok(children) = fs::read_dir(&current) {
                for child in children.flatten() {
                    stack.push(child.path());
                }
            }
            continue;
        }
        if metadata.nlink() <= 1
            || seen.insert(Identity {
                device: metadata.dev(),
                inode: metadata.ino(),
            })
        {
            total.bytes += count_bytes(&metadata);
        }
    }
    return total;
}

/// What:
/// `pub fn tree_bytes(path: &Path, deadline: &Deadline, count_bytes: ByteCount) -> Measured` is the bytes of one tree.
/// Why:
/// The common case of `add_tree_bytes` with a fresh set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const treeBytes = (path: string, deadline: Deadline, countBytes: ByteCount): Measured => addTreeBytes(path, new Set(), deadline, countBytes);
/// ```
pub fn tree_bytes(
    path: &Path,
    deadline: &Deadline,
    count_bytes: ByteCount,
) -> Measured {
    let mut seen: HashSet<Identity> = HashSet::new();
    return add_tree_bytes(path, &mut seen, deadline, count_bytes);
}

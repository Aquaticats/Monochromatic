//! What:
//!  The LFS objects a push introduces,
//! found in process:
//! the pointer blobs that the commits to be pushed add.
//! Why:
//!  `git-lfs push` starts a scanner (`git rev-list`,
//! `git cat-file`,
//! `git ls-remote`) and,
//! when it finds anything,
//! asks
//!  the LFS server about it;
//! most pushes add no pointer at all,
//! so the wrapper reads the commits itself with the
//!  in-process readers and starts `git-lfs` only when a pointer is found.
//! The commits are those reachable from the pushed
//!  tips and not reached before a known-good tip (the remote's tracking refs,
//! when they describe the destination).
//! Every
//!  commit is compared with each of its parents,
//! so a pointer added in one commit and removed in the next is still found.
//!  Anything the readers do not fully understand,
//! and any walk that grows past its limits,
//! returns `None`,
//! so the caller
//!  asks `git-lfs`.
//! What one scan reads and remembers,
//! and the committer time it keeps,
//! are in `lfs_pointer_scan_object_reads.rs`,
//! beside this file so that each file passes the package's length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const oids = newPointerObjects(odb, format, includeTips, stopTips, limits); // undefined: ask git-lfs
//! ```

#![cfg(unix)]

use crate::lfs_pointer::{
    Verdict,
    decode_pointer,
};
use crate::lfs_pointer_scan_object_reads::Scan;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::ObjectKind;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::CommitFields;
use std::collections::{
    BTreeSet,
    HashMap,
    HashSet,
};
use std::time::Duration;

/// The most commits one scan visits before it gives the question to `git-lfs`.
pub const MAX_COMMITS: usize = 100_000;

/// The longest one scan runs before it gives the question to `git-lfs`.
pub const SCAN_BUDGET: Duration = Duration::from_secs(3);

/// How many commits are still popped after only known commits remain queued,
/// because committer times can run backwards.
const CLOCK_SKEW_SLOP: usize = 5;

/// The queue of a date-ordered walk,
/// with each commit marked as known to the destination or not.
struct Walk {
    /// Commits waiting to be read,
    /// newest first.
    queue: std::collections::BinaryHeap<(i64, Vec<u8>)>,
    /// Whether each commit seen is known to the destination (`true`) or to be pushed (`false`).
    known: HashMap<Vec<u8>, bool>,
    /// Commits in the queue.
    queued: HashSet<Vec<u8>>,
    /// How many queued commits are not known to the destination.
    unknown_queued: usize,
    /// Commits whose parents have been followed.
    expanded: HashSet<Vec<u8>>,
}

impl Walk {
    /// Note that `name` is known (or unknown) to the destination,
    /// queueing it the first time and,
    /// when a commit already
    ///  followed turns out to be known,
    /// marking everything behind it as known too.
    fn mark(
        &mut self,
        scan: &mut Scan<'_>,
        name: &[u8],
        known: bool,
    ) -> Option<()> {
        let mut pending: Vec<Vec<u8>> = vec![name.to_vec()];
        while let Some(current) = pending.pop() {
            match self.known.get(current.as_slice()).copied() {
                None => {
                    scan.commit(current.as_slice())?;
                    self.known.insert(current.clone(), known);
                    self.queue
                        .push((scan.time(current.as_slice()), current.clone()));
                    self.queued.insert(current);
                    if !known {
                        self.unknown_queued += 1;
                    }
                }
                Some(false) if known => {
                    self.known.insert(current.clone(), true);
                    if self.queued.contains(current.as_slice()) {
                        self.unknown_queued -= 1;
                    } else if self.expanded.contains(current.as_slice()) {
                        pending.extend(scan.commit(current.as_slice())?.parents);
                    }
                }
                Some(_) => {}
            }
        }
        return Some(());
    }
}

/// What:
///  The commits reachable from `include` and not from `stop`,
/// found by a walk ordered by committer time as `git rev-list`
///  orders it.
/// Why:
///   A commit behind a known tip is known,
/// even when it is reached first through another branch,
/// so a merge of an old side
///  branch does not drag the already-pushed history behind it into the scan.
/// The walk ends once only known commits are
///  queued,
/// after a few more pops for clock skew;
/// a wrong order can only keep a known commit in the answer,
/// never drop
///  a new one,
/// because only a real parent chain from a known tip marks a commit known.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newCommits(scan, include, stop): Uint8Array[] | undefined;
/// ```
fn new_commits(
    scan: &mut Scan<'_>,
    include: &[Vec<u8>],
    stop: &[Vec<u8>],
) -> Option<Vec<Vec<u8>>> {
    let mut walk: Walk = Walk {
        queue: std::collections::BinaryHeap::new(),
        known: HashMap::new(),
        queued: HashSet::new(),
        unknown_queued: 0,
        expanded: HashSet::new(),
    };
    for name in stop {
        walk.mark(scan, name.as_slice(), true)?;
    }
    for name in include {
        walk.mark(scan, name.as_slice(), false)?;
    }
    let mut listed: Vec<Vec<u8>> = Vec::new();
    let mut slop: usize = 0;
    let mut popped: usize = 0;
    while let Some((_, name)) = walk.queue.pop() {
        walk.queued.remove(name.as_slice());
        let known: bool = walk.known.get(name.as_slice()).copied().unwrap_or(true);
        if !known {
            walk.unknown_queued -= 1;
            listed.push(name.clone());
        }
        popped += 1;
        if popped > MAX_COMMITS || scan.past_its_deadline() {
            return None;
        }
        let parents: Vec<Vec<u8>> = scan.commit(name.as_slice())?.parents;
        walk.expanded.insert(name);
        for parent in &parents {
            walk.mark(scan, parent.as_slice(), known)?;
        }
        if walk.unknown_queued == 0 {
            slop += 1;
            if slop > CLOCK_SKEW_SLOP {
                break;
            }
        } else {
            slop = 0;
        }
    }
    let mut unknown: Vec<Vec<u8>> = Vec::new();
    for name in listed {
        if !walk.known.get(name.as_slice()).copied().unwrap_or(false) {
            unknown.push(name);
        }
    }
    return Some(unknown);
}

/// What:
///  The names of the LFS objects that the commits reachable from `include` and not from `stop` introduce,
/// sorted,
/// or
///  `None` when the readers cannot answer.
/// Why:
///   The walk follows parents from the tips and does not cross a commit named in `stop`.
/// A pointer counts when its blob is
///  not what a parent holds at the same path,
/// for every parent of a merge,
/// so an object that exists anywhere in the
///  pushed history is found once.
/// A root commit is compared with an empty tree.
/// Every blob read is decoded like
///  `git-lfs` decodes it (`lfs_pointer`);
/// a read that fails,
/// a malformed tree,
/// a missing parent (a shallow clone) or a
///  walk past [`MAX_COMMITS`] or [`SCAN_BUDGET`] declines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newPointerObjects(odb, format, include: Uint8Array[], stop: Uint8Array[]): string[] | undefined;
/// ```
pub fn new_pointer_objects(
    database: &ObjectDatabase,
    format: ObjectFormat,
    include: &[Vec<u8>],
    stop: &[Vec<u8>],
    budget: Duration,
) -> Option<Vec<String>> {
    let mut scan: Scan<'_> = Scan::of(database, format, budget);
    for name in new_commits(&mut scan, include, stop)? {
        let commit: CommitFields = scan.commit(name.as_slice())?;
        let mut parent_trees: Vec<Vec<u8>> = Vec::with_capacity(commit.parents.len());
        for parent in &commit.parents {
            parent_trees.push(scan.commit(parent.as_slice())?.tree);
        }
        scan.compare(commit.tree.as_slice(), parent_trees.as_slice())?;
    }
    let mut objects: BTreeSet<String> = BTreeSet::new();
    for candidate in scan.candidates() {
        if scan.past_its_deadline() {
            return None;
        }
        let blob = database.read(candidate.as_slice())?;
        if blob.kind != ObjectKind::Blob {
            return None;
        }
        match decode_pointer(blob.bytes.as_slice()) {
            Verdict::NotUploadable => {}
            Verdict::Pointer(name) => {
                objects.insert(name);
            }
            Verdict::Decline => return None,
        }
    }
    return Some(objects.into_iter().collect());
}

/// The scan is controlled in the test module.
#[cfg(test)]
#[path = "lfs_pointer_scan_tests.rs"]
mod tests;

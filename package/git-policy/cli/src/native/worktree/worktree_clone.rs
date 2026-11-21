//! What:
//!  Rung 2 of the cheap-worktree ladder:
//! each tracked file of the clean source is cloned (`FICLONE`) into the new worktree
//!  with the source's mode and times,
//! and the source's index is installed with fresh stat fields
//!  for the paths the rung's fence may carry.
//! Why:
//!  A clone shares every data block with the source on btrfs,
//! on XFS with reflink and inside an overlay,
//! which the
//!  snapshot rung cannot reach (overlayfs and XFS have reflink but no subvolumes).
//! It takes no less time than Git's checkout
//!  on btrfs (`doc/planning/cli-git-btrfs-worktree.md`,
//! option B),
//! so it saves disk,
//! not time,
//! and it is the accelerated
//!  rung where a snapshot is impossible.
//! The wrapper's own index writer is `rewrite_index_stat_with`:
//! with each file's
//!  size and modification time kept,
//! only the inode,
//! change time and device of every entry change.
//! A path the conversion fence leaves to Git is installed with no stat data at all
//! (`install_the_index`),
//!  so no step between the copy and Git's own write records it clean.
//! The first failure ends
//!  the rung with its reason,
//! the entries this run created stay (a name cannot be checked and unlinked in one step),
//! and
//!  Git's own checkout completes the worktree and overwrites the tracked ones.
//!  Every source file is opened below a held handle of the source root without following a link,
//! and its descriptor's
//!  metadata,
//! not a path,
//! is compared with the frozen index entry before and after the clone;
//! every destination entry is
//!  created below a held handle of the destination.
//! A directory swapped for a link while the run goes on is an error,
//!  never a detour.
//! The rung is split across siblings because this file is over the line limit:
//! this one holds the run's entry points and the records it answers,
//! `worktree_clone_entry.rs` clones one entry,
//! `worktree_clone_entries.rs` runs the entries on this thread,
//! `worktree_clone_slices.rs` cuts them into slices of the blocking pool,
//! and `worktree_clone_index.rs` installs the index with the fields the clones recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cloned = cloneTrackedBytes(source, destination); // throws a reason
//! installTheIndex(admin, seed, cloned.stats, written, hashLength); // throws a reason
//! ```

use crate::side_by_side_on_the_runtime::AJob;
use crate::subvolume_index::StatFields;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_clone_entries::clone_with;
use crate::worktree_clone_entry::Seams;
pub(crate) use crate::worktree_clone_index::install_the_index;
use crate::worktree_clone_slices::{clone_in_slices, clone_worker_count};
use crate::worktree_clean::stored_stats;
use crate::worktree_snapshot_source::SnapshotSource;
#[cfg(any(test, feature = "measurement-routes"))]
use std::collections::HashSet;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// What:
///  An entry this run created,
/// named by its components below the destination.
/// Why:
///   Nothing is removed on a failure:
/// a name cannot be checked and unlinked in one step,
/// and the entry may by then be another
///  process's,
/// so the run keeps what it made and reports it.
/// The list is what the report names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Created = { names: string[]; directory: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Created {
    /// The components below the destination.
    pub names: Vec<OsString>,
    /// Whether the entry is a directory.
    pub directory: bool,
}

/// The directories this entry's path was walked through,
/// by their components below the destination (the empty list is the destination),
/// each
///  held open so a later file is placed through the directory it was entered by,
/// whatever its name became.
/// Only the levels of the entry being cloned are held;
/// the index is sorted by path,
/// so no directory the walk left is entered again.
pub(crate) type Directories = HashMap<Vec<OsString>, File>;

/// What:
///  What a successful clone run holds:
/// the entries it created.
/// Why:
///   A later step that fails (the tree difference) reports them;
/// Git's own checkout then overwrites the tracked ones,
/// and the
///  rest stay as untracked files for the person to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cloned = { created: Created[] };
/// ```
#[derive(Debug)]
pub struct Cloned {
    /// The entries created,
    /// in creation order.
    created: Vec<Created>,
}

impl Cloned {
    /// The entries the run created,
    /// in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }

    /// The entries the run created,
    /// in creation order,
    /// as the caller's own.
    pub fn into_created(self) -> Vec<Created> {
        return self.created;
    }
}

/// What:
///  What a clone run holds before any index is installed:
/// the entries it created,
/// and the stat fields of every file and link it made,
///  by path.
/// Why:
///   The index install is a step of its own,
/// so the fence beside the clone can answer between the copy and the install,
/// and a path Git
///  still has to write is installed with no stat data (`install_the_index`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClonedBytes = { created: Created[]; stats: Map<string, StatFields> };
/// ```
#[derive(Debug)]
pub struct ClonedBytes {
    /// The entries created,
    /// in creation order.
    created: Vec<Created>,
    /// The stat fields of the objects made,
    /// by path.
    stats: HashMap<Vec<u8>, StatFields>,
}

impl ClonedBytes {
    /// The entries the run created,
    /// in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }

    /// The stat fields of the objects made,
    /// by path.
    pub fn stats(&self) -> &HashMap<Vec<u8>, StatFields> {
        return &self.stats;
    }

    /// The entries the run created,
    /// in creation order,
    /// as the caller's own.
    pub fn into_created(self) -> Vec<Created> {
        return self.created;
    }
}

/// What:
///  The components of a path joined with slashes,
/// as text.
/// Why:
///   The report names a few of the entries the clones created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.join('/')
/// ```
fn joined_names(names: &[OsString]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for name in names {
        parts.push(name.to_string_lossy().into_owned());
    }
    return parts.join("/");
}

/// What:
///  The report of entries a clone run created and leaves in place.
/// Why:
///   The tracked ones are overwritten by the checkout that completes the worktree;
/// any other stays as an untracked file,
/// and
///  the person is told where to look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = leftover_report(created, destination);
/// ```
pub fn leftover_report(created: &[Created], destination: &Path) -> String {
    // The first three entries that are not directories, as `filter` and `take` chose them.
    let mut shown: Vec<String> = Vec::new();
    for entry in created {
        if entry.directory {
            continue;
        }
        if shown.len() == 3 {
            break;
        }
        shown.push(joined_names(entry.names.as_slice()));
    }
    return format!("{} entries the clones created stay in {} (for example {}); the checkout that completes the worktree overwrites the tracked ones, and `git -C {} status` lists any other as untracked", created.len(), destination.display(), shown.join(", "), destination.display());
}

/// The run itself:
/// every tracked file in the slices the host's budget allows;
/// the caller installs the index,
/// and removes `created` when this fails.
fn clone_all<S, T>(source: &SnapshotSource, destination: &File, created: &mut Vec<Created>, mut beside: Option<AJob<S, T>>) -> Result<(HashMap<Vec<u8>, StatFields>, Option<T>), String>
where
    S: Clone + Send + 'static,
    T: Send + 'static,
{
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    let workers: usize = clone_worker_count(entries.len());
    if workers > 1 {
        return clone_in_slices(source, Arc::from(entries), destination, created, workers, beside);
    }
    // A repository below the clone's break-even has no second thread for the copies, so the work beside them runs
    // first: it is the same answer, and on this size running it beside a one-thread copy would buy nothing.
    let beside_answer: Option<T> = beside.take().map(|(state, work)| return work(state));
    let mut directories: Directories = HashMap::new();
    directories.insert(Vec::new(), subject_failure(destination.try_clone(), &"the destination handle", "cannot be kept")?);
    let mut seams: Seams<'_> = Seams { after_lookup: &mut do_nothing, after_clone: &mut do_nothing, before_index: &mut do_nothing };
    return clone_with(source, &mut directories, created, &mut seams).map(|stats| return (stats, beside_answer));
}

/// Log what the index install did.
/// Do nothing;
/// the seams of the clones run this unless a control puts work there.
pub(crate) fn do_nothing() {}

/// What:
///  Clone every tracked file of the frozen index into the held `destination`.
/// On an error nothing is removed:
/// the entries this run created stay and
///  are counted in the reason.
/// Why:
///   The bytes come first and the index install is a step of its own,
/// so the conversion fence can read its inputs and decide while these files are copied,
/// and no
///  index exists until every path has its bytes.
/// The destination may hold files of other writers,
/// and they are not this run's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloned = cloneTrackedBytes(source, destination); // throws a reason
/// ```
pub fn clone_tracked_bytes(source: &SnapshotSource, destination: &File) -> Result<ClonedBytes, String> {
    let mut created: Vec<Created> = Vec::new();
    return match clone_all(source, destination, &mut created, None::<AJob<(), ()>>) {
        Ok((stats, _beside)) => Ok(ClonedBytes { created, stats }),
        Err(reason) => Err(if created.is_empty() { reason } else { format!("{reason}; {} entries it created stay", created.len()) }),
    };
}

/// What:
///  Copy every tracked file while the caller's job runs beside the copies,
///  and give both answers.
/// Why:
///   The conversion fence's reads are the work this carries:
/// they need no runtime,
/// they change nothing the copies read,
///  and the copy is the same wait whether they run beside it or not.
/// The job is a job of the process's blocking pool,
/// so the add's own thread still waits once,
///  and a pool that refuses a thread runs the job in turn before the slices answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [cloned, beside] = cloneTrackedBytesBeside(source, destination, job); // throws a reason
/// ```
pub fn clone_tracked_bytes_beside<S, T>(source: &SnapshotSource, destination: &File, beside: AJob<S, T>) -> Result<(ClonedBytes, Option<T>), String>
where
    S: Clone + Send + 'static,
    T: Send + 'static,
{
    let mut created: Vec<Created> = Vec::new();
    return match clone_all(source, destination, &mut created, Some(beside)) {
        Ok((stats, answered)) => Ok((ClonedBytes { created, stats }, answered)),
        Err(reason) => Err(if created.is_empty() { reason } else { format!("{reason}; {} entries it created stay", created.len()) }),
    };
}

/// What:
///  Clone every tracked file of the frozen index into the held `destination`,
/// and install that index in the held `admin`
///  with the stat fields of every file and link made.
/// Why:
///   This is option A′ of the conversion fence's report
/// (`doc/planning/cli-git-clone-rung-conversion-fence.md`):
/// the fence was decided before the clone,
/// so every copied path is proven to hold its blob and
///  the installed index may vouch for all of them.
/// It is kept only for a measurement build (`measurement-routes`);
/// the shipped shape
///  is `clone_tracked_bytes` and `install_the_index` around a fence that runs beside the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloned = cloneTrackedFiles(source, destination, admin); // throws a reason
/// ```
#[cfg(any(test, feature = "measurement-routes"))]
pub fn clone_tracked_files(source: &SnapshotSource, destination: &File, admin: &File) -> Result<Cloned, String> {
    let cloned: ClonedBytes = clone_tracked_bytes(source, destination)?;
    install_the_index(admin, source.clean.index_bytes.as_slice(), cloned.stats(), &HashSet::new(), source.clean.hash_length)?;
    return Ok(Cloned { created: cloned.created });
}

/// Clone rung controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_tests.rs"]
mod tests;

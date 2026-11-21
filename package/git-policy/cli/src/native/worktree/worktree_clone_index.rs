//! What:
//!  The clone rung's index:
//! the stat fields the clones recorded,
//! by path,
//!  none for the paths the conversion fence leaves to Git,
//!  and the name it waits under until Git has written them.
//! Why:
//!  The index is the one record that can call a path clean,
//! so it must refuse the written paths' stat data and take nothing else away,
//!  and it must not exist under the worktree's own name while Git still has those paths to write
//! (`worktree_clone_conversion_beside_the_clone.rs`);
//! the module lives here so `worktree_clone.rs` stays under the
//!  line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! installTheIndex(admin, seed, stats, written, hashLength); // throws a reason
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::diagnostic_log::debug;
use crate::subvolume_index::{StatFields, StatLookup};
#[cfg(any(test, feature = "measurement-routes"))]
use crate::subvolume_index_files::install_index_in;
use crate::subvolume_index_files::{discard_index_under_a_name, publish_index_under_a_name, write_index_under_a_name};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "cloneIndex";

/// The name the rung's index is written under while the paths Git must write are still unwritten.
/// Why:
///   A worktree's own index is what makes it look whole to the recovery and to `git status`,
/// so the name is taken only once the write step is done (`publish`).
const PENDING_INDEX: &str = "index.pending";

/// What:
///  The stat fields the clones recorded,
/// looked up by path for the index the new worktree takes,
///  and none for the paths the conversion fence leaves to Git.
/// Why:
///   Each clone recorded the fields of the file it made,
/// and the index install asks for them by the entry's path;
/// a path Git still has to write keeps the zeroed stat data its seed entry holds,
///  so nothing between the copy and Git's write records it clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StatsMadeByTheClones implements StatLookup { statNow(path) { return this.written.has(path) ? undefined : this.stats.get(path); } }
/// ```
struct StatsMadeByTheClones<'a> {
    /// The fields by path.
    stats: &'a HashMap<Vec<u8>, StatFields>,
    /// The paths Git writes,
    /// whose entries keep no stat data.
    written: &'a HashSet<Vec<u8>>,
}

impl StatLookup for StatsMadeByTheClones<'_> {
    fn stat_now(&mut self, path: &[u8]) -> Option<StatFields> {
        if self.written.contains(path) {
            return None;
        }
        return self.stats.get(path).copied();
    }
}

/// What:
///  Install `seed` as the index of the new worktree whose administrative directory is the open handle `admin`,
/// with the stat fields the clones recorded,
///  and none for the paths in `written`.
/// Why:
///   The seed the caller passes has those entries' stat data already zeroed
/// (`subvolume_index::with_zeroed_stat`),
/// and the install writes no fresh fields over them,
/// so the published index never vouches for a path Git still has to write.
/// The step is the install's own,
/// so the fence before the clone and the fence beside it
///  record it alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installTheIndex(admin, seed, stats, written, hashLength); // throws a reason
/// ```
#[cfg(any(test, feature = "measurement-routes"))]
pub(crate) fn install_the_index(admin: &File, seed: &[u8], stats: &HashMap<Vec<u8>, StatFields>, written: &HashSet<Vec<u8>>, hash_length: usize) -> Result<String, String> {
    let index_entered: bool = begin_step("cloneIndex");
    let installed: Result<String, String> = install_index_in(admin, seed, &mut StatsMadeByTheClones { stats, written }, hash_length);
    end_step(index_entered);
    let report: String = installed?;
    debug(TAG, report.as_str());
    return Ok(report);
}

/// What:
///  The index this rung wrote under its own name,
/// waiting for the worktree's name.
/// Why:
///   The file is held open from the moment it is written,
/// so publishing it links the object made here and never a name another process took;
/// a rung that ends first removes it (`discard`),
///  and the worktree's own `index` is then Git's checkout's to write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingIndex = { held: File; path: Path };
/// ```
pub(crate) struct PendingIndex {
    /// The open file the index was written into.
    held: File,
    /// Where it waits,
    /// in the worktree's administrative directory:
    /// the path Git is pointed at with `GIT_INDEX_FILE` while it writes.
    path: PathBuf,
}

impl PendingIndex {
    /// Where the index waits,
    /// as a path for the environment a Git run is given.
    pub(crate) fn path_in(&self) -> &Path {
        return self.path.as_path();
    }

    /// What:
    ///  Give the index the worktree's own name,
    /// after the last write through Git,
    /// so the stat data it holds describes what is on disk.
    /// Why:
    ///   This is the moment the worktree starts vouching for its files;
    /// before it,
    /// no index exists there at all,
    ///  so a kill leaves the unfinished shape the recovery already names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pending.publish(admin); // throws a reason
    /// ```
    pub(crate) fn publish(self, admin: &File) -> Result<(), String> {
        return publish_index_under_a_name(admin, &self.held, OsStr::new(PENDING_INDEX));
    }

    /// Remove the index written under its own name,
    /// when the rung ends before it is published.
    pub(crate) fn discard(self, admin: &File) {
        discard_index_under_a_name(admin, &self.held, OsStr::new(PENDING_INDEX));
    }
}

/// What:
///  Write the destination's index under the rung's own name,
/// with the stat fields the clones recorded and none for the paths in `written`,
///  and leave the worktree without an index.
/// Why:
///   The fence beside the clone publishes no index until every path Git must write has its bytes,
/// because a published index whose entry's clean form equals the blob makes `git status` skip the file
///  and refresh the entry instead of reporting it (issue 6323,
/// reached by a kill);
/// so the seed goes to `index.pending`,
/// Git is pointed at that file while it writes,
///  and `PendingIndex::publish` renames it afterwards.
/// The step is the install's own,
/// as for `install_the_index`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pending = installTheIndexBesideTheWrite([admin, adminPath], seed, stats, written, hashLength); // throws a reason
/// ```
pub(crate) fn install_the_index_beside_the_write((admin, admin_path): (&File, &Path), seed: &[u8], stats: &HashMap<Vec<u8>, StatFields>, written: &HashSet<Vec<u8>>, hash_length: usize) -> Result<PendingIndex, String> {
    let index_entered: bool = begin_step("cloneIndex");
    let written_index = write_index_under_a_name(admin, seed, &mut StatsMadeByTheClones { stats, written }, hash_length, OsStr::new(PENDING_INDEX));
    end_step(index_entered);
    let (held, report): (File, String) = written_index?;
    debug(TAG, report.as_str());
    return Ok(PendingIndex { held, path: admin_path.join(PENDING_INDEX) });
}

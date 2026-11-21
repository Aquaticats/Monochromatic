//! What:
//!  The install of the clone rung's index:
//! the stat fields the clones recorded,
//! by path,
//!  and none for the paths the conversion fence leaves to Git.
//! Why:
//!  The install is the one step where a path can be recorded clean,
//! so it must refuse the written paths' stat data and take nothing else away;
//! it lives here so `worktree_clone.rs` stays under the
//!  line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! installTheIndex(admin, seed, stats, written, hashLength); // throws a reason
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::diagnostic_log::debug;
use crate::subvolume_index::{StatFields, StatLookup};
use crate::subvolume_index_files::install_index_in;
use std::collections::{HashMap, HashSet};
use std::fs::File;

/// Tag used for diagnostics from this module.
const TAG: &str = "cloneIndex";

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
pub(crate) fn install_the_index(admin: &File, seed: &[u8], stats: &HashMap<Vec<u8>, StatFields>, written: &HashSet<Vec<u8>>, hash_length: usize) -> Result<String, String> {
    let index_entered: bool = begin_step("cloneIndex");
    let installed: Result<String, String> = install_index_in(admin, seed, &mut StatsMadeByTheClones { stats, written }, hash_length);
    end_step(index_entered);
    let report: String = installed?;
    debug(TAG, report.as_str());
    return Ok(report);
}

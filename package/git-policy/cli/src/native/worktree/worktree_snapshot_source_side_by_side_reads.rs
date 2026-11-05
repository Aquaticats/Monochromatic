//! What:
//!  The reads that run beside the cleanliness check of a snapshot source: the Git configuration on one thread and the ignore rule
//!  for the trash location on another, while the calling thread checks the files.
//! Why:
//!  The three read different things and write nothing, so they run side by side; each thread's time is added to the work
//!  record of the add by name (`add_timing.rs`). The functions sit in a file of their own so `worktree_snapshot_source.rs`
//!  stays under the file-size limit and this file needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [config, [ignored, checked]] = await Promise.all([readConfigurationTimed(args), checkTrashAndCleanliness(args)]);
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
use crate::side_by_side_threads::run_beside_caller;
use crate::transaction_git::GitContext;
use crate::worktree_clean::{CleanRefusal, CleanSource, check_clean};
use std::path::Path;

/// What:
///  The Git configuration of the source, read on a thread of its own and timed as the work `readConfig`.
/// Why:
///   The read writes nothing, so it runs beside the cleanliness check; the add timing record sums the work of threads by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const read = readConfigSnapshot(context, root); endWork('readConfig', began);
/// ```
pub(crate) fn read_configuration_timed((context, root): (&GitContext, &Path)) -> Option<ConfigSnapshot> {
    let began: std::time::Instant = begin_work();
    let read: Option<ConfigSnapshot> = read_config_snapshot(context, root);
    end_work("readConfig", began);
    return read;
}

/// Whether the place left-out directories are set aside is ignored, for a main checkout only, timed as the work `trashIgnored`.
fn trash_location_ignored_timed((context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)) -> bool {
    let began: std::time::Instant = begin_work();
    let ignored: bool = git_dir == common_dir && crate::worktree_trash::trash_location_is_ignored(context, root);
    end_work("trashIgnored", began);
    return ignored;
}

/// The cleanliness check of the source, which runs on the calling thread beside the two reads on threads of their own.
fn clean_check_of_source((context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)) -> Result<CleanSource, CleanRefusal> {
    return check_clean(context, root, git_dir, common_dir);
}

/// What the trash-location read on its thread and the cleanliness check on the calling thread found.
pub(crate) type TrashAndCleanVerdicts = (std::thread::Result<bool>, Result<CleanSource, CleanRefusal>);

/// What:
///  Read whether the trash location is ignored on a thread, and check the cleanliness of the source here, beside it.
/// Why:
///   The two read different things and write nothing; the verdicts are read in the order the checks have always had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [ignored, checked] = await Promise.all([trashLocationIgnored(args), cleanCheckOfSource(args)]);
/// ```
pub(crate) fn check_trash_and_cleanliness(state: (&GitContext, &Path, &Path, &Path)) -> TrashAndCleanVerdicts {
    return run_beside_caller((state, trash_location_ignored_timed), (state, clean_check_of_source));
}

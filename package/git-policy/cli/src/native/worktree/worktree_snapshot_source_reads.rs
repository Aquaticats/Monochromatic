//! What:
//!  The reads the source's checks make,
//! each timed as the work the add timing record names:
//! the Git configuration of the source and the reasons it rules the source out,
//! and the cleanliness of its files.
//! Why:
//!  The read of the configuration writes nothing,
//! so it runs beside the cleanliness check,
//! and the record sums the checks' work by name;
//! the reasons the configuration gives are words the caller refuses with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const config = readConfigurationTimed({ context, root });
//! const checked = cleanCheckOfSource({ context, root, gitDir, commonDir });
//! ```

use crate::add_timing::{
    begin_work,
    end_work,
};
use crate::config_snapshot::{
    ConfigEntry,
    ConfigSnapshot,
    read_config_snapshot,
    read_config_snapshot_awaiting,
};
use crate::transaction_git::GitContext;
use crate::worktree_clean::{
    CleanRefusal,
    CleanSource,
    check_clean,
};
use std::path::Path;

/// The configuration key that turns a sparse checkout on;
/// Git copies the patterns into a new worktree,
/// a snapshot cannot.
const SPARSE_KEY: &str = "core.sparsecheckout";

/// What:
///  The reason the configuration rules the source out,
/// or nothing.
/// Why:
///   Git copies the filtered worktree configuration and the sparse-checkout patterns into a new worktree,
/// and conversion
///  settings that live only in a worktree would disagree between the two worktrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configurationRefusal(entries: ConfigEntry[]): string | undefined;
/// ```
pub(crate) fn configuration_refusal(entries: &[ConfigEntry]) -> Option<String> {
    for entry in entries {
        if entry.key == SPARSE_KEY {
            return Some(String::from("core.sparseCheckout is set"));
        }
        if entry.scope == "worktree" {
            return Some(format!(
                "the worktree-specific configuration sets {}",
                entry.key
            ));
        }
    }
    return None;
}

/// What:
///  The Git configuration of the source,
/// timed as the work `readConfig`.
/// Why:
///   The read writes nothing,
/// so it runs beside the cleanliness check;
/// the add timing record sums the work of the checks by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const read = readConfigSnapshot(context, root); endWork('readConfig', began);
/// ```
pub(crate) fn read_configuration_timed(
    (context, root): (&GitContext, &Path)
) -> Option<ConfigSnapshot> {
    let began: std::time::Instant = begin_work();
    let read: Option<ConfigSnapshot> = read_config_snapshot(context, root);
    end_work("readConfig", began);
    return read;
}

/// What:
///  `read_configuration_timed` with the Git child awaited on the runtime.
/// Why:
///   The read is one of the three the source check makes on a task of the runtime,
/// which starts no child through the
///  synchronous face;
/// the run's bound and quiet window are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const read = await readConfigSnapshotAwaiting(context, root); endWork('readConfig', began);
/// ```
pub(crate) async fn read_configuration_timed_awaiting(
    (context, root): (&GitContext, &Path)
) -> Option<ConfigSnapshot> {
    let began: std::time::Instant = begin_work();
    let read: Option<ConfigSnapshot> = read_config_snapshot_awaiting(context, root).await;
    end_work("readConfig", began);
    return read;
}

/// The cleanliness check of the source,
/// which runs beside the two reads.
pub(crate) fn clean_check_of_source(
    (context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)
) -> Result<CleanSource, CleanRefusal> {
    return check_clean(context, root, git_dir, common_dir);
}

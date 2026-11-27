//! What:
//!  Make one directory tree match another,
//! in process:
//! copy what is missing or changed (cloning file data where the
//!  filesystem can),
//! optionally delete what the source no longer has,
//! or only add what is newer in the source.
//! Why:
//!  This is the work `cp --archive --reflink=always`,
//! `rsync --archive --delete` and `rsync --archive --update` did in
//!  the migration script,
//! without starting those programs.
//! One idempotent function serves all three passes,
//! so a
//!  pass interrupted at any point is repeated safely:
//! an entry already equal (same type,
//! size,
//! modification time and
//!  permission bits) is left alone.
//! Another command's stage container is left out of every pass,
//! and so is a folder that held nothing but one:
//! the container holds another worktree's ignored state while its own copy fills it,
//! and nothing names it in the tree this pass builds.
//! A container is a directory,
//! so a file or a link whose name merely begins the stage prefix is the source's own entry
//!  and is copied like any other,
//! and a deletion pass leaves a container in the destination where it stands,
//! because it belongs to the command filling it and not to this pass.
//! The walk uses a work stack,
//! not recursion,
//! so a deep tree cannot overflow the stack;
//!  a directory's own attributes are applied after its children,
//! because adding a child changes its modification time.
//!  Files this user may not read (for example root-owned mode 600 logs) are left where they are and listed;
//! any other
//!  failure,
//! and a directory this user cannot list,
//! stops the run (the rule of the 2026-11-01 real run:
//! copying around
//!  an unreadable directory with deletion on would erase its subtree from the copy).
//!  Every entry is written under a temporary name in its directory and renamed over its name when complete,
//! so a kill
//!  never leaves a half-written file at a name a later pass would take for a newer one.
//! The destination is reached
//!  through open directory descriptors (`subvolume_anchor.rs`),
//! so a directory replaced by a link during the pass
//!  redirects nothing.
//! An update-only pass never removes a destination entry that may hold newer work:
//! a directory,
//! or
//!  an entry of another type that is not older than the source,
//! is kept and listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stats = await syncTree({ platform, source, destination, delete: true, updateOnly: false, excluded: ['.git/index.lock'] });
//! ```
//!
//! The walk of one directory,
//! and the test-only callback a control installs at it,
//! are `subvolume_sync_directory_visit.rs`;
//! the dispatch of one source child by its type is `subvolume_sync_source_child.rs`.

#[cfg(test)]
pub(crate) use super::subvolume_sync_directory_visit::watch_entries;
use crate::subvolume_error::{
    Interrupt,
    MigrateError,
};
use crate::subvolume_platform::Platform;
use crate::subvolume_sync_attrs::preserve_on_descriptor;
use crate::subvolume_sync_pass_state_and_checks::*;
pub use crate::subvolume_sync_pass_state_and_checks::{
    SyncOptions,
    SyncStats,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::fs::Metadata;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Run one pass from `source` to `destination` (both existing directories).
/// Why:
///   See the module description;
/// the work stack replaces recursion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stats = await syncTree({ platform, source, destination, options, interrupt });
/// ```
pub fn sync_tree(
    platform: &dyn Platform,
    source: &Path,
    destination: &Path,
    options: SyncOptions<'_>,
    interrupt: &Interrupt,
) -> Result<SyncStats, MigrateError> {
    let mut pass: Pass<'_> = Pass {
        platform,
        source,
        destination,
        options,
        interrupt,
        stats: SyncStats::default(),
        links: HashMap::new(),
        stage_holders: HashSet::new(),
    };
    let root_metadata: Metadata =
        MigrateError::from_io_result(std::fs::symlink_metadata(source), "examine", source)?;
    let mut stack: Vec<Work> = vec![
        Work::Finish {
            relative: PathBuf::new(),
            metadata: root_metadata,
            created: false,
        },
        Work::Visit(PathBuf::new()),
    ];
    while let Some(work) = stack.pop() {
        match work {
            Work::Visit(relative) => pass.visit(relative.as_path(), &mut stack)?,
            Work::Finish {
                relative,
                metadata,
                created,
            } => {
                if created && pass.forgets_a_folder_of_stage_containers_only(relative.as_path())? {
                    continue;
                }
                let Some(anchor) = pass.open_destination(relative.as_path())? else {
                    continue;
                };
                let shown: PathBuf = pass.shown(relative.as_path());
                attribute_result(
                    preserve_on_descriptor(
                        source.join(&relative).as_path(),
                        anchor.file(),
                        &metadata,
                        !options.update_only,
                        &mut pass.stats.losses,
                    ),
                    shown.as_path(),
                )?;
            }
        }
    }
    pass.stats.unreadable.sort();
    pass.stats.conflicts.sort();
    pass.stats.conflicts.dedup();
    return Ok(pass.stats);
}

/// Tree sync controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sync_tests.rs"]
mod tests;

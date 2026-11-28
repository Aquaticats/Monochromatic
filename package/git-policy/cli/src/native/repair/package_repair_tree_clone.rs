//! What:
//!  Cloning one directory tree for a worktree repair:
//! every file cloned (or copied when the filesystem cannot clone),
//! every
//!  directory recreated with its mode,
//! every link recreated with its exact target,
//! modes and times kept.
//! Why:
//!  Cargo keeps its build state outside the worktree when the person's configuration says so,
//! so a copy of the worktree's
//!  ignored files does not carry it.
//! The state is reused by the new worktree's registry and git dependencies,
//! and Cargo
//!  decides what is fresh from file times,
//! so the clone keeps them.
//! The destination must not exist:
//! nothing is overwritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cloned = await cloneTree({ source, destination, workers });
//! ```
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_tree_clone_entries_and_places.rs`:
//!   what a walk of the source finds,
//!   what a clone made,
//!   and which places its steps take.
//! - `package_repair_tree_clone_walk.rs`:
//!   the walk,
//!   the destination's directories,
//!   and the modes they take last.
//! - `package_repair_tree_clone_placing.rs`:
//!   placing one entry and one slice of entries.

/// Import the process's runtime,
/// which the synchronous face enters.
use crate::lazy_runtime::run_on_runtime;
/// Import what a finished clone holds,
/// which of the process's places its steps take,
/// the entry kinds it reads,
/// the size below which it stays on one thread,
/// and the runner of one step.
pub use crate::package_repair_tree_clone_entries_and_places::Cloned;
/// Import what a walk of the source finds at one path,
/// which of the process's places its steps take,
/// the size below which it stays on one thread,
/// and the runner of one step,
/// all of which stay inside this folder.
pub(super) use crate::package_repair_tree_clone_entries_and_places::{
    Entry,
    Kind,
    PARALLEL_MIN_ENTRIES,
    PlacesOfAClone,
    run_a_clone_step,
};
/// Import the slices one clone places side by side.
pub(super) use crate::package_repair_tree_clone_placing::{
    ThePlacing,
    place_slices_awaiting,
};
/// Import the walk every clone starts with,
/// and the two steps that follow it.
pub(super) use crate::package_repair_tree_clone_walk::{
    TheWalk,
    set_the_modes_of,
    walk_and_make_of,
};
/// Import the borrowed and owned filesystem paths a clone names.
use std::path::{
    Path,
    PathBuf,
};
/// Import the shared entry list the jobs read from.
use std::sync::Arc;
/// What:
///  Clone `source` into the new directory `destination` with every wait awaited on the runtime,
///  as the service's prewarm of a new worktree runs it,
///  with the places slow work may take.
/// Why:
///  `clone_tree` enters the runtime,
/// which a caller that is itself a task cannot do
/// (`RuntimeError::Nested`),
/// and the clone of a build directory is one
///  of the waits a task of the Cargo workspace repair makes.
/// The prewarm is the one caller of this face and it is slow work nobody waits for,
///  so the clone leaves one place of the process's bound free of itself:
/// prewarm waves run four wide and the service's pool has four places,
/// and a wave of clones that took every place held the round's own storage steps for as long as it ran
///  (`landing/service_work_slow_lane.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cloneTreeAwaiting(source: string, destination: string, workers: number): Promise<Cloned>;
/// ```
pub(crate) async fn clone_tree_awaiting(
    source: &Path,
    destination: &Path,
    workers: usize,
) -> Result<Cloned, String> {
    return clone_tree_within(source, destination, workers, PlacesOfAClone::SlowWork).await;
}

/// What:
///  `clone_tree_awaiting`'s work,
///  with the places its steps take given.
/// Why:
///   The two faces of a clone differ only in whose work they are,
/// so the walk,
///  the slices and the modes are one account.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cloneTreeWithin(source, destination, workers, places): Promise<Cloned>;
/// ```
async fn clone_tree_within(
    source: &Path,
    destination: &Path,
    workers: usize,
    places: PlacesOfAClone,
) -> Result<Cloned, String> {
    let walked = run_a_clone_step(
        places,
        (source.to_path_buf(), destination.to_path_buf()),
        walk_and_make_of,
    )
    .await;
    let TheWalk {
        entries,
        mut left_out,
        directories,
        root_mode,
    }: TheWalk = match walked {
        Ok(Ok(found)) => found,
        // The walk itself failed and named the entry it could not read; nothing of the clone was made.
        Ok(Err(reason)) => return Err(reason),
        // Nothing of the clone was made, and the runtime's own reason is why.
        Err(refused) => {
            return Err(format!(
                "{} could not be cloned ({refused})",
                source.display()
            ));
        }
    };
    let mut leaves: Vec<usize> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.kind != Kind::Directory {
            leaves.push(index);
        }
    }
    let used: usize = if leaves.len() < PARALLEL_MIN_ENTRIES {
        1
    } else {
        workers.max(1)
    };
    let slice_length: usize = leaves.len().div_ceil(used).max(1);
    let slices: Vec<Vec<usize>> = leaves.chunks(slice_length).map(<[usize]>::to_vec).collect();
    let shared: Arc<[Entry]> = Arc::from(entries);
    let placing: ThePlacing = ThePlacing {
        entries: Arc::clone(&shared),
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
    };
    let outcomes: Vec<Result<Vec<PathBuf>, String>> =
        place_slices_awaiting(placing, slices, places).await;
    for outcome in outcomes {
        left_out.extend(outcome?);
    }
    left_out.sort();
    // Modes last, so a read-only directory does not stop its own contents from being placed.
    let modes = run_a_clone_step(
        places,
        (
            source.to_path_buf(),
            destination.to_path_buf(),
            Arc::clone(&shared),
            directories,
            root_mode,
        ),
        set_the_modes_of,
    )
    .await;
    match modes {
        Ok(Ok(())) => {}
        // A directory or the destination could not take its mode, and the message names which.
        Ok(Err(reason)) => return Err(reason),
        // Every entry is in the destination and the modes are the ones its creation gave, which the words say.
        Err(refused) => {
            return Err(format!(
                "{} could not take its modes ({refused})",
                destination.display()
            ));
        }
    }
    let mut skipped_files: usize = 0;
    for entry in shared.iter() {
        if entry.kind != Kind::Directory && left_out.contains(&entry.relative) {
            skipped_files += 1;
        }
    }
    return Ok(Cloned {
        made: shared.len() - skipped_files,
        left_out,
    });
}

/// What:
///  Clone `source` into the new directory `destination`.
/// Why:
///   The count of entries made is returned for the log;
/// a failure names the entry that could not be made,
/// and leaves what was
///  made in place for the person to remove with the destination.
///  The callers of this face are synchronous (the Cargo repair of a worktree inside the repair the add runs),
/// so it
///  enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
/// and the Cargo repair's own
///  items await `clone_tree_awaiting` instead.
/// A synchronous caller is a command a person waits for,
/// so its clone takes the process's general places and no place is kept for it:
/// only the prewarm's own clone,
/// which nobody waits for,
/// leaves a place free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneTree(args: { source: string; destination: string; workers: number }): Cloned;
/// ```
pub fn clone_tree(
    source: &Path,
    destination: &Path,
    workers: usize,
) -> Result<Cloned, String> {
    return match run_on_runtime(clone_tree_within(
        source,
        destination,
        workers,
        PlacesOfAClone::EveryJob,
    )) {
        Ok(cloned) => cloned,
        // Nothing of the clone was made, and the runtime's own reason is why.
        Err(refusal) => Err(format!(
            "{} could not be cloned ({refusal})",
            source.display()
        )),
    };
}

/// Tree clone controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tree_clone_tests.rs"]
mod tests;

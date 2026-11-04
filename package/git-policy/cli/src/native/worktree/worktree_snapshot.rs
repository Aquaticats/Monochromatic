//! What:
//!  The mechanics of rung 1 of the cheap-worktree ladder: a snapshot of the source subvolume taken at a stage name beside
//!  the destination, made into a worktree there (its own `.git` pointer, no stubs of nested worktrees, the source's index
//!  with refreshed stat fields), and swapped with the destination in one atomic step.
//! Why:
//!  The snapshot shares every data block with the source and carries its ignored state at no cost
//!  (`doc/planning/cli-git-btrfs-worktree.md`). It is made at a name Git never hands out, so the destination is
//!  complete or untouched at every instant: `RENAME_EXCHANGE` puts the finished subvolume at the destination and the
//!  empty directory real Git made at the stage name, which is then removed. A source that changed while the snapshot was
//!  taken is caught by comparing the snapshot's own files with the index entries copied from the source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await takeSnapshot({ source, plan, stage, admin }); // throws a reason; the caller destroys the stage
//! await putInPlace({ stage, destination, mode });
//! ```

use crate::add_timing::step as timed;
use crate::fs_beneath::{ObjectId, exchange_at, identity_at, open_directory_at, open_directory_beneath, open_file_at, open_root, plain_components, remove_at, rename_at};
use crate::fs_btrfs_ioctl::snapshot_create_at;
use crate::subvolume_index_files::install_index_in;
use crate::subvolume_remove::remove_tree;
use crate::worktree_add_plan::AddPlan;
use crate::subvolume_index::StatFields;
use crate::worktree_clean::{CleanRefusal, StatCursor, StatEntry, stored_stats, tree_fields_beneath};
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash::{Pruned, TrashManifest, prune_snapshot};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// What:
///  Remove one stub from the snapshot below the held handle of its root: the directory `path` names (a path inside the
///  source), after its `.git` pointer when `pointer` is set.
/// Why:
///   Every directory on the way is opened without following a link and on the snapshot's own device, so a registered
///  path that reaches the snapshot through a link (a link left where a directory was moved) is an error and never an
///  unlink in the tree the link points at. A nested subvolume's stub is removed only as an empty directory; only the
///  destination's own directory, which Git made, holds a pointer file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeStub(root, sourceRoot, path, pointer);
/// ```
fn remove_stub(root: &File, source_root: &Path, path: &Path, pointer: bool) -> Result<(), String> {
    let Ok(relative) = path.strip_prefix(source_root) else { return Ok(()) };
    let shown: String = path.display().to_string();
    let names: Vec<OsString> = plain_components(relative).map_err(|error| { return format!("{shown} is not a plain path inside the source ({error})"); })?;
    let Some((name, parents)) = names.split_last() else { return Ok(()) };
    let gone = |error: &std::io::Error| { return error.kind() == std::io::ErrorKind::NotFound; };
    let parent: File = match open_directory_beneath(root, parents) {
        Ok(found) => found,
        Err(error) if gone(&error) => return Ok(()),
        Err(error) => return Err(format!("{shown} is reached through a link or another mount in the snapshot ({error})")),
    };
    if pointer {
        match open_directory_at(&parent, name.as_os_str()) {
            Ok(stub) => match remove_at(&stub, OsStr::new(".git"), false) {
                Ok(()) => {}
                Err(error) if gone(&error) => {}
                Err(error) => return Err(format!("{shown}/.git cannot be removed from the snapshot ({error})")),
            },
            Err(error) if gone(&error) => return Ok(()),
            Err(error) => return Err(format!("{shown} cannot be entered in the snapshot ({error})")),
        }
    }
    return match remove_at(&parent, name.as_os_str(), true) {
        Ok(()) => Ok(()),
        Err(error) if gone(&error) => Ok(()),
        Err(error) => Err(format!("{shown} cannot be removed from the snapshot as an empty directory ({error})")),
    };
}

/// What:
///  Remove the stub that each nested worktree leaves in the snapshot, and the destination's own directory when it lies
///  inside the source.
/// Why:
///   A nested worktree that is a subvolume is an empty directory in the snapshot; the destination was made by Git before
///  the snapshot, so a source that contains it holds a directory with Git's `.git` file there. Neither belongs in the new
///  worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeStubs({ stage, sourceRoot, nested, destination });
/// ```
fn remove_stubs(stage: &Path, source: &SnapshotSource, plan: &AddPlan) -> Result<(), String> {
    let root: File = open_root(stage).map_err(|error| { return format!("the stage cannot be opened ({error})"); })?;
    for nested in &source.nested {
        remove_stub(&root, source.root.as_path(), nested.as_path(), false)?;
    }
    return remove_stub(&root, source.root.as_path(), plan.destination.as_path(), true);
}

/// What:
///  Put the new worktree's `.git` pointer into the snapshot in place of the source's `.git` (a directory for a main
///  checkout, a pointer file for a linked worktree).
/// Why:
///   The bytes are the ones real Git wrote for the new worktree, so the pointer is exactly Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await replaceGitPointer(stage, gitFileOfNewWorktree);
/// ```
fn replace_git_pointer(stage: &Path, pointer: &[u8]) -> Result<(), String> {
    let path = stage.join(".git");
    remove_tree(path.as_path()).map_err(|error| { return format!("the source's .git cannot be removed from the snapshot ({error})"); })?;
    return std::fs::write(path.as_path(), pointer).map_err(|error| { return format!("the .git pointer cannot be written ({error})"); });
}

/// What:
///  Check the snapshot against the source's index, then install that index in the new worktree's administrative
///  directory with stat fields refreshed and the accelerator extensions of the source dropped.
/// Why:
///   Every tracked file of the snapshot must equal its entry, which proves the snapshot and the index describe the same
///  state whatever the source did meanwhile. The extensions (`UNTR`, `FSMN`) name the source's worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await installIndex({ source, stage, admin });
/// ```
fn install_index(source: &SnapshotSource, stage: &Path, admin: &File, own_index: Option<&[u8]>) -> Result<(), String> {
    let root: File = open_root(stage).map_err(|error| { return format!("the stage cannot be opened ({error})"); })?;
    // A clean source proved its index equal to its files, so it is compared with the snapshot. A source with changes has no such
    //  proof: the snapshot's own index, one instant with its files, is installed as it is and the restore settles the rest.
    let bytes: &[u8] = own_index.unwrap_or(source.clean.index_bytes.as_slice());
    let entries: Vec<StatEntry> = if own_index.is_none() { stored_stats(bytes, source.clean.hash_length).ok_or_else(|| { return String::from("the source's index changed shape"); })? } else { Vec::new() };
    // One walk of the snapshot proves it equal to the source's index and returns the stat fields the new index takes: every
    //  file is looked at once, and the fields are the ones the verdict was given on.
    let walked: Vec<StatFields> = timed("walkSnapshot", || { return tree_fields_beneath(&root, entries.as_slice(), source.clean.index_seconds); })
        .map_err(|refusal: CleanRefusal| { return format!("the source changed while it was copied ({})", refusal.reason); })?;
    let known: std::collections::HashMap<&[u8], StatFields> = entries.iter().map(|entry: &StatEntry| { return entry.path.as_slice(); }).zip(walked).collect();
    // An entry the walk did not cover (a source with changes has no walk) is looked at through a cursor that keeps its directory.
    let mut cursor: StatCursor<'_> = StatCursor::new(&root);
    let report: String = timed("installIndexFile", || { return install_index_in(admin, bytes, &mut |path: &[u8]| { return known.get(path).copied().or_else(|| { return cursor.fields(path); }); }, source.clean.hash_length); })?;
    crate::diagnostic_log::debug("takeSnapshot", report.as_str());
    return Ok(());
}

/// What:
///  The bytes of the index inside the snapshot's own `.git`, read below handles that follow no link.
/// Why:
///   A source with uncommitted changes has no index that was proven equal to its files; the copy's index and the copy's files
///  are one instant, whatever the source does next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = await readOwnIndex(stage); // throws a reason
/// ```
fn read_own_index(stage: &Path) -> Result<Vec<u8>, String> {
    let root: File = open_root(stage).map_err(|error| { return format!("the stage cannot be opened ({error})"); })?;
    let git_dir: File = open_directory_at(&root, OsStr::new(".git")).map_err(|error| { return format!("the snapshot has no .git directory ({error})"); })?;
    let mut index: File = open_file_at(&git_dir, OsStr::new("index")).map_err(|error| { return format!("the snapshot has no index ({error})"); })?;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut index, &mut bytes).map_err(|error| { return format!("the snapshot's index is unreadable ({error})"); })?;
    return Ok(bytes);
}

/// What:
///  Take the snapshot at `stage` and make it the new worktree there; the caller destroys `stage` on an error.
/// Why:
///   Each step is one the later steps depend on; the first failure ends the rung with its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeSnapshot({ source, plan, stage, admin, pointer });
/// ```
pub fn take_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, admin: &File, pointer: &[u8]) -> Result<Snapshotted, SnapshotFailure> {
    timed("snapshotIoctl", || { return snapshot_create_at(source.root.as_path(), stage); }).map_err(|error| { return SnapshotFailure { reason: format!("the snapshot was refused ({error})"), stage: None }; })?;
    // From here the stage exists and this process made it; its identity is what a later removal checks.
    let made: Option<ObjectId> = open_root(stage.parent().unwrap_or(stage)).ok().and_then(|parent: File| { return identity_at(&parent, stage.file_name()?).ok(); });
    let failed = |reason: String| -> SnapshotFailure { return SnapshotFailure { reason, stage: made }; };
    std::fs::set_permissions(stage, std::fs::Permissions::from_mode(0o700)).map_err(|error| { return failed(format!("the stage cannot be made private ({error})")); })?;
    // The snapshot's own index is read before its `.git` goes: it is the index of the same instant as the files.
    let own_index: Option<Vec<u8>> = if source.dirty.is_some() { Some(timed("readOwnIndex", || { return read_own_index(stage); }).map_err(failed)?) } else { None };
    let pruned: Pruned = timed("prune", || { return prune_snapshot(stage, source); }).map_err(failed)?;
    timed("removeStubs", || { return remove_stubs(stage, source, plan); }).map_err(failed)?;
    timed("replacePointer", || { return replace_git_pointer(stage, pointer); }).map_err(failed)?;
    timed("installIndex", || { return install_index(source, stage, admin, own_index.as_deref()); }).map_err(failed)?;
    let stage_id: ObjectId = made.ok_or_else(|| { return failed(String::from("the stage cannot be examined")); })?;
    return Ok(Snapshotted { stage: stage_id, trash: pruned.trash });
}

/// What a snapshot that was taken left for the steps after it.
#[derive(Debug, Eq, PartialEq)]
pub struct Snapshotted {
    /// The identity of the stage this process made.
    pub stage: ObjectId,
    /// The trash the leave-out made in the stage, with the identity of everything it moved there; the disposal after the swap
    ///  empties exactly that.
    pub trash: Option<TrashManifest>,
}

/// Why a snapshot could not be taken, and the identity of the stage it left, if it left one.
#[derive(Debug, Eq, PartialEq)]
pub struct SnapshotFailure {
    /// The reason, in words.
    pub reason: String,
    /// The identity of the stage this process made before it failed.
    pub stage: Option<ObjectId>,
}

/// Why a swap of the stage with the destination did not complete, and whether the stage name still holds the snapshot.
#[derive(Debug, Eq, PartialEq)]
pub struct PlaceError {
    /// The reason, in words.
    pub reason: String,
    /// Whether the object at the stage name is the snapshot this process made. When it is not, nothing may be removed there:
    ///  the stage name holds somebody else's directory.
    pub stage_is_ours: bool,
}

/// The moments of a swap at which the controls move the tree.
pub struct PlaceSeams<'a> {
    /// Called after the destination was checked and before the exchange.
    pub after_check: &'a mut dyn FnMut(),
    /// Called after an exchange that found another directory at the stage name and before the swap back.
    pub before_swap_back: &'a mut dyn FnMut(),
}

/// What:
///  Swap the finished stage with the destination, after giving the stage the mode of the source's root.
/// Why:
///   `RENAME_EXCHANGE` leaves no instant at which the destination is missing or incomplete; the old destination, a
///  directory with Git's `.git` file, ends up at the stage name for `settle_displaced`. The names are looked up in the held
///  parent; the destination must be the directory Git made, and a directory that is not is swapped back. When the swap back
///  fails, the error says the stage name no longer holds the snapshot, so the caller does not remove what is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await putInPlace({ stage, destination, sourceMode, expected }); // expected: the identity of the directory Git made
/// ```
pub fn put_in_place(stage: &Path, destination: &Path, source_mode: u32, expected: ObjectId) -> Result<(), PlaceError> {
    return put_in_place_with(stage, destination, source_mode, expected, &mut PlaceSeams { after_check: &mut || {}, before_swap_back: &mut || {} });
}

/// `put_in_place` with the moments at which a control may move the tree.
pub fn put_in_place_with(stage: &Path, destination: &Path, source_mode: u32, expected: ObjectId, seams: &mut PlaceSeams<'_>) -> Result<(), PlaceError> {
    let ours = |reason: String| -> PlaceError { return PlaceError { reason, stage_is_ours: true }; };
    let (Some(parent_path), Some(stage_name), Some(destination_name)) = (stage.parent(), stage.file_name(), destination.file_name()) else {
        return Err(ours(String::from("the stage and the destination have no parent")));
    };
    if destination.parent() != Some(parent_path) {
        return Err(ours(String::from("the stage is not beside the destination")));
    }
    let parent: File = open_root(parent_path).map_err(|error| { return ours(format!("the parent of the destination cannot be opened ({error})")); })?;
    let snapshot: ObjectId = identity_at(&parent, stage_name).map_err(|error| { return ours(format!("the stage cannot be examined ({error})")); })?;
    let is_directory = |name: &OsStr| -> bool { return crate::fs_beneath::entry_metadata_at(&parent, name).is_ok_and(|found: std::fs::Metadata| { return found.is_dir(); }); };
    if !is_directory(destination_name) || identity_at(&parent, destination_name).ok() != Some(expected) {
        return Err(ours(format!("{} is not the directory Git made", destination.display())));
    }
    std::fs::set_permissions(stage, std::fs::Permissions::from_mode(source_mode)).map_err(|error| { return ours(format!("the stage cannot take the source's mode ({error})")); })?;
    (seams.after_check)();
    exchange_at(&parent, stage_name, destination_name).map_err(|error| { return ours(format!("the destination cannot be swapped with the stage ({error})")); })?;
    // The directory displaced to the stage name must be the one that was checked; another means the destination was replaced.
    if identity_at(&parent, stage_name).ok() == Some(expected) {
        return Ok(());
    }
    (seams.before_swap_back)();
    if let Err(error) = exchange_at(&parent, stage_name, destination_name) {
        return Err(PlaceError { reason: format!("{} changed while it was swapped and the swap could not be undone ({error})", destination.display()), stage_is_ours: false });
    }
    // The undo put the stage name back to what it held before only if that is the snapshot.
    let restored: bool = identity_at(&parent, stage_name).ok() == Some(snapshot);
    return Err(PlaceError { reason: format!("{} changed while it was swapped; the swap was undone", destination.display()), stage_is_ours: restored });
}

/// What:
///  Settle the directory real Git made, now at the stage name: move what was added to it meanwhile into the new worktree,
///  remove Git's pointer when it is the one that was written, remove the directory when it is empty, and return what stays.
/// Why:
///   The destination may have gained files between the look and the exchange, and a recursive removal would delete them.
///  Only the pointer and an empty directory are removed; every other entry is moved without replacing, or stays with a note.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notes = await settleDisplaced({ stage, destination, expected, pointer });
/// ```
pub fn settle_displaced(stage: &Path, destination: &Path, expected: ObjectId, pointer: &[u8]) -> Vec<String> {
    let (Some(parent_path), Some(stage_name), Some(destination_name)) = (stage.parent(), stage.file_name(), destination.file_name()) else {
        return vec![format!("{} has no parent", stage.display())];
    };
    let mut notes: Vec<String> = Vec::new();
    let opened = (|| -> std::io::Result<(File, File, File)> {
        let parent: File = open_root(parent_path)?;
        let displaced: File = open_directory_at(&parent, stage_name)?;
        let placed: File = open_directory_at(&parent, destination_name)?;
        return Ok((parent, displaced, placed));
    })();
    let Ok((parent, displaced, placed)) = opened else { return vec![format!("{} could not be opened to be settled", stage.display())] };
    if crate::fs_beneath::identity_of(&displaced).ok() != Some(expected) {
        return vec![format!("{} is not the directory Git made and stays", stage.display())];
    }
    let listing: Vec<OsString> = match std::fs::read_dir(crate::fs_beneath::handle_path(&displaced)) {
        Ok(found) => found.flatten().map(|entry| { return entry.file_name(); }).collect(),
        Err(error) => return vec![format!("{} cannot be listed ({error})", stage.display())],
    };
    for name in listing {
        if name == ".git" {
            let same: bool = open_file_at(&displaced, name.as_os_str()).ok().is_some_and(|mut file: File| {
                let mut found: Vec<u8> = Vec::new();
                return std::io::Read::read_to_end(&mut file, &mut found).is_ok() && found == pointer;
            });
            if same {
                if let Err(error) = remove_at(&displaced, name.as_os_str(), false) {
                    notes.push(format!("Git's pointer in {} cannot be removed ({error})", stage.display()));
                }
            } else {
                notes.push(format!(".git in {} is not the pointer Git wrote and stays", stage.display()));
            }
            continue;
        }
        if let Err(error) = rename_at(&displaced, name.as_os_str(), &placed, name.as_os_str(), false) {
            notes.push(format!("{} was added to {} while the worktree was made and stays in {} ({error})", name.to_string_lossy(), destination.display(), stage.display()));
        }
    }
    if let Err(error) = remove_at(&parent, stage_name, true) {
        notes.push(format!("{} is not removed ({error})", stage.display()));
    }
    return notes;
}

/// Snapshot mechanics controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_tests.rs"]
mod tests;

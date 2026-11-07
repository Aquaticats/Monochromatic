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

use crate::add_timing::{begin_step, end_step};
use crate::fs_beneath::{ObjectId, exchange_at, identity_at, open_directory_at, open_directory_beneath, open_file_at, open_root, plain_components, remove_at, rename_at};
use crate::fs_btrfs_ioctl::snapshot_create_at;
use crate::subvolume_index_files::install_index_in;
use crate::subvolume_remove::remove_tree;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_path_failure_message::subject_failure;
use crate::subvolume_index::{StatFields, StatLookup};
use crate::worktree_clean::{StatCursor, StatEntry, stored_stats, tree_fields_beneath};
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash::{Pruned, TrashManifest, prune_snapshot};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// What:
///  Whether an error says the entry is already gone.
/// Why:
///   A stub that is gone already is as good as removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.code === 'ENOENT'
/// ```
fn is_gone(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound;
}

/// What:
///  Do nothing; the seams of `put_in_place` run this unless a control puts work there.
/// Why:
///   The real swap has nothing to do between its steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
fn do_nothing() {}

/// What:
///  A placement step's failure marked as one where the stage is still this process's to destroy.
/// Why:
///   Every failure before the swap leaves the stage as this process made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcome.mapErr((reason) => ({ reason, stageIsOurs: true }))
/// ```
fn ours_on_failure<T>(outcome: Result<T, String>) -> Result<T, PlaceError> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(reason) => Err(ours(reason)),
    };
}

/// A placement failure with its reason, marked as one where the stage is still this process's to destroy.
fn ours(reason: String) -> PlaceError {
    return PlaceError { reason, stage_is_ours: true };
}

/// What:
///  A snapshot step's failure carrying the identity of the stage this process made, so the caller can destroy it.
/// Why:
///   Once the stage exists, every later failure must name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcome.mapErr((reason) => ({ reason, stage: made }))
/// ```
fn failing<T>(outcome: Result<T, String>, made: Option<ObjectId>) -> Result<T, SnapshotFailure> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(reason) => Err(SnapshotFailure { reason, stage: made }),
    };
}

/// What:
///  The identity of what a path's name leads to now, or nothing when its parent cannot be opened or the name cannot be looked up.
/// Why:
///   The stage's identity is what a later removal checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityAt(openRoot(parent(path)), basename(path)) ?? undefined
/// ```
fn identity_of_name(path: &Path) -> Option<ObjectId> {
    let parent: File = open_root(path.parent().unwrap_or(path)).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// What:
///  Whether the entry `name` below `parent` is a directory itself; an entry that cannot be examined is not.
/// Why:
///   Only the directory Git made may be swapped with the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lstat(join(parent, name)).isDirectory()
/// ```
fn is_directory_at(parent: &File, name: &OsStr) -> bool {
    return match crate::fs_beneath::entry_metadata_at(parent, name) {
        Ok(found) => found.is_dir(),
        Err(_unreadable) => false,
    };
}

/// What:
///  Open the parent directory and the displaced stage and the placed destination below it.
/// Why:
///   Settling works on handles, so a rename of a name cannot redirect it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [openRoot(parent), openDirectoryAt(parent, stage), openDirectoryAt(parent, destination)]
/// ```
fn open_for_settling(parent_path: &Path, stage_name: &OsStr, destination_name: &OsStr) -> std::io::Result<(File, File, File)> {
    let parent: File = open_root(parent_path)?;
    let displaced: File = open_directory_at(&parent, stage_name)?;
    let placed: File = open_directory_at(&parent, destination_name)?;
    return Ok((parent, displaced, placed));
}

/// What:
///  Whether the file `name` below `directory` can be read and holds exactly `expected`.
/// Why:
///   Git's pointer is removed only when it is the one Git wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(join(directory, name)).equals(expected)
/// ```
fn file_holds(directory: &File, name: &OsStr, expected: &[u8]) -> bool {
    let Ok(mut file) = open_file_at(directory, name) else { return false };
    let mut found: Vec<u8> = Vec::new();
    return std::io::Read::read_to_end(&mut file, &mut found).is_ok() && found == expected;
}

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
    let names: Vec<OsString> = subject_failure(plain_components(relative), &shown, "is not a plain path inside the source")?;
    let Some((name, parents)) = names.split_last() else { return Ok(()) };
    let parent: File = match open_directory_beneath(root, parents) {
        Ok(found) => found,
        Err(error) if is_gone(&error) => return Ok(()),
        Err(error) => return Err(format!("{shown} is reached through a link or another mount in the snapshot ({error})")),
    };
    if pointer {
        match open_directory_at(&parent, name.as_os_str()) {
            Ok(stub) => match remove_at(&stub, OsStr::new(".git"), false) {
                Ok(()) => {}
                Err(error) if is_gone(&error) => {}
                Err(error) => return Err(format!("{shown}/.git cannot be removed from the snapshot ({error})")),
            },
            Err(error) if is_gone(&error) => return Ok(()),
            Err(error) => return Err(format!("{shown} cannot be entered in the snapshot ({error})")),
        }
    }
    return match remove_at(&parent, name.as_os_str(), true) {
        Ok(()) => Ok(()),
        Err(error) if is_gone(&error) => Ok(()),
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
    let root: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
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
    subject_failure(remove_tree(path.as_path()), &"the source's .git", "cannot be removed from the snapshot")?;
    return subject_failure(std::fs::write(path.as_path(), pointer), &"the .git pointer", "cannot be written");
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
    let root: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    // A clean source proved its index equal to its files, so it is compared with the snapshot. A source with changes has no such
    //  proof: the snapshot's own index, one instant with its files, is installed as it is and the restore settles the rest.
    let bytes: &[u8] = own_index.unwrap_or(source.clean.index_bytes.as_slice());
    let entries: Vec<StatEntry> = if own_index.is_none() {
        let Some(stats) = stored_stats(bytes, source.clean.hash_length) else {
            return Err(String::from("the source's index changed shape"));
        };
        stats
    } else {
        Vec::new()
    };
    // One walk of the snapshot proves it equal to the source's index and returns the stat fields the new index takes: every
    //  file is looked at once, and the fields are the ones the verdict was given on.
    let walk_snapshot_entered: bool = begin_step("walkSnapshot");
    let walk_snapshot_outcome = tree_fields_beneath(&root, entries.as_slice(), source.clean.index_seconds);
    end_step(walk_snapshot_entered);
    let walked: Vec<StatFields> = match walk_snapshot_outcome {
        Ok(fields) => fields,
        Err(refusal) => return Err(format!("the source changed while it was copied ({})", refusal.reason)),
    };
    let mut known: std::collections::HashMap<&[u8], StatFields> = std::collections::HashMap::new();
    for (entry, fields) in entries.iter().zip(walked) {
        known.insert(entry.path.as_slice(), fields);
    }
    // An entry the walk did not cover (a source with changes has no walk) is looked at through a cursor that keeps its directory.
    let mut cursor: StatCursor<'_> = StatCursor::new(&root);
    let install_index_file_entered: bool = begin_step("installIndexFile");
    let install_index_file_outcome = install_index_in(admin, bytes, &mut WalkedThenCursor { known: &known, cursor: &mut cursor }, source.clean.hash_length);
    end_step(install_index_file_entered);
    let report: String = install_index_file_outcome?;
    crate::diagnostic_log::debug("takeSnapshot", report.as_str());
    return Ok(());
}

/// What:
///  The stat fields of the snapshot's files: the ones the walk returned, and a cursor for an entry the walk did not cover.
/// Why:
///   One walk of the snapshot gave the fields of the files it covered; a source with changes has no walk, so its entries are looked
///  at through a cursor that keeps its directory open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WalkedThenCursor implements StatLookup { statNow(path) { return this.known.get(path) ?? this.cursor.fields(path); } }
/// ```
struct WalkedThenCursor<'a, 'root> {
    /// The fields the walk returned, by path.
    known: &'a std::collections::HashMap<&'a [u8], StatFields>,
    /// The cursor for an entry the walk did not cover.
    cursor: &'a mut StatCursor<'root>,
}

impl StatLookup for WalkedThenCursor<'_, '_> {
    fn stat_now(&mut self, path: &[u8]) -> Option<StatFields> {
        return match self.known.get(path) {
            Some(fields) => Some(*fields),
            None => self.cursor.fields(path),
        };
    }
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
    let root: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    let git_dir: File = subject_failure(open_directory_at(&root, OsStr::new(".git")), &"the snapshot", "has no .git directory")?;
    let mut index: File = subject_failure(open_file_at(&git_dir, OsStr::new("index")), &"the snapshot", "has no index")?;
    let mut bytes: Vec<u8> = Vec::new();
    subject_failure(std::io::Read::read_to_end(&mut index, &mut bytes), &"the snapshot's index", "is unreadable")?;
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
    let snapshot_ioctl_entered: bool = begin_step("snapshotIoctl");
    let snapshot_ioctl_outcome = snapshot_create_at(source.root.as_path(), stage);
    end_step(snapshot_ioctl_entered);
    if let Err(error) = snapshot_ioctl_outcome {
        return Err(SnapshotFailure { reason: format!("the snapshot was refused ({error})"), stage: None });
    }
    // From here the stage exists and this process made it; its identity is what a later removal checks.
    let made: Option<ObjectId> = identity_of_name(stage);
    failing(subject_failure(std::fs::set_permissions(stage, std::fs::Permissions::from_mode(0o700)), &"the stage", "cannot be made private"), made)?;
    // The snapshot's own index is read before its `.git` goes: it is the index of the same instant as the files.
    let own_index: Option<Vec<u8>> = if source.dirty.is_some() {
        let read_own_index_entered: bool = begin_step("readOwnIndex");
        let read_own_index_outcome = read_own_index(stage);
        end_step(read_own_index_entered);
        Some(failing(read_own_index_outcome, made)?)
    } else {
        None
    };
    let prune_entered: bool = begin_step("prune");
    let prune_outcome = prune_snapshot(stage, source);
    end_step(prune_entered);
    let pruned: Pruned = failing(prune_outcome, made)?;
    let remove_stubs_entered: bool = begin_step("removeStubs");
    let remove_stubs_outcome = remove_stubs(stage, source, plan);
    end_step(remove_stubs_entered);
    failing(remove_stubs_outcome, made)?;
    let replace_pointer_entered: bool = begin_step("replacePointer");
    let replace_pointer_outcome = replace_git_pointer(stage, pointer);
    end_step(replace_pointer_entered);
    failing(replace_pointer_outcome, made)?;
    let install_index_entered: bool = begin_step("installIndex");
    let install_index_outcome = install_index(source, stage, admin, own_index.as_deref());
    end_step(install_index_entered);
    failing(install_index_outcome, made)?;
    let Some(stage_id) = made else {
        return Err(SnapshotFailure { reason: String::from("the stage cannot be examined"), stage: None });
    };
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
    return put_in_place_with(stage, destination, source_mode, expected, &mut PlaceSeams { after_check: &mut do_nothing, before_swap_back: &mut do_nothing });
}

/// `put_in_place` with the moments at which a control may move the tree.
pub fn put_in_place_with(stage: &Path, destination: &Path, source_mode: u32, expected: ObjectId, seams: &mut PlaceSeams<'_>) -> Result<(), PlaceError> {
    let (Some(parent_path), Some(stage_name), Some(destination_name)) = (stage.parent(), stage.file_name(), destination.file_name()) else {
        return Err(ours(String::from("the stage and the destination have no parent")));
    };
    if destination.parent() != Some(parent_path) {
        return Err(ours(String::from("the stage is not beside the destination")));
    }
    let parent: File = ours_on_failure(subject_failure(open_root(parent_path), &"the parent of the destination", "cannot be opened"))?;
    let snapshot: ObjectId = ours_on_failure(subject_failure(identity_at(&parent, stage_name), &"the stage", "cannot be examined"))?;
    if !is_directory_at(&parent, destination_name) || identity_at(&parent, destination_name).ok() != Some(expected) {
        return Err(ours(format!("{} is not the directory Git made", destination.display())));
    }
    ours_on_failure(subject_failure(std::fs::set_permissions(stage, std::fs::Permissions::from_mode(source_mode)), &"the stage", "cannot take the source's mode"))?;
    (seams.after_check)();
    ours_on_failure(subject_failure(exchange_at(&parent, stage_name, destination_name), &"the destination", "cannot be swapped with the stage"))?;
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
    let opened = open_for_settling(parent_path, stage_name, destination_name);
    let Ok((parent, displaced, placed)) = opened else { return vec![format!("{} could not be opened to be settled", stage.display())] };
    if crate::fs_beneath::identity_of(&displaced).ok() != Some(expected) {
        return vec![format!("{} is not the directory Git made and stays", stage.display())];
    }
    let listing: Vec<OsString> = match std::fs::read_dir(crate::fs_beneath::handle_path(&displaced)) {
        Ok(found) => {
            let mut names: Vec<OsString> = Vec::new();
            for entry in found.flatten() {
                names.push(entry.file_name());
            }
            names
        }
        Err(error) => return vec![format!("{} cannot be listed ({error})", stage.display())],
    };
    for name in listing {
        if name == ".git" {
            let same: bool = file_holds(&displaced, name.as_os_str(), pointer);
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

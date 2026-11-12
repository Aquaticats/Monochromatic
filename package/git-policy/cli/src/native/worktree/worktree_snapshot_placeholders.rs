//! What:
//!  The empty folders a snapshot holds where its source holds a nested subvolume it did not carry,
//! below the folders that hold a
//!  repository's worktrees:
//! found by one listing of each such folder,
//! and taken off the add's waiting path,
//! because a folder that holds
//!  nothing but such stand-ins goes into the snapshot's trash behind one rename and a fresh empty folder takes its
//!  place.
//! Why:
//!  A btrfs snapshot leaves out every nested subvolume and shows a stand-in directory in its place
//!  (`doc/troubleshooting/btrfs-snapshot-keeps-an-empty-placeholder-for-a-nested-subvolume.md`),
//! so a worktree made as a
//!  snapshot while another add stages its snapshot beside its destination carries an empty folder nothing names,
//! and a
//!  tool that compares the tree with another copy sees a folder where the source holds a subvolume.
//! Each stand-in removed inside
//!  the add is one btrfs metadata change,
//! and every add waits on the transaction the others commit,
//! so a repository with many
//!  worktrees made every add wait on every other one:
//! 33 of 34 sampled adds sat in `wait_current_trans` inside `unlinkat` on 2026-11-10
//!  (`doc/planning/cli-git-worktree-add-stand-ins-off-the-waiting-path.md`).
//! The folder goes into the same trash the last release gave the
//!  commit service,
//! so the stand-ins are deleted after the add returns,
//! one worktree's trashes at a time and at the lowest I/O
//!  priority (`worktree_trash_hand_over.rs`).
//! The listing is limited to the folders that hold worktrees,
//! and a folder is taken only
//!  when every entry of it is a stand-in that is inode 2 and empty,
//! because every other entry may be someone's data:
//!  such a folder stays where it is,
//! and only the stand-ins in it are removed one by one.
//! A folder that is taken whole is looked at again in the trash,
//! because a directory somebody made between the check and the
//!  rename is never deleted with the stand-ins it replaced:
//! such a folder goes back where it was,
//! and when a directory took
//!  that name meanwhile,
//! it is kept under a name of this command's own beside the trash of the new worktree,
//! and the warning names the
//!  path it will have there,
//! so nothing the add deletes afterwards can take it (`never discard user data`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeSnapshotPlaceholders({ root, source, plan, trash }); // one rename per worktree folder that holds only stand-ins
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{ObjectId, entry_metadata_at, handle_path, make_directory_at, open_directory_at, plain_components, remove_at};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash::{TRASH_PARENT, Trash, TrashManifest, entry_name, make_trash, reopen_trash};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSnapshotPlaceholders";

/// What:
///  The inode number btrfs gives a stand-in,
/// read from the module that owns the number.
/// Why:
///   The leave-out step needs the same number to leave a stand-in alone,
/// and one constant in
///  two places would drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STAND_IN_INODE = BTRFS_STAND_IN_INODE;
/// ```
const BTRFS_STAND_IN_INODE: u64 = crate::fs_kind::BTRFS_STAND_IN_INODE;

/// What:
///  The bits of a folder's mode that a fresh folder takes from the one it replaces.
/// Why:
///   A worktree folder carries the permissions its source has,
/// and a folder made again with other permissions would
///  change what a person and every tool may do in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MODE_BITS = 0o7777;
/// ```
const MODE_BITS: u32 = 0o7777;

/// What:
///  The folders the worktrees of this repository are made in:
/// the parent of each registered worktree inside the source and the parent
///  of the destination,
/// once each and only below the source.
/// Why:
///   A stage container of an add in flight is named beside its destination,
/// so it lies in one of these folders;
/// a folder
///  outside the source is in no snapshot of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeFolders({ nested, destination, sourceRoot }): string[];
/// ```
fn worktree_folders(nested: &[PathBuf], destination: &Path, source_root: &Path) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    let paths = nested.iter().map(PathBuf::as_path).chain(std::iter::once(destination));
    for path in paths {
        let Some(parent) = path.parent() else { continue };
        if parent.strip_prefix(source_root).is_err() || folders.contains(&parent.to_path_buf()) {
            continue;
        }
        folders.push(parent.to_path_buf());
    }
    return folders;
}

/// What:
///  The names a held folder lists.
/// Why:
///   The placeholders are found by looking at every entry of a folder that holds worktrees,
/// and an entry that cannot be read
///  keeps the rung from finishing quietly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (await readdir(handlePath(dir))).map((e) => e.name);
/// ```
fn listed_names(directory: &File) -> std::io::Result<Vec<OsString>> {
    let mut names: Vec<OsString> = Vec::new();
    for entry in std::fs::read_dir(handle_path(directory))? {
        names.push(entry?.file_name());
    }
    return Ok(names);
}

/// What:
///  Whether the entry `name` below `directory` is the stand-in btrfs shows for a nested subvolume a snapshot did not
///  carry.
/// Why:
///   The inode number and the directory type are the whole test;
/// an entry that cannot be examined is no stand-in and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (found) => found.isDirectory() && found.ino === STAND_IN_INODE
/// ```
fn is_stand_in(directory: &File, name: &OsStr) -> bool {
    return match entry_metadata_at(directory, name) {
        Ok(found) => found.is_dir() && found.ino() == BTRFS_STAND_IN_INODE,
        Err(_unreadable) => false,
    };
}

/// What:
///  Whether the entry `name` below `directory` is a stand-in that holds nothing.
/// Why:
///   The inode number,
/// the directory type and an empty listing are the whole test for what may go into the trash;
/// an
///  entry that cannot be opened or read is no stand-in and stays,
/// so an entry that somehow holds anything is never taken with
///  the folder it sits in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnEmptyStandIn = (dir, name) => isStandIn(dir, name) && readdirSync(handlePath(open(dir, name))).length === 0;
/// ```
fn is_an_empty_stand_in(directory: &File, name: &OsStr) -> bool {
    if !is_stand_in(directory, name) {
        return false;
    }
    return match open_directory_at(directory, name) {
        Ok(held) => listed_names(&held).is_ok_and(holds_nothing),
        Err(_unreadable) => false,
    };
}

/// What:
///  Whether a listing answered with no names at all.
/// Why:
///   A stand-in of a nested subvolume holds nothing,
/// so an empty listing is what makes it a folder that may go into
///  the trash.
fn holds_nothing(listed: Vec<OsString>) -> bool {
    return listed.is_empty();
}

/// What:
///  How many entries a manifest already names.
/// Why:
///   The folder taken here becomes the next entry of the same trash,
/// and its name must not collide with one the
///  leave-out moved there.
fn entries_of(manifest: &TrashManifest) -> usize {
    return manifest.entries.len();
}

/// What:
///  The directory `names` lead to below the held snapshot,
/// or nothing when the snapshot does not carry it or when a
///  stand-in of a nested subvolume stands on the way.
/// Why:
///   A stand-in holds nothing and belongs to no folder a caller could name,
/// so nothing is below it;
/// every other refusal stays
///  an error,
/// so a link or another mount is named and never walked through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openBelowStandIns({ root, names, shown }): File | undefined; // throws on a link or a mount
/// ```
pub(crate) fn open_below_stand_ins(root: &File, names: &[OsString], shown: &str) -> Result<Option<File>, String> {
    let mut current: File = subject_failure(root.try_clone(), &shown, "cannot be held")?;
    for name in names {
        let next: File = match open_directory_at(&current, name.as_os_str()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{shown} is reached through a link or another mount in the snapshot ({error})")),
        };
        let found: std::fs::Metadata = subject_failure(next.metadata(), &shown, "cannot be examined")?;
        if found.is_dir() && found.ino() == BTRFS_STAND_IN_INODE {
            return Ok(None);
        }
        current = next;
    }
    return Ok(Some(current));
}

/// What:
///  The moments at which a control may change the snapshot while a folder is taken whole.
/// Why:
///   A folder goes into the trash behind one rename and is checked before it,
/// so only a moment between the check and
///  the rename lets a control show what a directory made in that window comes to;
/// and a folder that is no longer only
///  stand-ins goes back where it was,
/// so only a moment between the rename and that move lets a control show what
///  comes of a name somebody took meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FoldSeams = { beforeTake: () => void; beforePutBack: () => void };
/// ```
pub(crate) struct FoldSeams<'a> {
    /// Called after every entry of a folder was checked and before the folder goes into the trash.
    pub(crate) before_take: &'a mut dyn FnMut(),
    /// Called after the copy in the trash proved to hold more than stand-ins and before the folder goes back.
    pub(crate) before_put_back: &'a mut dyn FnMut(),
}

/// What:
///  Do nothing;
/// the seams of the fold run this unless a control puts work there.
/// Why:
///   The real fold has nothing to do between its steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
fn do_nothing() {}

/// What:
///  The trash this step puts whole worktree folders into:
/// the one the leave-out answered with,
/// or one made here when
///  the leave-out set nothing aside,
/// and the worktree it is made at,
/// which names where a folder that cannot go back is kept.
/// Why:
///   One record names one trash,
/// so a folder taken here joins the leave-out's trash rather than making a second one;
/// and a source whose Git does not ignore the trash place must never get one,
/// because a trash inside
///  the new worktree that Git does not ignore can be staged,
/// and one add's ignored state must never reach another worktree's
///  index.
/// A folder that cannot go back is kept in the trash parent,
/// one rename inside the stage,
/// because a folder that
///  holds a stand-in of a nested subvolume cannot be renamed out of the snapshot at all,
/// and the warning names the path it will have in the new worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandOver = { carried?: TrashManifest; held?: Trash; taken: ObjectId[]; noTrashPlace: boolean; newWorktree?: string };
/// ```
pub(crate) struct HandOver {
    /// What the leave-out set aside;
    /// the manifest the add hands to the service.
    carried: Option<TrashManifest>,
    /// The trash directory,
    /// held open once something goes into it.
    held: Option<Trash>,
    /// The identities of the entries this snapshot moved into the trash,
    /// in the order taken.
    taken: Vec<ObjectId>,
    /// Whether this source has no place a trash could be made,
    /// so nothing is looked at again for one.
    no_trash_place: bool,
    /// The new worktree this snapshot becomes,
    /// when the sweep named it,
    /// for the path a kept folder is named by.
    new_worktree: Option<PathBuf>,
}

impl HandOver {
    /// What:
    ///  What the leave-out of one snapshot set aside,
    /// to be extended by the steps after it.
    /// Why:
    ///   One trash is one work item,
    /// so every step that sets something aside adds to the same manifest,
    /// and the add hands
    ///  the service one record of everything it left out.
    pub(crate) fn starting(carried: Option<TrashManifest>) -> HandOver {
        return HandOver { carried, held: None, taken: Vec::new(), no_trash_place: false, new_worktree: None };
    }

    /// What:
    ///  Name the worktree this snapshot becomes,
    /// so a folder kept inside it is named by the path the person will find.
    /// Why:
    ///   The name a folder came from can be taken between its rename into the trash and its put back,
    /// and the folder is then kept in the
    ///  trash parent of the stage,
    /// which becomes that path once the add puts the stage in place.
    pub(crate) fn name_the_new_worktree(&mut self, destination: &Path) {
        self.new_worktree = Some(destination.to_path_buf());
    }

    /// The trash a whole folder may go into,
    /// or nothing when this source has no trash place.
    fn opened(&mut self, root: &File, source: &SnapshotSource) -> Result<Option<&Trash>, String> {
        if self.held.is_none() && !self.no_trash_place {
            match self.carried.as_ref() {
                Some(manifest) => self.held = Some(reopen_trash(root, manifest)?),
                None if source.trash_ignored => match make_trash(root) {
                    Ok(made) => self.held = Some(made),
                    Err(reason) => {
                        debug(TAG, format!("{reason}; what this step sets aside is removed where it is instead").as_str());
                        self.no_trash_place = true;
                    }
                },
                None => self.no_trash_place = true,
            }
        }
        return Ok(self.held.as_ref());
    }

    /// What:
    ///  Move the entry `name` of `parent` into the trash,
    /// whatever it holds,
    /// and answer whether it went.
    /// Why:
    ///   The snapshot's own `.git` is a copy of the whole common directory,
    /// with one administrative directory for
    ///  every worktree of the repository,
    /// and nothing the new worktree needs lives in it once the index is installed,
    /// so
    ///  removing it inside the add would cost one btrfs metadata change for each worktree of the repository.
    /// `false`
    ///  means this source has no trash place and the caller removes the entry where it is.
    pub(crate) fn take_entry(&mut self, root: &File, source: &SnapshotSource, parent: &File, name: &OsStr, shown: &Path) -> Result<bool, String> {
        let Some((_to, moved)) = self.move_into_the_trash(root, source, parent, name, shown)? else { return Ok(false) };
        self.taken.push(moved);
        return Ok(true);
    }

    /// What:
    ///  Move the entry `name` of `parent` into the trash and answer the name it has there with its identity,
    /// or
    ///  nothing when this source has no trash place.
    /// Why:
    ///   A folder that is taken whole is looked at again once it is in the trash (`never discard user data`),
    /// so
    ///  a step that moves an entry without checking it afterwards needs the move without the record,
    /// and the record is
    ///  written once what arrived is proven to be what may go.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const moved = handOver.moveIntoTheTrash({ root, source, parent, name, shown }); // { to, id } | undefined
    /// ```
    fn move_into_the_trash(&mut self, root: &File, source: &SnapshotSource, parent: &File, name: &OsStr, shown: &Path) -> Result<Option<(String, ObjectId)>, String> {
        let index: usize = self.carried.as_ref().map_or(0, entries_of) + self.taken.len();
        let to: String = entry_name(index);
        let moved: ObjectId = match self.opened(root, source)? {
            Some(trash) => trash.set_aside(parent, name, to.as_str(), shown)?,
            None => return Ok(None),
        };
        return Ok(Some((to, moved)));
    }

    /// What:
    ///  The folder the trash holds as `to`,
    /// when it is still the object `moved` that was put there.
    /// Why:
    ///   The check after the move reads the very copy the service would delete,
    /// so it is reached through the trash's own
    ///  held directory and only after its identity is compared.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handOver.heldInTheTrash({ to: 'd3', moved: id });
    /// ```
    fn held_in_the_trash(&self, to: &str, moved: ObjectId) -> Result<Option<File>, String> {
        let Some(trash) = self.held.as_ref() else { return Err(String::from("the trash this folder was put in is not held any more")) };
        return trash.held_entry(to, moved);
    }

    /// What:
    ///  Put the folder the trash holds as `to` back at `name` below `parent`,
    /// or answer why it cannot go there.
    /// Why:
    ///   A folder that is no longer only empty stand-ins is not the folder the check passed,
    /// so it is not deleted with
    ///  them;
    /// it goes back where it was,
    /// and the caller removes its stand-ins one by one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handOver.putBack({ to: 'd3', parent, name, shown });
    /// ```
    fn put_back(&self, to: &str, parent: &File, name: &OsStr, shown: &Path) -> Result<(), String> {
        let Some(trash) = self.held.as_ref() else { return Err(String::from("the trash this folder was put in is not held any more")) };
        return trash.put_back(to, parent, name, shown);
    }

    /// What:
    ///  Move the whole folder `name` of `parent` into the trash and make a fresh empty folder in its place,
    /// when every
    ///  entry of it is an empty stand-in,
    /// and answer how many stand-ins went with it.
    /// Why:
    ///   btrfs refuses to rename a stand-in,
    /// so the only way to take one off the add's waiting path is to move the folder
    ///  that holds it;
    /// one entry that is not such a stand-in keeps the folder where it is,
    /// because it may be someone's
    ///  data,
    /// and the fresh folder keeps the mode of the one it replaces.
    /// `None` means the folder stays and its stand-ins are removed one by one.
    fn take_folder(&mut self, root: &File, source: &SnapshotSource, parent: &File, name: &OsStr, held: &File, listed: &[OsString], shown: &Path, seams: &mut FoldSeams<'_>) -> Result<Option<usize>, String> {
        if listed.is_empty() {
            return Ok(None);
        }
        for entry in listed {
            if !is_an_empty_stand_in(held, entry.as_os_str()) {
                return Ok(None);
            }
        }
        let mode: u32 = subject_failure(held.metadata(), &shown.display(), "cannot be examined")?.permissions().mode() & MODE_BITS;
        (seams.before_take)();
        let Some((to, moved)) = self.move_into_the_trash(root, source, parent, name, shown)? else { return Ok(None) };
        // The check above and the rename are two steps, and a directory somebody made in between has to be seen before the
        //  service is told to delete the folder: the copy that arrived in the trash is looked at again.
        if !self.arrived_holding_only_stand_ins(to.as_str(), moved)? {
            (seams.before_put_back)();
            return self.take_the_folder_back(to.as_str(), moved, parent, name, shown);
        }
        self.taken.push(moved);
        let made: bool = subject_failure(make_directory_at(parent, name, mode), &shown.display(), "cannot be made again in the snapshot")?;
        if !made {
            return Err(format!("{} was made again by somebody else while the snapshot was taken, so it is left as it is", shown.display()));
        }
        let fresh: File = subject_failure(open_directory_at(parent, name), &shown.display(), "is not a plain directory in the snapshot")?;
        subject_failure(fresh.set_permissions(std::fs::Permissions::from_mode(mode)), &shown.display(), "cannot take the mode of the folder it replaces")?;
        return Ok(Some(listed.len()));
    }

    /// What:
    ///  Whether the folder the trash holds as `to` still holds nothing but empty stand-ins.
    /// Why:
    ///   The folder was checked before the rename,
    /// and this is the same question asked of the copy the service would
    ///  delete;
    /// nothing is proven when the entries cannot be listed,
    /// and a part of a folder that cannot be read must never be deleted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handOver.arrivedHoldingOnlyStandIns({ to: 'd3', moved: id });
    /// ```
    fn arrived_holding_only_stand_ins(&self, to: &str, moved: ObjectId) -> Result<bool, String> {
        let folder: File = match self.held_in_the_trash(to, moved) {
            Ok(Some(found)) => found,
            // The entry is not in the trash any more, so nothing of it is left to delete.
            Ok(None) => return Ok(true),
            Err(reason) => {
                debug(TAG, format!("{reason}; the folder is put back").as_str());
                return Ok(false);
            }
        };
        return Ok(holds_only_empty_stand_ins(&folder));
    }

    /// What:
    ///  Put a folder that is no longer only empty stand-ins back where it was,
    /// and answer that the folder stays
    ///  where it is.
    /// Why:
    ///   Its stand-ins may then be removed one by one,
    /// and the directory somebody made beside them keeps its place and its
    ///  bytes;
    /// when the name it came from is taken meanwhile,
    /// the folder is kept beside the trash under a name of this command's
    ///  own and the warning names where,
    /// because the add's own cleanup removes the stage and everything in it that this add did
    ///  not keep.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handOver.takeTheFolderBack({ to: 'd3', moved: id, parent, name, shown }); // the folder stays
    /// ```
    fn take_the_folder_back(&self, to: &str, moved: ObjectId, parent: &File, name: &OsStr, shown: &Path) -> Result<Option<usize>, String> {
        match self.put_back(to, parent, name, shown) {
            Ok(()) => {
                debug(TAG, format!("{} changed after it was checked, so it stays and its stand-ins are removed one by one", shown.display()).as_str());
            }
            Err(reason) => {
                let why_it_cannot_go_back: String = format!("{reason}; {} changed after it was checked, so it is not deleted with its stand-ins", shown.display());
                // A folder that is not held in a trash is one this step never set aside, so nothing of it is kept.
                let keeping: Result<Option<PathBuf>, String> = match self.held.as_ref() {
                    Some(trash) => crate::worktree_kept_folders::keep_the_folder(trash, to, moved, shown),
                    None => Ok(None),
                };
                match keeping {
                    Ok(Some(kept)) => warn(TAG, format!("{why_it_cannot_go_back}, and it is kept whole at {}; its stand-ins are removed from there one by one", self.shown(&kept)).as_str()),
                    Ok(None) => warn(TAG, format!("{why_it_cannot_go_back}, and the trash no longer holds the folder this add set aside, so it is left where it is").as_str()),
                    Err(cannot_keep) => warn(
                        TAG,
                        format!(
                            "{why_it_cannot_go_back}, and it cannot be kept beside the trash ({cannot_keep}), so it stays in {}, which the cleanup of the stage this add ends without serving removes with the rest; copy it out before the add ends",
                            self.shown_trash_entry(to)
                        )
                        .as_str(),
                    ),
                }
            }
        }
        return Ok(None);
    }

    /// What:
    ///  The path to print for the trash entry `to`:
    /// the trash's own name and the entry's inside it.
    /// Why:
    ///   A person who has to copy a folder out of a trash needs the path it is at,
    /// and an entry's name alone names no
    ///  folder.
    fn shown_trash_entry(&self, to: &str) -> String {
        let Some(trash) = self.held.as_ref() else { return Path::new(TRASH_PARENT).join(to).display().to_string() };
        return Path::new(TRASH_PARENT).join(trash.name()).join(to).display().to_string();
    }

    /// What:
    ///  The path to print for the folder kept at `kept`:
    /// the path it has in the new worktree when the sweep named one,
    /// and the path it has now otherwise.
    /// Why:
    ///   A path inside the stage is the one that stops existing when the add puts the stage in place,
    /// so the person is told the
    ///  path the folder will have when the add serves it,
    /// and the stage's own path only when the add ends without serving it.
    fn shown(&self, kept: &Path) -> String {
        return match self.new_worktree.as_ref() {
            Some(worktree) => worktree.join(kept).display().to_string(),
            None => kept.display().to_string(),
        };
    }

    /// What:
    ///  The manifest the add hands to the service,
    /// or nothing when nothing is to be emptied.
    /// Why:
    ///   A trash this step made that stayed empty goes again,
    /// as the leave-out's does,
    /// so no folder is left
    ///  behind that no record names.
    pub(crate) fn finish(self) -> Option<TrashManifest> {
        return match self.carried {
            Some(mut manifest) => {
                manifest.entries.extend(self.taken);
                Some(manifest)
            }
            None => match self.held {
                Some(trash) => trash.finish(self.taken),
                None => None,
            },
        };
    }
}

/// What:
///  Whether every entry of the held folder `folder` is an empty stand-in and there is at least one.
/// Why:
///   The same question the check before a fold asks,
/// asked of the copy that arrived in the trash;
/// a folder that cannot
///  be listed proves nothing and is never deleted.
fn holds_only_empty_stand_ins(folder: &File) -> bool {
    let listed: Vec<OsString> = match listed_names(folder) {
        Ok(found) => found,
        Err(_unreadable) => return false,
    };
    if listed.is_empty() {
        return false;
    }
    for entry in &listed {
        if !is_an_empty_stand_in(folder, entry.as_os_str()) {
            return false;
        }
    }
    return true;
}

/// What:
///  Remove every stand-in of the held folder `held` as an empty directory,
/// and answer how many were removed.
/// Why:
///   A folder that holds one entry of someone else's is never moved whole,
/// so the stand-ins are the only thing here that
///  may go,
/// and the empty-directory removal keeps every file of that case too:
/// a folder that holds anything is refused.
fn remove_stand_ins(held: &File, shown: &str, listed: &[OsString]) -> Result<usize, String> {
    let mut removed: usize = 0;
    for name in listed {
        if !is_stand_in(held, name.as_os_str()) {
            continue;
        }
        let path: String = format!("{shown}/{}", name.to_string_lossy());
        match remove_at(held, name.as_os_str(), true) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{path} is a stand-in folder that cannot be removed from the snapshot as an empty directory ({error})")),
        }
    }
    return Ok(removed);
}

/// What:
///  Take the stand-ins a snapshot holds below the folder `folder` off the add's waiting path,
/// and answer how many
///  there were.
/// Why:
///   A folder the snapshot does not carry and a path outside the source are nothing to do;
/// a path that reaches the
///  snapshot through a link or another mount is an error,
/// so no entry is ever removed from the tree such a link points at.
/// The folder itself is taken when every entry of it is an empty stand-in,
/// and its stand-ins are removed one by one when
///  somebody else's entry sits beside them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standInsBelow({ root, source, folder, handOver }): number;
/// ```
fn stand_ins_below(root: &File, source: &SnapshotSource, folder: &Path, hand_over: &mut HandOver, seams: &mut FoldSeams<'_>) -> Result<usize, String> {
    let Ok(relative) = folder.strip_prefix(source.root.as_path()) else { return Ok(0) };
    let shown: String = folder.display().to_string();
    let names: Vec<OsString> = subject_failure(plain_components(relative), &shown, "is not a plain path inside the source")?;
    let Some((name, parents)) = names.split_last() else {
        // The source root is the snapshot itself, so no fresh folder can take its place here.
        let held: File = subject_failure(root.try_clone(), &shown, "cannot be held")?;
        let listed: Vec<OsString> = match listed_names(&held) {
            Ok(found) => found,
            Err(error) => return Err(format!("{shown} cannot be listed in the snapshot ({error})")),
        };
        return remove_stand_ins(&held, &shown, listed.as_slice());
    };
    let Some(parent) = open_below_stand_ins(root, parents, &shown)? else { return Ok(0) };
    let held: File = match open_directory_at(&parent, name.as_os_str()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(format!("{shown} is reached through a link or another mount in the snapshot ({error})")),
    };
    let listed: Vec<OsString> = match listed_names(&held) {
        Ok(found) => found,
        Err(error) => return Err(format!("{shown} cannot be listed in the snapshot ({error})")),
    };
    if let Some(count) = hand_over.take_folder(root, source, &parent, name.as_os_str(), &held, listed.as_slice(), folder, seams)? {
        return Ok(count);
    }
    return remove_stand_ins(&held, &shown, listed.as_slice());
}

/// What:
///  Remove the stand-ins the snapshot holds below the folders that hold this repository's worktrees and the destination,
/// and add every folder taken whole to the trash the add hands to the service.
/// Why:
///   The registered worktrees and the destination are removed one by one before this runs,
/// so what is left in those folders
///  is what no worktree names:
/// a stage container of an add in flight,
/// which the new worktree must not carry.
/// The trash the folders join is the leave-out's,
/// or one made here when the leave-out set nothing aside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeSnapshotPlaceholders({ root, source, plan, handOver });
/// ```
pub(crate) fn remove_snapshot_placeholders(root: &File, source: &SnapshotSource, plan: &AddPlan, hand_over: &mut HandOver) -> Result<(), String> {
    return remove_snapshot_placeholders_with(root, source, plan, hand_over, &mut FoldSeams { before_take: &mut do_nothing, before_put_back: &mut do_nothing });
}

/// What:
///  The same run of the step with the moments at which a control may change the snapshot given.
/// Why:
///   A control has no way to write into the snapshot between the check of a folder and its rename;
/// the seam is that moment,
///  and the production run puts nothing there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeSnapshotPlaceholdersWith({ root, source, plan, handOver, seams });
/// ```
pub(crate) fn remove_snapshot_placeholders_with(root: &File, source: &SnapshotSource, plan: &AddPlan, hand_over: &mut HandOver, seams: &mut FoldSeams<'_>) -> Result<(), String> {
    let mut removed: usize = 0;
    let folders_before: usize = hand_over.taken.len();
    // A folder this sweep takes whole is kept in the trash parent when it cannot go back to the name it came from,
    //  and the warning names the path it has once the stage is the new worktree.
    hand_over.name_the_new_worktree(plan.destination.as_path());
    for folder in worktree_folders(source.nested.as_slice(), plan.destination.as_path(), source.root.as_path()) {
        removed += stand_ins_below(root, source, folder.as_path(), hand_over, seams)?;
    }
    let took: usize = hand_over.taken.len() - folders_before;
    debug(TAG, format!("{}: {removed} stand-in folders of nested subvolumes removed from the snapshot, {took} worktree folders taken whole into the trash", plan.destination.display()).as_str());
    return Ok(());
}

/// Placeholder removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_placeholders_tests.rs"]
mod tests;

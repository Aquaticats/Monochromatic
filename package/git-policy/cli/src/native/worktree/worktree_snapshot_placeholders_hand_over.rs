//! What:
//!  The trash a whole worktree folder goes into:
//! the folder's move into the leave-out's trash or into one made here,
//! the fresh empty folder that takes its place,
//! the look at the copy that arrived,
//! and the folder kept beside the trash when the name it came from was taken.
//! Why:
//!  A folder that holds nothing but stand-ins goes into the snapshot's trash behind one rename,
//! so the stand-ins leave the add's waiting path without one btrfs metadata change each.
//! One record names one trash,
//! so a folder taken here joins the leave-out's trash rather than making a second one,
//! and a source whose Git does not ignore the trash place never gets one.
//! A directory somebody made between the check and the rename is never deleted with the stand-ins it replaced:
//! the copy that arrived is looked at again,
//! and a folder that cannot go back is kept whole and its path named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const handOver = HandOver.starting(carried);
//! handOver.takeEntry({ root, source, parent, name, shown });
//! const manifest = handOver.finish();
//! ```

use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::fs_beneath::{
    ObjectId,
    make_directory_at,
    open_directory_at,
};
use crate::the_names_and_the_manifest_of_the_trash::TRASH_PARENT;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_placeholders_entries::{
    FoldSeams,
    entries_of,
    holds_only_empty_stand_ins,
    is_an_empty_stand_in,
};
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash_manifest::{
    TrashManifest,
    entry_name,
};
use crate::worktree_trash_set_aside::{
    Trash,
    make_trash,
    reopen_trash,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSnapshotPlaceholders";

/// The bits of a folder's mode that a fresh folder takes from the one it replaces.
/// Why:
///  A worktree folder carries the permissions its source has,
/// and a folder made again with other permissions would
///  change what a person and every tool may do in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MODE_BITS = 0o7777;
/// ```
const MODE_BITS: u32 = 0o7777;

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
    pub(crate) taken: Vec<ObjectId>,
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
        return HandOver {
            carried,
            held: None,
            taken: Vec::new(),
            no_trash_place: false,
            new_worktree: None,
        };
    }

    /// What:
    ///  Name the worktree this snapshot becomes,
    /// so a folder kept inside it is named by the path the person will find.
    /// Why:
    ///   The name a folder came from can be taken between its rename into the trash and its put back,
    /// and the folder is then kept in the
    ///  trash parent of the stage,
    /// which becomes that path once the add puts the stage in place.
    pub(crate) fn name_the_new_worktree(
        &mut self,
        destination: &Path,
    ) {
        self.new_worktree = Some(destination.to_path_buf());
    }

    /// The trash a whole folder may go into,
    /// or nothing when this source has no trash place.
    fn opened(
        &mut self,
        root: &File,
        source: &SnapshotSource,
    ) -> Result<Option<&Trash>, String> {
        if self.held.is_none() && !self.no_trash_place {
            match self.carried.as_ref() {
                Some(manifest) => self.held = Some(reopen_trash(root, manifest)?),
                None if source.trash_ignored => {
                    match make_trash(root) {
                        Ok(made) => self.held = Some(made),
                        Err(reason) => {
                            debug(TAG, format!("{reason}; what this step sets aside is removed where it is instead").as_str());
                            self.no_trash_place = true;
                        }
                    }
                }
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
    pub(crate) fn take_entry(
        &mut self,
        root: &File,
        source: &SnapshotSource,
        parent: &File,
        name: &OsStr,
        shown: &Path,
    ) -> Result<bool, String> {
        let Some((_to, moved)) = self.move_into_the_trash(root, source, parent, name, shown)?
        else {
            return Ok(false);
        };
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
    fn move_into_the_trash(
        &mut self,
        root: &File,
        source: &SnapshotSource,
        parent: &File,
        name: &OsStr,
        shown: &Path,
    ) -> Result<Option<(String, ObjectId)>, String> {
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
    fn held_in_the_trash(
        &self,
        to: &str,
        moved: ObjectId,
    ) -> Result<Option<File>, String> {
        let Some(trash) = self.held.as_ref() else {
            return Err(String::from(
                "the trash this folder was put in is not held any more",
            ));
        };
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
    fn put_back(
        &self,
        to: &str,
        parent: &File,
        name: &OsStr,
        shown: &Path,
    ) -> Result<(), String> {
        let Some(trash) = self.held.as_ref() else {
            return Err(String::from(
                "the trash this folder was put in is not held any more",
            ));
        };
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
    pub(crate) fn take_folder(
        &mut self,
        root: &File,
        source: &SnapshotSource,
        parent: &File,
        name: &OsStr,
        held: &File,
        listed: &[OsString],
        shown: &Path,
        seams: &mut FoldSeams<'_>,
    ) -> Result<Option<usize>, String> {
        if listed.is_empty() {
            return Ok(None);
        }
        for entry in listed {
            if !is_an_empty_stand_in(held, entry.as_os_str()) {
                return Ok(None);
            }
        }
        let mode: u32 = subject_failure(held.metadata(), &shown.display(), "cannot be examined")?
            .permissions()
            .mode()
            & MODE_BITS;
        (seams.before_take)();
        let Some((to, moved)) = self.move_into_the_trash(root, source, parent, name, shown)? else {
            return Ok(None);
        };
        // The check above and the rename are two steps, and a directory somebody made in between has to be seen before the
        //  service is told to delete the folder: the copy that arrived in the trash is looked at again.
        if !self.arrived_holding_only_stand_ins(to.as_str(), moved)? {
            (seams.before_put_back)();
            return self.take_the_folder_back(to.as_str(), moved, parent, name, shown);
        }
        self.taken.push(moved);
        let made: bool = subject_failure(
            make_directory_at(parent, name, mode),
            &shown.display(),
            "cannot be made again in the snapshot",
        )?;
        if !made {
            return Err(format!(
                "{} was made again by somebody else while the snapshot was taken, so it is left as it is",
                shown.display()
            ));
        }
        let fresh: File = subject_failure(
            open_directory_at(parent, name),
            &shown.display(),
            "is not a plain directory in the snapshot",
        )?;
        subject_failure(
            fresh.set_permissions(std::fs::Permissions::from_mode(mode)),
            &shown.display(),
            "cannot take the mode of the folder it replaces",
        )?;
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
    fn arrived_holding_only_stand_ins(
        &self,
        to: &str,
        moved: ObjectId,
    ) -> Result<bool, String> {
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
    fn take_the_folder_back(
        &self,
        to: &str,
        moved: ObjectId,
        parent: &File,
        name: &OsStr,
        shown: &Path,
    ) -> Result<Option<usize>, String> {
        match self.put_back(to, parent, name, shown) {
            Ok(()) => {
                debug(TAG, format!("{} changed after it was checked, so it stays and its stand-ins are removed one by one", shown.display()).as_str());
            }
            Err(reason) => {
                let why_it_cannot_go_back: String = format!(
                    "{reason}; {} changed after it was checked, so it is not deleted with its stand-ins",
                    shown.display()
                );
                // A folder that is not held in a trash is one this step never set aside, so nothing of it is kept.
                let keeping: Result<Option<PathBuf>, String> = match self.held.as_ref() {
                    Some(trash) => {
                        crate::worktree_kept_folders::keep_the_folder(trash, to, moved, shown)
                    }
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
    fn shown_trash_entry(
        &self,
        to: &str,
    ) -> String {
        let Some(trash) = self.held.as_ref() else {
            return Path::new(TRASH_PARENT).join(to).display().to_string();
        };
        return Path::new(TRASH_PARENT)
            .join(trash.name())
            .join(to)
            .display()
            .to_string();
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
    fn shown(
        &self,
        kept: &Path,
    ) -> String {
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

//! What:
//!  The removals the recipe makes below held descriptors:
//!  a file or a tree,
//! the ref store's files before its folders,
//!  every entry a listing shows,
//!  the names one run works from,
//!  and why a run stopped.
//! Why:
//!  Every call is descriptor-relative and treats `ENOENT` as the step's own success,
//!  so one run serves every row the table clears
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The recipe that clears rows 1 to 5,
//! 7 and 8").
//!  Kept apart from the order so both stay under the file limit,
//!  and the names and the refusal sit here rather than beside the order,
//!  so the recipe's two files use each other in one direction only
//!  (`AGENTS.md` rule LYF).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await emptyAndRemove(parent, 'logs', 'HEAD'); await removeOneBelow(parent, name);
//! ```

use crate::entries_of_one_open_folder::entry_names;
use crate::fs_beneath::{
    entry_metadata_at,
    open_directory_at,
    remove_at,
};
use std::ffi::OsStr;
use std::io::Read as _;
use std::path::Path;

/// What:
///  The names one recipe run works from,
///  each a name below a descriptor the caller already holds.
/// Why:
///  A path read at the wrong moment names another object,
///  so the walk never leaves the two pinned parents.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToClear = { slotParent: fd; slotName: string; slot: string; worktreesParent: fd; registrationName: string; registration: string };
/// ```
pub struct ToClear<'a> {
    /// The `.claude/worktrees` parent's descriptor.
    pub slot_parent: &'a std::fs::File,
    /// The slot's own name below that parent,
    /// and the file the pointer is removed from.
    pub slot_pointer_name: &'a OsStr,
    /// The slot's path,
    /// which the pointer's bytes must name this registration from.
    pub slot: &'a Path,
    /// The `worktrees` parent's descriptor.
    pub worktrees_parent: &'a std::fs::File,
    /// The registration's name below that parent.
    pub registration_name: &'a OsStr,
    /// The registration's path,
    /// which the slot's pointer must name.
    pub registration: &'a Path,
}

/// What:
///  Why a recipe run stopped without finishing.
/// Why:
///  Every stop names the name it refused and the way forward,
///  and nothing it removed is put back:
///  the row that stands is one the finder still recognises.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { message: string };
/// ```
#[derive(Debug)]
pub enum RecipeRefusal {
    /// The slot's pointer names another registration,
    /// so it is not this add's to remove.
    PointerNamesAnother {
        /// The pointer's words.
        found: String,
    },
    /// A name still stands where the recipe expected an empty registration.
    EntryLeft {
        /// The name that stands.
        name: String,
    },
    /// The system refused a call.
    Io(std::io::Error),
}

impl RecipeRefusal {
    /// The sentence a person reads.
    pub fn message(&self) -> String {
        return match self {
            RecipeRefusal::PointerNamesAnother {
                found,
            } => format!(
                "the slot's `.git` holds `{found}` and not this add's registration, so nothing was removed. Ways forward: inspect the registration it names and remove the entry by hand if it is not a worktree."
            ),
            RecipeRefusal::EntryLeft {
                name,
            } => format!(
                "the registration still holds `{name}` after the recipe removed the names it knew, so it was not removed. Nothing else was changed. Ways forward: inspect the entry, remove what it is once you know, and run the command again."
            ),
            RecipeRefusal::Io(error) => format!(
                "a removal was refused ({error}), so the recipe stopped at that name and changed nothing else. Ways forward: fix the refusal and run the command again."
            ),
        };
    }
}

/// One system call's answer,
/// with `ENOENT` read as the step's own success.
pub(super) fn done_if_missing(result: std::io::Result<()>) -> std::io::Result<()> {
    return match result {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    };
}

/// Whether the bytes of a slot's `.git` name `registration`.
pub(super) fn pointer_names(
    bytes: &[u8],
    registration: &Path,
) -> bool {
    let text: String = String::from_utf8_lossy(bytes)
        .trim_end_matches('\n')
        .to_string();
    let Some(target) = text.strip_prefix("gitdir: ") else {
        return false;
    };
    return Path::new(target) == registration;
}

/// The slot's pointer,
/// removed only when its bytes name this registration.
pub(super) fn remove_the_slot_pointer(to: &ToClear) -> Result<(), RecipeRefusal> {
    let Some(slot_name) = to.slot.file_name() else {
        return Ok(());
    };
    let slot: std::fs::File = match open_directory_at(to.slot_parent, slot_name) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(RecipeRefusal::Io(error)),
    };
    let mut pointer: std::fs::File = match crate::entries_of_one_open_folder::open_entry_for_reading(
        &slot,
        OsStr::new(".git"),
    ) {
        Ok(opened) => opened,
        Err(crate::private_storage::ReadRefusal::Missing) => return Ok(()),
        Err(refusal) => {
            return Err(RecipeRefusal::Io(std::io::Error::other(format!(
                "the slot's `.git` could not be read ({refusal:?})"
            ))));
        }
    };
    let mut bytes: Vec<u8> = Vec::new();
    pointer.read_to_end(&mut bytes).map_err(RecipeRefusal::Io)?;
    if !pointer_names(&bytes, to.registration) {
        return Err(RecipeRefusal::PointerNamesAnother {
            found: String::from_utf8_lossy(&bytes)
                .trim_end_matches('\n')
                .to_string(),
        });
    }
    return done_if_missing(remove_at(&slot, OsStr::new(".git"), false)).map_err(RecipeRefusal::Io);
}

/// Remove one directory's whole contents below a descriptor,
/// then the directory itself.
pub(super) fn empty_and_remove(
    parent: &std::fs::File,
    name: &OsStr,
    first: Option<&OsStr>,
) -> std::io::Result<()> {
    let folder: std::fs::File = match open_directory_at(parent, name) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if let Some(first_name) = first {
        done_if_missing(remove_at(&folder, first_name, false))?;
    }
    for child in entry_names(&folder)? {
        remove_one_below(&folder, child.as_os_str())?;
    }
    return remove_at(parent, name, true);
}

/// Remove one entry below a descriptor,
/// a directory's whole tree when it is one.
pub(super) fn remove_one_below(
    folder: &std::fs::File,
    name: &OsStr,
) -> std::io::Result<()> {
    let metadata: std::fs::Metadata = match entry_metadata_at(folder, name) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.is_dir() {
        return empty_and_remove(folder, name, None);
    }
    return done_if_missing(remove_at(folder, name, false));
}

/// Remove `refs/` and `logs/`,
/// the files first and `logs/HEAD` first among them.
pub(super) fn remove_the_ref_store(registration: &std::fs::File) -> std::io::Result<()> {
    empty_and_remove(registration, OsStr::new("logs"), Some(OsStr::new("HEAD")))?;
    return empty_and_remove(registration, OsStr::new("refs"), None);
}

/// Remove every other entry the listing shows,
/// keeping `locked` for its own last step.
pub(super) fn remove_the_other_entries(registration: &std::fs::File) -> std::io::Result<()> {
    for child in entry_names(registration)? {
        if child == OsStr::new("locked") {
            continue;
        }
        remove_one_below(registration, child.as_os_str())?;
    }
    return Ok(());
}

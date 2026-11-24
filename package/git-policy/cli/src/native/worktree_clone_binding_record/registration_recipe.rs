//! What:
//!  The order that clears an incomplete registration Git's add left,
//!  and the one write that completes a cut-short managed lock.
//! Why:
//!  Every kill before the lock's removal must leave a locked row the finder recognises,
//!  and every kill after it a bare directory it recognises by name,
//!  so the files go before the lock and the directory goes last
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The recipe that clears rows 1 to 5, 7 and 8").
//!  The slot's pointer goes first and only when its bytes name this registration,
//!  `logs/HEAD` goes before `logs/`,
//!  `ENOENT` at any step means the name is already gone and never a failure,
//!  and each removal runs through the pinned parent descriptor rather than a path.
//!  The lock is completed in place with one truncating write,
//!  because a staging rename would leave the registration unprotected.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const step of REMOVAL_STEPS) await remove(step); // ENOENT is done
//! ```

use crate::entries_of_one_open_folder::entry_names;
use crate::fs_beneath::{entry_metadata_at, open_directory_at, remove_at};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::ffi::OsStr;
use std::io::{Read as _, Write as _};
use std::path::Path;

/// One name group the recipe removes,
/// in its fixed order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemovalStep {
    /// The slot's `.git` pointer,
    /// when its bytes name this registration.
    SlotPointer,
    /// The registration's `gitdir`.
    Gitdir,
    /// Its `commondir`.
    Commondir,
    /// Its `HEAD`.
    Head,
    /// Every file inside `refs/` and `logs/`,
    /// `logs/HEAD` first,
    /// and then those two directories.
    RefsAndLogs,
    /// Any other entry the listing showed.
    OtherEntries,
    /// `locked` last, so every earlier kill leaves an attributable row.
    Lock,
    /// The registration's own directory,
    /// by `rmdir` from the pinned parent.
    Directory,
}

/// The recipe's order,
/// which is the design's and never the caller's.
pub const REMOVAL_STEPS: [RemovalStep; 8] = [
    RemovalStep::SlotPointer,
    RemovalStep::Gitdir,
    RemovalStep::Commondir,
    RemovalStep::Head,
    RemovalStep::RefsAndLogs,
    RemovalStep::OtherEntries,
    RemovalStep::Lock,
    RemovalStep::Directory,
];

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
///  The two moments a control may stand at,
///  which the design's re-reads require.
/// Why:
///  The entry is read once before the first removal and again before the lock's,
///  and a change to the lock under the recipe is a refusal:
///  a control must be able to build that second state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Seams { beforeFirstRemoval(): void; beforeTheLock(): void }
/// ```
pub trait RecipeSeams {
    /// Runs before the first removal.
    fn before_first_removal(&mut self) -> std::io::Result<()>;
    /// Runs immediately before the lock's own removal.
    fn before_the_lock(&mut self) -> std::io::Result<()>;
}

/// The seams of a run with no control standing at either moment.
pub struct NoSeams;

impl RecipeSeams for NoSeams {
    /// Nothing stands before the first removal.
    fn before_first_removal(&mut self) -> std::io::Result<()> {
        return Ok(());
    }

    /// Nothing stands before the lock.
    fn before_the_lock(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
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
            RecipeRefusal::PointerNamesAnother { found } => format!(
                "the slot's `.git` holds `{found}` and not this add's registration, so nothing was removed. Ways forward: inspect the registration it names and remove the entry by hand if it is not a worktree."
            ),
            RecipeRefusal::EntryLeft { name } => format!(
                "the registration still holds `{name}` after the recipe removed the names it knew, so it was not removed. Nothing else was changed. Ways forward: inspect the entry, remove what it is once you know, and run the command again."
            ),
            RecipeRefusal::Io(error) => format!("a removal was refused ({error}), so the recipe stopped at that name and changed nothing else. Ways forward: fix the refusal and run the command again."),
        };
    }
}

/// One system call's answer,
/// with `ENOENT` read as the step's own success.
fn done_if_missing(result: std::io::Result<()>) -> std::io::Result<()> {
    return match result {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    };
}

/// Whether the bytes of a slot's `.git` name `registration`.
fn pointer_names(bytes: &[u8], registration: &Path) -> bool {
    let text: String = String::from_utf8_lossy(bytes).trim_end_matches('\n').to_string();
    let Some(target) = text.strip_prefix("gitdir: ") else { return false };
    return Path::new(target) == registration;
}

/// The slot's pointer,
/// removed only when its bytes name this registration.
fn remove_the_slot_pointer(to: &ToClear) -> Result<(), RecipeRefusal> {
    let Some(slot_name) = to.slot.file_name() else { return Ok(()) };
    let slot: std::fs::File = match open_directory_at(to.slot_parent, slot_name) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(RecipeRefusal::Io(error)),
    };
    let mut pointer: std::fs::File = match crate::entries_of_one_open_folder::open_entry_for_reading(&slot, OsStr::new(".git")) {
        Ok(opened) => opened,
        Err(crate::private_storage::ReadRefusal::Missing) => return Ok(()),
        Err(refusal) => return Err(RecipeRefusal::Io(std::io::Error::other(format!("the slot's `.git` could not be read ({refusal:?})")))),
    };
    let mut bytes: Vec<u8> = Vec::new();
    pointer.read_to_end(&mut bytes).map_err(RecipeRefusal::Io)?;
    if !pointer_names(&bytes, to.registration) {
        return Err(RecipeRefusal::PointerNamesAnother { found: String::from_utf8_lossy(&bytes).trim_end_matches('\n').to_string() });
    }
    return done_if_missing(remove_at(&slot, OsStr::new(".git"), false)).map_err(RecipeRefusal::Io);
}

/// Remove one directory's whole contents below a descriptor,
/// then the directory itself.
fn empty_and_remove(parent: &std::fs::File, name: &OsStr, first: Option<&OsStr>) -> std::io::Result<()> {
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
fn remove_one_below(folder: &std::fs::File, name: &OsStr) -> std::io::Result<()> {
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
fn remove_the_ref_store(registration: &std::fs::File) -> std::io::Result<()> {
    empty_and_remove(registration, OsStr::new("logs"), Some(OsStr::new("HEAD")))?;
    return empty_and_remove(registration, OsStr::new("refs"), None);
}

/// Remove every other entry the listing shows,
/// keeping `locked` for its own last step.
fn remove_the_other_entries(registration: &std::fs::File) -> std::io::Result<()> {
    for child in entry_names(registration)? {
        if child == OsStr::new("locked") {
            continue;
        }
        remove_one_below(registration, child.as_os_str())?;
    }
    return Ok(());
}

/// What:
///  Run the recipe's eight steps for one entry.
/// Why:
///  The rows differ only in which names stand,
///  and `ENOENT` is each step's own success,
///  so one run serves every row the table clears.
///
/// # Errors
/// A refusal naming what stands where the recipe expected nothing,
/// which stops the run and changes nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearRegistration(to, seams);
/// ```
pub fn clear_registration(to: &ToClear, seams: &mut dyn RecipeSeams) -> Result<(), RecipeRefusal> {
    seams.before_first_removal().map_err(RecipeRefusal::Io)?;
    for step in REMOVAL_STEPS {
        match step {
            RemovalStep::SlotPointer => remove_the_slot_pointer(to)?,
            RemovalStep::Gitdir | RemovalStep::Commondir | RemovalStep::Head => {
                let name: &str = match step {
                    RemovalStep::Gitdir => "gitdir",
                    RemovalStep::Commondir => "commondir",
                    _ => "HEAD",
                };
                let registration: std::fs::File = match open_directory_at(to.worktrees_parent, to.registration_name) {
                    Ok(opened) => opened,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                    Err(error) => return Err(RecipeRefusal::Io(error)),
                };
                done_if_missing(remove_at(&registration, OsStr::new(name), false)).map_err(RecipeRefusal::Io)?;
            }
            RemovalStep::RefsAndLogs | RemovalStep::OtherEntries | RemovalStep::Lock => {
                let registration: std::fs::File = match open_directory_at(to.worktrees_parent, to.registration_name) {
                    Ok(opened) => opened,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                    Err(error) => return Err(RecipeRefusal::Io(error)),
                };
                match step {
                    RemovalStep::RefsAndLogs => remove_the_ref_store(&registration).map_err(RecipeRefusal::Io)?,
                    RemovalStep::OtherEntries => remove_the_other_entries(&registration).map_err(RecipeRefusal::Io)?,
                    _ => {
                        seams.before_the_lock().map_err(RecipeRefusal::Io)?;
                        done_if_missing(remove_at(&registration, OsStr::new("locked"), false)).map_err(RecipeRefusal::Io)?;
                    }
                }
            }
            RemovalStep::Directory => {
                done_if_missing(remove_at(to.worktrees_parent, to.registration_name, true)).map_err(RecipeRefusal::Io)?;
            }
        }
    }
    return Ok(());
}

/// What:
///  Complete a cut-short managed lock in place.
/// Why:
///  A staging rename would leave the registration unprotected for a moment,
///  and a lock that is short still protects,
///  so one truncating write of the recomposed bytes is the completion.
///
/// # Errors
/// The lock cannot be opened or written.
pub fn complete_a_cut_short_lock(registration: &Path, nonce: &RequestNonce) -> std::io::Result<()> {
    let path: std::path::PathBuf = registration.join("locked");
    let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(super::lock_words::lock_file_bytes(nonce).as_slice())?;
    return file.sync_all();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_recipe_tests.rs"]
mod tests;

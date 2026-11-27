//! What:
//!  The swap of a finished snapshot with the destination:
//! the check that the destination is the directory Git made,
//! the exchange in one atomic step,
//! and the settling of what the exchange displaced,
//! with the moments a control may move the tree at.
//! Why:
//!  `RENAME_EXCHANGE` leaves no instant at which the destination is missing or incomplete:
//! it puts the finished subvolume at the destination and the empty directory real Git made at the stage name.
//! The names are looked up in the held parent,
//! so a rename of a name cannot redirect the step.
//! The destination may have gained files between the look and the exchange,
//! and a recursive removal would delete them:
//! only Git's pointer and an empty directory are removed,
//! every other entry is moved without replacing,
//! or stays with a note.
//! A directory that is not the one Git made is swapped back,
//! and when that fails the error says the stage name no longer holds the snapshot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const placed = putInPlace({ stage, destination, sourceMode, expected }); // throws a PlaceError
//! const notes = settleDisplaced({ stage, destination, expected, pointer });
//! ```

use crate::fs_beneath::{
    ObjectId,
    exchange_at,
    identity_at,
    open_directory_at,
    open_file_at,
    open_root,
    remove_at,
    rename_at,
};
use crate::worktree_path_failure_message::subject_failure;
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// What:
///  Do nothing;
/// the seams of `put_in_place` run this unless a control puts work there.
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

/// A placement failure with its reason,
/// marked as one where the stage is still this process's to destroy.
fn ours(reason: String) -> PlaceError {
    return PlaceError {
        reason,
        stage_is_ours: true,
    };
}

/// What:
///  Whether the entry `name` below `parent` is a directory itself;
/// an entry that cannot be examined is not.
/// Why:
///   Only the directory Git made may be swapped with the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lstat(join(parent, name)).isDirectory()
/// ```
fn is_directory_at(
    parent: &File,
    name: &OsStr,
) -> bool {
    return match crate::fs_beneath::entry_metadata_at(parent, name) {
        Ok(found) => found.is_dir(),
        Err(_unreadable) => false,
    };
}

/// What:
///  Open the parent directory and the displaced stage and the placed destination below it.
/// Why:
///   Settling works on handles,
/// so a rename of a name cannot redirect it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [openRoot(parent), openDirectoryAt(parent, stage), openDirectoryAt(parent, destination)]
/// ```
fn open_for_settling(
    parent_path: &Path,
    stage_name: &OsStr,
    destination_name: &OsStr,
) -> std::io::Result<(File, File, File)> {
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
fn file_holds(
    directory: &File,
    name: &OsStr,
    expected: &[u8],
) -> bool {
    let Ok(mut file) = open_file_at(directory, name) else {
        return false;
    };
    let mut found: Vec<u8> = Vec::new();
    return std::io::Read::read_to_end(&mut file, &mut found).is_ok() && found == expected;
}

/// Why a swap of the stage with the destination did not complete,
/// and whether the stage name still holds the snapshot.
#[derive(Debug, Eq, PartialEq)]
pub struct PlaceError {
    /// The reason,
    /// in words.
    pub reason: String,
    /// Whether the object at the stage name is the snapshot this process made.
    /// When it is not,
    /// nothing may be removed there:
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
///  Swap the finished stage with the destination,
/// after giving the stage the mode of the source's root.
/// Why:
///   `RENAME_EXCHANGE` leaves no instant at which the destination is missing or incomplete;
/// the old destination,
/// a
///  directory with Git's `.git` file,
/// ends up at the stage name for `settle_displaced`.
/// The names are looked up in the held
///  parent;
/// the destination must be the directory Git made,
/// and a directory that is not is swapped back.
/// When the swap back
///  fails,
/// the error says the stage name no longer holds the snapshot,
/// so the caller does not remove what is there.
/// The success is the object the destination names afterwards:
/// the snapshot,
/// whose identity the
///  reservation of the destination takes,
/// because the directory Git made is not there any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placed = await putInPlace({ stage, destination, sourceMode, expected }); // expected: the identity of the directory Git made
/// ```
pub fn put_in_place(
    stage: &Path,
    destination: &Path,
    source_mode: u32,
    expected: ObjectId,
) -> Result<ObjectId, PlaceError> {
    return put_in_place_with(
        stage,
        destination,
        source_mode,
        expected,
        &mut PlaceSeams {
            after_check: &mut do_nothing,
            before_swap_back: &mut do_nothing,
        },
    );
}

/// `put_in_place` with the moments at which a control may move the tree.
pub fn put_in_place_with(
    stage: &Path,
    destination: &Path,
    source_mode: u32,
    expected: ObjectId,
    seams: &mut PlaceSeams<'_>,
) -> Result<ObjectId, PlaceError> {
    let (Some(parent_path), Some(stage_name), Some(destination_name)) =
        (stage.parent(), stage.file_name(), destination.file_name())
    else {
        return Err(ours(String::from(
            "the stage and the destination have no parent",
        )));
    };
    if destination.parent() != Some(parent_path) {
        return Err(ours(String::from(
            "the stage is not beside the destination",
        )));
    }
    let parent: File = ours_on_failure(subject_failure(
        open_root(parent_path),
        &"the parent of the destination",
        "cannot be opened",
    ))?;
    let snapshot: ObjectId = ours_on_failure(subject_failure(
        identity_at(&parent, stage_name),
        &"the stage",
        "cannot be examined",
    ))?;
    if !is_directory_at(&parent, destination_name)
        || identity_at(&parent, destination_name).ok() != Some(expected)
    {
        return Err(ours(format!(
            "{} is not the directory Git made",
            destination.display()
        )));
    }
    ours_on_failure(subject_failure(
        std::fs::set_permissions(stage, std::fs::Permissions::from_mode(source_mode)),
        &"the stage",
        "cannot take the source's mode",
    ))?;
    (seams.after_check)();
    ours_on_failure(subject_failure(
        exchange_at(&parent, stage_name, destination_name),
        &"the destination",
        "cannot be swapped with the stage",
    ))?;
    // The directory displaced to the stage name must be the one that was checked; another means the destination was replaced.
    if identity_at(&parent, stage_name).ok() == Some(expected) {
        return Ok(snapshot);
    }
    (seams.before_swap_back)();
    if let Err(error) = exchange_at(&parent, stage_name, destination_name) {
        return Err(PlaceError {
            reason: format!(
                "{} changed while it was swapped and the swap could not be undone ({error})",
                destination.display()
            ),
            stage_is_ours: false,
        });
    }
    // The undo put the stage name back to what it held before only if that is the snapshot.
    let restored: bool = identity_at(&parent, stage_name).ok() == Some(snapshot);
    return Err(PlaceError {
        reason: format!(
            "{} changed while it was swapped; the swap was undone",
            destination.display()
        ),
        stage_is_ours: restored,
    });
}

/// What:
///  Settle the directory real Git made,
/// now at the stage name:
/// move what was added to it meanwhile into the new worktree,
///  remove Git's pointer when it is the one that was written,
/// remove the directory when it is empty,
/// and return what stays.
/// Why:
///   The destination may have gained files between the look and the exchange,
/// and a recursive removal would delete them.
///  Only the pointer and an empty directory are removed;
/// every other entry is moved without replacing,
/// or stays with a note.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notes = await settleDisplaced({ stage, destination, expected, pointer });
/// ```
pub fn settle_displaced(
    stage: &Path,
    destination: &Path,
    expected: ObjectId,
    pointer: &[u8],
) -> Vec<String> {
    let (Some(parent_path), Some(stage_name), Some(destination_name)) =
        (stage.parent(), stage.file_name(), destination.file_name())
    else {
        return vec![format!("{} has no parent", stage.display())];
    };
    let mut notes: Vec<String> = Vec::new();
    let opened = open_for_settling(parent_path, stage_name, destination_name);
    let Ok((parent, displaced, placed)) = opened else {
        return vec![format!(
            "{} could not be opened to be settled",
            stage.display()
        )];
    };
    if crate::fs_beneath::identity_of(&displaced).ok() != Some(expected) {
        return vec![format!(
            "{} is not the directory Git made and stays",
            stage.display()
        )];
    }
    let listing: Vec<OsString> = match std::fs::read_dir(crate::fs_beneath::handle_path(&displaced))
    {
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
                    notes.push(format!(
                        "Git's pointer in {} cannot be removed ({error})",
                        stage.display()
                    ));
                }
            } else {
                notes.push(format!(
                    ".git in {} is not the pointer Git wrote and stays",
                    stage.display()
                ));
            }
            continue;
        }
        if let Err(error) = rename_at(
            &displaced,
            name.as_os_str(),
            &placed,
            name.as_os_str(),
            false,
        ) {
            notes.push(format!(
                "{} was added to {} while the worktree was made and stays in {} ({error})",
                name.to_string_lossy(),
                destination.display(),
                stage.display()
            ));
        }
    }
    if let Err(error) = remove_at(&parent, stage_name, true) {
        notes.push(format!("{} is not removed ({error})", stage.display()));
    }
    return notes;
}

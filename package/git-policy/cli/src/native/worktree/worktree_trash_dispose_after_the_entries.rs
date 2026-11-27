//! What:
//!  What the emptying does once a trash's entries are gone:
//!  remove the manifest the wrapper wrote inside it,
//!  the trash directory itself,
//!  and the parent above it when nothing else is in it.
//! Why:
//!  Anything the snapshot did not put there keeps the whole directory,
//!  so nothing of another command is removed by mistake,
//!  and a parent that still holds another trash stays.
//!  The manifest goes only while the name still holds the object whose bytes were read as that manifest:
//!  a name can be taken between the read and the unlink,
//!  so comparing the identity means the file that is removed is one that was examined.
//!  `worktree_trash_dispose.rs` re-exports the step,
//!  and the emptying's two faces call it last.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function settleTrashRemoval(held: HeldTrash): void; // throws the reason something stays
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{
    handle_path,
    remove_at,
};
use crate::idle_io_priority::with_idle_io_priority;
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_dispose_the_reason_a_trash_stays::TrashLeft;
use crate::worktree_trash_dispose_the_trash_held_open::HeldTrash;
use std::fs::File;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashDispose";

/// The trash's own name,
/// the last component of its path,
/// for the manifest inside it.
fn trash_name_of(inside: &Path) -> String {
    let Some(name) = inside.file_name() else {
        return String::new();
    };
    return name.to_string_lossy().into_owned();
}

/// Whether an error says a directory is not empty.
fn is_not_empty(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(libc::ENOTEMPTY | libc::EEXIST));
}

/// What:
///  Remove the trash's own directory,
/// which only its permissions kept.
/// Why:
///   The name inside its parent is the trash's name,
/// so the removal deletes exactly what the snapshot made and nothing beside it.
fn remove_the_trash_parent(state: (&File, &Path)) -> std::io::Result<()> {
    let (parent, inside) = state;
    return remove_at(
        parent,
        std::ffi::OsStr::new(inside.file_name().unwrap_or_default()),
        true,
    );
}

/// What:
///  Remove the manifest file from the trash,
///  only while the name still holds the object whose bytes were read as that manifest.
/// Why:
///   A name can be taken between the read and the unlink,
/// and nothing in the kernel makes the two one step;
/// comparing the
///  identity means the file that is removed is one that was examined,
/// and a file another process moved onto the name
///  keeps the whole directory and is reported instead.
/// It is visible to this folder because the emptying's controls
/// (`worktree_trash_dispose_tests.rs`) ask only this step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeTheManifest(trash, inside, expected) { const outcome = removeIfIdentical(trash, '.manifest', expected); if (outcome !== 'removed' && outcome !== 'absent') throw kept(`the manifest inside ${inside} ...`); }
/// ```
pub(crate) fn remove_the_manifest(
    trash: &File,
    inside: &Path,
    expected: crate::fs_beneath::ObjectId,
) -> Result<(), TrashLeft> {
    return match crate::fs_beneath::remove_if_identical(
        trash,
        std::ffi::OsStr::new(crate::the_names_and_the_manifest_of_the_trash::MANIFEST_FILE_NAME),
        expected,
    ) {
        Ok(crate::fs_beneath::Owned::Removed | crate::fs_beneath::Owned::Absent) => Ok(()),
        Ok(crate::fs_beneath::Owned::Kept(reason)) => Err(TrashLeft::kept(format!(
            "the manifest inside {} is not the file that was read as it ({reason}), so the trash directory is left alone",
            inside.display()
        ))),
        Err(error) => Err(TrashLeft::kept(format!(
            "the manifest inside {} cannot be removed ({error}), so the trash directory is left alone",
            inside.display()
        ))),
    };
}

/// What:
///  Remove the trash directory itself once its entries are gone,
/// and the parent above it when nothing else is in it.
/// Why:
///   Anything the snapshot did not put there keeps the whole directory,
/// so nothing of another command is removed by mistake,
///  and a parent that still holds another trash stays.
/// It is visible to this folder because the emptying's two faces
///  (`worktree_trash_dispose.rs`) are what call it last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleTrashRemoval(held: HeldTrash): void; // throws the reason something stays
/// ```
pub(crate) fn settle_trash_removal(held: HeldTrash) -> Result<(), TrashLeft> {
    let HeldTrash {
        holders,
        trash,
        inside,
        shown,
        names,
    } = held;
    let own: Option<crate::the_names_and_the_manifest_of_the_trash::OwnManifest> =
        crate::worktree_trash_own_manifest::its_own(
            &trash,
            trash_name_of(inside.as_path()).as_str(),
        );
    let mut strangers: Vec<String> = Vec::new();
    for entry in path_failure(
        std::fs::read_dir(handle_path(&trash)),
        inside.as_path(),
        "cannot be listed",
    )
    .map_err(TrashLeft::kept)?
    .flatten()
    {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        // The manifest the wrapper wrote inside its own trash goes with it, and only while the file at that name is the object the
        //  bytes were read through; a file of that name that is not that manifest keeps the whole directory, as every other thing
        //  the snapshot did not put there does.
        if own.is_some()
            && name == crate::the_names_and_the_manifest_of_the_trash::MANIFEST_FILE_NAME
        {
            continue;
        }
        strangers.push(name);
    }
    if !strangers.is_empty() {
        return Err(TrashLeft::kept(format!(
            "{} also holds {}, which the snapshot did not put there, so the trash directory is left alone",
            inside.display(),
            strangers.join(", ")
        )));
    }
    if let Some(leaving) = own.as_ref() {
        remove_the_manifest(&trash, inside.as_path(), leaving.file)?;
    }
    drop(trash);
    let Some(parent) = holders.last() else {
        return Ok(());
    };
    // Everything the trash held is gone, so only its parent's permissions kept it: a removal that names it deletes
    //  exactly what the snapshot made.
    path_failure(
        with_idle_io_priority((parent.as_ref(), inside.as_path()), remove_the_trash_parent),
        inside.as_path(),
        "cannot be removed",
    )
    .map_err(TrashLeft::kept)?;
    // The parent goes when nothing else is in it; a refusal only means another trash directory is still there.
    if let [.., above, _] = holders.as_slice()
        && let Some(last) = names.last()
        && let Err(error) = remove_at(above.as_ref(), last.as_os_str(), true)
        && !is_not_empty(&error)
    {
        debug(
            TAG,
            format!("{} cannot be removed ({error}); it stays", shown.display()).as_str(),
        );
    }
    return Ok(());
}

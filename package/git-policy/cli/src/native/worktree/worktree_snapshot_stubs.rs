//! What:
//!  The stubs a snapshot holds for what must not be carried:
//! the empty directories nested worktrees leave,
//! the destination's own directory when it lies inside the source,
//! and the `.git` pointer the new worktree takes in place of the source's.
//! Why:
//!  A nested worktree that is a subvolume is an empty directory in the snapshot,
//! and the destination was made by Git before the snapshot,
//! so a source that contains it holds a directory with Git's `.git` file there.
//! Neither belongs in the new worktree.
//! Every directory on the way is opened without following a link and on the snapshot's own device,
//! so a registered path that reaches the snapshot through a link is an error and never an unlink in the tree the link points at.
//! A copied `.git` directory holds the whole common directory,
//! so it goes into the trash the commit service empties after the add returns rather than being removed one entry at a time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removeStubs({ stage, source, plan, handOver }); // throws
//! replaceGitPointer({ stage, pointer, source, handOver });
//! ```

use crate::fs_beneath::{
    open_directory_at,
    open_root,
    plain_components,
    remove_at,
};
use crate::subvolume_remove::remove_tree;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_placeholders::{
    HandOver,
    open_below_stand_ins,
    remove_snapshot_placeholders,
};
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash::is_directory_itself;
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};

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
///  Remove one stub from the snapshot below the held handle of its root:
/// the directory `path` names (a path inside the
///  source),
/// after its `.git` pointer when `pointer` is set.
/// Why:
///   Every directory on the way is opened without following a link and on the snapshot's own device,
/// so a registered
///  path that reaches the snapshot through a link (a link left where a directory was moved) is an error and never an
///  unlink in the tree the link points at.
/// A path below a stand-in of a nested subvolume names nothing:
/// such a stand-in holds nothing,
/// so the stub a registered worktree left there went with the folder when the stand-in step took it.
/// A nested subvolume's stub is removed only as an empty directory;
/// only the
///  destination's own directory,
/// which Git made,
/// holds a pointer file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeStub(root, sourceRoot, path, pointer);
/// ```
pub(crate) fn remove_stub(
    root: &File,
    source_root: &Path,
    path: &Path,
    pointer: bool,
) -> Result<(), String> {
    let Ok(relative) = path.strip_prefix(source_root) else {
        return Ok(());
    };
    let shown: String = path.display().to_string();
    let names: Vec<OsString> = subject_failure(
        plain_components(relative),
        &shown,
        "is not a plain path inside the source",
    )?;
    let Some((name, parents)) = names.split_last() else {
        return Ok(());
    };
    let Some(parent) = open_below_stand_ins(root, parents, &shown)? else {
        return Ok(());
    };
    if pointer {
        match open_directory_at(&parent, name.as_os_str()) {
            Ok(stub) => match remove_at(&stub, OsStr::new(".git"), false) {
                Ok(()) => {}
                Err(error) if is_gone(&error) => {}
                Err(error) => {
                    return Err(format!(
                        "{shown}/.git cannot be removed from the snapshot ({error})"
                    ));
                }
            },
            Err(error) if is_gone(&error) => return Ok(()),
            Err(error) => {
                return Err(format!(
                    "{shown} cannot be entered in the snapshot ({error})"
                ));
            }
        }
    }
    return match remove_at(&parent, name.as_os_str(), true) {
        Ok(()) => Ok(()),
        Err(error) if is_gone(&error) => Ok(()),
        Err(error) => Err(format!(
            "{shown} cannot be removed from the snapshot as an empty directory ({error})"
        )),
    };
}

/// What:
///  Remove the stub that each nested worktree leaves in the snapshot,
/// and the destination's own directory when it lies
///  inside the source,
/// after the stand-ins the snapshot holds for subvolumes no worktree names;
/// and answer the trash the add
///  hands to the service.
/// Why:
///   A nested worktree that is a subvolume is an empty directory in the snapshot;
/// the destination was made by Git before
///  the snapshot,
/// so a source that contains it holds a directory with Git's `.git` file there.
/// Neither belongs in the new
///  worktree,
/// and neither does the stand-in of a stage container of an add in flight
/// (`worktree_snapshot_placeholders.rs`).
/// The stand-in step runs first because it takes a folder that holds
///  nothing but stand-ins whole,
/// which is one rename and one folder made again rather than one removal for each
///  worktree of the repository;
/// a stub it left behind with a folder that holds somebody else's entry is then removed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trash = await removeStubs({ stage, sourceRoot, nested, destination, trash });
/// ```
pub(crate) fn remove_stubs(
    stage: &Path,
    source: &SnapshotSource,
    plan: &AddPlan,
    hand_over: &mut HandOver,
) -> Result<(), String> {
    let root: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    remove_snapshot_placeholders(&root, source, plan, hand_over)?;
    for nested in &source.nested {
        remove_stub(&root, source.root.as_path(), nested.as_path(), false)?;
    }
    remove_stub(
        &root,
        source.root.as_path(),
        plan.destination.as_path(),
        true,
    )?;
    return Ok(());
}

/// What:
///  Put the new worktree's `.git` pointer into the snapshot in place of the source's `.git` (a directory for a main
///  checkout,
/// a pointer file for a linked worktree).
/// Why:
///   The bytes are the ones real Git wrote for the new worktree,
/// so the pointer is exactly Git's.
/// A copied `.git` directory holds the
///  whole common directory,
/// with one administrative directory for every worktree of the repository,
/// so removing it here
///  would cost one btrfs metadata change for each of them on the add's waiting path;
/// it goes into the trash the leave-out and
///  the stand-ins go into,
/// which the commit service empties after the add returns.
/// Nothing the new worktree needs lives in it once the
///  index is installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await replaceGitPointer({ stage, pointer, source, handOver });
/// ```
pub(crate) fn replace_git_pointer(
    stage: &Path,
    pointer: &[u8],
    source: &SnapshotSource,
    hand_over: &mut HandOver,
) -> Result<(), String> {
    let root: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    let path: PathBuf = stage.join(".git");
    let taken: bool = is_directory_itself(path.as_path())
        && hand_over.take_entry(&root, source, &root, OsStr::new(".git"), Path::new(".git"))?;
    if !taken {
        subject_failure(
            remove_tree(path.as_path()),
            &"the source's .git",
            "cannot be removed from the snapshot",
        )?;
    }
    return subject_failure(
        std::fs::write(path.as_path(), pointer),
        &".git pointer",
        "cannot be written",
    );
}

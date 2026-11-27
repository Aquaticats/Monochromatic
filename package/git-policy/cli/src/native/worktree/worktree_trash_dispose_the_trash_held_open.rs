//! What:
//!  The directories the emptying holds open from the worktree down to one trash,
//!  and the proof that the trash is the directory the snapshot made.
//! Why:
//!  Every directory stays open by its own name,
//!  so nothing below the handles can be replaced by a link between two steps,
//!  and every step is one plain name below the handle of the one above.
//!  A trash that is already gone is done,
//!  and a worktree that is gone is done too:
//!  a removal of that worktree took the trash with it.
//!  `worktree_trash_dispose.rs` re-exports these, so it and its own faces name them there,
//!  and the step that removes what the entries left (`worktree_trash_dispose_after_the_entries.rs`)
//!  reads the handles this file holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type HeldTrash = { holders: File[]; trash: Arc<File>; inside: string; shown: string; names: OsString[] };
//! type ATrashToOpen = { repository: string; gitDir: string; manifest: TrashManifest };
//! function prepareTrashRemoval({ repository, gitDir, manifest }): HeldTrash | undefined;
//! ```

use crate::fs_beneath::{
    device_of,
    identity_of,
    open_directory_at,
    open_root,
    plain_components,
};
use crate::the_names_and_the_manifest_of_the_trash::TRASH_PARENT;
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_dispose_the_worktree_is_the_recorded_one::verify_association;
use crate::worktree_trash_manifest::TrashManifest;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;

/// What:
///  The trash of one worktree as the removal holds it:
/// the directories opened on the way down,
/// the trash itself,
/// the path every message names,
/// and the names walked to reach it.
/// Why:
///   Every directory stays open by its own name,
/// so nothing below the handles can be replaced by a link between two steps;
/// and the entries go before the directory that holds them,
/// so the two are separate steps of one removal,
///  each of which may be a job of the blocking pool.
/// Its fields are visible to this folder because the step after the entries
///  (`worktree_trash_dispose_after_the_entries.rs`) spends the same handles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldTrash = { holders: File[]; trash: Arc<File>; inside: string; shown: string; names: OsString[] };
/// ```
#[derive(Clone)]
pub(crate) struct HeldTrash {
    /// The worktree's root and every directory below it on the way to the trash parent,
    /// shared so a job the system refused a thread for can be done in turn with the same handles.
    pub(crate) holders: Vec<Arc<File>>,
    /// The trash directory,
    /// held open.
    pub(crate) trash: Arc<File>,
    /// The trash's path,
    /// which every message names.
    pub(crate) inside: PathBuf,
    /// The path the walk down has reached.
    pub(crate) shown: PathBuf,
    /// The plain names walked from the root to the trash parent.
    pub(crate) names: Vec<std::ffi::OsString>,
}

/// What:
///  The removal of one trash as the state of one job of the blocking pool.
/// Why:
///   The budget runner takes a function and the state it reads,
/// never a closure,
/// and this state is what an awaiting caller hands the pool the moment its handles are open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ATrashToOpen = { repository: string; gitDir: string; manifest: TrashManifest };
/// ```
#[derive(Clone)]
pub(crate) struct ATrashToOpen {
    /// The worktree that holds the trash.
    pub(crate) repository: PathBuf,
    /// That worktree's administrative directory.
    pub(crate) git_dir: PathBuf,
    /// What the leave-out made and moved.
    pub(crate) manifest: TrashManifest,
}

/// What:
///  Open every directory from the worktree down to the trash and prove the trash is the one the snapshot made,
/// or answer nothing when it is already gone.
/// Why:
///   These are lookups and identities,
/// and they come before the first removal:
/// a trash that is gone is done,
///  and a directory that is not the recorded object is left alone.
/// A worktree that is gone is done too:
/// a removal of that worktree took
///  the trash with it,
/// so a record that outlived it names nothing on this disk and needs no report.
/// It is visible to this folder because the emptying's two faces
///  (`worktree_trash_dispose.rs`) are what hand it the state it reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTrashRemoval({ repository, gitDir, manifest }): HeldTrash | undefined;
/// ```
pub(crate) fn prepare_trash_removal(
    ATrashToOpen {
        repository,
        git_dir,
        manifest,
    }: ATrashToOpen
) -> Result<Option<HeldTrash>, String> {
    let root: File = match open_root(repository.as_path()) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "{} cannot be opened as the worktree ({error})",
                repository.display()
            ));
        }
    };
    verify_association(&root, repository.as_path(), git_dir.as_path())?;
    let device: u64 = path_failure(device_of(&root), repository.as_path(), "cannot be examined")?;
    let names = match plain_components(Path::new(TRASH_PARENT)) {
        Ok(found) => found,
        Err(error) => return Err(format!("the trash parent is not a plain path ({error})")),
    };
    let mut shown: PathBuf = repository.clone();
    let mut holders: Vec<Arc<File>> = vec![Arc::new(root)];
    for name in &names {
        shown.push(name);
        let Some(held) = holders.last() else {
            return Err(format!("{} cannot be held", shown.display()));
        };
        let next: File = match open_directory_at(held.as_ref(), name.as_os_str()) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "{} is not a plain directory ({error}), so the trash below it is left alone",
                    shown.display()
                ));
            }
        };
        if device_of(&next).ok() != Some(device) {
            return Err(format!(
                "{} is on another device, so the trash below it is left alone",
                shown.display()
            ));
        }
        holders.push(Arc::new(next));
    }
    let Some(parent) = holders.last() else {
        return Ok(None);
    };
    let trash: Arc<File> = match open_directory_at(
        parent.as_ref(),
        std::ffi::OsStr::new(manifest.name.as_str()),
    )
    .map(Arc::new)
    {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "{} is not a plain directory ({error}), so it is left alone",
                shown.join(&manifest.name).display()
            ));
        }
    };
    let found = path_failure(
        identity_of(&trash),
        shown.join(&manifest.name).as_path(),
        "cannot be examined",
    )?;
    if found != manifest.trash {
        return Err(format!(
            "{} is not the directory the snapshot made ({} instead of {}), so it is left alone",
            shown.join(&manifest.name).display(),
            found.encode(),
            manifest.trash.encode()
        ));
    }
    let inside: PathBuf = shown.join(&manifest.name);
    return Ok(Some(HeldTrash {
        holders,
        trash,
        inside,
        shown,
        names,
    }));
}

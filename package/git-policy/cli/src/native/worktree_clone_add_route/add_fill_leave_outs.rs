//! What:
//!  The fill's leave-outs:
//! the trash the clone's own content is set aside in,
//! the
//!  record of what it holds,
//! and the trash a resumed add finds.
//! Why:
//!  The clone carries files the caller's checkout holds and the new worktree must
//!  not,
//! and each one goes into the trash rather than being removed,
//! so a kill
//!  after the leave-out leaves a record of what was set aside
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore",
//! step 5).
//!  A resumed add finds the trash its own earlier run made and reuses it instead
//!  of leaving a second one.
//!  `add_fill.rs` runs the fill in order and would go over the file-size limit
//!  carrying these,
//!  and the trash's own words are the wrapper's trash module's,
//! so this file is
//!  the leave-out and its record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const manifest = leaveOutTheClone(input); // the snapshot's own trash
//! const found = theExistingTrash(cloneRoot); // one trash, or none
//! ```

use super::add_fill_input_and_result::FillInput;
use crate::fs_beneath::open_root;
use crate::worktree_snapshot_placeholders::{
    HandOver,
    remove_snapshot_placeholders,
};
use crate::worktree_trash::{
    Pruned,
    TRASH_PARENT,
    TrashManifest,
    is_trash_name,
    prune_snapshot,
};
use crate::worktree_trash_own_manifest;
use std::path::{
    Path,
    PathBuf,
};

/// The trash the leave-out makes for the clone,
/// written so a kill after it
/// leaves a record of what it set aside.
pub(super) fn leave_out_the_clone(input: &FillInput<'_>) -> Result<Option<TrashManifest>, String> {
    let pruned: Pruned = prune_snapshot(input.clone_root, input.source)?;
    let mut hand_over: HandOver = HandOver::starting(pruned.trash);
    let root: std::fs::File = open_root(input.clone_root).map_err(|error| {
        return format!(
            "the clone root {} cannot be opened without a link ({error})",
            input.clone_root.display()
        );
    })?;
    remove_snapshot_placeholders(&root, input.source, input.plan, &mut hand_over)?;
    let manifest: Option<TrashManifest> = hand_over.finish();
    if let Some(written) = manifest.as_ref() {
        worktree_trash_own_manifest::write(input.clone_root, written)?;
    }
    return Ok(manifest);
}

/// The trash an earlier run of this add left,
/// when one stands.
pub(super) fn the_existing_trash(clone_root: &Path) -> Result<Option<TrashManifest>, String> {
    let parent: PathBuf = clone_root.join(TRASH_PARENT);
    let listing: std::fs::ReadDir = match std::fs::read_dir(parent.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "the trash place {} cannot be listed ({error})",
                parent.display()
            ));
        }
    };
    let mut found: Vec<TrashManifest> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| {
            return format!(
                "the trash place {} cannot be listed ({error})",
                parent.display()
            );
        })?;
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if !is_trash_name(name.as_str()) {
            continue;
        }
        let Some(manifest) = worktree_trash_own_manifest::read(clone_root, name.as_str()) else {
            return Err(format!(
                "the trash {name} under {} holds no record this add wrote, so nothing proves what it is; Ways forward: inspect it and remove it if it is not the add's",
                parent.display()
            ));
        };
        found.push(manifest);
    }
    return match found.len() {
        0 => Ok(None),
        1 => Ok(Some(found.remove(0))),
        _several => Err(format!(
            "the clone at {} holds more than one trash, and one add leaves one; Ways forward: inspect them and remove the ones this add did not make",
            clone_root.display()
        )),
    };
}

/// The trash a filled clone holds,
/// for the resume that finds the exchange already done.
pub fn the_trash_of_a_filled_clone(clone_root: &Path) -> Result<Option<TrashManifest>, String> {
    return the_existing_trash(clone_root);
}

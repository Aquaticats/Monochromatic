//! What:
//!  The identity of the object at a path beside a worktree:
//! the directory held open so that its names keep meaning the same objects,
//! the object at a name,
//! the object at a conversion's converting name,
//! and the object with its birth time.
//! Why:
//!  A conversion swaps and removes only the object its journal recorded,
//! so every step asks the same question of the same held directory.
//! A link is never followed and a name that is free answers nothing,
//! which is not an error;
//! a name that cannot be examined is one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parent = heldParent(path); // throws
//! const identity = identityOfPath(path); // ObjectId | undefined
//! const stage = stageIdentityOf(path); // ObjectId | undefined
//! ```

use crate::fs_beneath::{
    ObjectId,
    identity_at,
    open_root,
};
use crate::subvolume_error::MigrateError;
use crate::worktree_convert_inspection::converting_path;
use crate::worktree_convert_journal::io_step;
use std::path::{
    Path,
    PathBuf,
};

/// The directory that holds `path`,
/// held open so that names below it mean the same objects for as long as it is held.
pub fn held_parent(path: &Path) -> Result<std::fs::File, MigrateError> {
    let Some(parent) = path.parent() else {
        return Err(MigrateError::Journal(format!(
            "{} has no parent directory",
            path.display()
        )));
    };
    return io_step(
        open_root(parent),
        "hold the directory of the worktree",
        parent,
    );
}

/// The identity of the object at `path`,
/// a link not followed,
/// or nothing when it cannot be examined.
pub fn identity_of_path(path: &Path) -> Option<ObjectId> {
    let parent: std::fs::File = held_parent(path).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// The identity,
/// with the birth time,
/// of the directory at `path`.
pub fn stage_free_identity(path: &Path) -> Result<ObjectId, MigrateError> {
    let parent: std::fs::File = held_parent(path)?;
    return io_step(
        identity_at(&parent, path.file_name().unwrap_or_default()),
        "examine",
        path,
    );
}

/// The identity of the object at the converting name of `path`,
/// or nothing when the name is free.
pub fn stage_identity_of(path: &Path) -> Result<Option<ObjectId>, MigrateError> {
    let converting: PathBuf = converting_path(path);
    let parent: std::fs::File = held_parent(path)?;
    return match identity_at(&parent, converting.file_name().unwrap_or_default()) {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(MigrateError::io("examine", converting.as_path(), &error)),
    };
}

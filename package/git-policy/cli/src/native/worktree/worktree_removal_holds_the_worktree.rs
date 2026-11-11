//! What:
//!  The directories one removal holds open from before the proof of cleanliness to the destroy,
//! the last look at them
//!  before the destroy,
//! and the lock that keeps a worktree.
//! Why:
//!  A descriptor names the object it was opened on,
//! whatever happens to the path afterwards,
//! so every check after
//!  the proof of cleanliness is about these objects and never about the paths that name them.
//! The lock and the four
//!  identities are looked at once more immediately before the destroy,
//! and any difference leaves the removal to Git with
//!  nothing changed (`worktree_remove.rs`).
//! They live here,
//! apart from the removal's own steps,
//! because the checks are
//!  the whole of what this file does
//! and the removal is long enough without them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = hold(target, registration); // Held
//! confirm(held, registration);              // throws the reason the names are not the proven objects any more
//! ```

use crate::fs_beneath::{ObjectId, handle_path, identity_at, identity_of, open_directory_at, open_file_at, open_root};
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_remove_registry::Registration;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// What:
///  Whether the administrative directory holds a `locked` entry.
/// Why:
///   A lock is a promise to keep the worktree.
/// Only an entry that is certainly absent (`NotFound`) means unlocked;
/// any
///  other failure to look it up leaves the lock unknown,
/// and an unknown lock is not a reason to destroy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockState(admin: string): boolean; // throws when the entry cannot be looked up
/// ```
pub(crate) fn lock_state(admin: &Path) -> Result<bool, String> {
    let entry: PathBuf = admin.join("locked");
    return match entry.symlink_metadata() {
        Ok(_found) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("the lock entry {} cannot be looked up ({error}), so the worktree is not known to be unlocked", entry.display())),
    };
}


/// What:
///  The directories one removal holds open from before the proof of cleanliness to the destroy,
/// and the identities they
///  had when they were looked at.
/// Why:
///   A descriptor names the object it was opened on,
/// whatever happens to the path afterwards;
/// the identities let the
///  last look before the destroy tell a swapped name from the held object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Held = { parent: FileHandle; name: string; target: FileHandle; identity: ObjectId; admin: FileHandle; ... };
/// ```
#[derive(Debug)]
pub struct Held {
    /// The directory that holds the worktree directory.
    pub parent: File,
    /// The canonical path of `parent`,
    /// for messages and for the check that the path still names it.
    pub parent_path: PathBuf,
    /// The name of the worktree directory in `parent`.
    pub name: OsString,
    /// The worktree directory itself (the root of its subvolume).
    pub target: File,
    /// The identity of `target` when it was opened.
    pub identity: ObjectId,
    /// The identity of `parent` when it was opened.
    pub parent_identity: ObjectId,
    /// The `worktrees` directory of the repository.
    pub worktrees: File,
    /// The administrative directory of the worktree.
    pub admin: File,
    /// The identity of `admin` when it was opened.
    pub admin_identity: ObjectId,
}


/// What:
///  Open the worktree directory,
/// its parent,
/// and its registration,
/// and record what each one is.
/// Why:
///   Everything after this point is about these objects,
/// not about the paths that name them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function hold(target: string, registration: Registration): Promise<Held>; // throws the reason
/// ```
/// A step of holding the worktree's directories with its failure worded as a refusal to open `what` at `path` without
///  following a link.
fn refused<T>(outcome: std::io::Result<T>, what: &str, path: &Path) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{what} {} cannot be opened without following a link ({error})", path.display())),
    };
}

/// A read of a held object's identity with its failure worded as the identity of `target` being unreadable.
fn identity_unreadable<T>(outcome: std::io::Result<T>, target: &Path) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("the identity of {} cannot be read ({error})", target.display())),
    };
}

pub(crate) fn hold(target: &Path, registration: &Registration) -> Result<Held, String> {
    let Some(parent_of_target) = target.parent() else {
        return Err(format!("{} has no parent directory", target.display()));
    };
    let parent_path: PathBuf = parent_of_target.to_path_buf();
    let Some(last_component) = target.file_name() else {
        return Err(format!("{} has no last component", target.display()));
    };
    let name: OsString = last_component.to_os_string();
    let parent: File = refused(open_root(parent_path.as_path()), "the directory", parent_path.as_path())?;
    let held_target: File = refused(open_directory_at(&parent, name.as_os_str()), "the worktree", target)?;
    let worktrees: File = refused(open_root(registration.worktrees.as_path()), "the directory", registration.worktrees.as_path())?;
    let admin: File = refused(open_directory_at(&worktrees, registration.id.as_os_str()), "the registration", registration.admin.as_path())?;
    return Ok(Held {
        identity: identity_unreadable(identity_of(&held_target), target)?,
        parent_identity: identity_unreadable(identity_of(&parent), target)?,
        admin_identity: identity_unreadable(identity_of(&admin), target)?,
        parent,
        parent_path,
        name,
        target: held_target,
        worktrees,
        admin,
    });
}

/// What:
///  The last look before the destroy:
/// the lock,
/// and that the names still lead to the objects that were held and proven.
/// Why:
///   The proof of cleanliness takes time (it starts Git);
/// a lock taken,
/// a directory moved away and another put in its
///  place,
/// an ancestor replaced by a link,
/// or a registration replaced in that time must stop the destroy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function confirm(held: Held, registration: Registration): Promise<void>; // throws the reason
/// ```
pub(crate) fn confirm(held: &Held, registration: &Registration) -> Result<(), String> {
    let shown: PathBuf = held.parent_path.join(held.name.as_os_str());
    if lock_state(handle_path(&held.admin).as_path())? {
        return Err(format!("{} was locked after it was proven clean", shown.display()));
    }
    let named_now: ObjectId = path_failure(identity_at(&held.parent, held.name.as_os_str()), shown.as_path(), "cannot be looked up again")?;
    if named_now != held.identity {
        return Err(format!("{} is not the directory that was proven clean any more", shown.display()));
    }
    let parent_now: File = path_failure(open_root(held.parent_path.as_path()), held.parent_path.as_path(), "cannot be opened again without following a link")?;
    if path_failure(identity_of(&parent_now), held.parent_path.as_path(), "cannot be looked up again")? != held.parent_identity {
        return Err(format!("{} is not the directory that held the worktree any more", held.parent_path.display()));
    }
    let admin_now: ObjectId = path_failure(identity_at(&held.worktrees, registration.id.as_os_str()), registration.admin.as_path(), "cannot be looked up again")?;
    if admin_now != held.admin_identity {
        return Err(format!("{} is not the registration that was proven any more", registration.admin.display()));
    }
    let recorded: Vec<u8> = path_failure(read_held_file(held, "gitdir"), registration.admin.join("gitdir").as_path(), "cannot be read again")?;
    if recorded != registration.gitdir_text {
        return Err(format!("{} changed after it was proven", registration.admin.join("gitdir").display()));
    }
    return Ok(());
}

/// The bytes of a file of the held administrative directory.
fn read_held_file(held: &Held, name: &str) -> std::io::Result<Vec<u8>> {
    let mut file: File = open_file_at(&held.admin, std::ffi::OsStr::new(name))?;
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes)?;
    return Ok(bytes);
}

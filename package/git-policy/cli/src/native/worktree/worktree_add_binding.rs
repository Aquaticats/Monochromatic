//! What:
//!  The binding of one `git worktree add` to the objects real Git made for it (the destination directory and the
//!  administrative directory that registers it),
//! the record that carries the binding,
//! and the check that the binding still
//!  holds.
//! Why:
//!  The wrapper completes an add in place,
//! through the bound objects,
//! and never deletes or unregisters a worktree:
//! a path
//!  can be changed by another process between any check and any act,
//! so no pathname or identity recorded earlier proves the
//!  right to remove something later.
//! The binding is what a fast rung and the fallback checkout verify before they write into
//!  the worktree,
//! and what the report of an interrupted add names.
//! It is captured right after Git returned,
//! and an add
//!  whose pair cannot be proven is left exactly as Git made it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bound = await bindRegistration(commonDir, destination); // throws when the pair is not proven
//! await claim.recordBinding(bound.binding);
//! await verifyAssociation(bound); // throws when the objects are no longer where and what they were
//! ```

use crate::fs_beneath::{
    identity_of,
    open_root,
};
pub use crate::worktree_add_binding_record::{
    Binding,
    OwedCopy,
    OwedSelection,
    Recorded,
    decode_path,
    decode_record,
    encode_path,
    owed_copy_of,
    planned_record,
    planned_record_with,
    registered_record_with,
    running_child_of,
};
pub use crate::worktree_add_binding_registration_owner::{
    REGISTRATION_OWNER_NAME,
    RegistrationOwner,
    claim_registration,
};
pub use crate::worktree_add_binding_registration_path::path_identity;
use crate::worktree_add_binding_registration_path::{
    not_canonical,
    read_through,
    recorded_destination,
};
pub(crate) use crate::worktree_add_binding_registration_path::{
    pointer_target,
    resolve_lexically,
};
use crate::worktree_path_failure_message::subject_failure;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};

/// A binding with the two directories held open.
#[derive(Debug)]
pub struct Bound {
    /// The binding,
    /// for the record.
    pub binding: Binding,
    /// The destination directory as it was when it was bound.
    pub destination: File,
    /// The administrative directory as it was when it was bound.
    pub admin: File,
}

/// What:
///  Bind an add to the directories Git just made for it,
/// after proving that they belong together and to this repository.
/// Why:
///   The pointer must name an administrative directory directly below this repository's `worktrees`,
/// and that directory's
///  `gitdir` must name the pointer back;
/// only then are the identities of both directories and the pointer bytes captured,
///  with both directories held open.
/// A pointer edited to name another worktree's administrative directory fails the
///  back-link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = await bindRegistration(commonDir, destination); // throws a reason
/// ```
pub fn bind_registration(
    common_dir: &Path,
    destination: &Path,
) -> Result<Bound, String> {
    let shown: String = destination.display().to_string();
    let pointer_shown: String = format!("{shown}/.git");
    let registration_shown: String = format!("the registration of {shown}");
    let held: File = subject_failure(open_root(destination), &shown, "is not a real directory")?;
    let pointer: Vec<u8> = subject_failure(
        read_through(&held, OsStr::new(".git")),
        &pointer_shown,
        "is unreadable",
    )?;
    let pointer_path: PathBuf = destination.join(".git");
    let admin_path: PathBuf = pointer_target(destination, pointer.as_slice())?;
    // The target is opened as a canonical root first: a pointer through a link is refused here, with its own message.
    let admin: File = match open_root(admin_path.as_path()) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(not_canonical(
                shown.as_str(),
                admin_path.as_path(),
                error.to_string().as_str(),
            ));
        }
    };
    let worktrees: PathBuf = subject_failure(
        std::fs::canonicalize(common_dir.join("worktrees")),
        &"the worktrees of the repository",
        "are unreadable",
    )?;
    if admin_path.parent() != Some(worktrees.as_path()) {
        return Err(format!("{shown} is not registered in this repository"));
    }
    let recorded: Vec<u8> = subject_failure(
        read_through(&admin, OsStr::new("gitdir")),
        &registration_shown,
        "is unreadable",
    )?;
    let recorded_line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
    if recorded_destination(admin_path.as_path(), recorded_line)?.as_path()
        != pointer_path.as_path()
    {
        return Err(format!(
            "the registration {} does not name {shown}",
            admin_path.display()
        ));
    }
    let binding: Binding = Binding {
        destination: destination.to_path_buf(),
        destination_id: subject_failure(identity_of(&held), &shown, "cannot be examined")?,
        admin: admin_path,
        admin_id: subject_failure(
            identity_of(&admin),
            &"the registration",
            "cannot be examined",
        )?,
        pointer,
    };
    return Ok(Bound {
        binding,
        destination: held,
        admin,
    });
}

pub fn verify_association(bound: &Bound) -> Result<(), String> {
    let binding: &Binding = &bound.binding;
    let shown: String = binding.destination.display().to_string();
    if path_identity(binding.destination.as_path()) != Some(binding.destination_id) {
        return Err(format!(
            "{shown} is no longer the directory Git made for this add"
        ));
    }
    if path_identity(binding.admin.as_path()) != Some(binding.admin_id) {
        return Err(format!(
            "the registration {} is no longer the one Git made for this add",
            binding.admin.display()
        ));
    }
    let pointer: Vec<u8> = subject_failure(
        read_through(&bound.destination, OsStr::new(".git")),
        &format!("{shown}/.git"),
        "is unreadable",
    )?;
    if pointer != binding.pointer {
        return Err(format!("{shown}/.git is no longer the pointer Git wrote"));
    }
    // What Git resolves the unchanged pointer to now: the target is opened without a link and must be the held registration.
    let target: PathBuf = pointer_target(binding.destination.as_path(), pointer.as_slice())?;
    let resolved: File = match open_root(target.as_path()) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(not_canonical(
                shown.as_str(),
                target.as_path(),
                error.to_string().as_str(),
            ));
        }
    };
    if identity_of(&resolved).ok() != Some(binding.admin_id)
        || identity_of(&bound.admin).ok() != Some(binding.admin_id)
    {
        return Err(format!(
            "the pointer in {shown}/.git now resolves to another registration than the one Git made for this add"
        ));
    }
    let recorded: Vec<u8> = subject_failure(
        read_through(&bound.admin, OsStr::new("gitdir")),
        &format!("the registration of {shown}"),
        "is unreadable",
    )?;
    let line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
    if recorded_destination(binding.admin.as_path(), line)?.as_path()
        != binding.destination.join(".git").as_path()
    {
        return Err(format!(
            "the registration {} no longer names {shown}",
            binding.admin.display()
        ));
    }
    return Ok(());
}

/// What:
///  Check that the registration an add was bound to is still the one at its destination now,
/// by binding the destination again and comparing the two.
/// Why:
///   A copy that carries ignored state into a worktree,
/// and a recovery that resumes one,
/// hold only the record,
/// not the
///  open directories the creating command held;
/// this proves the path still leads to the same pair before the first entry is
///  written,
/// and the reason names both registrations when it does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await bindingStillHolds(commonDir, binding); // throws a reason naming both registrations
/// ```
pub fn binding_still_holds(
    common_dir: &Path,
    binding: &Binding,
) -> Result<(), String> {
    let found: Bound = bind_registration(common_dir, binding.destination.as_path())?;
    if found.binding == *binding {
        return Ok(());
    }
    return Err(format!(
        "{} is registered by {} now, and not by {} any more",
        binding.destination.display(),
        found.binding.admin.display(),
        binding.admin.display()
    ));
}

#[cfg(test)]
#[path = "worktree_add_binding_tests.rs"]
mod tests;

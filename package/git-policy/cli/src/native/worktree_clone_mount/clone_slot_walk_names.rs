//! What:
//!  The names a walk's sentences use and the refusal each step answers:
//!  the two folder names below the checkout,
//!  the slot's name as a person reads it,
//!  and the refusal of a checkout,
//!  a component,
//!  a folder or a slot name.
//! Why:
//!  Every sentence a walk prints names the component it stopped at,
//!  and the same naming is what lets a person find the folder in their own
//!  checkout,
//!  so the words live together rather than in each step
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  The steps that print them are `clone_slot_walk_to_the_parent.rs`,
//!  `clone_slot_walk_to_the_slot.rs` and
//!  `clone_slot_walk_after_the_attach.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const component = slotComponent('one');            // '.claude/worktrees/one'
//! const refusal = folderRefusal(component, error);    // names the folder a person must look at
//! ```

use super::clone_slot_walk_refusals::{
    CloneSlotRefusal,
    WalkRefusal,
    WalkRefusalReason,
    one_component_refusal,
};
use super::slot_name::slot_name_refusal;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// The folder below the checkout that holds the wrapper's own records,
/// and the folder below it whose identity the setup run records.
pub const WORKTREES_PARENT_NAMES: &[&str] = &[".claude", "worktrees"];

/// The names `WORKTREES_PARENT_NAMES` as the walk takes them.
pub(super) fn worktrees_parent_names() -> Vec<OsString> {
    let mut names: Vec<OsString> = Vec::with_capacity(WORKTREES_PARENT_NAMES.len());
    for name in WORKTREES_PARENT_NAMES {
        names.push(OsString::from(name));
    }
    return names;
}

/// The slot's name below the worktrees folder,
/// as a person reads it in a refusal.
pub(super) fn slot_component(slot: &str) -> String {
    return format!("{}/{slot}", WORKTREES_PARENT_NAMES.join("/"));
}

/// The checkout's canonical path,
/// or the refusal of the path given.
/// A path that is not already its own canonical form is refused rather than resolved,
/// because the kernel's resolution of it may name another folder the account chose.
pub(super) fn canonical_checkout(checkout: &Path) -> Result<PathBuf, CloneSlotRefusal> {
    let canonical: PathBuf = match std::fs::canonicalize(checkout) {
        Ok(canonical) => canonical,
        Err(error) => {
            return Err(CloneSlotRefusal::Walk(WalkRefusal {
                component: checkout.display().to_string(),
                reason: WalkRefusalReason::Checkout,
                detail: error.to_string(),
            }));
        }
    };
    if canonical != checkout {
        return Err(CloneSlotRefusal::Walk(WalkRefusal {
            component: checkout.display().to_string(),
            reason: WalkRefusalReason::NotTheCanonicalPath,
            detail: format!("its canonical path is {}", canonical.display()),
        }));
    }
    return Ok(canonical);
}

/// The refusal of the checkout's own root handle.
pub(super) fn checkout_root_refusal(
    canonical: &Path,
    error: std::io::Error,
) -> CloneSlotRefusal {
    if error.kind() == std::io::ErrorKind::InvalidInput {
        return CloneSlotRefusal::Walk(WalkRefusal {
            component: canonical.display().to_string(),
            reason: WalkRefusalReason::NotTheCanonicalPath,
            detail: error.to_string(),
        });
    }
    return CloneSlotRefusal::Walk(WalkRefusal {
        component: canonical.display().to_string(),
        reason: WalkRefusalReason::Checkout,
        detail: error.to_string(),
    });
}

/// The refusal of one folder the walk opens by name.
pub(super) fn folder_refusal(
    component: &str,
    error: std::io::Error,
) -> CloneSlotRefusal {
    return CloneSlotRefusal::Walk(one_component_refusal(component, &error));
}

/// What:
///  The refusal of a slot name the store could never record,
///  or nothing when it is one plain folder name.
/// Why:
///  The walk judges the same name the record is keyed by,
///  so a name the store refuses (`slot_name_refusal`) is refused here before any folder is opened,
///  and the sentence names what about the name is wrong.
pub(super) fn slot_name_walk_refusal(
    component: &str,
    slot: &str,
) -> Option<CloneSlotRefusal> {
    return slot_name_refusal(slot).map(|reason| {
        return CloneSlotRefusal::Walk(WalkRefusal {
            component: component.to_string(),
            reason: WalkRefusalReason::NotAPlainName,
            detail: reason,
        });
    });
}

/// The path a refusal names when the walk refused before it produced a canonical one.
pub(super) fn canonical_or_given(checkout: &Path) -> String {
    return checkout.display().to_string();
}

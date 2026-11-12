//! What:
//!  What a removal deletes beside the files Git keeps,
//!  and the sweep's last look that asks it again inside the removal.
//! Why:
//!   A removal deletes the worktree's own Git directory,
//!  which holds the push failures nobody has seen and the pushes the service has not run,
//!  and it deletes every worktree a registration names below it,
//!  with that worktree's files and its own record.
//!  Both can arrive while the sweep works:
//!  the push service records a refused push at any moment,
//!  and a command that adds a worktree inside this one is in the registrations only once it has finished.
//!  The sweep asks this before it examines a candidate,
//!  which answers early and keeps the expensive proof from starting,
//!  and the removal asks it again in the window it destroys in,
//!  which is what actually keeps them
//!  (a cross-vendor review of the landed sweep,
//!  round 208).
//!  It is its own file because `worktree_landed_removal.rs` is at the crate's line ceiling,
//!  and because the question and the type that asks it are read together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! nothingNewWouldGoWithIt(commonDir, registration); // throws the sentence a person reads
//! new NothingNewWouldGoWithIt({ commonDir, registration }).ask();
//! ```

use crate::worktree_landed_registrations::{Registration, registrations_of};
use crate::worktree_removal_rescue::snapshot_one;
use crate::worktree_remove::LastLook;
use std::path::Path;

/// What:
///  The other worktrees this repository registers whose checkout lies inside `one`,
///  which a removal of `one` would take with it.
/// Why:
///  A worktree can be added from inside another one,
/// and this repository's own agents do it:
///  the nested checkout is a directory of the outer worktree,
///  usually under an ignored name.
///  Removing the outer one took the nested worktree's files,
///  a modified tracked file and an untracked file among them,
///  and left its registration naming a path that is gone,
///  so `git worktree prune` could then delete the nested worktree's own record,
///  its reflog and the commits held in it
///  (a cross-vendor audit of `574063773d`,
///  finding F2).
///  The paths compared are the canonical ones Git's own registration names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreesInside(one, registered) // the registrations whose worktree starts with one.worktree
/// ```
pub(super) fn worktrees_inside<'a>(one: &Registration, registered: &'a [Registration]) -> Vec<&'a Registration> {
    let mut inside: Vec<&'a Registration> = Vec::new();
    for other in registered {
        if other.worktree != one.worktree && other.worktree.starts_with(one.worktree.as_path()) {
            inside.push(other);
        }
    }
    return inside;
}

/// What:
///  The paths of the worktrees found inside,
/// as the person reads them,
///  a few named and the rest counted.
/// Why:
///  A person decides about their own worktrees,
///  so the ones that keep this one are named,
///  and a list longer than a few would bury the sentence it sits in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesOf(inside) // '`a` and 2 more'
/// ```
pub(super) fn names_of(inside: &[&Registration]) -> String {
    const SHOWN: usize = 2;
    let mut parts: Vec<String> = Vec::new();
    for other in inside.iter().take(SHOWN) {
        parts.push(format!("`{}`", other.worktree.display()));
    }
    let rest: usize = inside.len().saturating_sub(parts.len());
    if rest > 0 {
        parts.push(format!("and {rest} more"));
    }
    return parts.join(", ");
}

/// What:
///  Why a worktree that holds another registered worktree stays.
/// Why:
///  The two checks that keep it,
///  the one over the registrations the sweep read at its start and the one over the registrations read as the removal
///  runs,
///  answer a person in the same words,
///  so the sentence is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nestedWorktreeReason(inside) // the sentence a kept parent prints
/// ```
pub(super) fn nested_worktree_reason(inside: &[&Registration]) -> String {
    return format!(
        "another worktree this repository registers is inside it ({}), and a removal of this one would take that worktree's files and its own record with it",
        names_of(inside)
    );
}

/// What:
///  Why a worktree whose own records could not all be copied stays,
///  naming the Git directory that still holds them.
/// Why:
///  The copy is the only thing left of those records once a removal deletes that directory,
///  so a copy that could not be made keeps the worktree,
///  and the person reads where the records still are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unsavedRecordsReason(admin, unsaved) // the sentence a kept worktree prints
/// ```
pub(super) fn unsaved_records_reason(admin: &Path, unsaved: &[String]) -> String {
    return format!(
        "the unshown push failures and the unfinished pushes its own Git directory holds could not all be copied into a removal snapshot, so nothing is removed and they stay in {}: {}",
        admin.display(),
        unsaved.join(" ")
    );
}

/// What:
///  The sweep's own last look for one candidate:
///  the question what a removal deletes beside the files Git keeps.
/// Why:
///  A named type stands where a closure would capture the directory and the registration,
///  because this codebase keeps closures out of calls like this one
///  (`LastLook` in `worktree_remove.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NothingNewWouldGoWithIt implements LastLook { ask() { return nothingNewWouldGoWithIt(this.commonDir, this.one) } }
/// ```
pub(super) struct NothingNewWouldGoWithIt<'a> {
    /// The common Git directory the removal works in.
    pub(super) common_dir: &'a Path,
    /// The registration the removal takes.
    pub(super) one: &'a Registration,
}

impl LastLook for NothingNewWouldGoWithIt<'_> {
    fn ask(&self) -> Result<(), String> {
        return nothing_new_would_go_with_it(self.common_dir, self.one);
    }
}

/// What:
///  Why nothing may be removed from the worktree `one` now,
///  or `Ok(())` when what a removal deletes beside the files Git keeps is still what the sweep read.
/// Why:
///  A removal deletes the worktree's own Git directory,
///  which holds the push failures nobody has seen and the pushes the service has not run,
///  and it deletes every worktree a registration names below it,
///  with that worktree's files and its own record.
///  Both can arrive while the sweep works:
///  the push service records a refused push at any moment,
///  and a command that adds a worktree inside this one is in the registrations only once it has finished.
///  The sweep asks this before it examines a candidate,
///  which answers early and keeps the expensive proof from starting,
///  and the removal asks it again in the window it destroys in,
///  which is what actually keeps them
///  (a cross-vendor review of the landed sweep,
///  round 208).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nothingNewWouldGoWithIt(place, registration) // throws the sentence a person reads
/// ```
pub(super) fn nothing_new_would_go_with_it(common_dir: &Path, one: &Registration) -> Result<(), String> {
    let unsaved: Vec<String> = snapshot_one(common_dir, one.admin.as_path());
    if !unsaved.is_empty() {
        return Err(unsaved_records_reason(one.admin.as_path(), unsaved.as_slice()));
    }
    let registered_now: Vec<Registration> = match registrations_of(common_dir) {
        Ok(found) => found,
        Err(reason) => return Err(format!("nothing proves no other worktree lies inside this one, so nothing is removed: {reason}")),
    };
    let inside_now: Vec<&Registration> = worktrees_inside(one, registered_now.as_slice());
    if !inside_now.is_empty() {
        return Err(nested_worktree_reason(inside_now.as_slice()));
    }
    return Ok(());
}

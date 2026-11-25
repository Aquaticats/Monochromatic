//! What:
//!  The recorded parent and slot identities against the live folders,
//!  naming the failing field or path component.
//! Why:
//!  The route attaches a clone to the folder whose identity the store
//!  recorded,
//!  compares it field by field and refuses on any difference,
//!  and `doctor` reports the same comparison without attaching anything
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  plan step 18,
//!  "the recorded parent and slot identities with the failing field or
//!  component").
//!  A changed identity is never permission to refresh the record,
//!  so a finding names the field and the administrator's adoption command,
//!  and a walk that could not read a folder is unknown rather than a
//!  difference.
//!  The refusals are the route's own,
//!  so a control hands in the one it built and no folder is walked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkSlotIdentity({ slot: 'one', refusal }); // pass | finding('mode') | unknown
//! ```

use super::check_outcome::{CheckOutcome, finding, pass, unknown};
use crate::worktree_clone_mount::clone_folder_identity::field_names;
use crate::worktree_clone_mount::{CloneFolderField, CloneFolderIdentity, CloneSlotRefusal, WalkRefusal, WalkRefusalReason};

/// What:
///  What the store's `.parent` record answered.
/// Why:
///  A store with no parent record,
///  a record this account could not read and a record that parsed are three
///  states,
///  and only the last can be compared with a live folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedIdentity = { kind: 'read'; identity } | { kind: 'absent'; why } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordedIdentity {
    /// The record parsed and carries the parent's identity.
    Read(Box<CloneFolderIdentity>),
    /// No parent record is filed.
    Absent {
        /// What the read answered.
        why: String,
    },
    /// The record could not be read.
    Unreadable {
        /// What the read answered.
        why: String,
    },
}

/// What:
///  What the walk to the live `.claude/worktrees` folder answered.
/// Why:
///  A folder that is not there,
///  one a link stands in the way of and one this account could not open are
///  different answers,
///  and the walk's own refusal already names the component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LiveIdentity = { kind: 'held'; identity } | { kind: 'refused'; refusal: WalkRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveIdentity {
    /// The walk held the folder and read its identity.
    Held(Box<CloneFolderIdentity>),
    /// The walk refused,
    /// naming the component and why.
    Refused(WalkRefusal),
}

/// What:
///  The parent folder's recorded identity and the live folder's.
/// Why:
///  Both sides travel together,
///  so no check compares one store's record with another checkout's folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParentIdentityFacts = { recorded: RecordedIdentity; live: LiveIdentity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParentIdentityFacts {
    /// What the store's `.parent` record answered.
    pub recorded: RecordedIdentity,
    /// What the walk answered.
    pub live: LiveIdentity,
}

/// One slot's recorded identity against its live folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotIdentityFacts = { slot: string; refusal?: CloneSlotRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotIdentityFacts {
    /// The slot's name.
    pub slot: String,
    /// What the walk answered,
    /// or nothing when it held the slot and every field agreed.
    pub refusal: Option<CloneSlotRefusal>,
}

/// What:
///  Whether a walk's refusal proves a missing or wrong component,
///  or only that this caller could not read it.
/// Why:
///  A missing folder,
///  a planted link and a name no slot may hold are findings a person fixes,
///  while a permission refusal or an unread `/proc` entry proves nothing
///  about the folder and must stay unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unproved = (reason) => ['unreadable', 'unreadable-fields', 'checkout'].includes(reason);
/// ```
pub fn walk_refusal_is_unproved(reason: WalkRefusalReason) -> bool {
    return matches!(reason, WalkRefusalReason::Unreadable | WalkRefusalReason::UnreadableFields | WalkRefusalReason::Checkout);
}

/// What:
///  Whether the recorded `.claude/worktrees` identity is the live folder's.
/// Why:
///  A difference names the fields and the administrator's adoption command,
///  because the record is never refreshed by this route,
///  and a store with no parent record sends a person to the same
///  administrator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkParentIdentity(facts); // pass | finding(field) | unknown
/// ```
pub fn check_parent_identity(facts: &ParentIdentityFacts) -> CheckOutcome {
    let recorded: &CloneFolderIdentity = match &facts.recorded {
        RecordedIdentity::Read(identity) => identity,
        RecordedIdentity::Absent { why } => {
            return finding("parent-record", format!("the store files no `{PARENT_RECORD}`, so no live folder can be compared ({why}); an administrator records the intended replacement by running the words `monochromatic-git-policy-cli zfs-root adopt-worktree-parent`, after the root-owned file's path, as root with the replacement's device and inode; plan step 19's setup installs that root-owned file and links it for ZED, and no release installs it yet, so nothing on this machine starts those words today, and nothing is attached meanwhile", PARENT_RECORD = crate::worktree_clone_mount::PARENT_RECORD_NAME));
        }
        RecordedIdentity::Unreadable { why } => return unknown(format!("`{PARENT_RECORD}` could not be read ({why}), so the parent's identity is unproved", PARENT_RECORD = crate::worktree_clone_mount::PARENT_RECORD_NAME)),
    };
    let found: &CloneFolderIdentity = match &facts.live {
        LiveIdentity::Held(identity) => identity,
        LiveIdentity::Refused(refusal) => {
            if walk_refusal_is_unproved(refusal.reason) {
                return unknown(format!("{} could not be read ({}), so the parent's live identity is unproved", refusal.component, refusal.message()));
            }
            return finding(refusal.component.clone(), refusal.message());
        }
    };
    let fields: Vec<CloneFolderField> = recorded.differences(found);
    if fields.is_empty() {
        return pass(String::from("the live `.claude/worktrees` folder is the one the store recorded"));
    }
    return finding(
        field_names(&fields),
        format!("the live `.claude/worktrees` folder is not the recorded one: the {} differs. Recorded: {}. Found: {}. Ways forward: restore the recorded folder, or let an administrator run `monochromatic-git-policy-cli zfs-root adopt-worktree-parent` as root with the replacement's device and inode", field_names(&fields), recorded.describe(), found.describe()),
    );
}

/// What:
///  Whether one slot's live folder is the folder its record holds.
/// Why:
///  The walk's refusal already names the field or the component,
///  so a report line repeats it rather than inventing a second vocabulary,
///  and a refusal that proves nothing about the folder stays unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkSlotIdentity({ slot, refusal }); // pass | finding(field) | unknown
/// ```
pub fn check_slot_identity(facts: &SlotIdentityFacts) -> CheckOutcome {
    let slot: &str = facts.slot.as_str();
    let Some(refusal) = &facts.refusal else {
        return pass(format!("slot `{slot}` is the folder its record holds"));
    };
    return match refusal {
        CloneSlotRefusal::SlotIdentity { fields, .. } => finding(field_names(fields), refusal.message()),
        CloneSlotRefusal::ParentIdentity { fields, .. } => finding(field_names(fields), refusal.message()),
        CloneSlotRefusal::TargetAnotherObject { .. } => finding("slot", refusal.message()),
        CloneSlotRefusal::Walk(walk) => {
            if walk_refusal_is_unproved(walk.reason) {
                unknown(format!("slot `{slot}` could not be read: {}", walk.message()))
            } else {
                finding(walk.component.clone(), walk.message())
            }
        }
        CloneSlotRefusal::Store(_) => unknown(format!("slot `{slot}`'s record could not be read: {}", refusal.message())),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "identity_checks_tests.rs"]
mod tests;

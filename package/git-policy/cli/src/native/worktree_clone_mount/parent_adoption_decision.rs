//! What:
//!  The decision an administrator's `adopt-worktree-parent` mode makes:
//!  whether a `.claude/worktrees` folder that was recreated or restored may replace the recorded one,
//!  and why not when it may not.
//! Why:
//!  Only an administrator may adopt a legitimate replacement, with managed operations quiesced,
//!  starting from the pinned checkout's no-crossing walk;
//!  the replacement must be identified by device and inode independently,
//!  its owner,
//!  mode and ACLs must match policy,
//!  every affected binding must be reconciled against live and staged mounts,
//!  and a fresh canonical lookup must match the held folder
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  Until then managed mutations on the mismatched parent are refused,
//!  and never refreshed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decision = decideAdoption({ walk, identified, policy, bindings, slots, fresh });
//! if (decision.ok) recordParentIdentity(store, decision.identity);
//! ```

use super::clone_folder_identity::{CloneFolderField, CloneFolderIdentity};
use super::clone_slot_walk_refusals::WalkRefusal;
use std::path::PathBuf;

/// What:
///  The owner,
///  mode and ACLs an adopted parent folder must hold,
///  taken from the identity the setup run recorded.
/// Why:
///   A recreated folder must not come back wider than the one it replaces,
///  so the recorded folder's own shape is the policy the replacement is judged against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const policy = { owner: recorded.owner, mode: recorded.mode, accessAcl: recorded.accessAcl, defaultAcl: recorded.defaultAcl };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParentPolicy {
    /// The user who must own the folder.
    pub owner: u32,
    /// The permission bits the folder must hold.
    pub mode: u32,
    /// The access ACL the folder must hold,
    /// or none.
    pub access_acl: Option<Vec<u8>>,
    /// The default ACL the folder must hold,
    /// or none.
    pub default_acl: Option<Vec<u8>>,
}

impl ParentPolicy {
    /// The policy the recorded identity sets,
    /// which is the folder's own owner,
    /// mode and ACLs.
    pub fn from_recorded(identity: &CloneFolderIdentity) -> ParentPolicy {
        return ParentPolicy { owner: identity.owner, mode: identity.mode, access_acl: identity.access_acl.clone(), default_acl: identity.default_acl.clone() };
    }

    /// The policy fields in which `held` differs from this policy.
    fn differences(&self, held: &CloneFolderIdentity) -> Vec<CloneFolderField> {
        let mut fields: Vec<CloneFolderField> = Vec::new();
        if held.owner != self.owner {
            fields.push(CloneFolderField::Owner);
        }
        if held.mode != self.mode {
            fields.push(CloneFolderField::Mode);
        }
        if held.access_acl != self.access_acl {
            fields.push(CloneFolderField::AccessAcl);
        }
        if held.default_acl != self.default_acl {
            fields.push(CloneFolderField::DefaultAcl);
        }
        return fields;
    }
}

/// What:
///  One binding the adoption must reconcile:
///  a live mount,
///  a mount held in staging,
///  or one it could not resolve.
/// Why:
///   A replacement may only be recorded once every affected binding is known to be live or staged,
///  and an unresolved one must be named with the recovery it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Binding = { state: 'live' | 'staged' | 'unresolved'; what: string; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingState {
    /// A mount the handler sees at its slot.
    Live {
        /// What the binding is,
        /// such as `slot one`.
        what: String,
        /// Where it is mounted.
        place: PathBuf,
    },
    /// A mount held in the staging folder.
    Staged {
        /// What the binding is.
        what: String,
        /// Where it is staged.
        place: PathBuf,
    },
    /// A binding whose state could not be resolved.
    Unresolved {
        /// What the binding is.
        what: String,
        /// What could not be read,
        /// and what recovery it needs.
        reason: String,
    },
}

/// What:
///  One surviving slot's reconciliation:
///  its live identity against its record.
/// Why:
///   A surviving slot identity is never overwritten,
///  so a slot whose live folder differs stops the adoption.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Slot = { state: 'agrees' | 'differs'; slot: string; field?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotState {
    /// The live folder is the recorded one.
    Agrees {
        /// The slot's name.
        slot: String,
    },
    /// The live folder differs from the recorded one in this field.
    Differs {
        /// The slot's name.
        slot: String,
        /// The field that differs.
        field: CloneFolderField,
    },
}

/// What:
///  Every fact the adoption decision reads.
/// Why:
///   The decision holds no device and starts no mount:
///  the administrator's walk,
///  the identified device and inode,
///  the policy,
///  the bindings,
///  the slots and the fresh lookup are supplied,
///  so the decision itself is a table a control can drive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Facts = { walk; identified: [device, inode]; policy; bindings; slots; fresh };
/// ```
#[derive(Clone, Debug)]
pub struct AdoptionFacts {
    /// The no-crossing walk's answer for the `.claude/worktrees` folder,
    /// or its refusal.
    pub walk: Result<CloneFolderIdentity, WalkRefusal>,
    /// The device the administrator identified independently.
    pub identified_device: u64,
    /// The inode the administrator identified independently.
    pub identified_inode: u64,
    /// The owner,
    /// mode and ACLs the replacement must hold.
    pub policy: ParentPolicy,
    /// Every binding the adoption reconciles.
    pub bindings: Vec<BindingState>,
    /// Every surviving slot's reconciliation.
    pub slots: Vec<SlotState>,
    /// A fresh canonical lookup of the parent folder,
    /// or its refusal.
    pub fresh: Result<CloneFolderIdentity, WalkRefusal>,
}

/// What:
///  Why the adoption was refused.
/// Why:
///   Each cause is one of the refusals the acceptance names:
///  a link,
///  a boundary,
///  an ACL mismatch,
///  an unresolved live or staged binding,
///  and the identity or policy the administrator did not identify.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'walk' | 'identity' | 'policy' | 'binding' | 'slot' | 'fresh' | 'fresh-differs'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdoptionRefusal {
    /// The walk refused the folder.
    Walk {
        /// What it refused.
        refusal: WalkRefusal,
    },
    /// The held folder is not the one the administrator identified.
    Identity {
        /// The field that differs.
        field: CloneFolderField,
        /// The value the administrator identified.
        identified: String,
        /// The value the held folder holds.
        held: String,
    },
    /// The held folder does not hold the policy's shape.
    Policy {
        /// The fields in which it differs.
        fields: Vec<CloneFolderField>,
    },
    /// One binding could not be resolved.
    BindingUnresolved {
        /// What the binding is.
        what: String,
        /// What could not be read,
        /// and what recovery it needs.
        reason: String,
    },
    /// One surviving slot differs from its record.
    SlotDiffers {
        /// The slot's name.
        slot: String,
        /// The field that differs.
        field: CloneFolderField,
    },
    /// The fresh canonical lookup refused.
    FreshLookup {
        /// What it refused.
        refusal: WalkRefusal,
    },
    /// The fresh canonical lookup reached another folder.
    FreshLookupDiffers {
        /// The fields in which the folders differ.
        fields: Vec<CloneFolderField>,
    },
}

impl AdoptionRefusal {
    /// The refusal in words,
    /// naming the failing component or field and every way forward.
    pub fn message(&self) -> String {
        return match self {
            AdoptionRefusal::Walk { refusal } => format!("the adoption's walk refused the parent folder: {}. Ways forward: fix the component it names and run the adoption again; nothing was recorded.", refusal.message()),
            AdoptionRefusal::Identity { field, identified, held } => format!(
                "the held `.claude/worktrees` folder is not the one the administrator identified: the {field} is {identified}, and the held folder holds {held}. Nothing was recorded. Ways forward: identify the intended replacement again from the restored checkout or the controlled recreation, and run the adoption with its own device and inode."
            ),
            AdoptionRefusal::Policy { fields } => format!(
                "the held `.claude/worktrees` folder does not hold the shape the recorded folder set: the {} differs from the policy. Nothing was recorded, because a replacement must not come back wider than the folder it replaces. Ways forward: give the folder the recorded owner, mode and ACLs and run the adoption again.",
                super::attach_post_check::names(fields)
            ),
            AdoptionRefusal::BindingUnresolved { what, reason } => format!(
                "the adoption cannot reconcile {what}: {reason}. Nothing was recorded, because a replacement is recorded only once every affected binding is known to be live or staged. Ways forward: settle or remove that binding as recovery requires, then run the adoption again."
            ),
            AdoptionRefusal::SlotDiffers { slot, field } => format!(
                "the surviving clone slot `{slot}` differs from its record in the {field}, and an adoption never overwrites a surviving slot identity. Nothing was recorded. Ways forward: restore the folder whose identity is recorded, or settle that binding with the removal workflow, then run the adoption again.",
                field = field.name()
            ),
            AdoptionRefusal::FreshLookup { refusal } => format!("the fresh canonical lookup after the walk refused: {}. Nothing was recorded. Ways forward: fix what it names and run the adoption again.", refusal.message()),
            AdoptionRefusal::FreshLookupDiffers { fields } => format!(
                "the fresh canonical lookup reached another folder than the held one: the {} differs. Nothing was recorded. Ways forward: stop the account from changing the folder, run the adoption again, and let it record only what both looks agree on.",
                super::attach_post_check::names(fields)
            ),
        };
    }
}

/// What:
///  What the adoption decided.
/// Why:
///   Only a recorded decision lets the caller write the new parent identity,
///  and the identity carried is the held folder's own,
///  not a value from a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Adoption = { identity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Adoption {
    /// The identity to record for `.claude/worktrees`.
    pub identity: CloneFolderIdentity,
}

/// What:
///  Decide whether the held parent folder may replace the recorded one.
/// Why:
///   The design fixes the order:
///  the walk,
///  the administrator's device and inode,
///  the owner,
///  mode and ACL policy,
///  the bindings,
///  the surviving slots,
///  and the fresh canonical lookup.
///  Every step refuses rather than adopting an unresolved replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = decideAdoption(facts); // throws the refusal
/// ```
///
/// # Errors
/// The walk's refusal,
/// an identity the administrator did not identify,
/// a policy field that differs,
/// an unresolved binding,
/// a slot that differs,
/// the fresh lookup's refusal,
/// or a fresh lookup that reached another folder.
pub fn decide_adoption(facts: &AdoptionFacts) -> Result<Adoption, AdoptionRefusal> {
    let held: &CloneFolderIdentity = match &facts.walk {
        Ok(identity) => identity,
        Err(refusal) => return Err(AdoptionRefusal::Walk { refusal: refusal.clone() }),
    };
    if held.device != facts.identified_device {
        return Err(AdoptionRefusal::Identity { field: CloneFolderField::Device, identified: facts.identified_device.to_string(), held: held.device.to_string() });
    }
    if held.inode != facts.identified_inode {
        return Err(AdoptionRefusal::Identity { field: CloneFolderField::Inode, identified: facts.identified_inode.to_string(), held: held.inode.to_string() });
    }
    let policy_fields: Vec<CloneFolderField> = facts.policy.differences(held);
    if !policy_fields.is_empty() {
        return Err(AdoptionRefusal::Policy { fields: policy_fields });
    }
    for binding in &facts.bindings {
        if let BindingState::Unresolved { what, reason } = binding {
            return Err(AdoptionRefusal::BindingUnresolved { what: what.clone(), reason: reason.clone() });
        }
    }
    for slot in &facts.slots {
        if let SlotState::Differs { slot, field } = slot {
            return Err(AdoptionRefusal::SlotDiffers { slot: slot.clone(), field: *field });
        }
    }
    let fresh: &CloneFolderIdentity = match &facts.fresh {
        Ok(identity) => identity,
        Err(refusal) => return Err(AdoptionRefusal::FreshLookup { refusal: refusal.clone() }),
    };
    let fresh_fields: Vec<CloneFolderField> = held.differences(fresh);
    if !fresh_fields.is_empty() {
        return Err(AdoptionRefusal::FreshLookupDiffers { fields: fresh_fields });
    }
    return Ok(Adoption { identity: held.clone() });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "parent_adoption_decision_tests.rs"]
mod tests;

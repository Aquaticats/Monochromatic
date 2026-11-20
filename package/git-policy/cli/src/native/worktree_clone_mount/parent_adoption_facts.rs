//! What:
//!  The shapes an adoption compares against:
//!  the owner,
//!  mode and ACLs a replacement must hold,
//!  and the state of every binding and every surviving slot.
//! Why:
//!   The adoption's decision reads these values and holds no device,
//!  so the shapes live in one file and the table that judges them in another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const policy = { owner, mode, accessAcl, defaultAcl };
//! const binding = { kind: 'live' | 'staged' | 'unresolved', what, place };
//! ```

use super::clone_folder_identity::{CloneFolderField, CloneFolderIdentity};
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
    pub(super) fn differences(&self, held: &CloneFolderIdentity) -> Vec<CloneFolderField> {
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

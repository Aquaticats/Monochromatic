//! What:
//!  The administrator's `adopt-worktree-parent` mode:
//!  the walk from the pinned checkout to the recreated `.claude/worktrees` folder,
//!  the identification the administrator read for it,
//!  the policy the recorded folder sets,
//!  every surviving slot the store records,
//!  the fresh canonical lookup,
//!  and the one store write the decision leads to.
//! Why:
//!  Only an administrator may record a replacement parent folder after a checkout restore or a
//!  controlled recreation,
//!  and only once every binding is reconciled and a fresh lookup agrees
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount",
//!  "The administrator's adoption").
//!  The decision table itself,
//! its order and its sentences are step 4's
//!  (`worktree_clone_mount::parent_adoption_decision`);
//!  this file supplies its facts from real descriptors and the store,
//!  and records nothing until the table agrees.
//!  The mode runs with managed operations quiesced,
//!  which is the administrator's own precondition and not something a program can prove.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const adopted = adopt({ checkout, device, inode }); // the recorded identity, or the refusal
//! ```

use crate::worktree_clone_mount::clone_folder_identity::{CloneFolderField, CloneFolderIdentity};
use crate::worktree_clone_mount::clone_slot_walk_refusals::WalkRefusal;
use crate::worktree_clone_mount::parent_adoption_decision::{Adoption, AdoptionFacts, AdoptionRefusal, decide_adoption};
use crate::worktree_clone_mount::parent_adoption_facts::{BindingState, ParentPolicy, SlotState};
use crate::worktree_clone_mount::slot_identity_store::SlotIdentityStore;
use crate::worktree_clone_mount::slot_identity_store_refusals::StoreRefusal;
use crate::worktree_clone_mount::{
    HeldWorktreesParent, SLOT_IDENTITY_STORE_PATH, hold_the_worktrees_parent, record_parent_identity_under_root_control,
};
use crate::worktree_clone_mount_handler::handler_mode_arguments::AdoptRequest;
use rustix::process::geteuid;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  Why this mode recorded nothing.
/// Why:
///   Each cause names the file or field that refused,
///  and the administrator reads which recovery the decision asks for before running the mode again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'not-root' } | { kind: 'store' } | { kind: 'no-parent-record' } | { kind: 'decision' } | { kind: 'write' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdoptFailure {
    /// The mode is not running as root,
    /// and only root writes the store.
    NotRoot {
        /// The effective user the mode found.
        effective_user: u32,
    },
    /// The store could not be read under root control,
    /// so no record decides anything.
    Store(StoreRefusal),
    /// The store holds no parent record,
    /// so there is no recorded folder a replacement could succeed.
    NoParentRecord {
        /// The store the mode read.
        path: PathBuf,
    },
    /// The adoption's own table refused the replacement.
    Decision(AdoptionRefusal),
    /// The store could not be written,
    /// so the decision stands and records nothing.
    Write(StoreRefusal),
}

impl AdoptFailure {
    /// The refusal in words,
    /// naming the file or field and what an administrator does next.
    pub fn message(&self) -> String {
        return match self {
            AdoptFailure::NotRoot { effective_user } => format!(
                "the adopt-worktree-parent mode runs as root, and this process's effective user is {effective_user}. Nothing was read or recorded, because the store under root control is the only record an adoption replaces and an account that could write it could adopt any replacement. Ways forward: run the mode as root, with managed operations quiesced."
            ),
            AdoptFailure::Store(refusal) => format!(
                "the recorded clone slot identities could not be read, so the adoption judged nothing: {}. Nothing was recorded. Ways forward: fix what the store refusal names and run the mode again; the parent record is replaced only after the store reads.",
                refusal.message()
            ),
            AdoptFailure::NoParentRecord { path } => format!(
                "{} holds no record for the `.claude/worktrees` parent folder, so no folder is recorded that a replacement could succeed. Nothing was recorded. Ways forward: let the setup run record the parent identity first (plan step 19), then run the adoption again.",
                path.display()
            ),
            AdoptFailure::Decision(refusal) => refusal.message(),
            AdoptFailure::Write(refusal) => format!(
                "the adoption was decided and the parent record could not be written: {}. The recorded parent folder is unchanged. Ways forward: fix what the store refusal names and run the mode again; the decision is made again from the folder as it is then.",
                refusal.message()
            ),
        };
    }
}

/// What:
///  Which of a recorded slot's fields differ,
///  and whether its folder could be read at all.
/// Why:
///   The decision wants one state per recorded slot:
///  agrees,
///  the one field that differs,
///  or an unresolved binding with the recovery it needs,
///  and this is the read that tells them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading = { kind: 'agrees' } | { kind: 'differs'; field } | { kind: 'unresolved'; reason };
/// ```
enum SlotReading {
    /// The folder is the recorded one.
    Agrees,
    /// The folder differs from its record in this field.
    Differs {
        /// The field.
        field: CloneFolderField,
    },
    /// The folder could not be read as a surviving slot.
    Unresolved {
        /// What could not be read,
        /// and what recovery it needs.
        reason: String,
    },
}

/// What:
///  Read one recorded slot's folder below the held parent.
/// Why:
///   The folder is opened from the held descriptor,
///  never from a path,
///  so the folder judged is the folder the walk reached;
///  a mount standing at the slot is refused by that open
///  (`fs_beneath::open_directory_at` never crosses one),
///  which is how this step refuses to adopt over a live binding it cannot yet reconcile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readSlot(parent, 'my-worktree', recorded);
/// ```
fn read_slot(parent: &HeldWorktreesParent, slot: &str, recorded: &CloneFolderIdentity) -> SlotReading {
    let folder: std::fs::File = match crate::fs_beneath::open_directory_at(&parent.folder, OsStr::new(slot)) {
        Ok(folder) => folder,
        Err(error) => return SlotReading::Unresolved { reason: slot_folder_reason(&error) },
    };
    let identity: CloneFolderIdentity = match CloneFolderIdentity::capture(&folder) {
        Ok(identity) => identity,
        Err(error) => {
            return SlotReading::Unresolved {
                reason: format!(
                    "its folder was opened and its identity could not be read ({error}); recovery: make that slot a readable folder the store's record describes, or settle the binding with the removal workflow, then run the adoption again"
                ),
            };
        }
    };
    return match recorded.differences(&identity).first() {
        Some(field) => SlotReading::Differs { field: *field },
        None => SlotReading::Agrees,
    };
}

/// What:
///  Why a recorded slot's folder could not be read as a surviving slot.
/// Why:
///   The three causes need three different recoveries:
///  a mount standing at the slot is the live binding the sweep and unmount steps own,
///  a missing folder is a registration the removal workflow settles,
///  and anything else is the folder itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = error.code === 'EXDEV' ? 'a mount stands there' : error.code === 'ENOENT' ? 'no folder is there' : error.message;
/// ```
fn slot_folder_reason(error: &std::io::Error) -> String {
    return match error.raw_os_error() {
        Some(libc::EXDEV) => String::from(
            "a mount stands at that slot, and this build reconciles no live or staged binding yet, so nothing proves what that mount survives; recovery: unmount or settle that binding with the unmount and removal steps, then run the adoption again",
        ),
        Some(libc::ENOENT) => String::from(
            "the recreated parent folder holds nothing at that slot, so nothing proves that the slot survives the replacement; recovery: settle that binding with the removal workflow, then run the adoption again",
        ),
        _ => format!(
            "the folder could not be read ({error}); recovery: make that slot a readable folder the store's record describes, or settle the binding with the removal workflow, then run the adoption again"
        ),
    };
}

/// What:
///  Every slot the store records,
///  as the decision's two lists:
///  a state for each surviving slot,
///  and a binding for each one whose folder cannot stand.
/// Why:
///   A slot the caller lists no state for is a refusal in its own right,
///  so every recorded name reaches one list or the other,
///  and an empty list never passes for a checkout with slots.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { slots, bindings } = recordedSlots(parent, store);
/// ```
fn recorded_slots(parent: Option<&HeldWorktreesParent>, store: &SlotIdentityStore) -> (Vec<SlotState>, Vec<BindingState>) {
    let mut slots: Vec<SlotState> = Vec::new();
    let mut bindings: Vec<BindingState> = Vec::new();
    let Some(worktrees) = parent else {
        return (slots, bindings);
    };
    for name in store.slot_names() {
        let Some(recorded) = store.slot(name.as_str()) else {
            continue;
        };
        match read_slot(worktrees, name.as_str(), recorded) {
            SlotReading::Agrees => slots.push(SlotState::Agrees { slot: name }),
            SlotReading::Differs { field } => slots.push(SlotState::Differs { slot: name, field }),
            SlotReading::Unresolved { reason } => bindings.push(BindingState::Unresolved {
                what: format!("the clone slot `{name}` below {}", worktrees.canonical.display()),
                reason,
            }),
        }
    }
    return (slots, bindings);
}

/// What:
///  The one line a successful adoption prints:
///  the folder it recorded,
///  and the identity it now stands under.
/// Why:
///   An administrator who ran the mode reads what changed and where,
///  and a later sweep compares against the record this line describes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = `recorded ${canonical} as the adopted parent: device ${device}, inode ${inode}`;
/// ```
fn recorded_line(parent: &HeldWorktreesParent, adoption: &Adoption) -> String {
    return format!(
        "recorded {} as the adopted `.claude/worktrees` parent folder: device {}, inode {}, owner {}, mode {:04o}. Every later sweep, attach and unmount compares against this identity, and a surviving clone slot keeps its own record. Nothing was mounted or unmounted.",
        parent.canonical.display(),
        adoption.identity.device,
        adoption.identity.inode,
        adoption.identity.owner,
        adoption.identity.mode & 0o7777
    );
}

/// What:
///  Judge the replacement folder and,
///  when every check agrees,
///  record its identity as the parent record.
/// Why:
///  The order is the decision's own:
///  the walk,
///  the identified device and inode,
///  the recorded folder's owner,
///  mode and ACLs,
///  the bindings,
///  the surviving slots,
///  and the fresh canonical lookup.
///  Nothing is written until all of them agree,
///  so a refusal leaves the record exactly as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const adopted = adopt(request); // the recorded identity, or a refusal naming the field
/// ```
///
/// # Errors
/// A mode that is not root,
/// a store that could not be read or holds no parent record,
/// a decision the table refused,
/// or a store that could not be written.
pub fn run(request: &AdoptRequest) -> Result<String, AdoptFailure> {
    let effective_user: u32 = geteuid().as_raw();
    return adopt_at(request, Path::new(SLOT_IDENTITY_STORE_PATH), geteuid().is_root(), effective_user);
}

/// What:
///  The adoption against a named store,
///  with the caller's answer to whether this process runs as root.
/// Why:
///   The store's real path and the process's effective user are the two facts a control cannot set,
///  so they are arguments here:
///  a control drives the not-root refusal without touching `/var/lib`,
///  and the program passes the store the design fixes and the user the kernel reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const adopted = adoptAt({ request, storePath, runningAsRoot: false, effectiveUser: 1000 });
/// ```
fn adopt_at(request: &AdoptRequest, store_path: &Path, running_as_root: bool, effective_user: u32) -> Result<String, AdoptFailure> {
    if !running_as_root {
        return Err(AdoptFailure::NotRoot { effective_user });
    }
    let store: SlotIdentityStore = SlotIdentityStore::read_under_root_control(store_path).map_err(AdoptFailure::Store)?;
    let Some(recorded_parent) = store.parent() else {
        return Err(AdoptFailure::NoParentRecord { path: store_path.to_path_buf() });
    };
    let policy: ParentPolicy = ParentPolicy::from_recorded(recorded_parent);
    let walked: Result<HeldWorktreesParent, WalkRefusal> = hold_the_worktrees_parent(request.checkout.as_path());
    let walk: Result<CloneFolderIdentity, WalkRefusal> = match &walked {
        Ok(held) => Ok(held.identity.clone()),
        Err(refusal) => Err(refusal.clone()),
    };
    let (slots, bindings) = recorded_slots(walked.as_ref().ok(), &store);
    let fresh: Result<CloneFolderIdentity, WalkRefusal> = match hold_the_worktrees_parent(request.checkout.as_path()) {
        Ok(looked) => Ok(looked.identity),
        Err(refusal) => Err(refusal),
    };
    let facts: AdoptionFacts = AdoptionFacts {
        walk,
        identified_device: request.device,
        identified_inode: request.inode,
        policy,
        bindings,
        slots,
        store: store.clone(),
        fresh,
    };
    let adoption: Adoption = decide_adoption(&facts).map_err(AdoptFailure::Decision)?;
    record_parent_identity_under_root_control(store_path, &adoption.identity).map_err(AdoptFailure::Write)?;
    let Ok(held) = walked else {
        unreachable!("a decided adoption walked the parent folder");
    };
    return Ok(recorded_line(&held, &adoption));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "adopt_worktree_parent_mode_tests.rs"]
mod tests;

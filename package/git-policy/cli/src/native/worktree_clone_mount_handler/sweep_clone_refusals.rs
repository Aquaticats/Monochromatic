//! What:
//!  Why one immediate child of the pinned parent is not a clone this sweep
//!  attaches,
//!  one variant per cause,
//!  and the words a person reads for each:
//!  the step word a receipt names,
//!  the dataset the refusal is about,
//!  and the sentence naming every way forward.
//! Why:
//!  Each cause sends a person to a different property,
//!  and the receipt's failed step is one of these words,
//!  so the words are the refusal's own rather than a sentence a caller parses
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The key's refusal is told apart from every identity or permission refusal,
//!  because an unavailable key is a question about the data rather than about
//!  the dataset's shape.
//!  The words are their own file so the checks that build these causes stay
//!  short enough to read whole
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CloneRefusal = { kind: 'not-a-child' | 'slot-name' | 'not-a-filesystem' | 'snapshot' | 'guid' | 'no-origin' | 'origin' | 'mountpoint' | 'setuid' | 'devices' | 'property' | 'key-unavailable'; ... };
//! refusal.step(); // 'clone-name', 'clone-guid', 'key-unavailable', ...
//! refusal.message(); // the sentence a person reads
//! ```

use super::sweep_clone_property_values::LEGACY_MOUNTPOINT;
use super::sweep_dataset_facts::{
    DEVICES_PROPERTY,
    DMU_OBJECT_SET_FILESYSTEM,
    INDEX_OFF,
    KEY_STATUS_PROPERTY,
    KEY_STATUS_UNAVAILABLE,
    MOUNTPOINT_PROPERTY,
    SETUID_PROPERTY,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuidRefusal;

/// What:
///  Why one child of the pinned parent is not a clone this sweep attaches.
/// Why:
///   Each cause sends a person to a different property,
///  and the receipt's failed step is one of these words,
///  so the words are the refusal's own rather than a sentence a caller parses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneRefusal = { kind: 'not-a-child' | 'slot-name' | 'not-a-filesystem' | 'snapshot' | 'guid' | 'no-origin' | 'origin' | 'mountpoint' | 'setuid' | 'devices' | 'property' | 'key-unavailable'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneRefusal {
    /// The name is not one component below the pinned parent.
    NotAChild {
        /// The dataset's name.
        name: String,
        /// The pinned parent.
        parent: String,
    },
    /// The name below the parent is no name this route mounts.
    SlotName {
        /// The dataset's name.
        name: String,
        /// The component below the parent.
        slot: String,
        /// Why the component is not a slot name.
        reason: String,
    },
    /// The object-set type is not a filesystem.
    NotAFilesystem {
        /// The dataset's name.
        name: String,
        /// The type the kernel reported.
        found: u32,
    },
    /// The kernel called the object a snapshot.
    Snapshot {
        /// The dataset's name.
        name: String,
    },
    /// The GUID is zero,
    /// so no receipt could be filed under it.
    NoGuid {
        /// The dataset's name.
        name: String,
    },
    /// The GUID could not be written as a receipt's file name.
    Guid {
        /// The dataset's name.
        name: String,
        /// Why the GUID is not canonical.
        reason: DatasetGuidRefusal,
    },
    /// The dataset is not a clone of anything.
    NoOrigin {
        /// The dataset's name.
        name: String,
    },
    /// The origin is not a snapshot of the checkout dataset the design names.
    OriginOutsideTheCheckout {
        /// The dataset's name.
        name: String,
        /// The origin the kernel reported.
        origin: String,
        /// The checkout dataset whose snapshot the add route clones.
        checkout_dataset: String,
    },
    /// The origin is a snapshot of the checkout dataset,
    /// but not one the add route made.
    OriginIsNotACloneOrigin {
        /// The dataset's name.
        name: String,
        /// The origin the kernel reported.
        origin: String,
        /// The prefix the add route gives its snapshots.
        prefix: &'static str,
    },
    /// The mountpoint property could not be read.
    Property {
        /// The dataset's name.
        name: String,
        /// What the reply said instead of a value.
        said: String,
        /// The receipt's word for this step.
        step: &'static str,
    },
    /// The mountpoint is not `legacy`,
    /// so `zfs mount -a` or the systemd generator could mount the clone first.
    MountpointIsNotLegacy {
        /// The dataset's name.
        name: String,
        /// The value the kernel reported.
        found: String,
    },
    /// The setuid property is on,
    /// so a mount of the clone would carry set-user-ID execution.
    SetuidIsOn {
        /// The dataset's name.
        name: String,
        /// The value the kernel reported.
        found: String,
    },
    /// The devices property is on,
    /// so a mount of the clone would carry device nodes.
    DevicesIsOn {
        /// The dataset's name.
        name: String,
        /// The value the kernel reported.
        found: String,
    },
    /// The key the clone's data is encrypted under is not loaded.
    KeyUnavailable {
        /// The dataset's name.
        name: String,
        /// The dataset whose key is missing,
        /// as the kernel named it.
        encryption_root: String,
        /// What the reply said about the encryption,
        /// so a person can tell an encrypted clone from one whose reply is
        /// incomplete.
        encryption: String,
    },
}

impl CloneRefusal {
    /// The word a receipt and a sweep line name this refusal's step with.
    pub fn step(&self) -> &'static str {
        return match self {
            CloneRefusal::NotAChild {
                ..
            }
            | CloneRefusal::SlotName {
                ..
            } => "clone-name",
            CloneRefusal::NotAFilesystem {
                ..
            }
            | CloneRefusal::Snapshot {
                ..
            } => "clone-type",
            CloneRefusal::NoGuid {
                ..
            }
            | CloneRefusal::Guid {
                ..
            } => "clone-guid",
            CloneRefusal::NoOrigin {
                ..
            }
            | CloneRefusal::OriginOutsideTheCheckout {
                ..
            }
            | CloneRefusal::OriginIsNotACloneOrigin {
                ..
            } => "clone-origin",
            CloneRefusal::Property {
                step,
                ..
            } => step,
            CloneRefusal::MountpointIsNotLegacy {
                ..
            } => "mountpoint",
            CloneRefusal::SetuidIsOn {
                ..
            } => "setuid",
            CloneRefusal::DevicesIsOn {
                ..
            } => "devices",
            CloneRefusal::KeyUnavailable {
                ..
            } => "key-unavailable",
        };
    }

    /// The refusal in words,
    /// naming the dataset,
    /// the property and every way forward.
    pub fn message(&self) -> String {
        let cause: String = match self {
            CloneRefusal::NotAChild {
                name,
                parent,
            } => format!(
                "the dataset `{name}` is not one immediate child of the pinned clone parent `{parent}`, so no slot is derived from its name. Nothing was mounted, unmounted or recorded on its behalf"
            ),
            CloneRefusal::SlotName {
                name,
                slot,
                reason,
            } => format!(
                "the dataset `{name}` sits below the pinned parent as `{slot}`, which is no slot name ({reason}). Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::NotAFilesystem {
                name,
                found,
            } => format!(
                "the dataset `{name}` reports object-set type {found}, and a clone this route attaches is a filesystem (type {DMU_OBJECT_SET_FILESYSTEM}). Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::Snapshot {
                name,
            } => format!(
                "`{name}` is a snapshot, not a clone dataset. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::NoGuid {
                name,
            } => format!(
                "the dataset `{name}` reports GUID 0, which cannot name a receipt file. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::Guid {
                name,
                reason,
            } => format!(
                "the dataset `{name}` reports a GUID this route cannot file a receipt under: {}. Nothing was mounted or recorded on its behalf",
                reason.message()
            ),
            CloneRefusal::NoOrigin {
                name,
            } => format!(
                "the dataset `{name}` is not a clone: the kernel reported no origin, so nothing ties it to a snapshot of the checkout dataset. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::OriginOutsideTheCheckout {
                name,
                origin,
                checkout_dataset,
            } => format!(
                "the clone `{name}` was made from `{origin}`, which is not a snapshot of the checkout dataset `{checkout_dataset}`, so this sweep does not attach it. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::OriginIsNotACloneOrigin {
                name,
                origin,
                prefix,
            } => format!(
                "the clone `{name}` was made from `{origin}`, whose snapshot name does not begin with `{prefix}`, and an origin the add route did not make is not one this sweep attaches. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::Property {
                name,
                said,
                ..
            } => format!(
                "the dataset `{name}` could not be judged: {said}. A property this route cannot read is not treated as absent, because an absent mountpoint and a legacy one are different instructions. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::MountpointIsNotLegacy {
                name,
                found,
            } => format!(
                "the clone `{name}` reports `{MOUNTPOINT_PROPERTY}={found}`, and a clone this route attaches inherits `mountpoint={LEGACY_MOUNTPOINT}` so that neither `zfs mount -a` nor the systemd mount generator mounts it first. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::SetuidIsOn {
                name,
                found,
            } => format!(
                "the clone `{name}` reports `{SETUID_PROPERTY}={found}`, and this route requires `{SETUID_PROPERTY}={INDEX_OFF}` on the datasets it attaches. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::DevicesIsOn {
                name,
                found,
            } => format!(
                "the clone `{name}` reports `{DEVICES_PROPERTY}={found}`, and this route requires `{DEVICES_PROPERTY}={INDEX_OFF}` on the datasets it attaches. Nothing was mounted or recorded on its behalf"
            ),
            CloneRefusal::KeyUnavailable {
                name,
                encryption_root,
                encryption,
            } => format!(
                "the key the clone `{name}` is encrypted under is not loaded: `{KEY_STATUS_PROPERTY}` is {KEY_STATUS_UNAVAILABLE} for the encryption root `{encryption_root}` ({encryption}). This is a load-key question and neither a changed folder identity nor a permission failure. Nothing was mounted or recorded on its behalf"
            ),
        };
        return format!(
            "{cause}. Ways forward: `zfs get -H -o value,source {MOUNTPOINT_PROPERTY},{SETUID_PROPERTY},{DEVICES_PROPERTY} {name}` shows the three properties, `zfs get -H -o value encryptionroot,keystatus <clone>` shows the encryption, and a key that is not loaded is loaded with `zfs load-key <encryptionroot>` before the next sweep or the boot unit's next start.",
            name = self.name()
        );
    }

    /// The dataset this refusal is about.
    pub fn name(&self) -> &str {
        return match self {
            CloneRefusal::NotAChild {
                name,
                ..
            }
            | CloneRefusal::SlotName {
                name,
                ..
            }
            | CloneRefusal::NotAFilesystem {
                name,
                ..
            }
            | CloneRefusal::Snapshot {
                name,
            }
            | CloneRefusal::NoGuid {
                name,
            }
            | CloneRefusal::Guid {
                name,
                ..
            }
            | CloneRefusal::NoOrigin {
                name,
            }
            | CloneRefusal::OriginOutsideTheCheckout {
                name,
                ..
            }
            | CloneRefusal::OriginIsNotACloneOrigin {
                name,
                ..
            }
            | CloneRefusal::Property {
                name,
                ..
            }
            | CloneRefusal::MountpointIsNotLegacy {
                name,
                ..
            }
            | CloneRefusal::SetuidIsOn {
                name,
                ..
            }
            | CloneRefusal::DevicesIsOn {
                name,
                ..
            }
            | CloneRefusal::KeyUnavailable {
                name,
                ..
            } => name.as_str(),
        };
    }
}

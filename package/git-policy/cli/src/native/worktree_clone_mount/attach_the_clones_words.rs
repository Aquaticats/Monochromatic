//! What:
//!  Every word and number one attach speaks in:
//!  the filesystem type a clone is attached as,
//!  the option that names its dataset,
//!  the two attributes the mount carries and the close-on-exec word each call
//!  takes,
//!  the four call names,
//!  what became of the mount,
//!  the refusal a failed call answers,
//!  and the mount the attach made.
//! Why:
//!  Each call is made once and its own errno is kept,
//!  so a refusal names the step a person must look at rather than a summary,
//!  and the sentence,
//!  the errno and every way forward are one message a handler
//!  prints as it stands
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `package/git-policy/cli/SPEC.md`,
//!  "The flags").
//!  The calls themselves are `attach_the_clones_calls.rs`,
//!  and the steps that make them are `attach_the_clone.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AttachRefusal = { call: 'fsopen' | 'fsconfig' | 'fsmount' | 'move_mount' | 'unmount'; source; errno; detail; placement };
//! const refused = refusedFor(call, source, error);   // the sentence a person reads
//! ```

use crate::diagnostic_log::warn;
use std::ffi::c_uint;

/// Module tag for diagnostics.
pub(super) const TAG: &str = "attachTheClone";

/// The filesystem type a clone's dataset is attached as.
pub const ZFS_FILE_SYSTEM_TYPE: &str = "zfs";

/// The context option that names the dataset.
pub const SOURCE_OPTION: &str = "source";

/// What:
///  The flags the mount descriptor is made with:
///  no set-user-ID execution and no device nodes.
/// Why:
///  Item 7 of the design's VM measurements showed a root-owned setuid executable gaining effective UID 0 on a clone
///  mount without these and keeping the account's UID with them,
///  so the attributes are passed to `fsmount` and read back from `mountinfo`
///  (`package/git-policy/cli/SPEC.md`,
///  "The flags").
///  They are the `MOUNT_ATTR_` spelling of the same two bits `ATTACH_MOUNT_FLAGS` carries for a report;
///  `the_two_flag_spellings_are_the_same_two_bits` holds the two together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributes = MOUNT_ATTR_NOSUID | MOUNT_ATTR_NODEV;
/// ```
pub const ATTACH_FSMOUNT_ATTRIBUTES: u64 = libc::MOUNT_ATTR_NOSUID | libc::MOUNT_ATTR_NODEV;

/// What:
///  The `fsopen` and `fsmount` flag that keeps the context and the mount out of a started child.
/// Why:
///  A descriptor another program inherits is a mount it could place;
///  both calls take the flag the kernel names for exactly that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fsopen('zfs', FSOPEN_CLOEXEC); fsmount(context, FSMOUNT_CLOEXEC, attributes);
/// ```
pub const ATTACH_CLOSE_ON_EXEC: (c_uint, c_uint) = (libc::FSOPEN_CLOEXEC, libc::FSMOUNT_CLOEXEC);

/// What:
///  The step of the attach a call belongs to.
/// Why:
///  A receipt names the step that failed,
///  and an operator needs the call rather than "the attach failed",
///  so each step has its own closed word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachCall = 'fsopen' | 'fsconfig-source' | 'fsconfig-create' | 'fsmount' | 'move-mount' | 'mount-id' | 'unmount';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachCall {
    /// `fsopen` of the ZFS filesystem type.
    FileSystemContext,
    /// `fsconfig` naming the dataset.
    SourceOption,
    /// `fsconfig` creating the superblock.
    Create,
    /// `fsmount` making the mount descriptor.
    MountDescriptor,
    /// `move_mount` placing the mount on the held slot.
    Place,
    /// Reading the mount descriptor's own mount number.
    MountNumber,
    /// The descriptor-bound unmount a rollback makes.
    Unmount,
}

impl AttachCall {
    /// The word a receipt and a refusal carry for this step.
    pub fn word(self) -> &'static str {
        return match self {
            AttachCall::FileSystemContext => "fsopen",
            AttachCall::SourceOption => "fsconfig-source",
            AttachCall::Create => "fsconfig-create",
            AttachCall::MountDescriptor => "fsmount",
            AttachCall::Place => "move-mount",
            AttachCall::MountNumber => "mount-id",
            AttachCall::Unmount => "unmount",
        };
    }

    /// The call's own name,
    /// as the kernel's manual names it.
    pub fn call_name(self) -> &'static str {
        return match self {
            AttachCall::FileSystemContext => "fsopen",
            AttachCall::SourceOption => "fsconfig(FSCONFIG_SET_STRING)",
            AttachCall::Create => "fsconfig(FSCONFIG_CMD_CREATE)",
            AttachCall::MountDescriptor => "fsmount",
            AttachCall::Place => "move_mount",
            AttachCall::MountNumber => "the mount number in fdinfo",
            AttachCall::Unmount => "umount2 through the held descriptor",
        };
    }
}

/// What:
///  One failed call of the attach,
///  its errno and the dataset it acted on.
/// Why:
///  The design asks a refusal to name the failing step,
///  and a receipt carries that step and errno beside the outcome,
///  so one value holds both and a caller cannot report an errno without its call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachRefusal = { call: AttachCall; errno: number; source: string; detail: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachRefusal {
    /// The call that refused.
    pub call: AttachCall,
    /// The dataset the attach acted on.
    pub source: String,
    /// The kernel's errno,
    /// or zero when the call answered without one.
    pub errno: i32,
    /// What the call answered,
    /// in the kernel's own words.
    pub detail: String,
    /// What became of the mount this call made,
    /// where the call is later than the move.
    pub placement: Placement,
}

/// What:
///  What became of the mount an attach made before a later call refused.
/// Why:
///  A refusal after the move is not "nothing was mounted":
///  a mount stands at the slot until the route takes it down,
///  and a report that said nothing was mounted would leave it unrecorded and
///  unlooked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placement = 'nothing-placed' | 'placed-then-taken-down' | 'placed-and-retained';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Placement {
    /// No mount was made,
    /// so nothing stands anywhere.
    NothingPlaced,
    /// A mount was placed and the route took it down again,
    /// so the slot holds none.
    PlacedThenTakenDown,
    /// A mount was placed and could not be taken down,
    /// so it stands where the attach left it.
    PlacedAndRetained,
}

impl AttachRefusal {
    /// The refusal in words,
    /// naming the call,
    /// the dataset,
    /// the errno,
    /// what became of the mount and every way forward.
    pub fn message(&self) -> String {
        let placement: &str = match self.placement {
            Placement::NothingPlaced => {
                "Nothing was mounted, moved or recorded by this call, and this clone's mount intent stands so a later sweep may act on it."
            }
            Placement::PlacedThenTakenDown => {
                "A mount had been placed at the slot by this attempt and the route took it down again through the descriptor it acted on, so the slot holds no mount and this clone's mount intent stands."
            }
            Placement::PlacedAndRetained => {
                "A mount had been placed at the slot by this attempt and could not be taken down again, so it stands where the attach left it and an administrator must take it down before the sweep runs again."
            }
        };
        return format!(
            "the clone `{}` could not be attached: {} answered {}{}. {placement} Ways forward: check that the dataset exists and is not already mounted elsewhere, that the kernel's `zfs` filesystem type is available, and read the handler's journal for the same call before the sweep runs again.",
            self.source,
            self.call.call_name(),
            self.detail,
            if self.errno == 0 {
                String::new()
            } else {
                format!(" (errno {})", self.errno)
            },
        );
    }
}

/// What:
///  The mount one attach made,
///  as the descriptor `fsmount` returned reported it after the move.
/// Why:
///  The post-attach check compares the mount the slot shows with the mount the attach made,
///  and only the descriptor names that mount rather than whatever stands at a name now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attached: AttachedMount = { mountId: 41, source: 'pool/clone' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachedMount {
    /// The mount's own number.
    pub mount_id: u64,
    /// The dataset the mount is of.
    pub source: String,
}

/// One failed call with its errno and the kernel's own words.
pub(super) fn refused(
    call: AttachCall,
    source: &str,
    said: std::io::Error,
) -> AttachRefusal {
    warn(
        TAG,
        format!("{} for `{source}` refused: {said}", call.word()).as_str(),
    );
    return AttachRefusal {
        call,
        source: String::from(source),
        errno: said.raw_os_error().unwrap_or(0),
        detail: said.to_string(),
        placement: Placement::NothingPlaced,
    };
}

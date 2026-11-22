//! What:
//!  The product attach itself:
//!  the file-system context of a clone's dataset,
//!  its source,
//!  the create,
//!  the mount descriptor with the two enforced attributes,
//!  the move onto the held slot descriptor,
//!  and the descriptor-bound unmount the rollback uses.
//! Why:
//!  The design attaches through descriptors rather than paths,
//!  enforces `nosuid,nodev` on the mount itself,
//!  and never resolves a path inside the account's tree
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `package/git-policy/cli/SPEC.md`,
//!  "The descriptor mount of a ZFS worktree clone",
//!  "The flags").
//!  Plan step 4 built the walk,
//!  the identities and the post-attach check and left these calls to the guest;
//!  plan step 8 is the first step that attaches a clone,
//!  so the calls live here,
//!  beside the walk they act on,
//!  and the sweep is their first caller.
//!  Each call is made once and its own errno is kept,
//!  so a refusal names the step a person must look at rather than a summary.
//!  The create is the plain `FSCONFIG_CMD_CREATE` rather than the start probe's
//!  `FSCONFIG_CMD_CREATE_EXCL`:
//!  the probe compares its attach with a holder's and uses the exclusive create
//!  as its freshness oracle,
//!  while the product only needs a superblock to mount,
//!  reusing one an earlier mount already made is harmless,
//!  and `nosuid` and `nodev` are per-mount attributes that hold either way.
//!  The walk's refusal of a slot that already stands on a mount is what stands
//!  in for "not already mounted here".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = attachTheClone('pool/clone', heldSlot.target); // the mount's own id
//! unmountThroughDescriptor(heldSlot.target);
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath_mount_facts::mount_id_of;
use std::ffi::{CString, c_int, c_uint};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "attachTheClone";

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
            Placement::NothingPlaced => "Nothing was mounted, moved or recorded by this call, and this clone's mount intent stands so a later sweep may act on it.",
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
            if self.errno == 0 { String::new() } else { format!(" (errno {})", self.errno) },
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
fn refused(call: AttachCall, source: &str, said: std::io::Error) -> AttachRefusal {
    warn(TAG, format!("{} for `{source}` refused: {said}", call.word()).as_str());
    return AttachRefusal { call, source: String::from(source), errno: said.raw_os_error().unwrap_or(0), detail: said.to_string(), placement: Placement::NothingPlaced };
}

/// The open file-system context of one filesystem type.
///
/// # Errors
/// `fsopen` refused.
fn open_context(filesystem: &str, source: &str) -> Result<OwnedFd, AttachRefusal> {
    let file_system_type: CString = CString::new(filesystem).expect("no NUL in a filesystem type");
    // SAFETY: the type string is this call's own and outlives it; the descriptor it returns is checked before it is owned.
    let context: c_int = unsafe { libc::syscall(libc::SYS_fsopen, file_system_type.as_ptr(), ATTACH_CLOSE_ON_EXEC.0) as c_int };
    if context < 0 {
        return Err(refused(AttachCall::FileSystemContext, source, std::io::Error::last_os_error()));
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(context) });
}

/// Name the dataset the context is for.
///
/// # Errors
/// `fsconfig` refused the option.
fn set_source(context: RawFd, source: &str) -> Result<(), AttachRefusal> {
    let key: CString = CString::new(SOURCE_OPTION).expect("no NUL in an option name");
    let value: CString = CString::new(source).expect("no NUL in a dataset name");
    // SAFETY: the context descriptor is the caller's, and both strings outlive the call.
    let set: c_int = unsafe { libc::syscall(libc::SYS_fsconfig, context, libc::FSCONFIG_SET_STRING, key.as_ptr(), value.as_ptr(), 0) as c_int };
    return if set == 0 { Ok(()) } else { Err(refused(AttachCall::SourceOption, source, std::io::Error::last_os_error())) };
}

/// Create the superblock from the context.
///
/// # Errors
/// `fsconfig` refused the create.
fn create(context: RawFd, source: &str) -> Result<(), AttachRefusal> {
    // SAFETY: the context descriptor is the caller's and the call reads no pointer here.
    let created: c_int = unsafe { libc::syscall(libc::SYS_fsconfig, context, libc::FSCONFIG_CMD_CREATE, std::ptr::null::<std::ffi::c_char>(), std::ptr::null::<std::ffi::c_char>(), 0) as c_int };
    return if created == 0 { Ok(()) } else { Err(refused(AttachCall::Create, source, std::io::Error::last_os_error())) };
}

/// Make the mount descriptor with the two enforced attributes.
///
/// # Errors
/// `fsmount` refused.
fn make_mount(context: RawFd, source: &str) -> Result<OwnedFd, AttachRefusal> {
    // SAFETY: the context descriptor is the caller's and has been created; the returned descriptor is owned once.
    let mount: c_int = unsafe { libc::syscall(libc::SYS_fsmount, context, ATTACH_CLOSE_ON_EXEC.1, ATTACH_FSMOUNT_ATTRIBUTES) as c_int };
    if mount < 0 {
        return Err(refused(AttachCall::MountDescriptor, source, std::io::Error::last_os_error()));
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(mount) });
}

/// What:
///  Attach one clone's dataset to the slot descriptor the walk held.
/// Why:
///  The mount is placed on the opened folder itself,
///  never on a path,
///  so a folder renamed after the walk still receives the mount this call made
///  and the post-attach check can say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attached = attachTheClone('pool/clone', held.target);
/// ```
///
/// # Errors
/// Every call's own refusal,
/// with the step a person must look at.
pub fn attach_the_clone(source: &str, target: &std::fs::File) -> Result<AttachedMount, AttachRefusal> {
    return attach_with(ZFS_FILE_SYSTEM_TYPE, source, target);
}

/// What:
///  The attach itself,
///  with the filesystem type named so a control can refuse one this kernel has.
/// Why:
///  A control cannot ask this host to mount a ZFS dataset,
///  and the refusal path is the one a machine without the module takes;
///  naming the type here is what lets that path be proved on a fixture folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attached = attachWith('zfs', 'pool/clone', held.target);
/// ```
///
/// # Errors
/// Every call's own refusal.
fn attach_with(filesystem: &str, source: &str, target: &std::fs::File) -> Result<AttachedMount, AttachRefusal> {
    let context: OwnedFd = open_context(filesystem, source)?;
    set_source(context.as_raw_fd(), source)?;
    create(context.as_raw_fd(), source)?;
    let mount: OwnedFd = make_mount(context.as_raw_fd(), source)?;
    drop(context);
    // The mount descriptor is owned as a file so the same reader the rest of the route uses can read its number and
    // the move can name it; a descriptor keeps naming this mount after the move.
    let held: std::fs::File = std::fs::File::from(mount);
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    // SAFETY: both descriptors are open, and the two empty paths are the NUL-terminated strings the flags tell the
    // call to read from those descriptors.
    let moved: c_int = unsafe {
        libc::syscall(libc::SYS_move_mount, held.as_raw_fd(), empty.as_ptr(), target.as_raw_fd(), empty.as_ptr(), libc::MOVE_MOUNT_F_EMPTY_PATH | libc::MOVE_MOUNT_T_EMPTY_PATH)
            as c_int
    };
    if moved < 0 {
        return Err(refused(AttachCall::Place, source, std::io::Error::last_os_error()));
    }
    let mount_id: u64 = match mount_id_of(&held) {
        Ok(found) => found,
        Err(error) => {
            // The move succeeded, so a mount now stands at the held target whatever this read answered. The descriptor
            // unmount acts on the very folder the move acted on and needs no mount number, so the mount is taken down
            // here rather than left at a slot the route cannot name.
            let placement: Placement = match unmount_through_descriptor(target) {
                Ok(()) => Placement::PlacedThenTakenDown,
                Err(_refusal) => Placement::PlacedAndRetained,
            };
            warn(TAG, format!("the mount number of `{source}` could not be read after the move, and the mount is {placement:?}").as_str());
            return Err(AttachRefusal {
                call: AttachCall::MountNumber,
                source: String::from(source),
                errno: error.raw_os_error().unwrap_or(0),
                detail: error.to_string(),
                placement,
            });
        }
    };
    debug(TAG, format!("attached `{source}` as mount {mount_id}").as_str());
    return Ok(AttachedMount { mount_id, source: String::from(source) });
}

/// What:
///  Take one mount down through the descriptor the attach acted through.
/// Why:
///  The design's rollback acts on the held descriptor and never on a path inside the account's tree,
///  so the kernel resolves this process's own descriptor entry
///  (`/proc/self/fd/<n>`),
///  which follows the folder even after a rename,
///  and neither force nor lazy detach is passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unmountThroughDescriptor(held.target);
/// ```
///
/// # Errors
/// The unmount's refusal,
/// which is a `EBUSY` while another process holds the mount.
pub fn unmount_through_descriptor(target: &std::fs::File) -> Result<(), AttachRefusal> {
    let named: CString = CString::new(format!("/proc/self/fd/{}", target.as_raw_fd())).expect("no NUL in a descriptor path");
    // SAFETY: `named` is a NUL-terminated string that outlives the call, and no flag asks for force or lazy detach.
    let taken: c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
    return if taken == 0 {
        Ok(())
    } else {
        Err(refused(AttachCall::Unmount, target_place(target).as_str(), std::io::Error::last_os_error()))
    };
}

/// The path a refusal names for the folder a descriptor holds,
/// or a word for the descriptor itself when the kernel keeps the path out of reach.
fn target_place(target: &std::fs::File) -> String {
    let path: PathBuf = std::fs::read_link(format!("/proc/self/fd/{}", target.as_raw_fd())).unwrap_or_else(|_error| return PathBuf::from("the held slot"));
    return path.display().to_string();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "attach_the_clone_tests.rs"]
mod tests;

//! What:
//!  The product attach itself:
//!  the four calls in their one order,
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
//!  Plan step 8 is the first step that attaches a clone,
//!  so the calls live beside the walk they act on and the sweep is their first
//!  caller.
//!  The words this file answers with are `attach_the_clones_words.rs`,
//!  and each kernel call is `attach_the_clones_calls.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = attachTheClone('pool/clone', heldSlot.target); // the mount's own id
//! unmountThroughDescriptor(heldSlot.target);
//! ```

use super::attach_the_clones_calls::{
    create,
    make_mount,
    open_context,
    set_source,
};
// What:     every word and shape of this module's attach, re-exported where
//           they were declared before the split, so a caller that names this
//           module still finds them.
// Why:      the route's other files name these through this module's path,
//           and a split that moved them silently would break every one of them.
//
// In TS you'd write (pseudocode):
// ```ts
// export { AttachCall, AttachRefusal, AttachedMount, Placement, ZFS_FILE_SYSTEM_TYPE } from './attachTheClonesWords.js';
// ```
pub use super::attach_the_clones_words::{
    ATTACH_CLOSE_ON_EXEC,
    ATTACH_FSMOUNT_ATTRIBUTES,
    AttachCall,
    AttachRefusal,
    AttachedMount,
    Placement,
    SOURCE_OPTION,
    ZFS_FILE_SYSTEM_TYPE,
};
use super::attach_the_clones_words::{
    TAG,
    refused,
};
use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::fs_beneath_mount_facts::mount_id_of;
use std::ffi::{
    CString,
    c_int,
};
use std::os::fd::{
    AsRawFd,
    OwnedFd,
};
use std::path::PathBuf;

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
pub fn attach_the_clone(
    source: &str,
    target: &std::fs::File,
) -> Result<AttachedMount, AttachRefusal> {
    return attach_with(
        ZFS_FILE_SYSTEM_TYPE,
        source,
        target,
        ATTACH_FSMOUNT_ATTRIBUTES,
        &mount_id_of,
    );
}

/// What:
///  Place one dataset at the folder a descriptor holds,
///  with the `fsmount` attributes the caller resolved.
/// Why:
///  The clone attach fixes its two attributes because every clone has the same
///  pair,
///  while a dataset mounted at its own mountpoint carries whatever its own
///  properties resolve to,
///  so the attribute set is the caller's and the four calls are the same one
///  sequence: `fsopen`, `fsconfig` twice, `fsmount`, `move_mount`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placed = attachWithAttributes('pool/nested', held.target, MOUNT_ATTR_NOATIME);
/// ```
///
/// # Errors
/// Every call's own refusal,
/// with the step a person must look at.
pub fn attach_with_attributes(
    source: &str,
    target: &std::fs::File,
    attributes: u64,
) -> Result<AttachedMount, AttachRefusal> {
    return attach_with(
        ZFS_FILE_SYSTEM_TYPE,
        source,
        target,
        attributes,
        &mount_id_of,
    );
}

/// What:
///  The attach itself,
///  with the filesystem type and the mount-number read named so a control can
///  drive either path.
/// Why:
///  A control cannot ask this host to mount a ZFS dataset,
///  and the refusal path is the one a machine without the module takes;
///  naming the type here is what lets that path be proved on a fixture folder.
///  The mount-number read is the one call after the move that can fail while a
///  mount stands,
///  and a control that fails it proves the take-down rather than trusting that
///  the call never fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attached = attachWith('zfs', 'pool/clone', held.target, mountIdOf);
/// ```
///
/// # Errors
/// Every call's own refusal.
fn attach_with<ReadMountNumber>(
    filesystem: &str,
    source: &str,
    target: &std::fs::File,
    attributes: u64,
    read_mount_number: &ReadMountNumber,
) -> Result<AttachedMount, AttachRefusal>
where
    ReadMountNumber: Fn(&std::fs::File) -> std::io::Result<u64>,
{
    let context: OwnedFd = open_context(filesystem, source)?;
    set_source(context.as_raw_fd(), source)?;
    create(context.as_raw_fd(), source)?;
    let mount: OwnedFd = make_mount(context.as_raw_fd(), source, attributes)?;
    drop(context);
    // The mount descriptor is owned as a file so the same reader the rest of the route uses can read its number and
    // the move can name it; a descriptor keeps naming this mount after the move.
    let held: std::fs::File = std::fs::File::from(mount);
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    // SAFETY: both descriptors are open, and the two empty paths are the NUL-terminated strings the flags tell the
    // call to read from those descriptors.
    let moved: c_int = unsafe {
        libc::syscall(
            libc::SYS_move_mount,
            held.as_raw_fd(),
            empty.as_ptr(),
            target.as_raw_fd(),
            empty.as_ptr(),
            libc::MOVE_MOUNT_F_EMPTY_PATH | libc::MOVE_MOUNT_T_EMPTY_PATH,
        ) as c_int
    };
    if moved < 0 {
        return Err(refused(
            AttachCall::Place,
            source,
            std::io::Error::last_os_error(),
        ));
    }
    let mount_id: u64 = match read_mount_number(&held) {
        Ok(found) => found,
        Err(error) => {
            // The move succeeded, so a mount now stands at the held target whatever this read answered. The descriptor
            // unmount acts on the very folder the move acted on and needs no mount number, so the mount is taken down
            // here rather than left at a slot the route cannot name. The mount descriptor is dropped first because an
            // open descriptor for the mount itself keeps the kernel's unmount answering EBUSY.
            drop(held);
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
    debug(
        TAG,
        format!("attached `{source}` as mount {mount_id}").as_str(),
    );
    return Ok(AttachedMount {
        mount_id,
        source: String::from(source),
    });
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
    let named: CString = CString::new(format!("/proc/self/fd/{}", target.as_raw_fd()))
        .expect("no NUL in a descriptor path");
    // SAFETY: `named` is a NUL-terminated string that outlives the call, and no flag asks for force or lazy detach.
    let taken: c_int = unsafe { libc::umount2(named.as_ptr(), 0) };
    return if taken == 0 {
        Ok(())
    } else {
        Err(refused(
            AttachCall::Unmount,
            target_place(target).as_str(),
            std::io::Error::last_os_error(),
        ))
    };
}

/// The path a refusal names for the folder a descriptor holds,
/// or a word for the descriptor itself when the kernel keeps the path out of reach.
fn target_place(target: &std::fs::File) -> String {
    let path: PathBuf = std::fs::read_link(format!("/proc/self/fd/{}", target.as_raw_fd()))
        .unwrap_or_else(|_error| return PathBuf::from("the held slot"));
    return path.display().to_string();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "attach_the_clone_tests.rs"]
mod tests;

//! What:
//!  The walk from a held directory down a chain of one-name hops,
//!  as every handle it opened or as the deepest one,
//!  and the last hop of a walk that may cross a mount at the folder it ends on.
//! Why:
//!  A caller that must compare a parent folder's identity with the one it recorded,
//!  and then act on the folder below it by descriptor,
//!  needs the parent and the target as separate handles from one walk:
//!  a second lookup by path can reach another object,
//!  and the parent's identity is what proves where the target hangs
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  One loop owns the device and boundary policy for every walk in this folder,
//!  so a hop cannot be walked with one rule here and another rule beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const chain = openChainBeneath(checkout, ['.claude', 'worktrees', 'slot']); // one handle per component
//! const deepest = openDirectoryBeneath(checkout, ['package', 'cli']);          // the last handle only
//! const mounted = openDirectoryAtAMount(worktrees, 'my-slot');                 // crosses an existing mount at the slot
//! ```

use super::fs_beneath::{RESOLVE_BENEATH, RESOLVE_NO_MAGICLINKS, RESOLVE_NO_SYMLINKS, device_of, open_beneath_by, open_directory_at};
use std::ffi::{OsStr, OsString};
use std::fs::File;

/// What:
///  Why a walk of one chain stopped,
///  and which component stopped it.
/// Why:
///   The component's position is what a caller turns into the folder's name for the person,
/// and the kind is what chooses the sentence:
/// a link planted in the way,
/// a boundary the walk never crosses,
/// or another failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChainRefusal = { kind: 'link' | 'boundary' | 'unreadable'; step: number; error?: Error };
/// ```
#[derive(Debug)]
pub enum ChainRefusal {
    /// The component at `step` is a link,
    /// which is never followed.
    Link {
        /// Position of the component in the chain.
        step: usize,
    },
    /// The component at `step` is a mount point or a folder on another device.
    Boundary {
        /// Position of the component in the chain.
        step: usize,
    },
    /// The component at `step` could not be opened,
    /// for this reason.
    Unreadable {
        /// Position of the component in the chain.
        step: usize,
        /// What the open answered.
        error: std::io::Error,
    },
}

impl ChainRefusal {
    /// The refusal for the open failure at `step`,
    /// by the error number it carries.
    fn of(step: usize, error: std::io::Error) -> ChainRefusal {
        return match error.raw_os_error() {
            Some(libc::ELOOP) => ChainRefusal::Link { step },
            Some(libc::EXDEV) => ChainRefusal::Boundary { step },
            _ => ChainRefusal::Unreadable { step, error },
        };
    }

    /// The error a caller that only needs the deepest handle answers for this refusal.
    ///
    /// Why:
    ///  Such a caller acts on a folder or reports a failure and never names the component,
    /// so the position is the whole of what it needs;
    /// the positional sentence keeps a link and a boundary apart from each other.
    fn as_io_error(self) -> std::io::Error {
        return match self {
            ChainRefusal::Unreadable { error, .. } => error,
            ChainRefusal::Link { step } => std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("component {} of the walk is a symbolic link", step + 1)),
            ChainRefusal::Boundary { step } => std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("component {} of the walk is a mount point or on another device", step + 1)),
        };
    }
}

/// What:
///  One hop of a walk:
///  the folder `name` below `parent`,
///  refused when it is a link,
///  a mount point or a folder on another device than `device`.
/// Why:
///   Every walk in this folder takes this one hop,
///  so the device and boundary policy stands in one place:
///  the chain keeps each hop it takes and the deepest-handle walk keeps only the one it is at,
///  and neither can drift to another rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const next = openOneHop(parent, 'worktrees', checkoutDevice, 1); // throws ChainRefusal
/// ```
fn open_one_hop(parent: &File, name: &OsStr, device: u64, step: usize) -> Result<File, ChainRefusal> {
    let next: File = open_directory_at(parent, name).map_err(|error| return ChainRefusal::of(step, error))?;
    if device_of(&next).map_err(|error| return ChainRefusal::Unreadable { step, error })? != device {
        return Err(ChainRefusal::Boundary { step });
    }
    return Ok(next);
}

/// What:
///  The handles of the directories `names` below `root`,
///  one per component,
///  each opened as `open_directory_at` opens them.
/// Why:
///   A caller that must compare a parent folder's identity with the one it recorded,
///  and then act on the folder below it by descriptor,
///  needs the parent and the target as separate handles from one walk:
///  a second lookup by path can reach another object,
///  and the parent's identity is what proves where the target hangs.
///  Every component is refused when it is a link,
///  a mount point or a folder on another device,
///  and the refusal says which hop stopped the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [claude, worktrees, slot] = openChainBeneath(checkout, ['.claude', 'worktrees', 'slot']);
/// ```
///
/// # Errors
/// The reason of the first component the walk refuses, and its position.
pub fn open_chain_beneath(root: &File, names: &[OsString]) -> Result<Vec<File>, ChainRefusal> {
    let device: u64 = device_of(root).map_err(|error| return ChainRefusal::Unreadable { step: 0, error })?;
    let mut chain: Vec<File> = Vec::with_capacity(names.len());
    for (step, name) in names.iter().enumerate() {
        let parent: &File = match chain.last() {
            Some(held) => held,
            None => root,
        };
        chain.push(open_one_hop(parent, name.as_os_str(), device, step)?);
    }
    return Ok(chain);
}

/// Open the directory at `names` below `root`,
/// one component at a time,
/// refusing a link,
/// a mount point or a directory on
///  another device (a subvolume of a btrfs filesystem has its own).
///
/// Why:
///  A caller that acts on the deepest folder and names no hop takes the same hop the chain above
///  takes,
///  so no second loop here can keep another device or boundary policy;
///  only the handles kept differ,
///  because this one holds one descriptor at a time and answers a path of any depth.
pub fn open_directory_beneath(root: &File, names: &[OsString]) -> std::io::Result<File> {
    let device: u64 = device_of(root)?;
    let mut current: File = root.try_clone()?;
    for (step, name) in names.iter().enumerate() {
        current = open_one_hop(&current, name.as_os_str(), device, step).map_err(|refusal| return refusal.as_io_error())?;
    }
    return Ok(current);
}

/// What:
///  Open the directory `name` of `parent` as `open_directory_at` opens it,
///  except that a mount point at `name` itself is crossed.
/// Why:
///   A caller that reads the folder a previous attach mounted a clone on needs the mounted root,
///  while every component above that folder must still be the walked ones:
///  the walk refuses a mount anywhere else,
///  and only this last hop may be one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mounted = openDirectoryAtAMount(worktrees, 'my-slot'); // crosses an existing mount at the slot only
/// ```
pub fn open_directory_at_a_mount(parent: &File, name: &OsStr) -> std::io::Result<File> {
    return open_beneath_by(parent, name, libc::O_RDONLY | libc::O_DIRECTORY, 0, RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_beneath_walk_chain_tests.rs"]
mod tests;

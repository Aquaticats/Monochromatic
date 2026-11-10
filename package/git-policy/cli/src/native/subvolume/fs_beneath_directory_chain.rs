//! What:
//!  The directories a path walk has already opened,
//!  kept as one chain from the root,
//! so a walk over paths in order opens each directory once.
//! Why:
//!  A tracked-file walk asks for the parent of every entry in turn;
//! opening that parent from the root again,
//!  once per directory change,
//! re-opens and re-examines every directory above it,
//! which on a checkout of twenty thousand files is thousands of
//!  lookups that name directories already held.
//! The chain keeps the handles,
//! drops the ones the next path does not share,
//! and opens only the
//!  components below the part it shares,
//! with the same refusals as `open_directory_beneath`:
//! no link followed,
//! no
//!  mount crossed,
//! and no directory on another device
//! (a nested btrfs subvolume has one of its own).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const chain = new DirectoryChain(root); // reads the root's device once
//! const directory = chain.directory(['package', 'cli', 'src']); // the handle those names lead to
//! ```

use super::fs_beneath::{device_of, open_directory_at};
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;

/// What:
///  One directory of the chain:
/// its name below the directory above it,
/// and the handle it was opened as.
/// Why:
///   The next path shares a prefix with the one before by name,
/// and the handle of each shared name is what the walk does not
///  open again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Opened = { name: string; directory: File };
/// ```
struct Opened {
    /// The component below the directory above it.
    name: OsString,
    /// The opened directory.
    directory: File,
}

/// What:
///  A root and the chain of directories opened below it,
/// with the root's device read once.
/// Why:
///   Every entry of a walk is reached through handles,
/// so no directory on the way is resolved twice,
/// and the
///  device every opened directory is compared against is the same one for the whole walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DirectoryChain { root: File; device: number; opened: Opened[]; }
/// ```
pub struct DirectoryChain<'root> {
    /// The held root every path is found below.
    root: &'root File,
    /// The device of the root,
    /// read once: a directory on another device is another filesystem or a nested subvolume.
    device: u64,
    /// The directories open below the root,
    /// nearest to it first.
    opened: Vec<Opened>,
}

/// What:
///  How many leading components the chain and `names` share,
/// by name.
/// Why:
///   That prefix is what the chain keeps open;
/// everything below it is opened again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shared = (opened, names) => opened.findIndex((each, index) => each.name !== names[index]);
/// ```
fn shared_prefix(opened: &[Opened], names: &[OsString]) -> usize {
    let mut shared: usize = 0;
    while shared < opened.len() && shared < names.len() {
        if opened[shared].name != names[shared] {
            return shared;
        }
        shared += 1;
    }
    return shared;
}

impl<'root> DirectoryChain<'root> {
    /// What:
    ///  A chain that has opened nothing below `root`,
    /// with the device it refuses to leave.
    /// Why:
    ///   A walk reads its root's device once rather than once per directory change;
    /// the root itself must be a plain
    ///  directory handle, which the caller opened.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const chain = new DirectoryChain(root); // throws when the root cannot be examined
    /// ```
    pub fn new(root: &'root File) -> std::io::Result<Self> {
        return Ok(Self { root, device: device_of(root)?, opened: Vec::new() });
    }

    /// What:
    ///  The directory `names` lead to below the root,
    /// opening only the components the chain does not already hold.
    /// Why:
    ///   A name that is a link,
    /// a mount point or a directory on another device is refused exactly as opening the whole
    ///  path would refuse it;
    /// what the chain skips is the work of opening and examining the components it still holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const directory = chain.directory(['package', 'cli']); // the handle, or why it is refused
    /// ```
    pub fn directory(&mut self, names: &[OsString]) -> std::io::Result<&File> {
        let shared: usize = shared_prefix(self.opened.as_slice(), names);
        self.opened.truncate(shared);
        for name in &names[shared..] {
            let parent: &File = match self.opened.last() {
                Some(opened) => &opened.directory,
                None => self.root,
            };
            let next: File = open_directory_at(parent, name.as_os_str())?;
            if device_of(&next)? != self.device {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} is on another device", name.to_string_lossy())));
            }
            self.opened.push(Opened { name: name.clone(), directory: next });
        }
        return match self.opened.last() {
            Some(deepest) => Ok(&deepest.directory),
            None => Ok(self.root),
        };
    }

    /// What:
    ///  The names this chain holds open,
    /// nearest the root first.
    /// Why:
    ///   A control asks what the chain skipped,
    /// and no other reader needs the handles themselves.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// chain.heldNames(); // ['package', 'cli'] after directory(['package', 'cli'])
    /// ```
    pub fn held_names(&self) -> Vec<&OsStr> {
        return self.opened.iter().map(|opened| opened.name.as_os_str()).collect();
    }
}

/// Chain controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_beneath_directory_chain_tests.rs"]
mod tests;

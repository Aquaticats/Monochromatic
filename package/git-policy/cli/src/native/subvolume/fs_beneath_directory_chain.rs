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
//! It keeps at most `MOST_DIRECTORIES_HELD` directories:
//! a deeper path is walked one
//!  descriptor at a time,
//! as every path was walked before the chain existed,
//! so no depth costs a descriptor per component.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const chain = new DirectoryChain(root); // reads the root's device once
//! const directory = chain.directory(['package', 'cli', 'src']); // the handle those names lead to
//! ```

use super::fs_beneath::{device_of, open_directory_at, open_directory_beneath};
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;

/// What:
///  How many directories one chain keeps open at most:
/// `32`,
/// one open descriptor each.
/// Why:
///   The chain held every ancestor of the path it walked,
/// so a valid checkout a
///  thousand directories deep held a thousand descriptors and the walk failed with `EMFILE`,
/// where the walk this replaced held one
///  directory at a time and answered the same paths;
/// 32 is above the deepest tracked path of this repository
/// (18 components)
/// and
///  the deepest path of the shapes an add meets,
/// so the saving stays where it is paid and a deeper path is walked
///  one descriptor at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MOST_DIRECTORIES_HELD = 32;
/// ```
const MOST_DIRECTORIES_HELD: usize = 32;

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
    /// read once:
    /// a directory on another device is another filesystem or a nested subvolume.
    device: u64,
    /// The directories open below the root,
    /// nearest to it first.
    opened: Vec<Opened>,
    /// The directory a path too deep for the chain to keep was found through,
    /// held alone;
    /// nothing when the last path was walked through the chain.
    deep_answer: Option<File>,
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

/// What:
///  The name one opened directory of the chain was opened as.
/// Why:
///   A control asks what the chain holds,
/// and naming that through a closure would be a function that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameOf = (opened) => opened.name;
/// ```
fn name_of(opened: &Opened) -> &OsStr {
    return opened.name.as_os_str();
}

impl<'root> DirectoryChain<'root> {
    /// What:
    ///  A chain that has opened nothing below `root`,
    /// with the device it refuses to leave.
    /// Why:
    ///   A walk reads its root's device once rather than once per directory change;
    /// the root itself must be a plain
    ///  directory handle,
    /// which the caller opened.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const chain = new DirectoryChain(root); // throws when the root cannot be examined
    /// ```
    pub fn new(root: &'root File) -> std::io::Result<Self> {
        // The chain has opened nothing, and no path has been too deep for it yet.
        return Ok(Self { root, device: device_of(root)?, opened: Vec::new(), deep_answer: None });
    }

    /// What:
    ///  The directory `names` lead to below the root,
    /// opening only the components the chain does not already hold.
    /// Why:
    ///   A name that is a link,
    /// a mount point or a directory on another device is refused exactly as opening the whole
    ///  path would refuse it;
    /// what the chain skips is the work of opening and examining the components it still holds.
    /// A path deeper than the chain may keep is walked one descriptor at a time
    ///  instead,
    /// so a valid depth is answered at any descriptor limit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const directory = chain.directory(['package', 'cli']); // the handle, or why it is refused
    /// ```
    pub fn directory(&mut self, names: &[OsString]) -> std::io::Result<&File> {
        // Keeping every ancestor of a deeper path would hold one descriptor per component, which a valid checkout
        //  can exceed; such a path is walked one directory at a time, as it was before the chain.
        if names.len() > MOST_DIRECTORIES_HELD {
            return self.too_deep_to_keep(names);
        }
        // A chain walked now answers through the handles it keeps; the answer a too-deep path left is not one of them.
        self.deep_answer = None;
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
        return Ok(self.deepest());
    }

    /// What:
    ///  The directory `names` lead to,
    /// opened one component at a time,
    /// with the answer held alone.
    /// Why:
    ///   A path deeper than the chain may keep would need a descriptor per component;
    /// the walk this replaced
    ///  held one directory at a time and answered the same paths,
    /// so a deep checkout is walked as it was then.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const directory = chain.tooDeepToKeep(names); // the same handle, one open at a time
    /// ```
    fn too_deep_to_keep(&mut self, names: &[OsString]) -> std::io::Result<&File> {
        let found: File = open_directory_beneath(self.root, names)?;
        self.opened.clear();
        self.deep_answer = Some(found);
        return Ok(self.deepest());
    }

    /// What:
    ///  The directory this chain answers with:
    /// the one a path too deep to keep was found through,
    /// else the deepest it keeps,
    /// else the root.
    /// Why:
    ///   A caller needs one handle whichever of the two walks found it,
    /// and the root is what a path with no components answers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const deepest = () => chain.deepAnswer ?? chain.opened.at(-1)?.directory ?? chain.root;
    /// ```
    fn deepest(&self) -> &File {
        if let Some(found) = self.deep_answer.as_ref() {
            return found;
        }
        return match self.opened.last() {
            Some(deepest) => &deepest.directory,
            None => self.root,
        };
    }

    /// What:
    ///  The names this chain holds open,
    /// nearest the root first;
    /// none for a path too deep to keep.
    /// Why:
    ///   A control asks what the chain skipped,
    /// and no other reader needs the handles themselves.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// chain.heldNames(); // ['package', 'cli'] after directory(['package', 'cli'])
    /// ```
    pub fn held_names(&self) -> Vec<&OsStr> {
        return self.opened.iter().map(name_of).collect();
    }
}

/// Chain controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_beneath_directory_chain_tests.rs"]
mod tests;

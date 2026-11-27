//! What:
//!  One directory of the folder of saved copies,
//! held open,
//! counted and emptied:
//! the walk that counts a directory's files without changing anything,
//! and the walk that removes them,
//! files of a directory before its subdirectories,
//! and stops at the first failure with what it had reached.
//! Why:
//!  The directory that was judged is the only one removed,
//! so the walk goes through the descriptor of that directory and never through a path,
//! and never enters a link.
//! Counting what went before a failure is what lets a partial removal be audited as partial.
//! Files come before subdirectories so that a failure deep in the tree leaves a predictable remainder.
//! `discarded_store.rs` opens the directory,
//! and `discarded_prune_removal.rs` empties it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tally = await directory.survey(); // counts every file below it
//! const removed = await directory.empty(); // removes them, leaving the directory
//! ```
/// The system calls this walk is built from.
use crate::discarded_store_calls_by_descriptor::{
    identity_in,
    is_directory,
    names_inside,
    open_inside,
    owned,
    status_of,
    unlink_inside,
};
/// The identity of a directory,
/// the walk's depth bound and what a walk left.
use crate::discarded_store_records::{
    Identity,
    Interrupted,
    MAX_DEPTH,
    Tally,
};
/// `OsString` is an owned name in bytes.
use std::ffi::OsString;
/// `OwnedFd` closes a descriptor when dropped.
use std::os::fd::{
    AsRawFd,
    OwnedFd,
};
/// A directory held open;
/// `OwnedFd` closes it when the value is dropped.
#[derive(Debug)]
pub struct Directory {
    /// The directory's descriptor.
    descriptor: OwnedFd,
}

/// One directory of the walk:
/// its descriptor,
/// its name in the directory above,
/// its relative path,
/// and the
/// subdirectories still to enter.
struct Frame {
    /// The directory's descriptor.
    descriptor: OwnedFd,
    /// The name it has in the directory above,
    /// or nothing for the root of the walk.
    name: Option<OsString>,
    /// The path below the root of the walk,
    /// with a trailing `/`,
    /// or empty for the root.
    prefix: String,
    /// The subdirectories not yet entered.
    directories: Vec<OsString>,
}

impl Directory {
    /// The identity of the directory itself.
    ///
    /// # Errors
    /// The system's error when the directory cannot be inspected.
    pub fn identity(&self) -> std::io::Result<Identity> {
        // SAFETY: `stat` is plain data for which all zero bytes is a valid value.
        let mut status: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: the descriptor is open and `status` is valid for one `stat`.
        let result: libc::c_int =
            unsafe { libc::fstat(self.descriptor.as_raw_fd(), &raw mut status) };
        if result != 0 {
            return Err(std::io::Error::last_os_error());
        }
        return Ok(identity_in(&status));
    }

    /// What `walk` does for a file met:
    /// counted,
    /// and removed when `remove`;
    /// a file this call did not remove itself is not counted
    /// when removing,
    /// because the pass that removed it counts it.
    /// A directory another pass already removed holds nothing.
    fn meet_files(
        frame: &mut Frame,
        remove: bool,
        tally: &mut Tally,
    ) -> std::io::Result<()> {
        let names: Vec<OsString> = match names_inside(frame.descriptor.as_raw_fd()) {
            Ok(found) => found,
            Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return Ok(()),
            Err(error) => return Err(error),
        };
        for name in names {
            let status: libc::stat = match status_of(frame.descriptor.as_raw_fd(), name.as_os_str())
            {
                Ok(found) => found,
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => continue,
                Err(error) => return Err(error),
            };
            if is_directory(&status) {
                frame.directories.push(name);
                continue;
            }
            if remove && !unlink_inside(frame.descriptor.as_raw_fd(), name.as_os_str(), false)? {
                continue;
            }
            tally.bytes += u128::try_from(status.st_size).unwrap_or(0);
            tally
                .names
                .push(format!("{}{}", frame.prefix, name.to_string_lossy()));
        }
        return Ok(());
    }

    /// What:
    ///  Counts the files below this directory,
    /// or removes them and the subdirectories,
    /// files of a directory before its
    ///  subdirectories,
    /// stopping at the first failure.
    /// Why:
    ///   Files first makes a failure deep in the tree leave a predictable remainder,
    /// and stopping keeps the rest of an
    ///  unreadable or protected tree rather than guessing.
    /// A link is a file here and is never entered.
    /// The directory
    ///  itself is left for the caller to remove.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const tally = await directory.walk(remove);
    /// ```
    fn walk(
        &self,
        remove: bool,
    ) -> Result<Tally, Interrupted> {
        let mut tally: Tally = Tally::default();
        let outcome: std::io::Result<()> = self.walk_into(remove, &mut tally);
        return match outcome {
            Ok(()) => Ok(tally),
            Err(error) => Err(Interrupted {
                tally,
                error,
            }),
        };
    }

    /// The walk of `walk`,
    /// adding to `tally` as it goes so that a failure still says how far it got.
    fn walk_into(
        &self,
        remove: bool,
        tally: &mut Tally,
    ) -> std::io::Result<()> {
        // SAFETY: the descriptor is open; the duplicate is owned by the frame.
        let root: OwnedFd =
            owned(unsafe { libc::fcntl(self.descriptor.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) })?;
        let mut first: Frame = Frame {
            descriptor: root,
            name: None,
            prefix: String::new(),
            directories: Vec::new(),
        };
        Directory::meet_files(&mut first, remove, tally)?;
        let mut stack: Vec<Frame> = vec![first];
        loop {
            let depth: usize = stack.len();
            let Some(top) = stack.last_mut() else { break };
            if let Some(name) = top.directories.pop() {
                if depth > MAX_DEPTH {
                    return Err(std::io::Error::other(format!(
                        "the tree is deeper than {MAX_DEPTH} levels"
                    )));
                }
                let prefix: String = format!("{}{}/", top.prefix, name.to_string_lossy());
                let inner: OwnedFd = match open_inside(top.descriptor.as_raw_fd(), name.as_os_str())
                {
                    Ok(opened) => opened,
                    Err(error) if error.raw_os_error() == Some(libc::ENOENT) => continue,
                    Err(error) => return Err(error),
                };
                let mut frame: Frame = Frame {
                    descriptor: inner,
                    name: Some(name),
                    prefix,
                    directories: Vec::new(),
                };
                Directory::meet_files(&mut frame, remove, tally)?;
                stack.push(frame);
                continue;
            }
            let Some(done) = stack.pop() else { break };
            if let (true, Some(name), Some(parent)) = (remove, done.name.as_ref(), stack.last()) {
                let _mine: bool =
                    unlink_inside(parent.descriptor.as_raw_fd(), name.as_os_str(), true)?;
            }
        }
        return Ok(());
    }

    /// Counts the files below this directory without changing anything.
    ///
    /// # Errors
    /// The first failure,
    /// with what was counted before it.
    pub fn survey(&self) -> Result<Tally, Interrupted> {
        return self.walk(false);
    }

    /// Removes every file and subdirectory below this directory,
    /// leaving the directory itself.
    ///
    /// # Errors
    /// The first failure,
    /// with what was removed before it.
    pub fn empty(&self) -> Result<Tally, Interrupted> {
        return self.walk(true);
    }
}

/// What:
///  The directory open on `descriptor`,
/// for the store that opened its name.
/// Why:
///  The store opens a directory of the folder and hands the descriptor here,
/// so the field stays private to this file and nothing else can put another descriptor in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = Directory.holding(fd);
/// ```
impl Directory {
    pub(crate) fn holding(descriptor: OwnedFd) -> Directory {
        return Directory {
            descriptor,
        };
    }
}

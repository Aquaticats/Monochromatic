//! What:
//!  The folder of saved copies (`<common git dir>/monochromatic-git-policy-cli/discarded`) opened once without following a link,
//! and every
//!  read and removal inside it addressed through descriptors:
//! the folder's,
//! then each directory's,
//! never a path walked again.
//! Why:
//!  A pass that lists a path and later removes a path can be led anywhere by a link or a rename that happens in between.
//!  Holding the folder and each directory open means a removal lands in the directory that was judged,
//! a name that now stands
//!  for another directory is noticed by comparing what it is (device and inode, never the change time its files
//! moving and another pass emptying it both move) with what the listing saw,
//! and a
//!  link met anywhere inside is removed as a link and never entered.
//! No lock keeps two passes from removing the same tree:
//!  each file has one owner,
//! the pass whose `unlinkat` of it succeeded,
//! and only that pass counts it,
//! so passes that meet the
//!  same directory share its files and every file is counted once.
//! A file or directory another pass already removed is not an
//!  error.
//! The repository's descriptor-relative module for the btrfs work is not on this branch yet;
//! `subvolume_anchor.rs`
//!  widens the permissions of a directory it cannot enter,
//! which a removal must never do.
//! The system calls are `discarded_store_calls_by_descriptor.rs`,
//! and one directory held open and walked is `discarded_store_directory_walk.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const store = await openStore(commonDir);          // throws on a link at monochromatic-git-policy-cli or discarded
//! const directory = await store.open('2026-11-02T14-01-07'); // never follows a link
//! await directory.empty();                           // unlinkat by descriptor, files before subdirectories
//! ```

/// A directory held open,
/// which the walk in `discarded_store_directory_walk.rs` holds.
#[cfg(unix)]
pub use crate::discarded_store_directory_walk::Directory;
/// The plain values of the folder's reader,
/// which this file and its siblings share.
pub use crate::discarded_store_records::{
    Identity,
    Interrupted,
    MAX_DEPTH,
    Tally,
};

/// The store a system without descriptor-relative removal has:
/// none.
#[cfg(not(unix))]
pub use other::{
    Directory,
    OpenFailure,
    Store,
};

/// The system calls the store is built from.
#[cfg(unix)]
use crate::discarded_store_calls_by_descriptor::{
    c_name,
    identity_in,
    is_directory,
    names_inside,
    open_inside,
    owned,
    status_of,
    unlink_inside,
};
/// `CString` is a NUL-terminated name,
/// and `OsStr`/`OsString` are borrowed/owned names in bytes.
#[cfg(unix)]
use std::ffi::{
    CString,
    OsStr,
    OsString,
};
/// `OwnedFd` closes a descriptor when dropped.
#[cfg(unix)]
use std::os::fd::{
    AsRawFd,
    OwnedFd,
};
/// `Path` is a borrowed filesystem path.
#[cfg(unix)]
use std::path::Path;
/// Why the folder could not be opened as a store.
#[derive(Debug)]
pub enum OpenFailure {
    /// A symbolic link,
    /// or something that is not a directory,
    /// stands at `monochromatic-git-policy-cli` or `discarded`.
    Link,
    /// The system refused to open it.
    Refused(std::io::Error),
}

/// The folder of saved copies,
/// held open by one pass.
#[derive(Debug)]
pub struct Store {
    /// The `discarded` directory's descriptor.
    descriptor: OwnedFd,
}

impl Store {
    /// What:
    ///  Opens `<common_dir>/monochromatic-git-policy-cli/discarded`,
    /// following a link in `common_dir` (Git names it) and none below it.
    ///  Nothing when the folder does not exist.
    /// Why:
    ///   A link at `monochromatic-git-policy-cli` or `discarded` would send every later read and removal to wherever it points.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const store = await Store.open(commonDir); // undefined when absent
    /// ```
    ///
    /// # Errors
    /// `OpenFailure::Link` when a link or a non-directory stands at a component below `common_dir`;
    /// `OpenFailure::Refused` for any other refusal.
    pub fn open(
        common_dir: &Path,
        relative: &str,
    ) -> Result<Option<Store>, OpenFailure> {
        let text: CString = c_name(common_dir.as_os_str()).map_err(OpenFailure::Refused)?;
        // SAFETY: `text` is a NUL-terminated string that outlives the call.
        let root: std::io::Result<OwnedFd> = owned(unsafe {
            libc::open(
                text.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
            )
        });
        let mut current: OwnedFd = root.map_err(OpenFailure::Refused)?;
        for component in relative.split('/') {
            current = match open_inside(current.as_raw_fd(), OsStr::new(component)) {
                Ok(opened) => opened,
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return Ok(None),
                Err(error) if matches!(error.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR)) => {
                    return Err(OpenFailure::Link);
                }
                Err(error) => return Err(OpenFailure::Refused(error)),
            };
        }
        return Ok(Some(Store {
            descriptor: current,
        }));
    }

    /// The names inside the folder.
    ///
    /// # Errors
    /// The system's error when the folder cannot be read.
    pub fn names(&self) -> std::io::Result<Vec<OsString>> {
        return names_inside(self.descriptor.as_raw_fd());
    }

    /// What `name` is inside the folder without following a link:
    /// its identity when it is a directory,
    /// nothing when it
    /// is anything else (a link,
    /// a file) or is gone.
    ///
    /// # Errors
    /// The system's error when it cannot be inspected for a reason other than being gone.
    pub fn directory_identity(
        &self,
        name: &OsStr,
    ) -> std::io::Result<Option<Identity>> {
        return match status_of(self.descriptor.as_raw_fd(), name) {
            Ok(status) if is_directory(&status) => Ok(Some(identity_in(&status))),
            Ok(_) => Ok(None),
            Err(error) if error.raw_os_error() == Some(libc::ENOENT) => Ok(None),
            Err(error) => Err(error),
        };
    }

    /// Opens the directory `name` of the folder,
    /// never following a link;
    /// nothing when it is gone.
    ///
    /// # Errors
    /// `NotADirectory` when a link or a file stands at `name`;
    /// otherwise the system's error,
    /// `EACCES` for a directory
    /// the person cannot read.
    pub fn open_directory(
        &self,
        name: &OsStr,
    ) -> std::io::Result<Option<Directory>> {
        return match open_inside(self.descriptor.as_raw_fd(), name) {
            Ok(descriptor) => Ok(Some(Directory::holding(descriptor))),
            Err(error) if error.raw_os_error() == Some(libc::ENOENT) => Ok(None),
            Err(error) if matches!(error.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR)) => {
                Err(std::io::ErrorKind::NotADirectory.into())
            }
            Err(error) => Err(error),
        };
    }

    /// Removes the empty directory `name` of the folder when it is still the directory `expected` describes,
    /// by device and
    /// inode (its change time moved when its contents went).
    /// A name that now stands for another directory is left,
    /// and a
    /// directory that is not empty is never removed by this call.
    /// True when this call removed it;
    /// false when the name no
    /// longer stands for that directory,
    /// whether another pass removed it first or something else took the name.
    ///
    /// # Errors
    /// The system's error when the removal fails,
    /// `ENOTEMPTY` included.
    pub fn remove_empty_directory(
        &self,
        name: &OsStr,
        expected: Identity,
    ) -> std::io::Result<bool> {
        let same: bool = match self.directory_identity(name)? {
            Some(found) => found.device == expected.device && found.inode == expected.inode,
            None => false,
        };
        if !same {
            return Ok(false);
        }
        return unlink_inside(self.descriptor.as_raw_fd(), name, true);
    }
}

#[cfg(not(unix))]
mod other {
    use super::{
        Identity,
        Interrupted,
        Tally,
    };
    use std::ffi::{
        OsStr,
        OsString,
    };
    use std::path::Path;

    /// Why the folder could not be opened as a store.
    #[derive(Debug)]
    pub enum OpenFailure {
        /// A symbolic link,
        /// or something that is not a directory,
        /// stands in the way.
        Link,
        /// The system refused to open it,
        /// or does not offer a way to do it safely.
        Refused(std::io::Error),
    }

    /// A directory held open:
    /// none can be on a system without descriptor-relative removal.
    #[derive(Debug)]
    pub struct Directory(std::convert::Infallible);

    impl Directory {
        /// The identity of the directory itself.
        ///
        /// # Errors
        /// Never;
        /// no value of this type exists here.
        pub fn identity(&self) -> std::io::Result<Identity> {
            match self.0 {}
        }

        /// Counts the files below the directory.
        ///
        /// # Errors
        /// Never;
        /// no value of this type exists here.
        pub fn survey(&self) -> Result<Tally, Interrupted> {
            match self.0 {}
        }

        /// Removes what is below the directory.
        ///
        /// # Errors
        /// Never;
        /// no value of this type exists here.
        pub fn empty(&self) -> Result<Tally, Interrupted> {
            match self.0 {}
        }
    }

    /// The folder of saved copies:
    /// none can be opened here,
    /// because a removal that cannot be addressed by descriptor is not
    /// done.
    #[derive(Debug)]
    pub struct Store(std::convert::Infallible);

    impl Store {
        /// Always refuses:
        /// this system offers no way to remove inside the folder without following a replaced path.
        ///
        /// # Errors
        /// `OpenFailure::Refused`,
        /// always.
        pub fn open(
            _common_dir: &Path,
            _relative: &str,
        ) -> Result<Option<Store>, OpenFailure> {
            return Err(OpenFailure::Refused(std::io::Error::other(
                "removing inside the folder without following a replaced path is not available on this system",
            )));
        }

        /// The names inside the folder.
        ///
        /// # Errors
        /// Never;
        /// no value of this type exists here.
        pub fn names(&self) -> std::io::Result<Vec<OsString>> {
            match self.0 {}
        }

        /// What `name` is inside the folder.
        ///
        /// # Errors
        /// Never;
        /// no value of this type exists here.
        pub fn directory_identity(
            &self,
            _name: &OsStr,
        ) -> std::io::Result<Option<Identity>> {
            match self.0 {}
        }

        /// Opens the directory `name` of the folder.
        ///
        /// # Errors
        /// Never;
        /// no value of this type exists here.
        pub fn open_directory(
            &self,
            _name: &OsStr,
        ) -> std::io::Result<Option<Directory>> {
            match self.0 {}
        }

        /// Removes the empty directory `name` of the folder.
        ///
        /// # Errors
        /// Never;
        /// no value of this type exists here.
        pub fn remove_empty_directory(
            &self,
            _name: &OsStr,
            _expected: Identity,
        ) -> std::io::Result<bool> {
            match self.0 {}
        }
    }
}

/// The store's controls.
#[cfg(test)]
#[path = "discarded_store_tests.rs"]
mod tests;

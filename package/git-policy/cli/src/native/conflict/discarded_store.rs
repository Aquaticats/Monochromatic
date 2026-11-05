//! What:
//!  The folder of saved copies (`<common git dir>/monochromatic-git-policy-cli/discarded`) opened once without following a link, and every
//!  read and removal inside it addressed through descriptors: the folder's, then each directory's, never a path walked again.
//! Why:
//!  A pass that lists a path and later removes a path can be led anywhere by a link or a rename that happens in between.
//!  Holding the folder and each directory open means a removal lands in the directory that was judged, a name that now stands
//!  for another directory is noticed by comparing what it is (device, inode and change time) with what the listing saw, and a
//!  link met anywhere inside is removed as a link and never entered. No lock keeps two passes from removing the same tree:
//!  each file has one owner, the pass whose `unlinkat` of it succeeded, and only that pass counts it, so passes that meet the
//!  same directory share its files and every file is counted once. A file or directory another pass already removed is not an
//!  error. The repository's descriptor-relative module for the btrfs work is not on this branch yet; `subvolume_anchor.rs`
//!  widens the permissions of a directory it cannot enter, which a removal must never do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const store = await openStore(commonDir);          // throws on a link at monochromatic-git-policy-cli or discarded
//! const directory = await store.open('2026-11-02T14-01-07'); // never follows a link
//! await directory.empty();                           // unlinkat by descriptor, files before subdirectories
//! ```

pub use crate::discarded_store_records::{Identity, Interrupted, MAX_DEPTH, Tally};
#[cfg(unix)]
pub use unix::{Directory, OpenFailure, Store};

#[cfg(not(unix))]
pub use other::{Directory, OpenFailure, Store};

#[cfg(unix)]
mod unix {
    use super::{Identity, Interrupted, MAX_DEPTH, Tally};
    use std::ffi::{CString, OsStr, OsString};
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use std::path::Path;

    /// Why the folder could not be opened as a store.
    #[derive(Debug)]
    pub enum OpenFailure {
        /// A symbolic link, or something that is not a directory, stands at `monochromatic-git-policy-cli` or `discarded`.
        Link,
        /// The system refused to open it.
        Refused(std::io::Error),
    }

    /// The name as the C string the system calls take.
    fn c_name(name: &OsStr) -> std::io::Result<CString> {
        return match CString::new(name.as_bytes()) {
            Ok(text) => Ok(text),
            Err(_) => Err(std::io::Error::from_raw_os_error(libc::EINVAL)),
        };
    }

    /// Takes ownership of a descriptor a system call returned, or the error it left.
    fn owned(descriptor: libc::c_int) -> std::io::Result<OwnedFd> {
        if descriptor < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: the call returned a new descriptor that nothing else owns.
        return Ok(unsafe { OwnedFd::from_raw_fd(descriptor) });
    }

    /// Opens the directory `name` inside `parent` for reading, never following a link at `name`.
    fn open_inside(parent: RawFd, name: &OsStr) -> std::io::Result<OwnedFd> {
        let text: CString = c_name(name)?;
        // SAFETY: `text` is a NUL-terminated string that outlives the call, and `parent` is an open descriptor.
        return owned(unsafe { libc::openat(parent, text.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) });
    }

    /// What `fstatat` says of `name` inside `parent`, without following a link at `name`.
    fn status_of(parent: RawFd, name: &OsStr) -> std::io::Result<libc::stat> {
        let text: CString = c_name(name)?;
        // SAFETY: `stat` is plain data for which all zero bytes is a valid value.
        let mut status: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: `text` is a NUL-terminated string that outlives the call, and `status` is valid for one `stat`.
        let result: libc::c_int = unsafe { libc::fstatat(parent, text.as_ptr(), &raw mut status, libc::AT_SYMLINK_NOFOLLOW) };
        if result != 0 {
            return Err(std::io::Error::last_os_error());
        }
        return Ok(status);
    }

    /// The identity a `stat` describes.
    fn identity_in(status: &libc::stat) -> Identity {
        return Identity {
            device: u64::from(status.st_dev),
            inode: u64::from(status.st_ino),
            changed_seconds: i64::from(status.st_ctime),
            changed_nanos: i64::from(status.st_ctime_nsec),
        };
    }

    /// Whether a `stat` describes a directory.
    fn is_directory(status: &libc::stat) -> bool {
        return status.st_mode & libc::S_IFMT == libc::S_IFDIR;
    }

    /// Sets the thread's error number to zero, so that the end of a directory can be told from a failed read.
    fn clear_error_number() {
        // SAFETY: the location is the calling thread's own error number, valid for a write.
        #[cfg(any(target_os = "linux", target_os = "android"))]
        unsafe {
            *libc::__errno_location() = 0;
        }
        // SAFETY: the location is the calling thread's own error number, valid for a write.
        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios"))]
        unsafe {
            *libc::__error() = 0;
        }
    }

    /// The names inside the directory open on `descriptor`, `.` and `..` left out.
    fn names_inside(descriptor: RawFd) -> std::io::Result<Vec<OsString>> {
        // SAFETY: `descriptor` is open; the duplicate is owned by the stream `fdopendir` makes of it.
        let duplicate: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_DUPFD_CLOEXEC, 0) };
        if duplicate < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: `duplicate` is a directory descriptor nothing else uses.
        let stream: *mut libc::DIR = unsafe { libc::fdopendir(duplicate) };
        if stream.is_null() {
            let error: std::io::Error = std::io::Error::last_os_error();
            // SAFETY: `fdopendir` failed, so `duplicate` is still ours to close.
            unsafe { libc::close(duplicate) };
            return Err(error);
        }
        // The duplicate shares its position with the descriptor, so an earlier read would leave this one at the end.
        // SAFETY: `stream` is an open directory stream used only by this thread.
        unsafe { libc::rewinddir(stream) };
        let mut names: Vec<OsString> = Vec::new();
        let mut failure: Option<std::io::Error> = None;
        loop {
            clear_error_number();
            // SAFETY: `stream` is an open directory stream used only by this thread.
            let entry: *mut libc::dirent = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                let error: std::io::Error = std::io::Error::last_os_error();
                if matches!(error.raw_os_error(), Some(number) if number != 0) {
                    failure = Some(error);
                }
                break;
            }
            // SAFETY: `readdir` returned a valid entry whose name is NUL-terminated; it is copied before the next call.
            let name: Vec<u8> = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes().to_vec();
            if name != b"." && name != b".." {
                names.push(OsString::from_vec(name));
            }
        }
        // SAFETY: `stream` is open and closed once; this also closes the duplicate.
        unsafe { libc::closedir(stream) };
        return match failure {
            Some(error) => Err(error),
            None => Ok(names),
        };
    }

    /// What:
    ///  Removes the entry `name` of `parent`: a directory when `directory`, otherwise anything that is not one. Whether this
    ///  call removed it.
    /// Why:
    ///   The kernel lets exactly one `unlinkat` of a name succeed, so the caller that gets `true` is the entry's one owner and
    ///  the only one that may count it. An entry another pass removed first is gone, which is the aim reached and no error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const mine = await unlinkAt(parent, name, directory) // false: somebody else removed it first
    /// ```
    fn unlink_inside(parent: RawFd, name: &OsStr, directory: bool) -> std::io::Result<bool> {
        let text: CString = c_name(name)?;
        let flags: libc::c_int = if directory { libc::AT_REMOVEDIR } else { 0 };
        // SAFETY: `text` is a NUL-terminated string that outlives the call, and `parent` is an open descriptor.
        let result: libc::c_int = unsafe { libc::unlinkat(parent, text.as_ptr(), flags) };
        if result != 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            // Gone already (another pass removed it): the aim is reached, and the other pass owns the count.
            if error.raw_os_error() == Some(libc::ENOENT) {
                return Ok(false);
            }
            return Err(error);
        }
        return Ok(true);
    }

    /// A directory held open; `OwnedFd` closes it when the value is dropped.
    #[derive(Debug)]
    pub struct Directory {
        /// The directory's descriptor.
        descriptor: OwnedFd,
    }

    /// One directory of the walk: its descriptor, its name in the directory above, its relative path, and the
    /// subdirectories still to enter.
    struct Frame {
        /// The directory's descriptor.
        descriptor: OwnedFd,
        /// The name it has in the directory above, or nothing for the root of the walk.
        name: Option<OsString>,
        /// The path below the root of the walk, with a trailing `/`, or empty for the root.
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
            let result: libc::c_int = unsafe { libc::fstat(self.descriptor.as_raw_fd(), &raw mut status) };
            if result != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(identity_in(&status));
        }

        /// What `walk` does for a file met: counted, and removed when `remove`; a file this call did not remove itself is not counted
        /// when removing, because the pass that removed it counts it. A directory another pass already removed holds nothing.
        fn meet_files(frame: &mut Frame, remove: bool, tally: &mut Tally) -> std::io::Result<()> {
            let names: Vec<OsString> = match names_inside(frame.descriptor.as_raw_fd()) {
                Ok(found) => found,
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return Ok(()),
                Err(error) => return Err(error),
            };
            for name in names {
                let status: libc::stat = match status_of(frame.descriptor.as_raw_fd(), name.as_os_str()) {
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
                tally.names.push(format!("{}{}", frame.prefix, name.to_string_lossy()));
            }
            return Ok(());
        }

        /// What:
        ///  Counts the files below this directory, or removes them and the subdirectories, files of a directory before its
        ///  subdirectories, stopping at the first failure.
        /// Why:
        ///   Files first makes a failure deep in the tree leave a predictable remainder, and stopping keeps the rest of an
        ///  unreadable or protected tree rather than guessing. A link is a file here and is never entered. The directory
        ///  itself is left for the caller to remove.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// const tally = await directory.walk(remove);
        /// ```
        fn walk(&self, remove: bool) -> Result<Tally, Interrupted> {
            let mut tally: Tally = Tally::default();
            let outcome: std::io::Result<()> = self.walk_into(remove, &mut tally);
            return match outcome {
                Ok(()) => Ok(tally),
                Err(error) => Err(Interrupted { tally, error }),
            };
        }

        /// The walk of `walk`, adding to `tally` as it goes so that a failure still says how far it got.
        fn walk_into(&self, remove: bool, tally: &mut Tally) -> std::io::Result<()> {
            // SAFETY: the descriptor is open; the duplicate is owned by the frame.
            let root: OwnedFd = owned(unsafe { libc::fcntl(self.descriptor.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) })?;
            let mut first: Frame = Frame { descriptor: root, name: None, prefix: String::new(), directories: Vec::new() };
            Directory::meet_files(&mut first, remove, tally)?;
            let mut stack: Vec<Frame> = vec![first];
            loop {
                let depth: usize = stack.len();
                let Some(top) = stack.last_mut() else { break };
                if let Some(name) = top.directories.pop() {
                    if depth > MAX_DEPTH {
                        return Err(std::io::Error::other(format!("the tree is deeper than {MAX_DEPTH} levels")));
                    }
                    let prefix: String = format!("{}{}/", top.prefix, name.to_string_lossy());
                    let inner: OwnedFd = match open_inside(top.descriptor.as_raw_fd(), name.as_os_str()) {
                        Ok(opened) => opened,
                        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => continue,
                        Err(error) => return Err(error),
                    };
                    let mut frame: Frame = Frame { descriptor: inner, name: Some(name), prefix, directories: Vec::new() };
                    Directory::meet_files(&mut frame, remove, tally)?;
                    stack.push(frame);
                    continue;
                }
                let Some(done) = stack.pop() else { break };
                if let (true, Some(name), Some(parent)) = (remove, done.name.as_ref(), stack.last()) {
                    let _mine: bool = unlink_inside(parent.descriptor.as_raw_fd(), name.as_os_str(), true)?;
                }
            }
            return Ok(());
        }

        /// Counts the files below this directory without changing anything.
        ///
        /// # Errors
        /// The first failure, with what was counted before it.
        pub fn survey(&self) -> Result<Tally, Interrupted> {
            return self.walk(false);
        }

        /// Removes every file and subdirectory below this directory, leaving the directory itself.
        ///
        /// # Errors
        /// The first failure, with what was removed before it.
        pub fn empty(&self) -> Result<Tally, Interrupted> {
            return self.walk(true);
        }
    }

    /// The folder of saved copies, held open by one pass.
    #[derive(Debug)]
    pub struct Store {
        /// The `discarded` directory's descriptor.
        descriptor: OwnedFd,
    }

    impl Store {
        /// What:
        ///  Opens `<common_dir>/monochromatic-git-policy-cli/discarded`, following a link in `common_dir` (Git names it) and none below it.
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
        pub fn open(common_dir: &Path, relative: &str) -> Result<Option<Store>, OpenFailure> {
            let text: CString = c_name(common_dir.as_os_str()).map_err(OpenFailure::Refused)?;
            // SAFETY: `text` is a NUL-terminated string that outlives the call.
            let root: std::io::Result<OwnedFd> = owned(unsafe { libc::open(text.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) });
            let mut current: OwnedFd = root.map_err(OpenFailure::Refused)?;
            for component in relative.split('/') {
                current = match open_inside(current.as_raw_fd(), OsStr::new(component)) {
                    Ok(opened) => opened,
                    Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return Ok(None),
                    Err(error) if matches!(error.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR)) => return Err(OpenFailure::Link),
                    Err(error) => return Err(OpenFailure::Refused(error)),
                };
            }
            return Ok(Some(Store { descriptor: current }));
        }

        /// The names inside the folder.
        ///
        /// # Errors
        /// The system's error when the folder cannot be read.
        pub fn names(&self) -> std::io::Result<Vec<OsString>> {
            return names_inside(self.descriptor.as_raw_fd());
        }

        /// What `name` is inside the folder without following a link: its identity when it is a directory, nothing when it
        /// is anything else (a link, a file) or is gone.
        ///
        /// # Errors
        /// The system's error when it cannot be inspected for a reason other than being gone.
        pub fn directory_identity(&self, name: &OsStr) -> std::io::Result<Option<Identity>> {
            return match status_of(self.descriptor.as_raw_fd(), name) {
                Ok(status) if is_directory(&status) => Ok(Some(identity_in(&status))),
                Ok(_) => Ok(None),
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => Ok(None),
                Err(error) => Err(error),
            };
        }

        /// Opens the directory `name` of the folder, never following a link; nothing when it is gone.
        ///
        /// # Errors
        /// `NotADirectory` when a link or a file stands at `name`; otherwise the system's error, `EACCES` for a directory
        /// the person cannot read.
        pub fn open_directory(&self, name: &OsStr) -> std::io::Result<Option<Directory>> {
            return match open_inside(self.descriptor.as_raw_fd(), name) {
                Ok(descriptor) => Ok(Some(Directory { descriptor })),
                Err(error) if error.raw_os_error() == Some(libc::ENOENT) => Ok(None),
                Err(error) if matches!(error.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR)) => Err(std::io::ErrorKind::NotADirectory.into()),
                Err(error) => Err(error),
            };
        }

        /// Removes the empty directory `name` of the folder when it is still the directory `expected` describes, by device and
        /// inode (its change time moved when its contents went). A name that now stands for another directory is left, and a
        /// directory that is not empty is never removed by this call. True when this call removed it; false when the name no
        /// longer stands for that directory, whether another pass removed it first or something else took the name.
        ///
        /// # Errors
        /// The system's error when the removal fails, `ENOTEMPTY` included.
        pub fn remove_empty_directory(&self, name: &OsStr, expected: Identity) -> std::io::Result<bool> {
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
}

#[cfg(not(unix))]
mod other {
    use super::{Identity, Interrupted, Tally};
    use std::ffi::{OsStr, OsString};
    use std::path::Path;

    /// Why the folder could not be opened as a store.
    #[derive(Debug)]
    pub enum OpenFailure {
        /// A symbolic link, or something that is not a directory, stands in the way.
        Link,
        /// The system refused to open it, or does not offer a way to do it safely.
        Refused(std::io::Error),
    }

    /// A directory held open: none can be on a system without descriptor-relative removal.
    #[derive(Debug)]
    pub struct Directory(std::convert::Infallible);

    impl Directory {
        /// The identity of the directory itself.
        ///
        /// # Errors
        /// Never; no value of this type exists here.
        pub fn identity(&self) -> std::io::Result<Identity> {
            match self.0 {}
        }

        /// Counts the files below the directory.
        ///
        /// # Errors
        /// Never; no value of this type exists here.
        pub fn survey(&self) -> Result<Tally, Interrupted> {
            match self.0 {}
        }

        /// Removes what is below the directory.
        ///
        /// # Errors
        /// Never; no value of this type exists here.
        pub fn empty(&self) -> Result<Tally, Interrupted> {
            match self.0 {}
        }
    }

    /// The folder of saved copies: none can be opened here, because a removal that cannot be addressed by descriptor is not
    /// done.
    #[derive(Debug)]
    pub struct Store(std::convert::Infallible);

    impl Store {
        /// Always refuses: this system offers no way to remove inside the folder without following a replaced path.
        ///
        /// # Errors
        /// `OpenFailure::Refused`, always.
        pub fn open(_common_dir: &Path, _relative: &str) -> Result<Option<Store>, OpenFailure> {
            return Err(OpenFailure::Refused(std::io::Error::other("removing inside the folder without following a replaced path is not available on this system")));
        }

        /// The names inside the folder.
        ///
        /// # Errors
        /// Never; no value of this type exists here.
        pub fn names(&self) -> std::io::Result<Vec<OsString>> {
            match self.0 {}
        }

        /// What `name` is inside the folder.
        ///
        /// # Errors
        /// Never; no value of this type exists here.
        pub fn directory_identity(&self, _name: &OsStr) -> std::io::Result<Option<Identity>> {
            match self.0 {}
        }

        /// Opens the directory `name` of the folder.
        ///
        /// # Errors
        /// Never; no value of this type exists here.
        pub fn open_directory(&self, _name: &OsStr) -> std::io::Result<Option<Directory>> {
            match self.0 {}
        }

        /// Removes the empty directory `name` of the folder.
        ///
        /// # Errors
        /// Never; no value of this type exists here.
        pub fn remove_empty_directory(&self, _name: &OsStr, _expected: Identity) -> std::io::Result<bool> {
            match self.0 {}
        }
    }
}

/// The store's controls.
#[cfg(test)]
#[path = "discarded_store_tests.rs"]
mod tests;

//! What:
//!  A directory held open, and names only this run holds under it,
//!  so what a check saw is what the act that follows meets.
//! Why:
//!  Four reviews in a row found the same shape at a further seam:
//!  a check made on a name, then an act made on that name again,
//!  and a proof about a name is not a proof about what the act will meet.
//!  So a directory the filling removes is first taken aside, never replacing,
//!  to a name of this run's own under the directory this run holds,
//!  and the claim is read again there before anything is deleted;
//!  a marker is taken aside the same way and its identity compared again there.
//!  A child program is given a path through this run's own descriptor
//!  (`/proc/<this process>/fd/<descriptor>/<name>`),
//!  so the child reaches the very directory this run holds,
//!  never the process the child itself is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prewarm/theHeldName.ts: the base of the prewarm family (the marker and the job both use it).
//! const held = openDirectory(path);                                        // O_DIRECTORY | O_NOFOLLOW
//! const aside = renameNoReplace(entry(held, name), entry(held, anOwnName(name))); // never replaces
//! readClaim(at(aside)); removeTree(aside);                                 // checked and acted on the same entry
//! childEnv.CARGO_BUILD_BUILD_DIR = `/proc/${process.pid}/fd/${held.fd}/${name}`;
//! ```

use crate::rename_that_never_replaces::rename_noreplace;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  A name under the directory this run holds that no other command holds:
///  the entry an act will meet, moved there without replacing anything.
/// Why:
///   A directory is removed, or a marker unlinked, only after it was taken aside to this name:
/// whatever arrived at the public name after the check is no longer what the act reaches,
/// and the entry here is the one this run read and verified.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheNameThisRunHolds = { at: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TheNameThisRunHolds {
    /// The path that reaches the entry this run holds.
    at: PathBuf,
}

impl TheNameThisRunHolds {
    /// What:
    ///  The path of the entry this run holds.
    /// Why:
    ///   Every act on the entry is made through this path,
    /// which names the entry under the directory this run holds and not a public name.
    pub(crate) fn at(&self) -> &Path {
        return self.at.as_path();
    }
}

/// What:
///  A name of this run's own derived from one the filling uses,
///  so a name a kill leaves behind is still a name the exclude patterns name.
/// Why:
///   The staging, its target and its marker are named by patterns of the form `<name>-*`,
/// and a taken-aside entry has to keep that shape
/// or the person's `git status` would show what a killed filling left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anOwnName = (named) => `${named}-${process.pid}-${nanos().toString(16)}`;
/// ```
pub(crate) fn a_name_only_this_run_holds(named: &Path) -> PathBuf {
    let since_the_epoch: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap_or_default();
    let nanos: u128 = since_the_epoch.as_nanos();
    let mut taken: OsString = named.as_os_str().to_os_string();
    taken.push(format!("-{}-{nanos:x}", std::process::id()));
    return PathBuf::from(taken);
}

/// What:
///  The directory the names live in,
///  held open,
///  and the path that reaches it through the descriptor.
/// Why:
///   A worktree can be removed and another added at the same path while a filling works on the names inside it,
/// and then a lease proves nothing:
/// the old filling holds a descriptor on a file no name reaches,
/// so the directory this run opened is what every removal is addressed through,
/// and a directory made again at the same path is a different directory.
///   The descriptor says nothing about which entry stands under a name inside it:
/// that is what taking the entry aside and reading its claim there is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theDirectory = (path) => { const held = openSync(path, O_DIRECTORY | O_NOFOLLOW); return { held, at: `/proc/self/fd/${held}` }; };
/// const entry = (directory, name) => join(directory.at, name);
/// ```
pub(crate) struct TheDirectory {
    /// The descriptor of the directory the names live in.
    /// It is what keeps the path beside it pointing at this directory and not at whatever stands there later.
    #[allow(dead_code, reason = "the descriptor is what the path beside it reaches: dropping it closes the path")]
    held: std::fs::File,
    /// The path that reaches that directory through the descriptor,
    /// `/proc/self/fd/<the descriptor>`.
    at: PathBuf,
}

impl TheDirectory {
    /// What:
    ///  Take the directory the names live in, without following a link,
    /// and answer the path that reaches it through the descriptor.
    /// Why:
    ///   The descriptor is what keeps the names this run took pointing at the directory it took them in,
    ///  whatever now stands at the path.
    ///
    /// # Errors
    ///
    /// A directory that cannot be opened.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// TheDirectory.of = (path) => { const held = openSync(path, O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC); return { held, at: `/proc/self/fd/${held.fd}` }; };
    /// ```
    pub(crate) fn of(directory: &Path) -> Result<TheDirectory, String> {
        use std::os::unix::io::AsRawFd as _;
        let opened: std::io::Result<std::fs::File> = {
            use std::os::unix::fs::OpenOptionsExt as _;
            std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(directory)
        };
        return match opened {
            Ok(held) => Ok(TheDirectory { at: PathBuf::from(format!("/proc/self/fd/{}", held.as_raw_fd())), held }),
            Err(error) => Err(format!("{} could not be taken as the directory the staging names live in ({error})", directory.display())),
        };
    }

    /// What:
    ///  The path of this name as it stands in the directory taken here,
    ///  for this process.
    /// Why:
    ///   Every removal, every claim and the move aside go through this path,
    /// so none of them can reach a directory that was made at the same path afterwards.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const entry = (directory, named) => join(directory.at, basename(named));
    /// ```
    pub(crate) fn along(&self, named: &Path) -> PathBuf {
        return match named.file_name() {
            Some(name) => self.at.join(name),
            None => self.at.clone(),
        };
    }

    /// What:
    ///  The path of this name as a child program must resolve it,
    ///  through this process's own descriptor.
    /// Why:
    ///   `/proc/self` inside a child names the child,
    /// and this run's descriptor is close-on-exec,
    /// so `/proc/self/fd/<n>` would name nothing there or, worse, whatever the child opened at that number.
    /// A path through this process's descriptor names the very directory this run holds for as long as the
    /// child runs, whoever resolves it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const forAChild = (directory, named) => `/proc/${process.pid}/fd/${directory.held.fd}/${basename(named)}`;
    /// ```
    pub(crate) fn for_a_child(&self, named: &Path) -> PathBuf {
        use std::os::unix::io::AsRawFd as _;
        let mut reached: PathBuf = PathBuf::from(format!("/proc/{}/fd/{}", std::process::id(), self.held.as_raw_fd()));
        if let Some(name) = named.file_name() {
            reached.push(name);
        }
        return reached;
    }

    /// What:
    ///  Take the entry at this name aside, never replacing anything,
    ///  and answer it,
    ///  or answer nothing when no name is there.
    /// Why:
    ///   The rename is the one step that decides which entry the act will meet,
    /// and `RENAME_NOREPLACE` refuses rather than replacing a name this run's own name already holds.
    ///
    /// # Errors
    ///
    /// A rename that failed for a reason other than a name that is not there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const takeAside = (directory, named, aside) => renameNoReplace(entry(directory, named), entry(directory, aside));
    /// ```
    pub(crate) fn take_aside(&self, named: &Path, aside: &Path) -> Result<Option<TheNameThisRunHolds>, String> {
        let from: PathBuf = self.along(named);
        let to: PathBuf = self.along(aside);
        return match rename_noreplace(from.as_path(), to.as_path()) {
            Ok(()) => Ok(Some(TheNameThisRunHolds { at: to })),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("{} could not be taken aside ({error})", named.display())),
        };
    }

    /// What:
    ///  Put an entry this run took aside back where it was found,
    ///  never replacing whatever stands there now.
    /// Why:
    ///   A replacement that arrived before the rename is not this run's,
    /// so it goes back to the name its writer used and is left,
    /// with the one line the caller writes saying what was met.
    ///
    /// # Errors
    ///
    /// A name that is taken again, or an entry that is no longer there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const putBack = (directory, held, named) => renameNoReplace(held.at, entry(directory, named));
    /// ```
    pub(crate) fn put_back(&self, held: &TheNameThisRunHolds, named: &Path) -> Result<(), String> {
        let to: PathBuf = self.along(named);
        return match rename_noreplace(held.at(), to.as_path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
                "{} was taken again while this run put back what it found, so what it found stands at {}",
                named.display(),
                held.at().display()
            )),
            Err(error) => Err(format!("what this run found at {} could not be put back ({error})", named.display())),
        };
    }

    /// What:
    ///  Remove the directory tree at a name this run holds.
    /// Why:
    ///   No other command holds this name,
    /// and the claim was read again at it,
    ///  so the tree that is removed is the one this run verified.
    ///
    /// # Errors
    ///
    /// A tree that cannot be removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const removeTheHeldTree = (directory, held) => rmSync(held.at, { recursive: true });
    /// ```
    pub(crate) fn remove_the_held_tree(&self, held: &TheNameThisRunHolds) -> Result<(), String> {
        return match std::fs::remove_dir_all(held.at()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("{} could not be removed ({error})", held.at().display())),
        };
    }

    /// What:
    ///  Unlink the file at a name this run holds.
    /// Why:
    ///   The file was taken aside and verified at that name,
    /// so the unlink reaches the very file this run read.
    ///
    /// # Errors
    ///
    /// A file that cannot be unlinked.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const removeTheHeldFile = (directory, held) => rmSync(held.at, { force: true });
    /// ```
    pub(crate) fn remove_the_held_file(&self, held: &TheNameThisRunHolds) -> Result<(), String> {
        return match std::fs::remove_file(held.at()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("{} could not be removed ({error})", held.at().display())),
        };
    }
}

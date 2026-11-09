//! What:
//!  The stand-in platform and the fixture checkouts of the subvolume migration controls.
//! Why:
//!  The migration's journal,
//! resume,
//! rollback,
//! refusal and index logic does not need btrfs,
//! only the five operations
//!  of `Platform` do.
//! The stand-in answers them on a plain directory tree:
//! a "subvolume" is a directory registered by
//!  its device and inode (an identity that survives the exchange and a rename,
//! as a real subvolume root's does),
//! file
//!  clones are refused so the byte-copy path runs,
//! and the exchange is the real `renameat2`.
//! These controls therefore
//!  run on the container gate's overlay too.
//! The controls that need the real ioctls use `btrfs_fixture`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const platform = new PlainPlatform({ removalAllowed: true });
//! const root = makeCheckout(parent, 'checkout');
//! ```

use crate::fs_clone_exchange::exchange_paths;
use crate::fs_kind::FilesystemKind;
use crate::fs_primitive_error::PrimitiveError;
use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_command::execute;
use crate::subvolume_error::Interrupt;
use crate::subvolume_platform::Platform;
use crate::test_support::{fixture, git, git_text, repository};
use std::collections::HashSet;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// What:
///  A platform on plain directories.
/// Why:
///   See the module description.
/// `kind` and `free` let a control make the filesystem look unsuitable;
/// `removal`
///  decides what `removal_allowed` answers and whether `destroy_subvolume` works;
/// `real_clone` forwards clones to the
///  kernel (true on a reflink filesystem only).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlainPlatform implements Platform { subvolumes = new Set<string>(); ... }
/// ```
pub(crate) struct PlainPlatform {
    /// Device and inode of every directory made by `create_subvolume`.
    pub subvolumes: Mutex<HashSet<(u64, u64)>>,
    /// What `filesystem_facts` reports as the kind.
    pub kind: FilesystemKind,
    /// What `filesystem_facts` reports as free bytes;
    /// `None` asks the real filesystem.
    pub free: Option<u64>,
    /// Whether subvolume destroy by the kernel call is allowed.
    pub removal: bool,
    /// Whether file clones go to the kernel.
    pub real_clone: bool,
    /// Whether `is_mount_point` answers true.
    pub mount_point: bool,
}

impl PlainPlatform {
    /// A platform that looks like btrfs with plenty of room,
    /// with removal allowed and clones refused.
    pub(crate) fn new() -> PlainPlatform {
        return PlainPlatform {
            subvolumes: Mutex::new(HashSet::new()),
            kind: FilesystemKind::Btrfs,
            free: Some(u64::MAX / 2),
            removal: true,
            real_clone: false,
            mount_point: false,
        };
    }

    /// Register a directory as a subvolume root.
    pub(crate) fn register(&self, path: &Path) {
        let metadata: std::fs::Metadata = std::fs::metadata(path).expect("registered path exists");
        self.subvolumes.lock().expect("the registry of subvolumes").insert((metadata.dev(), metadata.ino()));
    }
}

/// The five operations on plain directories.
impl Platform for PlainPlatform {
    fn conversion_environment(&self) -> crate::worktree_clean_content::ConversionEnvironment {
        return crate::worktree_clean_content::ConversionEnvironment {
            scope: crate::repo_read_config_scan::ScopeEnvironment { no_system: true, ..crate::repo_read_config_scan::ScopeEnvironment::default() },
            variables: Vec::new(),
            system_attributes: crate::worktree_clean_content::SystemAttributes::Disabled,
            alternate_attribute_source: None,
        };
    }

    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
        let (_, real_free) = crate::fs_kind::filesystem_facts(path)?;
        return Ok((self.kind, self.free.unwrap_or(real_free)));
    }

    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        let Ok(metadata) = std::fs::metadata(path) else { return Ok(false) };
        return Ok(self.subvolumes.lock().expect("the registry of subvolumes").contains(&(metadata.dev(), metadata.ino())));
    }

    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        std::fs::create_dir(path).map_err(|error: std::io::Error| {
            return PrimitiveError { operation: "BTRFS_IOC_SUBVOL_CREATE", path: path.to_path_buf(), errno: error.raw_os_error().unwrap_or(0) };
        })?;
        self.register(path);
        return Ok(());
    }

    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        if !self.removal {
            return Err(PrimitiveError { operation: "BTRFS_IOC_SNAP_DESTROY", path: path.to_path_buf(), errno: libc::EPERM });
        }
        let metadata: std::fs::Metadata = std::fs::metadata(path).map_err(|error: std::io::Error| {
            return PrimitiveError { operation: "BTRFS_IOC_SNAP_DESTROY", path: path.to_path_buf(), errno: error.raw_os_error().unwrap_or(0) };
        })?;
        std::fs::remove_dir_all(path).map_err(|error: std::io::Error| {
            return PrimitiveError { operation: "BTRFS_IOC_SNAP_DESTROY", path: path.to_path_buf(), errno: error.raw_os_error().unwrap_or(0) };
        })?;
        self.subvolumes.lock().expect("the registry of subvolumes").remove(&(metadata.dev(), metadata.ino()));
        return Ok(());
    }

    fn removal_allowed(&self, _path: &Path) -> bool {
        return self.removal;
    }

    fn is_mount_point(&self, _path: &Path) -> bool {
        return self.mount_point;
    }

    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        if self.real_clone {
            return crate::fs_clone_exchange::clone_file(source, destination, path);
        }
        return Err(PrimitiveError { operation: "FICLONE", path: path.to_path_buf(), errno: libc::EOPNOTSUPP });
    }

    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        return exchange_paths(first, second);
    }

    /// A control that uses this platform simulates a kill in the process,
    /// which loses no page cache,
    /// so the journal's flushes are left out.
    fn flushes_to_disk(&self) -> bool {
        return false;
    }
}

/// A platform that runs a hook before every clone and can make every hard link fail with `EXDEV`,
/// as links between two
///  subvolumes do.
pub(crate) struct Hooked {
    pub inner: PlainPlatform,
    /// Called with the number of earlier clone calls,
    /// before the clone is refused.
    pub on_clone: Box<dyn Fn(u32)>,
    /// How many clone calls were made.
    pub calls: std::cell::Cell<u32>,
    /// Whether every hard link fails with `EXDEV`.
    pub links_fail: bool,
}

/// Only the clone and the hard link are different.
impl Platform for Hooked {
    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
        return self.inner.filesystem_facts(path);
    }
    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        return self.inner.is_subvolume_root(path);
    }
    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.create_subvolume(path);
    }
    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.destroy_subvolume(path);
    }
    fn removal_allowed(&self, path: &Path) -> bool {
        return self.inner.removal_allowed(path);
    }
    fn is_mount_point(&self, path: &Path) -> bool {
        return self.inner.is_mount_point(path);
    }
    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        let calls: u32 = self.calls.get();
        self.calls.set(calls + 1);
        (self.on_clone)(calls);
        return self.inner.clone_file(source, destination, path);
    }
    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        return self.inner.exchange(first, second);
    }
    fn hard_link(&self, existing: &Path, new: &Path) -> std::io::Result<()> {
        if self.links_fail {
            return Err(std::io::Error::from_raw_os_error(libc::EXDEV));
        }
        return std::fs::hard_link(existing, new);
    }
    fn flushes_to_disk(&self) -> bool {
        return self.inner.flushes_to_disk();
    }
}

/// What:
///  A stand-in platform that answers that it flushes its writes,
/// and calls `on_question` before its first answer about the filesystem.
/// Why:
///   The switch that skips the flushes is the calling thread's,
/// so a control has to show that a call on a platform that
///  flushes puts the flushes back inside a call on a stand-in that skips them;
/// the question is where a control reads which policy
///  is in force inside that call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Flushing implements Platform {
///   constructor(readonly inner: PlainPlatform, readonly onQuestion: () => void) {}
///   flushesToDisk() { return true }
/// }
/// ```
pub(crate) struct Flushing {
    /// The stand-in whose answers every operation is delegated to.
    pub inner: PlainPlatform,
    /// Called before the first answer about the filesystem.
    pub on_question: Box<dyn Fn()>,
}

/// Every operation answers as the stand-in,
/// and the platform says it flushes.
impl Platform for Flushing {
    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
        (self.on_question)();
        return self.inner.filesystem_facts(path);
    }
    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        return self.inner.is_subvolume_root(path);
    }
    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.create_subvolume(path);
    }
    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.destroy_subvolume(path);
    }
    fn removal_allowed(&self, path: &Path) -> bool {
        return self.inner.removal_allowed(path);
    }
    fn is_mount_point(&self, path: &Path) -> bool {
        return self.inner.is_mount_point(path);
    }
    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.clone_file(source, destination, path);
    }
    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        return self.inner.exchange(first, second);
    }
    fn flushes_to_disk(&self) -> bool {
        return true;
    }
}



impl Hooked {
    /// A hooked platform with no hook and working links.
    pub(crate) fn new(hook: Box<dyn Fn(u32)>) -> Hooked {
        return Hooked { inner: PlainPlatform::new(), on_clone: hook, calls: std::cell::Cell::new(0), links_fail: false };
    }
}

/// What:
///  Set the modification time of every file and link of a tree,
/// except `.git`,
/// to a quarter of an hour ago.
/// Why:
///   Git trusts the stat fields of an entry only when the file was modified before the index was written.
/// Files made a
///  moment before `git add` are racy,
/// which the stat rewrite rightly leaves alone;
/// a real checkout's files are old.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await backdate(root);
/// ```
pub(crate) fn backdate(root: &Path) {
    let seconds: i64 = i64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("clock").as_secs()).expect("seconds") - 900;
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 0 }, libc::timespec { tv_sec: seconds, tv_nsec: 0 }];
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for item in std::fs::read_dir(&directory).expect("listing").flatten() {
            if item.file_name() == ".git" {
                continue;
            }
            let path: PathBuf = item.path();
            if item.file_type().expect("type").is_dir() {
                pending.push(path.clone());
            }
            let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
            // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
            let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
            assert_eq!(result, 0, "backdate {}", path.display());
        }
    }
}

/// What:
///  A real Git repository at `parent/name` with tracked files of the shapes the copy must carry:
/// nested directories,
///  an executable,
/// a symbolic link,
/// an empty file,
/// two names of one file,
/// a read-only directory,
/// and an ignored
///  directory;
/// committed with real Git so its index is real.
/// Why:
///   Real Git writes the index the stat rewrite must keep valid,
/// and checks the result afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = makeCheckout(parent, 'checkout');
/// ```
pub(crate) fn make_checkout(parent: &Path, name: &str) -> PathBuf {
    let root: PathBuf = repository(parent, name);
    std::fs::create_dir_all(root.join("src/deep/er")).expect("directories");
    std::fs::write(root.join("README.md"), b"readme\n").expect("readme");
    std::fs::write(root.join("src/main.rs"), b"fn main() {}\n").expect("main");
    std::fs::write(root.join("src/deep/er/leaf.txt"), vec![b'x'; 70_000]).expect("leaf");
    std::fs::write(root.join("empty"), b"").expect("empty");
    std::fs::write(root.join("script.sh"), b"an executable file that is not run\n").expect("script");
    std::fs::set_permissions(root.join("script.sh"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    std::os::unix::fs::symlink("src/main.rs", root.join("link")).expect("symlink");
    std::fs::write(root.join(".gitignore"), b"ignored/\n").expect("gitignore");
    // The ignored tree, with a second name for a tracked file, exists before `git add`: a new link changes the file's change
    // time, which Git compares.
    std::fs::create_dir_all(root.join("ignored/node_modules/pkg")).expect("ignored tree");
    std::fs::write(root.join("ignored/node_modules/pkg/index.js"), b"module.exports = 1;\n").expect("ignored file");
    std::fs::hard_link(root.join("src/main.rs"), root.join("ignored/main-link.rs")).expect("hard link");
    backdate(root.as_path());
    git(root.as_path(), &["add", "--all"]);
    git(root.as_path(), &["commit", "--quiet", "--message=files"]);
    return root;
}

/// The arguments of a plain migration run.
pub(crate) fn migrate_arguments() -> MigrateArguments {
    return MigrateArguments {
        mode: MigrateMode::Migrate,
        dry_run: false,
        nested_worktrees: true,
        refresh_indexes: true,
        discard_unreadable: false,
        discard_left_out: false,
        help: false,
    };
}

/// What:
///  Run the command to completion against a platform and collect its exit status and every progress line.
/// Why:
///   The controls assert on the status and the words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, lines } = runCommand(platform, args, root, interrupt);
/// ```
pub(crate) fn run_command(platform: &dyn Platform, arguments: &MigrateArguments, root: &Path, interrupt: &Interrupt) -> (i32, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let code: i32 = execute(platform, arguments, root, interrupt, &mut |text: &str| { lines.push(text.to_string()); });
    return (code, lines);
}

/// A checkout with one linked worktree inside it,
/// as the agents' layout has.
pub(crate) fn checkout_with_worktree(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "wt"]);
    // The agents' layout keeps its worktree directory ignored; without this the main checkout lists `wt/` as untracked.
    let exclude: PathBuf = root.join(".git/info/exclude");
    let mut text: String = std::fs::read_to_string(exclude.as_path()).unwrap_or_default();
    text.push_str("/wt/\n");
    std::fs::create_dir_all(root.join(".git/info")).expect("info directory");
    std::fs::write(exclude.as_path(), text).expect("exclude file");
    return (parent, root);
}


/// What:
///  A checkout with one linked worktree `wt`,
/// after the whole repository moved behind a symbolic link:
/// the checkout is renamed `moved` and a link is left at its old name,
/// so every registration still spells the old name while naming the same files.
/// Why:
///   A registration that spells the same file through a link is not stale,
/// which is what this repository looked like on 2026-11-08,
/// when `/var/home/user/Monochromatic` became a link to `/var/mnt/encrypted/repository/Monochromatic`.
/// The worktree's tracked files are old and its index is fresh,
/// so it is quiet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [parent, root, worktree] = checkoutMovedBehindALink('convert-link');
/// ```
pub(crate) fn checkout_moved_behind_a_link(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let moved: PathBuf = parent.join("moved");
    std::fs::rename(&root, &moved).expect("move the checkout aside");
    std::os::unix::fs::symlink(&moved, &root).expect("leave a link at the old name");
    let worktree: PathBuf = std::fs::canonicalize(moved.join("wt")).expect("canonical worktree");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (parent, moved, worktree);
}

/// What `git ls-files --debug` reports as the inode of a tracked path.
pub(crate) fn index_inode(root: &Path, path: &str) -> u64 {
    let text: String = git_text(root, &["ls-files", "--debug", "--", path]);
    let line: &str = text.lines().find(|candidate: &&str| { return candidate.contains("ino:"); }).expect("a stat line");
    return line.rsplit("ino:").next().expect("an inode").trim().parse::<u64>().expect("a number");
}

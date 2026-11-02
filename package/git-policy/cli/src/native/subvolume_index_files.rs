//! What:
//!  The linked worktrees of a main checkout, found from `.git/worktrees/*/gitdir`, and the stat rewrite applied to the
//!  index files of the checkout and of those worktrees on disk.
//! Why:
//!  The subvolume migration copies the main checkout and the linked worktrees inside it, so each of their indexes needs
//!  the rewrite of `subvolume_index.rs`. A file in `.git/worktrees/<name>/gitdir` holds the path of the worktree's
//!  `.git` file, so the worktree is its parent directory (the same reading the migration script used). An index is
//!  replaced through `index.lock` as Git replaces it, so a running Git command is never raced: the lock is taken
//!  before the index is read, and when it exists the index is left as copied and the report says so. The migration
//!  holds the locks itself while it completes the new checkout, and its rewrite then runs under them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const { adminDirectory, worktree } of linkedWorktrees(root)) refreshIndexFile(join(adminDirectory, 'index'), worktree);
//! ```

use super::subvolume_index::{IndexOutcome, StatFields, rewrite_index_stat};
use super::subvolume_lock::is_ours;
use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// What:
///  One linked worktree of the checkout.
/// Why:
///   The index lives in the administrative directory; the files it describes live in the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkedWorktree = { adminDirectory: string; worktree: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedWorktree {
    /// `<root>/.git/worktrees/<name>`.
    pub admin_directory: PathBuf,
    /// The directory the worktree's files are in.
    pub worktree: PathBuf,
}

/// What:
///  The linked worktrees of `root` whose directory is inside `root` and exists.
/// Why:
///   Only worktrees inside the checkout are copied with it. An administrative directory without a readable `gitdir`
///  file belongs to a worktree Git would prune, so it is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkedWorktreesInside(root: string): LinkedWorktree[];
/// ```
pub fn linked_worktrees_inside(root: &Path) -> Vec<LinkedWorktree> {
    let admin: PathBuf = root.join(".git").join("worktrees");
    let Ok(names) = std::fs::read_dir(&admin) else { return Vec::new() };
    let mut found: Vec<LinkedWorktree> = Vec::new();
    for name in names.flatten() {
        let Ok(recorded) = std::fs::read(name.path().join("gitdir")) else { continue };
        let text: String = String::from_utf8_lossy(recorded.as_slice()).trim().to_string();
        let Some(worktree) = Path::new(text.as_str()).parent() else { continue };
        if worktree.starts_with(root) && worktree != root && worktree.is_dir() {
            found.push(LinkedWorktree { admin_directory: name.path(), worktree: worktree.to_path_buf() });
        }
    }
    found.sort_by(|left: &LinkedWorktree, right: &LinkedWorktree| { return left.worktree.cmp(&right.worktree); });
    return found;
}

/// What:
///  What `lstat` of `tree/path` reports now, in index units, or nothing for a file that cannot be examined.
/// Why:
///   The rewrite leaves an entry as Git recorded it when its file is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFields(tree: string, path: Uint8Array): StatFields | undefined;
/// ```
fn stat_fields(tree: &Path, path: &[u8]) -> Option<StatFields> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(tree.join(OsStr::from_bytes(path))).ok()?;
    return Some(StatFields {
        ctime_seconds: metadata.ctime() as u32,
        ctime_nanos: metadata.ctime_nsec() as u32,
        mtime_seconds: metadata.mtime() as u32,
        mtime_nanos: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    });
}

/// What:
///  How the rewrite of one index file is protected from a Git command that writes the same index.
///  `&'a str` borrows the token of the migration whose lock is already held.
/// Why:
///   Git replaces an index through `index.lock`. A rewrite that runs by itself takes that lock, as Git does, before it
///  reads the index and holds it until the new index is in place. The migration already holds the lock of every index
///  while it completes the new checkout, so its rewrite uses the held lock instead of creating one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLock = { kind: 'acquire' } | { kind: 'held'; token: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexLock<'a> {
    /// Create `index.lock` for the duration of the rewrite and remove it after.
    Acquire,
    /// `index.lock` holds this token already; leave it in place for its owner to release.
    Held(&'a str),
}

/// What:
///  Rewrite one index file in place and report what happened in one line.
/// Why:
///   A missing index is not an error (a worktree never checked out has none); a held `index.lock` and every declined
///  shape are reported, not hidden, because the cost is a slow first `git status` there. The lock comes first, then
///  the read, so a Git command that installs a newer index in between is never overwritten with an older one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFile(indexPath: string, tree: string, lock: IndexLock): string;
/// ```
pub fn refresh_index_file_locked(index_path: &Path, tree: &Path, lock: IndexLock<'_>) -> String {
    let mut lock_name: std::ffi::OsString = index_path.as_os_str().to_os_string();
    lock_name.push(".lock");
    let lock_path: PathBuf = PathBuf::from(lock_name);
    let mut created: Option<std::fs::File> = None;
    match lock {
        IndexLock::Acquire => match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(opened) => created = Some(opened),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return String::from("skipped (index.lock exists: a Git command is running there; its first status will be slow)");
            }
            Err(error) => return format!("skipped (cannot create the lock: {error})"),
        },
        IndexLock::Held(token) => {
            if !is_ours(lock_path.as_path(), token) {
                return String::from("skipped (index.lock is held by another Git command; its first status will be slow)");
            }
        }
    }
    let outcome: String = rewrite_under_lock(index_path, tree, created.as_mut(), &lock_path);
    if created.is_some() {
        let _removed = std::fs::remove_file(&lock_path);
    }
    return outcome;
}

/// The read, rewrite and install of one index while its lock is held; `lock_file` is the lock when this call made it.
fn rewrite_under_lock(index_path: &Path, tree: &Path, lock_file: Option<&mut std::fs::File>, lock_path: &Path) -> String {
    let bytes: Vec<u8> = match std::fs::read(index_path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return String::from("no index file"),
        Err(error) => return format!("skipped (cannot read it: {error})"),
    };
    let outcome: IndexOutcome = rewrite_index_stat(bytes.as_slice(), &mut |path: &[u8]| { return stat_fields(tree, path); });
    let (rewritten, updated, total): (Vec<u8>, usize, usize) = match outcome {
        IndexOutcome::Rewritten { bytes: new_bytes, updated, total } => (new_bytes, updated, total),
        IndexOutcome::Skipped(reason) => return format!("skipped ({reason})"),
    };
    // With its own lock the new index is written into the lock file and the lock is renamed over the index, as Git does.
    // Under a held lock the lock stays, and a sibling file is renamed over the index instead.
    let installed: std::io::Result<()> = match lock_file {
        Some(file) => file.write_all(rewritten.as_slice()).and_then(|()| { return file.sync_all(); }).and_then(|()| { return std::fs::rename(lock_path, index_path); }),
        None => {
            let mut sibling_name: std::ffi::OsString = index_path.as_os_str().to_os_string();
            sibling_name.push(".cli-git-new");
            let sibling: PathBuf = PathBuf::from(sibling_name);
            let written: std::io::Result<()> = std::fs::write(&sibling, rewritten.as_slice())
                .and_then(|()| { return std::fs::File::open(&sibling); })
                .and_then(|file: std::fs::File| { return file.sync_all(); })
                .and_then(|()| { return std::fs::rename(&sibling, index_path); });
            if written.is_err() {
                let _removed = std::fs::remove_file(&sibling);
            }
            written
        }
    };
    if let Err(error) = installed {
        return format!("skipped (cannot install the new index: {error})");
    }
    return format!("{updated} of {total} entries refreshed");
}

/// What:
///  Rewrite one index file in place under a lock the rewrite takes itself; see `refresh_index_file_locked`.
/// Why:
///   The common case: nothing holds the lock yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFile(indexPath: string, tree: string): string;
/// ```
pub fn refresh_index_file(index_path: &Path, tree: &Path) -> String {
    return refresh_index_file_locked(index_path, tree, IndexLock::Acquire);
}

/// What:
///  Rewrite the index of the checkout and of every linked worktree inside it, reporting each through `say`.
///  `&mut dyn FnMut(&str)` is a borrowed callback that receives one report line at a time.
/// Why:
///   The first `git status` of each of them is otherwise a full re-hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshAllIndexes(root: string, lock: IndexLock, say: (line: string) => void): void;
/// ```
pub fn refresh_all_indexes(root: &Path, lock: IndexLock<'_>, say: &mut dyn FnMut(&str)) -> usize {
    let main_index: PathBuf = root.join(".git").join("index");
    say(format!("index {}: {}", main_index.display(), refresh_index_file_locked(main_index.as_path(), root, lock)).as_str());
    let mut count: usize = 1;
    for linked in linked_worktrees_inside(root) {
        let index: PathBuf = linked.admin_directory.join("index");
        say(format!("index {}: {}", index.display(), refresh_index_file_locked(index.as_path(), linked.worktree.as_path(), lock)).as_str());
        count += 1;
    }
    return count;
}

/// Index file controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_files_tests.rs"]
mod tests;

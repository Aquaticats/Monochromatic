//! What:
//!  The linked worktrees of a main checkout,
//! found from `.git/worktrees/*/gitdir`,
//! and the stat rewrite applied to the
//!  index files of the checkout and of those worktrees on disk.
//! Why:
//!  The subvolume migration copies the main checkout and the linked worktrees inside it,
//! so each of their indexes needs
//!  the rewrite of `subvolume_index.rs`.
//! A file in `.git/worktrees/<name>/gitdir` holds the path of the worktree's
//!  `.git` file,
//! so the worktree is its parent directory (the same reading the migration script used).
//! An index is
//!  replaced through `index.lock` as Git replaces it,
//! so a running Git command is never raced:
//! the lock is taken
//!  before the index is read,
//! and when it exists the index is left as copied and the report says so.
//! The migration
//!  holds the locks itself while it completes the new checkout,
//! and its rewrite then runs under them.
//!
//! Why the lock stays:
//!  `index.lock` is Git's protocol for replacing an index;
//! the rewrite reads the index,
//! computes new stat fields and renames
//!  a file over it,
//! which is lost if a Git writer replaces the index in between,
//! and only Git's own lock excludes Git.
//! The
//!  rewrite takes it with `O_EXCL`,
//! never waits (a lock that exists skips the index and says so),
//! holds it for one index and
//!  only from the read to the rename,
//! and gives it back as Git does.
//! Concurrency among this wrapper's own commands never
//!  reaches this file:
//! `single_owner_claim.rs` gives one command a checkout or a worktree without a lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const { adminDirectory, worktree } of linkedWorktrees(root)) refreshIndexFile(join(adminDirectory, 'index'), worktree);
//! ```

pub use crate::subvolume_index_install_into_new_worktree::{InstallSeams, install_index_in, install_index_in_with};
use crate::plain_file::{SMALL_FILE_LIMIT, read_plain_file};
use crate::subvolume_index::{IndexOutcome, RewriteContext, StatFields, StatLookup, StatTrust, StoredStat, rewrite_index_stat_with};
use crate::subvolume_file_write_steps::write_all_then_sync;
use crate::subvolume_lock::is_ours;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// What:
///  The linked worktrees of a main checkout and the registrations that are not one of them.
/// Why:
///   They are read together from `.git/worktrees`;
/// see `subvolume_registered_worktrees.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './subvolume_registered_worktrees';
/// ```
pub use crate::subvolume_registered_worktrees::{LeftOutWorktree, LinkedWorktree, LinkedWorktreeScan, linked_worktrees_inside, scan_linked_worktrees};

/// The call the refresh makes at its seam when no control wants one:
/// none.
fn do_nothing() {}

/// The largest index file this reads.
///  A plain file is read to its end,
/// so the bound is what keeps a file that is not an index from being read whole into memory;
///  the largest index this repository holds is 3,097,973 bytes (measured 2026-11-09,
/// one of its linked worktrees' `.git/worktrees/<name>/index`),
/// and a gigabyte is that with room for a repository several hundred times larger.
///  The merge pass reads an index up to the same bound (`INDEX_LIMIT_BYTES`,
/// `merge_second_pass_worktree.rs`).
const INDEX_FILE_LIMIT: u64 = 1 << 30;

/// What:
///  What `lstat` of `tree/path` reports now,
/// in index units,
/// or nothing for a file that cannot be examined.
/// Why:
///   The rewrite leaves an entry as Git recorded it when its file is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFields(tree: string, path: Uint8Array): StatFields | undefined;
/// ```
pub(crate) fn stat_fields(tree: &Path, path: &[u8]) -> Option<StatFields> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(tree.join(OsStr::from_bytes(path))).ok()?;
    return Some(fields_of(&metadata));
}

/// What:
///  The stat fields of metadata in hand,
/// in index units.
/// Why:
///   A caller that holds the object it made records the fields of that object,
/// not of whatever its path names later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fieldsOf(stats: Stats): StatFields;
/// ```
pub(crate) fn fields_of(metadata: &std::fs::Metadata) -> StatFields {
    return StatFields {
        ctime_seconds: metadata.ctime() as u32,
        ctime_nanos: metadata.ctime_nsec() as u32,
        mtime_seconds: metadata.mtime() as u32,
        mtime_nanos: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    };
}

/// What:
///  How the rewrite of one index file is protected from a Git command that writes the same index.
///  `Arc<str>` shares the token of the migration whose lock is already held,
///  because a rewrite that runs as a job of the blocking pool must own what it reads.
/// Why:
///   Git replaces an index through `index.lock`.
/// A rewrite that runs by itself takes that lock,
/// as Git does,
/// before it
///  reads the index and holds it until the new index is in place.
/// The migration already holds the lock of every index
///  while it completes the new checkout,
/// so its rewrite uses the held lock instead of creating one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLock = { kind: 'acquire' } | { kind: 'held'; token: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexLock {
    /// Create `index.lock` for the duration of the rewrite and remove it after.
    Acquire,
    /// `index.lock` holds this token already;
    /// leave it in place for its owner to release.
    Held(std::sync::Arc<str>),
}

/// What:
///  What a rewrite of one index file is told beyond the paths.
///  `&'a [PathBuf]` borrows the list of trees.
/// Why:
///   The lock decides how the index is protected,
/// the references decide which entries Git recorded for the files the
///  copy was made from,
/// and the hash length (read from the repository's configuration) decides how an index with an
///  all-zero checksum is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefreshOptions = { lock: IndexLock; references: string[]; hashLength?: number };
/// ```
#[derive(Clone, Debug)]
pub struct RefreshOptions<'a> {
    /// How the index is locked while it is rewritten.
    pub lock: IndexLock,
    /// Trees whose files the index may describe (the tree the copy was made from).
    /// An entry is refreshed only if the
    ///  file in one of them still has the stat fields the entry records;
    /// empty means every entry may be refreshed.
    pub references: &'a [PathBuf],
    /// The repository's hash length in bytes,
    /// when known.
    pub hash_length: Option<usize>,
}

/// Whether a file's present stat equals what an entry recorded (the device is not compared:
/// a subvolume has its own).
pub(crate) fn matches_stored(now: &StatFields, stored: &StoredStat) -> bool {
    return now.size == stored.size
        && now.ino == stored.ino
        && now.uid == stored.uid
        && now.gid == stored.gid
        && now.ctime_seconds == stored.ctime_seconds
        && (stored.ctime_nanos == 0 || now.ctime_nanos == stored.ctime_nanos)
        && now.mtime_seconds == stored.mtime_seconds
        && (stored.mtime_nanos == 0 || now.mtime_nanos == stored.mtime_nanos);
}

/// What:
///  Rewrite one index file in place and report what happened in one line.
/// Why:
///   A missing index is not an error (a worktree never checked out has none);
/// a held `index.lock` and every declined
///  shape are reported,
/// not hidden,
/// because the cost is a slow first `git status` there.
/// The lock comes first,
/// then
///  the read,
/// so a Git command that installs a newer index in between is never overwritten with an older one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFileWith(indexPath: string, tree: string, options: RefreshOptions): string;
/// ```
pub fn refresh_index_file_with(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>) -> String {
    return refresh_index_file_seamed(index_path, tree, options, &mut do_nothing);
}

/// `refresh_index_file_with` with a call at the moment right after the rewrite,
/// before the lock is looked at again,
/// where a
///  control may let another command take the lock name.
pub fn refresh_index_file_seamed(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>, after_rewrite: &mut dyn FnMut()) -> String {
    return refresh_index_file_report_seamed(index_path, tree, options, after_rewrite).line;
}

/// How a refresh ended,
/// apart from the words that describe it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshStatus {
    /// The new index was installed.
    Refreshed,
    /// There is no index file,
    /// which is nothing to refresh and no failure.
    NoIndex,
    /// The index was left as it was,
    /// for the reason in the line.
    Skipped,
}

/// A refresh's line for the person and its status for the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshReport {
    /// The one-line report.
    pub line: String,
    /// How it ended.
    pub status: RefreshStatus,
}

/// `refresh_index_file_with` that says how it ended,
/// so a caller that must not go on after a failed refresh can stop.
pub fn refresh_index_file_report(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>) -> RefreshReport {
    return refresh_index_file_report_seamed(index_path, tree, options, &mut do_nothing);
}

/// `refresh_index_file_report` with a call at the moment right after the rewrite.
pub fn refresh_index_file_report_seamed(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>, after_rewrite: &mut dyn FnMut()) -> RefreshReport {
    let mut lock_name: std::ffi::OsString = index_path.as_os_str().to_os_string();
    lock_name.push(".lock");
    let lock_path: PathBuf = PathBuf::from(lock_name);
    let mut created: Option<std::fs::File> = None;
    match &options.lock {
        IndexLock::Acquire => match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(opened) => created = Some(opened),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return RefreshReport { line: String::from("skipped (index.lock exists: a Git command is running there; its first status will be slow)"), status: RefreshStatus::Skipped };
            }
            Err(error) => return RefreshReport { line: format!("skipped (cannot create the lock: {error})"), status: RefreshStatus::Skipped },
        },
        IndexLock::Held(token) => {
            if !is_ours(lock_path.as_path(), token.as_ref()) {
                return RefreshReport { line: String::from("skipped (index.lock is held by another Git command; its first status will be slow)"), status: RefreshStatus::Skipped };
            }
        }
    }
    let (outcome, status): (String, RefreshStatus) = rewrite_under_lock(index_path, tree, options, created.as_mut(), &lock_path);
    after_rewrite();
    // Once the lock was renamed over the index the name is any Git command's to take; it is never touched again. Before that,
    // the lock is removed only if the name still holds the file this call made.
    if let (Some(file), true) = (created.as_ref(), status != RefreshStatus::Refreshed) {
        remove_if_same_file(lock_path.as_path(), file);
    }
    return RefreshReport { line: outcome, status };
}

/// Remove the file at `path` if it is the very file `held` is open on (same device and inode).
fn remove_if_same_file(path: &Path, held: &std::fs::File) {
    let same: bool = match (std::fs::symlink_metadata(path), held.metadata()) {
        (Ok(named), Ok(own)) => named.dev() == own.dev() && named.ino() == own.ino() && named.file_type().is_file(),
        _ => false,
    };
    if same {
        let _removed = std::fs::remove_file(path);
    }
}

/// The read,
/// rewrite and install of one index while its lock is held;
/// `lock_file` is the lock when this call made it.
/// The flag
///  says whether the new index was published,
/// after which the lock name is not this call's any more.
fn rewrite_under_lock(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>, lock_file: Option<&mut std::fs::File>, lock_path: &Path) -> (String, RefreshStatus) {
    let index_mtime: Option<(u32, u32)> = match std::fs::metadata(index_path) {
        Ok(metadata) => Some((metadata.mtime() as u32, metadata.mtime_nsec() as u32)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (String::from("no index file"), RefreshStatus::NoIndex),
        Err(error) => return (format!("skipped (cannot examine it: {error})"), RefreshStatus::Skipped),
    };
    let bytes: Vec<u8> = match read_plain_file(index_path, INDEX_FILE_LIMIT) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (String::from("no index file"), RefreshStatus::NoIndex),
        Err(error) => return (format!("skipped (cannot read it: {error})"), RefreshStatus::Skipped),
    };
    return rewrite_and_install(bytes.as_slice(), index_mtime, [index_path, tree, lock_path], options, lock_file);
}

/// What:
///  The stat fields of a path as the tree being rewritten reports them now.
/// Why:
///   The rewrite asks for each entry's path;
/// the tree it describes is the only state the answer needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StatOfTree implements StatLookup { constructor(readonly tree: string) {} statNow(path) { return statFields(this.tree, path); } }
/// ```
struct StatOfTree<'a> {
    /// The tree the index describes.
    tree: &'a Path,
}

impl StatLookup for StatOfTree<'_> {
    fn stat_now(&mut self, path: &[u8]) -> Option<StatFields> {
        return stat_fields(self.tree, path);
    }
}

/// What:
///  Whether an entry still agrees with the trees the checkout was copied from.
/// Why:
///   The rewrite trusts an entry's stored fields only for a file that is still the one the copy was made from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AgreeWithReferences implements StatTrust { isTrusted(path, stored) { return agreesWithAReference(this.references, path, stored); } }
/// ```
struct AgreeWithReferences<'a> {
    /// The trees the checkout was copied from,
    /// one per reference.
    references: &'a [PathBuf],
}

impl StatTrust for AgreeWithReferences<'_> {
    fn is_trusted(&mut self, path: &[u8], stored: &StoredStat) -> bool {
        return agrees_with_a_reference(self.references, path, stored);
    }
}

/// The rewrite of the stat fields of `bytes` and the install of the result over `index_path`,
/// with `paths` naming the index,
/// the tree it
///  describes and the lock file.
fn rewrite_and_install(bytes: &[u8], index_mtime: Option<(u32, u32)>, paths: [&Path; 3], options: &RefreshOptions<'_>, lock_file: Option<&mut std::fs::File>) -> (String, RefreshStatus) {
    let [index_path, tree, lock_path]: [&Path; 3] = paths;
    let context: RewriteContext = RewriteContext { hash_length: options.hash_length, index_mtime };
    let outcome: IndexOutcome = rewrite_index_stat_with(bytes, &context, &mut StatOfTree { tree }, &mut AgreeWithReferences { references: options.references });
    let (rewritten, updated, total): (Vec<u8>, usize, usize) = match outcome {
        IndexOutcome::Rewritten { bytes: new_bytes, updated, total } => (new_bytes, updated, total),
        IndexOutcome::Skipped(reason) => return (format!("skipped ({reason})"), RefreshStatus::Skipped),
    };
    // With its own lock the new index is written into the lock file and the lock is renamed over the index, as Git does.
    // Under a held lock the lock stays, and a sibling file is renamed over the index instead.
    let installed: std::io::Result<()> = match lock_file {
        Some(file) => write_sync_rename(file, rewritten.as_slice(), lock_path, index_path),
        None => stage_and_replace(index_path, rewritten.as_slice()),
    };
    if let Err(error) = installed {
        return (format!("skipped (cannot install the new index: {error})"), RefreshStatus::Skipped);
    }
    return (format!("{updated} of {total} entries refreshed"), RefreshStatus::Refreshed);
}

/// Whether some reference tree has the file `path` as the entry `stored` records;
/// with no reference tree,
/// every entry agrees.
fn agrees_with_a_reference(references: &[PathBuf], path: &[u8], stored: &StoredStat) -> bool {
    if references.is_empty() {
        return true;
    }
    for reference in references {
        if let Some(now) = stat_fields(reference.as_path(), path)
            && matches_stored(&now, stored)
        {
            return true;
        }
    }
    return false;
}

/// Write `bytes` into `file`,
/// flush it to disk,
/// then rename `from` over `to`;
/// each step runs only if the one before it worked.
fn write_sync_rename(file: &mut std::fs::File, bytes: &[u8], from: &Path, to: &Path) -> std::io::Result<()> {
    write_all_then_sync(file, bytes)?;
    return std::fs::rename(from, to);
}

/// What:
///  Write `bytes` to a staging file beside `index_path`,
/// created exclusively,
/// and rename it over the index.
/// Why:
///   Under a held lock no Git command writes the index,
/// but the staging name is still checked:
/// it is created with
///  `O_EXCL`,
/// which fails on an existing file or link instead of following or truncating it,
/// written and synced through its
///  own descriptor,
/// and renamed only while the name still holds that file.
/// A staging file that cannot be proven the call's
///  is left where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stageAndReplace(indexPath, bytes);
/// ```
fn stage_and_replace(index_path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut sibling_name: std::ffi::OsString = index_path.as_os_str().to_os_string();
    sibling_name.push(".cli-git-new");
    let sibling: PathBuf = PathBuf::from(sibling_name);
    let mut staged: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).open(&sibling)?;
    let written: std::io::Result<()> = write_sync_and_rename_if_same(&mut staged, bytes, sibling.as_path(), index_path);
    if written.is_err() {
        remove_if_same_file(sibling.as_path(), &staged);
    }
    return written;
}

/// Write `bytes` into `staged`,
/// flush it,
/// and rename `sibling` over `index_path` only while `sibling` still names `staged`.
fn write_sync_and_rename_if_same(staged: &mut std::fs::File, bytes: &[u8], sibling: &Path, index_path: &Path) -> std::io::Result<()> {
    write_all_then_sync(staged, bytes)?;
    let named = std::fs::symlink_metadata(sibling)?;
    let own = staged.metadata()?;
    if named.dev() != own.dev() || named.ino() != own.ino() {
        return Err(std::io::Error::other("the staging file was replaced while the index was written"));
    }
    return std::fs::rename(sibling, index_path);
}

/// What:
///  Rewrite one index file in place under a lock,
/// trusting every entry;
/// see `refresh_index_file_with`.
/// Why:
///   The shape the controls use when nothing else is known about the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFileLocked(indexPath: string, tree: string, lock: IndexLock): string;
/// ```
pub fn refresh_index_file_locked(index_path: &Path, tree: &Path, lock: IndexLock) -> String {
    return refresh_index_file_with(index_path, tree, &RefreshOptions { lock, references: &[], hash_length: None });
}

/// What:
///  Rewrite one index file in place under a lock the rewrite takes itself;
/// see `refresh_index_file_locked`.
/// Why:
///   The common case:
/// nothing holds the lock yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFile(indexPath: string, tree: string): string;
/// ```
pub fn refresh_index_file(index_path: &Path, tree: &Path) -> String {
    return refresh_index_file_locked(index_path, tree, IndexLock::Acquire);
}

/// What:
///  The repository's hash length in bytes from its configuration file:
/// 32 when `extensions.objectformat` is `sha256`,
///  20 when the file has no such setting,
/// nothing when the file cannot be read.
/// Why:
///   An index does not say which hash it uses;
/// the configuration does.
/// The reading is the plain `[section]` and
///  `key = value` grammar,
/// case-insensitive for section and key names,
/// without includes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoryHashLength(gitDirectory: string): 20 | 32 | undefined;
/// ```
pub fn repository_hash_length(git_directory: &Path) -> Option<usize> {
    let text: String = String::from_utf8_lossy(read_plain_file(&git_directory.join("config"), SMALL_FILE_LIMIT).ok()?.as_slice()).into_owned();
    let mut in_extensions: bool = false;
    for raw in text.lines() {
        let line: &str = raw.trim();
        if let Some(header) = line.strip_prefix('[') {
            in_extensions = match header.split(']').next() {
                Some(name) => name.trim().eq_ignore_ascii_case("extensions"),
                None => false,
            };
            continue;
        }
        if !in_extensions {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        if key.trim().eq_ignore_ascii_case("objectformat") && value.trim().eq_ignore_ascii_case("sha256") {
            return Some(32);
        }
    }
    return Some(20);
}

/// Index file controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_files_tests.rs"]
mod tests;

/// The controls for indexes rewritten side by side stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_parallel_tests.rs"]
mod parallel_tests;

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
//! Why the lock stays:
//!  `index.lock` is Git's protocol for replacing an index; the rewrite reads the index, computes new stat fields and renames
//!  a file over it, which is lost if a Git writer replaces the index in between, and only Git's own lock excludes Git. The
//!  rewrite takes it with `O_EXCL`, never waits (a lock that exists skips the index and says so), holds it for one index and
//!  only from the read to the rename, and gives it back as Git does. Concurrency among this wrapper's own commands never
//!  reaches this file: `single_owner_claim.rs` gives one command a checkout or a worktree without a lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const { adminDirectory, worktree } of linkedWorktrees(root)) refreshIndexFile(join(adminDirectory, 'index'), worktree);
//! ```

use super::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use super::parallel_runner::{Event, OrderedRun, run_ordered};
use super::subvolume_index::{IndexOutcome, RewriteContext, StatFields, StoredStat, rewrite_index_stat_with};
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
        // The recorded path is raw bytes: a directory named with bytes that are not UTF-8 is a valid path on Linux.
        let Some(worktree) = Path::new(OsStr::from_bytes(recorded.trim_ascii())).parent() else { continue };
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
pub(crate) fn stat_fields(tree: &Path, path: &[u8]) -> Option<StatFields> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(tree.join(OsStr::from_bytes(path))).ok()?;
    return Some(fields_of(&metadata));
}

/// What:
///  The stat fields of metadata in hand, in index units.
/// Why:
///   A caller that holds the object it made records the fields of that object, not of whatever its path names later.
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
///  What a rewrite of one index file is told beyond the paths.
///  `&'a [PathBuf]` borrows the list of trees.
/// Why:
///   The lock decides how the index is protected, the references decide which entries Git recorded for the files the
///  copy was made from, and the hash length (read from the repository's configuration) decides how an index with an
///  all-zero checksum is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefreshOptions = { lock: IndexLock; references: string[]; hashLength?: number };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RefreshOptions<'a> {
    /// How the index is locked while it is rewritten.
    pub lock: IndexLock<'a>,
    /// Trees whose files the index may describe (the tree the copy was made from). An entry is refreshed only if the
    ///  file in one of them still has the stat fields the entry records; empty means every entry may be refreshed.
    pub references: &'a [PathBuf],
    /// The repository's hash length in bytes, when known.
    pub hash_length: Option<usize>,
}

/// Whether a file's present stat equals what an entry recorded (the device is not compared: a subvolume has its own).
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
///   A missing index is not an error (a worktree never checked out has none); a held `index.lock` and every declined
///  shape are reported, not hidden, because the cost is a slow first `git status` there. The lock comes first, then
///  the read, so a Git command that installs a newer index in between is never overwritten with an older one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFileWith(indexPath: string, tree: string, options: RefreshOptions): string;
/// ```
pub fn refresh_index_file_with(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>) -> String {
    return refresh_index_file_seamed(index_path, tree, options, &mut || {});
}

/// `refresh_index_file_with` with a call at the moment right after the rewrite, before the lock is looked at again, where a
///  control may let another command take the lock name.
pub fn refresh_index_file_seamed(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>, after_rewrite: &mut dyn FnMut()) -> String {
    return refresh_index_file_report_seamed(index_path, tree, options, after_rewrite).line;
}

/// How a refresh ended, apart from the words that describe it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshStatus {
    /// The new index was installed.
    Refreshed,
    /// There is no index file, which is nothing to refresh and no failure.
    NoIndex,
    /// The index was left as it was, for the reason in the line.
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

/// `refresh_index_file_with` that says how it ended, so a caller that must not go on after a failed refresh can stop.
pub fn refresh_index_file_report(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>) -> RefreshReport {
    return refresh_index_file_report_seamed(index_path, tree, options, &mut || {});
}

/// `refresh_index_file_report` with a call at the moment right after the rewrite.
pub fn refresh_index_file_report_seamed(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>, after_rewrite: &mut dyn FnMut()) -> RefreshReport {
    let mut lock_name: std::ffi::OsString = index_path.as_os_str().to_os_string();
    lock_name.push(".lock");
    let lock_path: PathBuf = PathBuf::from(lock_name);
    let mut created: Option<std::fs::File> = None;
    match options.lock {
        IndexLock::Acquire => match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(opened) => created = Some(opened),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return RefreshReport { line: String::from("skipped (index.lock exists: a Git command is running there; its first status will be slow)"), status: RefreshStatus::Skipped };
            }
            Err(error) => return RefreshReport { line: format!("skipped (cannot create the lock: {error})"), status: RefreshStatus::Skipped },
        },
        IndexLock::Held(token) => {
            if !is_ours(lock_path.as_path(), token) {
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

/// The read, rewrite and install of one index while its lock is held; `lock_file` is the lock when this call made it. The flag
///  says whether the new index was published, after which the lock name is not this call's any more.
fn rewrite_under_lock(index_path: &Path, tree: &Path, options: &RefreshOptions<'_>, lock_file: Option<&mut std::fs::File>, lock_path: &Path) -> (String, RefreshStatus) {
    let index_mtime: Option<(u32, u32)> = match std::fs::metadata(index_path) {
        Ok(metadata) => Some((metadata.mtime() as u32, metadata.mtime_nsec() as u32)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (String::from("no index file"), RefreshStatus::NoIndex),
        Err(error) => return (format!("skipped (cannot examine it: {error})"), RefreshStatus::Skipped),
    };
    let bytes: Vec<u8> = match std::fs::read(index_path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (String::from("no index file"), RefreshStatus::NoIndex),
        Err(error) => return (format!("skipped (cannot read it: {error})"), RefreshStatus::Skipped),
    };
    return rewrite_and_install(bytes.as_slice(), index_mtime, [index_path, tree, lock_path], options, lock_file);
}

/// The rewrite of the stat fields of `bytes` and the install of the result over `index_path`, with `paths` naming the index, the tree it
///  describes and the lock file.
fn rewrite_and_install(bytes: &[u8], index_mtime: Option<(u32, u32)>, paths: [&Path; 3], options: &RefreshOptions<'_>, lock_file: Option<&mut std::fs::File>) -> (String, RefreshStatus) {
    let [index_path, tree, lock_path]: [&Path; 3] = paths;
    let context: RewriteContext = RewriteContext { hash_length: options.hash_length, index_mtime };
    let outcome: IndexOutcome = rewrite_index_stat_with(
        bytes,
        &context,
        &mut |path: &[u8]| { return stat_fields(tree, path); },
        &mut |path: &[u8], stored: &StoredStat| {
            if options.references.is_empty() {
                return true;
            }
            return options.references.iter().any(|reference: &PathBuf| {
                return stat_fields(reference.as_path(), path).is_some_and(|now: StatFields| { return matches_stored(&now, stored); });
            });
        },
    );
    let (rewritten, updated, total): (Vec<u8>, usize, usize) = match outcome {
        IndexOutcome::Rewritten { bytes: new_bytes, updated, total } => (new_bytes, updated, total),
        IndexOutcome::Skipped(reason) => return (format!("skipped ({reason})"), RefreshStatus::Skipped),
    };
    // With its own lock the new index is written into the lock file and the lock is renamed over the index, as Git does.
    // Under a held lock the lock stays, and a sibling file is renamed over the index instead.
    let installed: std::io::Result<()> = match lock_file {
        Some(file) => file.write_all(rewritten.as_slice()).and_then(|()| { return file.sync_all(); }).and_then(|()| { return std::fs::rename(lock_path, index_path); }),
        None => stage_and_replace(index_path, rewritten.as_slice()),
    };
    if let Err(error) = installed {
        return (format!("skipped (cannot install the new index: {error})"), RefreshStatus::Skipped);
    }
    return (format!("{updated} of {total} entries refreshed"), RefreshStatus::Refreshed);
}

/// What:
///  Write `bytes` to a staging file beside `index_path`, created exclusively, and rename it over the index.
/// Why:
///   Under a held lock no Git command writes the index, but the staging name is still checked: it is created with
///  `O_EXCL`, which fails on an existing file or link instead of following or truncating it, written and synced through its
///  own descriptor, and renamed only while the name still holds that file. A staging file that cannot be proven the call's
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
    let written: std::io::Result<()> = staged.write_all(bytes).and_then(|()| { return staged.sync_all(); }).and_then(|()| {
        let named = std::fs::symlink_metadata(&sibling)?;
        let own = staged.metadata()?;
        if named.dev() != own.dev() || named.ino() != own.ino() {
            return Err(std::io::Error::other("the staging file was replaced while the index was written"));
        }
        return std::fs::rename(&sibling, index_path);
    });
    if written.is_err() {
        remove_if_same_file(sibling.as_path(), &staged);
    }
    return written;
}

/// What:
///  Install `seed` as the index of the new worktree whose administrative directory is the open handle `admin`, with the stat
///  fields of `stats` for the entries it knows, and nothing for the others.
/// Why:
///   The index of a new worktree must never be written in place and never replace one that exists: `index.lock` is created
///  first (exclusively), the absence of an index is checked under it, the finished index is written into the lock, and
///  that open file is linked under the name of the index without replacing anything; the lock name is released afterwards
///  only if it still holds that file, because once it is released it is another Git process's to take. The stat fields come from the objects the caller made, so an entry whose file was
///  replaced afterwards no longer matches its inode and Git hashes it. Everything is done through the handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installIndexIn(admin, seed, stats, hashLength); // throws a reason
/// ```
pub fn install_index_in(admin: &std::fs::File, seed: &[u8], stats: &mut dyn FnMut(&[u8]) -> Option<StatFields>, hash_length: usize) -> Result<String, String> {
    return install_index_in_with(admin, seed, stats, hash_length, &mut InstallSeams { before_publish: &mut || {}, after_publish: &mut || {} });
}

/// The moments of an index install at which the controls move the tree.
pub struct InstallSeams<'a> {
    /// Called after the index was written into the lock and before it is linked under its name.
    pub before_publish: &'a mut dyn FnMut(),
    /// Called right after the index got its name and before the lock is released.
    pub after_publish: &'a mut dyn FnMut(),
}

/// `install_index_in` with the moments at which a control may move the tree.
pub fn install_index_in_with(admin: &std::fs::File, seed: &[u8], stats: &mut dyn FnMut(&[u8]) -> Option<StatFields>, hash_length: usize, seams: &mut InstallSeams<'_>) -> Result<String, String> {
    use super::fs_beneath::{create_file_at, entry_metadata_at, identity_of, link_open_file_at};
    let lock: &OsStr = OsStr::new("index.lock");
    let index: &OsStr = OsStr::new("index");
    let mut created: std::fs::File = match create_file_at(admin, lock, 0o600) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(String::from("index.lock exists: a Git command is running there")),
        Err(error) => return Err(format!("the lock cannot be created ({error})")),
    };
    let ours = identity_of(&created).map_err(|error| { return format!("the lock cannot be examined ({error})"); })?;
    // Before the publication the lock is ours to remove, but only if the name still holds the file made here.
    let give_up = |reason: String| -> Result<String, String> {
        let _kept = super::fs_beneath::remove_if_identical(admin, lock, ours);
        return Err(reason);
    };
    match entry_metadata_at(admin, index) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return give_up(format!("the index cannot be examined ({error})")),
        Ok(_) => return give_up(String::from("an index appeared in the new worktree while it was made")),
    }
    let now: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let context: RewriteContext = RewriteContext { hash_length: Some(hash_length), index_mtime: Some((now.as_secs() as u32, now.subsec_nanos())) };
    let (rewritten, updated, total): (Vec<u8>, usize, usize) = match rewrite_index_stat_with(seed, &context, stats, &mut |_path: &[u8], _stored: &StoredStat| { return true; }) {
        IndexOutcome::Rewritten { bytes, updated, total } => (bytes, updated, total),
        IndexOutcome::Skipped(reason) => return give_up(format!("the index cannot be refreshed ({reason})")),
    };
    if let Err(error) = created.write_all(rewritten.as_slice()).and_then(|()| { return created.sync_all(); }) {
        return give_up(format!("the new index cannot be written ({error})"));
    }
    (seams.before_publish)();
    // The open file is linked under the name of the index, which fails if the name exists: what is published is the file
    // written here, whatever the lock name refers to by now. The lock stays until the index has its name.
    if let Err(error) = link_open_file_at(&created, admin, index) {
        return give_up(format!("the new index cannot be put in place ({error})"));
    }
    (seams.after_publish)();
    // The lock name and the index are the same file now; the lock is released only if the name still holds it, so the lock
    // of the next Git command is never removed.
    let _released = super::fs_beneath::remove_if_identical(admin, lock, ours);
    return Ok(format!("{updated} of {total} entries refreshed"));
}

/// What:
///  Rewrite one index file in place under a lock, trusting every entry; see `refresh_index_file_with`.
/// Why:
///   The shape the controls use when nothing else is known about the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFileLocked(indexPath: string, tree: string, lock: IndexLock): string;
/// ```
pub fn refresh_index_file_locked(index_path: &Path, tree: &Path, lock: IndexLock<'_>) -> String {
    return refresh_index_file_with(index_path, tree, &RefreshOptions { lock, references: &[], hash_length: None });
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
///  The repository's hash length in bytes from its configuration file: 32 when `extensions.objectformat` is `sha256`,
///  20 when the file has no such setting, nothing when the file cannot be read.
/// Why:
///   An index does not say which hash it uses; the configuration does. The reading is the plain `[section]` and
///  `key = value` grammar, case-insensitive for section and key names, without includes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoryHashLength(gitDirectory: string): 20 | 32 | undefined;
/// ```
pub fn repository_hash_length(git_directory: &Path) -> Option<usize> {
    let text: String = String::from_utf8_lossy(std::fs::read(git_directory.join("config")).ok()?.as_slice()).into_owned();
    let mut in_extensions: bool = false;
    for raw in text.lines() {
        let line: &str = raw.trim();
        if let Some(header) = line.strip_prefix('[') {
            in_extensions = header.split(']').next().is_some_and(|name: &str| { return name.trim().eq_ignore_ascii_case("extensions"); });
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

/// The number of copies of an index file one rewrite holds at once: the bytes read, the parsed entries, and the bytes written.
///  The budget multiplies the largest index file by it to estimate what one rewrite holds.
///  Measured with `refresh_index_file` on throwaway repositories of this repository's tracked files, resident size read every 0.2 ms
///  in a fresh process (2026-11-03): an index of 2,786,564 bytes grew the process by 13,123,584 bytes (4.7 copies) and one of
///  1,066,894 bytes by 7,929,856 bytes (7.4 copies), which is 3.0 copies of the index plus about 4.7 MB that does not grow with
///  it. Four copies cover the growth for an index over 4.7 MB, the indexes this estimate exists for. Kept at 4 as the coordinator's pick under the owner's delegation, open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`, "Picked for these follow-ups", option A).
const INDEX_REWRITE_COPIES: u64 = 4;

/// What one rewrite of an index file is: the file, the tree it describes, and the trees its entries were copied from.
struct IndexTarget {
    /// The index file.
    index: PathBuf,
    /// The tree whose files the index describes.
    tree: PathBuf,
    /// The trees the entries were copied from (each is the reference tree joined with the worktree's place in the checkout).
    references: Vec<PathBuf>,
}

/// The rewrite of one index file, replaceable so a control can watch or delay it.
pub type RewriteOne<'a> = dyn Fn(&Path, &Path, &RefreshOptions<'_>) -> String + Sync + 'a;

/// What:
///  What a rewrite of every index of a checkout is told.
/// Why:
///   One value names the inputs: the checkout, how its indexes are locked, the trees they were copied from, the host the
///  budget is derived from, and the rewrite of one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllIndexes = { root: string; lock: IndexLock; references: string[]; host: HostResources; rewrite: RewriteOne };
/// ```
pub struct AllIndexes<'a> {
    /// The main checkout.
    pub root: &'a Path,
    /// How each index is protected while it is rewritten.
    pub lock: IndexLock<'a>,
    /// The trees the checkout was copied from; each linked worktree is looked up under the same relative path in them.
    pub references: &'a [PathBuf],
    /// The CPUs and memory the budget is derived from.
    pub host: &'a HostResources,
    /// The rewrite of one index file.
    pub rewrite: &'a RewriteOne<'a>,
}

/// What:
///  Rewrite the index of the checkout and of every linked worktree inside it, reporting each through `say`.
///  `&mut dyn FnMut(&str)` is a borrowed callback that receives one report line at a time.
///  `references` are the trees the checkout was copied from (empty trusts every entry); each linked worktree is looked up
///  under the same relative path in them.
/// Why:
///   The first `git status` of each of them is otherwise a full re-hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshAllIndexes(root: string, lock: IndexLock, references: string[], say: (line: string) => void): void;
/// ```
pub fn refresh_all_indexes(root: &Path, lock: IndexLock<'_>, references: &[PathBuf], say: &mut dyn FnMut(&str)) -> usize {
    return refresh_all_indexes_with(&AllIndexes { root, lock, references, host: &HostResources::read(), rewrite: &refresh_index_file_with }, say);
}

/// What:
///  `refresh_all_indexes` with the host and the rewrite of one file chosen by the caller; returns how many indexes it covered.
/// Why:
///   Each index is its own file under its own lock describing its own tree, so the rewrites run side by side within a budget
///  and are reported in the order the checkout's index first, then each linked worktree's by path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = await refreshAllIndexesWith({ root, lock, references, host, rewrite }, say);
/// ```
pub fn refresh_all_indexes_with(request: &AllIndexes<'_>, say: &mut dyn FnMut(&str)) -> usize {
    let AllIndexes { root, lock, references, host, rewrite } = *request;
    let hash_length: Option<usize> = repository_hash_length(root.join(".git").as_path());
    let mut targets: Vec<IndexTarget> = vec![IndexTarget { index: root.join(".git").join("index"), tree: root.to_path_buf(), references: references.to_vec() }];
    for linked in linked_worktrees_inside(root) {
        let relative: &Path = linked.worktree.strip_prefix(root).unwrap_or(Path::new(""));
        targets.push(IndexTarget {
            index: linked.admin_directory.join("index"),
            tree: linked.worktree.clone(),
            references: references.iter().map(|reference: &PathBuf| { return reference.join(relative); }).collect(),
        });
    }
    let largest: u64 = targets.iter().map(|target: &IndexTarget| { return std::fs::metadata(&target.index).map_or(0, |found: std::fs::Metadata| { return found.len(); }); }).max().unwrap_or(0);
    let workers: usize = worker_budget(&BudgetRequest { items: targets.len(), per_item_memory: largest.saturating_mul(INDEX_REWRITE_COPIES), ceiling: None, cap: None, host });
    let lanes: Vec<Vec<usize>> = (0..targets.len()).map(|index: usize| { return vec![index]; }).collect();
    run_ordered(OrderedRun {
        items: targets.as_slice(),
        lanes: lanes.as_slice(),
        workers,
        work: &|_index: usize, target: &IndexTarget, _worker_say: &mut dyn FnMut(&str)| {
            return rewrite(target.index.as_path(), target.tree.as_path(), &RefreshOptions { lock, references: target.references.as_slice(), hash_length });
        },
        on_event: &mut |event: Event<'_, String>| {
            if let Event::End(index, line) = event {
                say(format!("index {}: {line}", targets[index].index.display()).as_str());
            }
        },
    });
    return targets.len();
}

/// Index file controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_files_tests.rs"]
mod tests;

/// The controls for indexes rewritten side by side stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_parallel_tests.rs"]
mod parallel_tests;

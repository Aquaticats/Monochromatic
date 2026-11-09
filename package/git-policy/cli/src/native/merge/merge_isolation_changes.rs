//! What:
//!  The raw tree differences between the two commits of an isolated merge,
//! and the state of a worktree path
//!  against them.
//! Why:
//!  Recovery of a merge killed while Git was updating the worktree must tell,
//! for each path the merge changes,
//!  whether the worktree still holds the old content,
//! already holds the new content,
//! or holds something
//!  else that belongs to someone else and must not be overwritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const changes = parseRawChanges(await git(['diff-tree', '-r', '-z', '--raw', '--no-renames', '--no-abbrev', old, next]));
//! ```

/// Running real Git for a blob hash.
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// One path the merge changes,
/// with the mode and blob it has on each side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { path: Uint8Array; oldMode: string; newMode: string; oldOid: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    /// Path from the top of the worktree,
    /// exact bytes.
    pub path: Vec<u8>,
    /// Mode before the merge;
    /// `000000` when the path is added.
    pub old_mode: String,
    /// Mode after the merge;
    /// `000000` when the path is deleted.
    pub new_mode: String,
    /// Object before the merge;
    /// all zeros when the path is added.
    pub old_oid: String,
    /// Object after the merge;
    /// all zeros when the path is deleted.
    pub new_oid: String,
}

/// Methods over one change.
impl Change {
    /// Whether the path exists before the merge.
    pub fn existed(&self) -> bool {
        return self.old_mode != "000000";
    }

    /// Whether the path exists after the merge.
    pub fn exists_after(&self) -> bool {
        return self.new_mode != "000000";
    }
}

/// Whether one byte is the NUL byte that ends each field of a `-z` listing.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Read the NUL-separated raw output of `git diff-tree -r -z --raw --no-renames --no-abbrev`.
/// Why:
///   Each change is a header (`:<old mode> <new mode> <old oid> <new oid> <status>`) followed by its path.
///  Output that does not follow that shape is a failure,
/// never a partial reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRawChanges(output: Uint8Array): Change[]; // throws
/// ```
pub fn parse_raw_changes(output: &[u8]) -> Result<Vec<Change>, String> {
    let mut changes: Vec<Change> = Vec::new();
    let mut kept_fields: Vec<&[u8]> = Vec::new();
    for field in output.split(is_nul_byte) {
        if !field.is_empty() {
            kept_fields.push(field);
        }
    }
    let mut fields = kept_fields.into_iter();
    while let Some(header) = fields.next() {
        let text: String = String::from_utf8_lossy(header).into_owned();
        let parts: Vec<&str> = text.trim_start_matches(':').split(' ').collect();
        let Some(path) = fields.next() else {
            return Err(format!("the difference entry {text:?} has no path"));
        };
        if !header.starts_with(b":") || parts.len() < 5 {
            return Err(format!("the difference entry {text:?} is not in the raw format"));
        }
        changes.push(Change {
            path: path.to_vec(),
            old_mode: String::from(parts[0]),
            new_mode: String::from(parts[1]),
            old_oid: String::from(parts[2]),
            new_oid: String::from(parts[3]),
        });
    }
    return Ok(changes);
}

/// What the worktree holds at a path,
/// as the object Git would record for it and the mode it records for that kind of entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeState = { kind: 'missing' } | { kind: 'content'; oid: string; mode: string } | { kind: 'other' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorktreeState {
    /// Nothing exists at the path.
    Missing,
    /// A file or a symbolic link whose recorded object is this one,
    /// with the mode Git records for its kind.
    Content {
        /// The object the entry's bytes hash to.
        object: String,
        /// `100644`,
        /// `100755` or `120000`,
        /// as Git records it.
        mode: String,
    },
    /// A directory or another kind of entry.
    Other,
}

/// What:
///  What the file system holds at a worktree path when recovery judges it:
/// the device,
/// the inode,
/// the size and the modification time.
/// Why:
///   A file taken aside after the judgement is compared with this,
/// so the bytes a merge replaces are exactly the
///  bytes that were judged;
/// a file written between the judgement and the take differs here,
/// and is never
///  replaced.
/// The inode alone is never compared,
/// because a freed inode is reused,
/// so the modification time and the
///  size go with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileIdentity = { device: bigint; inode: bigint; size: bigint; modifiedSeconds: bigint; modifiedNanos: bigint };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    /// Device number.
    pub device: u64,
    /// Inode number.
    pub inode: u64,
    /// Size in bytes.
    pub size: u64,
    /// The mode bits,
    /// so a permission change made after the judgement is seen:
    /// a rename keeps them and changes nothing else here.
    pub mode: u32,
    /// Modification time,
    /// whole seconds since the epoch.
    pub modified_seconds: i64,
    /// Modification time,
    /// the nanoseconds part.
    pub modified_nanos: i64,
}

/// What:
///  The identity of whatever stands at `location`,
/// without following a link,
/// or nothing when nothing is there.
/// Why:
///   A symbolic link is a file of its own here,
/// because a take moves it and `checkout-index` writes it back as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fileIdentity(location: string): FileIdentity | undefined; // lstat, no link followed
/// ```
#[cfg(unix)]
pub fn file_identity(location: &Path) -> Option<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let found: std::fs::Metadata = std::fs::symlink_metadata(location).ok()?;
    return Some(FileIdentity {
        device: found.dev(),
        inode: found.ino(),
        size: found.len(),
        mode: found.mode(),
        modified_seconds: found.mtime(),
        modified_nanos: found.mtime_nsec(),
    });
}

/// The identity of whatever stands at `location`.
///  Always nothing on this platform:
/// the read of a device and an inode is a Unix one.
#[cfg(not(unix))]
pub fn file_identity(_location: &Path) -> Option<FileIdentity> {
    return None;
}

/// The filesystem path of a repository path given as raw bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(root, bytesToPath(path))
/// ```
pub fn worktree_path(root: &Path, path: &[u8]) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return root.join(std::ffi::OsStr::from_bytes(path));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return root.join(String::from_utf8_lossy(path).into_owned());
    }
}

/// What:
///  The state of one worktree path:
/// missing,
/// content with the object Git would record for it,
/// or another kind of entry.
/// Why:
///   A file is hashed through Git with its path,
/// so the clean filters and line-ending rules that apply to it apply
///  here;
/// a symbolic link is hashed from its target text,
/// which is what Git records and never the file it points at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function worktreeState({ context, root, path }): Promise<WorktreeState>;
/// ```
pub fn worktree_state(context: &GitContext, root: &Path, path: &[u8]) -> Result<WorktreeState, String> {
    return state_of_file(context, root, path, worktree_path(root, path).as_path());
}

/// What:
///  The state of the file at `location`,
/// read as the repository path `path` would be.
/// Why:
///  A take moves a file to a private name beside it,
/// and its bytes must be judged there with the attributes of the path
///  it belongs to;
/// one function reads both,
/// so the judgement of a taken file and of a file in place cannot differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stateOfFile({ context, root, path, location }): Promise<WorktreeState>;
/// ```
pub fn state_of_file(context: &GitContext, root: &Path, path: &[u8], location: &Path) -> Result<WorktreeState, String> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(location) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(WorktreeState::Missing),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(WorktreeState::Missing),
        Err(error) => return Err(format!("{} could not be read: {error}", location.display())),
    };
    if metadata.file_type().is_symlink() {
        let target: PathBuf = match std::fs::read_link(location) {
            Ok(read) => read,
            Err(error) => return Err(format!("{} could not be read: {error}", location.display())),
        };
        let mut request: GitRequest = GitRequest::new(root, &["hash-object", "--stdin"]);
        request.input = Some(target.as_os_str().as_encoded_bytes().to_vec());
        let output = match run_git_checked(context, &request) {
            Ok(done) => done,
            Err(error) => return Err(error.to_string()),
        };
        return Ok(WorktreeState::Content {
            object: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
            mode: String::from(SYMBOLIC_LINK_MODE),
        });
    }
    if !metadata.is_file() {
        return Ok(WorktreeState::Other);
    }
    // The attribute lookup is named by the path from the worktree top, where Git runs, and the bytes are read from `location`.
    let mut named: std::ffi::OsString = std::ffi::OsString::from("--path=");
    named.push(worktree_path(Path::new(""), path).into_os_string());
    let mut request: GitRequest = GitRequest::new(root, &["hash-object"]);
    request.arguments.push(named);
    request.arguments.push(std::ffi::OsString::from("--"));
    request.arguments.push(location.as_os_str().to_os_string());
    let output = match run_git_checked(context, &request) {
        Ok(done) => done,
        Err(error) => return Err(error.to_string()),
    };
    return Ok(WorktreeState::Content {
        object: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        mode: String::from(mode_of_file(&metadata)),
    });
}

/// The mode Git records for a symbolic link.
const SYMBOLIC_LINK_MODE: &str = "120000";

/// What:
///  The mode Git records for a regular file,
///  from the executable bit the file system holds.
/// Why:
///   Git records exactly two modes for a regular file,
/// and the bit between them is the one a person changes with
///  `chmod`;
/// reading it here is what lets a judgement see an executable bit set while a merge waited,
/// instead of
///  comparing the bytes alone and erasing the bit
///  (a cross-vendor review of the ninth round,
/// its seventh finding).
///  A platform whose permissions carry no such bit is read as non-executable,
/// which is the mode Git records there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modeOfFile(metadata): '100644' | '100755'; // stat.mode & 0o111 ? '100755' : '100644'
/// ```
#[cfg(unix)]
fn mode_of_file(metadata: &std::fs::Metadata) -> &'static str {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o111 != 0 {
        return "100755";
    }
    return "100644";
}

/// The mode Git records for a regular file,
/// where the platform's permissions carry no executable bit.
#[cfg(not(unix))]
fn mode_of_file(_metadata: &std::fs::Metadata) -> &'static str {
    return "100644";
}

/// What:
///  Whether this repository reads a worktree file's executable bit as part of the file.
/// Why:
///   `core.fileMode` is false where the filesystem cannot keep the bit,
/// and Git then takes no mode difference at
///  all;
/// a judgement that refused one there would stop the recovery of every merge touching an executable path on
///  such a filesystem,
/// which is the kind of failure this route exists to remove.
/// A value Git does not answer for,
/// and a key that is absent,
/// is read as true,
/// which is Git's default and the strict answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theRepositoryRecordsFileModes(context, root): boolean; // git config --bool core.filemode !== 'false'
/// ```
pub fn the_repository_records_file_modes(context: &GitContext, root: &Path) -> bool {
    let request: GitRequest = GitRequest::new(root, &["config", "--bool", "core.filemode"]);
    let Ok(output) = run_git_checked(context, &request) else {
        return true;
    };
    return String::from_utf8_lossy(&output.stdout).trim() != "false";
}

/// Raw difference reading and worktree states.
#[cfg(test)]
#[path = "merge_isolation_changes_tests.rs"]
mod tests;

/// What:
///  Whether a directory standing where the merge puts a file holds only files of the branch that the merge deletes,
///  and nothing else,
///  judged through every directory below it.
/// Why:
///   The branch's own directory is the merge's to replace,
/// but an untracked file in it is a person's,
/// and a directory
///  that keeps one cannot be replaced:
/// the merge would be half installed before the failure,
/// so it is refused
///  before anything is written.
/// The tree diff names the files the merge deletes and no entry for a directory of its own,
/// so a directory
///  below this one is judged by everything below it,
/// never by a change of its own name;
/// comparing the
///  immediate children with the diff read a directory of the branch's own nested files as someone else's,
/// and the
///  saved merge could never finish
///  (a cross-vendor review of the ninth round,
/// its eighth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aDirectoriesFilesAreAllDeletedByTheMerge({ change, changes, root }): boolean; // every entry below the directory is a deleted change or a directory of them
/// ```
pub fn the_directory_holds_only_what_the_merge_deletes(change: &Change, changes: &[Change], root: &Path) -> bool {
    let location: PathBuf = worktree_path(root, change.path.as_slice());
    return every_entry_below_is_one_the_merge_deletes(change.path.as_slice(), changes, location.as_path());
}

/// What:
///  Whether everything in one directory of the worktree is a path the merge deletes,
///  looked at in every directory
///  below it.
/// Why:
///   A directory cannot be named as a deleted path by a tree diff,
/// so a child that is a directory is judged by
///  its own entries.
/// An emptied directory below this one is accepted only where the merge deletes a path below it,
/// because
///  only then does the pruning of the merge's emptied directories reach it;
/// an emptied directory the pruning never
///  reaches keeps the checkout from holding the merge's file,
/// so it is refused before anything is written.
///  The directory this function is called on may itself be empty:
/// a recovery killed after finishing the last deleted
///  file and before removing the directory it emptied leaves exactly that,
/// and the merge's own deletions empty it.
///  A symbolic link is never followed:
/// its own path must be one the merge deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function everyEntryBelowIsOneTheMergeDeletes(prefix, changes, directory): boolean; // each child: a deleted change, or a directory whose every entry is
/// ```
fn every_entry_below_is_one_the_merge_deletes(prefix: &[u8], changes: &[Change], directory: &Path) -> bool {
    let listing: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(_) => return false,
    };
    for entry in listing.flatten() {
        let mut child: Vec<u8> = prefix.to_vec();
        child.push(b'/');
        child.extend_from_slice(entry.file_name().as_encoded_bytes());
        let deleted: bool = changes.iter().any(|other| return other.path == child && other.existed() && !other.exists_after());
        if deleted {
            continue;
        }
        let below: PathBuf = directory.join(entry.file_name());
        let is_a_directory: bool = std::fs::symlink_metadata(below.as_path()).map(|found| return found.is_dir()).unwrap_or(false);
        if !is_a_directory {
            return false;
        }
        let mut inner: std::fs::ReadDir = match std::fs::read_dir(below.as_path()) {
            Ok(found) => found,
            Err(_) => return false,
        };
        if inner.next().is_none() {
            if !the_merge_deletes_a_path_below(child.as_slice(), changes) {
                return false;
            }
            continue;
        }
        if !every_entry_below_is_one_the_merge_deletes(child.as_slice(), changes, below.as_path()) {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether the merge deletes a path below one directory of the branch.
/// Why:
///   The pruning of the merge's emptied directories walks up from each deleted path,
/// so an emptied directory is the
///  branch's only when such a path lies below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theMergeDeletesAPathBelow(prefix, changes): boolean; // changes.some((other) => other.path.startsWith(`${prefix}/`) && other.existed() && !other.existsAfter())
/// ```
fn the_merge_deletes_a_path_below(prefix: &[u8], changes: &[Change]) -> bool {
    let mut slash: Vec<u8> = prefix.to_vec();
    slash.push(b'/');
    for other in changes {
        if other.path.starts_with(slash.as_slice()) && other.existed() && !other.exists_after() {
            return true;
        }
    }
    return false;
}

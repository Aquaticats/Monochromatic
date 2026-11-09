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
/// as the object Git would record for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeState = { kind: 'missing' } | { kind: 'content'; oid: string } | { kind: 'other' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorktreeState {
    /// Nothing exists at the path.
    Missing,
    /// A file or a symbolic link whose recorded object is this one.
    Content(String),
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
        return Ok(WorktreeState::Content(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
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
    return Ok(WorktreeState::Content(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// Raw difference reading and worktree states.
#[cfg(test)]
#[path = "merge_isolation_changes_tests.rs"]
mod tests;

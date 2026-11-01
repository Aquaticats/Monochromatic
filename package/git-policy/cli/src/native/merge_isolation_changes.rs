//! What:
//!  The raw tree differences between the two commits of an isolated merge, and the state of a worktree path
//!  against them.
//! Why:
//!  Recovery of a merge killed while Git was updating the worktree must tell, for each path the merge changes,
//!  whether the worktree still holds the old content, already holds the new content, or holds something
//!  else that belongs to someone else and must not be overwritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const changes = parseRawChanges(await git(['diff-tree', '-r', '-z', '--raw', '--no-renames', '--no-abbrev', old, next]));
//! ```

/// Running real Git for a blob hash.
use super::transaction_git::{GitContext, GitRequest, run_git_checked};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// One path the merge changes, with the mode and blob it has on each side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { path: Uint8Array; oldMode: string; newMode: string; oldOid: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    /// Path from the top of the worktree, exact bytes.
    pub path: Vec<u8>,
    /// Mode before the merge; `000000` when the path is added.
    pub old_mode: String,
    /// Mode after the merge; `000000` when the path is deleted.
    pub new_mode: String,
    /// Object before the merge; all zeros when the path is added.
    pub old_oid: String,
    /// Object after the merge; all zeros when the path is deleted.
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

/// What:
///  Read the NUL-separated raw output of `git diff-tree -r -z --raw --no-renames --no-abbrev`.
/// Why:
///   Each change is a header (`:<old mode> <new mode> <old oid> <new oid> <status>`) followed by its path.
///  Output that does not follow that shape is a failure, never a partial reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRawChanges(output: Uint8Array): Change[]; // throws
/// ```
pub fn parse_raw_changes(output: &[u8]) -> Result<Vec<Change>, String> {
    let mut changes: Vec<Change> = Vec::new();
    let mut fields = output.split(|byte| { return *byte == 0; }).filter(|field| { return !field.is_empty(); });
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

/// What the worktree holds at a path, as the object Git would record for it.
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
///  The state of one worktree path: missing, content with the object Git would record for it, or another kind of entry.
/// Why:
///   A file is hashed through Git with its path, so the clean filters and line-ending rules that apply to it apply
///  here; a symbolic link is hashed from its target text, which is what Git records and never the file it points at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function worktreeState({ context, root, path }): Promise<WorktreeState>;
/// ```
pub fn worktree_state(context: &GitContext, root: &Path, path: &[u8]) -> Result<WorktreeState, String> {
    let location: PathBuf = worktree_path(root, path);
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(location.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(WorktreeState::Missing),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(WorktreeState::Missing),
        Err(error) => return Err(format!("{} could not be read: {error}", location.display())),
    };
    if metadata.file_type().is_symlink() {
        let target: PathBuf = std::fs::read_link(location.as_path())
            .map_err(|error| { return format!("{} could not be read: {error}", location.display()); })?;
        let mut request: GitRequest = GitRequest::new(root, &["hash-object", "--stdin"]);
        request.input = Some(target.as_os_str().as_encoded_bytes().to_vec());
        let output = run_git_checked(context, &request).map_err(|error| { return error.to_string(); })?;
        return Ok(WorktreeState::Content(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
    }
    if !metadata.is_file() {
        return Ok(WorktreeState::Other);
    }
    // The attribute lookup and the file are both named by the path from the worktree top, where Git runs.
    let relative: std::ffi::OsString = worktree_path(Path::new(""), path).into_os_string();
    let mut named: std::ffi::OsString = std::ffi::OsString::from("--path=");
    named.push(relative.as_os_str());
    let mut request: GitRequest = GitRequest::new(root, &["hash-object"]);
    request.arguments.push(named);
    request.arguments.push(std::ffi::OsString::from("--"));
    request.arguments.push(relative);
    let output = run_git_checked(context, &request).map_err(|error| { return error.to_string(); })?;
    return Ok(WorktreeState::Content(String::from_utf8_lossy(&output.stdout).trim().to_owned()));
}

/// Raw difference reading and worktree states.
#[cfg(test)]
#[path = "merge_isolation_changes_tests.rs"]
mod tests;

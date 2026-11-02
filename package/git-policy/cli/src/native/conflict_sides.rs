//! What:
//!  The decision and the file work behind taking one side of a conflicted file: which operation is in progress, which
//!  paths the index marks as conflicted, whether a command line names only such paths, and the saved copy of each file
//!  before the command replaces it.
//! Why:
//!  `linked-worktree-only` rejects `git checkout --theirs -- a.rs` because it overwrites the worktree file. During a
//!  merge, a rebase, a cherry-pick or a revert that command is how a conflict is settled, and the file it overwrites is the
//!  half-resolved copy a person may have edited by hand. The owner's decision is to let it through for conflicted paths
//!  after the current file is copied beside the audit log (`discard_audit.rs`), so nothing is lost and the copy is named.
//!  Everything here is read in process: the operation from the state files in the Git directory, the conflicted paths from
//!  the unmerged entries of the index file. The copy is written and synced before the caller lets Git run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const conflicted = unmergedPaths(await readIndex(gitDir));
//! const decision = decideSides({ pathspecs, prefix, conflicted });
//! if (decision.kind === 'allow') await saveCopies({ root, commonDir, paths: decision.paths });
//! ```

use super::iso_time::format_iso_milliseconds;
use super::private_storage::{create_private_directory, create_private_directory_all, sync_directory};
use super::repo_read_index::IndexEntry;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// Where the copies live, below the common Git directory, beside `cli-git-audit.jsonl`.
pub const SAVED_DIRECTORY: &str = "cli-git/discarded";

/// An operation that stops with conflicts and waits for them to be settled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// `git merge`.
    Merge,
    /// `git rebase`, either backend.
    Rebase,
    /// `git cherry-pick`.
    CherryPick,
    /// `git revert`.
    Revert,
}

/// What:
///  The operation in progress in a worktree's Git directory, if any.
/// Why:
///   Git leaves `MERGE_HEAD`, `CHERRY_PICK_HEAD` or `REVERT_HEAD` while a merge, cherry-pick or revert waits, and a
///  `rebase-merge` directory or a `rebase-apply` directory that holds `rebasing` for a rebase (the same directory with
///  `applying` is `git am`, whose conflicts are not covered).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function operationInProgress(gitDir: string): Operation | undefined;
/// ```
pub fn operation_in_progress(git_dir: &Path) -> Option<Operation> {
    if git_dir.join("MERGE_HEAD").exists() {
        return Some(Operation::Merge);
    }
    if git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply/rebasing").exists() {
        return Some(Operation::Rebase);
    }
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return Some(Operation::CherryPick);
    }
    if git_dir.join("REVERT_HEAD").exists() {
        return Some(Operation::Revert);
    }
    return None;
}

/// What:
///  The paths with an unmerged entry, once each.
/// Why:
///   A conflicted path has entries at stage 1, 2 or 3 in place of the one at stage 0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const conflicted = new Set(entries.filter((entry) => entry.stage !== 0).map((entry) => entry.path));
/// ```
pub fn conflicted_paths(entries: &[IndexEntry]) -> BTreeSet<Vec<u8>> {
    return entries.iter().filter(|entry| { return entry.stage != 0; }).map(|entry| { return entry.path.clone(); }).collect();
}

/// What:
///  The repository-relative path a pathspec token names, when it names exactly one file by its literal spelling.
/// Why:
///   Git resolves a pathspec below the directory the command runs in. Magic (`:(glob)`, `:/`), wildcards, a trailing
///  slash and a directory are not a single conflicted file, so they resolve to nothing and the usual rejection stays.
///  `.` and `..` segments are folded; a path that leaves the worktree resolves to nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function literalPath(prefix: string, root: string, token: string): string | undefined;
/// ```
pub fn literal_path(prefix: &[u8], root: &Path, token: &OsStr) -> Option<Vec<u8>> {
    let bytes: &[u8] = token.as_bytes();
    if bytes.is_empty() || bytes[0] == b':' || bytes.ends_with(b"/") || bytes.iter().any(|byte| { return matches!(byte, b'*' | b'?' | b'[' | b'\\'); }) {
        return None;
    }
    let joined: Vec<u8> = if bytes[0] == b'/' {
        let below: &Path = Path::new(token).strip_prefix(root).ok()?;
        below.as_os_str().as_bytes().to_vec()
    } else {
        [prefix, bytes].concat()
    };
    let mut kept: Vec<&[u8]> = Vec::new();
    for segment in joined.split(|byte| { return *byte == b'/'; }) {
        match segment {
            b"" | b"." => {}
            b".." => {
                kept.pop()?;
            }
            other => kept.push(other),
        }
    }
    if kept.is_empty() {
        return None;
    }
    return Some(kept.join(&b'/'));
}

/// What a command line that names paths came to, against the conflicted set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideDecision {
    /// Every path is a conflicted file: the repository-relative paths, in the order named, each once.
    Allow(Vec<Vec<u8>>),
    /// At least one token is not a conflicted file; these are the tokens, as typed.
    Refuse(Vec<OsString>),
}

/// What:
///  Whether every token names a conflicted file.
/// Why:
///   A pathspec that mixes conflicted and clean paths is refused whole, and the refusal names the clean ones, so a
///  person sees exactly what to leave out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideSides(tokens: string[], prefix: string, root: string, conflicted: Set<string>): SideDecision;
/// ```
pub fn decide_sides(tokens: &[OsString], prefix: &[u8], root: &Path, conflicted: &BTreeSet<Vec<u8>>) -> SideDecision {
    let mut allowed: Vec<Vec<u8>> = Vec::new();
    let mut others: Vec<OsString> = Vec::new();
    for token in tokens {
        match literal_path(prefix, root, token.as_os_str()) {
            Some(path) if conflicted.contains(&path) => {
                if !allowed.contains(&path) {
                    allowed.push(path);
                }
            }
            _ => others.push(token.clone()),
        }
    }
    if others.is_empty() {
        return SideDecision::Allow(allowed);
    }
    return SideDecision::Refuse(others);
}

/// Why a copy could not be written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveFailure {
    /// The repository-relative path being copied, or the place the copies go when none was reached.
    pub subject: String,
    /// The operating system's words.
    pub reason: String,
}

/// One saved file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedCopy {
    /// The repository-relative path.
    pub path: String,
    /// Where the copy is, or nothing when the path is deleted in the worktree and has nothing to copy.
    pub saved_to: Option<PathBuf>,
}

/// The name of a directory for copies made at `milliseconds`: `2026-11-02T14-01-07`.
fn stamp(milliseconds: i64) -> String {
    return format_iso_milliseconds(milliseconds).chars().take(19).map(|each| { return if each == ':' { '-' } else { each }; }).collect();
}

/// Create the directory for this invocation's copies, never one that exists: a second invocation in the same second
///  gets a numbered sibling, so no copy overwrites another.
fn fresh_directory(saved_root: &Path, name: &str) -> std::io::Result<PathBuf> {
    create_private_directory_all(saved_root)?;
    for number in 0_u32..1000 {
        let candidate: PathBuf = if number == 0 { saved_root.join(name) } else { saved_root.join(format!("{name}-{number}")) };
        match create_private_directory(candidate.as_path()) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    return Err(std::io::Error::other("a thousand copies were saved in this second"));
}

/// Copy one file's content, mode and kind to `destination`, which must not exist, and sync the file.
fn copy_one(source: &Path, destination: &Path) -> std::io::Result<()> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        std::os::unix::fs::symlink(std::fs::read_link(source)?, destination)?;
        return Ok(());
    }
    if !metadata.file_type().is_file() {
        return Err(std::io::Error::other("it is neither a file nor a symbolic link"));
    }
    let mut from: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(source)?;
    let mut to: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(destination)?;
    let mut buffer: Vec<u8> = vec![0; 1 << 16];
    loop {
        let count: usize = from.read(buffer.as_mut_slice())?;
        if count == 0 {
            break;
        }
        to.write_all(&buffer[..count])?;
    }
    to.set_permissions(std::fs::Permissions::from_mode_bits(metadata.mode() & 0o777))?;
    to.sync_all()?;
    return Ok(());
}

/// `Permissions` from raw mode bits, named for what the call does.
trait FromModeBits {
    /// The permissions with exactly these bits.
    fn from_mode_bits(bits: u32) -> Self;
}

impl FromModeBits for std::fs::Permissions {
    fn from_mode_bits(bits: u32) -> Self {
        return std::os::unix::fs::PermissionsExt::from_mode(bits);
    }
}

/// What:
///  Save the current worktree file of each path below the common Git directory and sync everything written.
/// Why:
///   The command that follows overwrites these files, so each is copied first with its content, its permission bits or
///  its symbolic-link target. A path with no file in the worktree has nothing to copy and is reported as such. The
///  first failure stops the work and is returned, so the caller refuses the command instead of discarding unsaved work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copies = await saveCopies({ root, commonDir, paths, milliseconds });
/// ```
pub fn save_copies(root: &Path, common_dir: &Path, paths: &[Vec<u8>], milliseconds: i64) -> Result<Vec<SavedCopy>, SaveFailure> {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    let mut copies: Vec<SavedCopy> = Vec::new();
    let mut directory: Option<PathBuf> = None;
    let mut written_directories: BTreeSet<PathBuf> = BTreeSet::new();
    for path in paths {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let source: PathBuf = root.join(OsString::from_vec(path.clone()));
        match std::fs::symlink_metadata(source.as_path()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                copies.push(SavedCopy { path: shown, saved_to: None });
                continue;
            }
            Err(error) => return Err(SaveFailure { subject: shown, reason: error.to_string() }),
            Ok(_) => {}
        }
        let base: PathBuf = match &directory {
            Some(existing) => existing.clone(),
            None => {
                let created: PathBuf = fresh_directory(saved_root.as_path(), stamp(milliseconds).as_str())
                    .map_err(|error| { return SaveFailure { subject: saved_root.display().to_string(), reason: error.to_string() }; })?;
                directory = Some(created.clone());
                created
            }
        };
        let destination: PathBuf = base.join(OsString::from_vec(path.clone()));
        let failed = |error: std::io::Error| -> SaveFailure { return SaveFailure { subject: shown.clone(), reason: error.to_string() }; };
        if let Some(parent) = destination.parent() {
            create_private_directory_all(parent).map_err(failed)?;
            let mut up: Option<&Path> = Some(parent);
            while let Some(each) = up {
                written_directories.insert(each.to_path_buf());
                if each == base.as_path() {
                    break;
                }
                up = each.parent();
            }
        }
        copy_one(source.as_path(), destination.as_path()).map_err(failed)?;
        copies.push(SavedCopy { path: shown, saved_to: Some(destination) });
    }
    if directory.is_some() {
        // Deepest first, then the directories that hold the copies' root, so every new entry is durable.
        let mut ordered: Vec<&PathBuf> = written_directories.iter().collect();
        ordered.sort_by_key(|each| { return std::cmp::Reverse(each.components().count()); });
        for each in ordered {
            sync_directory(each.as_path()).map_err(|error| { return SaveFailure { subject: each.display().to_string(), reason: error.to_string() }; })?;
        }
        let parent: PathBuf = common_dir.join("cli-git");
        for each in [saved_root.as_path(), parent.as_path()] {
            sync_directory(each).map_err(|error| { return SaveFailure { subject: each.display().to_string(), reason: error.to_string() }; })?;
        }
    }
    return Ok(copies);
}

/// What:
///  The plain-language notice for the copies, one block per path.
/// Why:
///   The person is told where each copy is before Git runs, so a file they edited by hand can be found again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedNotice(copies: SavedCopy[]): string;
/// ```
pub fn saved_notice(copies: &[SavedCopy]) -> String {
    let mut text: String = String::new();
    for copy in copies {
        match &copy.saved_to {
            Some(place) => text.push_str(
                format!(
                    "cli-git: saved your copy of {} to\n  {}\n  before this command replaces it ({} was conflicted).\n",
                    copy.path,
                    place.display(),
                    copy.path
                )
                .as_str(),
            ),
            None => text.push_str(
                format!(
                    "cli-git: {} was conflicted and is deleted in your worktree, so there is no copy to save; this command brings a file back.\n",
                    copy.path
                )
                .as_str(),
            ),
        }
    }
    return text;
}

/// What:
///  The sentence a rejection gets when the command qualified for a conflict side but some paths were not conflicted.
/// Why:
///   The person is told what is allowed and which paths to leave out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanPathsNote(others: string[], operation: Operation): string;
/// ```
pub fn clean_paths_note(others: &[OsString], operation: Operation) -> String {
    let named: Vec<String> = others.iter().map(|each| { return format!("`{}`", each.to_string_lossy()); }).collect();
    let word: &str = match operation {
        Operation::Merge => "merge",
        Operation::Rebase => "rebase",
        Operation::CherryPick => "cherry-pick",
        Operation::Revert => "revert",
    };
    return format!(
        " This {word} is in progress, and a command that names only conflicted files is let through after your copy is saved, \
         but these are not conflicted files: {}. Leave them out, or name each conflicted file itself (no directory, wildcard or \
         pathspec magic).",
        named.join(", ")
    );
}

/// The decision, the literal paths, the copies and the notices.
#[cfg(test)]
#[path = "conflict_sides_tests.rs"]
mod tests;

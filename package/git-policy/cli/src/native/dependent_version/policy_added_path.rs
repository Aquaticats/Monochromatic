//! What:
//! Whether a commit or a direct fix may change a tracked file it did not select,
//! the
//!       `patch-conflict` message when a fix may not,
//! and the note a commit writes instead.
//! Why:
//! `SPEC.md` ("Added paths") admits such a file only when `HEAD`,
//! the real index,
//! the
//!      private index and the worktree all hold the blob the correction was computed
//!      against,
//! so neither lifecycle discards nobody's change:
//! a fix refuses the work,
//! and a
//!      commit leaves the file out and says which and why,
//! because a commit never fails for it.
//! The reasons and the remedy are the
//!      installed wrapper's words (`commit-transaction-added-paths.ts`,
//! `assertAddablePath`
//!      and its per-lifecycle messages),
//! checked in its order.
//!
//! `commit-transaction-added-paths.ts` was deleted on 2026-10-13 in `642b6d72cb` with the TypeScript wrapper,
//!  and `git show 642b6d72cb^:package/git-policy/cli/src/policy-engine/commit-transaction-added-paths.ts` reads it there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await assertAddablePath({ ..., path, oid, lifecycle: 'direct-fix' }); // throws AddedPathPreconditionError
//! ```

/// The modes and object names the precondition compares.
use crate::candidate_object::{CandidateMode, ObjectId, mode_from_git, parse_object_id};
/// The pathname conversion the worktree copy is found through.
use crate::git_metadata::path_from_git_bytes;
/// The exact path bytes a Git command is given.
use crate::path_bytes::os_from_path_bytes;
/// The Git context and request the real index is listed in.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// The repository location Git answered for the invocation the listing serves.
use crate::repository_location::RepositoryLocation;
/// The worktree top the listing runs in.
use crate::worktree_identity::worktree_root;
/// `BTreeMap<K, V>` is a map kept sorted by key.
use std::collections::BTreeMap;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// The reason `HEAD` does not hold the file the correction was computed against.
pub const ADDED_PATH_HEAD_REASON: &str =
    "HEAD does not hold it as the ordinary file the fix was computed against";

/// The reason the index holds a different entry than `HEAD`.
pub const ADDED_PATH_STAGED_REASON: &str = "it has staged changes";

/// The reason the worktree has no copy of the file.
pub const ADDED_PATH_MISSING_REASON: &str = "its worktree copy is missing";

/// The reason the worktree copy differs from `HEAD`.
pub const ADDED_PATH_UNSTAGED_REASON: &str = "its worktree copy has unstaged changes";

/// What:
/// An index or tree entry:
/// its mode and object.
/// Why:
///  Entries are compared field by field,
/// as the installed wrapper compares mode text
///       and object names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { mode: CandidateMode; object: ObjectId };
/// ```
pub type Entry = (CandidateMode, ObjectId);

/// What:
/// The `patch-conflict` message for a file and a reason.
/// Why:
///  It names both ways out:
/// select the file,
/// or restore it to `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directFixMessage = ({ path, reason }) => `A policy fix needs to change ${path}, ...`;
/// ```
pub fn added_path_message(path: &str, reason: &str) -> String {
    return format!(
        "A policy fix needs to change {path}, which this fix did not select, but {reason}. \
         Include {path} in the fix pathspecs so the fix applies to its worktree copy, \
         or restore it to match HEAD (git restore --staged --worktree -- {path}), \
         then run the fix again."
    );
}

/// What:
/// The message for a dependent a commit left out because a copy of its manifest holds
///       changes the commit does not carry.
/// `name`,
///  `from` and `to` are the dependent's name and the version it keeps and would take.
/// Why:
///  The person must know which dependent stayed at its old version,
///  what the commit did
///       instead of writing to its copy,
///  and both ways to carry it:
/// commit that file too,
/// or restore
///       it to `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitMessage = ({ path, name, from, to, reason }) => `A policy fix needs to add ${path} to this commit, but ${reason}. ${name} keeps ${from} ...`;
/// ```
pub fn commit_added_path_message(path: &str, name: &str, from: &str, to: &str, reason: &str) -> String {
    return format!(
        "A policy fix needs to add {path} to this commit to bump {name} from {from} to {to}, but {reason}. {name} \
         keeps {from} in this commit, and its copy was left exactly as it is. Commit the file too, with its version \
         set to {to}, or restore it to match HEAD (git restore --staged --worktree -- {path}), then commit again."
    );
}

/// What:
/// Whether a mode is an ordinary file.
/// Why:
///  Only regular and executable files take a full-content correction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOrdinary = (mode) => mode === 'regular' || mode === 'executable';
/// ```
fn is_ordinary(mode: CandidateMode) -> bool {
    return mode == CandidateMode::Regular || mode == CandidateMode::Executable;
}

/// What:
/// `HEAD`'s mode for the file,
/// or why `HEAD` and the index rule it out.
/// `index` is
///       the private index's entry,
/// whose object the correction was computed against;
/// for
///       a file the fix did not select it equals the real index's entry.
/// Why:
///  `HEAD` must hold the same object as an ordinary file;
/// the index must then hold it
///       with `HEAD`'s mode.
/// The mode is what the worktree copy is checked against next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function headMode(head: Entry | undefined, index: Entry): CandidateMode; // throws the reason
/// ```
pub fn head_mode(head: Option<&Entry>, index: &Entry) -> Result<CandidateMode, &'static str> {
    let Some((mode, object)) = head else {
        return Err(ADDED_PATH_HEAD_REASON);
    };
    if *object != index.1 || !is_ordinary(*mode) {
        return Err(ADDED_PATH_HEAD_REASON);
    }
    if index.0 != *mode {
        return Err(ADDED_PATH_STAGED_REASON);
    }
    return Ok(*mode);
}

/// What:
/// Whether the file's executable bit matches `mode`.
/// On systems without one,
/// it does.
/// Why:
///  A worktree copy whose bit differs from `HEAD` has an unstaged mode change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const executableMatches = ((stat.mode & 0o100) !== 0) === (mode === 'executable');
/// ```
fn executable_matches(metadata: &std::fs::Metadata, mode: CandidateMode) -> bool {
    #[cfg(unix)]
    {
        /// Unix permission bits.
        use std::os::unix::fs::PermissionsExt;
        return (metadata.permissions().mode() & 0o100 != 0) == (mode == CandidateMode::Executable);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (metadata, mode);
        return true;
    }
}

/// What:
/// Why the worktree copy rules out the file,
/// if it does.
/// `Err` is a read the system
///       refused for another reason than the file being absent.
/// Why:
///  The copy must be a plain file,
/// not a link,
/// with `HEAD`'s executable bit and bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function worktreeRefusal(file: string, mode, headBytes): Promise<string | undefined>;
/// ```
pub fn worktree_refusal(
    file: &Path,
    mode: CandidateMode,
    head_bytes: &[u8],
) -> Result<Option<&'static str>, std::io::Error> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(file) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Some(ADDED_PATH_MISSING_REASON));
        }
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || !executable_matches(&metadata, mode) {
        return Ok(Some(ADDED_PATH_UNSTAGED_REASON));
    }
    if std::fs::read(file)? != head_bytes {
        return Ok(Some(ADDED_PATH_UNSTAGED_REASON));
    }
    return Ok(None);
}

/// What:
///  The four copies of one path a commit compares before it adds that path to
///       itself,
///  and the bytes the correction was computed against.
/// Why:
///   One record keeps them together,
///  so the rule that judges them cannot be handed a pair from
///       different files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitAddedPath = { path: string; head?: Entry; commit: Entry; real?: Entry; original: Uint8Array; root: string };
/// ```
pub struct CommitAddedPath<'a> {
    /// The repository path the correction names.
    pub path: &'a [u8],
    /// `HEAD`'s entry,
    ///  when `HEAD` holds the path.
    pub head: Option<Entry>,
    /// The commit index's entry,
    ///  whose bytes the correction was computed against.
    pub commit: Entry,
    /// The real index's entry,
    ///  when it holds the path.
    pub real: Option<Entry>,
    /// The bytes the correction was computed against.
    pub original: &'a [u8],
    /// The worktree root the copy is read under.
    pub root: &'a Path,
}

/// What:
///  Why a commit may not add one tracked path to itself,
///  or nothing when every copy holds
///       the file the correction was computed against.
///  `Err` is a read the system refused for another reason than a copy being absent.
/// Why:
///  A commit adds a path the person did not select only while `HEAD`,
///  the real index,
///  the
///       commit's own index and the worktree all hold that one blob,
///  so the commit lands what the
///       person saw and the worktree copy takes the committed bytes afterwards.
///  A copy that
///       holds something else is somebody's work,
///       and it is left exactly as it is.
///  The reasons are the fix's own,
///  checked in the same order,
///  so one rule answers both lifecycles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitRefusal(added: CommitAddedPath): Promise<string | undefined>;
/// ```
pub fn commit_refusal(added: &CommitAddedPath<'_>) -> Result<Option<&'static str>, std::io::Error> {
    let mode: CandidateMode = match head_mode(added.head.as_ref(), &added.commit) {
        Ok(found) => found,
        Err(reason) => return Ok(Some(reason)),
    };
    // The landing writes the commit's entry into the real index, so the real index must hold that entry already.
    if added.real.as_ref() != Some(&added.commit) {
        return Ok(Some(ADDED_PATH_STAGED_REASON));
    }
    let Some(relative) = path_from_git_bytes(added.path) else {
        return Ok(Some(ADDED_PATH_UNSTAGED_REASON));
    };
    let file: PathBuf = added.root.join(relative);
    return worktree_refusal(file.as_path(), mode, added.original);
}

/// What:
///  A Git failure as its message text.
///  `GitFailure` is the one-field failure a Git started by the wrapper reports.
/// Why:
///  `map_err` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failureMessage = (failure: GitFailure) => failure.message;
/// ```
fn git_failure_message(failure: GitFailure) -> String {
    return failure.0;
}

/// What:
///  Whether one byte is the NUL byte that ends each record of a `-z` listing.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///  `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether one byte is the tab that separates an entry's metadata from its path.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///  `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTabByte = (byte: number) => byte === 0x09;
/// ```
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  Whether one byte is the space that separates an entry's mode,
///  object and stage.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///  `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSpaceByte = (byte: number) => byte === 0x20;
/// ```
fn is_space_byte(byte: &u8) -> bool {
    return *byte == b' ';
}

/// What:
///  The real index's stage-0 entries for the given paths.
///  A path the index holds no stage-0 entry for is absent from the map.
/// Why:
///   `commit_refusal` compares these with the commit's own entries,
///  and the real index is not the
///       index a commit reads its candidates from.
///  One start lists every path,
///  so a commit that
///       adds several dependents pays one Git process and not one per file.
///  The listing keeps none of the caller's other options,
///  so it names the repository the location holds:
/// the pair Git resolved once,
///  rather than the caller's `--git-dir` and `--work-tree` read again from a directory the wrapper discovered.
///  A relative `--git-dir` is resolved after the caller's last `-C`,
///  so reading it again from the worktree top applies that directory change a second time
///  and names a repository that does not exist.
///  A Git directory beside its work tree has no `.git` inside the work tree to find,
/// so a listing that named no repository would answer "not a git repository"
/// for every dependent whose copy it was asked to prove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function realIndexEntries({ gitPath, location, realIndexPath, paths }): Promise<Map<string, { mode; oid }>>;
/// ```
pub fn real_index_entries(
    context: &GitContext,
    location: &RepositoryLocation,
    index: &Path,
    paths: &[Vec<u8>],
) -> Result<BTreeMap<Vec<u8>, Entry>, String> {
    let Some(repository_root) = worktree_root(&location.identity) else {
        return Err(String::from(
            "the repository holds no work tree to list the real index in",
        ));
    };
    let mut request: GitRequest = GitRequest::new(repository_root, &["ls-files", "--stage", "-z", "--full-name", "--"]).own_pathspecs();
    for path in paths {
        let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
        pathspec.extend_from_slice(path.as_slice());
        request.arguments.push(os_from_path_bytes(pathspec.as_slice()));
    }
    // The listing is the caller's own pathspecs, and the caller's pathspec mode would read them as file names.
    request.without_prefix = true;
    request.repository_selection = crate::repository_selection::of_identity(&location.identity);
    request.index = Some(index.to_path_buf());
    let output: GitOutput = run_git_checked(context, &request).map_err(git_failure_message)?;
    let mut entries: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in output.stdout.split(is_nul_byte) {
        let Some(tab) = record.iter().position(is_tab_byte) else {
            continue;
        };
        let metadata: &[u8] = &record[..tab];
        let fields: Vec<&[u8]> = metadata.split(is_space_byte).collect();
        let [raw_mode, raw_object, stage] = fields.as_slice() else {
            continue;
        };
        // An unmerged entry has three stages and no single content, so it can never be the one blob.
        if *stage != b"0" {
            continue;
        }
        let (Some(mode), Some(object)) = (mode_from_git(raw_mode), parse_object_id(raw_object)) else {
            continue;
        };
        entries.insert(record[tab + 1..].to_vec(), (mode, object));
    }
    return Ok(entries);
}

/// Precondition and message controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_added_path_tests.rs"]
mod tests;

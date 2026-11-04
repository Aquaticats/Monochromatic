//! What:
//!  The commits real Git builds for the commit service: an editor, a signature, and every command line the service does
//!  not build itself.
//! Why:
//!  Git owns the editor, the signer and the message semantics of whatever the service cannot embed, so this module points
//!  real `git commit` at a thin scratch repository (its `HEAD` and branch at the base, the policy-settled private index,
//!  the real configuration included, every hook off) whose new objects go straight into the real object store. Nothing is
//!  locked and nothing is registered: the scratch repository sits inside the commit's own scratch directory, is removed
//!  with it, and the commit it makes is only an object until the service moves the branch to it by compare-and-swap
//!  (`doc/planning/cli-git-resident-service.md`, "Commits Git builds").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { commit, parents } = await buildWithGit({ context, cwd, capture, scratch, commitIndex, arguments });
//! ```

use crate::capture_invocation::InvocationCapture;
use crate::commit_args::{resolve_private_commit_args, split_commit_invocation, without_all_flag};
use crate::commit_preparation::{PreparationRequest, read_prepared_commit, run_native_commit};
use crate::commit_transaction::TransactionStop;
use crate::global_arguments::global_layout;
use crate::shadow_create::create_shadow_repository_carrying;
use crate::transaction_git::GitContext;
use crate::transaction_journal::TransactionMode;
use std::ffi::OsString;
use std::path::Path;

/// What the scratch repository's `git commit` made.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitBuilt {
    /// The commit, already in the real object store.
    pub commit: String,
    /// Its parents in order.
    pub parents: Vec<String>,
    /// Its message as stored: what the person typed, after Git's clean-up.
    pub message: String,
}

/// The message after the header block of a stored commit.
pub fn message_of(raw: &[u8]) -> String {
    let start: usize = raw.windows(2).position(|window| { return window == b"\n\n"; }).map_or(raw.len(), |at| { return at + 2; });
    return String::from_utf8_lossy(&raw[start..]).into_owned();
}

/// Everything the build needs from the preparation that settled the private index.
pub struct BuildInput<'a> {
    /// How to run the real Git.
    pub context: &'a GitContext,
    /// The directory the command was given in.
    pub cwd: &'a Path,
    /// The invocation capture: the branch, the base, the object directory.
    pub capture: &'a InvocationCapture,
    /// The commit's own scratch directory, which holds the scratch repository and is removed with it.
    pub scratch: &'a Path,
    /// The private index the policies settled.
    pub commit_index: &'a Path,
    /// The arguments after the policies' fixed transforms, global options included.
    pub arguments: &'a [OsString],
    /// The positional pathspecs of the command line, which the private index already holds.
    pub pathspecs: &'a [String],
    /// Whether the line commits named paths or the index.
    pub mode: TransactionMode,
    /// Whether `-a` staged tracked changes into the private index.
    pub staged_all: bool,
    /// Whether the private index already holds a selection made with `--include`, `--patch` or `--interactive`, so that Git
    ///  is not given those options again.
    pub selected_privately: bool,
}

/// The parents named by a stored commit's header block.
pub fn parents_of(raw: &[u8]) -> Vec<String> {
    let header_end: usize = raw.windows(2).position(|window| { return window == b"\n\n"; }).unwrap_or(raw.len());
    return String::from_utf8_lossy(&raw[..header_end])
        .lines()
        .filter_map(|line| { return line.strip_prefix("parent ").map(str::to_string); })
        .collect();
}

/// What:
///  The kept global options and the commit options of the arguments, with the pathspecs and the selection options the
///  private index already holds removed.
/// Why:
///   Real `git commit` must commit the private index as it stands, so it is given neither the paths nor `-a`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// privateInvocation({ arguments, pathspecs, mode, stagedAll })
/// ```
pub fn private_invocation(
    arguments: &[OsString],
    pathspecs: &[String],
    mode: TransactionMode,
    staged_all: bool,
    selected_privately: bool,
) -> (Vec<String>, Vec<String>) {
    let text: Vec<String> = arguments.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect();
    let private_args: Vec<String> = resolve_private_commit_args(text.as_slice(), pathspecs, mode, selected_privately);
    let owned: Vec<OsString> = private_args.iter().map(OsString::from).collect();
    let index: usize = global_layout(owned.as_slice()).prefix_len;
    let (globals, commit_args): (Vec<String>, Vec<String>) = split_commit_invocation(private_args.as_slice(), index);
    return if staged_all { (globals, without_all_flag(commit_args.as_slice())) } else { (globals, commit_args) };
}

/// What:
///  Runs real `git commit` against the scratch repository and reads the commit it made.
/// Why:
///   Git's own output, editor and signer reach the person's terminal as they would from a plain `git commit`; a nonzero
///  status of Git is the command's status and nothing lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await buildWithGit(input)
/// ```
///
/// # Errors
///
/// `TransactionStop::NativeExit` with Git's status when Git declined or was aborted (an empty message, an editor that
///  failed, a signer that failed); `TransactionStop::Message` when Git could not be started.
pub fn build_with_git(input: &BuildInput<'_>) -> Result<GitBuilt, TransactionStop> {
    let shadow = input.scratch.join("shadow");
    // Named paths while another session's merge, cherry-pick or revert is unfinished: real Git refuses that as a partial commit,
    //  and a commit never fails for another session's unfinished work, so the scratch repository holds none of that state and
    //  builds an ordinary commit of the paths; the worktree's state stays for its owner to conclude on the new head.
    let git_dir: &Path = input.capture.layout.git_dir.as_path();
    let carry_conclusion: bool = !(input.mode == TransactionMode::ExplicitPath && ["MERGE_HEAD", "CHERRY_PICK_HEAD"].iter().any(|name| { return git_dir.join(name).exists(); }));
    create_shadow_repository_carrying(input.context, input.cwd, input.capture, shadow.as_path(), input.scratch, carry_conclusion)?;
    // The wrapper adds no option of its own to the person's command line: Git prints what it prints, and `--quiet` reaches it only
    //  when the person typed it.
    let (globals, commit_args): (Vec<String>, Vec<String>) = private_invocation(input.arguments, input.pathspecs, input.mode, input.staged_all, input.selected_privately);
    let request: PreparationRequest<'_> = PreparationRequest {
        capture: input.capture,
        shadow: shadow.as_path(),
        commit_index: input.commit_index,
        global_args: globals.as_slice(),
        commit_args: commit_args.as_slice(),
        object_directory: Some(input.capture.layout.object_directory.as_path()),
    };
    let status: i32 = run_native_commit(input.context, input.cwd, &request).map_err(|error| { return TransactionStop::Message(error.to_string()); })?;
    if status != 0 {
        return Err(TransactionStop::NativeExit(status));
    }
    let prepared = read_prepared_commit(input.context, shadow.as_path())?;
    return Ok(GitBuilt { parents: parents_of(prepared.raw.as_slice()), message: message_of(prepared.raw.as_slice()), commit: prepared.oid });
}

/// Controls for building with Git stay out of the release executable.
#[cfg(test)]
#[path = "service_native_tests.rs"]
mod tests;

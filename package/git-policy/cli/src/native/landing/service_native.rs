//! What:
//!  The commits real Git builds for the commit service:
//! an editor,
//! a signature,
//! and every command line the service does
//!  not build itself.
//! Why:
//!  Git owns the editor,
//! the signer and the message semantics of whatever the service cannot embed,
//! so this module points
//!  real `git commit` at a thin scratch repository (its `HEAD` and branch at the base,
//! the policy-settled private index,
//!  the real configuration included,
//! every hook off) whose new objects go straight into the real object store.
//! Nothing is
//!  locked and nothing is registered:
//! the scratch repository sits inside the commit's own scratch directory,
//! is removed
//!  with it,
//! and the commit it makes is only an object until the service moves the branch to it by compare-and-swap
//!  (`doc/planning/cli-git-resident-service.md`,
//! "Commits Git builds").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { commit, parents } = await buildWithGit({ context, cwd, capture, scratch, commitIndex, arguments });
//! ```

use crate::capture_invocation::InvocationCapture;
use crate::commit_args::{resolve_private_commit_args, split_commit_invocation, without_all_flag};
use crate::commit_preparation::{PreparationRequest, read_prepared_commit, run_native_commit};
/// The build phase's stretches,
/// which the creation and Git's build mark as they go.
use crate::commit_build_part_durations::BuildPartClock;
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
    /// The commit,
    /// already in the real object store.
    pub commit: String,
    /// Its parents in order.
    pub parents: Vec<String>,
    /// Its message as stored:
    /// what the person typed,
    /// after Git's clean-up.
    pub message: String,
}

/// What:
///  Where the first blank line of a stored commit starts,
/// nothing when there is none.
/// Why:
///  The header block of a stored commit ends at it and the message follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// raw.indexOf('\n\n')
/// ```
fn blank_line_at(raw: &[u8]) -> Option<usize> {
    for (at, window) in raw.windows(2).enumerate() {
        if window == b"\n\n" {
            return Some(at);
        }
    }
    return None;
}

/// What:
///  The parent a header line names,
/// nothing for any other line.
/// Why:
///  `parents_of` lists the parents of a stored commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parentOfLine = (line: string) => line.startsWith('parent ') ? line.slice(7) : undefined;
/// ```
fn parent_of_line(line: &str) -> Option<String> {
    return line.strip_prefix("parent ").map(str::to_string);
}

/// What:
///  An argument as text,
/// with bytes that are not valid text replaced.
/// Why:
///  The commit options are read as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lossyText = (token: Buffer) => token.toString('utf8');
/// ```
fn lossy_text(token: &OsString) -> String {
    return token.to_string_lossy().into_owned();
}

/// The message after the header block of a stored commit.
pub fn message_of(raw: &[u8]) -> String {
    // The message starts after the first blank line; a stored commit without one has none.
    let start: usize = match blank_line_at(raw) {
        Some(at) => at + 2,
        None => raw.len(),
    };
    return String::from_utf8_lossy(&raw[start..]).into_owned();
}

/// Everything the build needs from the preparation that settled the private index.
pub struct BuildInput<'a> {
    /// How to run the real Git.
    pub context: &'a GitContext,
    /// The directory the command was given in.
    pub cwd: &'a Path,
    /// The invocation capture:
    /// the branch,
    /// the base,
    /// the object directory.
    pub capture: &'a InvocationCapture,
    /// The commit's own scratch directory,
    /// which holds the scratch repository and is removed with it.
    pub scratch: &'a Path,
    /// The private index the policies settled.
    pub commit_index: &'a Path,
    /// The arguments after the policies' fixed transforms,
    /// global options included.
    pub arguments: &'a [OsString],
    /// The positional pathspecs of the command line,
    /// which the private index already holds.
    pub pathspecs: &'a [String],
    /// Whether the line commits named paths or the index.
    pub mode: TransactionMode,
    /// Whether `-a` staged tracked changes into the private index.
    pub staged_all: bool,
    /// Whether the private index already holds a selection made with `--include`,
    /// `--patch` or `--interactive`,
    /// so that Git
    ///  is not given those options again.
    pub selected_privately: bool,
    /// The build phase's clock,
    /// which the shadow's creation and Git's build name their stretches under.
    pub clock: &'a mut BuildPartClock,
}

/// The parents named by a stored commit's header block.
pub fn parents_of(raw: &[u8]) -> Vec<String> {
    let header_end: usize = blank_line_at(raw).unwrap_or(raw.len());
    return String::from_utf8_lossy(&raw[..header_end])
        .lines()
        .filter_map(parent_of_line)
        .collect();
}

/// What:
///  The kept global options and the commit options of the arguments,
/// with the pathspecs and the selection options the
///  private index already holds removed.
/// Why:
///   Real `git commit` must commit the private index as it stands,
/// so it is given neither the paths nor `-a`.
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
    let text: Vec<String> = arguments.iter().map(lossy_text).collect();
    let private_args: Vec<String> = resolve_private_commit_args(text.as_slice(), pathspecs, mode, selected_privately);
    let owned: Vec<OsString> = private_args.iter().map(OsString::from).collect();
    let index: usize = global_layout(owned.as_slice()).prefix_len;
    let (globals, commit_args): (Vec<String>, Vec<String>) = split_commit_invocation(private_args.as_slice(), index);
    return if staged_all { (globals, without_all_flag(commit_args.as_slice())) } else { (globals, commit_args) };
}

/// What:
///  Runs real `git commit` against the scratch repository and reads the commit it made.
/// Why:
///   Git's own output,
/// editor and signer reach the person's terminal as they would from a plain `git commit`;
/// a nonzero
///  status of Git is the command's status and nothing lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await buildWithGit(input)
/// ```
///
/// # Errors
///
/// `TransactionStop::NativeExit` with Git's status when Git declined or was aborted (an empty message,
/// an editor that
///  failed,
/// a signer that failed);
/// `TransactionStop::Message` when Git could not be started.
pub fn build_with_git(input: &mut BuildInput<'_>) -> Result<GitBuilt, TransactionStop> {
    let shadow = input.scratch.join("shadow");
    // Named paths while another session's merge, cherry-pick or revert is unfinished: real Git refuses that as a partial commit,
    //  and a commit never fails for another session's unfinished work, so the scratch repository holds none of that state and
    //  builds an ordinary commit of the paths; the worktree's state stays for its owner to conclude on the new head.
    let git_dir: &Path = input.capture.layout.git_dir.as_path();
    let mut has_conclusion_state: bool = false;
    for name in ["MERGE_HEAD", "CHERRY_PICK_HEAD"] {
        if git_dir.join(name).exists() {
            has_conclusion_state = true;
            break;
        }
    }
    let carry_conclusion: bool = !(input.mode == TransactionMode::ExplicitPath && has_conclusion_state);
    // The creation names each of its own stretches, so the shadow is named here only as the surface they run on.
    create_shadow_repository_carrying(input.context, input.cwd, input.capture, shadow.as_path(), input.scratch, carry_conclusion, input.clock)?;
    // The wrapper adds no option of its own to the person's command line: Git prints what it prints, and `--quiet` reaches it only
    //  when the person typed it.
    let (globals, commit_args): (Vec<String>, Vec<String>) = private_invocation(input.arguments, input.pathspecs, input.mode, input.staged_all, input.selected_privately);
    let request: PreparationRequest<'_> = PreparationRequest {
        capture: input.capture,
        shadow: shadow.as_path(),
        commit_index: input.commit_index,
        scratch: input.scratch,
        global_args: globals.as_slice(),
        commit_args: commit_args.as_slice(),
        object_directory: Some(input.capture.layout.object_directory.as_path()),
    };
    input.clock.next("client build");
    let status: i32 = match run_native_commit(input.context, input.cwd, &request) {
        Ok(code) => code,
        Err(error) => return Err(TransactionStop::Message(error.to_string())),
    };
    if status != 0 {
        return Err(TransactionStop::NativeExit(status));
    }
    input.clock.next("commit read back");
    let prepared = read_prepared_commit(input.context, shadow.as_path())?;
    return Ok(GitBuilt { parents: parents_of(prepared.raw.as_slice()), message: message_of(prepared.raw.as_slice()), commit: prepared.oid });
}

/// Controls for building with Git stay out of the release executable.
#[cfg(test)]
#[path = "service_native_tests.rs"]
mod tests;

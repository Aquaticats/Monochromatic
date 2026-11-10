//! What:
//!  The transaction facts recorded before any Git mutation:
//!       the repository layout,
//!       the symbolic `HEAD` target,
//!       the conclusion kind,
//!       the ref storage backend,
//!       and the preparation base.
//! Why:
//!  Every later read of `HEAD` inside a transaction uses the recorded base or the landed commit,
//!  never live `HEAD`,
//!  because another landing may have moved it since the invocation started.
//!  The layout query and the base read run as separate steps,
//!  so pruning sees the transaction before the base is read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const layout = await captureInvocationLayout({ gitPath, cwd, amend });
//! const capture = await captureInvocationBase({ gitPath, cwd, layout });
//! ```

/// Journal enums shared with the transaction record:
///  the base,
/// the symbolic `HEAD`,
/// the conclusion and the ref backend.
use crate::transaction_journal::{Base, Conclusion, RefFormat, SymbolicHead};
/// Failure text of one Git start that the layout reports as unusable.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git, run_git_checked};
/// The files-backend `HEAD` reader and where one worktree's ref files live.
use crate::ref_files::{RefFilesLayout, read_symbolic_head};
/// The in-process repository layout,
/// tried before the `rev-parse` report.
use crate::repo_read_adapter::invocation_report;
use crate::git_system_config::with_real_git;
use crate::repo_read_layout::read_layout_in;
/// Proof that a sequencer marker names nothing,
/// without a Git process.
use crate::sequencer_marker::markers_maybe_present;
/// The empty tree's name per object format,
/// known without a Git process.
use crate::empty_tree::empty_tree_name;
/// Trim with JavaScript's whitespace rules,
/// as `String.prototype.trim` does.
use crate::js_text::trim_javascript;
/// Invocation start time,
/// ISO-8601 with milliseconds.
use crate::iso_time::now_iso;
/// The one configuration read the invocation's consumers share.
use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
/// `PathBuf` is an owned filesystem path.
use std::path::{Path, PathBuf};

/// What:
///  Facts about the repository one invocation selects,
///  recorded before the transaction publishes anything.
/// Why:
///   `InvocationLayout` in the incumbent:
///  every field except the preparation base.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InvocationLayout = Omit<InvocationCapture, 'base'>;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationLayout {
    /// Symbolic `HEAD` target at invocation.
    pub symbolic_head: SymbolicHead,
    /// Ref the landing advances by compare-and-swap:
    ///  the branch,
    ///  or `HEAD` when detached.
    pub target_ref: String,
    /// Commit kind deciding moved-target behavior.
    pub conclusion: Conclusion,
    /// Canonical worktree root.
    pub repository_root: PathBuf,
    /// Absolute Git directory of the owning worktree.
    pub git_dir: PathBuf,
    /// Absolute common Git directory.
    pub common_dir: PathBuf,
    /// Absolute real index path,
    ///  honoring a caller-set `GIT_INDEX_FILE`.
    pub real_index_path: PathBuf,
    /// Absolute real object directory named by the shadow's alternates.
    pub object_directory: PathBuf,
    /// Absolute per-worktree transaction registry.
    pub registry_root: PathBuf,
    /// Ref storage backend.
    pub ref_format: RefFormat,
    /// Object hash algorithm name.
    pub object_format: String,
    /// Empty tree of the repository's object format,
    ///  standing in for an unborn base.
    pub empty_tree_oid: String,
    /// ISO-8601 invocation start time.
    pub invoked_at: String,
    /// The Git configuration read once with the caller's global options,
    /// or `None` when Git must answer per key.
    pub config_snapshot: Option<ConfigSnapshot>,
}

/// What:
///  Every invocation fact,
/// including the preparation base.
/// Why:
///   `InvocationCapture` in the incumbent is the layout plus the base read after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InvocationCapture = InvocationLayout & { base: PreparationBase };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationCapture {
    /// Target commit at invocation,
    /// or unborn.
    pub base: Base,
    /// Facts other than the base.
    pub layout: InvocationLayout,
}

/// What:
///  One required Git report,
/// decoded and trimmed.
/// Why:
///   Git output is strict UTF-8 in the incumbent;
///  a non-UTF-8 report is a failure,
/// not a lossy text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder('utf-8', { fatal: true }).decode(bytes).trim()
/// ```
fn decoded_trimmed(output: &GitOutput) -> Result<String, GitFailure> {
    // `from_utf8` reports invalid bytes instead of replacing them.
    let text: &str = match std::str::from_utf8(output.stdout.as_slice()) {
        Ok(decoded) => decoded,
        Err(_) => return Err(GitFailure(String::from("Git returned non-UTF-8 output."))),
    };
    return Ok(String::from(trim_javascript(text)));
}

/// What:
///  A Git request in `cwd` that keeps none of the invocation's global options,
///  and still names the repository the caller named.
/// Why:
///   The incumbent runs these queries in the effective directory without the global options;
///  a repository whose Git directory is beside its work tree is found only when it is named,
///  so the selection travels with the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runTransactionGit({ gitPath, cwd, args, repository: selection({ cwd, globalPrefix }) })
/// ```
fn transaction_request(context: &GitContext, cwd: &Path, arguments: &[&str]) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, arguments);
    request.without_prefix = true;
    request.repository_selection = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    return request;
}

/// What:
///  Runs a query whose nonzero exit is an answer,
///  returning the output whatever its status.
/// Why:
///   `allowFailure: true` in the incumbent:
///  a missing ref or a detached `HEAD` is not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args, allowFailure: true })
/// ```
fn probe(context: &GitContext, cwd: &Path, arguments: &[&str]) -> Result<GitOutput, GitFailure> {
    return match run_git(context, &transaction_request(context, cwd, arguments)) {
        Ok(output) => Ok(output),
        Err(error) => Err(GitFailure(format!(
            "git {} could not start: {error}",
            arguments.join(" ")
        ))),
    };
}

/// What:
///  Runs a query whose failure is an error.
/// Why:
///   Matches the incumbent's default of throwing on a nonzero exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args })
/// ```
fn required(context: &GitContext, cwd: &Path, arguments: &[&str]) -> Result<GitOutput, GitFailure> {
    return run_git_checked(context, &transaction_request(context, cwd, arguments));
}

/// What:
///  Reads one required line of a fixed-order layout report.
/// Why:
///   An empty line means Git gave an incomplete layout,
///  which must stop the transaction before any mutation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// requiredLine({ lines, index })
/// ```
fn required_line(lines: &[&str], index: usize) -> Result<String, GitFailure> {
    // `get` returns `None` past the end, which is the same empty answer.
    let line: &str = lines.get(index).copied().unwrap_or("");
    if line.is_empty() {
        return Err(GitFailure(String::from("Git returned incomplete repository layout.")));
    }
    return Ok(String::from(line));
}

/// What:
///  Resolves one reported path against the invocation directory.
/// Why:
///   Git may report a relative path;
/// the transaction stores absolute ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAbsolute(reported) ? reported : resolve(cwd, reported)
/// ```
fn absolute_reported(cwd: &Path, lines: &[&str], index: usize) -> Result<PathBuf, GitFailure> {
    let reported: PathBuf = PathBuf::from(required_line(lines, index)?);
    return Ok(if reported.is_absolute() {
        reported
    } else {
        cwd.join(reported)
    });
}

/// What:
///  Reports whether a pseudoref names an object in the owning worktree.
/// Why:
///   `MERGE_HEAD`,
/// `CHERRY_PICK_HEAD` and `REVERT_HEAD` decide the conclusion kind
///  by resolution,
/// not by file presence,
/// so a reftable repository answers the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await runTransactionGit({ args: ['rev-parse', '--verify', '--quiet', name], allowFailure: true })).exitCode === 0
/// ```
fn pseudoref_exists(context: &GitContext, cwd: &Path, name: &str) -> Result<bool, GitFailure> {
    return Ok(probe(context, cwd, &["rev-parse", "--verify", "--quiet", name])?.code == Some(0));
}

/// What:
///  Resolves the commit kind from the amend flag and the owning worktree's conclusion state.
/// Why:
///   An amend always wins;
///  otherwise the first present marker in the incumbent's order decides.
///  A marker whose every lookup place is empty on the files backend is absent without asking Git;
///  any entry in any place,
/// of any content,
/// is resolved by `rev-parse --verify`,
///  so a marker naming a missing object stays a conclusion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolveConclusionKind({ gitPath, cwd, amend });
/// ```
pub fn resolve_conclusion_kind(
    context: &GitContext,
    cwd: &Path,
    amend: bool,
    ref_files: &RefFilesLayout<'_>,
) -> Result<Conclusion, GitFailure> {
    if amend {
        return Ok(Conclusion::Amend);
    }
    let names: [&str; 3] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"];
    let kinds: [Conclusion; 3] = [Conclusion::Merge, Conclusion::CherryPick, Conclusion::Revert];
    let maybe_present: Vec<bool> = markers_maybe_present(ref_files, &names);
    for ((name, kind), maybe) in names.into_iter().zip(kinds).zip(maybe_present) {
        if maybe && pseudoref_exists(context, cwd, name)? {
            return Ok(kind);
        }
    }
    return Ok(Conclusion::None);
}

/// What:
///  Resolves the symbolic `HEAD` target in the owning worktree.
/// Why:
///   The `HEAD` file answers when it is one of the two plain spellings on the files backend,
///  read afresh on every call so the landing's re-read stays a read at landing time;
///  anything else asks Git,
///  where exit 1 from `symbolic-ref --quiet` means detached and any other failure is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readHeadFile(refFiles) ?? (await resolveSymbolicHeadWithGit({ gitPath, cwd }));
/// ```
pub fn resolve_symbolic_head(
    context: &GitContext,
    cwd: &Path,
    ref_files: &RefFilesLayout<'_>,
) -> Result<SymbolicHead, GitFailure> {
    if let Some(read) = read_symbolic_head(ref_files) {
        return Ok(read);
    }
    let output: GitOutput = probe(context, cwd, &["symbolic-ref", "--quiet", "HEAD"])?;
    if output.code == Some(1) {
        return Ok(SymbolicHead::Detached);
    }
    if output.code != Some(0) {
        return Err(GitFailure(format!(
            "git symbolic-ref HEAD failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )));
    }
    return Ok(SymbolicHead::Branch(decoded_trimmed(&output)?));
}

/// What:
///  Resolves a ref to its commit,
///  or to an unborn base when it names nothing.
/// Why:
///   The preparation base is the target's commit at invocation,
///  and an unborn target starts from the empty tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolveRefCommit({ gitPath, cwd, ref });
/// ```
pub fn resolve_ref_commit(context: &GitContext, cwd: &Path, reference: &str) -> Result<Base, GitFailure> {
    let peeled: String = format!("{reference}^{{commit}}");
    let output: GitOutput = probe(context, cwd, &["rev-parse", "--verify", "--quiet", peeled.as_str()])?;
    if output.code != Some(0) {
        return Ok(Base::Unborn);
    }
    return Ok(Base::Commit(decoded_trimmed(&output)?));
}

/// What:
///  Reads the ref storage backend.
/// Why:
///   A Git that cannot report it (before 2.45) is the files backend.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolveRefFormat({ gitPath, cwd });
/// ```
pub fn resolve_ref_format(context: &GitContext, cwd: &Path) -> Result<RefFormat, GitFailure> {
    return resolve_ref_format_with(context, cwd, None);
}

/// What:
///  Reads the ref storage backend,
/// from the configuration snapshot when it proves `files`.
/// Why:
///   `rev-parse --show-ref-format` prints `reftable` only for a repository whose configuration names
///  `extensions.refStorage`,
/// so a snapshot read without global options that has no such key proves `files`
///  without starting Git;
/// any other snapshot,
/// and a missing one,
/// asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot?.provesFilesRefStorage() ? 'files' : await resolveRefFormat({ gitPath, cwd });
/// ```
pub fn resolve_ref_format_with(
    context: &GitContext,
    cwd: &Path,
    snapshot: Option<&ConfigSnapshot>,
) -> Result<RefFormat, GitFailure> {
    if snapshot.is_some_and(ConfigSnapshot::proves_files_ref_storage) {
        return Ok(RefFormat::Files);
    }
    let output: GitOutput = probe(context, cwd, &["rev-parse", "--show-ref-format"])?;
    let reftable: bool = output.code == Some(0) && decoded_trimmed(&output)? == "reftable";
    return Ok(if reftable { RefFormat::Reftable } else { RefFormat::Files });
}

/// What:
///  The empty tree's object name under the repository's object format.
/// Why:
///   The name is a constant per object format,
/// so a known format needs no Git process;
///  a format the table does not know is hashed by Git,
/// which stays the authority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// emptyTreeName(objectFormat) ?? (await runTransactionGit({ args: ['hash-object', '-t', 'tree', '--stdin'] })).trim()
/// ```
fn resolve_empty_tree(context: &GitContext, cwd: &Path, object_format: &str) -> Result<String, GitFailure> {
    if let Some(known) = empty_tree_name(object_format) {
        return Ok(String::from(known));
    }
    let hashed: GitOutput = required(context, cwd, &["hash-object", "-t", "tree", "--stdin"])?;
    return decoded_trimmed(&hashed);
}

/// What:
///  Captures every invocation fact except the preparation base,
///  before any Git mutation.
/// Why:
///   One layout request reports the paths,
///  and the independent probes follow it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await captureInvocationLayout({ gitPath, cwd, amend });
/// ```
pub fn capture_invocation_layout(
    context: &GitContext,
    cwd: &Path,
    amend: bool,
) -> Result<InvocationLayout, GitFailure> {
    // Taken before any Git runs, as the incumbent does.
    let invoked_at: String = now_iso();
    let config_snapshot: Option<ConfigSnapshot> = read_config_snapshot(context, cwd);
    // The readers answer for the directory they are given and cannot honour a repository the caller named, so a command line
    //  that names one asks Git.
    let named: bool = !crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice()).is_empty();
    // A plain layout is read from the file system; Git is asked only when the reader declines it.
    let read_in_process: Option<String> = if named {
        None
    } else {
        read_layout_in(cwd, with_real_git(context.overlay.as_slice(), context.real_git.as_path()).as_slice()).as_ref().and_then(invocation_report)
    };
    let text: String = match read_in_process {
        Some(read) => read,
        None => {
            let report: GitOutput = required(
                context,
                cwd,
                &[
                    "rev-parse",
                    "--path-format=absolute",
                    "--show-toplevel",
                    "--absolute-git-dir",
                    "--git-common-dir",
                    "--git-path",
                    "index",
                    "--git-path",
                    "objects",
                    "--git-path",
                    "cli-git-transactions",
                    "--show-object-format",
                ],
            )?;
            decoded_trimmed(&report)?
        }
    };
    let lines: Vec<&str> = text.split('\n').collect();
    let git_dir: PathBuf = absolute_reported(cwd, &lines, 1)?;
    let common_dir: PathBuf = absolute_reported(cwd, &lines, 2)?;
    let object_format: String = required_line(&lines, 6)?;
    // The ref storage backend is read first: the `HEAD` file only answers for the files backend.
    let ref_format: RefFormat = resolve_ref_format_with(context, cwd, config_snapshot.as_ref())?;
    let ref_files: RefFilesLayout<'_> = RefFilesLayout {
        git_dir: git_dir.as_path(),
        common_dir: common_dir.as_path(),
        ref_format,
        object_format: object_format.as_str(),
    };
    let symbolic_head: SymbolicHead = resolve_symbolic_head(context, cwd, &ref_files)?;
    let target_ref: String = match &symbolic_head {
        SymbolicHead::Branch(reference) => reference.clone(),
        SymbolicHead::Detached => String::from("HEAD"),
    };
    let empty_tree_oid: String = resolve_empty_tree(context, cwd, object_format.as_str())?;
    let repository_root: PathBuf = match std::fs::canonicalize(absolute_reported(cwd, &lines, 0)?) {
        Ok(resolved) => resolved,
        Err(error) => return Err(GitFailure(format!("repository root is unreadable: {error}"))),
    };
    return Ok(InvocationLayout {
        symbolic_head,
        target_ref,
        conclusion: resolve_conclusion_kind(context, cwd, amend, &ref_files)?,
        repository_root,
        git_dir,
        common_dir,
        real_index_path: absolute_reported(cwd, &lines, 3)?,
        object_directory: absolute_reported(cwd, &lines, 4)?,
        registry_root: absolute_reported(cwd, &lines, 5)?,
        ref_format,
        object_format,
        empty_tree_oid,
        invoked_at,
        config_snapshot,
    });
}

/// What:
///  Completes the capture with the preparation base:
///  the target's commit now.
/// Why:
///   Read after the layout and after the capture sequence is taken,
///  so the base is never older than the sequence the transaction records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await captureInvocationBase({ gitPath, cwd, layout });
/// ```
pub fn capture_invocation_base(
    context: &GitContext,
    cwd: &Path,
    layout: InvocationLayout,
) -> Result<InvocationCapture, GitFailure> {
    let base: Base = resolve_ref_commit(context, cwd, layout.target_ref.as_str())?;
    return Ok(InvocationCapture { base, layout });
}

/// What:
///  The revision the transaction reads as its baseline tree:
///  the base commit,
/// or the empty tree when unborn.
/// Why:
///   Every baseline read of the transaction uses this one revision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// baseRevision(capture) // capture.base.kind === 'commit' ? capture.base.oid : capture.emptyTreeOid
/// ```
pub fn base_revision(capture: &InvocationCapture) -> String {
    return match &capture.base {
        Base::Commit(oid) => oid.clone(),
        Base::Unborn => capture.layout.empty_tree_oid.clone(),
    };
}

/// Invocation capture controls stay out of the release executable.
#[cfg(test)]
#[path = "capture_invocation_tests.rs"]
mod tests;

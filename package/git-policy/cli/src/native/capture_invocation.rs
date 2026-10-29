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
//!  so pruning sees the transaction before the base is read
//!  (`src/policy-engine/commit-transaction-capture.ts`,
//!  `captureInvocationLayout` and `captureInvocationBase`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const layout = await captureInvocationLayout({ gitPath, cwd, amend });
//! const capture = await captureInvocationBase({ gitPath, cwd, layout });
//! ```

/// Journal enums shared with the transaction record:
///  the base, the symbolic `HEAD`, the conclusion and the ref backend.
use super::transaction_journal::{Base, Conclusion, RefFormat, SymbolicHead};
/// Failure text of one Git start that the layout reports as unusable.
use super::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git, run_git_checked};
/// Trim with JavaScript's whitespace rules, as `String.prototype.trim` does.
use super::js_text::trim_javascript;
/// Invocation start time, ISO-8601 with milliseconds.
use super::iso_time::now_iso;
/// The one configuration read the invocation's consumers share.
use super::config_snapshot::{ConfigSnapshot, read_config_snapshot};
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
    /// The Git configuration read once with the caller's global options, or `None` when Git must answer per key.
    pub config_snapshot: Option<ConfigSnapshot>,
}

/// What:
///  Every invocation fact, including the preparation base.
/// Why:
///   `InvocationCapture` in the incumbent is the layout plus the base read after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InvocationCapture = InvocationLayout & { base: PreparationBase };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationCapture {
    /// Target commit at invocation, or unborn.
    pub base: Base,
    /// Facts other than the base.
    pub layout: InvocationLayout,
}

/// What:
///  One required Git report, decoded and trimmed.
/// Why:
///   Git output is strict UTF-8 in the incumbent;
///  a non-UTF-8 report is a failure, not a lossy text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder('utf-8', { fatal: true }).decode(bytes).trim()
/// ```
fn decoded_trimmed(output: &GitOutput) -> Result<String, GitFailure> {
    // `from_utf8` reports invalid bytes instead of replacing them.
    let text: &str = std::str::from_utf8(output.stdout.as_slice())
        .map_err(|_| { return GitFailure(String::from("Git returned non-UTF-8 output.")); })?;
    return Ok(String::from(trim_javascript(text)));
}

/// What:
///  A Git request in `cwd` that ignores the invocation's global options,
///  as the incumbent's transaction Git does.
/// Why:
///   The incumbent runs these queries in the effective directory without the global options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runTransactionGit({ gitPath, cwd, args })
/// ```
fn transaction_request(cwd: &Path, arguments: &[&str]) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, arguments);
    request.without_prefix = true;
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
    return match run_git(context, &transaction_request(cwd, arguments)) {
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
    return run_git_checked(context, &transaction_request(cwd, arguments));
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
///   Git may report a relative path; the transaction stores absolute ones.
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
///   `MERGE_HEAD`, `CHERRY_PICK_HEAD` and `REVERT_HEAD` decide the conclusion kind
///  by resolution, not by file presence, so a reftable repository answers the same way.
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolveConclusionKind({ gitPath, cwd, amend });
/// ```
pub fn resolve_conclusion_kind(
    context: &GitContext,
    cwd: &Path,
    amend: bool,
) -> Result<Conclusion, GitFailure> {
    if amend {
        return Ok(Conclusion::Amend);
    }
    if pseudoref_exists(context, cwd, "MERGE_HEAD")? {
        return Ok(Conclusion::Merge);
    }
    if pseudoref_exists(context, cwd, "CHERRY_PICK_HEAD")? {
        return Ok(Conclusion::CherryPick);
    }
    if pseudoref_exists(context, cwd, "REVERT_HEAD")? {
        return Ok(Conclusion::Revert);
    }
    return Ok(Conclusion::None);
}

/// What:
///  Resolves the symbolic `HEAD` target in the owning worktree.
/// Why:
///   Exit 1 from `symbolic-ref --quiet` means detached;
///  any other failure is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolveSymbolicHead({ gitPath, cwd });
/// ```
pub fn resolve_symbolic_head(context: &GitContext, cwd: &Path) -> Result<SymbolicHead, GitFailure> {
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
///  Reads the ref storage backend, from the configuration snapshot when it proves `files`.
/// Why:
///   `rev-parse --show-ref-format` prints `reftable` only for a repository whose configuration names
///  `extensions.refStorage`, so a snapshot read without global options that has no such key proves `files`
///  without starting Git; any other snapshot, and a missing one, asks Git.
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
    let text: String = decoded_trimmed(&report)?;
    let lines: Vec<&str> = text.split('\n').collect();
    let symbolic_head: SymbolicHead = resolve_symbolic_head(context, cwd)?;
    let target_ref: String = match &symbolic_head {
        SymbolicHead::Branch(reference) => reference.clone(),
        SymbolicHead::Detached => String::from("HEAD"),
    };
    let empty_tree: GitOutput = required(context, cwd, &["hash-object", "-t", "tree", "--stdin"])?;
    let repository_root: PathBuf = std::fs::canonicalize(absolute_reported(cwd, &lines, 0)?)
        .map_err(|error| { return GitFailure(format!("repository root is unreadable: {error}")); })?;
    return Ok(InvocationLayout {
        symbolic_head,
        target_ref,
        conclusion: resolve_conclusion_kind(context, cwd, amend)?,
        repository_root,
        git_dir: absolute_reported(cwd, &lines, 1)?,
        common_dir: absolute_reported(cwd, &lines, 2)?,
        real_index_path: absolute_reported(cwd, &lines, 3)?,
        object_directory: absolute_reported(cwd, &lines, 4)?,
        registry_root: absolute_reported(cwd, &lines, 5)?,
        ref_format: resolve_ref_format_with(context, cwd, config_snapshot.as_ref())?,
        object_format: required_line(&lines, 6)?,
        empty_tree_oid: decoded_trimmed(&empty_tree)?,
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
///  the base commit, or the empty tree when unborn.
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

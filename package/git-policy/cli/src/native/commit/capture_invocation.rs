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
/// A reference name as the bytes Git printed it.
use crate::ref_name::RefName;
/// The in-process repository layout,
/// tried before the `rev-parse` report,
///  and the report shape it answers with.
use crate::repo_read_adapter::{InvocationReport, invocation_report};
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
/// The answers of one report,
///  and the pathname each of them names.
use crate::git_report_answers::{absolute_reported, answers_of, object_format_of};
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
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
    ///  or `HEAD` when detached,
    ///  by the bytes Git printed for its name.
    pub target_ref: RefName,
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
fn transaction_request<S: AsRef<OsStr>>(context: &GitContext, cwd: &Path, arguments: &[S]) -> GitRequest {
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
fn probe<S: AsRef<OsStr>>(context: &GitContext, cwd: &Path, arguments: &[S]) -> Result<GitOutput, GitFailure> {
    return match run_git(context, &transaction_request(context, cwd, arguments)) {
        Ok(output) => Ok(output),
        Err(error) => {
            // The words are a message here, so a name that is not text is shown as the shell reads it back.
            let shown: Vec<String> = arguments.iter().map(|word| { return crate::shell_quote::shell_quote(crate::path_bytes::argument_bytes(word.as_ref())); }).collect();
            Err(GitFailure(format!("git {} could not start: {error}", shown.join(" "))))
        }
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
fn required<S: AsRef<OsStr>>(context: &GitContext, cwd: &Path, arguments: &[S]) -> Result<GitOutput, GitFailure> {
    return run_git_checked(context, &transaction_request(context, cwd, arguments));
}

/// How many answers the one layout query prints:
/// six pathnames and the object format.
const LAYOUT_ANSWERS: usize = 6 + 1;

/// The `rev-parse` options of the one layout query,
/// in the order its answers are read.
const LAYOUT_ARGUMENTS: [&str; 11] = [
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
];

/// How many pathname answers the layout has.
const LAYOUT_PATHS: usize = 6;

/// What:
///  The pathnames of one layout asked for one at a time,
///  in the order the shared report prints them.
/// Why:
///   One question per pathname,
/// so a line feed inside a name cannot end any other answer;
///  `landing_location.rs` reads its two answers the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GIT_DIR_QUESTION = ['rev-parse', '--path-format=absolute', '--absolute-git-dir'];
/// ```
const PATH_QUESTIONS: [&[&str]; LAYOUT_PATHS] = [
    &["rev-parse", "--path-format=absolute", "--show-toplevel"],
    &["rev-parse", "--path-format=absolute", "--absolute-git-dir"],
    &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    &["rev-parse", "--path-format=absolute", "--git-path", "index"],
    &["rev-parse", "--path-format=absolute", "--git-path", "objects"],
    &["rev-parse", "--path-format=absolute", "--git-path", "cli-git-transactions"],
];

/// The question that prints the object format,
///  which is text and not a pathname.
const OBJECT_FORMAT_QUESTION: [&str; 2] = ["rev-parse", "--show-object-format"];

/// What:
///  The layout the seven answers of one report name.
/// Why:
///  The answers are in the order `LAYOUT_ARGUMENTS` asks for them,
///  and each path is the bytes Git printed for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ topLevel: absolute(cwd, answers[0]), gitDir: absolute(cwd, answers[1]), ... })
/// ```
fn report_of_answers(cwd: &Path, answers: &[&[u8]; LAYOUT_ANSWERS]) -> Result<InvocationReport, GitFailure> {
    return Ok(InvocationReport {
        top_level: absolute_reported(cwd, answers[0])?,
        git_dir: absolute_reported(cwd, answers[1])?,
        common_dir: absolute_reported(cwd, answers[2])?,
        index_path: absolute_reported(cwd, answers[3])?,
        object_directory: absolute_reported(cwd, answers[4])?,
        registry_root: absolute_reported(cwd, answers[5])?,
        object_format: object_format_of(answers[6])?,
    });
}

/// What:
///  Every pathname of the layout asked for on its own,
///  resolved against the invocation directory.
/// Why:
///  A report that cannot be split is the only caller of this,
/// which needs one question per pathname anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// PATH_QUESTIONS.map(async (question) => absolute(cwd, await ask(question)))
/// ```
fn absolute_answers(context: &GitContext, cwd: &Path) -> Result<[PathBuf; LAYOUT_PATHS], GitFailure> {
    let mut answers: Vec<PathBuf> = Vec::with_capacity(LAYOUT_PATHS);
    for question in PATH_QUESTIONS {
        let output: GitOutput = required(context, cwd, question)?;
        answers.push(absolute_reported(cwd, output.stdout.as_slice())?);
    }
    return match <[PathBuf; LAYOUT_PATHS]>::try_from(answers) {
        Ok(exactly) => Ok(exactly),
        Err(_) => Err(GitFailure(String::from("Git returned incomplete repository layout."))),
    };
}

/// What:
///  The layout of one repository,
///  with every pathname asked for on its own.
/// Why:
///  One report cannot be split once a pathname holds a line feed,
///  and a pathname that is not UTF-8 cannot be read out of text at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [topLevel, gitDir, commonDir, indexPath, objectDirectory, registryRoot] = await absoluteAnswers();
/// ```
fn layout_report_asked_for(context: &GitContext, cwd: &Path) -> Result<InvocationReport, GitFailure> {
    let [top_level, git_dir, common_dir, index_path, object_directory, registry_root] = absolute_answers(context, cwd)?;
    let output: GitOutput = required(context, cwd, OBJECT_FORMAT_QUESTION.as_slice())?;
    return Ok(InvocationReport {
        top_level,
        git_dir,
        common_dir,
        index_path,
        object_directory,
        registry_root,
        object_format: object_format_of(output.stdout.as_slice())?,
    });
}

/// What:
///  The layout Git reports for the invocation directory,
///  read as the bytes Git printed.
/// Why:
///  One query answers a plain layout;
///  a report with more answers than the layout has pathnames holds a pathname that carries the line feed separating two answers,
///  so no answer's place is known and each pathname is asked for on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = split(await ask(['rev-parse', ...LAYOUT_ARGUMENTS])); answers.length === 7 ? reportOf(answers) : askedFor()
/// ```
fn layout_report_from_git(context: &GitContext, cwd: &Path) -> Result<InvocationReport, GitFailure> {
    let mut arguments: Vec<&str> = vec!["rev-parse"];
    arguments.extend_from_slice(LAYOUT_ARGUMENTS.as_slice());
    let output: GitOutput = required(context, cwd, arguments.as_slice())?;
    let answers: Vec<&[u8]> = answers_of(output.stdout.as_slice());
    return match <[&[u8]; LAYOUT_ANSWERS]>::try_from(answers.as_slice()) {
        Ok(exactly) => report_of_answers(cwd, &exactly),
        Err(_) => layout_report_asked_for(context, cwd),
    };
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
    // The name is the bytes Git printed: a branch name is a byte string, and a reading that replaced a byte
    //  would name another ref (`ref_name.rs`).
    return Ok(SymbolicHead::Branch(crate::ref_name::RefName::from_git_output(output.stdout.as_slice())));
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
pub fn resolve_ref_commit(context: &GitContext, cwd: &Path, reference: &RefName) -> Result<Base, GitFailure> {
    // The question is the one spelling the in-process readers answer, `<ref>^{commit}`, and both halves are bytes.
    let mut peeled: OsString = reference.as_os_str().to_os_string();
    peeled.push("^{commit}");
    let output: GitOutput = probe(context, cwd, &[OsString::from("rev-parse"), OsString::from("--verify"), OsString::from("--quiet"), peeled])?;
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
    let read_in_process: Option<InvocationReport> = if named {
        None
    } else {
        read_layout_in(cwd, with_real_git(context.overlay.as_slice(), context.real_git.as_path()).as_slice()).as_ref().and_then(invocation_report)
    };
    let report: InvocationReport = match read_in_process {
        Some(read) => read,
        None => layout_report_from_git(context, cwd)?,
    };
    let git_dir: PathBuf = report.git_dir;
    let common_dir: PathBuf = report.common_dir;
    let object_format: String = report.object_format;
    // The ref storage backend is read first: the `HEAD` file only answers for the files backend.
    let ref_format: RefFormat = resolve_ref_format_with(context, cwd, config_snapshot.as_ref())?;
    let ref_files: RefFilesLayout<'_> = RefFilesLayout {
        git_dir: git_dir.as_path(),
        common_dir: common_dir.as_path(),
        ref_format,
        object_format: object_format.as_str(),
    };
    let symbolic_head: SymbolicHead = resolve_symbolic_head(context, cwd, &ref_files)?;
    let target_ref: RefName = match &symbolic_head {
        SymbolicHead::Branch(reference) => reference.clone(),
        SymbolicHead::Detached => RefName::from_text("HEAD"),
    };
    let empty_tree_oid: String = resolve_empty_tree(context, cwd, object_format.as_str())?;
    let repository_root: PathBuf = match std::fs::canonicalize(report.top_level.as_path()) {
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
        real_index_path: report.index_path,
        object_directory: report.object_directory,
        registry_root: report.registry_root,
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
    let base: Base = resolve_ref_commit(context, cwd, &layout.target_ref)?;
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

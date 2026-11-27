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

/// The one layout report of this capture,
///  asked of Git when the file system cannot answer for the directory.
use crate::capture_invocation_layout_report::layout_report_from_git;
/// The preparation base and the empty tree an unborn base starts from;
///  the symbolic `HEAD`,
///  the conclusion kind and the ref storage are re-exported below.
use crate::capture_invocation_ref_storage_and_base::{
    resolve_empty_tree,
    resolve_ref_commit,
};
/// The one configuration read the invocation's consumers share.
use crate::config_snapshot::{
    ConfigSnapshot,
    read_config_snapshot,
};
/// The Git start a repository layout is read with the real Git of the invocation named.
use crate::git_system_config::with_real_git;
/// Invocation start time,
/// ISO-8601 with milliseconds.
use crate::iso_time::now_iso;
/// Where one worktree's ref files live.
use crate::ref_files::RefFilesLayout;
/// A reference name as the bytes Git printed it.
use crate::ref_name::RefName;
/// The in-process repository layout,
/// tried before the `rev-parse` report,
///  and the report shape it answers with.
use crate::repo_read_adapter::{
    InvocationReport,
    invocation_report,
};
use crate::repo_read_layout::read_layout_in;
/// The transaction Git wrapper:
///  the context a question is asked in,
///  and the failure its answer is reported as.
use crate::transaction_git::{
    GitContext,
    GitFailure,
};
/// Journal enums shared with the transaction record:
///  the base,
/// the symbolic `HEAD`,
/// the conclusion and the ref backend.
use crate::transaction_journal::{
    Base,
    Conclusion,
    RefFormat,
    SymbolicHead,
};
/// `PathBuf` is an owned filesystem path.
use std::path::{
    Path,
    PathBuf,
};
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
///  The four answers of this capture that its callers name by this module,
///  re-exported where they already look for them.
/// Why:
///  The questions moved to files of this subject's own,
///  and a caller of `capture_invocation::` keeps its path,
///  so the move changes no call site.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { resolveConclusionKind, resolveSymbolicHead } from './head-and-conclusion';
/// export { resolveRefFormatWith } from './ref-storage-and-base';
/// ```
pub use crate::capture_invocation_head_and_conclusion::{
    resolve_conclusion_kind,
    resolve_symbolic_head,
};
/// The ref storage backend read where a configuration snapshot cannot prove `files`,
///  re-exported where every caller of it already looks.
pub use crate::capture_invocation_ref_storage_and_base::resolve_ref_format_with;
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
    let named: bool =
        !crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice())
            .is_empty();
    // A plain layout is read from the file system; Git is asked only when the reader declines it.
    let read_in_process: Option<InvocationReport> = if named {
        None
    } else {
        read_layout_in(
            cwd,
            with_real_git(context.overlay.as_slice(), context.real_git.as_path()).as_slice(),
        )
        .as_ref()
        .and_then(invocation_report)
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
        Err(error) => {
            return Err(GitFailure(format!(
                "repository root is unreadable: {error}"
            )));
        }
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
    return Ok(InvocationCapture {
        base,
        layout,
    });
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

#[cfg(test)]
#[path = "capture_invocation_tests.rs"]
mod tests;

//! What:
//!  The values every commit of the service route shares: the invocation, how it can stop, what it came to, the selection
//!  the command line makes, and the policy checks over the private index.
//! Why:
//!  The preparation (`service_prepare.rs`) and the commit command (`commit_command.rs`) pass these between them. The
//!  transaction that once landed a commit itself, with its registry, recovery and replay, is gone: the resident service is the
//!  only writer of a commit (`doc/planning/cli-git-resident-service.md`, "Landing every command line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const selection = decideSelection(region, conclusion);
//! ```

use super::candidate_object::{ObjectId, parse_object_id};
use super::candidate_prediction::CandidateRequest;
use super::capture_invocation::InvocationCapture;
use super::command_commit::CommitRegion;
use super::commit_facts::PrivateIndexFacts;
use super::config_schema::CliGitConfig;
use super::diagnostics::EngineFailureCode;
use super::path_bytes::os_from_path_bytes;
use super::markdown_linter::embedded_linter;
use super::policy_checks::{ShippedChecks, shipped_checks};
use super::policy_forbidden_strings::default_scanner_settings;
use super::policy_markdown::markdown_state;
use super::policy_content::LifecycleContent;
use super::policy_convergence::Convergence;
use super::policy_events::PolicyEvent;
use super::repository_facts::RepositoryFacts;
use super::transaction_git::{GitContext, GitFailure};
use super::transaction_journal::{Base, Conclusion, TransactionMode};
use super::transaction_phase::PhaseMarkerError;
use super::wrapper_controls::Controls;
use std::ffi::OsString;
use std::path::{Path, PathBuf};


/// What:
///  Everything the invocation fixes before the transaction starts.
/// Why:
///   The steps share one identity, one parsed commit region and one configuration; passing them together keeps a
///   step from reading another invocation's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitInvocation = { args; gitPath; cwd; region; policyOptions; concurrency };
/// ```
pub struct CommitInvocation<'a> {
    /// Real Git, the child environment overlay, and the invocation's global options.
    pub context: &'a GitContext,
    /// The invocation's environment, read for leases and inherited configuration.
    pub environment: &'a [(OsString, OsString)],
    /// The effective invocation directory (the process directory moved by every `-C`).
    pub cwd: &'a Path,
    /// The arguments with the wrapper's controls removed.
    pub arguments: &'a [OsString],
    /// The index of `commit` within `arguments`.
    pub subcommand_index: usize,
    /// The commit options Git's table read.
    pub region: &'a CommitRegion,
    /// The configuration of the worktree.
    pub config: &'a CliGitConfig,
    /// The wrapper controls given on the command line.
    pub controls: &'a Controls,
}

/// What:
///  Why the transaction stopped without a policy result.
/// Why:
///   The incumbent's boundary maps each to a different ending: Git's own exit code with no wrapper text, a message
///   with exit status 1, a finding with exit status 2, or a verbatim stop text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// NativeCommitFailedError | CommitTransactionGitError | IndexLockUnprovenOwnerError | Error
/// ```
#[derive(Debug)]
pub enum TransactionStop {
    /// Native `git commit` exited nonzero during preparation: its status is the invocation's, and Git printed why.
    NativeExit(i32),
    /// A Git step failed: the message reaches standard error and the status is 1.
    Message(String),
    /// A transaction failure reported as an engine failure event with status 2.
    Failed {
        /// The failure's stable code.
        code: EngineFailureCode,
        /// What went wrong.
        message: String,
    },
}

impl From<GitFailure> for TransactionStop {
    /// A Git step failure keeps its message and ends the command with status 1.
    fn from(failure: GitFailure) -> Self {
        return TransactionStop::Message(failure.0);
    }
}


impl From<PhaseMarkerError> for TransactionStop {
    /// A malformed test marker fails the transaction.
    fn from(failure: PhaseMarkerError) -> Self {
        return failed(failure.to_string());
    }
}


/// What:
///  A transaction failure with the generic code.
/// Why:
///   Most unexpected failures are reported the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// initialTransactionFailure({ args, message })
/// ```
pub fn failed(message: String) -> TransactionStop {
    return TransactionStop::Failed {
        code: EngineFailureCode::TransactionFailed,
        message,
    };
}

/// What:
///  What the transaction decided.
/// Why:
///   The command prints the events, runs the post-commit lifecycle for a landed commit, and exits with the code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitTransactionResult = { policyResult: PolicyEngineResult; committed: boolean; landedOid?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionResult {
    /// The events the invocation reports, in order.
    pub events: Vec<PolicyEvent>,
    /// The exit status when nothing landed; 0 after a landed commit.
    pub exit_code: i32,
    /// The landed commit, when the transaction landed one.
    pub landed_oid: Option<String>,
    /// Whether the Git configuration read when the commit began holds LFS keys, so the commit's auto-push must upload
    ///  LFS objects first; `false` when nothing landed.
    pub uses_lfs: bool,
}

/// What:
///  The commit-region facts the transaction decides from, derived once.
/// Why:
///   The mode and the selection flags are read in several steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { concludesSequencer, readOnlySelection, selectsPaths, stagedAll, mode } = ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    /// Whether a pathless commit concludes a merge, cherry-pick or revert.
    pub concludes_sequencer: bool,
    /// Whether the selection UI (include, interactive, patch) leaves the policies read-only.
    pub read_only: bool,
    /// Whether any path was selected.
    pub selects_paths: bool,
    /// Whether `-a` stages privately.
    pub staged_all: bool,
    /// The private index mode.
    pub mode: TransactionMode,
}

/// What:
///  Decide the selection facts of one commit invocation.
/// Why:
///   Invalid combinations reach native Git unchanged, so it rejects them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mode = region.hasNoOnlyFlag || concludesSequencer || readOnlySelection || region.hasAllFlag || (!selectsPaths) ? 'index' : 'explicit-path';
/// ```
pub fn decide_selection(region: &CommitRegion, conclusion: Conclusion) -> Selection {
    return decide_selection_naming(region, conclusion, false);
}

/// What:
///  `decide_selection`, told whether the paths the commit names include one with an unmerged index entry.
/// Why:
///   A path in conflict cannot be committed: such a command keeps concluding the sequencer, so real Git's own refusal reaches
///  the person, in its words, and nothing is committed with conflict markers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideSelectionNaming(region, conclusion, namesUnmerged)
/// ```
pub fn decide_selection_naming(region: &CommitRegion, conclusion: Conclusion, names_unmerged: bool) -> Selection {
    let read_only: bool = region.include || region.interactive || region.patch;
    let selects_paths: bool = !region.pathspecs.is_empty() || region.pathspec_from_file.is_some();
    // Named paths never conclude another session's unfinished merge or stopped cherry-pick: real Git refuses them there as a
    //  partial commit, and the commit lands as an ordinary commit of those paths while the state is carried on (`SPEC.md`,
    //  "Merges and commits never fail"). Real Git takes named paths during a revert and concludes it with them, which stays.
    let paths_only: bool = selects_paths && !read_only && !region.all && region.only != Some(false);
    let refused_by_git: bool = matches!(conclusion, Conclusion::Merge | Conclusion::CherryPick) && !names_unmerged;
    let concludes_sequencer: bool = conclusion != Conclusion::None && conclusion != Conclusion::Amend && !(paths_only && refused_by_git);
    let staged_all: bool = region.all && !selects_paths && !read_only && region.only != Some(true);
    let index_mode: bool = region.only == Some(false)
        || concludes_sequencer
        || read_only
        || region.all
        || !selects_paths;
    return Selection {
        concludes_sequencer,
        read_only,
        selects_paths,
        staged_all,
        mode: if index_mode { TransactionMode::Index } else { TransactionMode::ExplicitPath },
    };
}

/// What:
///  Rebuild the shipped checks over the private commit index: rule facts stay real, the candidates come from the
///  private index against the recorded base.
/// Why:
///   The incumbent's rules ask real Git about the real index, while the content policies read what the transaction
///   corrects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = createPrivateIndexFacts({ gitPath, cwd, indexPath, paths, baseRevision, objectDirectory });
/// ```
pub fn private_checks<'a, F: RepositoryFacts>(
    checks: &'a mut ShippedChecks<F>,
    index: &Path,
    object_directory: &Path,
    candidates: CandidateRequest,
) -> ShippedChecks<PrivateIndexFacts<&'a mut F>> {
    // The scanner settings and the Markdown linter move to the rebuilt checks; the originals are placeholders that
    // nothing reads again, because the rebuilt checks run every policy of this transaction.
    let scanner_settings = std::mem::replace(&mut checks.scanner_settings, default_scanner_settings());
    let markdown = std::mem::replace(&mut checks.markdown, markdown_state(Box::new(embedded_linter())));
    let arguments: Vec<OsString> = checks.arguments.clone();
    let allowed_worktree_dirs: Vec<PathBuf> = checks.allowed_worktree_dirs.clone();
    let mut rebuilt: ShippedChecks<PrivateIndexFacts<&'a mut F>> = shipped_checks(
        PrivateIndexFacts::new(&mut checks.facts, index, object_directory),
        arguments,
        LifecycleContent::Requested(candidates),
        allowed_worktree_dirs,
    );
    rebuilt.scanner_settings = scanner_settings;
    rebuilt.markdown = markdown;
    return rebuilt;
}

/// What:
///  How many passes changed the state when the convergence settled; any other ending counts none.
/// Why:
///   The fix summary names the passes of a settled convergence only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// convergence.kind === 'settled' ? convergence.changedPasses : 0
/// ```
pub fn settled_passes(convergence: &Convergence) -> u64 {
    return match convergence {
        Convergence::Settled { changed_passes } => *changed_passes as u64,
        _ => 0,
    };
}

/// What:
///  The candidate request for the paths a private index changes against the recorded base.
/// Why:
///   The policies check exactly the paths the commit changes; each path is literal and rooted at the top level, so a
///   name with glob characters or a directory below the current one still names one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createPrivateIndexFacts({ paths: candidatePaths, baseRevision })
/// ```
pub fn staged_at_base(base: Option<ObjectId>, paths: &[Vec<u8>]) -> CandidateRequest {
    return CandidateRequest::StagedAtBase {
        base,
        pathspecs: paths
            .iter()
            .map(|path| {
                let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
                pathspec.extend_from_slice(path.as_slice());
                return os_from_path_bytes(pathspec.as_slice());
            })
            .collect(),
    };
}

/// What:
///  The recorded base as an object ID, or nothing for an unborn branch.
/// Why:
///   The candidate listing compares the private index with this commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// capture.base.kind === 'commit' ? capture.base.oid : undefined
/// ```
pub fn base_object(capture: &InvocationCapture) -> Result<Option<ObjectId>, TransactionStop> {
    return match &capture.base {
        Base::Unborn => Ok(None),
        Base::Commit(oid) => match parse_object_id(oid.as_bytes()) {
            Some(found) => Ok(Some(found)),
            None => Err(failed(format!("the preparation base {oid} is not an object name"))),
        },
    };
}

/// What:
///  Whether any pathspec of a commit could name one of the unmerged paths, judged without expanding any pathspec.
/// Why:
///   An unmerged path must never be committed by a commit that lands beside an unfinished merge. A pathspec that names a path
///  or a directory above it says yes; a pathspec the check cannot read exactly (a glob, a magic prefix, a path outside the
///  current directory, a pathspec file) says yes whenever anything is unmerged, which keeps Git's refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesAnUnmergedPath(unmerged, pathspecs, fromFile)
/// ```
pub fn names_an_unmerged_path(unmerged: &[Vec<u8>], pathspecs: &[Vec<u8>], from_file: bool) -> bool {
    if unmerged.is_empty() {
        return false;
    }
    if from_file || pathspecs.is_empty() {
        return true;
    }
    return pathspecs.iter().any(|spec| {
        let inexact: bool = spec.is_empty()
            || spec.iter().any(|byte| { return matches!(*byte, b'*' | b'?' | b'[' | b'\\' | b':'); })
            || spec.starts_with(b"/")
            || spec.starts_with(b"..")
            || spec.as_slice() == b".";
        if inexact {
            return true;
        }
        let directory: &[u8] = spec.strip_suffix(b"/").unwrap_or(spec.as_slice());
        let above: Vec<u8> = [directory, b"/".as_slice()].concat();
        return unmerged.iter().any(|path| { return path.as_slice() == directory || path.starts_with(above.as_slice()); });
    });
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_transaction_tests.rs"]
mod tests;

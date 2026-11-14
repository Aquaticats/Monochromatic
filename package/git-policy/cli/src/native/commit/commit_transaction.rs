//! What:
//!  The values every commit of the service route shares:
//! the invocation,
//! how it can stop,
//! what it came to,
//! the selection
//!  the command line makes,
//! and the policy checks over the private index.
//! Why:
//!  The preparation (`service_prepare.rs`) and the commit command (`commit_command.rs`) pass these between them.
//! The
//!  transaction that once landed a commit itself,
//! with its registry,
//! recovery and replay,
//! is gone:
//! the resident service is the
//!  only writer of a commit (`doc/planning/cli-git-resident-service.md`,
//! "Landing every command line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const selection = decideSelection(region, conclusion);
//! ```

use crate::candidate_object::{ObjectId, parse_object_id};
use crate::candidate_prediction::CandidateRequest;
use crate::capture_invocation::InvocationCapture;
use crate::command_commit::CommitRegion;
use crate::commit_facts::PrivateIndexFacts;
use crate::config_schema::CliGitConfig;
use crate::diagnostics::EngineFailureCode;
use crate::effective_target::ToolCacheDirectory;
use crate::path_bytes::os_from_path_bytes;
use crate::markdown_linter::embedded_linter;
use crate::policy_checks::{ShippedChecks, shipped_checks};
use crate::policy_forbidden_strings::default_scanner_settings;
use crate::policy_markdown::markdown_state;
use crate::policy_content::LifecycleContent;
use crate::policy_convergence::Convergence;
use crate::policy_events::PolicyEvent;
use crate::repository_facts::RepositoryFacts;
use crate::transaction_git::{GitContext, GitFailure};
use crate::transaction_journal::{Base, Conclusion, TransactionMode};
use crate::transaction_phase::PhaseMarkerError;
use crate::wrapper_controls::Controls;
use std::ffi::OsString;
use std::path::Path;


/// What:
///  Everything the invocation fixes before the transaction starts.
/// Why:
///   The steps share one identity,
/// one parsed commit region and one configuration;
/// passing them together keeps a
///   step from reading another invocation's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitInvocation = { args; gitPath; cwd; region; policyOptions; concurrency };
/// ```
pub struct CommitInvocation<'a> {
    /// Real Git,
    /// the child environment overlay,
    /// and the invocation's global options.
    pub context: &'a GitContext,
    /// The invocation's environment,
    /// read for leases and inherited configuration.
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
///   The incumbent's boundary maps each to a different ending:
/// Git's own exit code with no wrapper text,
/// a message
///   with exit status 1,
/// a finding with exit status 2,
/// or a verbatim stop text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// NativeCommitFailedError | CommitTransactionGitError | IndexLockUnprovenOwnerError | Error
/// ```
#[derive(Debug)]
pub enum TransactionStop {
    /// Native `git commit` exited nonzero during preparation:
    /// its status is the invocation's,
    /// and Git printed why.
    NativeExit(i32),
    /// A Git step failed:
    /// the message reaches standard error and the status is 1.
    Message(String),
    /// What:
    ///  The command itself decided that this commit cannot be made,
    ///  and the failure carries both the words the person reads and the name the record states.
    /// Why:
    ///  A failure the client decided is one the client can name:
    ///  a service of another release that would not stop,
    ///  a service manager that answered nothing within its bound,
    ///  a service that refused the request.
    ///  Carrying the failure rather than the words it rendered is what lets the timing record say which one it was,
    ///  where a `Message` of the wrapper's own still needs a separate answer (`commit_stop_reason.rs`).
    Client(crate::service_messages::ClientFailure),
    /// A transaction failure reported as an engine failure event with status 2.
    Failed {
        /// The failure's stable code.
        code: EngineFailureCode,
        /// What went wrong.
        message: String,
    },
    /// The person stopped the command with a signal while it waited for the commit service:
    /// the message reaches standard error and the command ends the way that signal ends a program.
    EndedBySignal {
        /// The signal the person sent.
        signal: i32,
        /// The process that sent it,
        /// where the kernel says which one did;
        /// nothing for a signal a terminal or the kernel raised.
        sender: Option<i32>,
        /// What the command says before it ends.
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
///  A stop the command decided,
/// from the failure whose words the person reads.
/// Why:
///  Every place that ends a commit with a `ClientFailure` builds it this way,
///  so the failure reaches both ends of the command:
///  the words standard error shows and the name the record keeps (`service_messages::failure_code`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clientStop = (failure) => new TransactionStop(failure);
/// ```
pub fn client_stop(failure: crate::service_messages::ClientFailure) -> TransactionStop {
    return TransactionStop::Client(failure);
}

/// What:
///  What the transaction decided.
/// Why:
///   The command prints the events,
/// runs the post-commit lifecycle for a landed commit,
/// and exits with the code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitTransactionResult = { policyResult: PolicyEngineResult; committed: boolean; landedOid?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionResult {
    /// The events the invocation reports,
    /// in order.
    pub events: Vec<PolicyEvent>,
    /// The exit status when nothing landed;
    /// 0 after a landed commit.
    pub exit_code: i32,
    /// The landed commit,
    /// when the transaction landed one.
    pub landed_oid: Option<String>,
    /// Whether the Git configuration read when the commit began holds LFS keys,
    /// so the commit's auto-push must upload
    ///  LFS objects first;
    /// `false` when nothing landed.
    pub uses_lfs: bool,
}

/// What:
///  The commit-region facts the transaction decides from,
/// derived once.
/// Why:
///   The mode and the selection flags are read in several steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { concludesSequencer, readOnlySelection, selectsPaths, stagedAll, mode } = ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    /// Whether a pathless commit concludes a merge,
    /// cherry-pick or revert.
    pub concludes_sequencer: bool,
    /// Whether the selection UI (include,
    /// interactive,
    /// patch) leaves the policies read-only.
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
///   Invalid combinations reach native Git unchanged,
/// so it rejects them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mode = region.hasNoOnlyFlag || concludesSequencer || readOnlySelection || region.hasAllFlag || (!selectsPaths) ? 'index' : 'explicit-path';
/// ```
pub fn decide_selection(region: &CommitRegion, conclusion: Conclusion) -> Selection {
    let read_only: bool = region.include || region.interactive || region.patch;
    let selects_paths: bool = !region.pathspecs.is_empty() || region.pathspec_from_file.is_some();
    // Named paths never conclude another session's unfinished merge or stopped cherry-pick: real Git refuses them there as a
    //  partial commit, and the commit lands as an ordinary commit of those paths while the state is carried on (`SPEC.md`,
    //  "Merges and commits never fail"). Real Git takes named paths during a revert and concludes it with them, which stays.
    let paths_only: bool = selects_paths && !read_only && !region.all && region.only != Some(false);
    let refused_by_git: bool = matches!(conclusion, Conclusion::Merge | Conclusion::CherryPick);
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
///  Rebuild the shipped checks over the private commit index:
/// rule facts stay real,
/// the candidates come from the
///  private index against the recorded base.
/// Why:
///   The incumbent's rules ask real Git about the real index,
/// while the content policies read what the transaction
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
    real_index: &Path,
    candidates: CandidateRequest,
) -> ShippedChecks<PrivateIndexFacts<&'a mut F>> {
    // The scanner settings and the Markdown linter move to the rebuilt checks; the originals are placeholders that
    // nothing reads again, because the rebuilt checks run every policy of this transaction.
    let scanner_settings = std::mem::replace(&mut checks.scanner_settings, default_scanner_settings());
    let markdown = std::mem::replace(&mut checks.markdown, markdown_state(Box::new(embedded_linter())));
    let arguments: Vec<OsString> = checks.arguments.clone();
    let allowed_worktree_dirs: Vec<ToolCacheDirectory> = checks.allowed_worktree_dirs.clone();
    let mut rebuilt: ShippedChecks<PrivateIndexFacts<&'a mut F>> = shipped_checks(
        PrivateIndexFacts::new(&mut checks.facts, index, object_directory),
        arguments,
        LifecycleContent::Requested(candidates),
        allowed_worktree_dirs,
    );
    rebuilt.scanner_settings = scanner_settings;
    rebuilt.markdown = markdown;
    // A policy of this commit that adds a path compares the real index with the commit's own entries before it writes one.
    rebuilt.real_index = Some(real_index.to_path_buf());
    return rebuilt;
}

/// What:
///  How many passes changed the state when the convergence settled;
/// any other ending counts none.
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
///   The policies check exactly the paths the commit changes;
/// each path is literal and rooted at the top level,
/// so a
///   name with glob characters or a directory below the current one still names one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createPrivateIndexFacts({ paths: candidatePaths, baseRevision })
/// ```
pub fn staged_at_base(base: Option<ObjectId>, paths: &[Vec<u8>]) -> CandidateRequest {
    let mut pathspecs: Vec<OsString> = Vec::new();
    for path in paths {
        let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
        pathspec.extend_from_slice(path.as_slice());
        pathspecs.push(os_from_path_bytes(pathspec.as_slice()));
    }
    return CandidateRequest::StagedAtBase {
        base,
        pathspecs,
    };
}

/// What:
///  The recorded base as an object ID,
/// or nothing for an unborn branch.
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

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_transaction_tests.rs"]
mod tests;

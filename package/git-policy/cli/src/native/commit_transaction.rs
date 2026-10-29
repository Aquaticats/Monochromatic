//! What:
//!  The commit transaction in the incumbent's order, up to the settled policies:
//!  layout capture, the published workspace, the preparation base, `preparing.json`, the shadow repository, the
//!  private index under capture order, interactive selection, and the bounded policy convergence.
//! Why:
//!  The incumbent (`src/policy-engine/commit-transaction.ts`) captures before it mutates, publishes the directory
//!  before the base is read, and takes the landing lock only inside the landing attempt, never through preparation:
//!  holding it here would serialize concurrent commits.
//!  The conclusion (`commit_conclusion.rs`) takes the settled state to a landed commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await runCommitTransaction({ args, gitPath, policyOptions, concurrency });
//! ```

use super::candidate_object::{ObjectId, parse_object_id};
use super::candidate_prediction::CandidateRequest;
use super::capture_invocation::{
    InvocationCapture, InvocationLayout, base_revision, capture_invocation_base, capture_invocation_layout,
};
use super::capture_order::{CaptureIndexes, CaptureOrderError, capture_commit_content};
use super::capture_store::read_next_capture_sequence;
use super::command_commit::CommitRegion;
use super::command_options::OptionValue;
use super::command_options_query::value_bytes;
use super::commit_conclusion::{ConclusionInput, conclude_commit_transaction};
use super::commit_convergence::{CommitConvergence, CommitConvergenceInput, converge_commit_policies};
use super::commit_facts::PrivateIndexFacts;
use super::commit_index::{
    IndexSelection, initialize_commit_index, list_changed_index_paths, list_unmerged_index_paths,
    materialize_pathspec_file, prepare_interactive_selection,
};
use super::commit_staged_deletions::{
    STAGED_DELETION_CODE, STAGED_DELETION_CORE_ID, staged_deletion_message, staged_deletions_present_on_disk,
};
use super::commit_workspace::{CommitWorkspace, WorkspaceError, create_commit_workspace};
use super::config_schema::CliGitConfig;
use super::diagnostic_log::debug;
use super::diagnostics::EngineFailureCode;
use super::landing_attempt::LandingError;
use super::path_bytes::{os_from_path_bytes, path_message_text};
use super::markdown_linter::executable_linter;
use super::policy_checks::{ShippedChecks, shipped_checks};
use super::policy_forbidden_strings::default_scanner_settings;
use super::policy_markdown::markdown_state;
use super::policy_content::LifecycleContent;
use super::policy_convergence::Convergence;
use super::policy_engine::{StageRequest, pass_exit_code};
use super::policy_events::PolicyEvent;
use super::policy_pass::{PassResult, run_policy_pass};
use super::policy_trigger::Trigger;
use super::private_storage::{sync_directory, write_private_file};
use super::repository_facts::RepositoryFacts;
use super::transaction_git::{GitContext, GitFailure};
use super::transaction_journal::{Base, Conclusion, PREPARING_FILENAME, PreparingRecord, TransactionMode};
use super::traditional_hook_warning::warn_traditional_hooks;
use super::transaction_journal_encode::encode_preparing;
use super::transaction_phase::PhaseMarkerError;
use super::wrapper_controls::Controls;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "runCommitTransaction";

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
    /// Whether this invocation runs under a valid outer preparation lease.
    pub inherited_lease_valid: bool,
    /// This executable, which the hook entries link to.
    pub executable: &'a Path,
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

impl From<WorkspaceError> for TransactionStop {
    /// An unpublishable workspace fails the transaction.
    fn from(failure: WorkspaceError) -> Self {
        return failed(failure.to_string());
    }
}

impl From<CaptureOrderError> for TransactionStop {
    /// A capture failure fails the transaction.
    fn from(failure: CaptureOrderError) -> Self {
        return match failure {
            CaptureOrderError::Git(git) => TransactionStop::Message(git.0),
            other => failed(other.to_string()),
        };
    }
}

impl From<PhaseMarkerError> for TransactionStop {
    /// A malformed test marker fails the transaction.
    fn from(failure: PhaseMarkerError) -> Self {
        return failed(failure.to_string());
    }
}

impl From<LandingError> for TransactionStop {
    /// A failed landing attempt: an unproven lock owner has its own code, the rest are transaction failures.
    fn from(failure: LandingError) -> Self {
        return match failure {
            LandingError::Lock(super::index_writer_coordination::CoordinationFailure::UnprovenOwner(text)) => {
                TransactionStop::Failed {
                    code: EngineFailureCode::IndexLockUnprovenOwner,
                    message: text,
                }
            }
            LandingError::Git(git) => TransactionStop::Message(git.0),
            other => failed(other.to_string()),
        };
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
///  The transaction directory's names, derived from the registry root and the transaction ID.
/// Why:
///   Callers that build facts before the transaction creates anything use the same names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// transactionPaths({ registryRoot, commonDir, transactionId });
/// ```
pub fn transaction_directory(registry_root: &Path, transaction_id: &str) -> PathBuf {
    return registry_root.join(transaction_id);
}

/// What:
///  Write one journal record durably: exclusively, then the directory entry.
/// Why:
///   A state file is never rewritten, so a crash leaves the newest complete state readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeJournalRecord({ directory, filename, record });
/// ```
pub fn write_journal_record(directory: &Path, filename: &str, text: &str) -> Result<(), TransactionStop> {
    let path: PathBuf = directory.join(filename);
    write_private_file(path.as_path(), text.as_bytes())
        .map_err(|error| { return failed(format!("writing {} failed: {error}", path.display())); })?;
    sync_directory(directory)
        .map_err(|error| { return failed(format!("syncing {} failed: {error}", directory.display())); })?;
    return Ok(());
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
    let concludes_sequencer: bool = conclusion != Conclusion::None && conclusion != Conclusion::Amend;
    let read_only: bool = region.include || region.interactive || region.patch;
    let selects_paths: bool = !region.pathspecs.is_empty() || region.pathspec_from_file.is_some();
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
///  The positional pathspec tokens of the commit region, exactly as the command line gave them.
/// Why:
///   The private index selection and the journal must hand Git the same bytes the person typed,
///  including a name that is not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// region.pathspecs
/// ```
fn pathspec_arguments(invocation: &CommitInvocation<'_>) -> Vec<OsString> {
    let region_arguments: &[OsString] = &invocation.arguments[invocation.subcommand_index + 1..];
    return invocation
        .region
        .pathspecs
        .iter()
        .map(|index| { return region_arguments[*index].clone(); })
        .collect();
}

/// What:
///  The positional pathspec tokens of the commit region as text.
/// Why:
///   The private commit arguments are text, and a pathspec is removed from them by equal text;
///  a name that is not UTF-8 is compared by its readable form, the form the arguments themselves have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// region.pathspecs
/// ```
fn pathspec_texts(invocation: &CommitInvocation<'_>) -> Vec<String> {
    let region_arguments: &[OsString] = &invocation.arguments[invocation.subcommand_index + 1..];
    return invocation
        .region
        .pathspecs
        .iter()
        .map(|index| { return region_arguments[*index].to_string_lossy().into_owned(); })
        .collect();
}

/// What:
///  The pathspec-file source the commit names, as text.
/// Why:
///   Standard input is copied once into the transaction so no second read sees different bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// region.pathspecFile
/// ```
fn pathspec_file_source(invocation: &CommitInvocation<'_>) -> Option<String> {
    let region_arguments: &[OsString] = &invocation.arguments[invocation.subcommand_index + 1..];
    let value: OptionValue = invocation.region.pathspec_from_file?;
    return Some(String::from_utf8_lossy(value_bytes(region_arguments, value)).into_owned());
}

/// What:
///  The prepared state a converged commit hands to its conclusion.
/// Why:
///   The conclusion needs the pass that settled and the paths the commit carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettledCommitTransaction = { workspace; capture; mode; pass; changedPasses; changedPaths; ... };
/// ```
struct Converged {
    /// The settled pass, whose events the invocation reports.
    pass: PassResult,
    /// How many passes changed content before it settled.
    changed_passes: u64,
    /// The paths the corrections wrote.
    changed_paths: Vec<Vec<u8>>,
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
    let markdown = std::mem::replace(&mut checks.markdown, markdown_state(Box::new(executable_linter(&[]))));
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
///  Whether the capture lists worktree changes that the commit stages: `-a` stages every tracked change and
///  `--include` stages the named paths.
/// Why:
///   The capture record names those paths, so recovery copies exactly the worktree files the commit staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stagedAll || include
/// ```
pub fn stages_worktree(staged_all: bool, include: bool) -> bool {
    return staged_all || include;
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
///  Run the pre-forward policies over the private index until they settle.
/// Why:
///   A read-only selection commits exactly what the person chose, so its single pass applies no patch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readOnlySelection ? { kind: 'settled', pass: firstPass } : await convergeCommitPolicies({ ... });
/// ```
fn converge<G: RepositoryFacts>(
    request: &StageRequest,
    checks: &mut ShippedChecks<PrivateIndexFacts<G>>,
    context: &GitContext,
    cwd: &Path,
    workspace: &CommitWorkspace,
    read_only: bool,
) -> Converged {
    if read_only {
        checks.can_apply_patches = false;
        let pass: PassResult = run_policy_pass(request, checks);
        return Converged {
            pass,
            changed_passes: 0,
            changed_paths: Vec::new(),
        };
    }
    let outcome: CommitConvergence = converge_commit_policies(CommitConvergenceInput {
        request,
        checks,
        stage: super::private_index_stage::PrivateStage {
            context,
            cwd,
            index: workspace.landing.commit_index.as_path(),
            object_directory: Some(workspace.object_directory.as_path()),
        },
    });
    let changed_passes: u64 = settled_passes(&outcome.convergence);
    return Converged {
        pass: outcome.pass,
        changed_passes,
        changed_paths: outcome.changed_paths,
    };
}

/// What:
///  Run one commit through the transaction: capture, private preparation, serial landing and post-landing completion.
/// Why:
///   Every non-dry-run commit runs through it, including clean commits and `commit -a`, because native `git commit`
///   keeps `index.lock` on disk through hooks and the editor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return await runCommitTransaction({ args, gitPath, policyOptions, concurrency });
/// ```
pub fn run_commit_transaction<F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &mut ShippedChecks<F>,
    request: &StageRequest,
) -> Result<TransactionResult, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let layout: InvocationLayout = capture_invocation_layout(context, cwd, invocation.region.amend)?;
    // Decided from the snapshot the layout already holds: no process and no file system call, and it never blocks.
    warn_traditional_hooks(layout.config_snapshot.as_ref(), &mut std::io::stderr());
    let selection: Selection = decide_selection(invocation.region, layout.conclusion);
    let mut workspace: CommitWorkspace = create_commit_workspace(&layout)?;
    // Read after publication and before the base, so landed-capture pruning keeps every record this transaction
    // may replay over.
    let next_sequence_before_base: i64 = read_next_capture_sequence(layout.git_dir.as_path())
        .map_err(|error| { return failed(error.0); })?;
    let capture: InvocationCapture = capture_invocation_base(context, cwd, layout)?;
    let base: String = base_revision(&capture);
    let pathspecs: Vec<String> = pathspec_texts(invocation);
    let pathspec_arguments: Vec<OsString> = pathspec_arguments(invocation);
    if selection.mode == TransactionMode::ExplicitPath {
        // Before any private staging, so Git's "nothing to commit" or its force-add hint never stands in for the reason.
        let present: Vec<Vec<u8>> = staged_deletions_present_on_disk(
            context,
            cwd,
            capture.layout.repository_root.as_path(),
            capture.layout.real_index_path.as_path(),
            base.as_str(),
            pathspec_arguments.as_slice(),
        )?;
        if !present.is_empty() {
            debug(TAG, "an explicit-path commit selects a staged deletion whose path is on disk");
            return Ok(TransactionResult {
                events: vec![PolicyEvent::CoreFinding {
                    core_id: STAGED_DELETION_CORE_ID,
                    code: STAGED_DELETION_CODE,
                    message: staged_deletion_message(present.as_slice()),
                }],
                exit_code: 1,
                landed_oid: None,
                uses_lfs: false,
            });
        }
    }
    write_journal_record(
        workspace.landing.directory.as_path(),
        PREPARING_FILENAME,
        encode_preparing(&PreparingRecord {
            transaction_id: workspace.landing.transaction_id.clone(),
            mode: selection.mode,
            base: capture.base.clone(),
            symbolic_head: capture.layout.symbolic_head.clone(),
            target_ref: capture.layout.target_ref.clone(),
            conclusion: capture.layout.conclusion,
            repository_root: capture.layout.repository_root.display().to_string(),
            git_dir: capture.layout.git_dir.display().to_string(),
            common_dir: capture.layout.common_dir.display().to_string(),
            real_index_path: capture.layout.real_index_path.display().to_string(),
            object_directory: capture.layout.object_directory.display().to_string(),
            ref_format: capture.layout.ref_format,
            empty_tree_oid: capture.layout.empty_tree_oid.clone(),
            shadow_path: workspace.landing.shadow.display().to_string(),
            selected_pathspecs: pathspec_arguments
                .iter()
                .map(|pathspec| { return pathspec.as_encoded_bytes().to_vec(); })
                .collect(),
            invoked_at: capture.layout.invoked_at.clone(),
        })
        .as_str(),
    )?;
    // Before the private index exists, so every object preparation writes lands in the shadow store, which a
    // concurrent real `gc --prune=now` never sees.
    super::shadow_create::create_shadow_repository(
        context,
        cwd,
        &capture,
        workspace.landing.transaction_id.as_str(),
        workspace.landing.directory.as_path(),
    )?;
    let pathspec_file: Option<PathBuf> = materialize_pathspec_file(
        workspace.landing.directory.as_path(),
        cwd,
        pathspec_file_source(invocation).as_deref(),
        &mut std::io::stdin(),
    )
    .map_err(|error| { return failed(format!("reading the pathspec file failed: {error}")); })?;
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: workspace.landing.captured_index.clone(),
        commit_index: workspace.landing.commit_index.clone(),
        object_directory: workspace.object_directory.clone(),
    };
    let index_selection: IndexSelection<'_> = IndexSelection {
        mode: selection.mode,
        pathspecs: pathspec_arguments.as_slice(),
        pathspec_file: pathspec_file.as_deref(),
        pathspec_file_nul: invocation.region.pathspec_file_nul,
        stage_into_index: invocation.region.include,
        stage_tracked_changes: selection.staged_all,
        base_revision: base.as_str(),
    };
    capture_commit_content(
        context,
        cwd,
        capture.layout.git_dir.as_path(),
        workspace.landing.directory.as_path(),
        &indexes,
        selection.mode,
        stages_worktree(selection.staged_all, invocation.region.include),
        base.as_str(),
        next_sequence_before_base,
        || {
            return initialize_commit_index(
                context,
                cwd,
                capture.layout.real_index_path.as_path(),
                &indexes,
                &index_selection,
            )
            .map_err(CaptureOrderError::from);
        },
    )?;
    if invocation.region.interactive || invocation.region.patch {
        prepare_interactive_selection(
            context,
            cwd,
            workspace.landing.commit_index.as_path(),
            workspace.object_directory.as_path(),
            invocation.region.patch,
            &index_selection,
        )?;
    }
    let unmerged: Vec<Vec<u8>> =
        list_unmerged_index_paths(context, cwd, workspace.landing.commit_index.as_path())?;
    if !unmerged.is_empty() {
        return Ok(TransactionResult {
            events: vec![PolicyEvent::EngineFailure {
                code: EngineFailureCode::ContentUnavailable,
                message: format!(
                    "Automatic commit fixes do not support unmerged index paths: {}",
                    unmerged
                        .iter()
                        .map(|path| { return path_message_text(path.as_slice()); })
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                trigger: Some(Trigger::PreForward),
                policy: None,
                path: None,
            }],
            exit_code: 2,
            landed_oid: None,
            uses_lfs: false,
        });
    }
    let candidate_paths: Vec<Vec<u8>> = list_changed_index_paths(
        context,
        cwd,
        workspace.landing.commit_index.as_path(),
        base.as_str(),
        workspace.object_directory.as_path(),
    )?;
    let mut private: ShippedChecks<PrivateIndexFacts<&mut F>> = private_checks(
        checks,
        workspace.landing.commit_index.as_path(),
        workspace.object_directory.as_path(),
        staged_at_base(base_object(&capture)?, candidate_paths.as_slice()),
    );
    let converged: Converged = converge(request, &mut private, context, cwd, &workspace, selection.read_only);
    let code: i32 = pass_exit_code(converged.pass.events.as_slice(), converged.pass.end);
    if code != 0 {
        debug(TAG, "policies blocked the commit before preparation");
        return Ok(TransactionResult {
            events: converged.pass.events,
            exit_code: code,
            landed_oid: None,
            uses_lfs: false,
        });
    }
    return conclude_commit_transaction(ConclusionInput {
        invocation,
        workspace: &mut workspace,
        capture: &capture,
        selection: &selection,
        request,
        checks: &mut private,
        pass: converged.pass,
        changed_passes: converged.changed_passes,
        changed_paths: converged.changed_paths,
        committed_paths: candidate_paths,
        pathspecs: pathspecs.as_slice(),
    });
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_transaction_tests.rs"]
mod tests;

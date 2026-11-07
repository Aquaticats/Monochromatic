//! What:
//!  Steps of a wrapped command that do not decide its ending:
//! the escape-hatch use record,
//! the push failure notices
//!  shown before the next command,
//! the snapshot taken before a linked worktree is removed,
//! and the preparation of a
//!  manual push.
//! Why:
//!  `wrapped_command.rs` runs the guarded stages in order;
//! these steps stand beside them and depend on none of its
//!  outcome types,
//! so they sit in a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // recordEscapeUse(checks); showPushFailures(stripped, checks); snapshotBeforeWorktreeRemoval(stripped, checks); prepareManualPush(...)
//! ```

use crate::auto_push_single_flight::PushGit;
use crate::child_environment::environment_with_overlay;
/// `git add` asks for what it would stage.
use crate::candidate_prediction::CandidateRequest;
use crate::discard_audit::{AuditOutcome, record_audit};
use crate::manual_push_cause::ManualPushCause;
use crate::manual_push_lifecycle::manual_push_scope;
use crate::policy_checks::ShippedChecks;
/// What the command offers its content policies.
use crate::policy_content::LifecycleContent;
use crate::policy_registry::PolicyId;
use crate::landed_summary::short_name;
use crate::transaction_git::GitContext;
use crate::push_failure_notice::{CommitNamer, ORPHAN_NOTICE_DIRECTORY, is_machine_read, notice_directory, take_orphaned_failures, take_push_failures_for};
use crate::worktree_removal_rescue::{settle_removals, snapshot_linked_worktrees};
use crate::rule_linked_worktree::{LinkedWorktreeDecision, decide_linked_worktree};
use crate::repository_facts::RepositoryFacts;
use crate::worktree_identity::worktree_root;
use crate::wrapper_controls::is_escaped;
use crate::wrapper_invocation::{StrippedInvocation, command_region, command_word};
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments and environment values are never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;
use std::io::Write;
/// Paths of the probe's working directory.
use std::path::{Path, PathBuf};
/// What:
///  Record a guarded command form that runs because the invocation carries the escape for the linked-worktree-only
///  policy.
/// Why:
///   The policy never runs under its escape,
/// so nothing else would say that a discarding form was deliberately let
///  through;
/// a later loss must be attributable to the escape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordEscapeUse(stripped, checks);
/// ```
pub(crate) fn record_escape_use<F: RepositoryFacts>(stripped: &StrippedInvocation, checks: &mut ShippedChecks<F>) {
    if !is_escaped(&stripped.controls, PolicyId::LinkedWorktreeOnly) {
        return;
    }
    let command = match decide_linked_worktree(stripped.arguments.as_slice()) {
        LinkedWorktreeDecision::Pass => return,
        LinkedWorktreeDecision::NeedsEffectiveTarget(command) | LinkedWorktreeDecision::WhenDirty(command) => command,
    };
    if let Ok(location) = checks.facts.location() {
        record_audit(&location.identity, command, AuditOutcome::Hatch, stripped.arguments.as_slice(), None);
    }
}

/// What:
///  Shows,
/// on standard error,
/// each backup push failure the service recorded since one was last shown and that belongs to this
///  worktree,
/// before the command runs.
/// Why:
///   The service pushes after a commit returned,
/// so the command that made the commit cannot print the failure;
/// the next
///  command in the worktree whose commit it was does,
/// because the records live under that worktree's own Git directory.
///  A command whose output a program reads shows nothing and takes
///  nothing,
/// and neither does a command run where the repository is not found without starting Git.
/// When nothing
///  failed the check is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isMachineRead(command)) for (const text of takePushFailures(gitDir)) stderr.write(text + '\n');
/// ```
pub(crate) fn show_push_failures<F: RepositoryFacts>(stripped: &StrippedInvocation, checks: &mut ShippedChecks<F>) {
    if is_machine_read(command_word(stripped), command_region(stripped)) {
        return;
    }
    // The records are the worktree's own, so a notice shows in the worktree whose push failed.
    let Some(git_dir) = checks.facts.known_git_dir() else { return };
    // Without a known common directory, the worktree's own Git directory stands in.
    let common_dir: PathBuf = match checks.facts.known_common_dir() {
        Some(known) => known,
        None => git_dir.clone(),
    };
    // A worktree Git removed took its Git directory along; what it held is shown to whoever asks next. What could not be kept
    //  for the person is said here, each time, until it can be.
    for problem in settle_removals(common_dir.as_path()) {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{problem}");
    }
    // Nothing failed: two directory checks, and no question to Git.
    let own: bool = notice_directory(git_dir.as_path()).is_dir();
    let orphaned: bool = common_dir.join(ORPHAN_NOTICE_DIRECTORY).is_dir();
    if !own && !orphaned {
        return;
    }
    let context = checks.facts.transaction_context();
    let cwd: PathBuf = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(_unreadable) => git_dir.clone(),
    };
    let objects: PathBuf = common_dir.join("objects");
    // The name Git gives the commit in this repository as it is now, asked of Git once per shown notice.
    let abbreviate: GitShortName<'_> = GitShortName { context: &context, cwd: cwd.as_path(), objects: objects.as_path() };
    let mut texts: Vec<String> = take_push_failures_for(git_dir.as_path(), None, &abbreviate);
    texts.extend(take_orphaned_failures(common_dir.as_path(), &abbreviate));
    for text in texts {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{text}");
    }
}

/// What:
///  The name Git gives a commit in this repository as it is now,
/// for the failure notices.
/// Why:
///   The notices abbreviate each commit the way Git does at the moment they are shown;
/// the Git context,
/// the directory and the
///  object directory are the state each question needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GitShortName implements CommitNamer { nameOf(commit) { return shortName(context, cwd, objects, undefined, commit); } }
/// ```
struct GitShortName<'a> {
    /// The Git program and the directories the question uses.
    context: &'a GitContext,
    /// The directory the question is asked in.
    cwd: &'a Path,
    /// The repository's object directory.
    objects: &'a Path,
}

impl CommitNamer for GitShortName<'_> {
    fn name_of(&self, commit: &str) -> String {
        return short_name(self.context, self.cwd, self.objects, None, commit);
    }
}

/// What:
///  Before `git worktree remove` or `git worktree prune`,
/// copies the records of the linked worktrees into snapshots that
///  survive the removal.
/// Why:
///   Git deletes a removed worktree's Git directory,
/// and with it the failed pushes nobody was told about and the pushes the
///  service had not run yet;
/// the process is handed to Git,
/// so only a copy made first can keep them.
/// Which worktrees go is
///  decided later,
/// from whether each still exists (`worktree_removal_rescue.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (command === 'worktree' && ['remove', 'prune'].includes(region[0])) await snapshotLinkedWorktrees(commonDir);
/// ```
pub(crate) fn snapshot_before_worktree_removal<F: RepositoryFacts>(stripped: &StrippedInvocation, checks: &mut ShippedChecks<F>) {
    if command_word(stripped) != b"worktree" {
        return;
    }
    let removes: bool = match command_region(stripped).first() {
        Some(word) => matches!(word.as_encoded_bytes(), b"remove" | b"prune"),
        None => false,
    };
    if !removes {
        return;
    }
    if let Some(common_dir) = checks.facts.known_common_dir() {
        // A record that could not be saved is lost with the worktree, so the person is told before Git runs.
        for problem in snapshot_linked_worktrees(common_dir.as_path()) {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _written = writeln!(std::io::stderr(), "{problem}");
        }
    }
}

/// Lifecycle order,
///  fact use and every ending stay out of the release executable.
/// What:
///  Discover the scope of a manual push and hand it to the manual-push stage as its candidates.
/// Why:
///   The stage reads only what the push publishes,
/// and the discovery runs against the repository the
///  command targets,
/// with the invocation's global options and environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scope = await manualPushScope({ gitPath, cwd, args }); // then the manual-push stage reads it
/// ```
pub(crate) fn prepare_manual_push<F: RepositoryFacts>(
    checks: &mut ShippedChecks<F>,
    environment: &[(OsString, OsString)],
    post_subcommand: &[OsString],
) -> Result<(), crate::manual_push_lifecycle::ManualPushFailure> {
    let worktree: Option<PathBuf> = match checks.facts.location() {
        Ok(location) => worktree_root(&location.identity).map(Path::to_path_buf),
        Err(message) => return Err(crate::manual_push_lifecycle::ManualPushFailure::step(ManualPushCause::SelfExplained, message)),
    };
    let context = checks.facts.transaction_context();
    let cwd = match worktree {
        Some(root) => root,
        None => match std::env::current_dir() {
            Ok(directory) => directory,
            Err(error) => {
                return Err(crate::manual_push_lifecycle::ManualPushFailure::step(ManualPushCause::SelfExplained, format!(
                    "the working directory could not be read: {error}"
                )));
            }
        },
    };
    let git = PushGit {
        real_git: context.real_git,
        cwd,
        environment: environment_with_overlay(environment, context.overlay.as_slice()),
        // The scope probe is a dry-run push; an upload during it would run before any policy has decided.
        lfs: false,
    };
    let scope = manual_push_scope(&git, context.global_prefix.as_slice(), post_subcommand)?;
    checks.candidates = LifecycleContent::Requested(CandidateRequest::Pushed(scope));
    return Ok(());
}

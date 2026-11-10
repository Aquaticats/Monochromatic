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
use crate::held_push_commit_on_the_remote::GitRemoteRecord;
use crate::transaction_git::GitContext;
use crate::push_failure_notice::{CommitNamer, Notice, ORPHAN_NOTICE_DIRECTORY, is_machine_read, notice_directory, orphaned_push_failures, push_failures_for};
use crate::worktree_removal_rescue::{settle_removals, snapshot_linked_worktrees};
use crate::worktree_removal_interrupted_destroy::finish_interrupted_destroys;
use crate::worktree_trash_dispose::name_unaccounted_trash;
use crate::rule_linked_worktree::LinkedWorktreeDecision;
/// The two forms that are recorded and never refused.
use crate::policy_audited::record_audited;
/// The guarded decision about the command Git will run,
///  resolved once with the guard and the transforms.
use crate::policy_checks::guarded_forwarded;
/// The expansion an alias names,
///  which the guard decided on.
use crate::guarded_forwarded_command::GuardedForwarded;
use crate::repository_facts::RepositoryFacts;
use crate::worktree_identity::{WorktreeIdentity, common_directory, worktree_root};
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
    // An alias reaches the command Git really runs, so this record names the expansion like the guard does.
    let forwarded: GuardedForwarded = guarded_forwarded(checks);
    let seen: &[OsString] = forwarded.arguments.as_slice();
    let command = match forwarded.decision {
        LinkedWorktreeDecision::Pass => return,
        LinkedWorktreeDecision::NeedsEffectiveTarget(command) | LinkedWorktreeDecision::WhenDirty(command) => command,
        // The owner asked for one line for these two recorded forms, and the policy that writes it is skipped under
        //  the hatch: the line is written here instead, with the same facts a run without the hatch would carry.
        LinkedWorktreeDecision::Audited(audited) => {
            if let Ok(location) = checks.facts.location() {
                record_audited(&mut checks.facts, seen, audited, &location);
            }
            return;
        }
    };
    if let Ok(location) = checks.facts.location() {
        record_audit(&location.identity, command, AuditOutcome::Hatch, seen, None);
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
    // A removal this program began and its process did not finish (its registration outlived its files) is finished here,
    // so the path it used is free again without the person being sent to `git worktree prune`.
    for problem in finish_interrupted_destroys(common_dir.as_path()) {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{problem}");
    }
    // The trash of the worktree this command runs in is named here when no record accounts for it: the add that made it
    // empties it before it returns, so a trash still there is one that could not be emptied, and nothing else mentions it.
    if let Ok(location) = checks.facts.location()
        && let Some(worktree) = crate::worktree_identity::worktree_root(&location.identity)
    {
        let mut stderr = std::io::stderr();
        name_unaccounted_trash(git_dir.as_path(), worktree, &mut stderr);
    }
    // The directory every question of this step is asked in: the command's own, or the Git directory when it cannot be read.
    let cwd: PathBuf = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(_unreadable) => git_dir.clone(),
    };
    // The held records are asked first, and whether or not any notice directory exists: a push held by a command that never said
    //  anything is said here, because the record is the only place its state exists, and the line stands while the record does.
    //  A held record whose commit Git already holds in the branch's remote is settled here instead of being named
    //  (`held_push_commit_on_the_remote.rs`).
    let context: GitContext = checks.facts.transaction_context();
    let record: GitRemoteRecord<'_> = GitRemoteRecord { context: &context, cwd: cwd.as_path() };
    for line in crate::service_handoff::held_push_notices(git_dir.as_path(), &record) {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{line}");
    }
    // A push record no service has taken is a queued push that no command has said anything about: the one that left it was
    //  killed before it could, or the service went away before it looked. The record is where that statement lives.
    for line in crate::service_handoff::queued_push_notices(git_dir.as_path(), std::time::SystemTime::now()) {
        // A closed terminal is not a reason to stop the command the person asked for.
        let _written = writeln!(std::io::stderr(), "{line}");
    }
    let objects: PathBuf = common_dir.join("objects");
    // The name Git gives the commit in this repository as it is now, asked of Git once per shown notice.
    let abbreviate: GitShortName<'_> = GitShortName { context: &context, cwd: cwd.as_path(), objects: objects.as_path() };
    for notice in &push_notices_for(git_dir.as_path(), common_dir.as_path(), &abbreviate) {
        // A closed terminal is not a reason to stop the command the person asked for, and a record whose words could not be
        //  written stays for the next command.
        let _written = notice.show(&mut std::io::stderr());
    }
}

/// What:
///  Settles the commits a reference lock keeps off their branch,
///  and says what became of them,
///  on standard error.
/// Why:
///   The owner chose "Make it,
/// hold until lock goes" (2026-11-09):
///  the commit is written and kept,
///  the command exits `0`,
///  and the next command that finds the lock gone moves the branch.
///  This is that next command,
///  in every command of the worktree,
///  because the record is the only place the state exists and nothing waits for anything:
///  a lock that still stands costs one look and names the lock,
///  and the count says when held commits are collecting.
///  A command whose output a program reads settles them too,
///  because a script's command is still the next command,
///  and only the words are kept from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [held, settled] of settleHeldCommits(gitDir)) if (!machineRead) console.error(lineOf(held, settled));
/// ```
pub(crate) fn show_held_commits<F: RepositoryFacts>(stripped: &StrippedInvocation, checks: &mut ShippedChecks<F>) {
    let Some(git_dir) = checks.facts.known_git_dir() else { return };
    let context: GitContext = checks.facts.transaction_context();
    let (settled, damaged) = crate::held_commit_settle::all(&context, git_dir.as_path(), crate::held_commit_settle::Clearing::Never);
    if settled.is_empty() && damaged.is_empty() {
        return;
    }
    let silent: bool = is_machine_read(command_word(stripped), command_region(stripped));
    let mut lines: Vec<String> = Vec::new();
    let mut waiting: Vec<String> = Vec::new();
    for (held, outcome) in &settled {
        match outcome {
            crate::held_commit_settle::Settled::Landed { commit } => lines.push(crate::held_commit_words::landed(held, commit, false)),
            crate::held_commit_settle::Settled::LandedTheIndexWaits { commit } => lines.push(crate::held_commit_words::landed(held, commit, true)),
            crate::held_commit_settle::Settled::AlreadyThere { commit } => lines.push(crate::held_commit_words::already_there(held, commit)),
            crate::held_commit_settle::Settled::WaitsForLocks(_standing) => {
                // The record names the locks it was held behind, which is what the words give; the ones that stand now are the
                //  same set unless another program took one meanwhile, and the next command settles the commit either way.
                waiting.push(crate::held_commit_words::waiting(held, commit_name_of(&context, held).as_str()));
            }
            crate::held_commit_settle::Settled::Conflicts { commit, paths } => waiting.push(crate::held_commit_words::conflicts(held, commit, paths.as_slice())),
            crate::held_commit_settle::Settled::Cannot { commit, because } => waiting.push(crate::held_commit_words::cannot(held, commit, because.as_str())),
        }
    }
    if !waiting.is_empty() {
        lines.push(crate::held_commit_words::heading(waiting.len()));
        lines.extend(waiting);
    }
    if !damaged.is_empty() {
        lines.push(crate::held_commit_words::unreadable(damaged.as_slice()));
    }
    if silent {
        return;
    }
    for line in lines {
        let _written = writeln!(std::io::stderr(), "{}: {line}", crate::wrapper_names_old_and_new::PROGRAM.current);
    }
}

/// The name Git gives the held commit in this repository as it is now.
fn commit_name_of(context: &GitContext, held: &crate::held_commit_record::Held) -> String {
    let objects: std::path::PathBuf = held.common_dir.join("objects");
    return crate::landed_summary::short_name(context, held.repository.as_path(), objects.as_path(), None, held.new.as_str());
}

/// What:
///  The failures the service recorded since one was last shown.
/// Why:
///   Each comes with what removes it once its words are out,
/// and when nothing failed the check is two
///  directory reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushNoticesFor(gitDir, commonDir) // take the notices
/// ```
pub(crate) fn push_notices_for(git_dir: &Path, common_dir: &Path, abbreviate: &dyn CommitNamer) -> Vec<Notice> {
    let own: bool = notice_directory(git_dir).is_dir();
    let orphaned: bool = common_dir.join(ORPHAN_NOTICE_DIRECTORY).is_dir();
    if !own && !orphaned {
        return Vec::new();
    }
    let mut notices: Vec<Notice> = push_failures_for(git_dir, None, abbreviate);
    notices.extend(orphaned_push_failures(common_dir, abbreviate));
    return notices;
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

/// What:
///  The directory a manual push's discovery runs Git in,
///  for the repository shape the invocation selects.
/// Why:
///   Every query of the discovery (`for-each-ref`,
/// `cat-file --batch-check`,
/// `rev-list`) names no repository of its own,
///  so Git resolves the one its working directory is in.
///  A worktree has its top level for that answer;
///  a bare repository has no worktree and is itself the directory whose refs and objects the push publishes,
///  so a push from a bare repository that ran these queries in the process's directory
///  outside every repository
///  failed with `content-unavailable` before it reached Git.
///  Nothing is answered for a command outside every repository:
///  there the process's own directory stands,
/// and Git refuses the push itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = identity.worktreeRoot ?? identity.commonDir ?? undefined;
/// ```
fn manual_push_directory(identity: &WorktreeIdentity) -> Option<&Path> {
    if let Some(root) = worktree_root(identity) {
        return Some(root);
    }
    return common_directory(identity);
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
    let repository: Option<PathBuf> = match checks.facts.location() {
        Ok(location) => manual_push_directory(&location.identity).map(Path::to_path_buf),
        Err(message) => return Err(crate::manual_push_lifecycle::ManualPushFailure::step(ManualPushCause::SelfExplained, message)),
    };
    let context = checks.facts.transaction_context();
    let cwd = match repository {
        Some(directory) => directory,
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

/// Controls for what a command says about a worktree's pushes stay out of the release executable.
#[cfg(test)]
#[path = "wrapped_command_notices_snapshots_and_manual_push_tests.rs"]
mod tests;

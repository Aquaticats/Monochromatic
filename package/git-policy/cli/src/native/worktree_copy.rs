//! What:
//!  The linked-worktree ignored-state copy around one forwarded Git invocation: the public entry point the
//!  integrator calls, the settlement and summary it writes, and the exit status it returns.
//! Why:
//!  A worktree that Git creates (or a bare repository's) needs its ignored files from the source worktree,
//!  copied by the resident service, which is the only process that runs copies, from a request the wrapper writes after
//!  Git returned (`doc/worktree-copy-claim.md`). Nothing here takes a lock or waits for another command.
//!
//! Integrator contract:
//!  The caller decides the bypass facts (`--no-worktree-copy`, short-circuit global help or version, whether the
//!  command creates or moves worktrees after alias resolution) with the native modules that already own them,
//!  passes the repository identity and the effective working directory, and supplies the runner that starts real Git.
//!  This module never runs the commit refusal; it is not wired yet.
//!  A main-checkout source is a bypass unless the caller also names `Invocation::main_checkout_paths` (the configured
//!  forbidden-strings rules files, each entry of an array); then only those paths are copied, and only when Git ignores them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use super::json_record::quote;
use super::service_copy::CopyOutcome;
use super::transaction_git::GitContext;
use super::worktree_copy_error::{WorktreeCopyError, lifecycle_failure, raw_failure};
use super::worktree_copy_intent::{Intent, mark_git_returned, remove_intent, write_intent};
use super::worktree_copy_publish::{Published, is_added_root, publish_request};
use super::worktree_copy_registry::{
    CreatedWorktree, CreatedWorktrees, WorktreeCopyObservation, find_created_worktrees, is_initializing,
    observe_copy_source,
};
use super::worktree_copy_request::{copy_pending, new_request_id};
use super::worktree_identity::WorktreeIdentity;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  How a real Git run ended.
/// Why:
///   Its numeric status is the exit status the wrapper keeps; a signal has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedGitExecution = { failure?: SubprocessError };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitStatus {
    /// The exit code, absent when Git ended by a signal.
    pub code: Option<i32>,
}

impl GitStatus {
    /// Whether Git exited with status zero.
    pub fn succeeded(&self) -> bool {
        return self.code == Some(0);
    }
}

/// What:
///  Everything the lifecycle needs to decide how one invocation is settled.
/// Why:
///   The decisions that depend on argv and global options are made by their owners and passed in as facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runGitWithWorktreeCopy({ args, gitPath, identity, command, environment })
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invocation {
    /// The effective repository identity captured before real Git.
    pub identity: WorktreeIdentity,
    /// The working directory the repository queries run in.
    pub effective_cwd: PathBuf,
    /// Whether the command creates or moves worktrees after alias resolution.
    pub creates_or_moves: bool,
    /// Whether `--no-worktree-copy` appeared in flag position.
    pub opted_out: bool,
    /// Whether real Git only prints help or its version.
    pub short_circuit: bool,
    /// Repository paths a main-checkout source hands over (the configured rules file);
    ///  empty keeps the main checkout a bypass, and linked and bare sources ignore it.
    pub main_checkout_paths: Vec<String>,
    /// What the command line says about the worktree it asks `git worktree add` to create.
    pub added: AddedPath,
    /// The real Git executable and the variables every Git child receives.
    pub context: GitContext,
}

/// What:
///  What the command line says about the worktree a command creates.
/// Why:
///   A worktree locked by `--lock` is the command's own to fill, and only the command line can say which one that is; a
///  command line this module cannot read is a failure to attribute, not a reason to skip the worktree silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddedPath = { kind: 'named'; path: string } | { kind: 'not-add' } | { kind: 'unreadable' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddedPath {
    /// A plain `worktree add` that names this destination, as written and not yet resolved.
    Named(PathBuf),
    /// Not a `worktree add`: a move, or a command whose worktrees need no attribution.
    NotAdd,
    /// A `worktree add` this module cannot read, or a command Git expanded from an alias.
    Unreadable,
}

impl AddedPath {
    /// The named destination, if there is one.
    pub fn named(&self) -> Option<&Path> {
        return match self {
            AddedPath::Named(path) => Some(path.as_path()),
            AddedPath::NotAdd | AddedPath::Unreadable => None,
        };
    }
}

/// What:
///  The result of one settled invocation: Git's status and the copy failure, if any.
/// Why:
///   Both facts decide the exit status, so they travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a resolved run, or a ForwardedGitWorktreeCopyError carrying both facts
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    /// How real Git ended.
    pub git: GitStatus,
    /// The ignored-state copy failure, when copying failed.
    pub copy_failure: Option<WorktreeCopyError>,
}

/// Runs real Git once.
pub type GitRunner<'a> = dyn FnMut() -> Result<GitStatus, WorktreeCopyError> + 'a;

/// What:
///  The one summary line a successful copy writes.
/// Why:
///   Users and agents read the count, the source, and the destination count from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderSummary({ copiedEntries, destinationCount, sourceRoot })
/// ```
pub fn summary_line(copied: usize, destinations: usize, source: Option<&Path>) -> String {
    let source_text: String = match source {
        None => String::from("bare repository with an empty source set"),
        Some(path) => quote(path.to_string_lossy().as_ref()),
    };
    let worktree_noun: &str = if destinations == 1 {
        "worktree"
    } else {
        "worktrees"
    };
    let entry_noun: &str = if copied == 1 { "entry" } else { "entries" };
    return format!(
        "cli-git: copied {copied} ignored filesystem {entry_noun} from {source_text} into {destinations} new {worktree_noun}.\n"
    );
}

/// What:
///  The exit status of a settled invocation.
/// Why:
///   A copy-only failure is 2; Git's own status is kept when Git failed; a signal without a status is 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // src/bin.ts: copy failure 2, Git status kept, signal 1
/// ```
pub fn exit_code(settlement: &Settlement) -> i32 {
    let git_code: i32 = settlement.git.code.unwrap_or(1);
    // `match` on whether Git and the copy each succeeded.
    match (
        settlement.git.succeeded(),
        settlement.copy_failure.is_some(),
    ) {
        (true, false) => return 0,
        (false, false) => return git_code,
        (true, true) => return 2,
        (false, true) => return git_code,
    }
}

/// What:
///  Runs one forwarded invocation with no synchronization, and settles its status.
/// Why:
///   Every bypass forwards exactly the same way, so the runner is called once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forwardOnly({ args, gitPath, environment });
/// ```
fn forward_only(run_git: &mut GitRunner) -> Result<Settlement, WorktreeCopyError> {
    let git: GitStatus = run_git()?;
    return Ok(Settlement {
        git,
        copy_failure: None,
    });
}

/// What:
///  What a wrapper hands the resident service: the written requests, and a nudge for requests left pending.
/// Why:
///   The copy is the service's work. A test supplies a requester that runs the service's drain in the same process, and
///  production supplies one that talks to the service over its connection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CopyRequester { request(commonDir, ids, stderr): Promise<Outcome[]>; wake(commonDir): void }
/// ```
pub trait CopyRequester {
    /// Hands over the requests with these identities, which are written, and waits until each is answered.
    fn request(
        &mut self,
        common_dir: &Path,
        ids: &[String],
        stderr: &mut dyn Write,
    ) -> Result<Vec<(String, CopyOutcome)>, WorktreeCopyError>;

    /// Tells the service that requests or journals are pending, and waits for nothing.
    fn wake(&mut self, common_dir: &Path);

    /// Says that the failures this requester returned were shown to the person, so the notices the service left for them
    ///  are withdrawn.
    fn failures_shown(&mut self, _common_dir: &Path) {}
}

/// What:
///  The notice for a worktree that was removed before anything was installed into it.
/// Why:
///   Nothing was created, so the user is told why no copy happened instead of seeing a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: skipped the ignored-state copy into ${JSON.stringify(root)}, which is no longer the registered worktree it was made for.`
/// ```
fn removed_notice(destination: &Path) -> String {
    return format!(
        "cli-git: skipped the ignored-state copy into {}, which is no longer the registered worktree it was made for.\n",
        quote(destination.to_string_lossy().as_ref())
    );
}

/// The sentence a copy failure ends with unless the copy was given up on.
const KEPT_SENTENCE: &str =
    "The request is kept and runs again the next time any git command runs in this repository.";

/// What:
///  The worktrees this command may fill: the one it asked Git to add, and every new one that Git does not lock.
/// Why:
///   A worktree another command is still creating is locked by Git, whatever its text says in the user's language, so it
///  is that command's to fill; a worktree that is not locked is complete, and the service fills each registration once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// created.filter((worktree) => !worktree.locked || worktree.root === realpath(addedPath))
/// ```
pub fn ready_worktrees<'a>(created: &'a [CreatedWorktree], added_path: Option<&Path>) -> Vec<&'a CreatedWorktree> {
    return created
        .iter()
        .filter(|worktree| return !worktree.locked || is_added_root(added_path, worktree.root.as_path()))
        .collect();
}

/// The variable a control sets to end the wrapper at a named point.
pub const WRAPPER_FAULT_VARIABLE: &str = "CLI_GIT_TEST_ONLY_WRAPPER_FAULT";

/// What:
///  Ends the wrapper without running anything more when a control asked for it at this point.
/// Why:
///   The tests of a wrapper that is killed after Git created a worktree need a wrapper that dies at exactly that
///  instruction; the exit leaves on disk what a kill there leaves, and unlike a kill by signal it is not mistaken for the
///  time bound of the test harness.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.env.CLI_GIT_TEST_ONLY_WRAPPER_FAULT === point) process.exit(137);
/// ```
fn wrapper_fault(point: &str) {
    if std::env::var(WRAPPER_FAULT_VARIABLE).is_ok_and(|armed| return armed == point) {
        eprintln!("cli-git: test fault at {point}");
        std::process::exit(137);
    }
}

/// What:
///  Nudges the service when a copy is pending, and waits for nothing.
/// Why:
///   Any command, whether or not it creates a worktree, is a chance for a copy that a killed wrapper or service left to
///  finish; the nudge is harmless to repeat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (await copyPending(commonDir)) requester.wake(commonDir);
/// ```
fn wake_if_pending(common_dir: &Path, requester: &mut dyn CopyRequester) -> Result<(), WorktreeCopyError> {
    if copy_pending(common_dir)? {
        requester.wake(common_dir);
    }
    return Ok(());
}

/// What:
///  Writes the request for each worktree this command may fill, hands them to the service, waits for the answers, and
///  writes the summary line for the copies that ran.
/// Why:
///   The requests are on disk before the service hears of them, and the intent goes once they are, so a wrapper or a
///  service that is killed at any point loses nothing; the failures are reported once Git's status is known.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = await findCreatedWorktrees(...); await writeRequests(found); const outcomes = await service.request(ids); report(outcomes);
/// ```
fn copy_after(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    intent: Option<&Intent>,
    requester: &mut dyn CopyRequester,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    let common_dir: &Path = observation.common_dir.as_path();
    let found: CreatedWorktrees = find_created_worktrees(&invocation.context, observation)?;
    let ready: Vec<&CreatedWorktree> = ready_worktrees(found.created.as_slice(), invocation.added.named());
    // `match` on the source: a bare repository has nothing to copy, so only a worktree source is copied.
    let Some(source) = observation.source_root.as_ref() else {
        if ready.is_empty() {
            return Ok(());
        }
        let line: String = summary_line(0, ready.len(), None);
        return stderr.write_all(line.as_bytes()).map_err(|error| return raw_failure(&error));
    };
    let mut problems: Vec<String> = Vec::new();
    // A new worktree that Git has locked and that the command line does not name may be this command's own (`--lock`) or
    // another command's; when the command line cannot be read there is no telling, and skipping it would be silent.
    if invocation.added == AddedPath::Unreadable {
        for worktree in found.created.iter().filter(|worktree| return worktree.locked) {
            problems.push(format!(
                "cli-git: the worktree {} is locked and cli-git could not tell from this command line whether this command created it, so no ignored files were copied into it. Add the worktree again with a plain `git worktree add <path>` command line, or copy its ignored files yourself.",
                quote(worktree.root.to_string_lossy().as_ref())
            ));
        }
    }
    let mut destinations: Vec<(String, PathBuf)> = Vec::new();
    for worktree in ready {
        let allow_locked: bool = is_added_root(invocation.added.named(), worktree.root.as_path());
        // `match` on the publication: a request stands, the copy is over already, or the registration is not the one observed.
        match publish_request(common_dir, source.as_path(), &observation.selection, worktree, allow_locked)? {
            Published::Request(id) => destinations.push((id, worktree.root.clone())),
            Published::Already => {}
            Published::Gone => stderr
                .write_all(removed_notice(worktree.root.as_path()).as_bytes())
                .map_err(|error| return raw_failure(&error))?,
        }
    }
    wrapper_fault("wrapper-after-requests");
    // Every request this command owes is durable, so the intent has done its work.
    if let Some(done) = intent {
        let _removed = remove_intent(common_dir, done);
    }
    if destinations.is_empty() {
        wake_if_pending(common_dir, requester)?;
    } else {
        let ids: Vec<String> = destinations.iter().map(|(id, _)| return id.clone()).collect();
        let answers: Vec<(String, CopyOutcome)> = requester.request(common_dir, ids.as_slice(), stderr)?;
        let mut copied: usize = 0;
        let mut finished: usize = 0;
        for (id, outcome) in answers {
            // `match` on how the request ended: only a copy that ran counts toward the summary.
            match outcome {
                CopyOutcome::Copied(entries) => {
                    copied += entries;
                    finished += 1;
                }
                CopyOutcome::AlreadyCopied => {}
                CopyOutcome::Removed => {
                    if let Some((_, root)) = destinations.iter().find(|(known, _)| return *known == id) {
                        stderr
                            .write_all(removed_notice(root.as_path()).as_bytes())
                            .map_err(|error| return raw_failure(&error))?;
                    }
                }
                CopyOutcome::Waiting(text) => problems.push(format!("{text} {KEPT_SENTENCE}")),
                CopyOutcome::Failed(text) => {
                    let kept: &str = if text.contains("given up") { "" } else { KEPT_SENTENCE };
                    problems.push(format!("{text} {kept}").trim_end().to_string());
                }
            }
        }
        if finished > 0 {
            let line: String = summary_line(copied, finished, observation.source_root.as_deref());
            stderr.write_all(line.as_bytes()).map_err(|error| return raw_failure(&error))?;
        }
    }
    if problems.is_empty() {
        return Ok(());
    }
    return Err(WorktreeCopyError(problems.join("\n")));
}

/// What:
///  Settles one forwarded invocation: bypasses forward untouched; a creating command runs real Git and then has the
///  service copy into the worktrees it registered; any other command only nudges the service when a copy is pending.
/// Why:
///   The integrator calls this once per forwarded command and turns the settlement into the exit status with [`exit_code`].
///  Nothing here waits for another command or takes a lock: the only wait is for the service's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
pub fn run_with_worktree_copy(
    invocation: &Invocation,
    run_git: &mut GitRunner,
    requester: &mut dyn CopyRequester,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    return settle_invocation(invocation, run_git, requester, stderr).map_err(lifecycle_failure);
}

/// What:
///  Whether the invocation runs from a linked worktree that Git is still creating.
/// Why:
///   Such a worktree has not been filled yet, so it is no source; a filter or hook of an enclosing `git worktree add`
///  that creates another worktree leaves the copy to the enclosing command, which copies from the real source. The test
///  is Git's own English word in the lock file, a heuristic with no other consumer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind === 'linked-worktree' && await isInitializing(identity.gitDir)
/// ```
fn source_is_being_created(identity: &WorktreeIdentity) -> Result<bool, WorktreeCopyError> {
    let WorktreeIdentity::LinkedWorktree { git_dir, .. } = identity else {
        return Ok(false);
    };
    let (Some(admin_root), Some(name)) = (git_dir.parent(), git_dir.file_name()) else {
        return Ok(false);
    };
    return is_initializing(admin_root, name.to_string_lossy().as_ref());
}

/// What:
///  The settlement of one invocation, before the lifecycle words its raw failures.
/// Why:
///   Only the top of the lifecycle words raw failures, as the incumbent's `asWorktreeCopyError` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // body of runGitWithWorktreeCopy
/// ```
fn settle_invocation(
    invocation: &Invocation,
    run_git: &mut GitRunner,
    requester: &mut dyn CopyRequester,
    stderr: &mut dyn Write,
) -> Result<Settlement, WorktreeCopyError> {
    // `if` chain in the incumbent's order: each bypass forwards without touching the repository.
    if invocation.opted_out || invocation.short_circuit {
        return forward_only(run_git);
    }
    let Some(observation) = observe_copy_source(
        &invocation.identity,
        invocation.effective_cwd.as_path(),
        invocation.main_checkout_paths.as_slice(),
    )?
    else {
        return forward_only(run_git);
    };
    // A creating command that runs inside the checkout of its own source (a filter or hook of an enclosing
    // `git worktree add`) has a source that does not hold its ignored files yet; the enclosing command copies instead.
    if invocation.creates_or_moves && source_is_being_created(&invocation.identity)? {
        return forward_only(run_git);
    }
    if !invocation.creates_or_moves {
        // Requests or journals a killed wrapper or service left are the service's to finish; the command does not wait.
        wake_if_pending(observation.common_dir.as_path(), requester)?;
        return forward_only(run_git);
    }
    // The intent is durable before Git runs: a wrapper killed after Git created a worktree leaves it for the service to find.
    let intent: Option<Intent> = match observation.source_root.as_ref() {
        Some(source) => {
            let written: Intent = Intent {
                id: new_request_id()?,
                source_root: source.clone(),
                selection: observation.selection.clone(),
                before_admin_ids: observation.before_admin_ids.clone(),
                added_path: invocation.added.named().map(Path::to_path_buf),
                git_returned: false,
            };
            write_intent(observation.common_dir.as_path(), &written)?;
            Some(written)
        }
        None => None,
    };
    let git: GitStatus = match run_git() {
        Ok(status) => status,
        Err(failure) => {
            // Git did not start, so nothing was created for the intent to find.
            if let Some(unused) = &intent {
                let _removed = remove_intent(observation.common_dir.as_path(), unused);
            }
            return Err(failure);
        }
    };
    // Git has returned: the service may now write the requests from the intent and drop it. A failed write only leaves the
    // intent as it was, which the service keeps looking at for an hour.
    if let Some(started) = &intent {
        let _marked = mark_git_returned(observation.common_dir.as_path(), started);
    }
    wrapper_fault("wrapper-after-git");
    let copy_failure: Option<WorktreeCopyError> =
        copy_after(invocation, &observation, intent.as_ref(), requester, stderr).err();
    return Ok(Settlement { git, copy_failure });
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;

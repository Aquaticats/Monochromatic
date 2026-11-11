//! What:
//!  Running a forwarded Git command through the linked-worktree ignored-state copy:
//!  which invocations the copy applies to,
//! and the child-process run that performs it.
//! Why:
//!  A command that creates a worktree must copy the ignored state once real Git returns,
//! and a command run from a linked
//!  worktree must first recover a copy a dead process left;
//! replacing the process with Git would drop both duties,
//! so
//!  these invocations run Git as a child.
//! No lock is held:
//! concurrent commands never wait for one another.
//!  The main checkout is a source only for the configured rules file,
//! so it takes this path only when the
//!  command creates a worktree and the configuration names that file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use crate::forwarding::{ChildOutcome, ChildStarted, run_real_git, run_real_git_naming, run_real_git_until};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_remove_plan::RemovePlan;
use crate::worktree_removal_stops_units::{WorktreeAboutToGo, before_removal, stop_units_of_removed_worktree};
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// What:
///  Everything the copy needs to know about one forwarded invocation,
/// decided before Git starts.
/// Why:
///   The plan decides that the copy applies;
/// the run performs it with real Git and the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyForward = { identity: GitWorktreeIdentity; command: ResolvedGitCommand; mainCheckoutPaths: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyFacts {
    /// The repository shape the invocation selects.
    pub identity: WorktreeIdentity,
    /// Whether the command,
    /// after alias resolution,
    /// creates a worktree.
    pub creates: bool,
    /// The destination the creating command's line names,
    /// in the form Git records it;
    /// absent for every other command and for
    ///  a form of `git worktree add` whose destination is not read.
    pub destination: Option<PathBuf>,
    /// The destination of a `git worktree add` in a form the cheap-worktree ladder serves;
    /// absent for every other
    ///  command and every form the ladder does not know.
    pub add_plan: Option<AddPlan>,
    /// The directory of a `git worktree remove` in the plain form the wrapper may serve with a subvolume destroy;
    ///  absent for every other command.
    pub remove_plan: Option<RemovePlan>,
}

/// What:
///  The words of a step as one line for a message,
/// each quoted the way a shell reads it.
/// Why:
///   A stopped step is named by the command it ran,
/// so a person can run it again,
/// and a word holding
///  a space,
/// a `$` or a quote has to reach their shell as one word and no expansion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// arguments.map((word) => shellQuote(word)).join(' ')
/// ```
fn words_of(arguments: &[OsString]) -> String {
    let mut text: String = String::new();
    for (index, word) in arguments.iter().enumerate() {
        if index > 0 {
            text.push(' ');
        }
        text.push_str(crate::shell_quote::shell_quote(word.as_bytes()).as_str());
    }
    return text;
}

/// What:
///  Whether the copy applies to an invocation.
/// Why:
///   Linked worktrees and bare repositories always take the lifecycle (recovery,
/// creation);
///  the main checkout only when it creates a worktree and has paths to hand over,
/// so its other commands
///  keep their plain forwarding and cost no filesystem reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind !== 'outside-worktree' && (identity.kind !== 'main-worktree' || (creates && paths.length > 0))
/// ```
pub fn copy_applies(identity: &WorktreeIdentity, creates: bool) -> bool {
    return copy_applies_with(identity, creates, false);
}

/// What:
///  Whether the copy applies to an invocation,
/// when a `git worktree add` or `git worktree remove` that the ladder
///  serves may also run in the main checkout.
/// Why:
///   The ladder snapshots or clones from the creating worktree,
/// and the main checkout is one;
/// a removal is served from
///  any checkout.
/// A creating command from the main checkout takes the lifecycle whatever the configuration names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyApplies(identity, creates) || (servesLadder && identity.kind === 'main-worktree')
/// ```
pub fn copy_applies_with(identity: &WorktreeIdentity, creates: bool, serves_ladder: bool) -> bool {
    // `match` on the repository shape; the main checkout takes the lifecycle when the command creates a worktree
    // or the ladder serves it.
    match identity {
        WorktreeIdentity::OutsideWorktree => return false,
        WorktreeIdentity::MainWorktree { .. } => {
            return serves_ladder || creates;
        }
        WorktreeIdentity::LinkedWorktree { .. } | WorktreeIdentity::BareRepository { .. } => {
            return true;
        }
    }
}

/// What:
///  How one forwarded run ended.
/// Why:
///   Only a run that failed before or after Git needs its own text;
/// otherwise the status is Git's,
/// and a Git that
///  was ended by a signal is this process's to end by (`Killed`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedRun = { exitCode: number } | { signal: number } | { stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForwardedRun {
    /// Git ran (and the copy,
    /// when one applied,
    /// finished);
    /// this is the exit status.
    Ran(i32),
    /// Git was ended by this signal,
    /// so this process ends by it too;
    /// a wrapper cannot always die of the same signal,
    /// but [`crate::forwarding::end_as_git_ended`] does where it can.
    Killed(i32),
    /// The run could not start or settle;
    /// the standard error text and exit status.
    Failed {
        /// Text for standard error.
        stderr: String,
        /// Exit status.
        code: i32,
    },
}

/// What:
///  The repository's common Git directory,
///  which owns the registrations a sweep reads and the record of a worktree this program made.
/// Why:
///   Both jobs write beside the registrations,
///  and a bare repository has a common directory too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.commonDir
/// ```
#[cfg(unix)]
fn common_directory(identity: &WorktreeIdentity) -> Option<&Path> {
    // `match` on the repository shape: only a place outside a repository has none.
    match identity {
        WorktreeIdentity::MainWorktree { common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, .. }
        | WorktreeIdentity::BareRepository { common_dir, .. } => return Some(common_dir.as_path()),
        WorktreeIdentity::OutsideWorktree => return None,
    }
}

/// What:
///  The directory the repository queries of the copy run in.
/// Why:
///   Every query names absolute paths,
/// so any directory of the repository works;
/// the worktree root exists whenever
///  the identity has a worktree,
/// and a bare repository's common directory is itself a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.effectiveCwd
/// ```
#[cfg(unix)]
fn query_directory(identity: &WorktreeIdentity) -> PathBuf {
    // `match` on the repository shape: a worktree root, or the bare repository's directory.
    match identity {
        WorktreeIdentity::MainWorktree { worktree_root, .. }
        | WorktreeIdentity::LinkedWorktree { worktree_root, .. } => return worktree_root.clone(),
        WorktreeIdentity::BareRepository { common_dir, .. } => return common_dir.clone(),
        WorktreeIdentity::OutsideWorktree => return PathBuf::new(),
    }
}

/// What:
///  Runs real Git as a child with the given variables and reports how it ended.
/// Why:
///   One start for the plain and the settled runs,
/// so both report an unstartable Git the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nanoSpawn(gitPath, args, { env, stdio: 'inherit' });
/// ```
fn run_plain(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> ForwardedRun {
    // `match` on the start result: the status is Git's, the death by a signal is Git's own too, and an unstartable Git
    //  is an engine failure.
    match run_real_git(real_git, arguments, overlay) {
        Ok(ChildOutcome::Exited(code)) => return ForwardedRun::Ran(code),
        // What: A Git that a signal ended stays a death by that signal.
        // Why:  This route keeps a process around Git, so the death comes back as a value; flattening it into 128 + the
        //       signal made the caller see a normal exit where native Git's caller sees `WIFSIGNALED`
        //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 3 of the round-2 review).
        Ok(ChildOutcome::Signaled(signal)) => return ForwardedRun::Killed(signal),
        Err(error) => {
            return ForwardedRun::Failed {
                stderr: format!("monochromatic-git-policy-cli: cannot start {}: {error}\n", real_git.display()),
                code: crate::action::ENGINE_FAILURE_EXIT_CODE,
            };
        }
    }
}

/// What:
///  Runs the forwarded Git command,
/// settling the ignored-state copy around it when one applies.
/// Why:
///   Git's status is the answer unless the copy failed:
/// a copy failure after a successful Git is status 2,
///  after a failed Git it keeps Git's status,
/// and a failure of the lifecycle itself is status 2,
/// each with its message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
pub fn run_forwarded_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    copy: Option<&CopyFacts>,
) -> ForwardedRun {
    return run_forwarded_git_with_unit_stop(real_git, arguments, overlay, copy, &stop_units_of_removed_worktree);
}

/// What:
///  The same run with the stop of a removed worktree's units given.
///  `&dyn Fn(&WorktreeAboutToGo)` is a borrowed function that stops the units of one removed worktree.
/// Why:
///   The stop of a removal's units is this run's own wiring,
/// and reaching the session manager from a control would need a bus;
///  the controls hand in a recorder instead,
/// so a change that drops the wiring fails there
///  rather than only in the worktree-removal tests that share the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runForwardedGit({ args, gitPath, copy, stopUnits });
/// ```
pub fn run_forwarded_git_with_unit_stop(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    copy: Option<&CopyFacts>,
    stop_units: &dyn Fn(&WorktreeAboutToGo),
) -> ForwardedRun {
    #[cfg(unix)]
    {
        if let Some(facts) = copy {
            // The instance of a removed worktree is a hash of a directory the removal deletes, so it is named first.
            let about: Option<WorktreeAboutToGo> = units_of_worktree_about_to_go(facts);
            let finished: ForwardedRun = run_served(real_git, arguments, overlay, facts);
            // Only a removal that ended well can have taken the worktree; the stop checks that its Git directory is gone.
            if let (Some(gone), ForwardedRun::Ran(0)) = (about.as_ref(), &finished) {
                stop_units(gone);
            }
            return finished;
        }
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = copy;
        let _ = stop_units;
    }
    return run_plain(real_git, arguments, overlay);
}

/// What:
///  The worktree a plain `git worktree remove` names,
/// as its commit service units know it.
/// Why:
///   The units of a removed worktree are stopped once the removal ended (`worktree_removal_stops_units.rs`),
/// and the name they have is made from the registration the removal deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const about = facts.removePlan && beforeRemoval(commonDir, facts.removePlan);
/// ```
#[cfg(unix)]
fn units_of_worktree_about_to_go(facts: &CopyFacts) -> Option<WorktreeAboutToGo> {
    let plan = facts.remove_plan.as_ref()?;
    let common_dir: &Path = match &facts.identity {
        WorktreeIdentity::MainWorktree { common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, .. }
        | WorktreeIdentity::BareRepository { common_dir, .. } => common_dir.as_path(),
        WorktreeIdentity::OutsideWorktree => return None,
    };
    return before_removal(common_dir, plan);
}

/// What:
///  Serve the invocation the way the copy applies to it:
/// a removal by subvolume destroy when every condition holds,
/// else the settled run with Git as a child.
/// Why:
///   The two ways end in the same `ForwardedRun`,
/// so what follows a removal is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await removeBySubvolume(facts)) ?? (await runSettled(facts));
/// ```
#[cfg(unix)]
fn run_served(real_git: &Path, arguments: &[OsString], overlay: &[(OsString, OsString)], facts: &CopyFacts) -> ForwardedRun {
    #[cfg(target_os = "linux")]
    if let Some(finished) = remove_by_subvolume(real_git, overlay, facts) {
        return finished;
    }
    return run_settled(real_git, arguments, overlay, facts);
}

/// What:
///  Remove the worktree a plain `git worktree remove` names by destroying its subvolume,
/// when every condition holds.
/// Why:
///   Git's own removal unlinks every file;
/// the destroy is one call.
/// `None` leaves the whole command to Git;
/// a removal
///  that destroyed the files but could not remove the registration fails the command with the repair steps and is never
///  handed to Git,
/// whose removal would find the worktree already gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// facts.removePlan === undefined ? undefined : outcomeToRun(await removeSubvolumeWorktree(request))
/// ```
#[cfg(all(unix, target_os = "linux"))]
fn remove_by_subvolume(real_git: &Path, overlay: &[(OsString, OsString)], facts: &CopyFacts) -> Option<ForwardedRun> {
    use std::io::Write;
    use crate::worktree_remove::{RemoveOutcome, RemoveRequest, remove_subvolume_worktree};
    use crate::worktree_identity::WorktreeIdentity;
    let plan = facts.remove_plan.as_ref()?;
    let common_dir: &Path = match &facts.identity {
        WorktreeIdentity::MainWorktree { common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, .. }
        | WorktreeIdentity::BareRepository { common_dir, .. } => common_dir.as_path(),
        WorktreeIdentity::OutsideWorktree => return None,
    };
    let context: crate::transaction_git::GitContext = crate::transaction_git::GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    match remove_subvolume_worktree(&RemoveRequest { context: &context, plan, common_dir }) {
        RemoveOutcome::LeftToGit => return None,
        RemoveOutcome::Removed => {
            // Exact terminal output: the person runs a destroy that cannot be undone and accepts the risk that comes with it.
            let _closed = writeln!(std::io::stderr(), "{}", crate::worktree_remove::DESTROY_NOTICE);
            return Some(ForwardedRun::Ran(0));
        }
        RemoveOutcome::Incomplete(report) => {
            return Some(ForwardedRun::Failed { stderr: format!("{report}\n"), code: crate::action::ENGINE_FAILURE_EXIT_CODE });
        }
        RemoveOutcome::Refused(report) => {
            // A refusal is not a failure of a step: nothing changed, and the status is the one every refusal of this wrapper
            // uses.
            return Some(ForwardedRun::Failed { stderr: format!("{report}\n"), code: crate::subvolume_command::REFUSED_EXIT_CODE });
        }
    }
}

/// What:
///  The settled run:
/// the lifecycle with real Git as its runner and the process's standard error as its stream.
/// Why:
///   Real Git runs as a child so the wrapper can copy the ignored state once it returns;
/// no lock is held around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
#[cfg(unix)]
fn run_settled(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    facts: &CopyFacts,
) -> ForwardedRun {
    use std::io::Write;
    use crate::checkout_workers::{insert_parallel_checkout, is_fallback_checkout, parallel_checkout_wanted};
    use crate::transaction_git::GitContext;
    use crate::worktree_copy::{GitRunner, GitStatus, GitStep, Invocation, exit_code, run_with_worktree_copy};
    use crate::worktree_copy_error::WorktreeCopyError;

    /// What:
    ///  Decide once whether the checkout gets parallel workers,
    /// and say the decision.
    /// Why:
    ///   The first checkout step decides from what the person set,
    /// and later steps use that answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// parallel ??= parallelCheckoutWanted(context, cwd, arguments); return parallel;
    /// ```
    fn decided_parallel(parallel: &mut Option<bool>, invocation: &Invocation, arguments: &[OsString]) -> bool {
        if parallel.is_none() {
            *parallel = Some(parallel_checkout_wanted(&invocation.context, invocation.effective_cwd.as_path(), arguments));
        }
        return *parallel == Some(true);
    }

    let context: GitContext = GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        // Every copy query names absolute paths and leaves the caller's global options out.
        global_prefix: Vec::new(),
        // The copy queries send no input, so no scratch directory is needed.
        common_dir: None,
    };
    let invocation: Invocation = Invocation {
        identity: facts.identity.clone(),
        effective_cwd: query_directory(&facts.identity),
        creates: facts.creates,
        destination: facts.destination.clone(),
        opted_out: false,
        short_circuit: false,
        context,
        add_plan: facts.add_plan.clone(),
        #[cfg(target_os = "linux")]
        repair: crate::package_repair::RepairSwitch::from_arguments(arguments),
    };
    let mut stderr = std::io::stderr();
    /// What:
    ///  The real Git started as a child for each step of the ladder,
    /// with the options the step needs.
    /// Why:
    ///   The ladder calls it once per Git step;
    /// it carries the invocation,
    /// the real Git,
    /// the overlay and the parallel-checkout
    ///  decision that the first checkout step makes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// class ForwardedGit implements GitRunner { parallel?: boolean; run(step) { return runRealGit(realGit, argumentsFor(step), overlayFor(step)); } }
    /// ```
    struct ForwardedGit<'a> {
        /// The variables added to the environment of every step.
        overlay: &'a [(OsString, OsString)],
        /// The arguments the person gave.
        arguments: &'a [OsString],
        /// What the plan found out about the command.
        facts: &'a CopyFacts,
        /// The invocation the ladder runs for.
        invocation: &'a Invocation,
        /// The real Git.
        real_git: &'a Path,
        /// Whether the checkout may get parallel workers,
        /// decided at the first checkout step from what the person set,
        /// and not again.
        parallel: Option<bool>,
        /// The signal the last Git step was ended by,
        /// so a forwarded command's caller sees what Git's caller sees.
        signalled: Option<i32>,
    }

    impl GitRunner for ForwardedGit<'_> {
        fn run(&mut self, step: &GitStep<'_>, child_started: &mut dyn ChildStarted) -> Result<GitStatus, WorktreeCopyError> {
            let mut child_overlay: Vec<(OsString, OsString)> = self.overlay.to_vec();
            let mut child_arguments: Vec<OsString> = self.arguments.to_vec();
            match step {
                GitStep::Person => {
                    // The person's `git worktree add` is the checkout of the new worktree: it gets parallel workers where nothing sets them.
                    if self.facts.creates && decided_parallel(&mut self.parallel, self.invocation, self.arguments) {
                        insert_parallel_checkout(&mut child_arguments);
                    }
                }
                GitStep::WithoutCheckout(at) => {
                    // The index was read from the same arguments, so the word before it is `add`.
                    if *at == 0 || *at > child_arguments.len() || child_arguments[*at - 1] != "add" {
                        return Err(WorktreeCopyError(String::from("monochromatic-git-policy-cli: the worktree plan does not match the arguments.")));
                    }
                    child_arguments.insert(*at, OsString::from("--no-checkout"));
                }
                GitStep::Other { arguments: other, environment } => {
                    child_arguments = other.to_vec();
                    child_overlay.extend(environment.iter().cloned());
                    // The ladder's own checkout after a failed snapshot is the same checkout, so it takes the same option.
                    if is_fallback_checkout(child_arguments.as_slice()) && decided_parallel(&mut self.parallel, self.invocation, self.arguments) {
                        insert_parallel_checkout(&mut child_arguments);
                    }
                }
            }
            // Only a step the ladder started has a ceiling; the person's own command waits as long as native Git does, whichever
            //  word it names, and a filter or a filesystem that never answers must not hold a ladder step (owner: nothing waits
            //  forever).
            let ladder_started: bool = matches!(step, GitStep::Other { .. });
            let ceiling: Option<std::time::Duration> = if ladder_started { crate::ladder_step_ceilings::ceiling_of(child_arguments.as_slice()) } else { None };
            let ran: std::io::Result<ChildOutcome> = match ceiling {
                Some(longest) => run_real_git_until(self.real_git, child_arguments.as_slice(), child_overlay.as_slice(), longest),
                None => run_real_git_naming(self.real_git, child_arguments.as_slice(), child_overlay.as_slice(), child_started),
            };
            match ran {
                Ok(ChildOutcome::Exited(code)) => return Ok(GitStatus { code: Some(code) }),
                Ok(ChildOutcome::Signaled(signal)) => {
                    self.signalled = Some(signal);
                    return Ok(GitStatus { code: None });
                }
                // A step that was ended at its ceiling did start, so the failure says what happened rather than that it could not start.
                Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                    return Err(WorktreeCopyError(format!("monochromatic-git-policy-cli: the step `{}` did not finish: {error}", words_of(child_arguments.as_slice()))));
                }
                Err(error) => {
                    return Err(WorktreeCopyError(format!(
                        "monochromatic-git-policy-cli: cannot start {}: {error}",
                        self.real_git.display()
                    )));
                }
            }

        }
    }
    // The runner starts Git as a child, so the wrapper is still there to copy once Git returns.
    let mut run_git: ForwardedGit<'_> = ForwardedGit { overlay, arguments, facts, invocation: &invocation, real_git, parallel: None, signalled: None };
    match run_with_worktree_copy(&invocation, &mut run_git, &mut stderr) {
        Ok(settlement) => {
            if let Some(failure) = &settlement.copy_failure {
                // A closed standard error leaves nowhere to report the failure, and the exit status still carries it.
                let _ = writeln!(stderr, "{}", failure.0);
            }
            // What: A Git the person ran that was ended by a signal ends this process by that signal, unless the copy itself
            //       failed, whose message and status are the wrapper's own.
            // Why:  The plain forward replaces this process with Git, so a caller's `wait` sees `WIFSIGNALED` and a shell
            //       reports 128 + the signal; the copy route has to end the same way or a script that tells "the reader went
            //       away" from "Git failed" takes the wrong branch (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
            //       finding 3). A settled copy of a creating command keeps its own status, which is never a signal.
            if settlement.copy_failure.is_none()
                && settlement.git.code.is_none()
                && let Some(signal) = run_git.signalled
            {
                return ForwardedRun::Killed(signal);
            }
            // A worktree Git just registered is recorded as one this program made, so a later merge that lands its branch
            // removes it by itself (`worktree_landed_removal.rs`). A record that cannot be written costs that one removal
            // and never the add.
            if facts.creates
                && settlement.copy_failure.is_none()
                && settlement.git.code == Some(0)
                && let (Some(destination), Some(common_dir)) = (facts.destination.as_ref(), common_directory(&facts.identity))
            {
                crate::worktree_added_by_the_wrapper::record_or_log(common_dir, destination.as_path());
            }
            return ForwardedRun::Ran(exit_code(&settlement));
        }
        Err(failure) => {
            return ForwardedRun::Failed {
                stderr: format!("{}\n", failure.0),
                code: crate::action::ENGINE_FAILURE_EXIT_CODE,
            };
        }
    }
}

/// Forwarding-run controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_forward_tests.rs"]
mod tests;

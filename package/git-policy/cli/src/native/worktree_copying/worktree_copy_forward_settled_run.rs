//! What:
//!  The settled run of one forwarded invocation:
//!  the copy's lifecycle with real Git as its runner
//!  and this process's standard error as its stream.
//! Why:
//!  Real Git runs as a child so the wrapper can copy the ignored state once it returns;
//!  no lock is held around it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_command_facts::CopyFacts;
use crate::worktree_copy::{
    Invocation,
    exit_code,
    run_with_worktree_copy,
};
use crate::worktree_copy_forward_outcome::{
    ForwardedRun,
    common_directory,
    query_directory,
};
use crate::worktree_copy_forward_step_runner::ForwardedGit;
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

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
pub(super) fn run_settled(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    facts: &CopyFacts,
) -> ForwardedRun {
    let context: GitContext = GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        // Every copy query names absolute paths and leaves the caller's global options out.
        global_prefix: Vec::new(),
        // The copy queries send no input, so they need no scratch directory; the package repair that runs after the copy
        //  keeps the `Cargo.lock` bytes a clean displaces under the repository's Git directory, so the directory the
        //  identity names is handed on (`package_repair_cargo_lock.rs`).
        common_dir: common_directory(&facts.identity).map(Path::to_path_buf),
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
        quiet: facts.quiet,
        #[cfg(target_os = "linux")]
        repair: crate::package_repair::RepairSwitch::from_arguments(arguments),
    };
    let mut stderr = std::io::stderr();
    // The runner starts Git as a child, so the wrapper is still there to copy once Git returns.
    let mut run_git: ForwardedGit<'_> = ForwardedGit {
        overlay,
        arguments,
        facts,
        invocation: &invocation,
        real_git,
        parallel: None,
        signalled: None,
    };
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
            // Git's own refusal of an occupied destination says only that the directory exists, and a person who removed the
            // worktree there was told what a removal had to leave. The line names each volume and what frees the path, so
            // "already exists" is answered by what stands there.
            if settlement.copy_failure.is_none()
                && settlement.git.code != Some(0)
                && let Some(destination) = facts.destination.as_ref()
            {
                let left: Vec<crate::subvolume_remove_walk::Boundary> =
                    crate::worktree_removal_goes_around_a_foreign_volume::volumes_below(
                        destination.as_path(),
                    );
                if !left.is_empty() {
                    let line: String = crate::worktree_removal_goes_around_a_foreign_volume::add_beside_a_volume_line(destination.as_path(), left.as_slice());
                    let _written = writeln!(stderr, "{line}");
                }
            }
            // A worktree Git just registered is recorded as one this program made, so a later merge that lands its branch
            // removes it by itself (`worktree_landed_removal.rs`). A record that cannot be written costs that one removal
            // and never the add.
            if facts.creates
                && settlement.copy_failure.is_none()
                && settlement.git.code == Some(0)
                && let (Some(destination), Some(common_dir)) = (
                    facts.destination.as_ref(),
                    common_directory(&facts.identity),
                )
            {
                crate::worktree_added_by_the_wrapper::record_or_log(
                    common_dir,
                    destination.as_path(),
                );
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

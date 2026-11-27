//! What:
//!  `git monochromatic-git-policy-cli convert-worktree-to-subvolume`:
//! parse the options,
//! find the worktrees to convert,
//! run the conversion on each,
//!  and turn the results into lines and an exit status.
//! Why:
//!  A conversion takes seconds to minutes per worktree and is bound by the filesystem's metadata work,
//! not by any one
//!  worktree's state,
//! so the worktrees are converted side by side within a budget the host allows (`parallel_budget.rs`);
//!  progress lines go to standard error as they happen for the first unfinished worktree,
//! and every worktree's lines come in
//!  input order (`parallel_runner.rs`).
//! Each conversion is one job of the process's blocking pool,
// awaited by a task of the runtime
//!  (`parallel_runner_on_the_runtime_awaiting.rs`),
//! so the process's one bound covers it and the lines it says while it works go to the
//!  consumer as they are said.
//! Exit status 0 means every worktree is converted,
//! already a subvolume,
//! previewed,
//! undone by `--abandon`,
//! or
//!  skipped under `--all`;
//! 1 means a named worktree was refused before anything changed;
//! 2 means a step failed after the
//!  journal began,
//! with the way forward.
//! One worktree's failure never stops another:
//! each has its own journal and claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await executeConvert({ platform, args, cwd, interrupt, say });
//! ```

use crate::action::Action;
use crate::lazy_runtime::run_on_runtime;
use crate::parallel_budget::HostResources;
use crate::subvolume_error::Interrupt;
use crate::subvolume_platform::{
    LinuxPlatform,
    Platform,
};
use crate::worktree_convert_arguments::{
    CONVERT_HELP,
    ConvertArguments,
};
pub use crate::worktree_convert_budget::{
    CONVERSION_MEMORY,
    CONVERSION_STORAGE_CEILING,
    conversion_workers,
};
use crate::worktree_convert_run_awaiting::execute_run_awaiting;
pub use crate::worktree_convert_run_awaiting::{
    ConvertRun,
    FAILED_EXIT_CODE,
    REFUSED_EXIT_CODE,
};
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

/// What:
///  Run the command over its targets and return the exit status;
/// every line goes through `say`.
/// Why:
///   The platform and the interrupt are injected so the controls run the real command on any filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await execute(platform, arguments, cwd, interrupt, say);
/// ```
pub fn execute(
    platform: Arc<dyn Platform + Send + Sync>,
    arguments: &ConvertArguments,
    cwd: &Path,
    interrupt: Arc<Interrupt>,
    say: &mut dyn FnMut(&str),
) -> i32 {
    return execute_run(
        &ConvertRun {
            platform,
            arguments,
            cwd,
            interrupt,
            host: &HostResources::read(),
        },
        say,
    );
}

/// `execute` with the host's resources chosen by the caller.
/// The callers of this face are synchronous (the command of the wrapper);
/// so it enters the process's runtime once and waits there,
/// and from inside the runtime it refuses,
/// where `execute_run_awaiting` is the face such a caller uses.
pub fn execute_run(
    run: &ConvertRun<'_>,
    say: &mut dyn FnMut(&str),
) -> i32 {
    return match run_on_runtime(execute_run_awaiting(run, say)) {
        Ok(code) => code,
        Err(refusal) => {
            say(format!("git monochromatic-git-policy-cli convert-worktree-to-subvolume: could not start the work: {refusal}").as_str());
            return FAILED_EXIT_CODE;
        }
    };
}

/// What:
///  Say one line of progress on standard error.
/// Why:
///   The command prints progress as it goes;
/// a write that fails (a closed pipe) must not stop the conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sayOnStandardError = (text: string) => { try { process.stderr.write(`${text}\n`); } catch {} };
/// ```
fn say_on_standard_error(text: &str) {
    let mut stderr = std::io::stderr().lock();
    let _written = writeln!(stderr, "{text}");
}

/// What:
///  The action of the command:
/// help,
/// a refusal,
/// or the run on the real platform with progress on standard error.
/// Why:
///   The command reads `.git` itself and calls the filesystem,
/// and starts no Git.
/// It still needs to know which Git the wrapper
///  forwards to (`real_git`,
/// resolved by the caller),
/// because where that Git reads its system files from is a fact only that
///  Git reports,
/// and the wrapper's own environment does not name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertAction(arguments: ConvertArguments, cwd: string, environment: [string, string][], realGit?: string): Action;
/// ```
pub fn convert_action(
    arguments: &ConvertArguments,
    cwd: &Path,
    environment: &[(OsString, OsString)],
    real_git: Option<&Path>,
) -> Action {
    if arguments.help {
        return Action::Exit {
            code: 0,
            stdout: String::from(CONVERT_HELP),
            stderr: String::new(),
        };
    }
    // The conversion runs as a job of the blocking pool, which is inside the process's runtime, where no child may be
    //  started through the bridge; so the facts about the Git this command forwards to are settled here, on this thread.
    let platform: LinuxPlatform = LinuxPlatform::forwarding_to(environment, real_git)
        .with_its_conversion_environment_settled();
    let code: i32 = execute(
        Arc::new(platform),
        arguments,
        cwd,
        Arc::new(Interrupt::never()),
        &mut say_on_standard_error,
    );
    return Action::Exit {
        code,
        stdout: String::new(),
        stderr: String::new(),
    };
}

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_command_tests.rs"]
mod tests;

/// The controls for worktrees converted side by side stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_parallel_tests.rs"]
mod parallel_tests;

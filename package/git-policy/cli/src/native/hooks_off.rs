//! What:
//!  Start the real Git with traditional hooks switched off.
//! Why:
//!  Traditional hooks (executable files in `<git-common-dir>/hooks` or a `core.hooksPath` directory) are unsupported,
//!  and `git lfs` writes its four shims there again whenever it cleans a file, so a hook file must never run
//!  because of a start the wrapper made.
//!  Every real-Git start of the wrapper therefore carries `-c core.hooksPath=<a directory that never exists>` ahead of
//!  its arguments: Git then finds no hook file, while hooks defined in the configuration
//!  (`hook.<name>.event` with `hook.<name>.command`) still run.
//!  A `-c core.hooksPath=...` the caller writes comes later on the command line, so it wins.
//!  The option is part of the argument vector, so it adds no system call and no process start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawn(realGit, ['-c', `core.hooksPath=${NO_HOOKS_DIRECTORY}`, ...args]);
//! ```

/// The global-option parser, to find the subcommand behind the caller's global options.
use super::global_arguments::{GlobalOutcome, global_layout};
/// Owned operating-system text for the arguments.
use std::ffi::OsString;
/// The switch that turns the option on for the process.
use std::sync::atomic::{AtomicBool, Ordering};
/// The child that gets the option.
use std::process::Command;

/// The `core.hooksPath` value of every start: below `/dev/null`, which is a device, so nothing can create it.
pub const NO_HOOKS_DIRECTORY: &str = "/dev/null/cli-git-no-hooks";

/// What:
///  Whether `arguments` run `git config`.
/// Why:
///   A configuration query reports what the repository configures, and the wrapper itself reads `core.hooksPath` that
///  way; the injected value must not answer for it.
///  A query runs no hook, so leaving it out costs no protection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// layout.outcome === 'command' && args[layout.prefixLen] === 'config'
/// ```
fn is_configuration_query(arguments: &[OsString]) -> bool {
    let layout = global_layout(arguments);
    return layout.outcome == GlobalOutcome::Command && arguments[layout.prefix_len] == "config";
}

/// Whether this process starts real Git with hooks off; the executable switches it on before any start.
static ACTIVE: AtomicBool = AtomicBool::new(false);

/// What:
///  Switch the hooks-off option on for every real-Git start this process makes from now on.
/// Why:
///   The executable's entry calls it first, so no production start misses it.
///  The library's unit controls, which fake Git with scripts that record the exact arguments they receive, leave it
///  off, and the binary-level controls prove the option on the built executable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooksOff.active = true;
/// ```
pub fn activate() {
    ACTIVE.store(true, Ordering::SeqCst);
}

/// What:
///  Add the hooks-off option to a command, unless it is a configuration query.
/// Why:
///   The option goes before the arguments the command already carries or is about to carry, so a `-c core.hooksPath`
///  of the caller comes later and wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isConfigurationQuery(args)) command.args.unshift('-c', `core.hooksPath=${NO_HOOKS_DIRECTORY}`);
/// ```
pub fn add_hooks_off_option(command: &mut Command, arguments: &[OsString]) {
    if is_configuration_query(arguments) {
        return;
    }
    command.arg("-c").arg(format!("core.hooksPath={NO_HOOKS_DIRECTORY}"));
}

/// What:
///  Put the hooks-off option in front of a real-Git command's arguments once the process has switched it on.
/// Why:
///   One function serves both builders of real-Git commands, so no start can miss it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (hooksOff.active) addHooksOffOption(command, args);
/// ```
pub fn disable_traditional_hooks(command: &mut Command, arguments: &[OsString]) {
    if ACTIVE.load(Ordering::SeqCst) {
        add_hooks_off_option(command, arguments);
    }
}

/// The option's position, its exemption and its effect on real Git are controlled in the test module.
#[cfg(test)]
#[path = "hooks_off_tests.rs"]
mod tests;

//! What:
//!  Start the real Git with every hook switched off.
//! Why:
//!  The wrapper supports no external hook program: not a traditional hook (an executable file in
//!  `<git-common-dir>/hooks` or a `core.hooksPath` directory; `git lfs` writes its four shims there again whenever it
//!  cleans a file) and not a hook defined in the configuration (`hook.<name>.event` with `hook.<name>.command`).
//!  Every real-Git start of the wrapper therefore carries `-c core.hooksPath=<a directory that never exists>`, which
//!  finds no hook file, and `-c hook.<event>.enabled=false` for every hook event Git knows, which switches every
//!  configured hook off whatever its name, ahead of its arguments.
//!  A `-c` the caller writes comes later on the command line, so it wins.
//!  The options are part of the argument vector, so they add no system call and no process start.
//!  `pre-push` is the one event left on, because the wrapper's own Git LFS upload step rides it while the push phase
//!  of the self-invocation ban is open.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawn(realGit, ['-c', `core.hooksPath=${NO_HOOKS_DIRECTORY}`, ...HOOK_EVENTS.flatMap((e) => ['-c', `hook.${e}.enabled=false`]), ...args]);
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

/// Every hook event of Git 2.56.0 (`githooks(5)`), server-side events included.
///  A configured hook for an event missing here would run, so the control `every_listed_event_is_a_hook_event_of_real_git`
///  asks the host Git whether each name is one.
pub const HOOK_EVENTS: &[&str] = &[
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "proc-receive",
    "post-receive",
    "post-update",
    "reference-transaction",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "fsmonitor-watchman",
    "p4-changelist",
    "p4-prepare-changelist",
    "p4-post-changelist",
    "p4-pre-submit",
    "post-index-change",
];

/// The event the wrapper's own Git LFS upload hook rides; its configured hooks stay on for now.
pub const PUSH_EVENT: &str = "pre-push";

/// The events a real-Git start of the wrapper switches off.
pub fn switched_off_events() -> impl Iterator<Item = &'static str> {
    return HOOK_EVENTS.iter().copied().filter(|event| { return *event != PUSH_EVENT; });
}

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
///   The options go before the arguments the command already carries or is about to carry, so a `-c core.hooksPath`
///  or `-c hook.<event>.enabled` of the caller comes later and wins.
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
    for event in switched_off_events() {
        command.arg("-c").arg(format!("hook.{event}.enabled=false"));
    }
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

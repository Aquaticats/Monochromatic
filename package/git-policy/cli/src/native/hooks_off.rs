//! What:
//!  Start the real Git with every hook switched off.
//! Why:
//!  The wrapper supports no external hook program: not a traditional hook (an executable file in
//!  `<git-common-dir>/hooks` or a `core.hooksPath` directory; `git lfs` writes its four shims there again whenever it
//!  cleans a file) and not a hook defined in the configuration (`hook.<name>.event` with `hook.<name>.command`).
//!  Every real-Git start of the wrapper therefore carries `-c core.hooksPath=<a directory that never exists>`, which
//!  finds no hook file, and `-c hook.<event>.enabled=false` for every hook event Git knows, which switches every
//!  configured hook off whatever its name.
//!  They follow every global option of the caller and precede the subcommand, so no `-c` or `--config-env` of the caller
//!  can switch a hook or the hooks path back on.
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
/// A lock that survives a panic of another holder.
use std::sync::{Mutex, PoisonError};
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

/// Names of the configured hooks this process has read from a repository's configuration.
static CONFIGURED_NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());

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
///  Remember configured hook names, so every later real-Git start of this process switches each off by name.
/// Why:
///   `pre-push` is the one event not switched off as a whole (the wrapper's own Git LFS upload step rides it), so a
///  configured `pre-push` hook is switched off by its name instead.
///  The names come from the configuration snapshot the push path already reads, so no extra Git start is needed, and
///  the wrapper's own injected hook is never among them: it is added to the command line after the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooksOff.configuredNames.push(...names);
/// ```
pub fn remember_hook_names(names: impl IntoIterator<Item = String>) {
    let mut remembered = CONFIGURED_NAMES.lock().unwrap_or_else(PoisonError::into_inner);
    for name in names {
        if !remembered.contains(&name) {
            remembered.push(name);
        }
    }
}

/// What:
///  The configured hook names remembered so far, in the order they were first seen.
/// Why:
///   The argument builder reads them once per start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...hooksOff.configuredNames];
/// ```
pub fn remembered_hook_names() -> Vec<String> {
    return CONFIGURED_NAMES.lock().unwrap_or_else(PoisonError::into_inner).clone();
}

/// What:
///  The arguments a real-Git start gets, with the hooks-off options in front of the subcommand.
/// Why:
///   The options follow every global option of the caller (`-c`, `--config-env`, `--git-dir`), so a caller cannot
///  switch a hook event or the hooks path back on, and they precede the subcommand, so they apply to it.
///  A configuration query gets none, because it must report what the repository configures, and a start that names no
///  subcommand (`--version`, a help query) has no global options to follow, so it gets them first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...args.slice(0, layout.prefixLen), ...hooksOffOptions, ...args.slice(layout.prefixLen)]
/// ```
pub fn hooks_off_arguments(arguments: &[OsString]) -> Vec<OsString> {
    if is_configuration_query(arguments) {
        return arguments.to_vec();
    }
    let layout = global_layout(arguments);
    let split: usize = if layout.outcome == GlobalOutcome::Command { layout.prefix_len } else { 0 };
    let mut result: Vec<OsString> = arguments[..split].to_vec();
    result.push(OsString::from("-c"));
    result.push(OsString::from(format!("core.hooksPath={NO_HOOKS_DIRECTORY}")));
    for event in switched_off_events() {
        result.push(OsString::from("-c"));
        result.push(OsString::from(format!("hook.{event}.enabled=false")));
    }
    for name in remembered_hook_names() {
        result.push(OsString::from("-c"));
        result.push(OsString::from(format!("hook.{name}.enabled=false")));
    }
    result.extend_from_slice(&arguments[split..]);
    return result;
}

/// What:
///  Add `arguments` to a real-Git command, with the hooks-off options once the process has switched them on.
/// Why:
///   One function serves both builders of real-Git commands, so no start can miss the options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command.args(hooksOff.active ? hooksOffArguments(args) : args);
/// ```
pub fn add_arguments(command: &mut Command, arguments: &[OsString]) {
    if ACTIVE.load(Ordering::SeqCst) {
        command.args(hooks_off_arguments(arguments));
    } else {
        command.args(arguments);
    }
}

#[cfg(test)]
#[path = "hooks_off_tests.rs"]
mod tests;

//! What:
//!  Start the real Git with every hook switched off,
//! and with Git's file-watching monitor switched off for the Git the wrapper starts for its own work.
//! Why:
//!  The wrapper supports no external hook program:
//! not a traditional hook (an executable file in
//!  `<git-common-dir>/hooks` or a `core.hooksPath` directory;
//! `git lfs` writes its four shims there again whenever it
//!  cleans a file) and not a hook defined in the configuration (`hook.<name>.event` with `hook.<name>.command`).
//!  Every real-Git start of the wrapper therefore carries `-c core.hooksPath=<a directory that never exists>`,
//! which
//!  finds no hook file,
//! and `-c hook.<event>.enabled=false` for every hook event Git knows,
//! which switches every
//!  configured hook off whatever its name.
//!  The Git the wrapper starts for its own work also carries `-c core.fsmonitor=false`
//! (`StartOwner::Wrapper`):
//! a Git that reads the index asks the file-watching daemon first and waits for it,
//! for up to
//!  `fsmonitor.startTimeout` (60 s by default),
//! so a daemon that is starting or gone holds the step that long.
//!  The command the person typed keeps the person's setting (`StartOwner::Person`),
//! because forwarding a command
//!  changes nothing about it.
//!  They follow every global option of the caller and precede the subcommand,
//! so no `-c` or `--config-env` of the caller
//!  can switch a hook,
//! the hooks path or the monitor back on.
//!  The options are part of the argument vector,
//! so they add no system call and no process start.
//!  The wrapper injects no hook of its own,
//! so `pre-push` is switched off like every other event.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawn(realGit, ['-c', `core.hooksPath=${NO_HOOKS_DIRECTORY}`, ...(owner === 'wrapper' ? ['-c', 'core.fsmonitor=false'] : []), ...HOOK_EVENTS.flatMap((e) => ['-c', `hook.${e}.enabled=false`]), ...args]);
//! ```

/// The global-option parser,
/// to find the subcommand behind the caller's global options.
use crate::global_arguments::{GlobalOutcome, global_layout};
/// Owned operating-system text for the arguments.
use std::ffi::OsString;
/// The switch that turns the option on for the process.
use std::sync::atomic::{AtomicBool, Ordering};
/// The child that gets the option.
use std::process::Command;

/// The `core.hooksPath` value of every start:
/// below `/dev/null`,
/// which is a device,
/// so nothing can create it.
pub const NO_HOOKS_DIRECTORY: &str = "/dev/null/cli-git-no-hooks";

/// The `core.fsmonitor` setting every start of the wrapper's own Git carries.
///  Git's file-watching daemon registers a watch on every directory before it answers,
/// and a Git that reads the index
///  waits for it for up to `fsmonitor.startTimeout`,
/// so a commit step must not depend on the daemon being there or starting.
pub const MONITOR_OFF_SETTING: &str = "core.fsmonitor=false";

/// Whose Git a start runs.
///  The two owners get the same hooks-off prefix and differ in one setting:
/// whether Git's file-watching monitor
///  is left to the person's configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartOwner {
    /// Git the wrapper starts for its own work:
    /// the commit's index builds and trees,
    /// `read-tree`,
    /// `diff-tree`,
    /// `reset`,
    /// the scratch commit,
    /// the landing and service steps,
    /// and queries.
    Wrapper,
    /// The command the person typed,
    /// which the wrapper forwards and changes nothing about:
    /// the person's `core.fsmonitor` stays.
    Person,
}

/// Every hook event of Git 2.56.0 (`githooks(5)`),
/// server-side events included.
///  A configured hook for an event missing here would run,
/// so the control `every_listed_event_is_a_hook_event_of_real_git`
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

/// The events a real-Git start of the wrapper switches off:
/// every hook event Git knows.
pub fn switched_off_events() -> impl Iterator<Item = &'static str> {
    return HOOK_EVENTS.iter().copied();
}

/// What:
///  Whether `arguments` run `git config` to report or change configuration,
/// without opening an editor.
/// Why:
///   A configuration query reports what the repository configures,
/// and the wrapper itself reads `core.hooksPath` that
///  way;
/// the injected value must not answer for it.
///  A query starts no program,
/// so leaving the options out costs no protection.
///  `--edit` (or `-e`) starts the editor,
/// and a Git the editor starts inherits the options through
///  `GIT_CONFIG_PARAMETERS`,
/// so it gets them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// layout.outcome === 'command' && args[layout.prefixLen] === 'config' && !opensEditor(args.slice(layout.prefixLen + 1))
/// ```
fn is_configuration_query(arguments: &[OsString]) -> bool {
    let layout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command || arguments[layout.prefix_len] != "config" {
        return false;
    }
    // Words after `--` are paths, so the scan stops there; the first editor option found answers it.
    let mut opens_editor: bool = false;
    for token in &arguments[layout.prefix_len + 1..] {
        if *token == "--" {
            break;
        }
        if token == "--edit" || token == "-e" {
            opens_editor = true;
            break;
        }
    }
    return !opens_editor;
}

/// Whether this process starts real Git with hooks off;
/// the executable switches it on before any start.
static ACTIVE: AtomicBool = AtomicBool::new(false);

/// What:
///  Switch the hooks-off option on for every real-Git start this process makes from now on.
/// Why:
///   The executable's entry calls it first,
/// so no production start misses it.
///  The library's unit controls,
/// which fake Git with scripts that record the exact arguments they receive,
/// leave it
///  off,
/// and the binary-level controls prove the option on the built executable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooksOff.active = true;
/// ```
pub fn activate() {
    ACTIVE.store(true, Ordering::SeqCst);
}

/// What:
///  The arguments a real-Git start gets,
/// with the options in front of the subcommand:
/// the hooks path and the event switches always,
/// and the monitor switch for `StartOwner::Wrapper`.
/// Why:
///   The options follow every global option of the caller (`-c`,
/// `--config-env`,
/// `--git-dir`),
/// so a caller cannot
///  switch a hook event,
/// the hooks path or the monitor back on,
/// and they precede the subcommand,
/// so they apply to it.
///  A configuration query gets none,
/// because it must report what the repository configures,
/// and a start that names no
///  subcommand (`--version`,
/// a help query) has no global options to follow,
/// so it gets them first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...args.slice(0, layout.prefixLen), ...hooksOffOptions, ...ownerOptions, ...args.slice(layout.prefixLen)]
/// ```
pub fn hooks_off_arguments(arguments: &[OsString], owner: StartOwner) -> Vec<OsString> {
    if is_configuration_query(arguments) {
        return arguments.to_vec();
    }
    let layout = global_layout(arguments);
    let split: usize = if layout.outcome == GlobalOutcome::Command { layout.prefix_len } else { 0 };
    let mut result: Vec<OsString> = arguments[..split].to_vec();
    result.push(OsString::from("-c"));
    result.push(OsString::from(format!("core.hooksPath={NO_HOOKS_DIRECTORY}")));
    if owner == StartOwner::Wrapper {
        result.push(OsString::from("-c"));
        result.push(OsString::from(MONITOR_OFF_SETTING));
    }
    for event in switched_off_events() {
        result.push(OsString::from("-c"));
        result.push(OsString::from(format!("hook.{event}.enabled=false")));
    }
    result.extend_from_slice(&arguments[split..]);
    return result;
}

/// What:
///  Add `arguments` to a real-Git command,
/// with the options of `owner` once the process has switched them on.
/// Why:
///   One function serves both builders of real-Git commands,
/// so no start can miss the options,
/// and the caller
///  names whose Git it starts instead of choosing the options itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command.args(hooksOff.active ? hooksOffArguments(args, owner) : args);
/// ```
pub fn add_arguments(command: &mut Command, arguments: &[OsString], owner: StartOwner) {
    if ACTIVE.load(Ordering::SeqCst) {
        command.args(hooks_off_arguments(arguments, owner));
    } else {
        command.args(arguments);
    }
}

#[cfg(test)]
#[path = "hooks_off_tests.rs"]
mod tests;

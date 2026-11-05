//! What:
//!  Whether the checkout of a new worktree may be given Git's parallel checkout, and the option that does it.
//! Why:
//!  Git's `checkout.workers` defaults to one worker, and a checkout of twenty thousand files measured several times faster with
//!  `checkout.workers=0` (one worker per logical CPU). The owner decided on 2026-11-03 "Pass only if unset": a value the person
//!  set anywhere Git reads (a configuration file of any scope, `GIT_CONFIG_*`, `GIT_CONFIG_PARAMETERS`, a `-c` or
//!  `--config-env` they typed) is theirs and stands alone; only where nothing sets the key does the wrapper put
//!  `-c checkout.workers=0` before the subcommand of its own checkout run. The files are read through the one configuration
//!  read the wrapper already makes, so no extra Git start is added.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!workersSetAnywhere(args, env, configSnapshot)) args.splice(subcommandIndex, 0, '-c', 'checkout.workers=0');
//! ```

use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
use crate::diagnostic_log::debug;
use crate::global_arguments::{GlobalOutcome, global_layout};
use crate::transaction_git::{GitContext, subcommand_of};
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// The configuration key, in the canonical form Git lists it.
pub const KEY: &str = "checkout.workers";

/// The option the wrapper adds: one worker per logical CPU.
const PARALLEL: &str = "checkout.workers=0";

/// What:
///  How many leading words of the arguments are global options, which is where the subcommand starts.
/// Why:
///   The wrapper reads global options with one reader (`--git-dir <path>` and `--work-tree <path>` take a separate value, `-c`
///  and `-C` too, and `--option=value` forms are one word), so the position agrees with where the wrapper's own hook options go.
///  Arguments with no subcommand have no position, and the whole list is global options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prefixLength = layout.outcome === 'command' ? layout.prefixLen : args.length;
/// ```
fn global_prefix_length(arguments: &[OsString]) -> usize {
    let layout = global_layout(arguments);
    return if layout.outcome == GlobalOutcome::Command { layout.prefix_len } else { arguments.len() };
}

/// What:
///  Whether a global option the person typed before the subcommand names the key: `-c checkout.workers=<n>`,
///  `-ccheckout.workers=<n>` or `--config-env=checkout.workers=<variable>`.
/// Why:
///   A typed value is the person's, and the options come before the subcommand, so reading the words up to it finds them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// args.slice(0, subcommandIndex).some((word) => word.toLowerCase().includes('checkout.workers'))
/// ```
pub fn typed_before_subcommand(arguments: &[OsString]) -> bool {
    let before: &[OsString] = &arguments[..global_prefix_length(arguments)];
    return before.iter().map(OsString::as_os_str).any(word_names_key);
}

/// What:
///  Whether one typed word contains the key, in any letter case.
/// Why:
///   `-c`, `-ccheckout.workers=<n>` and `--config-env=...` spell the key differently, and the check is on the text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// word.toLowerCase().includes('checkout.workers')
/// ```
fn word_names_key(word: &OsStr) -> bool {
    return word.to_string_lossy().to_lowercase().contains(KEY);
}

/// What:
///  Whether the environment sets the key: a `GIT_CONFIG_KEY_<n>` variable naming it, or `GIT_CONFIG_PARAMETERS` mentioning it.
/// Why:
///   Git reads these after every file, so a value there is the person's whatever the files say; the check is on the text, so a
///  spelling the reader would decline is still found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Object.entries(env).some(([name, value]) => (name.startsWith('GIT_CONFIG_KEY_') || name === 'GIT_CONFIG_PARAMETERS') && value.toLowerCase().includes('checkout.workers'))
/// ```
pub fn environment_sets<'a>(variables: impl IntoIterator<Item = (&'a OsStr, &'a OsStr)>) -> bool {
    return variables.into_iter().any(variable_sets_key);
}

/// What:
///  Whether one environment variable is a Git configuration variable whose value mentions the key.
/// Why:
///   Only `GIT_CONFIG_KEY_<n>` and `GIT_CONFIG_PARAMETERS` carry configuration, so other variables are not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (name.startsWith('GIT_CONFIG_KEY_') || name === 'GIT_CONFIG_PARAMETERS') && value.toLowerCase().includes('checkout.workers')
/// ```
fn variable_sets_key((name, value): (&OsStr, &OsStr)) -> bool {
    let variable: String = name.to_string_lossy().into_owned();
    return (variable.starts_with("GIT_CONFIG_KEY_") || variable == "GIT_CONFIG_PARAMETERS") && value.to_string_lossy().to_lowercase().contains(KEY);
}

/// What:
///  Whether any entry of the configuration read has the key, in any scope.
/// Why:
///   The listing holds every file Git reads, in precedence order; one entry of any value is the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.entries.some((entry) => entry.key === 'checkout.workers')
/// ```
pub fn configured(snapshot: &ConfigSnapshot) -> bool {
    // The loop stops at the first entry with the key, as `any` did.
    for entry in snapshot.entries() {
        if entry.key == KEY {
            return true;
        }
    }
    return false;
}

/// What:
///  A variable's name and value as borrowed OS strings.
/// Why:
///   `environment_sets` reads borrowed pairs, and the process environment and the overlay both hold owned ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ([name, value]) => [name, value]
/// ```
fn as_os_str_pair((name, value): &(OsString, OsString)) -> (&OsStr, &OsStr) {
    return (name.as_os_str(), value.as_os_str());
}

/// What:
///  Whether the wrapper may give the checkout parallel workers: nothing the person typed, exported or configured sets the key.
/// Why:
///   A configuration that cannot be read leaves the answer unknown, and unknown means the person's setting may exist, so
///  nothing is added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wanted = await parallelCheckoutWanted({ context, cwd, arguments });
/// ```
pub fn parallel_checkout_wanted(context: &GitContext, cwd: &Path, arguments: &[OsString]) -> bool {
    if typed_before_subcommand(arguments) {
        debug("parallelCheckoutWanted", "the person typed the key; nothing is added");
        return false;
    }
    let process: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let exported = process.iter().chain(context.overlay.iter()).map(as_os_str_pair);
    if environment_sets(exported) {
        debug("parallelCheckoutWanted", "the environment sets the key; nothing is added");
        return false;
    }
    let Some(snapshot) = read_config_snapshot(context, cwd) else {
        debug("parallelCheckoutWanted", "the configuration could not be read; nothing is added");
        return false;
    };
    let wanted: bool = !configured(&snapshot);
    debug("parallelCheckoutWanted", if wanted { "no configuration file sets the key; the option is added" } else { "a configuration file sets the key; nothing is added" });
    return wanted;
}

/// What:
///  Put `-c checkout.workers=0` in front of the subcommand of the arguments.
/// Why:
///   Git takes `-c` only as a global option, before the subcommand, after the options the person and the wrapper put there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// args.splice(subcommandIndex, 0, '-c', 'checkout.workers=0');
/// ```
pub fn insert_parallel_checkout(arguments: &mut Vec<OsString>) {
    let layout = global_layout(arguments.as_slice());
    if layout.outcome != GlobalOutcome::Command {
        return;
    }
    arguments.splice(layout.prefix_len..layout.prefix_len, [OsString::from("-c"), OsString::from(PARALLEL)]);
}

/// What:
///  Whether the arguments are the wrapper's own checkout of a new worktree after a snapshot or clone attempt failed:
///  `reset --hard` as the subcommand.
/// Why:
///   That run is the checkout the person's `git worktree add` would have made, so it takes the same option; every other step
///  of the ladder is a bookkeeping or difference step and stays as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// subcommand(args) === 'reset' && args.includes('--hard')
/// ```
pub fn is_fallback_checkout(arguments: &[OsString]) -> bool {
    return matches!(subcommand_of(arguments), Some((_index, name)) if name == "reset") && arguments.iter().map(OsString::as_os_str).any(is_hard_flag);
}

/// What:
///  Whether a word is the `--hard` flag of `reset`.
/// Why:
///   The wrapper's own fallback checkout is `reset --hard`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// word === '--hard'
/// ```
fn is_hard_flag(word: &OsStr) -> bool {
    return word == "--hard";
}

/// Parallel-checkout controls stay out of the release executable.
#[cfg(test)]
#[path = "checkout_workers_tests.rs"]
mod tests;

//! What:
//!  The stop of an invocation that names the repository through a link to the 860 drive that does not resolve.
//! Why:
//!  `entry.rs` holds the planning order and stays under the line limit;
//! this is the one call it makes to the repository link gate.
//! The gate runs before anything of a wrapped command,
//! forwarded read-only commands included,
//! because Git's own `fatal:` line for a dangling link names neither the drive nor the way out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stop = repositoryLinkStop({ globalClean, layout, environment, currentDirectory }); // an exit action, or undefined
//! ```

use crate::action::Action;
use crate::child_environment::environment_value;
use crate::global_arguments::{GlobalLayout, GlobalOutcome};
use crate::repository_link_answer::EXIT_GIT_FATAL;
use crate::repository_link_gate::{LinkGate, invocation_link_gate};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The exit action when the invocation's paths lie under the repository link and the link is not usable,
/// nothing otherwise.
/// Why:
///  Only a command (not a query such as `--version`) works in a repository.
/// The home directory comes from the invocation's environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryLinkStop({ globalClean, layout, environment, currentDirectory })
/// ```
pub fn repository_link_stop(global_clean: &[OsString], layout: &GlobalLayout, environment: &[(OsString, OsString)], current_directory: &Path) -> Option<Action> {
    if layout.outcome != GlobalOutcome::Command {
        return None;
    }
    let home: Option<PathBuf> = match environment_value(environment, "HOME") {
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => None,
    };
    let prefix: &[OsString] = &global_clean[..layout.prefix_len];
    return match invocation_link_gate(home.as_deref(), current_directory, prefix, environment) {
        LinkGate::Clear => None,
        LinkGate::Stop(stderr) => Some(Action::Exit { code: EXIT_GIT_FATAL, stdout: String::new(), stderr }),
    };
}

/// The stop of a dangling repository link at the entry stays out of the release executable.
#[cfg(test)]
#[path = "entry_repository_link_tests.rs"]
mod tests;

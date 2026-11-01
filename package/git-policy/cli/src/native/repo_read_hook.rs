//! What:
//!  The two places where the wrapper's Git runners try the in-process answers first.
//! Why:
//!  `run_metadata_git` (captured queries) and `run_git` (transaction requests) are the choke points every read-only
//!  Git start goes through, so one call in each gives every caller the saving and keeps the fallback to Git in one place.
//!  Setting `CLI_GIT_IN_PROCESS_READS=off` in the wrapper's environment turns the answers off, for a measurement that
//!  compares both and for diagnosing a difference.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = tryInProcess(args, overlay) ?? (await spawnGit(args, overlay));
//! ```

use super::git_metadata::MetadataOutput;
#[cfg(unix)]
use super::repo_read_config_scan::ScopeEnvironment;
#[cfg(unix)]
use super::repo_read_query::{Answer, Query, answer_with};
use super::transaction_git::{GitContext, GitOutput, GitRequest};
use std::ffi::OsString;
#[cfg(unix)]
use std::path::Path;

/// The wrapper environment variable that turns the in-process answers off when it holds `off`.
pub const SWITCH_VARIABLE: &str = "CLI_GIT_IN_PROCESS_READS";

/// Whether the answers are enabled in this process and for this run's added variables.
pub fn enabled(variables: &[(OsString, OsString)]) -> bool {
    let added_off: bool = variables.iter().any(|(name, value)| { return name == SWITCH_VARIABLE && value == "off"; });
    return !added_off && std::env::var_os(SWITCH_VARIABLE).is_none_or(|value| { return value != "off"; });
}

/// What:
///  The overlay entry that makes a runner start Git for every command line.
/// Why:
///   Controls that count the starts of a stand-in Git, or that need an unstartable Git to fail, measure the process
///  path, which stays the fallback for everything the readers decline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const switchOff = [['CLI_GIT_IN_PROCESS_READS', 'off']];
/// ```
#[cfg(test)]
pub fn switch_off() -> [(OsString, OsString); 1] {
    return [(OsString::from(SWITCH_VARIABLE), OsString::from("off"))];
}

/// The answer for a command line, if the readers have one and the switch allows it.
#[cfg(unix)]
fn attempt(
    arguments: &[OsString],
    directory: Option<&Path>,
    base: Vec<(OsString, OsString)>,
    variables: &[(OsString, OsString)],
) -> Option<Answer> {
    if !enabled(variables) {
        return None;
    }
    let mut effective: Vec<(OsString, OsString)> = base.clone();
    effective.extend(variables.iter().cloned());
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(effective.as_slice());
    return answer_with(&Query { arguments, directory, variables }, base, &scope);
}

/// What:
///  The captured output of a query run in the process directory with `overlay` added to Git's environment.
/// Why:
///   `None` means "start Git".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryMetadata(args, overlay): MetadataOutput | undefined
/// ```
#[cfg(unix)]
pub fn try_metadata(arguments: &[OsString], overlay: &[(OsString, OsString)]) -> Option<MetadataOutput> {
    let found: Answer = attempt(arguments, None, std::env::vars_os().collect(), overlay)?;
    return Some(MetadataOutput { success: found.code == 0, code: Some(found.code), stdout: found.stdout, stderr: Vec::new() });
}

/// Without file system readers for this platform, every query starts Git.
#[cfg(not(unix))]
pub fn try_metadata(_arguments: &[OsString], _overlay: &[(OsString, OsString)]) -> Option<MetadataOutput> {
    return None;
}

/// What:
///  The output of a transaction request, if the readers can answer it.
/// Why:
///   The request names its directory, its private index and object directory, the variables to add and to remove,
///  and whether the invocation's global options apply; the readers see exactly the command Git would have run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryRequest(context, request): GitOutput | undefined
/// ```
#[cfg(unix)]
pub fn try_request(context: &GitContext, request: &GitRequest) -> Option<GitOutput> {
    if request.input.is_some() {
        return None;
    }
    let mut arguments: Vec<OsString> = Vec::new();
    if !request.without_prefix {
        arguments.extend(context.global_prefix.iter().cloned());
    }
    arguments.extend(request.arguments.iter().cloned());
    let mut variables: Vec<(OsString, OsString)> = context.overlay.clone();
    variables.extend(request.environment.iter().cloned());
    if let Some(index) = &request.index {
        variables.push((OsString::from("GIT_INDEX_FILE"), index.clone().into_os_string()));
    }
    if let Some(objects) = &request.object_directory {
        variables.push((OsString::from("GIT_OBJECT_DIRECTORY"), objects.clone().into_os_string()));
    }
    let base: Vec<(OsString, OsString)> = std::env::vars_os()
        .filter(|(name, _)| { return !request.unset.contains(name); })
        .collect();
    let found: Answer = attempt(arguments.as_slice(), Some(request.cwd.as_path()), base, variables.as_slice())?;
    return Some(GitOutput { code: Some(found.code), stdout: found.stdout, stderr: Vec::new() });
}

/// Without file system readers for this platform, every request starts Git.
#[cfg(not(unix))]
pub fn try_request(_context: &GitContext, _request: &GitRequest) -> Option<GitOutput> {
    return None;
}

#[cfg(test)]
#[path = "repo_read_hook_tests.rs"]
mod tests;

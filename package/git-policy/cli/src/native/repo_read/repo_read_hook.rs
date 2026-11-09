//! What:
//!  The two places where the wrapper's Git runners try the in-process answers first.
//! Why:
//!  `run_metadata_git` (captured queries) and `run_git` (transaction requests) are the choke points every read-only
//!  Git start goes through,
//! so one call in each gives every caller the saving and keeps the fallback to Git in one place.
//!  Setting `CLI_GIT_IN_PROCESS_READS=off` in the wrapper's environment turns the answers off,
//! for a measurement that
//!  compares both and for diagnosing a difference.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = tryInProcess(args, overlay) ?? (await spawnGit(args, overlay));
//! ```

use crate::call_catching_panics::call_catching_panics;
use crate::git_metadata::MetadataOutput;
#[cfg(unix)]
use crate::repo_read_config_scan::ScopeEnvironment;
#[cfg(unix)]
use crate::repo_read_query::{Answer, Query, answer_with};
use crate::transaction_git_request_and_reply_types::{GitContext, GitOutput, GitRequest};
use std::ffi::OsString;
#[cfg(unix)]
use std::path::Path;

/// The wrapper environment variable that turns the in-process answers off when it holds `off`.
pub const SWITCH_VARIABLE: &str = "CLI_GIT_IN_PROCESS_READS";

/// Whether the answers are enabled in this process and for this run's added variables.
pub fn enabled(variables: &[(OsString, OsString)]) -> bool {
    for (name, value) in variables {
        if name == SWITCH_VARIABLE && value == "off" {
            return false;
        }
    }
    return match std::env::var_os(SWITCH_VARIABLE) {
        Some(value) => value != "off",
        None => true,
    };
}

/// What:
///  The overlay entry that makes a runner start Git for every command line.
/// Why:
///   Controls that count the starts of a stand-in Git,
/// or that need an unstartable Git to fail,
/// measure the process
///  path,
/// which stays the fallback for everything the readers decline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const switchOff = [['CLI_GIT_IN_PROCESS_READS', 'off']];
/// ```
#[cfg(test)]
pub fn switch_off() -> [(OsString, OsString); 1] {
    return [(OsString::from(SWITCH_VARIABLE), OsString::from("off"))];
}

/// What:
///  Run a reader and turn a panic inside it into "start Git".
/// Why:
///   The readers parse bytes of any repository;
/// a defect that panics must cost one Git start,
/// not the wrapper's command.
///  The release profile keeps `panic = "unwind"` for this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return read(state); } catch (error) { log(error); return undefined; }
/// ```
pub fn guarded<S, T>(state: S, read: fn(S) -> Option<T>) -> Option<T> {
    return match call_catching_panics(state, read) {
        Ok(found) => found,
        Err(payload) => {
            let message: &str = if let Some(text) = payload.downcast_ref::<&str>() {
                text
            } else if let Some(text) = payload.downcast_ref::<String>() {
                text.as_str()
            } else {
                "a panic without text"
            };
            crate::diagnostic_log::warn("repo-read-guard", format!("an in-process read panicked and Git answers instead: {message}").as_str());
            None
        }
    };
}

/// The answer of the readers for one query,
/// as the guard calls it with the query,
/// the base environment and the scope.
#[cfg(unix)]
fn answer_for_query((query, base, scope): (Query<'_>, Vec<(OsString, OsString)>, &ScopeEnvironment)) -> Option<Answer> {
    return answer_with(&query, base, scope);
}

/// The answer for a command line,
/// if the readers have one and the switch allows it.
#[cfg(unix)]
fn attempt(
    arguments: &[OsString],
    directory: Option<&Path>,
    base: Vec<(OsString, OsString)>,
    variables: &[(OsString, OsString)],
    input: Option<&[u8]>,
) -> Option<Answer> {
    if !enabled(variables) {
        return None;
    }
    let mut effective: Vec<(OsString, OsString)> = base.clone();
    effective.extend(variables.iter().cloned());
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(effective.as_slice());
    return guarded((Query { arguments, directory, variables, input }, base, &scope), answer_for_query);
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
    let found: Answer = attempt(arguments, None, std::env::vars_os().collect(), overlay, None)?;
    return Some(MetadataOutput { success: found.code == 0, code: Some(found.code), stdout: found.stdout, stderr: found.stderr });
}

/// Without file system readers for this platform,
/// every query starts Git.
#[cfg(not(unix))]
pub fn try_metadata(_arguments: &[OsString], _overlay: &[(OsString, OsString)]) -> Option<MetadataOutput> {
    return None;
}

/// What:
///  The answer to a query run in `directory` with a complete `environment`,
/// as the auto-push coordination runs them.
/// Why:
///   That runner builds the whole child environment itself,
/// so nothing of this process's own is added to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryComplete(args, directory, environment): { code: number; stdout: Uint8Array; stderr: Uint8Array } | undefined
/// ```
#[cfg(unix)]
pub fn try_complete(arguments: &[OsString], directory: &Path, environment: &[(OsString, OsString)]) -> Option<(Option<i32>, Vec<u8>, Vec<u8>)> {
    let found: Answer = attempt(arguments, Some(directory), Vec::new(), environment, None)?;
    return Some((Some(found.code), found.stdout, found.stderr));
}

/// Without file system readers for this platform,
/// every query starts Git.
#[cfg(not(unix))]
pub fn try_complete(_arguments: &[OsString], _directory: &std::path::Path, _environment: &[(OsString, OsString)]) -> Option<(Option<i32>, Vec<u8>, Vec<u8>)> {
    return None;
}

/// What:
///  The output of a transaction request,
/// if the readers can answer it.
/// Why:
///   The request names its directory,
/// its private index and object directory,
/// the variables to add and to remove,
///  and whether the invocation's global options apply;
/// the readers see exactly the command Git would have run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryRequest(context, request): GitOutput | undefined
/// ```
#[cfg(unix)]
pub fn try_request(context: &GitContext, request: &GitRequest) -> Option<GitOutput> {
    // The same assembly the started command uses, so an in-process answer reads what Git would.
    let arguments: Vec<OsString> = request.full_arguments(context.global_prefix.as_slice());
    // The context names the Git it runs, which the readers need to learn the system configuration file.
    // The context names the Git it runs, which the readers need to learn the system configuration file.
    let mut variables: Vec<(OsString, OsString)> = crate::git_system_config::with_real_git(context.overlay.as_slice(), context.real_git.as_path());
    variables.extend(request.environment.iter().cloned());
    if let Some(index) = &request.index {
        variables.push((OsString::from("GIT_INDEX_FILE"), index.clone().into_os_string()));
    }
    if let Some(objects) = &request.object_directory {
        variables.push((OsString::from("GIT_OBJECT_DIRECTORY"), objects.clone().into_os_string()));
    }
    let mut base: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in std::env::vars_os() {
        if !request.unset.contains(&name) {
            base.push((name, value));
        }
    }
    let found: Answer = attempt(arguments.as_slice(), Some(request.cwd.as_path()), base, variables.as_slice(), request.input.as_deref())?;
    return Some(GitOutput { code: Some(found.code), stdout: found.stdout, stderr: found.stderr });
}

/// Without file system readers for this platform,
/// every request starts Git.
#[cfg(not(unix))]
pub fn try_request(_context: &GitContext, _request: &GitRequest) -> Option<GitOutput> {
    return None;
}

#[cfg(test)]
#[path = "repo_read_hook_tests.rs"]
mod tests;

/// Differential controls for the in-process `cat-file --batch` request.
#[cfg(test)]
#[path = "repo_read_batch_tests.rs"]
mod batch_tests;

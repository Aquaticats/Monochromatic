//! What:
//!  The Git questions one invocation capture asks,
//!  and the words a question that could not start is reported with.
//! Why:
//!  Every question here runs in the invocation directory without the invocation's global options,
//!  and it still names the repository the caller named,
//!  so a repository whose Git directory stands beside its work tree is found.
//!  A question whose nonzero exit is an answer
//!  (`rev-parse --verify --quiet` for a ref that names nothing)
//!  returns its output either way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const asked = await runTransactionGit({ cwd, args, withoutPrefix: true, allowFailure: true });
//! ```

/// Trim with JavaScript's whitespace rules,
/// as `String.prototype.trim` does.
use crate::js_text::trim_javascript;
/// The transaction Git wrapper:
///  the request a question is asked with,
///  the failure its answer is reported as,
///  and the two starts that differ only in what a nonzero exit means.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
    GitRequest,
    run_git,
    run_git_checked,
};
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  One required Git report,
/// decoded and trimmed.
/// Why:
///   Git output is strict UTF-8 in the incumbent;
///  a non-UTF-8 report is a failure,
/// not a lossy text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder('utf-8', { fatal: true }).decode(bytes).trim()
/// ```
pub(crate) fn decoded_trimmed(output: &GitOutput) -> Result<String, GitFailure> {
    // `from_utf8` reports invalid bytes instead of replacing them.
    let text: &str = match std::str::from_utf8(output.stdout.as_slice()) {
        Ok(decoded) => decoded,
        Err(_) => return Err(GitFailure(String::from("Git returned non-UTF-8 output."))),
    };
    return Ok(String::from(trim_javascript(text)));
}

/// What:
///  A Git request in `cwd` that keeps none of the invocation's global options,
///  and still names the repository the caller named.
/// Why:
///   The incumbent runs these queries in the effective directory without the global options;
///  a repository whose Git directory is beside its work tree is found only when it is named,
///  so the selection travels with the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runTransactionGit({ gitPath, cwd, args, repository: selection({ cwd, globalPrefix }) })
/// ```
fn transaction_request<S: AsRef<OsStr>>(
    context: &GitContext,
    cwd: &Path,
    arguments: &[S],
) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, arguments);
    request.without_prefix = true;
    request.repository_selection =
        crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    return request;
}

/// What:
///  Runs a query whose nonzero exit is an answer,
///  returning the output whatever its status.
/// Why:
///   `allowFailure: true` in the incumbent:
///  a missing ref or a detached `HEAD` is not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args, allowFailure: true })
/// ```
pub(crate) fn probe<S: AsRef<OsStr>>(
    context: &GitContext,
    cwd: &Path,
    arguments: &[S],
) -> Result<GitOutput, GitFailure> {
    return match run_git(context, &transaction_request(context, cwd, arguments)) {
        Ok(output) => Ok(output),
        Err(error) => {
            // The words are a message here, so a name that is not text is shown as the shell reads it back.
            let mut shown: Vec<String> = Vec::with_capacity(arguments.len());
            for argument in arguments {
                shown.push(crate::shell_quote::shell_quote(
                    crate::path_bytes::argument_bytes(argument.as_ref()),
                ));
            }
            Err(GitFailure(format!(
                "git {} could not start: {error}",
                shown.join(" ")
            )))
        }
    };
}

/// What:
///  Runs a query whose failure is an error.
/// Why:
///   Matches the incumbent's default of throwing on a nonzero exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args })
/// ```
pub(crate) fn required<S: AsRef<OsStr>>(
    context: &GitContext,
    cwd: &Path,
    arguments: &[S],
) -> Result<GitOutput, GitFailure> {
    return run_git_checked(context, &transaction_request(context, cwd, arguments));
}

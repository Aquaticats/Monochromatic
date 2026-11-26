//! What:
//!  The identities a commit the service lands records,
//!  and what building one came to:
//!  the author and committer it asks Git for,
//!  the ones an amend keeps from the commit it replaces,
//!  and the names and dates the command line overrides.
//! Why:
//!   A commit's identity is Git's own answer for the person, the environment and the configuration,
//!  and an amend keeps the author of the commit it replaces;
//!  so the questions are asked here,
//!  once,
//!  and both faces of the message build the same identity from them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const built: Built = await identityAwaiting(invocation, parsed);
//! ```

use crate::commit_transaction::{
    TransactionStop,
    failed,
};
use crate::service_commit_args::ServiceCommitArgs;
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::ffi::OsString;
use std::path::Path;

/// What a command line's message came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Built {
    /// The whole commit message.
    pub message: String,
    /// The author line a reused commit supplies,
    /// `Name <email> seconds zone`.
    pub reused_author: Option<String>,
}

/// One identity line (`Name <email> seconds zone`) from Git's own resolution of the variable,
/// with the given overrides;
///  nothing when Git cannot make one (a date it does not read),
/// so native `git commit` gives its own message.
pub fn identity(
    context: &GitContext,
    cwd: &Path,
    variable: &str,
    overrides: &[(&str, String)],
) -> Result<Option<String>, TransactionStop> {
    return identity_answered(run_git(
        context,
        &identity_request(cwd, variable, overrides),
    ));
}

/// What:
///  `identity` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot call the synchronous face,
/// which enters the runtime to wait for its child
///  (`transaction_git_awaiting.rs`),
/// and the identities are read from the landing once that landing is a task.
/// The question and the
///  reading of the answer are the synchronous face's,
/// so the two cannot differ in what they make of Git's words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await identityAwaiting(git, cwd, 'GIT_AUTHOR_IDENT', overrides)
/// ```
pub async fn identity_awaiting(
    context: &GitContext,
    cwd: &Path,
    variable: &str,
    overrides: &[(&str, String)],
) -> Result<Option<String>, TransactionStop> {
    return identity_answered(
        run_git_awaiting(context, &identity_request(cwd, variable, overrides)).await,
    );
}

/// What:
///  The question `identity` asks Git.
/// Why:
///  Both faces ask the one question,
/// so neither can read another variable or miss an override the other
///  carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd, args: ['var', variable], environment: overrides };
/// ```
fn identity_request(
    cwd: &Path,
    variable: &str,
    overrides: &[(&str, String)],
) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, &["var", variable]);
    for (name, value) in overrides {
        request
            .environment
            .push((OsString::from(name), OsString::from(value)));
    }
    return request;
}

/// What:
///  The identity line Git's answer names,
/// or nothing for a variable Git does not resolve.
/// Why:
///  Both faces read Git's answer through this one function,
/// so an unusable date and an identity Git
///  cannot make are the same answer for both of them,
/// and a failure names the same step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function identityAnswered(answered: GitOutput | Error): string | undefined; // throws on a failed start
/// ```
///
/// # Errors
///
/// `TransactionStop` naming the identity when Git cannot be started.
fn identity_answered(
    answered: std::io::Result<GitOutput>
) -> Result<Option<String>, TransactionStop> {
    let output = match answered {
        Ok(finished) => finished,
        Err(error) => {
            return Err(failed(format!(
                "reading the commit identity failed: {error:?}"
            )));
        }
    };
    if output.code != Some(0) {
        return Ok(None);
    }
    return Ok(Some(
        String::from_utf8_lossy(output.stdout.as_slice())
            .trim()
            .to_string(),
    ));
}

/// The author variables `--author` and `--date` stand for.
pub fn author_overrides(parsed: &ServiceCommitArgs) -> Vec<(&'static str, String)> {
    let mut overrides: Vec<(&'static str, String)> = Vec::new();
    if let Some(author) = parsed.author.as_deref() {
        let (name, rest): (&str, &str) = author.split_once('<').unwrap_or((author, ""));
        overrides.push(("GIT_AUTHOR_NAME", name.trim().to_string()));
        overrides.push((
            "GIT_AUTHOR_EMAIL",
            rest.trim_end().trim_end_matches('>').to_string(),
        ));
    }
    if let Some(date) = parsed.date.as_deref() {
        overrides.push(("GIT_AUTHOR_DATE", date.to_string()));
    }
    return overrides;
}

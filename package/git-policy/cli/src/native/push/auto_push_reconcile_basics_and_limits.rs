//! What:
//!  The words, limits and small Git steps that the reconcile of a rejected push shares: the line prefixes, the attempt
//!  limit, the stale-rejection test, the failed-attempt value, the ancestor and fetch queries, and the journal identifier.
//! Why:
//!  `auto_push_reconcile.rs` runs the loop, `auto_push_reconcile_merge_commit.rs` builds the merge commit and
//!  `auto_push_reconcile_branch_advance.rs` moves the branch after the push; all three need these, and none of them may
//!  depend on a file that depends on it, so they live in a file that depends on none of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const RECONCILE_LINE_PREFIX = 'monochromatic-git-policy-cli: auto-push: ';
//! export function wasRejectedAsStale(output: string): boolean;
//! ```

use crate::auto_push_single_flight::{
    PushAttempt, PushGit, SingleFlightError, git_command, git_output, run_query,
};
use crate::bounded_push::{STOP_GRACE, run_bounded_push};
use crate::push_deadline_and_patience_settings::push_patience;
use crate::journald_native::{CLIENT_IDENTIFIER, IDENTIFIER};
use crate::inner_git_words::record_text;
use std::ffi::OsString;
use crate::auto_push_destination::Destination;
/// Tag used for diagnostics from this module.
pub(crate) const TAG: &str = "monochromatic-git-policy-cli";

/// Prefix of every line this module adds to a push's output, so the terminal filter can keep it.
pub const RECONCILE_LINE_PREFIX: &str = "monochromatic-git-policy-cli: auto-push: ";

/// Start of the line that explains why a rejected push stays unpublished; the generic "run `git push`" note is then
/// wrong advice and is not printed.
pub const RECONCILE_FAILED_PREFIX: &str = "monochromatic-git-policy-cli: auto-push: not published: ";

/// How many times the remote tip is fetched, merged and pushed before the push is reported as failed.
pub const MAX_RECONCILE_ATTEMPTS: u32 = 10;

/// The marker Git prints for a push the remote refused as stale; the word is not translated.
pub(crate) const REJECTED_MARKER: &str = "[rejected]";

/// The marker Git prints for an update that the remote's own rules refused, such as GitHub's protected branch (GH006).
/// A remote that is updated by two pushes at once refuses the slower one this way, with no word about staleness.
pub(crate) const REMOTE_REJECTED_MARKER: &str = "[remote rejected]";

/// What:
///  Whether a push's output shows a refusal that a moved remote branch can explain: Git's stale `[rejected]`, or the
///  remote's own `[remote rejected]`.
/// Why:
///   Only a refusal of an update that raced another push can be cured by merging; an authentication failure or an offline
///  error must be reported as they are.
///  The message cannot tell a race from a rule that refuses every push, so the caller compares the remote tip with the
///  branch and returns the refusal unchanged when the remote has not moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wasRejectedAsStale = (output: string) => output.includes('[rejected]') || output.includes('[remote rejected]');
/// ```
pub fn was_rejected_as_stale(output: &str) -> bool {
    return output.contains(REJECTED_MARKER) || output.contains(REMOTE_REJECTED_MARKER);
}

/// A failed attempt that explains itself and keeps the push output that came before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { exitCode: 1, output: `${previous}\n${RECONCILE_FAILED_PREFIX}${reason}` }
/// ```
pub(crate) fn unpublished(failed_prefix: &str, previous: &str, exit_code: i64, reason: &str) -> PushAttempt {
    return PushAttempt {
        exit_code: if exit_code == 0 { 1 } else { exit_code },
        output: format!("{previous}\n{failed_prefix}{reason}"),
    };
}

/// Argument list from string literals and owned text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['merge-tree', '--write-tree', ...]
/// ```
pub(crate) fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// What:
///  Whether the first commit is the second or one of its ancestors, or nothing when Git cannot tell.
/// Why:
///   Exit `0` and `1` are the answers; any other exit means an object is missing, and that is not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function isAncestor({ ancestor, descendant }): Promise<boolean | undefined>;
/// ```
pub(crate) fn is_ancestor(
    git: &PushGit,
    ancestor: &str,
    descendant: &str,
) -> Result<Option<bool>, SingleFlightError> {
    let output = run_query(
        git,
        &argv(&["merge-base", "--is-ancestor", ancestor, descendant]),
    )?;
    return Ok(match output.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    });
}

/// What:
///  Fetch the remote branch into a private ref and return the commit it names.
/// Why:
///   `FETCH_HEAD` is shared by every fetch in the repository, so another process could replace it between the fetch
///  and the read; a ref named for this process cannot be replaced, and it is deleted before this returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fetchRemoteTip({ destination }): Promise<string | PushAttempt>;
/// ```
pub(crate) fn fetch_remote_tip(
    git: &PushGit,
    destination: &Destination,
) -> Result<Result<String, PushAttempt>, SingleFlightError> {
    let private_ref = format!("refs/cli-git/auto-push/fetch-{}", std::process::id());
    let refspec = format!("+{}:{private_ref}", destination.merge_ref);
    let arguments = argv(&[
        "fetch",
        "--no-tags",
        &destination.remote,
        &refspec,
    ]);
    // No low-speed abort of the wrapper's own is added (`bounded_push.rs`): the fetch waits while its connection is open.
    let command = git_command(git, &arguments);
    let fetched = run_bounded_push(command, push_patience(&git.environment), STOP_GRACE, git.environment.as_slice())?;
    if fetched.exit_code != 0 {
        return Ok(Err(fetched));
    }
    // The fetch carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
    record_text(&["fetch", "--no-tags", destination.remote.as_str(), refspec.as_str()], fetched.output.as_str());
    let tip = git_output(
        git,
        &argv(&[
            "rev-parse",
            "--verify",
            &format!("{private_ref}^{{commit}}"),
        ]),
    );
    // The private ref is scaffolding; a failed deletion leaves one small ref that the next run overwrites.
    let _deleted = run_query(git, &argv(&["update-ref", "-d", &private_ref]));
    return Ok(Ok(tip?));
}

/// What:
///  The journal identifier of the entries of a merged file: the wrapper's for a push a person typed, the service's for the
///  auto-push, which runs inside the service.
/// Why:
///   An entry belongs next to the others of the program that sent it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identifier = explicit ? 'monochromatic-git-policy-cli' : 'cli-git-service';
/// ```
pub fn journal_identifier(explicit: bool) -> &'static str {
    return if explicit { CLIENT_IDENTIFIER } else { IDENTIFIER };
}

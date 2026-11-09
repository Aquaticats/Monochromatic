//! What:
//!  The words a found record becomes:
//! the text of a failed backup push,
//! of a push the service did not make,
//! and of
//!  a push lost with its worktree,
//! with Git's own words indented under them.
//! Why:
//!   The reading and the claiming of a record and the words it is shown as are different concerns,
//! and apart
//!  neither file outgrows the line limit;
//! the wording keeps what a person is told in one place,
//! where the
//!  rules about naming the commit make sense together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = worded(branchRef, shortName(commit), cleanReason(gitSaid));
//! ```

use crate::auto_push::PUSH_FAILED_NOTE;
use crate::push_failure_notice::{NO_REASON, clean_reason};
use crate::service_messages::RESTART_COMMAND;

/// What:
///  The text shown for a failed backup push,
/// with the commit named as given and Git's words already cleaned.
/// Why:
///   It names the branch and the commit,
/// gives Git's own reason,
/// says the commit is safe on this computer,
/// and says what to
///  do;
/// the service runs with its own settings,
/// which is the first thing to check when a push that works in a terminal
///  fails here.
/// The command that shows it may be anyone's,
/// so it never calls the commit the reader's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worded({ branchRef, shown, reason })
/// ```
pub(crate) fn worded(branch_ref: &str, shown: &str, reason: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    let detail: String = if reason.is_empty() {
        NO_REASON.to_string()
    } else {
        format!("Git said:\n{}", indented(reason))
    };
    return format!(
        "The backup push of branch {branch} failed for commit {shown}, so that commit exists only on this computer for now. \
         It is saved in this repository and nothing was lost.\n{detail}\n\
         Fix what Git names, then run `git push`. If it says the remote has commits you do not have, run `git pull` first. \
         The commit service runs with its own settings, not your terminal's, so a push that works in your terminal can \
         fail here when it needs a variable only your terminal has; give the service such a variable with \
         `systemctl --user set-environment NAME=value` and then `{RESTART_COMMAND}`.\n{PUSH_FAILED_NOTE}"
    );
}

/// What:
///  The text shown for a failed backup push,
/// from Git's raw words.
/// Why:
///   The commit is named as the caller gives it,
/// so a caller that has the repository passes Git's own abbreviation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushFailureText({ branchRef, commit, said })
/// ```
pub fn push_failure_text(branch_ref: &str, commit: &str, said: &str) -> String {
    return worded(branch_ref, commit, clean_reason(said).as_str());
}

/// What:
///  The text shown for a push the service did not make,
/// with the commit named as given and the service's words already cleaned.
/// Why:
///   Nothing failed and nothing was sent,
/// so the text says that and not that the push was refused;
/// it names the branch and the
///  commit,
/// gives the service's reason,
/// says the commit is saved here,
/// and names both ways out:
/// a destination the service can
///  name,
/// or a push by hand.
/// The command that shows it may be anyone's,
/// so it never calls the commit the reader's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordedSkipped({ branchRef, shown, reason })
/// ```
pub(crate) fn worded_skipped(branch_ref: &str, shown: &str, reason: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    let detail: String = if reason.is_empty() {
        String::from("The commit service gave no reason.")
    } else {
        format!("The commit service said:\n{}", indented(reason))
    };
    return format!(
        "The backup push of branch {branch} for commit {shown} was not made, so that commit exists only on this computer for now. \
         It is saved in this repository and nothing was lost.\n{detail}\n\
         Either give the branch a destination the commit service can name (the `push.default` setting of `git help config` says \
         which values do), or push it yourself with `git push <remote> {branch}`."
    );
}

/// What:
///  The text shown for a push that had not finished when the worktree that made the commit was removed.
/// Why:
///   Nothing says whether the push ran,
/// failed or never started,
/// so the text says only what is known:
/// the commit is saved here
///  and the branch may not be on the remote.
pub(crate) fn worded_lost(branch_ref: &str, shown: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    return format!(
        "The backup push of branch {branch} for commit {shown} had not finished when the worktree that made the commit was \
         removed, so that commit may exist only on this computer for now. It is saved in this repository and nothing was lost. \
         Run `git push` in a worktree that has branch {branch} checked out, or `git push <remote> {branch}` from any worktree."
    );
}

/// The lines of `reason`,
/// each with four spaces in front,
/// joined by newlines.
fn indented(reason: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in reason.lines() {
        lines.push(format!("    {line}"));
    }
    return lines.join("\n");
}

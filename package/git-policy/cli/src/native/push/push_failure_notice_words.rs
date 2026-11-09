//! What:
//!  The words a found record becomes:
//! the text of a failed backup push,
//! of a push that did not finish,
//! of a push the service did not make,
//! and of
//!  a push lost with its worktree,
//! with Git's own words indented under them.
//!  What the push came to is a fact of the push's own outcome and never of the text Git or a remote hook printed,
//! so the framing of the notice follows it.
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
///  What a push that left a record came to.
/// Why:
///   A refusal proves nothing was delivered;
/// a stop or a failure that can pass proves nothing about the remote,
/// and the retry plan
///  decides what the person is told next;
/// the wording follows this,
/// never the text of Git's or a remote hook's output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushOutcome = 'refused' | 'unfinished' | 'unfinished-with-no-retry';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PushOutcome {
    /// Git refused the push,
    /// so the commit did not reach the remote.
    Refused,
    /// The push was stopped or failed for a reason that can pass,
    /// so whether the remote took the commit is not known,
    /// and the service keeps it in its retry plan.
    Unfinished,
    /// The push did not finish and no retry is planned,
    /// so nothing tries it again by itself.
    UnfinishedWithNoRetry,
}

/// The words every notice of a push that did not finish carries,
/// and no notice of a push Git or the server refused:
/// a stop or a failure that can pass proves nothing about the remote,
/// so the person is told that and what happens
///  next instead of being told the push failed for good.
pub const PUSH_UNFINISHED_NOTE: &str = "did not finish, so the remote may or may not have taken that commit. The commit is saved in this repository and nothing was lost";

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
pub(crate) fn worded(branch_ref: &str, shown: &str, outcome: PushOutcome, reason: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    if matches!(outcome, PushOutcome::Unfinished | PushOutcome::UnfinishedWithNoRetry) {
        return worded_unfinished(branch, shown, outcome, reason);
    }
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
pub fn push_failure_text(branch_ref: &str, commit: &str, outcome: PushOutcome, said: &str) -> String {
    return worded(branch_ref, commit, outcome, clean_reason(said).as_str());
}

/// What:
///  The text shown for a backup push that did not finish,
/// so whether the remote took the commit is not known.
/// Why:
///   A stop or a failure that can pass proves nothing here,
/// so the text says what is known and no more,
/// and it
///  says what happens next from the retry plan that still exists:
/// a plan kept is a retry the person can wait
///  for,
/// and no plan at all sends them to push by hand.
///  It carries no advice about the service's settings either:
/// nothing about them was refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordedUnfinished(branch, shown, outcome, reason)
/// ```
pub(crate) fn worded_unfinished(branch: &str, shown: &str, outcome: PushOutcome, reason: &str) -> String {
    let detail: String = if reason.is_empty() {
        NO_REASON.to_string()
    } else {
        format!("monochromatic-git-policy-cli and Git said:\n{}", indented(reason))
    };
    let (next, closing): (&str, &str) = match outcome {
        PushOutcome::UnfinishedWithNoRetry => (
            "and the commit service is not trying this push again by itself",
            "run `git push` when you can",
        ),
        _ => (
            "and the commit service keeps the push in its retry plan, so it tries again on its own",
            "if it does not show the commit on the remote, run `git push`",
        ),
    };
    return format!(
        "The backup push of branch {branch} for commit {shown} {PUSH_UNFINISHED_NOTE}, {next}.\n{detail}\n\
         That account is what monochromatic-git-policy-cli found and what Git said when the push ended; {closing}."
    );
}

/// What:
///  The text shown for what a push that succeeded did,
/// with the commit named as given and the service's words already cleaned.
/// Why:
///   Nothing failed,
/// so the text is the service's own account of what the push did and nothing about a backup
///  that did not happen:
/// it already begins with the tool's name and says which branch was merged,
/// what became of the local
///  branch and the command that finishes the move.
/// The command that ran the push has returned,
/// so this is the only way
///  those words reach the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordedReport(branchRef, shown, reason) // the service's words, as they are
/// ```
pub(crate) fn worded_report(branch_ref: &str, shown: &str, reason: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    if reason.is_empty() {
        return format!("The backup push of branch {branch} for commit {shown} was made, and the commit service gave no account of it.");
    }
    return format!("The backup push of branch {branch} for commit {shown} was made, and the commit service said:\n{}", indented(reason));
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

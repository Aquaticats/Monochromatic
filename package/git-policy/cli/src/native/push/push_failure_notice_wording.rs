//! What:
//!  The words a push record's notice is made of:
//! what the push came to,
//! Git's own reason with its credentials masked,
//! and the sentence that says what to do.
//! Why:
//!  A record outlives the push that wrote it,
//! and the notice is worded when a later command shows it,
//! so the wording is decided
//!  here from what the push came to and never from the text of Git's output,
//! which a remote's hook controls:
//! a hook
//!  that prints the wrapper's own delivery sentence changes no framing.
//!  It is a file of its own because the record's names,
//! its files and its reading are one job and its wording is
//!  another,
//! and the record module reads this one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! worded({ branchRef, shown, outcome, reason }); // the text one record is shown as
//! ```

/// The masking of a URL's user name and password,
/// which Git's words may carry.
use crate::lfs_output_mask::mask_userinfo_text;
/// The note a failed push ends with,
/// which says the commit stays local.
use crate::auto_push::PUSH_FAILED_NOTE;
/// The command a person restarts the service with,
/// named where a push failed on the service's own settings.
use crate::service_messages::RESTART_COMMAND;

/// What:
///  What a push that left a record came to.
/// Why:
///   A refusal proves nothing was delivered,
/// and a stop or a failure that can pass proves nothing about the remote;
///  the wording follows this,
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

/// Said when Git gave no words at all.
pub const NO_REASON: &str = "Git gave no reason.";

/// The most bytes of Git's own words a record keeps.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const LONGEST_REASON: usize = 4000;

/// What:
///  Git's words for the person:
/// the user name and password of every URL masked,
/// control characters dropped,
/// and the
///  length bounded.
/// Why:
///   The text came from a remote and from Git,
/// and is printed on a terminal;
/// a URL with a user name and password,
/// or an
///  escape sequence,
/// must not reach the screen or a log of it.
/// Only the credentials are masked,
/// never the line around them,
///  because the line often says why the push failed (the wrapper's own Git LFS failure line names the endpoint setting and
///  carries the exit status).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanReason(said)
/// ```
pub fn clean_reason(said: &str) -> String {
    let masked: String = mask_userinfo_text(said);
    let mut kept: Vec<String> = Vec::new();
    for line in masked.lines() {
        let mut plain: String = String::new();
        for character in line.chars() {
            if character == '\t' || !character.is_control() {
                plain.push(character);
            }
        }
        let trimmed: &str = plain.trim_end();
        if !trimmed.is_empty() {
            kept.push(trimmed.to_string());
        }
    }
    let mut text: String = kept.join("\n");
    if text.len() > LONGEST_REASON {
        let mut cut: usize = LONGEST_REASON;
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
        text.push_str("\n(Git's output is cut here.)");
    }
    return text;
}

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
///  What the push came to decides the framing,
/// and never the words of Git's output,
/// which a remote's hook controls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worded({ branchRef, shown, outcome, reason })
/// ```
pub fn worded(branch_ref: &str, shown: &str, outcome: PushOutcome, reason: &str) -> String {
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
///  The text shown for a backup push that did not finish,
/// so whether the remote took the commit is not known.
/// Why:
///   A stop proves nothing here:
/// the proof a stop runs is the push's own,
/// it never gave a yes,
/// so the text says what is known
///  and no more;
/// the push stays in the service's retry plan,
/// and that retry is what settles the
///  question.
///  It carries no advice about the service's settings either:
/// nothing about them was refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordedUnfinished(branch, shown, reason)
/// ```
pub fn worded_unfinished(branch: &str, shown: &str, outcome: PushOutcome, reason: &str) -> String {
    let detail: String = if reason.is_empty() {
        NO_REASON.to_string()
    } else {
        format!("monochromatic-git-policy-cli and Git said:\n{}", indented(reason))
    };
    // What happens next is a fact of the retry plan, and the plan may have been dropped by the round that wrote this.
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
        "The backup push of branch {branch} for commit {shown} did not finish, so the remote may or may not have taken that \
         commit. The commit is saved in this repository and nothing was lost, {next}.\n{detail}\n\
         That account is what monochromatic-git-policy-cli found and what Git said when the push ended; {closing}."
    );
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
pub fn worded_skipped(branch_ref: &str, shown: &str, reason: &str) -> String {
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
pub fn worded_lost(branch_ref: &str, shown: &str) -> String {
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

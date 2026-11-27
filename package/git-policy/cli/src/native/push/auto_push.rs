//! What:
//!  The commit service's push of a landed commit's branch:
//! which push runs,
//! what its output shows,
//!  and how a failure is reported without failing the commit,
//!  decided once in `auto_push_awaiting.rs`,
//! which holds both faces of the push and is where `push_once` runs.
//! Why:
//!  Every commit in this repository is backed up (`AGENTS.md` rule APG),
//!  the service pushes the branch its record names and never reads `HEAD`,
//!  prints only GitHub `remote:` lines on success and the full output on failure,
//!  and leaves a notice for the next command when the push fails.
//!  The service is the only pusher of the repositories it serves and coalesces pending pushes of one branch
//!  into one call,
//! so the push needs no lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await pushOnce({ gitPath, cwd }, branchRef, sentUpTo, blocked, undecided);
//! ```

/// The reconcile of a push that a moved remote branch rejected,
/// and the lines it adds to the output.
use crate::auto_push_reconcile::{
    RECONCILE_FAILED_PREFIX,
    RECONCILE_LINE_PREFIX,
};
/// The service's Git,
/// the failed attempt it reports and its error type.
use crate::auto_push_single_flight::{
    PushAttempt,
    SingleFlightError,
};
/// The verdicts a post-commit gate kept,
/// and where they make this push stop.
use crate::blocked_commit_lines::{
    branch_name,
    held_back_line,
    unanswered_line,
    undecided_line,
};
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
use crate::push_held_back_by_a_blocked_commit::WhereThePushStops;
/// The words a failed push that wrote nothing is reported with,
/// and the making of a record's words.
use crate::push_retry_failure::record_words;
/// The blocked verdicts of the worktree the push is for.
/// Owned OS text for Git arguments,
/// which the controls below build their environments from.
#[cfg(test)]
use std::ffi::OsString;
/// `Write` is the stderr sink the terminal lines go to.
use std::io::Write;

/// Tag used for diagnostics from this module.
pub(crate) const TAG: &str = "monochromatic-git-policy-cli";

/// Prefix of the GitHub server-side lines a clean push surfaces.
const REMOTE_LINE_PREFIX: &str = "remote: ";

/// Note printed after every failed push:
/// the commit stays local.
pub const PUSH_FAILED_NOTE: &str = "monochromatic-git-policy-cli: commit saved locally, but auto-push to origin failed; run `git push` when ready.";

/// What Git prints when another process holds the lock of a reference the push updates.
pub(crate) const REFERENCE_LOCK_REFUSAL: &str = "cannot lock ref";

/// How many times the service pushes again when a held reference lock refused the update of a remote-tracking reference.
pub const PUSH_LOCK_RETRIES: u32 = 3;

/// The pause before the first of those pushes;
/// each later one waits a multiple of it.
pub(crate) const PUSH_LOCK_PAUSE: std::time::Duration = std::time::Duration::from_millis(250);

/// What:
///  Which branch the auto-push took.
/// Why:
///   `Skipped` covers both "nowhere to back up to" and a setting under which Git pushes the branch nowhere;
/// the first
///  exits 0 silently,
/// the second with a note that the person must see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AutoPushOutcome = 'skipped' | 'held-back' | 'pushed' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoPushOutcome {
    /// Nothing was pushed.
    Skipped,
    /// Nothing was pushed because a post-commit verdict holds the branch back,
    /// or because a matching push's branch moved past the commit that push may send;
    /// the branch is tried again by itself.
    HeldBack,
    /// A successful push delivered the commit.
    Pushed,
    /// The push that would deliver the commit failed or could not run.
    Failed,
}

/// What:
///  The result of one auto-push attempt.
/// Why:
///   `shown` is what the terminal received,
/// so tests compare the surfaced text directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AutoPushResult = { outcome; exitCode; shown };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoPushResult {
    /// Which branch the auto-push took.
    pub outcome: AutoPushOutcome,
    /// Push exit code;
    /// `0` when skipped or pushed,
    /// Git's code (or `1`) on failure.
    pub exit_code: i64,
    /// Text surfaced to stderr;
    /// empty on a silent skip or a quiet clean push,
    /// the note on a skip with one.
    pub shown: String,
}

/// What:
///  Select the output a push shows:
/// only `remote:` lines after a clean push,
/// everything after a failure,
/// and the refusal of a reference lock in either case.
/// Why:
///   A rejection,
/// a hook refusal or an offline error stays diagnosable only when the full output shows.
///   A push whose remote-tracking reference Git could not update exits `0` because the commits were delivered,
/// so the refusal
///  is the only sign the local view is stale:
/// without it `git status` says the branch is ahead of a remote that already has the commits,
/// and nothing says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filterPushOutput({ output, exitCode }): string;
/// ```
pub fn filter_push_output(
    output: &str,
    exit_code: i64,
) -> String {
    if exit_code != 0 {
        return output.to_owned();
    }
    let mut kept: Vec<&str> = Vec::new();
    let refused: bool = output.contains(REFERENCE_LOCK_REFUSAL);
    for line in output.split('\n') {
        let about_the_refusal: bool = refused && line.contains(REFERENCE_LOCK_REFUSAL);
        if line.starts_with(REMOTE_LINE_PREFIX)
            || line.starts_with(RECONCILE_LINE_PREFIX)
            || about_the_refusal
        {
            kept.push(line);
        }
    }
    return kept.join("\n");
}

/// What:
///  Where a push of the branch sends it.
/// Why:
///   The commit the push sends is not the branch's tip when a post-commit verdict holds the branch back,
/// so the destination and the
///  source are decided apart:
/// the destination is the branch's,
/// and the source is the branch or the base below the blocked commits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Sent = { remote: string; targets: string[] };
/// ```
pub(crate) struct Sent {
    /// The remote the push goes to.
    pub(crate) remote: String,
    /// The remote references the branch goes to,
    /// or nothing to send when Git matches the branches itself (`push.default=matching`).
    pub(crate) targets: Vec<String>,
}

/// What a push plan decided before anything was pushed.
pub(crate) enum Planned {
    /// Nothing to push;
    /// the result says why.
    Skip(AutoPushResult),
    /// Push the branch to this destination.
    Push {
        /// The branch the commit landed on.
        branch_ref: String,
        /// The remote-tracking reference the branch's pushes update,
        /// and nothing when it has no upstream.
        upstream: Option<String>,
        /// Where the push goes.
        sent: Sent,
        /// Whether this push sets the branch's upstream,
        /// which it does only for a branch that has none.
        set_upstream: bool,
    },
}

/// What:
///  The refspec source of a push,
///  and whether it names the branch so Git can give it an upstream.
/// Why:
///   Git sets an upstream only from a refspec that names a branch,
///  and a branch may be named only while it still names the commit this push may send:
///  a branch a verdict stops before,
///  or one that moved after the reading,
///  is sent as that commit alone,
///  so the push that would set the upstream publishes nothing the branch holds beyond it,
///  and the upstream waits for the first push that can name the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refspec = { source, set_upstream };
/// ```
pub(crate) struct Refspec {
    /// The branch or the commit the refspec sends.
    pub(crate) source: String,
    /// Whether the source is the branch itself,
    /// which is what `--set-upstream` needs to set one.
    pub(crate) set_upstream: bool,
}

/// What:
///  The refspec of a push,
///  from the commit it may send and whether the branch still names that commit.
/// Why:
///   One place decides what the refspec sends and whether Git may set the branch's upstream,
///  so the two faces of the push cannot assemble them differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refspecOf = (pinned, branchRef, namesTheBranch) => namesTheBranch ? { source: branchRef, set_upstream: true } : { source: pinned, set_upstream: false };
/// ```
pub(crate) fn refspec_of(
    pinned: &str,
    branch_ref: &str,
    names_the_branch: bool,
) -> Refspec {
    return match names_the_branch {
        true => Refspec {
            source: branch_ref.to_owned(),
            set_upstream: true,
        },
        false => Refspec {
            source: pinned.to_owned(),
            set_upstream: false,
        },
    };
}

/// What:
///  The `git push` arguments that send `source` to the planned destination.
/// Why:
///   The source is the branch when nothing is blocked and the base below the blocked commits when something is,
/// and the destination
///  is the same either way;
/// the arguments are built here so the two cannot be assembled differently.
///   A push under `push.default=matching` names no refspec at all:
/// Git matches the branches itself,
///  which is why its source is sent only when the branch has not moved past it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argv = (source, sent) => sent.targets.length === 0 ? ['push', sent.remote] : ['push', ...(setUpstream ? ['--set-upstream'] : []), sent.remote, ...sent.targets.map((target) => `${source}:${target}`)];
/// ```
pub(crate) fn push_arguments(
    source: &str,
    sent: &Sent,
    set_upstream: bool,
) -> Vec<String> {
    if sent.targets.is_empty() {
        return vec![String::from("push"), sent.remote.clone()];
    }
    let mut arguments: Vec<String> = vec![String::from("push")];
    if set_upstream {
        arguments.push(String::from("--set-upstream"));
    }
    arguments.push(sent.remote.clone());
    for target in &sent.targets {
        arguments.push(format!("{source}:{target}"));
    }
    return arguments;
}

/// What:
///  The commit a push sends,
/// or nothing when it must send nothing at all.
/// Why:
///   A verdict on the branch's way out is what decides it,
/// and the one case Git cannot express as a refspec is a push that
///  matches branches itself:
/// nothing is sent then,
/// rather than the whole branch and the blocked commit with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourceOf = (stop, sent, sentUpTo) => stop.kind === 'nothing' ? sentUpTo : stop.pushable;
/// ```
///   A push that no verdict holds back sends the commit the branch named before this push read the worktree's verdicts,
/// never the branch's reference:
/// a commit that landed after that reading has no verdict in it,
/// its own gate may still
///  refuse it,
/// and sending the reference would publish it before that answer.
/// The push that gives a branch its upstream names the branch instead,
/// because Git sets an upstream
///  only from a refspec that names a branch;
/// the caller that builds the refspec decides that,
/// and it may name the branch only while the branch still names this commit
///  (`refspec_of`).
pub(crate) fn source_of(
    stop: &WhereThePushStops,
    sent: &Sent,
    sent_up_to: &str,
) -> Option<String> {
    return match stop {
        WhereThePushStops::Nothing => Some(sent_up_to.to_owned()),
        WhereThePushStops::Before(stopped) | WhereThePushStops::Undecided(stopped) => {
            match sent.targets.is_empty() {
                true => None,
                false => stopped.pushable.clone(),
            }
        }
        WhereThePushStops::Unanswered(_) => None,
    };
}

/// What:
///  The result of a matching push that sent nothing because the branch moved past the commit it may send.
/// Why:
///   A push under `push.default=matching` gives Git no refspec,
///  so Git resolves the branch itself and would send a commit that landed after the reading,
///  whose own gate may still keep it local.
///  Nothing is sent,
///  so nothing is lost,
///  and the retry plan the service keeps tries the branch again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const movedPastThePin = (branchRef, err) => ({ outcome: 'held-back', exitCode: 0, shown: line });
/// ```
pub(crate) fn moved_past_the_pin(
    branch_ref: &str,
    err: &mut dyn Write,
) -> AutoPushResult {
    let line: String = format!(
        "{} moved after this push read it, and `push.default=matching` gives Git no refspec, so nothing was sent; the commit service tries the branch again.",
        branch_name(branch_ref)
    );
    debug(TAG, line.as_str());
    // Terminal output error: the line is the only diagnostic of a push that sent nothing.
    let _written = writeln!(err, "{line}");
    return AutoPushResult {
        outcome: AutoPushOutcome::HeldBack,
        exit_code: 0,
        shown: line,
    };
}

/// What:
///  The plan that skips a branch with a note,
/// after writing the note to the terminal sink.
/// Why:
///   The note is the only diagnostic of a push that never ran,
/// so the result carries it as well,
/// for the caller that
///  reports it elsewhere than the sink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skippedWith(note: string, err: Writable): Planned;
/// ```
pub(crate) fn skipped_with(
    note: String,
    err: &mut dyn Write,
) -> Planned {
    // Terminal output error: the note itself is the only diagnostic; a failed write to stderr is not a push failure.
    let _written = writeln!(err, "{note}");
    return Planned::Skip(AutoPushResult {
        outcome: AutoPushOutcome::Skipped,
        exit_code: 0,
        shown: note,
    });
}

/// What:
///  The result of a push that sent nothing because a verdict holds the branch back.
/// Why:
///   The line the queue records for the person and the line every command prints are the same words,
/// and the outcome tells
///  the service to try the branch again rather than to report a failure or to settle the push as done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heldBack = (branchRef, stop, err) => ({ outcome: 'held-back', exitCode: 0, shown: line });
/// ```
pub(crate) fn held_back_result(
    branch_ref: &str,
    stop: &WhereThePushStops,
    err: &mut dyn Write,
) -> AutoPushResult {
    let line: String = match stop {
        WhereThePushStops::Nothing => String::new(),
        WhereThePushStops::Before(stopped) => held_back_line(branch_ref, stopped),
        WhereThePushStops::Undecided(stopped) => undecided_line(branch_ref, stopped),
        WhereThePushStops::Unanswered(stopped) => unanswered_line(branch_ref, stopped),
    };
    debug(
        TAG,
        format!("nothing is pushed of {branch_ref}: {line}").as_str(),
    );
    // Terminal output error: the line is the only diagnostic of a push that sent nothing.
    let _written = writeln!(err, "{line}");
    return AutoPushResult {
        outcome: AutoPushOutcome::HeldBack,
        exit_code: 0,
        shown: line,
    };
}

/// What:
///  The remote-tracking reference a push of this shape updates,
///  or nothing when no single reference is its.
/// Why:
///   The reference is written by hand only where the push sends the branch to its own name on the remote the branch tracks,
/// because that is the one case in which Git updates the branch's upstream ref:
/// a push that matches the branches
///  itself,
/// a push to another destination,
/// and a push whose remote is not the branch's tracked remote
///  (`branch.<name>.pushRemote` and `push.default=current`) all leave that reference to Git,
/// and writing it there would name a
///  commit the other remote never received.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ref = (upstream, sent, branchRef) => [sent.remote + '/' + short, 'refs/remotes/' + sent.remote + '/' + short].includes(upstream) && sent.targets.length === 1 && sent.targets[0] === branchRef ? upstream : undefined;
/// ```
pub(crate) fn pushed_tracking_ref<'a>(
    upstream: Option<&'a str>,
    sent: &Sent,
    branch_ref: &str,
) -> Option<&'a str> {
    let of: &'a str = upstream?;
    let target: &String = sent.targets.first()?;
    if sent.targets.len() != 1 || target.as_str() != branch_ref {
        return None;
    }
    let short: &str = branch_ref.strip_prefix("refs/heads/")?;
    // The push plan asks Git for the upstream and keeps the name Git answers with, which is the abbreviated `origin/main`; the
    //  full form is accepted as well, because a reader that spells it out names the same reference.
    let full: String = format!("refs/remotes/{}/{short}", sent.remote);
    let abbreviated: String = format!("{}/{short}", sent.remote);
    return (of == full || of == abbreviated).then_some(of);
}

/// What:
///  The result of the push that ran,
/// with the line that names what held it back when anything did.
/// Why:
///   A held-back push delivered the commits before the blocked one and is still not the backup the branch needs,
/// so the
///  outcome says `HeldBack` and the queue tries the branch again;
/// the person reads one line that names the commit and the
///  check,
/// and Git's own lines follow it when there are any.
///  `delivered` is the repository's proof that this branch arrived although Git's status says the push failed
/// (`matching_push_delivered_the_recorded_branch.rs`),
///  and a push it proves says `Pushed` with exit code `0` while Git's own lines about the
///  branches that were refused stay in the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reported = (attempt, delivered, stop, branchRef, err) => ({ outcome: failed ? 'failed' : held ? 'held-back' : 'pushed', exitCode, shown });
/// ```
pub(crate) fn reported(
    attempt: PushAttempt,
    delivered: bool,
    stop: &WhereThePushStops,
    branch_ref: &str,
    note: &str,
    err: &mut dyn Write,
) -> Result<AutoPushResult, SingleFlightError> {
    let failed: bool = attempt.exit_code != 0 && !delivered;
    let held_back: Option<String> = match stop {
        WhereThePushStops::Nothing => None,
        WhereThePushStops::Before(stopped) => Some(held_back_line(branch_ref, stopped)),
        WhereThePushStops::Undecided(stopped) => Some(undecided_line(branch_ref, stopped)),
        WhereThePushStops::Unanswered(stopped) => Some(unanswered_line(branch_ref, stopped)),
    };
    let said: String = filter_push_output(&attempt.output, attempt.exit_code);
    let mut shown: String = record_words(&attempt, held_back.as_deref(), said.as_str(), failed);
    // A push that delivered but could not write the branch's tracking reference is said even when it printed nothing: silence
    //  there is a reference that disagrees with the remote until a push or a fetch writes it.
    if !note.is_empty() {
        shown = match shown.is_empty() {
            true => note.to_owned(),
            false => format!("{shown}\n{note}"),
        };
    }
    if !shown.is_empty() {
        writeln!(err, "{shown}").map_err(SingleFlightError::Io)?;
    }
    if failed && !attempt.output.contains(RECONCILE_FAILED_PREFIX) {
        writeln!(err, "{PUSH_FAILED_NOTE}").map_err(SingleFlightError::Io)?;
    }
    return Ok(AutoPushResult {
        outcome: match (failed, held_back.is_some()) {
            (true, _) => AutoPushOutcome::Failed,
            (false, true) => AutoPushOutcome::HeldBack,
            (false, false) => AutoPushOutcome::Pushed,
        },
        exit_code: match delivered {
            true => 0,
            false => attempt.exit_code,
        },
        shown,
    });
}

/// Auto-push behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_tests.rs"]
mod tests;

/// Controls that the service never pushes where a plain `git push` of the branch would not.
#[cfg(test)]
#[path = "auto_push_semantics_tests.rs"]
mod semantics_tests;

//! What:
//!  The commit service's push of a landed commit's branch:
//! which push runs,
//! what its output shows,
//!  and how a failure is reported without failing the commit.
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

/// Where a plain push of a named branch goes.
use crate::auto_push_destination::{NamedPush, named_push};
/// The reconcile of a push that a moved remote branch rejected,
/// and the lines it adds to the output.
use crate::auto_push_reconcile::{RECONCILE_FAILED_PREFIX, RECONCILE_LINE_PREFIX, reconcile_divergent_push};
/// The same reconcile,
/// awaited by a caller that is itself a task of the runtime.
/// The service's Git,
/// the failed attempt it reports and its error type.
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, run_query};
/// The auto-push's bounded `git push` with the Git LFS upload.
use crate::lfs_bounded_push::run_push;
/// The verdicts a post-commit gate kept,
/// and where they make this push stop.
use crate::blocked_commit_lines::{held_back_line, undecided_line, unanswered_line};
use crate::blocked_commit_questions::QueueHistory;
use crate::push_held_back_by_a_blocked_commit::{WhereThePushStops, where_the_push_stops};
/// The blocked verdicts of the worktree the push is for.
use crate::work_item_holding::BlockedVerdict;
/// Whether a push Git reported as failed published the recorded branch all the same.
use crate::matching_push_delivered_the_recorded_branch::{MatchingPush, delivered_although_git_failed};
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// `Write` is the stderr sink the terminal lines go to.
use std::io::Write;

/// Tag used for diagnostics from this module.
pub(crate) const TAG: &str = "monochromatic-git-policy-cli";

/// Prefix of the GitHub server-side lines a clean push surfaces.
const REMOTE_LINE_PREFIX: &str = "remote: ";

/// Note printed after every failed push:
/// the commit stays local.
pub const PUSH_FAILED_NOTE: &str =
    "monochromatic-git-policy-cli: commit saved locally, but auto-push to origin failed; run `git push` when ready.";

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
    /// Nothing was pushed because a post-commit verdict holds the branch back;
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
pub fn filter_push_output(output: &str, exit_code: i64) -> String {
    if exit_code != 0 {
        return output.to_owned();
    }
    let mut kept: Vec<&str> = Vec::new();
    let refused: bool = output.contains(REFERENCE_LOCK_REFUSAL);
    for line in output.split('\n') {
        let about_the_refusal: bool = refused && line.contains(REFERENCE_LOCK_REFUSAL);
        if line.starts_with(REMOTE_LINE_PREFIX) || line.starts_with(RECONCILE_LINE_PREFIX) || about_the_refusal {
            kept.push(line);
        }
    }
    return kept.join("\n");
}

/// What:
///  Whether a read-only Git query exits zero.
/// Why:
///   A non-zero exit is an answer ("no upstream",
/// "no origin");
/// a failure to start Git is not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await gitQuerySucceeds({ gitPath, cwd, args }); // false on SubprocessError only
/// ```
fn query_succeeds(git: &PushGit, arguments: &[&str]) -> Result<bool, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return Ok(run_query(git, &argv)?.code == Some(0));
}

/// What:
///  Run one real `git push` and capture its interleaved output,
/// with one trailing line break removed,
///  stopping it when its deadline passes.
/// Why:
///   Git's output of stdout and stderr shares one file so the lines interleave in the order they were written,
///  the trailing line break is removed as the incumbent's captured output is,
///  and a push that stalls must not hold the service's push of the branch forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { exitCode, output } = await nanoSpawn(gitPath, pushArgs, { cwd }) // SubprocessError -> { exitCode ?? 1, output }
/// ```
fn run_push_command(git: &PushGit, arguments: &[&str]) -> Result<PushAttempt, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return run_push(git, &argv);
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
///  The `git push` arguments that send `source` to the planned destination.
/// Why:
///   The source is the branch when nothing is blocked and the base below the blocked commits when something is,
/// and the destination
///  is the same either way;
/// the arguments are built here so the two cannot be assembled differently.
///   A push under `push.default=matching` names no refspec at all:
/// Git matches the branches itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argv = (source, sent) => sent.targets.length === 0 ? ['push', sent.remote] : ['push', ...(setUpstream ? ['--set-upstream'] : []), sent.remote, ...sent.targets.map((target) => `${source}:${target}`)];
/// ```
pub(crate) fn push_arguments(source: &str, sent: &Sent, set_upstream: bool) -> Vec<String> {
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
/// The one push that gives a branch its upstream names the branch instead,
/// because Git sets an upstream
///  only from a refspec that names a branch;
/// the caller that builds the refspec decides that,
/// and every later
///  push of the branch is fixed.
pub(crate) fn source_of(stop: &WhereThePushStops, sent: &Sent, sent_up_to: &str) -> Option<String> {
    return match stop {
        WhereThePushStops::Nothing => Some(sent_up_to.to_owned()),
        WhereThePushStops::Before(stopped) | WhereThePushStops::Undecided(stopped) => match sent.targets.is_empty() {
            true => None,
            false => stopped.pushable.clone(),
        },
        WhereThePushStops::Unanswered(_) => None,
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
pub(crate) fn skipped_with(note: String, err: &mut dyn Write) -> Planned {
    // Terminal output error: the note itself is the only diagnostic; a failed write to stderr is not a push failure.
    let _written = writeln!(err, "{note}");
    return Planned::Skip(AutoPushResult { outcome: AutoPushOutcome::Skipped, exit_code: 0, shown: note });
}

/// What:
///  Decide how to push a recorded branch whatever the worktree has checked out:
/// no upstream and no origin skips,
/// otherwise
///  the push names the branch on both sides and never reads `HEAD`.
/// Why:
///   The service pushes after the command returned,
/// so the person may switch branches or detach the head at any instant,
///  including between a check and the push;
/// the record names the branch that received the commit,
/// and the plain
///  `git push` would publish the checked-out one instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = await planNamedPush({ gitPath, cwd }, branchRef);
/// ```
fn plan_named_push(git: &PushGit, branch_ref: &str, err: &mut dyn Write) -> Result<Planned, SingleFlightError> {
    // Git reads `refs/heads/x@{upstream}` as a branch named `refs/heads/x` ("no such branch"), so the query takes the short name;
    //  a ref outside `refs/heads` has no upstream.
    let upstream: Option<String> = match branch_ref.strip_prefix("refs/heads/") {
        Some(name) => upstream_of(git, name)?,
        None => None,
    };
    if upstream.is_none() && !query_succeeds(git, &["remote", "get-url", "origin"])? {
        debug(TAG, "no upstream and no origin remote; skipping auto-push");
        return Ok(Planned::Skip(AutoPushResult { outcome: AutoPushOutcome::Skipped, exit_code: 0, shown: String::new() }));
    }
    return Ok(match &upstream {
        Some(_upstream_ref) => match named_push(git, branch_ref)? {
            NamedPush::Push { remote, targets } => Planned::Push {
                branch_ref: branch_ref.to_owned(),
                upstream,
                sent: Sent { remote, targets },
                set_upstream: false,
            },
            // Git matches the branches itself: the plain push of the remote, with no refspec.
            NamedPush::Matching { remote } => Planned::Push {
                branch_ref: branch_ref.to_owned(),
                upstream,
                sent: Sent { remote, targets: Vec::new() },
                set_upstream: false,
            },
            NamedPush::Skip(note) => skipped_with(note, err),
        },
        // A branch with no upstream goes to `origin` under its own name and takes `origin` as its upstream.
        None => Planned::Push {
            branch_ref: branch_ref.to_owned(),
            upstream: None,
            sent: Sent { remote: String::from("origin"), targets: vec![branch_ref.to_owned()] },
            set_upstream: true,
        },
    });
}

/// What:
///  The remote-tracking reference a branch's pushes update,
/// and nothing when it has no upstream.
/// Why:
///   `@{upstream}` is Git's own answer,
/// so a branch whose upstream was never set is not guessed at from the settings,
/// and
///  the same answer decides where the push goes and whether a blocked commit is already published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const upstreamOf = (git, short) => querySuccess(git, ['rev-parse', '--abbrev-ref', '--symbolic-full-name', `${short}@{upstream}`]) || undefined;
/// ```
fn upstream_of(git: &PushGit, short: &str) -> Result<Option<String>, SingleFlightError> {
    let asked: String = format!("{short}@{{upstream}}");
    let arguments: Vec<OsString> = ["rev-parse", "--abbrev-ref", "--symbolic-full-name", asked.as_str()].iter().map(OsString::from).collect();
    let output = run_query(git, &arguments)?;
    if output.code != Some(0) {
        return Ok(None);
    }
    let named: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return Ok((!named.is_empty()).then_some(named));
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
pub(crate) fn held_back_result(branch_ref: &str, stop: &WhereThePushStops, err: &mut dyn Write) -> AutoPushResult {
    let line: String = match stop {
        WhereThePushStops::Nothing => String::new(),
        WhereThePushStops::Before(stopped) => held_back_line(branch_ref, stopped),
        WhereThePushStops::Undecided(stopped) => undecided_line(branch_ref, stopped),
        WhereThePushStops::Unanswered(stopped) => unanswered_line(branch_ref, stopped),
    };
    debug(TAG, format!("nothing is pushed of {branch_ref}: {line}").as_str());
    // Terminal output error: the line is the only diagnostic of a push that sent nothing.
    let _written = writeln!(err, "{line}");
    return AutoPushResult { outcome: AutoPushOutcome::HeldBack, exit_code: 0, shown: line };
}

/// What:
///  Push a landed commit's branch once,
/// with no coordination with other processes.
/// Why:
///   The service's push consumer is the only pusher of the repositories it serves and coalesces pending pushes of one
///  branch into this single call,
/// so it needs no lock.
///   `sent_up_to` is the commit the branch named before `blocked` and `undecided` were read,
/// and it is what a push that no verdict holds back sends:
/// the branch may move between that reading and this push,
/// a commit that lands in between has no verdict in the reading,
/// and sending the branch's reference would publish it before its own gate answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = await pushOnce({ gitPath, cwd }, branchRef, sentUpTo, blocked, undecided);
/// ```
pub fn push_once(
    git: &PushGit,
    branch_ref: &str,
    sent_up_to: &str,
    blocked: &[BlockedVerdict],
    undecided: &[String],
    err: &mut dyn Write,
) -> Result<AutoPushResult, SingleFlightError> {
    // The record names the branch, and the worktree may switch away from it at any instant, including between a check of
    //  `HEAD` and the push itself: the plan never reads `HEAD`, so what is pushed is always the recorded branch.
    let planned: Planned = plan_named_push(git, branch_ref, err)?;
    let (pushed_ref, upstream, sent, set_upstream): (String, Option<String>, Sent, bool) = match planned {
        Planned::Skip(result) => return Ok(result),
        Planned::Push { branch_ref: planned_ref, upstream, sent, set_upstream } => (planned_ref, upstream, sent, set_upstream),
    };
    // A post-commit verdict on the branch's way out decides whether this push may send the branch's tip or only the base below it,
    //  and so does a commit whose own checks still run: its answer may keep it local, and a push of the tip would publish it.
    let stop: WhereThePushStops = where_the_push_stops(&QueueHistory { git }, pushed_ref.as_str(), upstream.as_deref(), blocked, undecided);
    let Some(pinned) = source_of(&stop, &sent, sent_up_to) else {
        return Ok(held_back_result(pushed_ref.as_str(), &stop, err));
    };
    // Git sets a branch's upstream only from a refspec that names a branch, and a push of a commit gives the branch none
    //  (measured with Git 2.56.0: `git push --set-upstream origin <commit>:refs/heads/new` delivers the commit and leaves
    //  `branch.new.merge` unset). The one push that gives a branch its upstream therefore sends the branch, and every other push
    //  sends the commit that push was allowed to send.
    let source: String = match set_upstream {
        true => pushed_ref.clone(),
        false => pinned,
    };
    let pushing_whole: bool = stop == WhereThePushStops::Nothing;
    let arguments: Vec<String> = push_arguments(source.as_str(), &sent, set_upstream);
    let argument_views: Vec<&str> = arguments.iter().map(String::as_str).collect();
    debug(TAG, &format!("pushing {sent_up_to} with {argument_views:?} on {pushed_ref}"));
    let mut attempt: PushAttempt = run_push_command(git, argument_views.as_slice())?;
    // The instances of two worktrees share the repository's references. While another holds the lock of the remote-tracking
    //  reference this push updates, Git still delivers the push, prints the refusal to update the reference, and exits 0, which
    //  leaves the reference stale; the same push again finds nothing to deliver and brings the reference up to date.
    let mut pause: u32 = 0;
    while attempt.output.contains(REFERENCE_LOCK_REFUSAL) && pause < PUSH_LOCK_RETRIES {
        pause += 1;
        std::thread::sleep(PUSH_LOCK_PAUSE * pause);
        attempt = run_push_command(git, argument_views.as_slice())?;
    }
    // The push delivered, and Git may have been refused the update of the branch's remote-tracking reference while another Git
    //  held its lock. The repeats above bring the reference up to date when that lock went in the meantime, and one that outlives
    //  them leaves the reference behind, so it is written here and the state is said when it could not be
    //  (`push_tracking_ref_repair.rs`). Nothing a person asked for waits on it: the push is over.
    let mut note: String = String::new();
    if attempt.exit_code == 0
        && let Some(of) = pushed_tracking_ref(upstream.as_deref(), &sent, pushed_ref.as_str())
    {
        // The reference is written by its full name: the name a branch's upstream is answered with, `origin/main`, is no ref.
        let full: String = match crate::push_tracking_ref_repair::resolve(git, of) {
            Some(named) => named,
            None => of.to_owned(),
        };
        if !crate::push_tracking_ref_repair::bring_up_to_date(git, full.as_str(), source.as_str()) {
            note = crate::push_tracking_ref_repair::still_behind(full.as_str(), source.as_str());
        }
    }
    // A push held back by a verdict is never reconciled: the merge the reconcile publishes is made above the local tip, so it would
    //  carry the very commit the gate refused. It stays a plain push of the base, and the branch is tried again by itself.
    if upstream.is_some() && pushing_whole && attempt.exit_code != 0 {
        attempt = reconcile_divergent_push(git, &pushed_ref, sent_up_to, attempt)?;
    }
    // Under `push.default=matching` this one push carried every branch the remote has, so Git's status is the worst of them: a
    //  sibling the remote refused makes it non-zero while this branch arrived. The remote says which, and a delivery it proves is
    //  not reported as a failure nor retried.
    let delivered: bool = delivered_although_git_failed(
        git,
        &MatchingPush {
            git_matched: sent.targets.is_empty(),
            remote: sent.remote.as_str(),
            branch_ref: pushed_ref.as_str(),
            exit_code: attempt.exit_code,
            landed_oid: sent_up_to,
        },
    )?;
    return reported(attempt, delivered, &stop, pushed_ref.as_str(), note.as_str(), err);
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
pub(crate) fn pushed_tracking_ref<'a>(upstream: Option<&'a str>, sent: &Sent, branch_ref: &str) -> Option<&'a str> {
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
pub(crate) fn reported(attempt: PushAttempt, delivered: bool, stop: &WhereThePushStops, branch_ref: &str, note: &str, err: &mut dyn Write) -> Result<AutoPushResult, SingleFlightError> {
    let failed: bool = attempt.exit_code != 0 && !delivered;
    let held_back: Option<String> = match stop {
        WhereThePushStops::Nothing => None,
        WhereThePushStops::Before(stopped) => Some(held_back_line(branch_ref, stopped)),
        WhereThePushStops::Undecided(stopped) => Some(undecided_line(branch_ref, stopped)),
        WhereThePushStops::Unanswered(stopped) => Some(unanswered_line(branch_ref, stopped)),
    };
    let said: String = filter_push_output(&attempt.output, attempt.exit_code);
    let mut shown: String = match (held_back.clone(), said.is_empty()) {
        (None, _) => said,
        (Some(line), true) => line,
        (Some(line), false) => format!("{line}\n{said}"),
    };
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

/// The same push as a task of the runtime.
pub use crate::auto_push_awaiting::push_once_awaiting;

/// Auto-push behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_tests.rs"]
mod tests;

/// Controls that the service never pushes where a plain `git push` of the branch would not.
#[cfg(test)]
#[path = "auto_push_semantics_tests.rs"]
mod semantics_tests;

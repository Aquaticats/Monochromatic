//! What:
//!  The words,
//! limits and small Git steps that the reconcile of a rejected push shares:
//! the line prefixes,
//! the attempt
//!  limit,
//! the stale-rejection test,
//! the failed-attempt value,
//! the ancestor and fetch queries,
//! and the journal identifier.
//! Why:
//!  `auto_push_reconcile.rs` runs the loop,
//! `auto_push_reconcile_merge_commit.rs` builds the merge commit and
//!  `auto_push_reconcile_branch_advance.rs` moves the branch after the push;
//! all three need these,
//! and none of them may
//!  depend on a file that depends on it,
//! so they live in a file that depends on none of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const RECONCILE_LINE_PREFIX = 'monochromatic-git-policy-cli: auto-push: ';
//! export function wasRejectedAsStale(output: string): boolean;
//! ```

use crate::auto_push_single_flight::{
    PushAttempt, PushGit, SingleFlightError, git_command, git_output_awaiting, run_query, run_query_awaiting,
};
/// The synchronous reading of a start's output,
/// which only the controls that compare the two faces still ask.
#[cfg(test)]
use crate::auto_push_single_flight::git_output;
use crate::bounded_push::{STOP_GRACE, run_bounded_push_awaiting};
/// The synchronous push of the fetch,
/// which only the control that compares the two faces still asks.
#[cfg(test)]
use crate::bounded_push::run_bounded_push;
use crate::bounded_query::QueryOutput;
use crate::push_deadline_and_patience_settings::push_patience;
use crate::journald_native::{CLIENT_IDENTIFIER, IDENTIFIER};
use crate::inner_git_words::record_text;
use std::ffi::OsString;
use crate::auto_push_destination::Destination;
use crate::diagnostic_log::debug;
/// Tag used for diagnostics from this module.
pub(crate) const TAG: &str = "monochromatic-git-policy-cli";

/// Prefix of every line this module adds to a push's output,
/// so the terminal filter can keep it.
pub const RECONCILE_LINE_PREFIX: &str = "monochromatic-git-policy-cli: auto-push: ";

/// Start of the line that explains why a rejected push stays unpublished;
/// the generic "run `git push`" note is then
/// wrong advice and is not printed.
pub const RECONCILE_FAILED_PREFIX: &str = "monochromatic-git-policy-cli: auto-push: not published: ";

/// What:
///  The start of every line a push report adds under this label.
/// Why:
///   The reconcile loop and the route that hands it a failed push word their lines once,
/// so neither can print a prefix the other does not.
///
pub fn run_line_prefix(label: &str) -> String {
    return format!("monochromatic-git-policy-cli: {label}: ");
}

/// How many times the remote tip is fetched,
/// merged and pushed before the push is reported as failed.
pub const MAX_RECONCILE_ATTEMPTS: u32 = 10;

/// The marker Git prints for a push the remote refused as stale;
/// the word is not translated.
pub(crate) const REJECTED_MARKER: &str = "[rejected]";

/// The marker Git prints for an update that the remote's own rules refused,
/// such as GitHub's protected branch (GH006).
/// A remote that is updated by two pushes at once refuses the slower one this way,
/// with no word about staleness.
pub(crate) const REMOTE_REJECTED_MARKER: &str = "[remote rejected]";

/// What:
///  Whether a push's output shows a refusal that a moved remote branch can explain:
/// Git's stale `[rejected]`,
/// or the
///  remote's own `[remote rejected]`.
/// Why:
///   Only a refusal of an update that raced another push can be cured by merging;
/// an authentication failure or an offline
///  error must be reported as they are.
///  The message cannot tell a race from a rule that refuses every push,
/// so the caller compares the remote tip with the
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
///  Whether the first commit is the second or one of its ancestors,
/// or nothing when Git cannot tell.
/// Why:
///   Exit `0` and `1` are the answers;
/// any other exit means an object is missing,
/// and that is not an answer.
///
/// Why a caller still asks this face:
///   The loop awaits this question (`is_ancestor_awaiting`),
/// and the automatic push that is not a task of the runtime asks it of the Git it pushes with,
/// where a start of Git is allowed
/// (`QueueHistory` in `push/blocked_commit_questions.rs`,
/// called by `push/auto_push.rs`),
/// as does the control that compares the two faces.
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
    return Ok(ancestry_of(&output));
}

/// What:
///  Whether the first commit is the second or one of its ancestors,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's reconcile relates the branch to the remote tip as a task,
/// and the synchronous query enters the runtime and is refused from there.
/// The question and the reading of the exit status are the synchronous face's,
/// so the two cannot answer differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await isAncestorOnTheRuntime({ ancestor, descendant }): Promise<boolean | undefined>;
/// ```
pub async fn is_ancestor_awaiting(
    git: &PushGit,
    ancestor: &str,
    descendant: &str,
) -> Result<Option<bool>, SingleFlightError> {
    let output = run_query_awaiting(
        git,
        &argv(&["merge-base", "--is-ancestor", ancestor, descendant]),
    )
    .await?;
    return Ok(ancestry_of(&output));
}

/// What:
///  The answer one `merge-base --is-ancestor` gave.
/// Why:
///   Both faces of `is_ancestor` read one exit status here:
/// `0` and `1` are the answers,
/// and any other exit means an object is missing,
/// which is not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// code === 0 ? true : code === 1 ? false : undefined;
/// ```
fn ancestry_of(output: &QueryOutput) -> Option<bool> {
    return match output.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    };
}

/// What:
///  The namespace of the reconcile's private fetch refs.
/// Why:
///   Every ref under it is the reconcile's own scaffolding,
/// named for the process that fetched,
/// so the sweep can tell its own leftovers from the rest of the reference space.
pub(crate) const PRIVATE_FETCH_PREFIX: &str = "refs/cli-git/auto-push/fetch-";

/// The namespace those refs live in,
/// which is the pattern that lists them:
/// `for-each-ref` reads a pattern without a glob as an exact name,
/// so the namespace is given with its trailing slash.
const PRIVATE_FETCH_NAMESPACE: &str = "refs/cli-git/auto-push/";

/// What:
///  Delete the private fetch refs of reconciles that are not running any more.
/// Why:
///   A ref named for a dead process is overwritten by nothing:
/// the instance that wrote it is gone,
/// so it stays for good,
/// pins the objects it names against `gc`,
/// and a push of every ref would carry it.
/// A ref whose process still runs belongs to another worktree of this repository reconciling at the same time,
/// so it is left where it is.
/// This process's own leftovers are taken too:
/// its fetch writes the same name again,
/// and a kill between the fetch and the deletion leaves one.
///
/// Why a caller that is not the loop still asks this face:
///   The loop sweeps through the awaited face (`sweep_private_fetch_refs_awaiting`),
/// and the delivery proof of a failed matching push sweeps and fetches through this one
/// (`matching_push_delivered_the_recorded_branch.rs`),
/// whose control compares the two faces as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const ref of await git.for_each_ref(PRIVATE_FETCH_PREFIX)) if (!isAlive(pidOf(ref)) || pidOf(ref) === process.pid) await git.update_ref(['-d', ref]);
/// ```
pub(crate) fn sweep_private_fetch_refs(git: &PushGit) -> Result<(), SingleFlightError> {
    let listed = run_query(git, &argv(&["for-each-ref", "--format=%(refname)", PRIVATE_FETCH_NAMESPACE]))?;
    if listed.code != Some(0) {
        debug(TAG, "the private fetch refs could not be listed; the fetch goes on without the sweep");
        return Ok(());
    }
    let ours: u32 = std::process::id();
    for line in String::from_utf8_lossy(&listed.stdout).lines() {
        let name: &str = line.trim();
        let Some(number) = name.strip_prefix(PRIVATE_FETCH_PREFIX) else { continue };
        // A name this release did not write is not scaffolding it can judge.
        let Ok(pid) = number.parse::<u32>() else { continue };
        if pid != ours && process_is_alive(u64::from(pid)) {
            continue;
        }
        let _deleted = run_query(git, &argv(&["update-ref", "-d", name]));
    }
    return Ok(());
}

/// Whether the process number `pid` still names a process,
/// as the owner records of a transaction ask.
pub(crate) fn process_is_alive(pid: u64) -> bool {
    let Ok(number) = i64::try_from(pid) else { return false };
    return crate::process_identity::signal_probe(number).unwrap_or(true);
}

/// What:
///  Fetch the remote branch into a private ref and return the commit it names.
/// Why:
///   `FETCH_HEAD` is shared by every fetch in the repository,
/// so another process's `git pull` may be reading it,
///  and the fetch writes none of it (`--no-write-fetch-head`);
/// a ref named for this process cannot be replaced by another's fetch either,
///  and it is deleted before this returns.
///   The refs a killed instance left are swept first,
/// so a leak lasts until the next fetch instead of for good.
///
/// Why a caller that is not the loop still asks this face:
///   The loop fetches through the awaited face (`fetch_remote_tip_awaiting`),
/// and the delivery proof of a failed matching push fetches the tip its own question needs through this one
/// (`matching_push_delivered_the_recorded_branch.rs`),
/// whose control compares the two faces as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fetchRemoteTip({ destination }): Promise<string | PushAttempt>;
/// ```
pub(crate) fn fetch_remote_tip(
    git: &PushGit,
    destination: &Destination,
) -> Result<Result<String, PushAttempt>, SingleFlightError> {
    sweep_private_fetch_refs(git)?;
    let private_ref = format!("{PRIVATE_FETCH_PREFIX}{}", std::process::id());
    let refspec = format!("+{}:{private_ref}", destination.merge_ref);
    let arguments = argv(&[
        "fetch",
        "--no-tags",
        "--no-write-fetch-head",
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
    record_text(&["fetch", "--no-tags", "--no-write-fetch-head", destination.remote.as_str(), refspec.as_str()], fetched.output.as_str());
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
///  The same sweep,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's reconcile sweeps the scaffolding of a killed instance as a task,
/// and the synchronous queries enter the runtime and are refused from there.
/// The listing,
/// the rule that decides which refs are this process's and the deletion are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const ref of await listing) if (!isAlive(pidOf(ref)) || pidOf(ref) === process.pid) await updateRef(['-d', ref]);
/// ```
pub async fn sweep_private_fetch_refs_awaiting(git: &PushGit) -> Result<(), SingleFlightError> {
    let listed = run_query_awaiting(git, &argv(&["for-each-ref", "--format=%(refname)", PRIVATE_FETCH_NAMESPACE])).await?;
    if listed.code != Some(0) {
        debug(TAG, "the private fetch refs could not be listed; the fetch goes on without the sweep");
        return Ok(());
    }
    let ours: u32 = std::process::id();
    for name in refs_to_sweep(listed.stdout.as_slice(), ours) {
        let _deleted = run_query_awaiting(git, &argv(&["update-ref", "-d", name.as_str()])).await;
    }
    return Ok(());
}

/// What:
///  The private fetch refs one finished listing says this process may delete:
/// its own leftovers and those of a process that is gone.
/// Why:
///   Both faces of the sweep keep and delete the same refs here,
/// and a ref whose process still runs belongs to another worktree of this repository reconciling at the
///  same time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing.split('\n').map((line) => line.trim()).filter((name) => !foreignOrUnnamed(name, ours));
/// ```
fn refs_to_sweep(listing: &[u8], ours: u32) -> Vec<String> {
    let mut sweepable: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(listing).lines() {
        let name: &str = line.trim();
        let Some(number) = name.strip_prefix(PRIVATE_FETCH_PREFIX) else { continue };
        // A name this release did not write is not scaffolding it can judge.
        let Ok(pid) = number.parse::<u32>() else { continue };
        if pid != ours && process_is_alive(u64::from(pid)) {
            continue;
        }
        sweepable.push(name.to_owned());
    }
    return sweepable;
}

/// What:
///  The same fetch,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's reconcile fetches the remote tip as a task,
/// and the fetch and the reads around it enter the runtime on the synchronous face and are
///  refused from there.
/// The ref name,
/// what the fetch says and the deletion are the synchronous face's,
/// so the two cannot leave different scaffolding behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await fetchRemoteTipOnTheRuntime({ destination }): Promise<string | PushAttempt>;
/// ```
pub async fn fetch_remote_tip_awaiting(
    git: &PushGit,
    destination: &Destination,
) -> Result<Result<String, PushAttempt>, SingleFlightError> {
    sweep_private_fetch_refs_awaiting(git).await?;
    let private_ref = format!("{PRIVATE_FETCH_PREFIX}{}", std::process::id());
    let refspec = format!("+{}:{private_ref}", destination.merge_ref);
    let arguments = argv(&[
        "fetch",
        "--no-tags",
        &destination.remote,
        &refspec,
    ]);
    // No low-speed abort of the wrapper's own is added (`bounded_push.rs`): the fetch waits while its connection is open.
    let command = git_command(git, &arguments);
    let fetched = run_bounded_push_awaiting(command, push_patience(&git.environment), STOP_GRACE, git.environment.as_slice()).await?;
    if fetched.exit_code != 0 {
        return Ok(Err(fetched));
    }
    // The fetch carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
    record_text(&["fetch", "--no-tags", destination.remote.as_str(), refspec.as_str()], fetched.output.as_str());
    let tip = git_output_awaiting(
        git,
        &argv(&[
            "rev-parse",
            "--verify",
            &format!("{private_ref}^{{commit}}"),
        ]),
    )
    .await;
    // The private ref is scaffolding; a failed deletion leaves one small ref that the next run overwrites.
    let _deleted = run_query_awaiting(git, &argv(&["update-ref", "-d", &private_ref])).await;
    return Ok(Ok(tip?));
}

/// What:
///  The journal identifier of the entries of a merged file:
/// the wrapper's for a push a person typed,
/// the service's for the
///  auto-push,
/// which runs inside the service.
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

//! What:
//!  Whether a push Git reported as failed published the recorded branch all the same,
//! because `push.default=matching` made one push carry every branch the remote has.
//! Why:
//!  Under `matching` the wrapper names no refspec and leaves the matching to Git,
//! so one push sends every local branch whose name the remote advertises,
//!  and Git's exit status is the worst fate among them:
//! a sibling the remote refused makes the status non-zero while the recorded branch was delivered.
//!  Reading that status as the recorded branch's failure told the person
//! "the remote may or may not have taken that commit" about a commit the remote had,
//!  and planned a retry ten seconds later,
//! then twenty,
//! then forty,
//! which could not clear while the sibling stayed refused,
//! so a published branch kept its push retried for as long as the plan lived.
//!  The remote is asked instead,
//! with `git ls-remote`,
//! and the answer is the ref's own object:
//! the branch arrived when that object is the commit this push sent.
//!  The remote-tracking ref cannot answer this:
//! it is a local copy that lags the remote,
//! so it still names a commit whose branch was deleted on the remote,
//! and a push to a remote that cannot be reached at all would read as delivered
//!  (`a_matching_push_with_no_branch_in_common_fails_with_gits_words`,
//! `a_matching_push_to_an_unreachable_remote_fails_loudly`).
//!  A remote that cannot be asked,
//!  or that names another object,
//! proves nothing and leaves the push the failure it was,
//! because a delivery this wrapper cannot prove is never
//!  claimed,
//! and the next push of something the remote already has succeeds and clears the record.
//!  The question is put to the remote's fetch URL,
//! the one the remote-tracking refs are read from and the one a person's `git status` speaks of;
//! a push URL that differs can therefore leave the answer unproven,
//! which errs toward the retry and never toward a claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const arrived = gitMatched && exitCode !== 0
//!   ? (await lsRemote(git, [remote, refOf(branchRef)])).get(refOf(branchRef)) === landedOid
//!   : false;
//! ```

/// The real Git of a push and the failure type of its coordination,
/// with the queries that ask it.
use crate::auto_push_single_flight::{PushGit, SingleFlightError, run_query, run_query_awaiting};
/// The two markers Git prints for a refused ref:
/// the stale `[rejected]` and the remote's own `[remote rejected]`.
use crate::auto_push_reconcile_basics_and_limits::{REJECTED_MARKER, REMOTE_REJECTED_MARKER};
/// Owned OS strings for the query's arguments.
use std::ffi::OsString;

/// The ref a matching push sends one branch to:
/// the remote's ref of the branch's own name.
///  Nothing for a ref outside `refs/heads`,
/// which no matching push sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = branchRef.startsWith('refs/heads/') ? branchRef.slice('refs/heads/'.length) : undefined;
/// return name === undefined ? undefined : `refs/heads/${name}`;
/// ```
pub(crate) fn remote_ref_of(branch_ref: &str) -> Option<String> {
    let name: &str = branch_ref.strip_prefix("refs/heads/")?;
    return Some(format!("refs/heads/{name}"));
}

/// What:
///  Whether a push's status leaves the recorded branch's fate to the remote.
/// Why:
///  Only a matching push carries branches the wrapper did not name,
/// so only its status can be another branch's failure;
///  a push that named its refspec,
/// and one Git reported as done,
/// say what happened to this branch themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitMatched && exitCode !== 0
/// ```
pub(crate) fn asks_the_remote(git_matched: bool, exit_code: i64) -> bool {
    return git_matched && exit_code != 0;
}

/// What:
///  The facts one matching push's status is judged by.
/// Why:
///   The judgement needs the push's remote and branch,
/// the status Git gave and the commit that was meant to arrive,
/// and naming them here keeps the two faces' calls identical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MatchingPush = { gitMatched; remote; branchRef; exitCode; landedOid };
/// ```
pub(crate) struct MatchingPush<'facts> {
    /// Whether Git matched the branches itself,
    /// which is what `push.default=matching` asks of it.
    pub(crate) git_matched: bool,
    /// The push remote.
    pub(crate) remote: &'facts str,
    /// The branch the commit landed on.
    pub(crate) branch_ref: &'facts str,
    /// The status Git gave the whole push.
    pub(crate) exit_code: i64,
    /// The commit that was pushed.
    pub(crate) landed_oid: &'facts str,
}

/// What:
///  The object `ls-remote` says one ref of a remote names,
/// from its own listing.
/// Why:
///  The listing is one record per ref,
/// `<object>` and the ref's full name separated by a tab,
/// and a ref outside the answer is not in it.
/// A record whose object or name is not whole is no answer,
///  so a listing this release does not understand proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const line of listing.split('\n')) { const [oid, ref] = line.split('\t'); if (ref === wanted) return oid; }
/// ```
pub(crate) fn object_named_by(listing: &str, wanted: &str) -> Option<String> {
    for line in listing.lines() {
        let (object, name): (&str, &str) = line.split_once('\t')?;
        if name == wanted && !object.is_empty() {
            return Some(object.to_owned());
        }
    }
    return None;
}

/// What:
///  The lines in which Git says it refused a ref,
/// or nothing.
/// Why:
///  A matching push that the remote refused for a sibling still delivers the recorded branch,
/// and that branch's push is then reported as done;
///  the refused sibling is published nowhere,
/// and the service's journal is the only place a person can learn it,
///  so the refusal is named there from Git's own line for that ref.
///  This is a line for a reader,
/// never a verdict:
/// every decision about what the remote holds is asked of the remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shown.split('\n').filter((line) => line.includes('[rejected]') || line.includes('[remote rejected]'))
/// ```
pub(crate) fn refused_lines(shown: &str) -> Vec<&str> {
    return shown.lines().filter(|line| line.contains(REJECTED_MARKER) || line.contains(REMOTE_REJECTED_MARKER)).collect();
}

/// What:
///  The arguments that ask a remote what one ref names.
/// Why:
///   `--refs` keeps symbolic refs out of the listing,
/// so one ref cannot be answered twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['ls-remote', '--refs', remote, ref]
/// ```
fn ask_arguments<'facts>(remote: &'facts str, remote_ref: &'facts str) -> [&'facts str; 4] {
    return ["ls-remote", "--refs", remote, remote_ref];
}

/// What:
///  Whether the remote proves the recorded branch arrived although Git's status says the push failed,
/// asked of the remote itself.
/// Why:
///  The remote is the only authority on what it holds,
/// and the question costs one query on a push that already failed.
///  A question that cannot be put proves nothing,
/// and the push keeps the failure it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!asksTheRemote(push.gitMatched, push.exitCode)) return false;
/// const ref = remoteRefOf(push.branchRef); if (ref === undefined) return false;
/// try { return objectNamedBy(await lsRemote(git, [push.remote, ref]), ref) === push.landedOid; } catch { return false; }
/// ```
pub(crate) fn delivered_although_git_failed(git: &PushGit, push: &MatchingPush<'_>) -> Result<bool, SingleFlightError> {
    if !asks_the_remote(push.git_matched, push.exit_code) {
        return Ok(false);
    }
    let Some(remote_ref): Option<String> = remote_ref_of(push.branch_ref) else {
        return Ok(false);
    };
    let arguments: [&str; 4] = ask_arguments(push.remote, remote_ref.as_str());
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    let listing = match run_query(git, &argv) {
        Ok(output) if output.code == Some(0) => output,
        // A remote that cannot be asked, or that answered with an error, holds nothing this wrapper can prove.
        Ok(_) | Err(_) => return Ok(false),
    };
    return Ok(object_named_by(String::from_utf8_lossy(listing.stdout.as_slice()).as_ref(), remote_ref.as_str()).as_deref() == Some(push.landed_oid));
}

/// What:
///  The same judgement,
/// asked by a caller that is itself a task of the runtime.
/// Why:
///  The service's push is a task,
/// and the synchronous query enters the runtime and is refused from there.
/// The question,
///  the ref and the reading of the answer are the synchronous face's,
/// so the two cannot disagree about a branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deliveredAlthoughGitFailedOnTheRuntime(git, push)
/// ```
pub(crate) async fn delivered_although_git_failed_awaiting(git: &PushGit, push: &MatchingPush<'_>) -> Result<bool, SingleFlightError> {
    if !asks_the_remote(push.git_matched, push.exit_code) {
        return Ok(false);
    }
    let Some(remote_ref): Option<String> = remote_ref_of(push.branch_ref) else {
        return Ok(false);
    };
    let arguments: [&str; 4] = ask_arguments(push.remote, remote_ref.as_str());
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    let listing = match run_query_awaiting(git, &argv).await {
        Ok(output) if output.code == Some(0) => output,
        // A remote that cannot be asked, or that answered with an error, holds nothing this wrapper can prove.
        Ok(_) | Err(_) => return Ok(false),
    };
    return Ok(object_named_by(String::from_utf8_lossy(listing.stdout.as_slice()).as_ref(), remote_ref.as_str()).as_deref() == Some(push.landed_oid));
}

/// The judgement's own controls stay out of the release executable.
#[cfg(test)]
#[path = "matching_push_delivered_the_recorded_branch_tests.rs"]
mod tests;

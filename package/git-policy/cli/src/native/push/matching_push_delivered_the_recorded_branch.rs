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
//!  The remote that was pushed to is asked instead,
//! with `git ls-remote`,
//! and the answer is the ref's own object:
//! the branch arrived when every push URL of the remote names the commit this push sent.
//!  Two other sources were tried and are wrong.
//! The push's own status is the thing that started this.
//! The remote-tracking ref is a local copy that lags the remote,
//! so it still names a commit whose branch was deleted on the remote,
//!  and a push to a remote that cannot be reached at all would read as delivered.
//! The remote's fetch URL is not the place the push went either:
//! `remote.<name>.pushurl` can name another repository,
//!  and one that holds the commit while the push URL refused it would read as delivered
//! (`a_matching_push_whose_push_url_lacks_the_branch_is_a_failure`).
//!  A push remote can also be a URL rather than a name the repository holds
//! (`branch.<name>.pushRemote` or `remote.pushDefault`),
//! and `git remote get-url` answers nothing for one,
//! so the setting is where that URL is read from
//! (`a_matching_push_to_a_push_remote_that_is_a_url_still_publishes_the_branch`).
//!  A push URL that cannot be asked,
//! or that names another object,
//! proves nothing and leaves the push the failure it was,
//!  because a delivery this wrapper cannot prove is never claimed,
//! and the next push of something the remote already has succeeds and clears the record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const arrived = gitMatched && exitCode !== 0
//!   ? (await pushUrls(git, remote)).every((url) => lsRemote(git, [url, refOf(branchRef)]).get(refOf(branchRef)) === landedOid)
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
    /// The commit the push was meant to deliver,
    /// which is the commit the branch named before the push read the worktree's verdicts.
    /// A matching push names no refspec,
    /// so that reading is the only commit the wrapper fixes for it,
    /// and a branch that moves while the push runs leaves a remote ref naming another commit,
    /// which proves nothing and plans the retry that publishes the branch.
    pub(crate) landed_oid: &'facts str,
}

/// What:
///  The URLs one `git remote get-url --push --all` answered with.
/// Why:
///   One URL is printed per line,
/// and a remote with no push URL answers with the URLs it fetches from,
/// which is where Git itself pushes in that case.
/// A line that is empty names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing.split('\n').map((line) => line.trim()).filter((line) => line !== '')
/// ```
pub(crate) fn urls_of(listing: &str) -> Vec<&str> {
    return listing.lines().map(str::trim).filter(is_a_url).collect();
}

/// Whether one line of that listing names a URL.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (line) => line !== ''
/// ```
fn is_a_url(line: &&str) -> bool {
    return !line.is_empty();
}

/// What:
///  The URLs a push went to,
/// from the answer to `git remote get-url --push --all`.
/// Why:
///   One URL is printed per line,
/// and a remote with no push URL answers with the URLs it fetches from,
/// which is where Git itself pushes in that case.
///  A remote this repository does not name is a URL the push went to itself:
/// Git accepts a URL where a remote name belongs,
/// and `git remote get-url` refuses to answer for one,
/// so the setting that named it is the only place that URL is written down.
/// Nothing written down is no URL at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing === undefined ? [remote] : listing.split('\n').map((line) => line.trim()).filter((line) => line !== '')
/// ```
pub(crate) fn urls_the_push_went_to(listing: Option<&str>, remote: &str) -> Vec<String> {
    let Some(answered): Option<&str> = listing else {
        return vec![remote.to_owned()];
    };
    return urls_of(answered).into_iter().map(str::to_owned).collect();
}

/// What:
///  Whether every push URL named the commit,
/// which is what proves the recorded branch arrived.
/// Why:
///   Git reports a push as done only when every push URL of the remote took it,
/// so one URL that names nothing else is not a delivery.
/// No URL at all proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answers.length > 0 && answers.every((answer) => answer === landedOid)
/// ```
pub(crate) fn every_url_named_it(answers: &[Option<String>], landed_oid: &str) -> bool {
    if answers.is_empty() {
        return false;
    }
    for answer in answers {
        if answer.as_deref() != Some(landed_oid) {
            return false;
        }
    }
    return true;
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
    return shown.lines().filter(is_a_refusal).collect();
}

/// Whether one line of a push's output is a refused ref.
///  Both markers Git prints for one are a refusal,
/// and a line that names no ref is none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (line) => line.includes('[rejected]') || line.includes('[remote rejected]')
/// ```
fn is_a_refusal(line: &&str) -> bool {
    return line.contains(REJECTED_MARKER) || line.contains(REMOTE_REJECTED_MARKER);
}

/// The arguments that name the URLs a remote pushes to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['remote', 'get-url', '--push', '--all', remote]
/// ```
fn push_url_arguments<'facts>(remote: &'facts str) -> [&'facts str; 5] {
    return ["remote", "get-url", "--push", "--all", remote];
}

/// The arguments that ask one remote what one ref names.
///  `--refs` keeps symbolic refs out of the listing,
/// so one ref cannot be answered twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['ls-remote', '--refs', url, ref]
/// ```
fn ask_arguments<'facts>(url: &'facts str, remote_ref: &'facts str) -> [&'facts str; 4] {
    return ["ls-remote", "--refs", url, remote_ref];
}

/// The argument vectors of one query,
/// as owned OS strings.
fn argv_of<'text>(arguments: &[&'text str]) -> Vec<OsString> {
    return arguments.iter().map(OsString::from).collect();
}

/// What:
///  Whether the remote proves the recorded branch arrived although Git's status says the push failed,
/// asked of every URL that remote pushes to.
/// Why:
///  The remote is the only authority on what it holds,
/// and the question costs a few queries on a push that already failed.
///  A question that cannot be put proves nothing,
/// and the push keeps the failure it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!asksTheRemote(push.gitMatched, push.exitCode)) return false;
/// const ref = remoteRefOf(push.branchRef); if (ref === undefined) return false;
/// const urls = pushUrls(git, push.remote); if (urls.length === 0) return false;
/// return everyUrlNamedIt(urls.map((url) => lsRemote(git, [url, ref])), push.landedOid);
/// ```
pub(crate) fn delivered_although_git_failed(git: &PushGit, push: &MatchingPush<'_>) -> Result<bool, SingleFlightError> {
    if !asks_the_remote(push.git_matched, push.exit_code) {
        return Ok(false);
    }
    let Some(remote_ref): Option<String> = remote_ref_of(push.branch_ref) else {
        return Ok(false);
    };
    let listing: Option<String> = match run_query(git, argv_of(&push_url_arguments(push.remote)).as_slice()) {
        Ok(output) if output.code == Some(0) => Some(String::from_utf8_lossy(output.stdout.as_slice()).into_owned()),
        // A remote this repository does not name is the URL the push went to.
        Ok(_) => None,
        Err(_) => return Ok(false),
    };
    let mut answers: Vec<Option<String>> = Vec::new();
    for url in urls_the_push_went_to(listing.as_deref(), push.remote) {
        let answer = match run_query(git, argv_of(&ask_arguments(url.as_str(), remote_ref.as_str())).as_slice()) {
            Ok(output) if output.code == Some(0) => output,
            // A push URL that cannot be asked holds nothing this wrapper can prove.
            Ok(_) | Err(_) => return Ok(false),
        };
        answers.push(object_named_by(String::from_utf8_lossy(answer.stdout.as_slice()).as_ref(), remote_ref.as_str()));
    }
    return Ok(every_url_named_it(answers.as_slice(), push.landed_oid));
}

/// What:
///  The same judgement,
/// asked by a caller that is itself a task of the runtime.
/// Why:
///  The service's push is a task,
/// and the synchronous query enters the runtime and is refused from there.
/// The questions,
///  the ref and the reading of the answers are the synchronous face's,
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
    let listing: Option<String> = match run_query_awaiting(git, argv_of(&push_url_arguments(push.remote)).as_slice()).await {
        Ok(output) if output.code == Some(0) => Some(String::from_utf8_lossy(output.stdout.as_slice()).into_owned()),
        // A remote this repository does not name is the URL the push went to.
        Ok(_) => None,
        Err(_) => return Ok(false),
    };
    let mut answers: Vec<Option<String>> = Vec::new();
    for url in urls_the_push_went_to(listing.as_deref(), push.remote) {
        let answer = match run_query_awaiting(git, argv_of(&ask_arguments(url.as_str(), remote_ref.as_str())).as_slice()).await {
            Ok(output) if output.code == Some(0) => output,
            // A push URL that cannot be asked holds nothing this wrapper can prove.
            Ok(_) | Err(_) => return Ok(false),
        };
        answers.push(object_named_by(String::from_utf8_lossy(answer.stdout.as_slice()).as_ref(), remote_ref.as_str()));
    }
    return Ok(every_url_named_it(answers.as_slice(), push.landed_oid));
}

/// The judgement's own controls stay out of the release executable.
#[cfg(test)]
#[path = "matching_push_delivered_the_recorded_branch_tests.rs"]
mod tests;

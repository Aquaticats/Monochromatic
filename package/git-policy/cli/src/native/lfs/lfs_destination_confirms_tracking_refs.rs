//! What:
//!  Whether this clone's tracking refs for a remote describe the server a push goes to,
//! decided from what that server
//!  itself reports about its branches.
//! Why:
//!  `refs/remotes/<remote>/*` are what a fetch wrote down about the URL the remote had when that fetch ran.
//!  `git remote set-url`,
//! a remote removed and added again under the same name,
//! a moved `url.<base>.insteadOf`
//!  rewrite and a server swapped underneath a name all leave those refs describing a server this push does
//!  not go to,
//! and an upload that stops where those refs reach then leaves pointer files on the destination with no
//!  content behind them while the push reports success.
//! So the plan asks the destination for its branch heads
//! (`git ls-remote --heads <remote>`,
//!  which resolves the URL this push resolves,
//! rewrites included)
//! and keeps a tracking ref as evidence only when the
//!  destination reports the same branch at the same commit.
//! A branch the destination moved on,
//! a branch it no longer has,
//!  a destination that cannot be asked,
//! a local listing this cannot read:
//! those refs are then no evidence at all,
//! and
//!  the whole pushed history is read instead,
//! which is slower and can leave out nothing the destination might lack.
//! A
//!  clone that holds no tracking refs for the remote has nothing to skip,
//! so it asks nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const trusted = trackingRefsDescribeDestination(remote, query); // false: send everything
//! ```

/// The debug line a decision is recorded on.
use crate::diagnostic_log::debug;
/// The read-only Git questions,
/// and the answers their callers read.
use crate::lfs_upload_settings::{Query, QueryAwaiting};
/// The destination's branches,
/// by ref name.
use std::collections::HashMap;

/// The tag of the decision lines.
const TAG: &str = "monochromatic-git-policy-cli lfs-destination";

/// What:
///  The `for-each-ref` format the tracking refs are read in:
/// the commit,
/// the ref a symbolic ref points at,
/// and
///  the ref name.
/// Why:
///   `refs/remotes/<remote>/HEAD` is a symbolic ref that holds no belief about the destination of its own,
///  and a tab separates the three without the spaces a ref name may hold.
/// The empty field of a ref that is not
///  symbolic sits between two filled ones,
/// because every caller of a Git query trims the answer's ends,
///  which would eat a trailing or leading tab.
/// A line in any other shape is refused rather than read as a ref with no
///  commit.
const TRACKING_FORMAT: &str = "--format=%(objectname)%09%(symref)%09%(refname)";

/// One of this clone's tracking refs:
/// the branch it came from and the commit it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackingRef = { branch: string; oid: string };
/// ```
struct TrackingRef {
    /// The branch on the other side,
    /// without `refs/heads/`.
    branch: String,
    /// The commit this clone believes that branch holds.
    oid: String,
}

/// Whether a listing field holds text,
/// which for `%(symref)` means the ref is symbolic.
fn is_filled(field: &str) -> bool {
    return !field.is_empty();
}

/// What:
///  The refs of a `for-each-ref` listing of one remote's tracking refs that hold a belief of their own,
///  or nothing when a line is not the shape the format asks for.
/// Why:
///   A symbolic ref and a name outside `refs/remotes/<remote>/` are no evidence about the destination,
/// and a
///  listing read as "no tracking refs" would trust everything:
/// an answer this cannot read is refused instead,
///  so the caller reads the whole pushed history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing.split('\n').flatMap((line) => { const fields = line.split('\t'); if (!line) return []; if (fields.length !== 3) throw new Error(line); return fields[1] || !fields[2].startsWith(prefix) ? [] : [{ oid: fields[0], branch: fields[2].slice(prefix.length) }]; });
/// ```
fn tracking_refs(listing: &str, remote: &str) -> Option<Vec<TrackingRef>> {
    let prefix: String = format!("refs/remotes/{remote}/");
    let mut tracked: Vec<TrackingRef> = Vec::new();
    for line in listing.lines() {
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let (Some(oid), Some(symbolic), Some(refname)) = (fields.next(), fields.next(), fields.next()) else {
            return None;
        };
        if fields.next().is_some() {
            return None;
        }
        if is_filled(symbolic) {
            continue;
        }
        let Some(branch) = refname.strip_prefix(prefix.as_str()) else {
            continue;
        };
        tracked.push(TrackingRef {
            branch: branch.to_owned(),
            oid: oid.to_owned(),
        });
    }
    return Some(tracked);
}

/// What:
///  The branch heads a destination reported,
/// by ref name,
/// or nothing when a line is not one.
/// Why:
///   A listing this cannot read is no evidence about the destination,
/// and the caller then reads the whole
///  pushed history rather than trusting a part of it;
/// one map answers every tracking ref in one lookup,
///  whatever the destination reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heads = new Map(listing.split('\n').filter(Boolean).map((line) => { const [oid, name] = line.split('\t'); if (!name) throw new Error(line); return [name, oid]; }));
/// ```
fn destination_heads(reported: &str) -> Option<HashMap<String, String>> {
    let mut heads: HashMap<String, String> = HashMap::new();
    for line in reported.lines() {
        if line.is_empty() {
            continue;
        }
        let (oid, refname) = line.split_once('\t')?;
        heads.insert(refname.to_owned(), oid.to_owned());
    }
    return Some(heads);
}

/// What:
///  Whether the destination reports every tracking ref at the commit this clone holds.
/// Why:
///   Equality is the one check that needs nothing from the clone but the ref itself:
/// a destination that
///  reports the same commit has that commit and everything behind it,
/// and one that reports another commit or no
///  branch at all leaves this ref unable to say what the destination holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tracked.every(({ branch, oid }) => heads.get(`refs/heads/${branch}`) === oid);
/// ```
fn every_tracking_ref_is_reported(tracked: &[TrackingRef], heads: &HashMap<String, String>) -> bool {
    for tracked_ref in tracked {
        let wanted: String = format!("refs/heads/{}", tracked_ref.branch);
        if heads.get(wanted.as_str()) != Some(&tracked_ref.oid) {
            return false;
        }
    }
    return true;
}

/// What:
///  The decision one destination's answer supports,
/// and the line it is recorded on.
/// Why:
///   Both faces of the plan decide here,
/// so a synchronous push and the service's landing round cannot
///  differ in what they trust.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function evidenceHolds(tracked, heads, remote): boolean;
/// ```
fn evidence_holds(tracked: &[TrackingRef], heads: Option<HashMap<String, String>>, remote: &str) -> bool {
    if tracked.is_empty() {
        return true;
    }
    let Some(reported) = heads else {
        debug(
            TAG,
            &format!("the destination of {remote} did not report its branches, so the whole pushed history is read"),
        );
        return false;
    };
    if every_tracking_ref_is_reported(tracked, &reported) {
        debug(
            TAG,
            &format!(
                "the destination of {remote} reports every one of this clone's {} tracking refs, so the upload stops where they reach",
                tracked.len()
            ),
        );
        return true;
    }
    debug(
        TAG,
        &format!(
            "the destination of {remote} does not report this clone's {} tracking refs at the commits held here, so the whole pushed history is read",
            tracked.len()
        ),
    );
    return false;
}

/// What:
///  The tracking refs of `remote` from one `for-each-ref`,
/// or nothing when Git cannot list them or the listing is not
///  the shape asked for.
/// Why:
///   A listing that cannot be read is no evidence,
/// and the caller then sends everything;
/// a named function stands
///  where a closure would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// query(['for-each-ref', format, `refs/remotes/${remote}`]) -> rows
/// ```
fn read_tracking_refs(remote: &str, query: Query<'_>) -> Option<Vec<TrackingRef>> {
    let prefix: String = format!("refs/remotes/{remote}");
    return match query.ask(&["for-each-ref", TRACKING_FORMAT, prefix.as_str()]) {
        Ok(listing) => match tracking_refs(listing.as_str(), remote) {
            Some(tracked) => Some(tracked),
            None => {
                debug(
                    TAG,
                    &format!("git for-each-ref {prefix} answered in a shape this cannot read, so the whole pushed history is read"),
                );
                None
            }
        },
        Err(why) => {
            debug(
                TAG,
                &format!("git for-each-ref {prefix} did not answer ({why}), so the whole pushed history is read"),
            );
            None
        }
    };
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime.
async fn read_tracking_refs_awaiting(remote: &str, query: QueryAwaiting<'_>) -> Option<Vec<TrackingRef>> {
    let prefix: String = format!("refs/remotes/{remote}");
    return match query.ask(&["for-each-ref", TRACKING_FORMAT, prefix.as_str()]).await {
        Ok(listing) => match tracking_refs(listing.as_str(), remote) {
            Some(tracked) => Some(tracked),
            None => {
                debug(
                    TAG,
                    &format!("git for-each-ref {prefix} answered in a shape this cannot read, so the whole pushed history is read"),
                );
                None
            }
        },
        Err(why) => {
            debug(
                TAG,
                &format!("git for-each-ref {prefix} did not answer ({why}), so the whole pushed history is read"),
            );
            None
        }
    };
}

/// What:
///  The branch heads the destination itself reports,
/// or nothing when it cannot be asked.
/// Why:
///   `--heads` is the whole question:
/// a branch is where a fetch of `refs/heads/*` puts a tracking ref,
/// and nothing else is
///  read as a branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// query(['ls-remote', '--heads', remote]) -> heads
/// ```
fn read_destination_heads(remote: &str, query: Query<'_>) -> Option<HashMap<String, String>> {
    return match query.ask(&["ls-remote", "--heads", remote]) {
        Ok(reported) => destination_heads(reported.as_str()),
        Err(why) => {
            debug(
                TAG,
                &format!("git ls-remote --heads {remote} did not answer ({why}), so the whole pushed history is read"),
            );
            None
        }
    };
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime.
async fn read_destination_heads_awaiting(remote: &str, query: QueryAwaiting<'_>) -> Option<HashMap<String, String>> {
    return match query.ask(&["ls-remote", "--heads", remote]).await {
        Ok(reported) => destination_heads(reported.as_str()),
        Err(why) => {
            debug(
                TAG,
                &format!("git ls-remote --heads {remote} did not answer ({why}), so the whole pushed history is read"),
            );
            None
        }
    };
}

/// What:
///  Whether this clone's tracking refs for `remote` are evidence about the server this push goes to.
/// Why:
///   False means the destination does not confirm them,
/// so the upload must read the whole pushed history and
///  send what is not already there;
/// asking costs one local listing and one round trip to the destination,
/// and none
///  at all when the clone holds no tracking refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trackingRefsDescribeDestination(remote: string, query: GitQuery): boolean;
/// ```
pub(crate) fn tracking_refs_describe_destination(remote: &str, query: Query<'_>) -> bool {
    let Some(tracked) = read_tracking_refs(remote, query) else {
        return false;
    };
    if tracked.is_empty() {
        return true;
    }
    let heads: Option<HashMap<String, String>> = read_destination_heads(remote, query);
    return evidence_holds(tracked.as_slice(), heads, remote);
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime,
/// with the answers and the decision of the
///  synchronous face.
pub(crate) async fn tracking_refs_describe_destination_awaiting(remote: &str, query: QueryAwaiting<'_>) -> bool {
    let Some(tracked) = read_tracking_refs_awaiting(remote, query).await else {
        return false;
    };
    if tracked.is_empty() {
        return true;
    }
    let heads: Option<HashMap<String, String>> = read_destination_heads_awaiting(remote, query).await;
    return evidence_holds(tracked.as_slice(), heads, remote);
}

/// The question,
/// its readings and the decision are controlled in the test module.
#[cfg(test)]
#[path = "lfs_destination_confirms_tracking_refs_tests.rs"]
mod tests;

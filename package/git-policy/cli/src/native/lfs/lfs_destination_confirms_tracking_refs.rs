//! What:
//!  Whether this clone's tracking refs for a remote describe the server a push goes to,
//! decided from what that server
//!  itself reports about its branches,
//! ref by ref.
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
//! and keeps each tracking ref as evidence only when the
//!  destination reports the same branch at the same commit.
//! A branch the destination moved on or no longer has is then no evidence,
//!  and only that ref is dropped:
//! a clone routinely holds one tracking ref a teammate moved on or a merged branch left behind,
//!  and one such ref must not make every push read the whole history.
//! A destination that cannot be asked,
//! a local listing this cannot read,
//!  and a clone whose tracking refs the destination confirms none of
//! leave nothing to stop at,
//! and the whole
//!  pushed history is read instead,
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
use crate::lfs_upload_settings::{
    Query,
    QueryAwaiting,
};
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
fn tracking_refs(
    listing: &str,
    remote: &str,
) -> Option<Vec<TrackingRef>> {
    let prefix: String = format!("refs/remotes/{remote}/");
    let mut tracked: Vec<TrackingRef> = Vec::new();
    for line in listing.lines() {
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let (Some(oid), Some(symbolic), Some(refname)) =
            (fields.next(), fields.next(), fields.next())
        else {
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
///  The branch heads one destination of this push reported,
///  and the remote they were read for.
/// Why:
///   The manual-push discovery of a typed push already confirms the push's own updates with one
///  `git ls-remote --refs` against the destination,
///  in the one connection that read makes;
/// asking that
///  question again from the upload plan would add a connection to every typed push.
///  The remote name is carried so a plan for another remote cannot read these answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestinationHeads = { remote: string; heads: Map<string, string> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DestinationHeads {
    /// The remote name the discovery resolved for this push.
    remote: String,
    /// The branch heads by ref name,
    /// as the destination reported them.
    heads: HashMap<String, String>,
}

impl DestinationHeads {
    /// What:
    ///  The heads one `ls-remote` listing reports,
    ///  read for `remote`,
    ///  or nothing when the listing is not a listing.
    /// Why:
    ///   An answer that cannot be read is no evidence,
    /// so the plan asks its own question instead of trusting it;
    /// a line without an object name and a ref name is refused for the same reason a
    ///  tracking ref listing is.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// DestinationHeads.fromListing(remote, listing); // { remote, heads } | undefined
    /// ```
    pub fn from_listing(
        remote: &str,
        listing: &str,
    ) -> Option<DestinationHeads> {
        return Some(DestinationHeads {
            remote: remote.to_owned(),
            heads: destination_heads(listing)?,
        });
    }

    /// The heads this answer holds for `remote`,
    /// or nothing when it was read for another remote.
    pub(crate) fn heads_for(
        &self,
        remote: &str,
    ) -> Option<HashMap<String, String>> {
        if self.remote != remote {
            return None;
        }
        return Some(self.heads.clone());
    }
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
///  What the destination's answer confirmed about this clone's tracking refs,
///  and whether it confirmed every one.
/// Why:
///   A confirmed ref is the only place an upload may stop:
///  the destination reported that very commit,
/// so it holds that commit and everything behind it,
///  while an unconfirmed ref says nothing about the server this push goes to and is dropped
///  instead of dropping every ref with it.
///  Whether every ref is confirmed decides the form `git-lfs` scans in,
/// because its own skip list matches a
///  cached ref by branch name alone (`calcSkippedRefs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfirmedTrackingRefs = { confirmed: string[]; every: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmedTrackingRefs {
    /// The full names of the tracking refs the destination reports at the commit held here.
    confirmed: Vec<String>,
    /// Whether the destination confirmed every tracking ref this clone holds for the remote.
    every: bool,
}

impl ConfirmedTrackingRefs {
    /// What:
    ///  Nothing is confirmed,
    ///  which is what a caller holds before it asks,
    ///  and what it keeps when it cannot ask.
    /// Why:
    ///   A destination that was not asked leaves no place to stop at,
    /// so the upload reads the whole pushed
    ///  history rather than trusting a ref the server was never asked about;
    /// a caller that holds the destination's answer of
    ///  this push builds the value from it instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ConfirmedTrackingRefs.unknown(); // { confirmed: [], every: false }
    /// ```
    pub fn unknown() -> ConfirmedTrackingRefs {
        return unconfirmed(false);
    }

    /// The refs an upload may stop at,
    /// in the order the local listing held them.
    pub(crate) fn names(&self) -> &[String] {
        return self.confirmed.as_slice();
    }

    /// Whether the destination confirmed every tracking ref,
    /// so `git-lfs` may scan without `--all`.
    pub(crate) fn every_tracking_ref_is_confirmed(&self) -> bool {
        return self.every;
    }
}

/// What:
///  No evidence at all:
///  the destination could not be asked,
///  or its answer could not be read,
///  or the clone holds no tracking ref
///  that could be confirmed.
/// Why:
///   A destination nobody could ask leaves nothing to stop at,
/// so the upload reads the whole pushed
///  history;
/// a clone with no tracking refs has nothing to skip either,
/// and `git-lfs` scans the refs itself,
///  so its bounded form is the right one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unconfirmed = (every: boolean): ConfirmedTrackingRefs => ({ confirmed: [], every });
/// ```
fn unconfirmed(every: bool) -> ConfirmedTrackingRefs {
    return ConfirmedTrackingRefs {
        confirmed: Vec::new(),
        every,
    };
}

/// What:
///  The tracking refs the destination reports at the commit this clone holds.
/// Why:
///   Equality is the one check that needs nothing from the clone but the ref itself:
/// a destination that
///  reports the same commit has that commit and everything behind it,
/// and one that reports another commit or no
///  branch at all leaves this ref unable to say what the destination holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const confirmed = tracked.filter(({ branch, oid }) => heads.get(`refs/heads/${branch}`) === oid);
/// ```
fn confirmed_tracking_refs(
    tracked: &[TrackingRef],
    heads: &HashMap<String, String>,
    remote: &str,
) -> Vec<String> {
    let mut confirmed: Vec<String> = Vec::new();
    for tracked_ref in tracked {
        let wanted: String = format!("refs/heads/{}", tracked_ref.branch);
        if heads.get(wanted.as_str()) != Some(&tracked_ref.oid) {
            continue;
        }
        confirmed.push(format!("refs/remotes/{remote}/{}", tracked_ref.branch));
    }
    return confirmed;
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
/// function evidenceHolds(tracked, heads, remote): ConfirmedTrackingRefs;
/// ```
fn evidence_holds(
    tracked: &[TrackingRef],
    heads: Option<HashMap<String, String>>,
    remote: &str,
) -> ConfirmedTrackingRefs {
    if tracked.is_empty() {
        return unconfirmed(true);
    }
    let Some(reported) = heads else {
        debug(
            TAG,
            &format!(
                "the destination of {remote} did not report its branches, so the whole pushed history is read"
            ),
        );
        return unconfirmed(false);
    };
    let confirmed: Vec<String> = confirmed_tracking_refs(tracked, &reported, remote);
    if confirmed.len() == tracked.len() {
        debug(
            TAG,
            &format!(
                "the destination of {remote} reports every one of this clone's {} tracking refs, so the upload stops where they reach and git-lfs scans without --all",
                tracked.len()
            ),
        );
        return ConfirmedTrackingRefs {
            confirmed,
            every: true,
        };
    }
    debug(
        TAG,
        &format!(
            "the destination of {remote} reports {} of this clone's {} tracking refs at the commits held here, so the upload stops only where those reach and the whole pushed history is scanned otherwise",
            confirmed.len(),
            tracked.len()
        ),
    );
    return ConfirmedTrackingRefs {
        confirmed,
        every: false,
    };
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
fn read_tracking_refs(
    remote: &str,
    query: Query<'_>,
) -> Option<Vec<TrackingRef>> {
    let prefix: String = format!("refs/remotes/{remote}");
    return match query.ask(&["for-each-ref", TRACKING_FORMAT, prefix.as_str()]) {
        Ok(listing) => match tracking_refs(listing.as_str(), remote) {
            Some(tracked) => Some(tracked),
            None => {
                debug(
                    TAG,
                    &format!(
                        "git for-each-ref {prefix} answered in a shape this cannot read, so the whole pushed history is read"
                    ),
                );
                None
            }
        },
        Err(why) => {
            debug(
                TAG,
                &format!(
                    "git for-each-ref {prefix} did not answer ({why}), so the whole pushed history is read"
                ),
            );
            None
        }
    };
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime.
async fn read_tracking_refs_awaiting(
    remote: &str,
    query: QueryAwaiting<'_>,
) -> Option<Vec<TrackingRef>> {
    let prefix: String = format!("refs/remotes/{remote}");
    return match query
        .ask(&["for-each-ref", TRACKING_FORMAT, prefix.as_str()])
        .await
    {
        Ok(listing) => match tracking_refs(listing.as_str(), remote) {
            Some(tracked) => Some(tracked),
            None => {
                debug(
                    TAG,
                    &format!(
                        "git for-each-ref {prefix} answered in a shape this cannot read, so the whole pushed history is read"
                    ),
                );
                None
            }
        },
        Err(why) => {
            debug(
                TAG,
                &format!(
                    "git for-each-ref {prefix} did not answer ({why}), so the whole pushed history is read"
                ),
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
fn read_destination_heads(
    remote: &str,
    query: Query<'_>,
) -> Option<HashMap<String, String>> {
    return match query.ask(&["ls-remote", "--heads", remote]) {
        Ok(reported) => destination_heads(reported.as_str()),
        Err(why) => {
            debug(
                TAG,
                &format!(
                    "git ls-remote --heads {remote} did not answer ({why}), so the whole pushed history is read"
                ),
            );
            None
        }
    };
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime.
async fn read_destination_heads_awaiting(
    remote: &str,
    query: QueryAwaiting<'_>,
) -> Option<HashMap<String, String>> {
    return match query.ask(&["ls-remote", "--heads", remote]).await {
        Ok(reported) => destination_heads(reported.as_str()),
        Err(why) => {
            debug(
                TAG,
                &format!(
                    "git ls-remote --heads {remote} did not answer ({why}), so the whole pushed history is read"
                ),
            );
            None
        }
    };
}

/// What:
///  Which of this clone's tracking refs for `remote` the destination itself confirms,
///  and whether it confirms every one.
/// Why:
///   The confirmed refs are the only places the upload may stop,
/// so an upload keeps every ref the destination
///  reports and drops only the others;
/// an empty answer leaves nothing to stop at and the whole pushed history is
///  read;
/// asking costs one local listing and one connection to the destination,
/// and none at all when the
///  clone holds no tracking refs or the caller already holds the destination's branches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trackingRefsConfirmedBy(remote: string, known: DestinationHeads | undefined, query: GitQuery): ConfirmedTrackingRefs;
/// ```
pub fn tracking_refs_confirmed_by(
    remote: &str,
    known: Option<&DestinationHeads>,
    query: Query<'_>,
) -> ConfirmedTrackingRefs {
    let Some(tracked) = read_tracking_refs(remote, query) else {
        return unconfirmed(false);
    };
    if tracked.is_empty() {
        return unconfirmed(true);
    }
    let heads: Option<HashMap<String, String>> = match known.and_then(|carried| {
        return carried.heads_for(remote);
    }) {
        Some(answered) => Some(answered),
        None => read_destination_heads(remote, query),
    };
    return evidence_holds(tracked.as_slice(), heads, remote);
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime,
/// with the answers and the decision of the
///  synchronous face.
pub async fn tracking_refs_confirmed_by_awaiting(
    remote: &str,
    known: Option<&DestinationHeads>,
    query: QueryAwaiting<'_>,
) -> ConfirmedTrackingRefs {
    let Some(tracked) = read_tracking_refs_awaiting(remote, query).await else {
        return unconfirmed(false);
    };
    if tracked.is_empty() {
        return unconfirmed(true);
    }
    let heads: Option<HashMap<String, String>> = match known.and_then(|carried| {
        return carried.heads_for(remote);
    }) {
        Some(answered) => Some(answered),
        None => read_destination_heads_awaiting(remote, query).await,
    };
    return evidence_holds(tracked.as_slice(), heads, remote);
}

/// The question,
/// its readings and the decision are controlled in the test module.
#[cfg(test)]
#[path = "lfs_destination_confirms_tracking_refs_tests.rs"]
mod tests;

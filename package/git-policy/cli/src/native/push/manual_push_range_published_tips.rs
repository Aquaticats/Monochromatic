//! What:
//!  What the remote has already published,
//!  read from the commits a prior destination value names and from the tracking refs Git keeps.
//! Why:
//!  Only a locally present commit can bound a range,
//!  an object Git never fetched is unknown rather than published,
//!  and the tracking refs are what a fetch wrote for every branch the remote had.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tips = publishedTips(updates, peeled, tracking); // a map from remote to the commits already there
//! ```

use crate::auto_push_single_flight::PushGit;
use crate::manual_push_cause::ManualPushCause;
use crate::manual_push_probe::PushUpdate;
use crate::manual_push_range::{ManualPushRangeError, lines_of, range_error, run_git_bytes};
use std::ffi::OsString;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Remote-tracking ref namespace Git's default fetch refspec writes.
const TRACKING_NAMESPACE: &str = "refs/remotes/";

/// What:
///  The commit a prior destination value names,
/// when it exists locally and is a commit.
///  `Option<T>` is "a value or nothing".
/// Why:
///   Only locally present commits can bound the range;
/// an object Git never fetched is unknown,
/// not published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// peeled.get(oid)?.type === 'commit' ? [oid] : []
/// ```
pub fn peel_commits(
    git: &PushGit,
    oids: &[String],
) -> Result<BTreeMap<String, String>, ManualPushRangeError> {
    let mut unique: Vec<String> = oids.to_vec();
    unique.sort();
    unique.dedup();
    if unique.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut input: String = String::new();
    for oid in &unique {
        input.push_str(format!("{oid}^{{}}\n").as_str());
    }
    let arguments: Vec<OsString> = ["cat-file", "--batch-check=%(objectname) %(objecttype)"]
        .iter()
        .map(OsString::from)
        .collect();
    let output = run_git_bytes(git, &arguments, Some(&input))?;
    let records = lines_of(&output);
    if records.len() != unique.len() {
        return Err(range_error(
            ManualPushCause::UnexpectedOutput,
            "Git object batch-check returned an unexpected record count.".to_owned(),
        ));
    }
    let mut commits: BTreeMap<String, String> = BTreeMap::new();
    for (oid, record) in unique.iter().zip(records.iter()) {
        if record == &format!("{oid}^{{}} missing") {
            continue;
        }
        let fields: Vec<&str> = record.split(' ').collect();
        if fields.len() != 2 {
            return Err(range_error(ManualPushCause::UnexpectedOutput, format!(
                "Git object batch-check returned malformed record: {record}"
            )));
        }
        if fields[1] == "commit" {
            commits.insert(oid.clone(), fields[0].to_owned());
        }
    }
    return Ok(commits);
}

/// What:
///  The object name the listing gives each ref,
///  from that ref's first record.
/// Why:
///   One `ls-remote --refs` prints one record per ref asked for,
/// and a mirror or `--all` push publishes hundreds of them;
///  a map answers each update's question by naming the one record it compares with,
///  so the confirmation is one pass over the listing (`AGENTS.md` `RG2`),
///  and the first record a ref gets is the one the scan took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reported = new Map(listing.split('\n').flatMap((line) => { const [oid, name] = line.split('\t'); return name ? [[name, oid]] : [] }));
/// ```
pub(crate) fn oids_by_ref(listing: &str) -> HashMap<&str, &str> {
    let mut by_ref: HashMap<&str, &str> = HashMap::new();
    for line in listing.lines() {
        let Some((oid, name)) = line.split_once('\t') else { continue };
        crate::collected_elements_looked_at::note_looked_at(1);
        by_ref.entry(name).or_insert(oid);
    }
    return by_ref;
}

/// What:
///  Confirm one destination's values with `git ls-remote --refs` and turn each reported update into a public update.
/// Why:
///   A destination that changed after the dry run reported would publish something the scan did not see,
/// so a
///  mismatch between the report and the confirmation is an error,
/// never a silent repair.
/// A report that did not state the
///  remote's value (a deletion,
/// a refusal) is taken from the confirmation alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveRemoteGroup({ gitPath, cwd, remoteLocation, updates }): Promise<PushUpdate[]>;
/// ```
/// What:
///  The commits each destination remote is known to have,
/// from its remote-tracking refs.
/// Why:
///   The namespace is matched literally,
/// so a remote name is never read as a pattern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function trackingCommits({ gitPath, cwd, remoteNames }): Promise<ReadonlyMap<string, string[]>>;
/// ```
pub fn tracking_commits(
    git: &PushGit,
    remote_names: &[String],
) -> Result<BTreeMap<String, Vec<String>>, ManualPushRangeError> {
    if remote_names.is_empty() {
        return Ok(BTreeMap::new());
    }
    let arguments: Vec<OsString> = [
        "for-each-ref",
        "--format=%(objectname) %(objecttype) %(refname)",
        TRACKING_NAMESPACE,
    ]
    .iter()
    .map(OsString::from)
    .collect();
    let refs = lines_of(&run_git_bytes(git, &arguments, None)?);
    let mut by_remote: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for remote_name in remote_names {
        let prefix = format!("{TRACKING_NAMESPACE}{remote_name}/");
        let mut commits: Vec<String> = Vec::new();
        for record in &refs {
            let fields: Vec<&str> = record.split(' ').collect();
            if fields.len() != 3 {
                return Err(range_error(ManualPushCause::UnexpectedOutput, format!(
                    "Git for-each-ref returned malformed record: {record}"
                )));
            }
            if fields[1] == "commit" && fields[2].starts_with(&prefix) {
                commits.push(fields[0].to_owned());
            }
        }
        by_remote.insert(remote_name.clone(), commits);
    }
    return Ok(by_remote);
}

/// What:
///  The published tips of each remote:
/// its tracking commits and the commits of its prior destination values.
/// Why:
///   Every update to one remote contributes its prior value,
/// so a second branch is bounded by the first's prior state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function publishedTips({ updates, peeled, tracking }): ReadonlyMap<string, readonly GitObjectId[]>;
/// ```
pub fn published_tips(
    updates: &[PushUpdate],
    prior_commits: &BTreeMap<String, String>,
    tracking: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, Vec<String>> {
    let mut tips: BTreeMap<String, Vec<String>> = BTreeMap::new();
    // What each remote's list already holds, so a question is answered by naming one collected name instead of walking
    //  the whole accumulated list for every update (`AGENTS.md` RG2).
    let mut collected: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    for update in updates {
        let remote: &String = &update.remote_name;
        let entry = match tips.entry(remote.clone()) {
            std::collections::btree_map::Entry::Occupied(present) => present.into_mut(),
            std::collections::btree_map::Entry::Vacant(absent) => absent.insert(tracking.get(remote).cloned().unwrap_or_default()),
        };
        let seen: &mut HashSet<String> = match collected.entry(remote.clone()) {
            std::collections::btree_map::Entry::Occupied(present) => present.into_mut(),
            std::collections::btree_map::Entry::Vacant(absent) => absent.insert(entry.iter().cloned().collect()),
        };
        if let Some(prior) = update.remote_oid.as_ref()
            && let Some(commit) = prior_commits.get(prior)
        {
            // One collected name looked at per question; the set answers it by naming that one.
            crate::collected_elements_looked_at::note_looked_at(1);
            if seen.insert(commit.clone()) {
                entry.push(commit.clone());
            }
        }
    }
    return tips;
}

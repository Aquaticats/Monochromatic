//! What:
//!  The manual-push candidate scope of one `git push`: the commits and trees the destination lacks.
//! Why:
//!  The manual-push policies (`final-newline`, `forbidden-strings`) scan the content a push publishes.
//!  The incumbent (`src/policy-engine/manual-push-lifecycle.ts`) discovers the updates, bounds each by what its
//!  destination already has, and hands the scan to the policy engine; a failure stops the push with exit `2`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const updates = await probeManualPushUpdates({ gitPath, cwd, args });
//! ```

/// The probe, the published set and the range, and the candidate request they feed.
use super::auto_push_single_flight::PushGit;
use super::candidate_object::parse_object_id;
use super::candidate_prediction::PushedScope;
use super::manual_push_probe::{PushUpdate, probe_manual_push_updates};
use super::manual_push_range::{
    ManualPushRangeError, UpdateScope, peel_commits, published_tips, tracking_commits, update_scope,
};
/// Maps keyed by remote name and by object name.
use std::collections::BTreeMap;
/// Owned OS strings for the replayed global options and the post-subcommand arguments.
use std::ffi::OsString;

/// What:
///  Why a push's scope could not be established.
/// Why:
///   The lifecycle reports every such failure as a content-unavailable engine failure and stops the push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushFailure {
    /// Diagnostic naming the failed step.
    pub message: String,
}

impl std::fmt::Display for ManualPushFailure {
    /// Writes the diagnostic text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

impl std::error::Error for ManualPushFailure {}

/// Converts a probe failure into the lifecycle failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caughtValueText(error)
/// ```
fn from_probe(error: &super::manual_push_probe::ManualPushProbeError) -> ManualPushFailure {
    return ManualPushFailure {
        message: error.message.clone(),
    };
}

/// Converts a range failure into the lifecycle failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caughtValueText(error)
/// ```
fn from_range(error: &ManualPushRangeError) -> ManualPushFailure {
    return ManualPushFailure {
        message: error.message.clone(),
    };
}

/// What:
///  The scope of every update of one push, in the order Git negotiated the updates.
/// Why:
///   Each update is bounded by the published tips of its own destination remote only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function manualPushScopes({ gitPath, cwd, args }): Promise<UpdateScope[]>;
/// ```
pub fn scopes_of_updates(
    git: &PushGit,
    updates: &[PushUpdate],
) -> Result<Vec<UpdateScope>, ManualPushFailure> {
    let remote_names: Vec<String> = updates.iter().fold(Vec::new(), |mut names, update| {
        if !names.contains(&update.remote_name) {
            names.push(update.remote_name.clone());
        }
        return names;
    });
    let tracking: BTreeMap<String, Vec<String>> =
        tracking_commits(git, &remote_names).map_err(|error| {
            return from_range(&error);
        })?;
    let prior_oids: Vec<String> = updates
        .iter()
        .filter_map(|update| {
            return update.remote_oid.clone();
        })
        .collect();
    let prior: BTreeMap<String, String> = peel_commits(git, &prior_oids).map_err(|error| {
        return from_range(&error);
    })?;
    let tips: BTreeMap<String, Vec<String>> = published_tips(updates, &prior, &tracking);
    let mut scopes: Vec<UpdateScope> = Vec::new();
    for update in updates {
        let published: Vec<String> = tips.get(&update.remote_name).cloned().unwrap_or_default();
        scopes.push(update_scope(git, update, &published).map_err(|error| {
            return from_range(&error);
        })?);
    }
    return Ok(scopes);
}

/// What:
///  Collect the commits and trees the candidate layer must list for a set of scopes.
/// Why:
///   Each commit is listed once even when several updates publish it, so the scan never repeats a delta.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateTargets(scopes: UpdateScope[]): { commits: string[]; trees: string[] };
/// ```
pub fn pushed_scope(scopes: &[UpdateScope]) -> Result<PushedScope, ManualPushFailure> {
    let mut commits: Vec<super::candidate_object::ObjectId> = Vec::new();
    let mut trees: Vec<super::candidate_object::ObjectId> = Vec::new();
    for scope in scopes {
        match scope {
            UpdateScope::None => {}
            UpdateScope::Commits(names) => {
                for name in names {
                    let oid = parse_object_id(name.as_bytes()).ok_or_else(|| {
                        return ManualPushFailure {
                            message: format!(
                                "Git named a commit that is not an object name: {name}"
                            ),
                        };
                    })?;
                    if !commits.contains(&oid) {
                        commits.push(oid);
                    }
                }
            }
            UpdateScope::FinalTree(tip) => {
                let oid = parse_object_id(tip.as_bytes()).ok_or_else(|| {
                    return ManualPushFailure {
                        message: format!("Git named a commit that is not an object name: {tip}"),
                    };
                })?;
                if !trees.contains(&oid) {
                    trees.push(oid);
                }
            }
        }
    }
    return Ok(PushedScope { commits, trees });
}

/// What:
///  Discover the updates of one push and the candidate scope they publish.
/// Why:
///   The whole chain runs before any policy reads content, so a failure in any step refuses the push
///  with its diagnostic and no partial scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const updates = await probeManualPushUpdates({ gitPath, cwd, args });
/// const scopes = await manualPushScopes(updates);
/// ```
pub fn manual_push_scope(
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<PushedScope, ManualPushFailure> {
    let updates =
        probe_manual_push_updates(git, global_prefix, post_subcommand).map_err(|error| {
            return from_probe(&error);
        })?;
    let scopes = scopes_of_updates(git, &updates)?;
    return pushed_scope(&scopes);
}

/// The lifecycle's scope is controlled in the test module.
#[cfg(test)]
#[path = "manual_push_lifecycle_tests.rs"]
mod tests;

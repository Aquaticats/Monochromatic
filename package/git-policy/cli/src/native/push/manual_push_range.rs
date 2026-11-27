//! What:
//!  The commits a push newly publishes,
//! relative to what each destination remote already has,
//!  read with the batch query that names what a push would update.
//! Why:
//!  The manual-push policies scan only what becomes public.
//! A pushed commit is already published when a
//!  remote-tracking ref of the destination,
//! or an authoritative prior value of an update to that remote,
//!  reaches it.
//! The rest of the history is the scan's range.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const published = publishedTips({ updates, peeled, tracking });
//! ```

/// The Git runner,
/// and the update,
/// its failure and the batch query a scope is read with.
use crate::auto_push_single_flight::PushGit;
use crate::manual_push_cause::ManualPushCause;
use crate::manual_push_range_published_tips::peel_commits;
use crate::manual_push_update_and_query_limits::{
    ManualPushRangeError,
    PushUpdate,
    lines_of,
    range_error,
    run_git_bytes,
};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// What:
///  The scope one pushed update contributes to the scan.
/// Why:
///   A deletion publishes nothing;
/// a commit with a known published base contributes its new commits;
///  a commit with no known base contributes its whole history's tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UpdateScope = { kind: 'none' } | { kind: 'commits'; commits: string[] } | { kind: 'final-tree'; tip: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateScope {
    /// Nothing new is published:
    /// a deletion or a commit the remote already has.
    None,
    /// The commits not reachable from anything the remote has.
    Commits(Vec<String>),
    /// No published base is known:
    /// the pushed tip's whole tree is new.
    FinalTree(String),
}

/// What:
///  The scope of one update,
/// from its local object and the published tips of its remote.
/// Why:
///   `rev-list` names exactly the commits the remote lacks,
/// so a scan never repeats published history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scopeOf(update, publishedTips): Promise<UpdateScope>;
/// ```
pub fn update_scope(
    git: &PushGit,
    update: &PushUpdate,
    published: &[String],
) -> Result<UpdateScope, ManualPushRangeError> {
    let Some(local) = update.local_oid.as_ref() else {
        return Ok(UpdateScope::None);
    };
    let commit_check = peel_commits(git, std::slice::from_ref(local))?;
    if !commit_check.contains_key(local) {
        return Err(range_error(
            ManualPushCause::UnsupportedObject,
            format!(
                "pushed object {local} is not a commit, and the manual-push policies read commits only"
            ),
        ));
    }
    if published.is_empty() {
        return Ok(UpdateScope::FinalTree(local.clone()));
    }
    let mut arguments: Vec<OsString> = vec![
        OsString::from("rev-list"),
        OsString::from(local),
        OsString::from("--not"),
    ];
    arguments.extend(published.iter().map(OsString::from));
    let commits = lines_of(&run_git_bytes(git, &arguments, None)?);
    return Ok(UpdateScope::Commits(commits));
}

/// The range functions are controlled in the test module.
#[cfg(test)]
#[path = "manual_push_range_tests.rs"]
mod tests;

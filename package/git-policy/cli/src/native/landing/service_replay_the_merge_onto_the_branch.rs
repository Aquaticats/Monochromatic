//! What:
//!  The tree the captured change makes,
//!  and the merge of that tree onto the branch as it is now:
//!  the first merge with the preparation's base,
//!  the subsumption of the paths another commit changed and the capture already holds,
//!  and the commit that landed first on the paths that still conflict.
//! Why:
//!   The merge base is the preparation's base,
//!  never the worktree's state,
//!  and a path the other commits changed that the captured change already contains is kept as captured,
//!  so a retry lands exactly the bytes the person chose.
//!  A conflict names the commit that landed first in the words the person reads in their own log.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merged = mergeCaptured(site, captured, tip, attempt); // { tree } | { conflict, paths } | { combined }
//! const landed = firstLanded(context, cwd, base, tip, paths);
//! ```

use super::service_replay_the_captured_change::{
    Captured,
    MergeSite,
    Merged,
};
use super::service_replay_the_word_pass::word_pass;

use crate::commit_transaction::{
    TransactionStop,
    failed,
};
use crate::path_bytes::os_from_path_bytes;
use crate::replay_capture_order::CaptureOrderDecision;
use crate::replay_merge::{
    ReplayMerge,
    merge_replay_tree,
};
use crate::replay_shared_paths::{
    SharedPath,
    list_shared_paths,
};
use crate::replay_subsumption::{
    SubsumptionBase,
    subsume_landed_changes,
};
use crate::service_prepare_ready_shapes::LandedFirst;
/// The capture order of every shared path of a replay,
/// from one walk over the history.
use crate::service_replay_capture_order::capture_order_of_each;
use crate::service_replay_line_text::is_not_empty_line;
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git_checked,
};
use std::collections::BTreeMap;
use std::path::Path;

/// What:
///  Merges the captured change onto `tip` with `base` as the merge base.
/// Why:
///  `merge_captured` merges twice at most:
/// with the base of the preparation and with the base the compared commits settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeAt = (site, captured, tip, base) => mergeReplayTree(site, base, tip, captured.tree)
/// ```
fn merge_at(
    site: &MergeSite<'_>,
    captured: &Captured,
    tip: &str,
    base: &str,
) -> Result<ReplayMerge, TransactionStop> {
    return match merge_replay_tree(
        site.context,
        site.common_dir,
        site.cwd,
        base,
        tip,
        captured.tree.as_str(),
    ) {
        Ok(merged) => Ok(merged),
        Err(error) => Err(failed(format!(
            "merging the commit onto the branch failed: {error:?}"
        ))),
    };
}

/// What:
///  Merges the captured change onto `tip` with the preparation base as the merge base.
/// Why:
///   Clean merges,
/// the common case,
/// are one Git run.
/// A conflict is looked at again for the paths whose captured bytes
///  already contain what landed (the person's worktree held the other commit's edit when this one was run),
/// which the
///  merge alone reports as a conflict on adjacent edits;
/// only paths still in conflict after that are a conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mergeCaptured(site, captured, tip) // { tree } | { conflict: paths }
/// ```
pub fn merge_captured(
    site: &MergeSite<'_>,
    captured: &Captured,
    tip: &str,
    attempt: u64,
) -> Result<Merged, TransactionStop> {
    let (conflicting, first_tree, first_conflicts) =
        match merge_at(site, captured, tip, captured.base.as_str())? {
            ReplayMerge::Merged {
                tree_oid,
            } => return Ok(Merged::Tree(tree_oid)),
            ReplayMerge::Conflict {
                paths,
                tree_oid,
                conflicts,
            } => (paths, tree_oid, conflicts),
        };
    let listed: Vec<SharedPath> = match list_shared_paths(
        site.context,
        site.common_dir,
        site.cwd,
        captured.base.as_str(),
        tip,
        captured.tree.as_str(),
    ) {
        Ok(found) => found,
        Err(error) => {
            return Err(failed(format!(
                "listing the paths the branch changed failed: {error:?}"
            )));
        }
    };
    // Only the paths still in conflict are compared with what landed.
    let mut shared: Vec<SharedPath> = Vec::new();
    for entry in listed {
        if conflicting.contains(&entry.path) {
            shared.push(entry);
        }
    }
    let order: BTreeMap<Vec<u8>, CaptureOrderDecision> =
        capture_order_of_each(site, captured, tip, shared.as_slice())?;
    let compared = subsume_landed_changes(
        site.context,
        site.common_dir,
        site.object_directory,
        site.cwd,
        site.scratch,
        attempt,
        captured.base.as_str(),
        shared.as_slice(),
        &order,
    );
    let settled: SubsumptionBase = match compared {
        Ok(base) => base,
        Err(error) => {
            return Err(failed(format!(
                "comparing the commit with what landed failed: {error:?}"
            )));
        }
    };
    if settled.subsumed_paths.is_empty() && settled.kept_landed_paths.is_empty() {
        return Ok(word_pass(
            site,
            tip,
            (first_tree.as_str(), first_conflicts.as_slice()),
            conflicting,
        ));
    }
    return match merge_at(site, captured, tip, settled.merge_base.as_str())? {
        ReplayMerge::Merged {
            tree_oid,
        } => Ok(Merged::Tree(tree_oid)),
        ReplayMerge::Conflict {
            paths,
            tree_oid,
            conflicts,
        } => Ok(word_pass(
            site,
            tip,
            (tree_oid.as_str(), conflicts.as_slice()),
            paths,
        )),
    };
}

/// The revision a walk of the landed history leaves out,
/// which is the base the change was prepared against.
fn excluded_from_the_walk(revision: &str) -> String {
    return format!("^{revision}");
}

/// What:
///  The first commit after `base`,
/// up to `tip`,
/// that changed one of `paths`,
/// with its abbreviated name and subject.
/// Why:
///   The conflict names the commit that landed first,
/// in the words the person sees in their own log.
///  A change captured on a branch that did not exist yet has no commit to leave out,
///  so every commit the branch holds is a candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await firstLanded(context, cwd, base, tip, paths) // { short, subject } | undefined
/// ```
pub fn first_landed(
    context: &GitContext,
    cwd: &Path,
    base: Option<&str>,
    tip: &str,
    paths: &[Vec<u8>],
) -> Result<Option<LandedFirst>, TransactionStop> {
    let exclusion: Option<String> = base.map(excluded_from_the_walk);
    let mut words: Vec<&str> = vec![
        "log",
        "--reverse",
        "--format=%h%x00%s",
        "--no-color",
        "--no-show-signature",
        tip,
    ];
    if let Some(left_out) = exclusion.as_deref() {
        words.push(left_out);
    }
    words.push("--");
    let mut spec_words: GitRequest = GitRequest::new(cwd, words.as_slice());
    for path in paths {
        let mut pathspec: Vec<u8> = b":(top,literal)".to_vec();
        pathspec.extend_from_slice(path.as_slice());
        spec_words
            .arguments
            .push(os_from_path_bytes(pathspec.as_slice()));
    }
    // Every spec here is the wrapper's own, so no pathspec mode the caller set reaches it.
    let request: GitRequest = spec_words.own_pathspecs();
    let listed = run_git_checked(context, &request)?;
    let text: String = String::from_utf8_lossy(listed.stdout.as_slice()).into_owned();
    let Some(first) = text.lines().find(is_not_empty_line) else {
        return Ok(None);
    };
    let (short, subject) = first.split_once('\0').unwrap_or((first, ""));
    return Ok(Some(LandedFirst {
        short: short.to_string(),
        subject: subject.to_string(),
    }));
}

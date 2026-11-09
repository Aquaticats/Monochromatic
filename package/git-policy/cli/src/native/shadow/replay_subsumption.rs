//! What:
//!  Subsumption before replay's three-way merge:
//!       the per-path rules,
//! and the synthetic merge base that makes `git merge-tree` take the prepared bytes
//!       for every subsumed path and the landed bytes for every path capture order drops.
//! Why:
//!  A path whose prepared bytes already contain the landed change keeps the prepared entry,
//!       matching what native sequential commits produce in a shared worktree.
//!       Setting the synthetic base to the landed content means the merge sees no landed change there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await subsumeLandedChanges({ gitPath, shadowPath, objectDirectory, cwd, directory, replay, mergeBase, current, prepared, orderPaths });
//! ```

/// Shared paths and their entries.
use crate::replay_shared_paths::{SharedPath, SharedPathError, TreeEntry, is_regular_triple, is_text_candidate, same_entry};
/// Capture order's per-path decision.
use crate::replay_capture_order::CaptureOrderDecision;
/// The text half of the check.
use crate::replay_subsumption_text::{TextSubsumptionError, subsumed_text_paths};
use crate::replay_shadow::{replay_base_index, run_shadow_checked};
use crate::transaction_git::{GitContext, GitFailure};
/// Decisions per path and the path-keyed maps.
/// The capture-order decisions by path,
/// and the set of the subsumed text paths.
use std::collections::{BTreeMap, HashSet};
/// `Path` names the shadow and the directories.
use std::path::Path;

/// What:
///  Why a subsumption step failed:
///       a listing,
/// the text check,
/// or a Git command that writes the synthetic base.
/// Why:
///   The replay reports each as `transaction-failed`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(message);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubsumptionError {
    /// The shared-path listing failed.
    Shared(SharedPathError),
    /// The text check failed.
    Text(TextSubsumptionError),
    /// A Git command that writes the synthetic base failed.
    Git(GitFailure),
}

/// What:
///  The effective merge base of one replay,
/// and the paths each side keeps.
/// Why:
///   The replay merges against `merge_base`,
/// and reports the prepared and landed paths it settled without a merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SubsumptionBase = { mergeBase; subsumedPaths; keptLandedPaths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubsumptionBase {
    /// Commit to pass as `--merge-base`.
    pub merge_base: String,
    /// Paths whose prepared entries replay as they are:
    /// subsumed ones and those the prepared commit captured later,
    /// in Git order.
    /// Each path is its bytes.
    pub subsumed_paths: Vec<Vec<u8>>,
    /// Paths whose landed entries stay because a landed commit captured them later,
    /// in Git order,
    /// each as its bytes.
    pub kept_landed_paths: Vec<Vec<u8>>,
}

/// What:
///  How the synthetic base settles one path.
/// Why:
///   A subsumed path takes the landed content,
/// a prepared-ordered path takes the landed entry,
///       and a landed-ordered path takes the prepared entry,
/// so the merge keeps the wanted side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BaseEdit = { shared: SharedPath; rule: 'subsumed' | 'prepared' | 'landed' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseRule {
    /// Subsumed by the text check,
    /// identity,
    /// or deletion on both sides.
    Subsumed,
    /// Captured later by the prepared commit:
    /// the synthetic base takes the landed entry.
    Prepared,
    /// Captured later by a landed commit:
    /// the synthetic base takes the prepared entry.
    Landed,
}

/// What:
///  One synthetic-base edit:
/// the path and its rule.
/// Why:
///   The edits are written as one `update-index --index-info` batch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BaseEdit = { shared; rule };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseEdit {
    /// The shared path.
    pub shared: SharedPath,
    /// The settlement rule.
    pub rule: BaseRule,
}

/// What:
///  The object ID width of a shared path's present entries;
/// zero when all are absent.
/// Why:
///   Absent-entry records write zeros of the repository's width.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// oidWidth(shared);
/// ```
fn oid_width(shared: &SharedPath) -> usize {
    for entry in [&shared.base, &shared.landed, &shared.prepared] {
        if let Some(present) = entry {
            return present.oid.len();
        }
    }
    return 0;
}

/// What:
///  One `update-index --index-info` record:
/// its mode,
/// its object and its path,
/// NUL-terminated.
/// Why:
///   The path stays the bytes Git listed,
/// so a record names the file the change came from and not a spelling of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = (mode, oid, path) => Buffer.concat([Buffer.from(`${mode} ${oid}\t`), path, Buffer.from([0])]);
/// ```
fn index_info_record(mode: &str, oid: &str, path: &[u8]) -> Vec<u8> {
    let mut record: Vec<u8> = format!("{mode} {oid}\t").into_bytes();
    record.extend_from_slice(path);
    record.push(0);
    return record;
}

/// What:
///  The `update-index --index-info` record that sets a subsumed path to its landed content.
/// Why:
///   A regular file keeps the base's mode so a mode change still merges three-way;
///       a path the landed side deleted is removed with mode `0`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// subsumedRecord(shared); // NUL-terminated
/// ```
pub fn subsumed_record(shared: &SharedPath) -> Vec<u8> {
    let Some(landed) = &shared.landed else {
        let width: usize = match shared.base.as_ref() {
            Some(entry) => entry.oid.len(),
            None => 0,
        };
        return index_info_record("0", "0".repeat(width).as_str(), shared.path.as_slice());
    };
    let mode: &str = match &shared.base {
        Some(base) if is_regular_triple(shared) => base.mode.as_str(),
        _ => landed.mode.as_str(),
    };
    return index_info_record(mode, landed.oid.as_str(), shared.path.as_slice());
}

/// What:
///  The `update-index --index-info` record that sets a path exactly to one side's entry,
///       so `git merge-tree` sees no change on that side and takes the other side's entry.
/// Why:
///   Capture order replaces the side it drops with the one it keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exactRecord({ path, entry, width });
/// ```
pub fn exact_record(path: &[u8], entry: &Option<TreeEntry>, width: usize) -> Vec<u8> {
    return match entry {
        None => index_info_record("0", "0".repeat(width).as_str(), path),
        Some(present) => index_info_record(present.mode.as_str(), present.oid.as_str(), path),
    };
}

/// What:
///  The record one edit contributes to the synthetic base.
/// Why:
///   The rule picks the subsumed record or the exact entry of the side the merge should keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// baseRecord(edit);
/// ```
pub fn base_record(edit: &BaseEdit) -> Vec<u8> {
    return match edit.rule {
        BaseRule::Subsumed => subsumed_record(&edit.shared),
        BaseRule::Prepared => exact_record(edit.shared.path.as_slice(), &edit.shared.landed, oid_width(&edit.shared)),
        BaseRule::Landed => exact_record(edit.shared.path.as_slice(), &edit.shared.prepared, oid_width(&edit.shared)),
    };
}

/// What:
///  Writes the synthetic merge base:
///       the preparation base with every edit's record applied,
/// then a commit over that tree.
/// Why:
///   The merge base holds the landed content at each subsumed path,
/// so a three-way merge takes the prepared bytes there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeSubsumedBase({ gitPath, shadowPath, directory, replay, mergeBase, subsumed });
/// ```
fn write_subsumed_base(
    context: &GitContext,
    shadow: &Path,
    cwd: &Path,
    directory: &Path,
    replay: u64,
    merge_base: &str,
    edits: &[BaseEdit],
) -> Result<String, SubsumptionError> {
    let index = replay_base_index(directory, replay);
    run_shadow_checked(context, shadow, cwd, &["read-tree", merge_base], None, Some(index.as_path()), &[])
        .map_err(SubsumptionError::Git)?;
    // `mut` lets the records be appended in edit order.
    let mut input: Vec<u8> = Vec::new();
    for edit in edits {
        input.extend_from_slice(&base_record(edit));
    }
    run_shadow_checked(
        context,
        shadow,
        cwd,
        &["update-index", "-z", "--index-info"],
        Some(&input),
        Some(index.as_path()),
        &[],
    )
    .map_err(SubsumptionError::Git)?;
    let tree_output = run_shadow_checked(context, shadow, cwd, &["write-tree"], None, Some(index.as_path()), &[])
        .map_err(SubsumptionError::Git)?;
    let tree: String = String::from_utf8_lossy(&tree_output.stdout).trim().to_string();
    let message: String = format!(
        "tree {tree}\nauthor monochromatic-git-policy-cli <monochromatic-git-policy-cli@localhost> 0 +0000\ncommitter monochromatic-git-policy-cli <monochromatic-git-policy-cli@localhost> 0 +0000\n\nmonochromatic-git-policy-cli replay base with subsumed landed changes\n"
    );
    let commit = run_shadow_checked(
        context,
        shadow,
        cwd,
        &["hash-object", "-t", "commit", "-w", "--stdin"],
        Some(message.as_bytes()),
        None,
        &[],
    )
    .map_err(SubsumptionError::Git)?;
    return Ok(String::from_utf8_lossy(&commit.stdout).trim().to_string());
}

/// What:
///  Whether the prepared and landed entries of a shared path are the same entry:
///  the same mode and object,
/// or both absent.
/// Why:
///   The identity half of the subsumption rules,
/// and only that half:
/// a path whose two sides hold the same entry reaches the merge settled,
///  without the text check.
///  Capture order is not asked here either;
/// `subsume_landed_changes` skips the paths it ordered
///  before calling this,
/// and asks the text check about the entries this returns `false` for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sameEntry({ left: shared.prepared, right: shared.landed })
/// ```
pub fn is_identical(shared: &SharedPath) -> bool {
    return same_entry(&shared.prepared, &shared.landed);
}

/// The capture-order decision for one shared path;
/// a path the order does not name is unordered.
fn decision_of(order: &BTreeMap<Vec<u8>, CaptureOrderDecision>, shared_path: &SharedPath) -> CaptureOrderDecision {
    return order.get(&shared_path.path).copied().unwrap_or(CaptureOrderDecision::Unordered);
}

/// What:
///  Computes the effective merge base of one replay from the shared paths the caller listed once.
/// Why:
///   The result carries the synthetic base when any path settles before the merge,
/// otherwise the preparation base.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await subsumeLandedChanges({ ... });
/// ```
pub fn subsume_landed_changes(
    context: &GitContext,
    shadow: &Path,
    object_directory: &Path,
    cwd: &Path,
    directory: &Path,
    replay: u64,
    merge_base: &str,
    shared: &[SharedPath],
    order: &BTreeMap<Vec<u8>, CaptureOrderDecision>,
) -> Result<SubsumptionBase, SubsumptionError> {
    let mut text_candidates: Vec<SharedPath> = Vec::new();
    for entry in shared {
        if decision_of(order, entry) == CaptureOrderDecision::Unordered && is_text_candidate(entry) && !is_identical(entry) {
            text_candidates.push(entry.clone());
        }
    }
    // A set answers "is this path a subsumed text path" by naming the one collected path it
    //  compares with, so deciding every unordered path is one pass
    //  (`AGENTS.md` `RG2`); the paths beside the question are not read in order.
    let text: HashSet<Vec<u8>> = subsumed_text_paths(context, shadow, cwd, object_directory, &text_candidates)
        .map_err(SubsumptionError::Text)?
        .into_iter()
        .collect();
    // Every synthetic base edit in Git order.
    let mut edits: Vec<BaseEdit> = Vec::new();
    for entry in shared {
        match decision_of(order, entry) {
            CaptureOrderDecision::Prepared => edits.push(BaseEdit { shared: entry.clone(), rule: BaseRule::Prepared }),
            CaptureOrderDecision::Landed => edits.push(BaseEdit { shared: entry.clone(), rule: BaseRule::Landed }),
            CaptureOrderDecision::Unordered => {
                let subsumed: bool = is_identical(entry) || {
                    crate::collected_elements_looked_at::note_looked_at(1);
                    text.contains(&entry.path)
                };
                if subsumed {
                    edits.push(BaseEdit { shared: entry.clone(), rule: BaseRule::Subsumed });
                }
            }
        }
    }
    let mut subsumed_paths: Vec<Vec<u8>> = Vec::new();
    let mut kept_landed_paths: Vec<Vec<u8>> = Vec::new();
    for edit in &edits {
        if edit.rule == BaseRule::Landed {
            kept_landed_paths.push(edit.shared.path.clone());
        } else {
            subsumed_paths.push(edit.shared.path.clone());
        }
    }
    if edits.is_empty() {
        return Ok(SubsumptionBase {
            merge_base: String::from(merge_base),
            subsumed_paths: Vec::new(),
            kept_landed_paths: Vec::new(),
        });
    }
    let synthetic: String = write_subsumed_base(context, shadow, cwd, directory, replay, merge_base, &edits)?;
    return Ok(SubsumptionBase {
        merge_base: synthetic,
        subsumed_paths,
        kept_landed_paths,
    });
}

#[cfg(test)]
#[path = "replay_subsumption_tests.rs"]
mod tests;

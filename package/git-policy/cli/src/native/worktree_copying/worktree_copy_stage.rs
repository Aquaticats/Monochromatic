//! What:
//!  The private,
//! validated copy of one source's ignored state,
//!  staged in a mode-`0700` directory beside a destination worktree.
//! Why:
//!  The stage sits on the destination's filesystem,
//! so a file clone stays local and the install is a rename or a link.
//!  The source is copied entry by entry and compared again at the end,
//!  so a source that changed during the copy never reaches the destination.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const snapshot = await stageIgnoredSnapshot({ sourceRoot, destinationRoot, registeredRoots, gitPath });
//! ```

use crate::private_storage::create_private_directory;
use crate::random_id::random_uuid;
use crate::transaction_git::GitContext;
use crate::diagnostic_log::debug;
pub use crate::worktree_copy_stage_leaf_copying::copy_file_exclusive;
use crate::worktree_copy_stage_leaf_copying::{CopiedManifest, copy_manifest_entries_with, copy_worker_count, create_private_tree, try_clone};
use crate::worktree_copy_settling::{Settled, SettlingRequest, settle_the_copy};
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of, wrap_raw};
use crate::worktree_copy_manifest::{
    FileStamp, WorktreeCopyEntry, collect_pruned_manifest,
};
use crate::worktree_copy_prune::Prune;
use crate::worktree_copy_selection::{CopySelection, STAGE_PREFIX, filesystem_path, is_separator_byte, select_roots, shown_repository_path};
use crate::json_record::quote;
use std::path::{Path, PathBuf};

/// What:
///  The name of the payload directory inside a stage container.
/// Why:
///   The payload mirrors the source's repository paths,
/// so the container can hold other private state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(stageContainer, 'payload')
/// ```
const PAYLOAD_NAME: &str = "payload";

/// What:
///  The validated private snapshot of one source,
/// ready to install.
/// Why:
///   Installation needs the entries it will create,
/// the roots they came from and the stage to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedWorktreeSnapshot = { entries; selectedRoots; sourceRoot; stageContainer; stageRoot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedSnapshot {
    /// Parent-first manifest that was staged and verified.
    pub entries: Vec<WorktreeCopyEntry>,
    /// Git-selected ignored roots that the manifest walked.
    pub selected_roots: Vec<Vec<u8>>,
    /// Canonical source worktree.
    pub source_root: PathBuf,
    /// Private container holding the payload;
    /// removed after installation.
    pub stage_container: PathBuf,
    /// Payload root mirroring the source's repository paths.
    pub stage_root: PathBuf,
    /// Repository paths the user could not read,
    /// left out of the stage;
    ///  empty for a stage rebuilt from a journal.
    pub unreadable: Vec<Vec<u8>>,
    /// Repository paths that kept changing under the copy and were left in their source;
    ///  empty for a stage rebuilt from a journal.
    pub unsettled: Vec<Vec<u8>>,
}

/// What:
///  The registered roots strictly nested under the source,
/// and the stage container,
/// as exclusions.
/// Why:
///   A nested worktree or the stage itself must never be walked as ignored state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sourceExclusions({ sourceRoot, registeredRoots, stageContainer })
/// ```
fn source_exclusions(
    source_root: &Path,
    registered_roots: &[PathBuf],
    stage_container: &Path,
) -> Vec<PathBuf> {
    let mut exclusions: Vec<PathBuf> = registered_roots.to_vec();
    exclusions.push(stage_container.to_path_buf());
    return nested_under(source_root, exclusions.as_slice());
}

/// What:
///  The roots strictly inside the source root,
/// never the source root itself.
/// Why:
///   A registered worktree nested in the source is another checkout's state,
/// so the source's own walk must skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.filter((root) => root !== sourceRoot && isInside(sourceRoot, root))
/// ```
fn nested_under(source_root: &Path, roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut nested: Vec<PathBuf> = Vec::new();
    for root in roots {
        if root.as_path() != source_root && root.starts_with(source_root) {
            nested.push(root.clone());
        }
    }
    return nested;
}

/// What:
///  Whether a path component begins the name of a private stage container or of a snapshot the wrapper made ahead.
/// Why:
///   A path that goes through one is part of another copy's stage,
/// or a whole copy of the source that no worktree owns yet,
/// and is
///  never staged:
/// a snapshot ahead that the ignored state of one new worktree carried would be a second copy of the
///  source inside a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// component.startsWith(STAGE_PREFIX) || component.startsWith(AHEAD_PREFIX)
/// ```
fn starts_private_stage(component: &[u8]) -> bool {
    return component.starts_with(STAGE_PREFIX.as_bytes()) || component.starts_with(crate::worktree_snapshot_ahead::AHEAD_PREFIX.as_bytes());
}

/// What:
///  Whether a path is at or below any of the roots.
/// Why:
///   Registered worktrees and the stage itself inside the source are never staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.some((root) => path.startsWith(root))
/// ```
fn lies_within_any(path: &Path, roots: &[PathBuf]) -> bool {
    for root in roots {
        if path.starts_with(root.as_path()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a repository path names a private stage at any component.
/// Why:
///   A private stage is never selected,
/// even when Git lists it as ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryPath.split('/').some((component) => component.startsWith(STAGE_PREFIX))
/// ```
fn names_private_stage(repository_path: &[u8]) -> bool {
    return repository_path
        .split(is_separator_byte)
        .any(starts_private_stage);
}

/// What:
///  The unreadable entries of one stage as the text a person reads,
/// comma-separated.
/// Why:
///   The verbose log names every entry the copy left out,
/// so the person can look for those names in the new worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = (paths) => paths.map(shownRepositoryPath).join(', ')
/// ```
fn shown_unreadable(unreadable: &[Vec<u8>]) -> String {
    let mut names: Vec<String> = Vec::with_capacity(unreadable.len());
    for path in unreadable {
        names.push(shown_repository_path(path.as_slice()));
    }
    return names.join(", ");
}

/// What:
///  Names a fresh stage container beside a destination worktree,
/// creating nothing.
/// Why:
///   The copy records the name durably before it creates the container,
/// so an interrupted copy can be cleaned up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(dirname(destinationRoot), `${STAGE_PREFIX}${randomUUID()}`)
/// ```
pub fn new_stage_container(destination_root: &Path) -> Result<PathBuf, WorktreeCopyError> {
    let destination_parent: &Path = destination_root.parent().unwrap_or(destination_root);
    let unique: String = match random_uuid() {
        Ok(made) => made,
        Err(error) => return Err(raw_failure(&error.0)),
    };
    return Ok(destination_parent.join(format!("{STAGE_PREFIX}{unique}")));
}

/// What:
///  Stages every selected ignored entry of one source into a named stage container.
/// Why:
///   The stage is removed on any failure,
/// so a failed copy leaves nothing behind for a later invocation to replay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stageIgnoredSnapshot({ sourceRoot, stageContainer, registeredRoots, gitPath }): Promise<StagedWorktreeSnapshot>;
/// ```
pub fn stage_ignored_snapshot(
    context: &GitContext,
    source_root: &Path,
    stage_container: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let selecting: std::time::Instant = std::time::Instant::now();
    let nested: Vec<PathBuf> = nested_under(source_root, registered_roots);
    let git_selected: Vec<Vec<u8>> = select_roots(context, source_root, nested.as_slice(), selection)?;
    debug(
        "worktreeCopyStage",
        format!("selected {} ignored roots in {} ms", git_selected.len(), selecting.elapsed().as_millis()).as_str(),
    );
    create_private_directory(stage_container).map_err(raw_failure_of)?;
    let stage_root: PathBuf = stage_container.join(PAYLOAD_NAME);
    create_private_tree(stage_root.as_path())?;
    let staged: Result<StagedSnapshot, WorktreeCopyError> = stage_into(
        source_root,
        registered_roots,
        stage_container,
        stage_root.as_path(),
        git_selected,
        selection.prune(),
        try_clone,
    );
    // Any failure removes the whole container, so no partial stage survives the attempt.
    match staged {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => {
            // A failed removal replaces the copy failure, as the incumbent's cleanup throws its own error there.
            std::fs::remove_dir_all(stage_container)
                .map_err(raw_failure_of)?;
            return Err(words_for_source(error, source_root));
        }
    }
}

/// What:
///  Words a raw failure of the staging steps with the source it was copying.
/// Why:
///   The user learns which worktree's ignored state could not be staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`monochromatic-git-policy-cli: could not stage ignored state from ${JSON.stringify(sourceRoot)}.`, error)
/// ```
fn words_for_source(error: WorktreeCopyError, source_root: &Path) -> WorktreeCopyError {
    let message: String = format!(
        "monochromatic-git-policy-cli: could not stage ignored state from {}.",
        quote(source_root.to_string_lossy().as_ref())
    );
    return wrap_raw(error, message.as_str());
}

/// What:
///  The copy steps that run once the private stage exists.
/// Why:
///   Separating them lets one cleanup cover every failure after the container is created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { entries = await collectEntryManifest(...); await copyManifestEntries(...); await applyEntryModes(...); await assertFinalSourceEquivalence(...); } catch { await rm(stageContainer, ...); }
/// ```
fn stage_into(
    source_root: &Path,
    registered_roots: &[PathBuf],
    stage_container: &Path,
    stage_root: &Path,
    git_selected: Vec<Vec<u8>>,
    prune: Prune,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let excluded_source_roots: Vec<PathBuf> =
        source_exclusions(source_root, registered_roots, stage_container);
    let mut candidate_roots: Vec<Vec<u8>> = Vec::new();
    for repository_path in git_selected {
        if names_private_stage(repository_path.as_slice()) || prune.skips_path(repository_path.as_slice()) {
            continue;
        }
        let Ok(source_path) = filesystem_path(source_root, repository_path.as_slice()) else {
            continue;
        };
        if lies_within_any(source_path.as_path(), excluded_source_roots.as_slice()) {
            continue;
        }
        candidate_roots.push(repository_path);
    }
    let started: std::time::Instant = std::time::Instant::now();
    let mut unreadable: Vec<Vec<u8>> = Vec::new();
    let entries: Vec<WorktreeCopyEntry> = collect_pruned_manifest(
        source_root,
        candidate_roots.as_slice(),
        excluded_source_roots.as_slice(),
        prune,
        &mut unreadable,
    )?;
    // A root the walk left out whole (another checkout, an unreadable directory) has no entry and is no root of the stage.
    let mut staged_paths: std::collections::HashSet<&[u8]> = std::collections::HashSet::new();
    for entry in &entries {
        staged_paths.insert(entry.relative_path.as_slice());
    }
    let mut selected_roots: Vec<Vec<u8>> = Vec::new();
    for repository_path in candidate_roots {
        if staged_paths.contains(repository_path.as_slice()) {
            selected_roots.push(repository_path);
        }
    }
    drop(staged_paths);
    if !unreadable.is_empty() {
        // The person gets the count and the first paths in one notice line; the verbose log keeps the whole list.
        debug(
            "worktreeCopyStage",
            format!(
                "{} unreadable ignored entries are left out of the copy from {}: {}",
                unreadable.len(),
                source_root.display(),
                shown_unreadable(unreadable.as_slice())
            )
            .as_str(),
        );
    }
    let walked: std::time::Duration = started.elapsed();
    let copied_manifest: CopiedManifest =
        copy_manifest_entries_with(source_root, stage_root, entries.as_slice(), clone)?;
    let copied: std::time::Duration = started.elapsed();
    // The copy settles the stage against the source: what went away is dropped, what changed is copied again,
    //  and what will not hold still stays in its source and is named rather than failing an add Git already made.
    let settled: Settled = settle_the_copy(SettlingRequest {
        source_root,
        stage_root,
        selected_roots: selected_roots.as_slice(),
        excluded_source_roots: excluded_source_roots.as_slice(),
        prune,
        entries,
        stamps: copied_manifest.stamps,
        clone,
        gone: copied_manifest.gone,
    })?;
    let staged_entries: Vec<WorktreeCopyEntry> = settled.entries;
    let stamps: Vec<Option<FileStamp>> = settled.stamps;
    debug(
        "worktreeCopyStage",
        format!(
            "staged {} entries ({} cloned under a stamp) in {} ms: walk {} ms, copy by {} workers {} ms, settling and modes {} ms; {} went away and {} kept changing",
            staged_entries.len(),
            stamps.iter().flatten().count(),
            started.elapsed().as_millis(),
            walked.as_millis(),
            copy_worker_count(staged_entries.len()),
            (copied - walked).as_millis(),
            (started.elapsed() - copied).as_millis(),
            settled.gone,
            settled.unsettled.len()
        )
        .as_str(),
    );
    return Ok(StagedSnapshot {
        entries: staged_entries,
        selected_roots,
        source_root: source_root.to_path_buf(),
        stage_container: stage_container.to_path_buf(),
        stage_root: stage_root.to_path_buf(),
        unreadable,
        unsettled: settled.unsettled,
    });
}

/// Stage controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_tests.rs"]
mod tests;

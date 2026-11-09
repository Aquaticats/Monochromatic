//! What:
//!  The settling of a staged copy against the source it was copied from:
//! the passes that compare the two trees,
//! that copy one entry again when it
//!  changed under the copy,
//! and that leave an entry in its source and name it when it will not hold still.
//! Why:
//!  A main checkout that builds and runs agents rewrites and removes its ignored files
//!  constantly,
//! so a copy taken over seconds always meets a source that moved:
//! a file that went away has nothing
//!  left to copy,
//! and a cache file that changed is worth one more copy.
//! Failing the whole add for that
//!  costs the person a worktree with no ignored state and a branch they did not ask for,
//! twice in an hour
//!  on the owner's machine (2026-11-09),
//! so an entry that keeps changing is left where it is,
//! in its source,
//! and
//!  named,
//! never deleted and never installed half-copied.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { entries, stamps, unsettled } = settleTheCopy({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, prune, entries, stamps, clone });
//! ```

use crate::worktree_copy_compare::{FinalCheck, differences_from_the_source};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_manifest::{FileStamp, WorktreeCopyEntry, apply_entry_modes};
use crate::worktree_copy_prune::Prune;
use crate::worktree_copy_selection::filesystem_path;
use crate::worktree_copy_stage_leaf_copying::{CopiedLeaf, copy_one_leaf};
use std::path::{Path, PathBuf};

/// What:
///  How many more times one entry that changed under the copy is copied before it is left in its source.
/// Why:
///   The changes this exists for are a build rewriting one stamp or cache file,
/// which settles within the
///  next copy;
/// two more tries cover a writer that is a moment behind the copy without turning a
///  churning tree into a long loop.
/// The coordinator's suggestion under the owner's requirement that an add does not
///  fail for ignored churn,
/// open to the owner's veto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SETTLING_RETRIES = 2;
/// ```
const SETTLING_RETRIES: usize = 2;

/// What:
///  How many comparison passes one copy may make before it reports the entry that will not settle.
/// Why:
///   A pass drops or copies again at least one entry,
/// so the loop ends on its own;
/// this ceiling keeps a
///  checkout whose ignored tree churns faster than the copy can prove it from costing an unbounded
///  number of walks ("nothing should ever wait forever or indefinitely").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MOST_PASSES = 6;
/// ```
const MOST_PASSES: usize = 6;

/// What:
///  Everything one settling run reads:
/// the two roots,
/// the manifest,
/// the stamps the copy took,
/// the rule the
///  staged walk used,
/// and the clone request.
/// Why:
///   One value names the inputs,
/// and the entries and their stamps travel together because a pass
///  filters both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettlingRequest = { sourceRoot: string; stageRoot: string; selectedRoots: string[]; excludedSourceRoots: string[]; prune: Prune; entries: WorktreeCopyEntry[]; stamps: (FileStamp | undefined)[]; clone: CloneRequest };
/// ```
pub struct SettlingRequest<'a> {
    /// The checkout the copy came from.
    pub source_root: &'a Path,
    /// The private payload the entries are copied into.
    pub stage_root: &'a Path,
    /// The roots Git selected, relative to the source.
    pub selected_roots: &'a [String],
    /// The source roots no walk enters.
    pub excluded_source_roots: &'a [PathBuf],
    /// The leave-out rule the staged walk uses.
    pub prune: Prune,
    /// The manifest the copy staged.
    pub entries: Vec<WorktreeCopyEntry>,
    /// The stamp of each entry, in manifest order.
    pub stamps: Vec<Option<FileStamp>>,
    /// The clone request the copy and its retries use.
    pub clone: fn(&std::fs::File, &std::fs::File) -> bool,
    /// The manifest entries the copy found gone, which the settling drops before it compares anything.
    pub gone: Vec<String>,
}

/// What:
///  The manifest the stage now holds,
/// its stamps,
/// and the entries left in their source because they would not settle.
/// Why:
///   The caller installs exactly these entries and tells the person about the others;
/// nothing left out is
///  deleted from the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Settled = { entries: WorktreeCopyEntry[]; stamps: (FileStamp | undefined)[]; unsettled: string[] };
/// ```
pub struct Settled {
    /// The entries the stage holds.
    pub entries: Vec<WorktreeCopyEntry>,
    /// The stamp of each of them, in entry order.
    pub stamps: Vec<Option<FileStamp>>,
    /// The entries left in the source because they kept changing under the copy.
    pub unsettled: Vec<String>,
    /// How many manifest entries the source no longer held when their turn came.
    pub gone: usize,
}

/// What:
///  Compare the stage with the source until the two agree about every entry the stage holds,
/// copying an
///  entry that changed again and dropping one the source no longer has.
/// Why:
///   The stage may only be installed for what the source held at the instant it was compared,
/// and an
///  entry the source removed has nothing to install.
/// An entry that changed after the last retry stays in its
///  source,
/// named,
/// rather than failing an add whose worktree Git already made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleTheCopy(request) { for (let pass = 0; ; pass += 1) { const d = differences(request); if (!d.gone.length && !d.changed.length) return { entries, stamps, unsettled }; dropGone(d.gone); if (d.changed.length) { if (pass < SETTLING_RETRIES && pass < MOST_PASSES) copyAgain(d.changed) else { leave(d.changed); unsettled.push(...d.changed) } } else if (pass > MOST_PASSES) return failure } }
/// ```
pub fn settle_the_copy(request: SettlingRequest<'_>) -> Result<Settled, WorktreeCopyError> {
    let mut entries: Vec<WorktreeCopyEntry> = request.entries;
    let mut stamps: Vec<Option<FileStamp>> = request.stamps;
    let mut unsettled: Vec<String> = Vec::new();
    let gone_count: usize = request.gone.len();
    // An entry the copy found gone has no stage file, so it leaves the manifest before the modes are applied.
    leave_out(&mut entries, &mut stamps, request.gone.as_slice(), request.stage_root)?;
    // The comparison compares modes, and a copy makes its own entries private, so the source's modes are applied first.
    apply_entry_modes(request.stage_root, entries.as_slice())?;
    let mut pass: usize = 0;
    loop {
        let differences = differences_from_the_source(
            request.source_root,
            request.stage_root,
            request.selected_roots,
            request.excluded_source_roots,
            entries.as_slice(),
            &FinalCheck { prune: request.prune, stamps: stamps.as_slice() },
        )?;
        if differences.gone.is_empty() && differences.changed.is_empty() {
            return Ok(Settled { entries, stamps, unsettled, gone: gone_count });
        }
        // An entry the source removed has no stage file to keep and nothing to install.
        leave_out(&mut entries, &mut stamps, differences.gone.as_slice(), request.stage_root)?;
        if differences.changed.is_empty() {
            pass = next_pass(pass, differences.gone.as_slice())?;
            continue;
        }
        if pass < SETTLING_RETRIES {
            copy_again(&mut entries, &mut stamps, differences.changed.as_slice(), request.source_root, request.stage_root, request.clone)?;
            pass += 1;
            continue;
        }
        // Left where it is, in its source, and named: a file a build keeps writing is no reason to fail the add.
        leave_out(&mut entries, &mut stamps, differences.changed.as_slice(), request.stage_root)?;
        unsettled.extend(differences.changed);
        pass = next_pass(pass, &[])?;
    }
}

/// What:
///  The pass number of the next comparison,
/// or the reason the copy stops looking.
/// Why:
///   Every pass drops or copies again at least one entry,
/// so the loop would end on its own;
/// the
///  ceiling is what stops a source that churns faster than the copy can prove it from walking forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nextPass = (pass, dropped) => { if (pass >= MOST_PASSES) throw `the ignored entries would not hold still while the copy was made`; return pass + 1; };
/// ```
fn next_pass(pass: usize, dropped: &[String]) -> Result<usize, WorktreeCopyError> {
    let _counted: usize = dropped.len();
    if pass >= MOST_PASSES {
        return Err(crate::worktree_copy_error::failure_with_cause(
            "monochromatic-git-policy-cli: the ignored entries of the source kept changing for every pass of the copy, so the add did not install the staged state; nothing of the source was changed or removed.",
            None,
        ));
    }
    return Ok(pass + 1);
}

/// What:
///  Copy the given entries again into the stage,
/// after removing what the last attempt put there.
/// Why:
///   `copy_file_exclusive_with` creates its destination exclusively,
/// so the staged copy of an entry that
///  changed has to go before it can be copied again;
/// the source is only ever read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of changed) { rm(join(stageRoot, path), { force: true }); const slot = entries.findIndex((e) => e.relativePath === path); stamps[slot] = copyOneLeaf(...); }
/// ```
fn copy_again(
    entries: &mut [WorktreeCopyEntry],
    stamps: &mut [Option<FileStamp>],
    changed: &[String],
    source_root: &Path,
    stage_root: &Path,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<(), WorktreeCopyError> {
    for path in changed {
        let Some(index) = entries.iter().position(|entry: &WorktreeCopyEntry| { return entry.relative_path == *path; }) else { continue };
        let stage_path: PathBuf = filesystem_path(stage_root, path.as_str())?;
        remove_the_staged_copy(stage_path.as_path())?;
        let entry: WorktreeCopyEntry = entries[index].clone();
        let copied: CopiedLeaf = copy_one_leaf(source_root, stage_path.as_path(), &entry, clone)?;
        // The comparison compares modes, and a copy makes its own file with the private one, so the source's mode is put
        //  back before the next pass reads it.
        apply_entry_modes(stage_root, std::slice::from_ref(&entry))?;
        stamps[index] = match copied {
            CopiedLeaf::Staged(stamp) => stamp,
            // The source removed it between the comparison and this copy, so the next pass drops it.
            CopiedLeaf::Gone => None,
        };
    }
    return Ok(());
}

/// What:
///  Drop the given entries from the manifest and the stage.
/// Why:
///   An entry the stage does not hold is no entry of the manifest,
/// and one the source no longer holds
///  leaves nothing to install;
/// the stage file goes so the staged tree names only what will be installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of paths) { rm(join(stageRoot, path), { force: true }); entries = entries.filter((e) => e.relativePath !== path); }
/// ```
fn leave_out(
    entries: &mut Vec<WorktreeCopyEntry>,
    stamps: &mut Vec<Option<FileStamp>>,
    paths: &[String],
    stage_root: &Path,
) -> Result<(), WorktreeCopyError> {
    for path in paths {
        let stage_path: PathBuf = filesystem_path(stage_root, path.as_str())?;
        remove_the_staged_copy(stage_path.as_path())?;
        let Some(index) = entries.iter().position(|entry: &WorktreeCopyEntry| { return entry.relative_path == *path; }) else { continue };
        entries.remove(index);
        stamps.remove(index);
    }
    return Ok(());
}

/// What:
///  Remove one staged copy of an entry,
/// and say nothing when it is already gone.
/// Why:
///   The stage is this command's private folder,
/// and an entry that was never copied has nothing there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rmSync(path, { force: true })
/// ```
fn remove_the_staged_copy(stage_path: &Path) -> Result<(), WorktreeCopyError> {
    return match std::fs::remove_file(stage_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(crate::worktree_copy_error::failure_with_cause(
            format!("{} cannot be removed from the private stage ({error})", stage_path.display()).as_str(),
            None,
        )),
    };
}

/// Settling controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_settling_tests.rs"]
mod tests;

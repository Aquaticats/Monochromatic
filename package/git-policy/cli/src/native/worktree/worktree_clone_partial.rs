//! What:
//!  The clone rung for a source with uncommitted changes:
//! clone every tracked path the proof settles,
//! build the new worktree's index from the tree of the source's `HEAD`,
//! and let Git write every other path,
//!  the changed ones and the ones the conversion fence takes.
//! Why:
//!  A source checkout that is not clean used to end the clone rung,
//! so the machine whose main checkout other
//!  sessions keep dirty paid the clean proof and then Git's whole checkout on every add.
//! The files Git's
//!  checkout would write are the tree's bytes,
//! and a path whose file its own index entry still proves already holds
//!  them,
//! so those paths are cloned and only the paths a change reaches are written by Git
//!  (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`).
//! The index cannot come
//!  from the source:
//! a staged path records another blob,
//! a staged deletion is missing from it,
//! and a staged addition
//!  names a path no tree has,
//! so the index is built from the tree with the stat fields of the files the clones made,
//!  and written under the fence's own waiting name (`worktree_clone_index`).
//! Git writes the rest through the fence's one
//!  `git checkout-index`,
//! which runs the checkout's conversion filters exactly as Git's own checkout does,
//! and the index takes the
//!  worktree's name only after that run,
//! so a kill in the window leaves no index that could vouch for a path Git had not
//!  written.
//! The two provisional limits that send a source to Git's own checkout instead live beside this module
//!  (`worktree_clone_rung_limits`),
//! and the ladder asks them before it chooses this rung.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const part = cloneTheUnchanged({ context, source, destination, admin, adminPath, fence }); // throws the reason Git's checkout is needed
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::diagnostic_log::debug;
use crate::parallel_budget::HostResources;
use crate::subvolume_index::StatFields;
use crate::transaction_git::GitContext;
#[cfg(test)]
use crate::transaction_git::{
    GitRequest,
    run_git,
};
use crate::worktree_clone::Created;
use crate::worktree_clone::clone_these_bytes_beside;
use crate::worktree_clone_conversion_beside_the_clone::fence_beside_the_clone;
use crate::worktree_clone_conversion_fence::{
    AttributeFence,
    Fence,
};
use crate::worktree_clone_index::{
    PendingIndex,
    install_the_index_beside_the_write,
};
use crate::worktree_clone_partial_index::index_of_the_tree;
use crate::worktree_clone_partial_split::{
    Split,
    split_the_tree,
};
use crate::worktree_clone_partial_staged::{
    a_staged_path_among_the_clones,
    staged_paths,
};
use crate::worktree_clone_partial_tracked::{
    Tracked,
    read_tracked,
    the_recorded_stats,
    the_written_paths,
};
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::{
    HashMap,
    HashSet,
};
use std::fs::File;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeClonePartial";

/// What:
///  What one partial clone made.
/// Why:
///   A later step that fails reports the entries the clones created,
/// and the two counts say what the rung did
///  without reading the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PartCloned = { cloned: number; written: number; created: Created[] };
/// ```
#[derive(Debug)]
pub struct PartCloned {
    /// How many tracked paths were cloned.
    pub cloned: usize,
    /// How many tracked paths Git wrote.
    pub written: usize,
    /// The entries the clones created,
    /// in creation order.
    pub created: Vec<Created>,
}

impl PartCloned {
    /// The entries the clones created,
    /// in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }
}

/// What:
///  One partial clone's inputs:
/// what it reads from Git,
/// what the source check proved,
/// where its bytes go,
/// the fence the attributes
///  half already answered,
/// and the host the copies' budget is derived from.
/// Why:
///   The rung reads the source,
/// the destination and the administrative directory of the new worktree,
/// and it must not choose the conversion fence itself:
/// the ladder decided the decline and read the attributes before any copy (`the_fence_of_this_build`).
/// The host is an input of the same kind:
/// a control states the processors and the memory its own run is budgeted on,
/// and the ladder reads the live host once (issue 6537).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PartialClone = { context: GitContext; source: SnapshotSource; destination: File; admin: File; adminPath: Path; fence: AttributeFence; host: HostResources };
/// ```
pub(crate) struct PartialClone<'a> {
    /// The Git face the rung's own runs go through.
    pub context: &'a GitContext,
    /// What the source check proved about the source.
    pub source: &'a SnapshotSource,
    /// The index entries the limits already read,
    /// or nothing when this step must read them.
    pub entries: Option<Vec<crate::worktree_clean::StatEntry>>,
    /// The directory Git made for the new worktree,
    /// with no checkout,
    /// held open.
    pub destination: &'a File,
    /// The new worktree's administrative directory,
    /// held open.
    pub admin: &'a File,
    /// The new worktree's administrative directory,
    /// as the path a Git run is given.
    pub admin_path: &'a Path,
    /// The attributes the fence answered before the copies,
    /// and the rules it judged them by.
    pub fence: AttributeFence,
    /// The processors and the memory the copies' budget is derived from.
    pub host: HostResources,
}

/// What:
///  Clone the unchanged tracked paths of a source with changes into the destination,
/// write the index built from
///  the tree of the source's `HEAD` under the fence's waiting name,
/// let the fence answer beside the copies,
/// and answer the
///  entries the clones made,
/// the fence's decision widened by the changed paths,
/// and the waiting index.
/// Why:
///   See the module description.
/// The order is the fence's own (`worktree_clone_conversion_beside_the_clone`):
/// the attributes and the decline were
///  decided before anything was copied,
/// the remaining reads run beside the copies,
/// the index is written under a name of its own with no
///  stat data for a path Git still has to write,
/// and it takes the worktree's name only after that write,
/// which the
///  ladder's serving step does.
/// Nothing is removed on a failure:
/// the entries the clones created stay and are counted by the
///  caller,
/// and the checkout that completes the worktree overwrites the tracked ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { created, decided, pending } = cloneTheUnchanged({ context, source, destination, admin, adminPath, fence }); // throws the reason
/// ```
pub(crate) fn clone_the_unchanged(
    work: PartialClone<'_>
) -> Result<(PartCloned, Fence, PendingIndex), String> {
    let PartialClone {
        context,
        source,
        entries,
        destination,
        admin,
        admin_path,
        fence,
        host,
    }: PartialClone<'_> = work;
    let read_entered: bool = begin_step("partRead");
    let read_outcome = read_tracked(source, entries);
    end_step(read_entered);
    let tracked: Tracked = read_outcome?;
    let classify_entered: bool = begin_step("partClassify");
    let classify_outcome = split_the_tree(
        tracked.entries.as_slice(),
        tracked.indexed.as_slice(),
        tracked.tree.as_slice(),
        source.root.as_path(),
        source.clean.index_seconds,
    );
    end_step(classify_entered);
    let split: Split = classify_outcome.map_err(|refusal| {
        return refusal.reason;
    })?;
    // What:
    //  A split that leaves Git more paths to write than the changed-file cut ends the rung here,
    //  before any file is copied.
    // Why:
    //  The cut the ladder asks counts `git status` entries,
    //  and a path whose recorded file times are stale but whose bytes are unchanged is no status entry,
    //  although the split writes it and Git pays for it:
    //  the integration pass measured 20,000 such paths at 1.39 times `main` (its finding M1).
    //  The split already holds the count,
    //  so the same refusal path the ladder takes costs nothing here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cut = theChangedFileCut(tracked.entries.length);
    // if (split.writes.length > cut) throw `...`;
    // ```
    let cut: usize = crate::worktree_clone_rung_limits::the_changed_file_cut(tracked.entries.len());
    if split.writes.len() > cut {
        let changed: usize = split.writes.len() - split.stale_times.len();
        return Err(format!(
            "Git must write {} of the {} tracked files of {}: the file times recorded for {} of them no longer match their files, and {} of them changed or were staged; that is more than the {} the add clones before Git writes the rest. Run `git status` once in {} to refresh the times, and the next add clones",
            split.writes.len(),
            tracked.entries.len(),
            crate::worktree_snapshot_source::plain(source.root.display().to_string().as_str()),
            split.stale_times.len(),
            changed,
            cut,
            crate::worktree_snapshot_source::plain(source.root.display().to_string().as_str())
        ));
    }
    let staged_entered: bool = begin_step("partStaged");
    let staged_outcome = staged_paths(context, source).and_then(|staged| {
        return match a_staged_path_among_the_clones(
            staged.as_slice(),
            tracked.entries.as_slice(),
            split.clones.as_slice(),
        ) {
            Some(refusal) => Err(refusal),
            None => Ok(()),
        };
    });
    end_step(staged_entered);
    staged_outcome?;
    let cloned_count: usize = split.clones.len();
    let beside_entered: bool = begin_step("conversionBesideTheClone");
    let (cloned, answered) = clone_these_bytes_beside(
        source,
        split.cloned_entries(tracked.entries.as_slice()),
        destination,
        &host,
        fence_beside_the_clone(source, context, fence),
    )?;
    end_step(beside_entered);
    // A fence's reason, or a pool that would not run it, ends the rung here before any index exists: Git's own checkout
    // writes the worktree.
    let decided: Fence = match answered {
        Some(Ok(decided)) => decided,
        Some(Err(reason)) => return Err(reason),
        None => return Err(String::from("the fence beside the copies answered nothing")),
    };
    let written: Vec<Vec<u8>> = the_written_paths(&split, &decided);
    let written_count: usize = written.len();
    let set: HashSet<Vec<u8>> = written.iter().cloned().collect();
    let recorded: HashMap<Vec<u8>, StatFields> = the_recorded_stats(
        cloned.stats().clone(),
        &split,
        tracked.entries.as_slice(),
        &set,
    );
    let index_entered: bool = begin_step("partIndex");
    let seed: Result<Vec<u8>, String> =
        index_of_the_tree(tracked.tree.as_slice(), &recorded, source.clean.hash_length);
    let index_outcome = seed.and_then(|bytes| {
        return install_the_index_beside_the_write(
            (admin, admin_path),
            bytes.as_slice(),
            &recorded,
            &set,
            source.clean.hash_length,
        );
    });
    end_step(index_entered);
    let pending: PendingIndex = index_outcome?;
    debug(
        TAG,
        format!(
            "cloned {cloned_count} of {} tracked paths and had Git write {written_count} of them",
            tracked.tree.len()
        )
        .as_str(),
    );
    let part: PartCloned = PartCloned {
        cloned: cloned_count,
        written: written_count,
        created: cloned.into_created(),
    };
    return Ok((
        part,
        Fence {
            git_writes: written,
            carried: decided.carried,
        },
        pending,
    ));
}

/// What:
///  Prove that the new worktree holds what Git's own `worktree add` would have written:
/// no worktree file
///  differs from the index entry Git's own status judges,
/// and the index equals the new `HEAD`'s tree.
/// Why:
///   The clones and the paths Git wrote must leave the state Git's checkout leaves,
/// and Git's status is the
///  judge a person would ask,
/// so the proof is Git's own answer and not this process's opinion of it.
/// `--no-optional-locks` is
///  what keeps the proof from writing an index:
/// a read-only proof must not leave a mark on the worktree it
///  judges,
/// and an entry whose fields stayed stale costs the person one hash later,
/// never a wrong byte.
/// It is a control's answer,
/// not a step of the rung:
/// the shipped rung's guarantee is the index it wrote and the fence's write,
/// and this reads the result the
///  way the measurement harness does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheWorktree({ context, destination, newCommit }); // throws the reason it is not Git's result
/// ```
#[cfg(test)]
pub fn prove_the_worktree(
    context: &GitContext,
    destination: &Path,
    admin_path: &Path,
    new_commit: &str,
) -> Result<(), String> {
    let (listed_code, listed) = ask_in_the_worktree(
        context,
        destination,
        admin_path,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
        ],
    )?;
    if listed_code != Some(0) {
        return Err(format!(
            "`git status` ended with {listed_code:?} in the new worktree"
        ));
    }
    if let Some(first) = listed
        .split(|byte: &u8| {
            return *byte == 0;
        })
        .find(|entry| {
            return !entry.is_empty();
        })
    {
        let shown: String = String::from_utf8_lossy(&first[3.min(first.len())..]).into_owned();
        return Err(format!(
            "the new worktree is not what Git's checkout makes: {} differs",
            crate::worktree_snapshot_source::plain(shown.as_str())
        ));
    }
    let (compared_code, _quiet) = ask_in_the_worktree(
        context,
        destination,
        admin_path,
        &[
            "--no-optional-locks",
            "diff-index",
            "--cached",
            "--quiet",
            new_commit,
        ],
    )?;
    if compared_code != Some(0) {
        return Err(String::from(
            "the new worktree's index does not equal the tree of the new HEAD",
        ));
    }
    return Ok(());
}

/// What:
///  Run one captured Git query in the new worktree and answer its status and standard output,
/// or the reason it
///  could not be run at all.
/// Why:
///   The proof asks Git twice,
/// and both runs name the worktree's own Git directory and work tree,
/// so neither
///  can read the source's.
/// A run that reports a failure through its status is an answer,
/// not an error:
/// a quiet difference
///  check says "they differ" with status 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, stdout } = askInTheWorktree({ context, destination, adminPath, words }); // throws the reason it did not start
/// ```
#[cfg(test)]
fn ask_in_the_worktree(
    context: &GitContext,
    destination: &Path,
    admin_path: &Path,
    words: &[&str],
) -> Result<(Option<i32>, Vec<u8>), String> {
    let request: GitRequest = GitRequest {
        cwd: destination.to_path_buf(),
        arguments: words
            .iter()
            .copied()
            .map(std::ffi::OsString::from)
            .collect(),
        without_prefix: true,
        environment: crate::worktree_ladder_target::checkout_environment(admin_path, destination),
        ..GitRequest::default()
    };
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(format!(
                "`git {}` could not start ({error})",
                words.join(" ")
            ));
        }
    };
    return Ok((output.code, output.stdout));
}

/// Partial-clone controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_partial_tests.rs"]
mod tests;

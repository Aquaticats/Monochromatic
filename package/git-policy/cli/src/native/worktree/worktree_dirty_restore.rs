//! What:
//!  The restore of a snapshot of a main checkout that had uncommitted changes:
//! make the copy's index and files equal the
//!  tree of the new worktree's `HEAD` and prove it in the stage.
//! Why:
//!  `git worktree add` from a checkout with changes makes a worktree that holds the committed files and nothing of the changes.
//!  The snapshot carries the changes,
//! so the copy is reset the way Git's own `reset --hard` would reset it,
//! from the index the
//!  snapshot carries:
//! that index still records the files as they were before they were edited,
//! so the reset rewrites exactly the
//!  edited,
//! deleted and staged paths and leaves the rest.
//! Untracked files that no ignore rule covers go,
//! ignored files stay:
//! the `clean` runs
//!  before the reset,
//! while the stage still carries the source's ignore rules,
//! so a commit that ignores
//!  less than the source does cannot put the person's cache,
//! local configuration or environment file in the
//!  way of the rule that decides what goes.
//! The stage then holds what the ignored-state copy hands over on
//!  every other filesystem (`SPEC.md`,
//! "Linked-worktree ignored-state synchronization"),
//! and the proof
//!  counts those paths as carried instead of as untracked.
//!  Both Git runs are the ladder's own steps on the new worktree (its Git directory and its stage),
//! started as real Git
//!  children by the lifecycle's runner,
//! so no policy of the wrapper that guards a person's `reset` or `clean` applies to
//!  them.
//! The reset rewrites a path whose file Git's own stat check can see changed;
//! an edit of the same size,
//!  made in the same second as the entry that recorded the file,
//! keeps every field that check compares,
//!  so the files the index's stat data cannot vouch for are read and compared with the blobs their
//! entries name,
//! and those whose bytes differ are removed before the reset,
//! which then writes the commit's bytes
//!  there.
//! The proof runs before the stage is swapped in;
//! a proof that fails ends the rung and Git's own checkout makes the
//!  worktree.
//! The main checkout is read only to be snapshotted and listed,
//! and is never written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await restoreDirtySnapshot({ source, target, stage, runGit, context }); // throws the reason it cannot be proven
//! ```

use crate::fs_beneath::{
    open_root,
    plain_components,
    remove_at,
};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::inner_git_words::record_words;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
use crate::worktree_clean::{
    ContentTarget,
    paths_whose_content_differs,
    stored_stats,
};
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_copy::GitRunner;
pub use crate::worktree_dirty_restore_left_out::{
    left_out_note,
    record_left_out,
};
use crate::worktree_dirty_restore_proof::prove_restored;
use crate::worktree_dirty_restore_stage_git_questions::{
    arguments_in_stage,
    listed_ignored,
};
use crate::worktree_ladder_target::{
    Target,
    checkout_environment_with_index,
    run_step,
};
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::{
    ChangedPaths,
    SnapshotSource,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeDirtyRestore";

/// What:
///  The reason the new worktree's index could not be read,
/// in words.
/// Why:
///   The walk over the stage names what it cannot read,
/// and a step that fails ends the rung rather than
///  guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = `the index of the new worktree is unreadable (${error})`;
/// ```
fn index_unreadable(error: std::io::Error) -> String {
    return format!("the index of the new worktree is unreadable ({error})");
}

/// The reason the new worktree's index could not be examined,
/// in words.
fn index_unexamined(error: std::io::Error) -> String {
    return format!("the index of the new worktree cannot be examined ({error})");
}

/// The words of a refusal the content decision made.
fn the_reason_of(refusal: crate::worktree_clean::CleanRefusal) -> String {
    return refusal.reason;
}

/// What:
///  Remove from the stage the tracked files the index's stat data cannot vouch for,
/// so that the reset writes the
///  commit's bytes there,
/// and answer how many were removed.
/// Why:
///   `git read-tree --reset -u` rewrites a path whose file it can see changed,
/// and an edit that keeps a file's
///  size,
/// its modification time and the second of its change time is not one it can see:
/// removing the
///  file is what makes the reset write it again.
/// The stage is the wrapper's own snapshot of a checkout the main checkout
///  keeps,
/// and the copy is published only after the restore proved it,
/// so no byte of a person's is destroyed here;
///  a file the walk could not judge is removed too,
/// which costs a write of the commit's content and never publishes the
///  snapshot's.
/// A step that cannot finish ends the rung,
/// which destroys the stage and lets Git's own checkout make the
///  worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = removeTheFilesTheResetCannotSee({ source, target, stage, conversion }); // throws the reason
/// ```
fn remove_the_files_the_reset_cannot_see(
    source: &SnapshotSource,
    target: &Target,
    stage: &Path,
    conversion: &ConversionEnvironment,
) -> Result<usize, String> {
    let index_path: PathBuf = target.index_file();
    let index: Vec<u8> = std::fs::read(index_path.as_path()).map_err(index_unreadable)?;
    let index_seconds: u32 = std::fs::metadata(index_path.as_path())
        .map_err(index_unexamined)?
        .mtime() as u32;
    let hash_length: usize = source.clean.hash_length;
    let format: ObjectFormat = if hash_length == 32 {
        ObjectFormat::Sha256
    } else {
        ObjectFormat::Sha1
    };
    let (Some(entries), Some(indexed)) = (
        stored_stats(index.as_slice(), hash_length),
        parse_index(index.as_slice(), format),
    ) else {
        return Err(String::from(
            "the index of the new worktree has a layout the reader does not know",
        ));
    };
    let content: ContentTarget<'_> = ContentTarget {
        root: stage,
        git_dir: target.admin.as_path(),
        common_dir: source.common_dir.as_path(),
        environment: conversion,
        format,
    };
    let hidden: Vec<Vec<u8>> = paths_whose_content_differs(
        &content,
        entries.as_slice(),
        indexed.as_slice(),
        index_seconds,
    )
    .map_err(the_reason_of)?;
    if hidden.is_empty() {
        return Ok(0);
    }
    let held: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    let Ok(mut chain) = DirectoryChain::new(&held) else {
        return Err(String::from("the stage cannot be examined"));
    };
    for path in &hidden {
        let shown: String = String::from_utf8_lossy(path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(path.as_slice()))) else {
            return Err(format!("{shown} is not a plain path"));
        };
        let Some((name, parents)) = names.split_last() else {
            return Err(format!("{shown} is not a plain path"));
        };
        let Some(directory) = chain.directory(parents).ok() else {
            return Err(format!(
                "{shown} cannot be reached without following a link"
            ));
        };
        match remove_at(directory, name.as_os_str(), false) {
            Ok(()) => {}
            // A file that is already gone is one the reset writes, so nothing is lost when it cannot be removed twice.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "{shown} could not be removed from the snapshot ({error})"
                ));
            }
        }
    }
    crate::diagnostic_log::debug(
        TAG,
        format!(
            "removed {} files the reset cannot see: the commit's bytes replace them",
            hidden.len()
        )
        .as_str(),
    );
    return Ok(hidden.len());
}

/// What:
///  Restore the stage,
/// which is a snapshot of `source` made with its own index,
/// to the tree of the target commit,
/// and prove it.
/// Why:
///   See the module description.
/// The stage is never the source's own directory,
/// and the Git directory is the new worktree's,
///  so neither run can reach the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await restoreDirty({ source, target, stage, runGit, context, conversion }); // throws the reason
/// ```
pub fn restore_dirty(
    source: &SnapshotSource,
    target: &Target,
    stage: &Path,
    run_git_step: &mut dyn GitRunner,
    context: &GitContext,
    conversion: &ConversionEnvironment,
) -> Result<(), String> {
    if stage == source.root.as_path()
        || target.admin == source.git_dir
        || target.admin == source.common_dir
    {
        return Err(String::from(
            "the restore would run in the source's own directory",
        ));
    }
    let began: std::time::Instant = std::time::Instant::now();
    let environment = checkout_environment_with_index(
        (target.admin.as_path(), stage),
        Some(target.index_file().as_path()),
    );
    // The ignored paths are listed, and the clean runs, while the stage still carries the source's ignore rules: an
    //  untracked file the source ignores is the person's, and a rule the new commit removed must not turn it into one the
    //  clean removes.
    let carried: Vec<Vec<u8>> = listed_ignored(context, target, stage)?;
    // The `clean` carries no `--quiet`, and what it says ("Removing notes.txt" for each untracked file of the stage) is about
    //  the stage, which the person's command never names, so Git's words are kept in the journal and not printed. It is run
    //  captured, like the proof that follows, in the same stage under the same Git directory.
    let clean_request: GitRequest = GitRequest {
        cwd: stage.to_path_buf(),
        arguments: ["clean", "--force", "-d"]
            .iter()
            .map(OsString::from)
            .collect(),
        without_prefix: true,
        environment: environment.clone(),
        ..GitRequest::default()
    };
    let clean = match run_git(context, &clean_request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("`git clean --force -d` could not start ({error})")),
    };
    if !clean.succeeded() {
        return Err(format!(
            "`git clean --force -d` ended with {:?}",
            clean.code
        ));
    }
    record_words(&["clean", "--force", "-d"], &clean);
    let clean_ms: u128 = began.elapsed().as_millis();
    // The reset rewrites the paths whose files its own stat check can see changed. A file an edit of the same size, made in
    //  the same second as the entry it was recorded against, leaves looking unchanged, so it is removed here and the reset
    //  writes the commit's bytes into it.
    let removed: usize = remove_the_files_the_reset_cannot_see(source, target, stage, conversion)?;
    let reset = run_step(
        run_git_step,
        arguments_in_stage(
            stage,
            &["read-tree", "--reset", "-u", target.commit.as_str()],
        )
        .as_slice(),
        environment.as_slice(),
    )?;
    if !reset.succeeded() {
        return Err(format!(
            "`git read-tree --reset -u` ended with {:?}",
            reset.code
        ));
    }
    let reset_ms: u128 = began.elapsed().as_millis();
    prove_restored(context, target, stage, carried.as_slice())?;
    crate::diagnostic_log::debug(
        TAG,
        format!(
            "restored {} to {}, carried {} ignored paths, replaced {} files the reset cannot see and proved it: clean done at {clean_ms} ms, reset at {reset_ms} ms, proof at {} ms",
            stage.display(),
            target.commit,
            carried.len(),
            removed,
            began.elapsed().as_millis()
        )
        .as_str(),
    );
    return Ok(());
}

/// What:
///  The paths the source has changed since its last commit,
/// when there are any.
/// Why:
///   A snapshot carries the source's uncommitted state,
/// so the person is told which of it was left out of the new worktree,
/// and the record of what was left out is written only for a source that had changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// source.dirty?.hasChanges() ? source.dirty : undefined
/// ```
pub fn changes_left_out(source: &SnapshotSource) -> Option<&ChangedPaths> {
    return match source.dirty.as_ref() {
        Some(found) if found.has_changes() => Some(found),
        _ => None,
    };
}

/// Dirty restore controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_dirty_restore_tests.rs"]
mod tests;

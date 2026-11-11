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

use crate::fs_beneath::{open_root, plain_components, remove_at};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::inner_git_words::record_words;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::worktree_clean::{ContentTarget, paths_whose_content_differs, stored_stats};
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_copy::GitRunner;
use crate::worktree_ladder_target::{Target, checkout_environment, run_step};
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::{ChangedPaths, SnapshotSource};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeDirtyRestore";

/// What:
///  The line that tells the person a snapshot of a checkout with changes left the changes out.
/// Why:
///   The new worktree matches a clean `git worktree add`,
/// which the person may not expect from a copy of a dirty checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: the uncommitted changes of the main checkout (${count} paths) were left out of the new worktree.\n`
/// ```
pub fn left_out_note(changed: &ChangedPaths) -> String {
    let paths: &str = if changed.count == 1 { "path" } else { "paths" };
    return format!("monochromatic-git-policy-cli: the uncommitted changes of the main checkout ({} {paths}) were left out of the new worktree, which holds what Git's own checkout holds.\n", changed.count);
}

/// What:
///  Record in the journal,
/// at the ordinary level,
/// that a snapshot left the uncommitted changes of the main checkout out of the
///  new worktree,
/// with the destination and the number of paths as fields.
/// Why:
///   The line on standard error is gone when the command ends;
/// the journal keeps what the add did with a dirty checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journal.info(note, { CLI_GIT_WORKTREE: destination, CLI_GIT_DIRTY_PATHS: count });
/// ```
pub fn record_left_out(destination: &Path, changed: &ChangedPaths) {
    let note: String = left_out_note(changed);
    let fields: [(&str, String); 3] = [
        ("CLI_GIT_WORKTREE", destination.to_string_lossy().into_owned()),
        ("CLI_GIT_DIRTY_PATHS", changed.count.to_string()),
        ("CLI_GIT_DIRTY_FIRST", changed.first.clone().unwrap_or_default()),
    ];
    crate::client_journal::record_entry(crate::journald_native::Priority::Info, note.trim_end(), &fields);
}

/// What:
///  Whether a byte is the NUL that ends each name in Git's `-z` output.
/// Why:
///   The untracked and the ignored names are split on it,
/// and a path may hold any other byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The words of the query that lists the stage's ignored paths under the stage's own rules,
/// which are the source's until the
///  reset brings the new commit's.
/// Why:
///   `--directory` collapses a whole ignored directory into one name,
/// so a carried `node_modules` costs one
///  entry rather than every file in it,
/// and every name ends with `/` when it is a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IGNORED_WORDS = ['ls-files', '--others', '--ignored', '--exclude-standard', '--directory', '--no-empty-directory', '-z']
/// ```
const IGNORED_WORDS: [&str; 7] = ["ls-files", "--others", "--ignored", "--exclude-standard", "--directory", "--no-empty-directory", "-z"];

/// What:
///  Whether a name split out of `-z` output holds anything.
/// Why:
///   The output ends with a NUL,
/// so the split leaves an empty last piece that names no path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.length > 0
/// ```
fn is_nonempty_name(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// What:
///  The words of a Git run in the stage:
/// `-C <stage>` and then `words`.
/// Why:
///   Every run of the restore names the stage directory itself,
/// so none can run in the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['-C', stage, ...words]
/// ```
fn arguments_in_stage(stage: &Path, words: &[&str]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = vec![OsString::from("-C"), stage.as_os_str().to_os_string()];
    arguments.extend(words.iter().copied().map(OsString::from));
    return arguments;
}

/// What:
///  Run one captured Git read in the stage under the new worktree's Git directory.
/// Why:
///   The proof reads the stage the way the restore wrote it,
/// never the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, stdout } = await runIn(stage, admin, ['diff-files', '--quiet']);
/// ```
fn ask(context: &GitContext, target: &Target, work_tree: &Path, arguments: &[&str]) -> Result<(Option<i32>, Vec<u8>), String> {
    let request: GitRequest = GitRequest {
        cwd: work_tree.to_path_buf(),
        arguments: arguments.iter().copied().map(OsString::from).collect(),
        without_prefix: true,
        environment: checkout_environment(target.admin.as_path(), work_tree),
        ..GitRequest::default()
    };
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("`git {}` could not start ({error})", arguments.join(" "))),
    };
    return Ok((output.code, output.stdout));
}

/// What:
///  Whether a name Git lists as untracked is one the restore carried through:
/// the same name,
/// or a file below a
///  carried directory.
/// Why:
///   A carried directory is one name ending in `/`,
/// while a path inside it is listed whole when a tracked file
///  of the new tree also lives there,
/// so the prefix decides for the rest of one directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// carried.some((kept) => name.equals(kept) || (kept.endsWith('/') && name.startsWith(kept)))
/// ```
fn is_carried(name: &[u8], carried: &[Vec<u8>]) -> bool {
    for kept in carried {
        if name == kept.as_slice() || (kept.ends_with(b"/") && name.starts_with(kept.as_slice())) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a name Git lists as untracked is a whole directory that holds a path the restore carried.
/// Why:
///   `--directory` collapses a directory whose every path is untracked into that one name,
/// so
///  `cache/private/token` carried by the restore is reported as `cache/`;
/// the name matches no carried
///  path and its content decides,
/// which needs the same query without the collapse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.endsWith('/') && carried.some((kept) => kept.startsWith(name))
/// ```
fn holds_carried(name: &[u8], carried: &[Vec<u8>]) -> bool {
    if !name.ends_with(b"/") {
        return false;
    }
    for kept in carried {
        if kept.starts_with(name) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a name lies below one of the directories Git reported as one collapsed name.
/// Why:
///   Only the paths inside such a directory are judged when the collapse is opened;
/// every other name was
///  carried already.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// directories.some((directory) => name.startsWith(directory))
/// ```
fn lies_below(name: &[u8], directories: &[Vec<u8>]) -> bool {
    for directory in directories {
        if name.starts_with(directory.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Prove that the stage equals what Git's checkout of the target commit makes,
/// plus the ignored paths the
///  restore carried:
/// the index equals the commit's tree,
/// Git shows no untracked file besides those,
/// and no tracked file
///  differs from the index.
/// Why:
///   The restore trusts Git's reset and clean;
/// the proof asks Git again from the other side,
/// so a state either of them left
///  (a path the reset skipped,
/// a directory the clean could not remove) ends the rung instead of reaching the person.
/// A carried path is what the source ignored and the new commit does not,
/// which the person keeps and the
///  ignored-state copy hands over on every other filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await proveRestored({ context, target, workTree, carried }); // throws the first thing that is not as Git's checkout leaves it
/// ```
pub fn prove_restored(context: &GitContext, target: &Target, work_tree: &Path, carried: &[Vec<u8>]) -> Result<(), String> {
    let cached: Option<i32> = ask(context, target, work_tree, &["diff-index", "--cached", "--quiet", target.commit.as_str()])?.0;
    if cached != Some(0) {
        return Err(String::from("the restored index does not equal the tree of the new HEAD"));
    }
    let (listed, untracked) = ask(context, target, work_tree, &["ls-files", "--others", "--exclude-standard", "--directory", "--no-empty-directory", "-z"])?;
    if listed != Some(0) {
        return Err(String::from("Git could not list the untracked files of the restored worktree"));
    }
    let names: Vec<Vec<u8>> = untracked.split(is_nul_byte).filter(is_nonempty_name).map(<[u8]>::to_vec).collect();
    // The first untracked name the restore did not carry; the loop stops there, as `find` did.
    let mut first_untracked: Option<Vec<u8>> = None;
    let mut collapsed: Vec<Vec<u8>> = Vec::new();
    for name in &names {
        if is_carried(name.as_slice(), carried) {
            continue;
        }
        if holds_carried(name.as_slice(), carried) {
            collapsed.push(name.clone());
            continue;
        }
        first_untracked = Some(name.clone());
        break;
    }
    // A whole directory that holds carried paths is decided by its content, which the collapsed listing does not carry:
    //  the same query without `--directory` names every path below it.
    if first_untracked.is_none() && !collapsed.is_empty() {
        let (expanded_code, expanded) = ask(context, target, work_tree, &["ls-files", "--others", "--exclude-standard", "-z"])?;
        if expanded_code != Some(0) {
            return Err(String::from("Git could not list the untracked files below the carried directories of the restored worktree"));
        }
        // Inside a reported directory only its own paths are judged; every other name was carried already.
        for name in expanded.split(is_nul_byte).filter(is_nonempty_name) {
            if lies_below(name, collapsed.as_slice()) && !is_carried(name, carried) {
                first_untracked = Some(name.to_vec());
                break;
            }
        }
    }
    if let Some(name) = first_untracked {
        return Err(format!("{} is still untracked in the restored worktree", crate::worktree_snapshot_source::plain(String::from_utf8_lossy(name.as_slice()).as_ref())));
    }
    let files: Option<i32> = ask(context, target, work_tree, &["diff-files", "--quiet"])?.0;
    if files != Some(0) {
        return Err(String::from("a tracked file of the restored worktree differs from its index entry"));
    }
    return Ok(());
}

/// What:
///  The ignored paths of the stage,
/// listed under the rules the stage has now,
/// as the boundaries of a whole ignored
///  directory collapse them.
/// Why:
///   The restore carries them through the clean and the reset,
/// so it must know them before either
///  runs;
/// a query that fails ends the rung instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carried = listedIgnored({ context, target, stage }); // throws the reason
/// ```
fn listed_ignored(context: &GitContext, target: &Target, stage: &Path) -> Result<Vec<Vec<u8>>, String> {
    let (listed, names) = ask(context, target, stage, IGNORED_WORDS.as_slice())?;
    if listed != Some(0) {
        return Err(String::from("Git could not list the ignored files of the snapshot"));
    }
    return Ok(names.split(is_nul_byte).filter(is_nonempty_name).map(<[u8]>::to_vec).collect());
}

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
fn remove_the_files_the_reset_cannot_see(source: &SnapshotSource, target: &Target, stage: &Path, conversion: &ConversionEnvironment) -> Result<usize, String> {
    let index_path: PathBuf = target.admin.join("index");
    let index: Vec<u8> = std::fs::read(index_path.as_path()).map_err(index_unreadable)?;
    let index_seconds: u32 = std::fs::metadata(index_path.as_path()).map_err(index_unexamined)?.mtime() as u32;
    let hash_length: usize = source.clean.hash_length;
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let (Some(entries), Some(indexed)) = (stored_stats(index.as_slice(), hash_length), parse_index(index.as_slice(), format)) else {
        return Err(String::from("the index of the new worktree has a layout the reader does not know"));
    };
    let content: ContentTarget<'_> = ContentTarget { root: stage, git_dir: target.admin.as_path(), common_dir: source.common_dir.as_path(), environment: conversion, format };
    let hidden: Vec<Vec<u8>> = paths_whose_content_differs(&content, entries.as_slice(), indexed.as_slice(), index_seconds).map_err(the_reason_of)?;
    if hidden.is_empty() {
        return Ok(0);
    }
    let held: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    let Ok(mut chain) = DirectoryChain::new(&held) else { return Err(String::from("the stage cannot be examined")) };
    for path in &hidden {
        let shown: String = String::from_utf8_lossy(path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(path.as_slice()))) else { return Err(format!("{shown} is not a plain path")) };
        let Some((name, parents)) = names.split_last() else { return Err(format!("{shown} is not a plain path")) };
        let Some(directory) = chain.directory(parents).ok() else { return Err(format!("{shown} cannot be reached without following a link")) };
        match remove_at(directory, name.as_os_str(), false) {
            Ok(()) => {}
            // A file that is already gone is one the reset writes, so nothing is lost when it cannot be removed twice.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{shown} could not be removed from the snapshot ({error})")),
        }
    }
    crate::diagnostic_log::debug(TAG, format!("removed {} files the reset cannot see: the commit's bytes replace them", hidden.len()).as_str());
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
pub fn restore_dirty(source: &SnapshotSource, target: &Target, stage: &Path, run_git_step: &mut dyn GitRunner, context: &GitContext, conversion: &ConversionEnvironment) -> Result<(), String> {
    if stage == source.root.as_path() || target.admin == source.git_dir || target.admin == source.common_dir {
        return Err(String::from("the restore would run in the source's own directory"));
    }
    let began: std::time::Instant = std::time::Instant::now();
    let environment = checkout_environment(target.admin.as_path(), stage);
    // The ignored paths are listed, and the clean runs, while the stage still carries the source's ignore rules: an
    //  untracked file the source ignores is the person's, and a rule the new commit removed must not turn it into one the
    //  clean removes.
    let carried: Vec<Vec<u8>> = listed_ignored(context, target, stage)?;
    // The `clean` carries no `--quiet`, and what it says ("Removing notes.txt" for each untracked file of the stage) is about
    //  the stage, which the person's command never names, so Git's words are kept in the journal and not printed. It is run
    //  captured, like the proof that follows, in the same stage under the same Git directory.
    let clean_request: GitRequest = GitRequest {
        cwd: stage.to_path_buf(),
        arguments: ["clean", "--force", "-d"].iter().map(OsString::from).collect(),
        without_prefix: true,
        environment: environment.clone(),
        ..GitRequest::default()
    };
    let clean = match run_git(context, &clean_request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("`git clean --force -d` could not start ({error})")),
    };
    if !clean.succeeded() {
        return Err(format!("`git clean --force -d` ended with {:?}", clean.code));
    }
    record_words(&["clean", "--force", "-d"], &clean);
    let clean_ms: u128 = began.elapsed().as_millis();
    // The reset rewrites the paths whose files its own stat check can see changed. A file an edit of the same size, made in
    //  the same second as the entry it was recorded against, leaves looking unchanged, so it is removed here and the reset
    //  writes the commit's bytes into it.
    let removed: usize = remove_the_files_the_reset_cannot_see(source, target, stage, conversion)?;
    let reset = run_step(run_git_step, arguments_in_stage(stage, &["read-tree", "--reset", "-u", target.commit.as_str()]).as_slice(), environment.as_slice())?;
    if !reset.succeeded() {
        return Err(format!("`git read-tree --reset -u` ended with {:?}", reset.code));
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

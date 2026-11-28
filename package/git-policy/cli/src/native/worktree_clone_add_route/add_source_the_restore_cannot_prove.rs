//! What:
//!  The reasons a main checkout cannot be the source of the clone this route fills,
//!  because the fill's own restore would refuse it after the clone is made:
//!  a Git repository of its own that `git clean --force -d` keeps,
//!  and a directory where the index records a file.
//! Why:
//!  The fill restores the mounted clone through the snapshot rung's own restore
//!  (`add_fill.rs`,
//!  `worktree_dirty_restore.rs`),
//!  and both shapes end that restore after the snapshot has been taken:
//!  the clean keeps a repository of its own and the proof names it as untracked,
//!  and the removal before the reset cannot unlink a directory with the tools it
//!  holds.
//!  Nothing of this release settles a failed restore at a slot:
//!  the claim,
//!  the locked registration,
//!  the mounted clone and the origin snapshot stay,
//!  and the settle that destroys them is plan step 14's
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  and `SPEC.md`,
//!  "What this release does not yet do").
//!  So the preflight asks these questions before the claim,
//!  and an add whose source holds one takes the ladder,
//!  whose failed restore destroys its own stage and falls back to Git's own
//!  checkout.
//!  A clean source is asked nothing:
//!  the fill restores nothing then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reasons = reasonsTheRestoreCannotProve({ context, source }); // []: the fill can hold this source
//! ```

use crate::command_clean_nested_repositories::repositories_beneath;
use crate::fs_beneath::{
    entry_metadata_at,
    open_root,
    plain_components,
};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::worktree_clean::stored_stats;
use crate::worktree_clean_untracked_scan::UNTRACKED_LISTING_WORDS;
use crate::worktree_snapshot_source::{
    SnapshotSource,
    plain,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "addSourceTheRestoreCannotProve";

/// What:
///  Every reason the fill's restore could not prove a copy of this source,
///  or why one of the questions could not be asked.
/// Why:
///  The preflight asks this before the claim,
///  and a reason here makes the whole route decline:
///  the ladder serves the add and settles a restore that fails there.
///  An answer the wrapper could not get is an error rather than a reason,
///  because a question that was never answered proves nothing about the source,
///  and the caller declines on either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reasonsTheRestoreCannotProve({ context, source }): string[]; // throws when a question cannot be asked
/// ```
pub fn reasons_the_restore_cannot_prove(
    (context, source): (&GitContext, &SnapshotSource)
) -> Result<Vec<String>, String> {
    let mut reasons: Vec<String> = Vec::new();
    if source.dirty.is_none() {
        return Ok(reasons);
    }
    if let Some(path) = a_directory_where_the_index_records_a_file(source)? {
        reasons.push(format!("{} is a directory where the index records a file, so the restored worktree could not equal Git's own checkout", shown(path.as_slice())));
    }
    let mut kept: Vec<String> = repositories_the_clean_keeps_untracked((context, source))?;
    reasons.append(&mut kept);
    for reason in &reasons {
        crate::diagnostic_log::debug(TAG, reason.as_str());
    }
    return Ok(reasons);
}

/// What:
///  The paths of the source whose names are printed in a refusal,
///  with every control character replaced.
/// Why:
///   A path in a sentence a person reads must not carry an escape sequence or
///   start a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = (path: Buffer) => plain(path.toString());
/// ```
fn shown(path: &[u8]) -> String {
    return plain(String::from_utf8_lossy(path).as_ref());
}

/// What:
///  The first tracked path that holds a directory in the source,
///  if any.
/// Why:
///   The restore removes the files whose bytes its reset cannot see before it
///   resets,
///   and it removes each of them as a file or a link.
///   A directory at a tracked path cannot be removed that way,
///   so the restore ends,
///   and Git's own checkout would have replaced it with the commit's file.
///  The entries are the ones the source check proved,
///  so nothing is read twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aDirectoryWhereTheIndexRecordsAFile(source): string | undefined;
/// ```
fn a_directory_where_the_index_records_a_file(
    source: &SnapshotSource
) -> Result<Option<Vec<u8>>, String> {
    let Some(entries) = stored_stats(
        source.clean.index_bytes.as_slice(),
        source.clean.hash_length,
    ) else {
        return Err(String::from(
            "the index of the main checkout has a layout the reader does not know",
        ));
    };
    let held: std::fs::File = open_root(source.root.as_path()).map_err(|error| {
        return format!("the main checkout cannot be opened without following a link ({error})");
    })?;
    let Ok(mut chain) = DirectoryChain::new(&held) else {
        return Err(String::from("the main checkout cannot be examined"));
    };
    for entry in &entries {
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice())))
        else {
            continue;
        };
        let Some((name, parents)) = names.split_last() else {
            continue;
        };
        let Some(directory) = chain.directory(parents).ok() else {
            continue;
        };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else {
            continue;
        };
        if metadata.is_dir() {
            return Ok(Some(entry.path.clone()));
        }
    }
    return Ok(None);
}

/// What:
///  The reasons the copy's clean would leave an untracked path behind,
///  one per Git repository of the source's own that the proof would then name.
/// Why:
///   `git clean --force -d` keeps every directory that holds a `.git` entry of
///   its own (`builtin/clean.c`,
///  `REMOVE_DIR_KEEP_NESTED_GIT`),
///   and the proof lists what it left as still untracked.
///   A directory Git lists as untracked is not one the restore carries:
///  the two listings are Git's own and a directory is either untracked or
///  ignored,
///  so the proof names every such directory and the route declines it.
///  Git answers the question with one run,
///  the same listing the restore's proof reads,
///  and a repository under an ignored directory is not in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoriesTheCleanKeepsUntracked(context, source): string[]; // throws when Git cannot answer
/// ```
fn repositories_the_clean_keeps_untracked(
    (context, source): (&GitContext, &SnapshotSource)
) -> Result<Vec<String>, String> {
    let untracked: Vec<Vec<u8>> =
        listed_names((context, source.root.as_path(), UNTRACKED_LISTING_WORDS))?;
    let mut reasons: Vec<String> = Vec::new();
    for name in untracked {
        if !name.ends_with(b"/") {
            continue;
        }
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(name.as_slice()))) else {
            continue;
        };
        if repositories_beneath(source.root.as_path(), names.as_slice())?.is_empty() {
            continue;
        }
        reasons.push(format!("{} holds a Git repository of its own, which `git clean --force -d` keeps, and no ignore rule covers the directory, so the restored worktree would still name it as untracked", shown(name.as_slice())));
    }
    return Ok(reasons);
}

/// What:
///  The names one listing of the source answers,
///  or why the run gave none.
/// Why:
///   A listing that fails proves nothing about the source,
///  so it is an error naming the run and what Git said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listedNames({ context, root, words }): Buffer[]; // throws
/// ```
fn listed_names(
    (context, root, words): (&GitContext, &Path, &[&str])
) -> Result<Vec<Vec<u8>>, String> {
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments: words.iter().copied().map(OsString::from).collect(),
        without_prefix: true,
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(context, &request)
        .map_err(|error| return format!("`git {}` could not start ({error})", words.join(" ")))?;
    if output.code != Some(0) {
        return Err(format!(
            "`git {}` ended with {:?} ({})",
            words.join(" "),
            output.code,
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    return Ok(output
        .stdout
        .split(is_nul_byte)
        .filter(is_nonempty)
        .map(<[u8]>::to_vec)
        .collect());
}

/// Whether a byte is the NUL that ends each name in Git's `-z` output.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether a name split out of `-z` output holds anything.
fn is_nonempty(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// Source shapes the restore cannot prove stay out of the release executable.
#[cfg(test)]
#[path = "add_source_the_restore_cannot_prove_tests.rs"]
mod tests;

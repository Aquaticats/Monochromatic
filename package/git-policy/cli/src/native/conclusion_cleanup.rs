//! What:
//!  Native conclusion cleanup reproduced in the owning worktree's Git directory after a
//!       merge,
//!  cherry-pick or revert conclusion lands,
//!  and the names of that state.
//! Why:
//!  Native `git commit` removes `MERGE_HEAD` and its siblings in the shadow;
//!  the real
//!      worktree must end the same way,
//!  but an entry is removed only while it still holds the
//!      bytes or value copied at preparation,
//!  because a changed entry belongs to a later command
//!      (`src/shadow-repository/shadow-conclusion-cleanup.ts`,
//!  `-files.ts`,
//!  `-names.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await reproduceConclusionCleanup({ gitPath, cwd, gitDir, shadowPath, transactionDirectory, refFormat });
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Tree removal and private files.
use super::private_storage::remove_tree;
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// Exact reads and existence probes.
use super::recovery_files::{path_present, read_optional, remove_file_if_present};
/// Shadow commands.
use super::shadow_git::shadow_request;
/// Running real Git.
use super::transaction_git::{GitContext, GitRequest, run_git, run_git_checked};
/// The ref storage backend.
use super::transaction_journal::RefFormat;
/// `BTreeMap` is a sorted map.
use std::collections::BTreeMap;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Conclusion state files copied into the shadow at preparation.
pub const CONCLUSION_STATE_FILES: [&str; 8] = [
    "MERGE_HEAD",
    "MERGE_MSG",
    "MERGE_MODE",
    "SQUASH_MSG",
    "AUTO_MERGE",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "MERGE_RR",
];

/// Conclusion state directory copied into the shadow at preparation.
pub const SEQUENCER_DIRECTORY: &str = "sequencer";

/// Pseudorefs a reftable repository keeps in its ref store rather than as files.
pub const STORE_HELD_PSEUDOREFS: [&str; 3] = ["AUTO_MERGE", "CHERRY_PICK_HEAD", "REVERT_HEAD"];

/// Entries landing removes from the owning worktree when native Git removed them in the shadow.
pub const REMOVED_CONCLUSION_FILES: [&str; 7] = [
    "AUTO_MERGE",
    "MERGE_HEAD",
    "MERGE_MODE",
    "MERGE_MSG",
    "SQUASH_MSG",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
];

/// Private directory of the transaction holding the preparation-time copies.
pub const CONCLUSION_COPY_DIRECTORY: &str = "conclusion";

/// Record of the store-held pseudoref values copied at preparation.
pub const STORE_RECORD_FILENAME: &str = "store-held.json";

/// What:
///  Whether an entry lives in the ref store under this backend.
/// Why:
///   `isStoreHeld`:
///  reftable keeps three pseudorefs in the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isStoreHeld({ refFormat, name })
/// ```
pub fn is_store_held(format: RefFormat, name: &str) -> bool {
    return format == RefFormat::Reftable && STORE_HELD_PSEUDOREFS.contains(&name);
}

/// What:
///  Resolve a pseudoref with `git rev-parse --verify --quiet`,
///  or nothing when absent.
/// Why:
///   `resolvePseudoref`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolvePseudoref({ gitPath, cwd, name }): Promise<string | typeof ENTRY_ABSENT>;
/// ```
pub fn resolve_pseudoref(context: &GitContext, request: GitRequest) -> Option<String> {
    let output: super::transaction_git::GitOutput = run_git(context, &request).ok()?;
    if !output.succeeded() {
        return None;
    }
    let text: &str = std::str::from_utf8(output.stdout.as_slice()).ok()?;
    return Some(String::from(super::js_text::trim_javascript(text)));
}

/// What:
///  Every regular file under a directory,
///  by path relative to it.
/// Why:
///   `readTree`:
///  the sequencer copy and the real sequencer are compared whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readTree(root: string): Promise<ReadonlyMap<string, Uint8Array>>;
/// ```
fn read_tree(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>, RecoveryError> {
    // `mut` allows collecting files while walking.
    let mut files: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
    if !path_present(root)? {
        return Ok(files);
    }
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let listing: std::fs::ReadDir = match std::fs::read_dir(&directory) {
            Ok(found) => found,
            Err(error) => return Err(io_failure("listing", directory.as_path(), &error)),
        };
        for item in listing {
            let entry: std::fs::DirEntry = match item {
                Ok(found) => found,
                Err(error) => return Err(io_failure("listing", directory.as_path(), &error)),
            };
            let kind: std::fs::FileType = match entry.file_type() {
                Ok(found) => found,
                Err(error) => return Err(io_failure("inspecting", entry.path().as_path(), &error)),
            };
            let path: PathBuf = entry.path();
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() {
                let bytes: Vec<u8> = match std::fs::read(&path) {
                    Ok(read) => read,
                    Err(error) => return Err(io_failure("reading", path.as_path(), &error)),
                };
                let relative: PathBuf = path
                    .strip_prefix(root)
                    .unwrap_or(path.as_path())
                    .to_path_buf();
                files.insert(relative, bytes);
            }
        }
    }
    return Ok(files);
}

/// What:
///  Whether two directory trees hold the same regular files with the same bytes,
///  and are
///       not empty.
/// Why:
///   `sameTree`:
///  the sequencer is removed only while it is exactly the copied state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sameTree({ left, right }): Promise<boolean>;
/// ```
pub fn same_tree(left: &Path, right: &Path) -> Result<bool, RecoveryError> {
    let first: BTreeMap<PathBuf, Vec<u8>> = read_tree(left)?;
    let second: BTreeMap<PathBuf, Vec<u8>> = read_tree(right)?;
    return Ok(!first.is_empty() && first == second);
}

/// What:
///  The store-held values copied at preparation,
///  from `store-held.json`.
/// Why:
///   A value that is not text is no copy;
///  the record itself must be valid UTF-8 JSON.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.parse(decode(await readFile(join(copies, STORE_RECORD_FILENAME))))
/// ```
fn read_store_record(copies: &Path) -> Result<serde_json::Value, RecoveryError> {
    let path: PathBuf = copies.join(STORE_RECORD_FILENAME);
    let bytes: Vec<u8> = match std::fs::read(&path) {
        Ok(read) => read,
        Err(error) => return Err(io_failure("reading", path.as_path(), &error)),
    };
    let Some(text) = super::json_record::decode_fatal(bytes.as_slice()) else {
        return Err(RecoveryError(format!("{} is not UTF-8.", path.display())));
    };
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(value) => return Ok(value),
        Err(_) => return Err(RecoveryError(format!("{} is not JSON.", path.display()))),
    }
}

/// What:
///  Remove one store-held pseudoref from the owning worktree when the shadow no longer
///       holds it and the worktree still holds the copied value.
/// Why:
///   A reftable repository deletes it with `git update-ref -d <name> <copied value>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (inShadow.exitCode !== 0 && inReal === copied) await runTransactionGit({ args: ['update-ref', '-d', name, copied] });
/// ```
fn clean_store_held(
    context: &GitContext,
    cwd: &Path,
    shadow: &Path,
    name: &str,
    copied: &str,
) -> Result<(), RecoveryError> {
    let in_shadow: bool = match run_git(
        context,
        &shadow_request(shadow, shadow, &["rev-parse", "--verify", "--quiet", name]),
    ) {
        Ok(output) => output.succeeded(),
        Err(_) => false,
    };
    let in_real: Option<String> = resolve_pseudoref(
        context,
        GitRequest::new(cwd, &["rev-parse", "--verify", "--quiet", name]),
    );
    if !in_shadow
        && in_real.as_deref() == Some(copied)
        && let Err(failure) = run_git_checked(
            context,
            &GitRequest::new(cwd, &["update-ref", "-d", name, copied]),
        )
    {
        return Err(RecoveryError(failure.0));
    }
    return Ok(());
}

/// What:
///  Reproduce native conclusion cleanup in the owning worktree's Git directory.
/// Why:
///   `reproduceConclusionCleanup`:
///  removed entries,
///  `MERGE_RR` copied back,
///  `ORIG_HEAD`
///       kept,
///  and `sequencer/` removed only when native Git removed the shadow copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reproduceConclusionCleanup({ gitPath, cwd, gitDir, shadowPath, transactionDirectory, refFormat }): Promise<void>;
/// ```
pub fn reproduce_conclusion_cleanup(
    context: &GitContext,
    cwd: &Path,
    git_dir: &Path,
    shadow: &Path,
    transaction_directory: &Path,
    format: RefFormat,
) -> Result<(), RecoveryError> {
    let copies: PathBuf = transaction_directory.join(CONCLUSION_COPY_DIRECTORY);
    if !path_present(copies.as_path())? {
        debug(
            "reproduceConclusionCleanup",
            format!(
                "no conclusion state was copied: {}",
                transaction_directory.display()
            )
            .as_str(),
        );
        return Ok(());
    }
    let store: serde_json::Value = read_store_record(copies.as_path())?;
    for name in REMOVED_CONCLUSION_FILES {
        if is_store_held(format, name) {
            if let Some(copied) = store.get(name).and_then(serde_json::Value::as_str) {
                clean_store_held(context, cwd, shadow, name, copied)?;
            }
            continue;
        }
        let copy: Option<Vec<u8>> = read_optional(copies.join(name).as_path())?;
        let in_shadow: Option<Vec<u8>> = read_optional(shadow.join(name).as_path())?;
        let real: Option<Vec<u8>> = read_optional(git_dir.join(name).as_path())?;
        if copy.is_some() && in_shadow.is_none() && copy == real {
            remove_file_if_present(git_dir.join(name).as_path())?;
            debug(
                "reproduceConclusionCleanup",
                format!("removed conclusion entry {name} as native Git did").as_str(),
            );
        }
    }
    if let Some(merge_rr) = read_optional(shadow.join("MERGE_RR").as_path())? {
        let temporary: PathBuf = git_dir.join(format!("MERGE_RR.cli-git-{}", std::process::id()));
        if let Err(error) = std::fs::write(&temporary, merge_rr.as_slice()) {
            return Err(io_failure("writing", temporary.as_path(), &error));
        }
        let destination: PathBuf = git_dir.join("MERGE_RR");
        if let Err(error) = std::fs::rename(&temporary, &destination) {
            return Err(io_failure("replacing", destination.as_path(), &error));
        }
    }
    let sequencer_copy: PathBuf = copies.join(SEQUENCER_DIRECTORY);
    let removed_in_shadow: bool = path_present(sequencer_copy.as_path())?
        && !path_present(shadow.join(SEQUENCER_DIRECTORY).as_path())?;
    let real_sequencer: PathBuf = git_dir.join(SEQUENCER_DIRECTORY);
    if removed_in_shadow && same_tree(sequencer_copy.as_path(), real_sequencer.as_path())? {
        if let Err(error) = remove_tree(real_sequencer.as_path()) {
            return Err(io_failure("removing", real_sequencer.as_path(), &error));
        }
        debug(
            "reproduceConclusionCleanup",
            "removed sequencer state as native Git did after the last pick",
        );
    }
    return Ok(());
}

/// Cleanup controls stay out of the release executable.
#[cfg(test)]
#[path = "conclusion_cleanup_tests.rs"]
mod tests;

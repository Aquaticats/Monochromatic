//! What:
//!  The merge, cherry-pick and revert conclusion state copied into the shadow at preparation,
//!       and into the transaction's comparison copies.
//! Why:
//!  Native `git commit` in the shadow reads the conclusion state the real worktree holds at invocation.
//!  Landing later compares the real entries with these copies, so an entry changed by another command
//!  after invocation is kept (`conclusion_cleanup.rs` holds the landing side).
//!  Pseudorefs of a reftable repository live in the ref store and are copied with `update-ref`
//!  (`src/shadow-repository/shadow-conclusion-state.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyConclusionState({ gitPath, cwd, gitDir, shadowPath, transactionDirectory, refFormat });
//! ```

/// Names and helpers the landing side already uses.
use super::conclusion_cleanup::{
    CONCLUSION_COPY_DIRECTORY, CONCLUSION_STATE_FILES, SEQUENCER_DIRECTORY, STORE_RECORD_FILENAME,
    is_store_held, resolve_pseudoref,
};
/// Private file writes and directory creation.
use super::private_storage::{create_private_directory, write_private_file};
/// Optional reads that treat a missing entry as absent.
use super::recovery_files::{path_present, read_optional};
/// JSON object writer for the store-held record.
use super::json_record::ObjectWriter;
/// The shadow ref helper and the process context.
use super::shadow_git::shadow_request;
use super::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
/// Ref storage backend.
use super::transaction_journal::RefFormat;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Copies a directory tree, keeping links as links.
/// Why:
///   `cp(..., { recursive: true })` copies the sequencer directory into the shadow and the copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cp(source, destination, { recursive: true });
/// ```
fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    create_private_directory(destination)?;
    for entry_result in std::fs::read_dir(source)? {
        let entry: std::fs::DirEntry = entry_result?;
        let target: PathBuf = destination.join(entry.file_name());
        let kind: std::fs::FileType = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(entry.path().as_path(), target.as_path())?;
        } else if kind.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::fs::read_link(entry.path())?, &target)?;
            // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
            #[cfg(not(unix))]
            std::fs::copy(entry.path(), &target).map(|_| ())?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    return Ok(());
}

/// What:
///  Reads one pseudoref's value in the owning worktree, if it resolves.
/// Why:
///   A reftable pseudoref has no file to copy, so its value is copied through the ref store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolvePseudoref({ gitPath, cwd, name });
/// ```
fn owning_pseudoref(context: &GitContext, cwd: &Path, name: &str) -> Option<String> {
    let mut request: GitRequest = GitRequest::new(cwd, &["rev-parse", "--verify", "--quiet", name]);
    request.without_prefix = true;
    return resolve_pseudoref(context, request);
}

/// What:
///  Copies every present conclusion entry of the owning worktree into the shadow and the transaction.
/// Why:
///   The shadow copy is what native commit reads; the transaction copy is what landing compares against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyConclusionState({ gitPath, cwd, gitDir, shadowPath, transactionDirectory, refFormat });
/// ```
pub fn copy_conclusion_state(
    context: &GitContext,
    cwd: &Path,
    git_dir: &Path,
    shadow: &Path,
    transaction_directory: &Path,
    ref_format: RefFormat,
) -> Result<(), GitFailure> {
    let io = |step: &str, error: std::io::Error| { return GitFailure(format!("{step} failed: {error}")); };
    let copies: PathBuf = transaction_directory.join(CONCLUSION_COPY_DIRECTORY);
    create_private_directory(copies.as_path()).map_err(|error| { return io("creating the conclusion copies", error); })?;
    // Store-held values in the order of the copied names, as the incumbent's JSON record lists them.
    let mut store_held: ObjectWriter = ObjectWriter::new();
    for name in CONCLUSION_STATE_FILES {
        if is_store_held(ref_format, name) {
            if let Some(value) = owning_pseudoref(context, cwd, name) {
                run_git_checked(
                    context,
                    &shadow_request(shadow, shadow, &["update-ref", name, value.as_str()]),
                )?;
                store_held.string(name, value.as_str());
            }
            continue;
        }
        let Some(bytes) = read_optional(git_dir.join(name).as_path())
            .map_err(|error| { return GitFailure(error.0); })?
        else {
            continue;
        };
        write_private_file(shadow.join(name).as_path(), bytes.as_slice())
            .map_err(|error| { return io("copying a conclusion entry into the shadow", error); })?;
        write_private_file(copies.join(name).as_path(), bytes.as_slice())
            .map_err(|error| { return io("copying a conclusion entry for comparison", error); })?;
    }
    let record: String = store_held.finish();
    write_private_file(
        copies.join(STORE_RECORD_FILENAME).as_path(),
        format!("{record}\n").as_bytes(),
    )
    .map_err(|error| { return io("writing the store-held record", error); })?;
    let sequencer: PathBuf = git_dir.join(SEQUENCER_DIRECTORY);
    if path_present(sequencer.as_path()).map_err(|error| { return GitFailure(error.0); })? {
        copy_tree(sequencer.as_path(), shadow.join(SEQUENCER_DIRECTORY).as_path())
            .map_err(|error| { return io("copying the sequencer into the shadow", error); })?;
        copy_tree(sequencer.as_path(), copies.join(SEQUENCER_DIRECTORY).as_path())
            .map_err(|error| { return io("copying the sequencer for comparison", error); })?;
    }
    return Ok(());
}

/// Conclusion copy controls stay out of the release executable.
#[cfg(test)]
#[path = "conclusion_copy_tests.rs"]
mod tests;

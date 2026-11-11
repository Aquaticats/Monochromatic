//! What:
//!  The merge,
//! cherry-pick and revert conclusion state copied into the shadow at preparation,
//!       and into the transaction's comparison copies.
//! Why:
//!  Native `git commit` in the shadow reads the conclusion state the real worktree holds at invocation.
//!  Landing later compares the real entries with these copies,
//! so an entry changed by another command
//!  after invocation is kept (`conclusion_cleanup.rs` holds the landing side).
//!  Pseudorefs of a reftable repository live in the ref store and are copied with `update-ref`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyConclusionState({ gitPath, cwd, gitDir, shadowPath, transactionDirectory, refFormat });
//! ```

/// Names and helpers the landing side already uses.
use crate::conclusion_cleanup::{
    CONCLUSION_COPY_DIRECTORY, CONCLUSION_STATE_FILES, SEQUENCER_DIRECTORY, STORE_RECORD_FILENAME,
    is_store_held, resolve_pseudoref, store_names_conclusions,
};
/// Private file writes and directory creation.
///  A conclusion copy is written without a flush: nothing recovers it from this directory.
use crate::private_storage::{create_private_directory, write_private_file, write_private_file_unsynced};
/// Optional reads that treat a missing entry as absent.
use crate::recovery_files::{path_present, read_optional};
/// The error of a recovery-file read,
/// whose text becomes a Git failure.
use crate::recovery_error::RecoveryError;
/// JSON object writer for the store-held record.
use crate::json_record::ObjectWriter;
/// The shadow ref helper and the process context.
use crate::shadow_git::shadow_request;
use crate::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
/// Ref storage backend.
use crate::transaction_journal::RefFormat;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Turns the result of one filesystem step into a Git failure that names the step.
///  `std::io::Result<T>` is "the value,
/// or an input/output error".
/// Why:
///   Every step of `copy_conclusion_state` reports its failure as `<step> failed: <error>`,
/// and a named function
///  stands where a closure that built that failure stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ioStep<T>(step: string, attempt: () => T): T; // throws GitFailure(`${step} failed: ${error}`)
/// ```
fn io_step<T>(step: &str, result: std::io::Result<T>) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(format!("{step} failed: {error}"))),
    };
}

/// What:
///  Turns the result of one recovery-file read into a Git failure that carries the recovery error's text.
/// Why:
///   Reads of optional and present entries report their failure the same way,
/// and a named function stands where a
///  closure that unwrapped the error text stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoveryStep<T>(attempt: () => T): T; // throws GitFailure(error.message)
/// ```
fn recovery_step<T>(result: Result<T, RecoveryError>) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(error.0)),
    };
}

/// What:
///  Copies a directory tree,
/// keeping links as links.
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
///  Reads one pseudoref's value in the owning worktree,
/// if it resolves.
/// Why:
///   A reftable pseudoref has no file to copy,
/// so its value is copied through the ref store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolvePseudoref({ gitPath, cwd, name });
/// ```
fn owning_pseudoref(context: &GitContext, cwd: &Path, name: &str) -> Option<String> {
    let mut request: GitRequest = GitRequest::new(cwd, &["rev-parse", "--verify", "--quiet", name]);
    request.without_prefix = true;
    // The value is read in the caller's repository, which a Git directory beside its work tree only names.
    request.repository_selection = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    return resolve_pseudoref(context, request);
}

/// What:
///  Copies every present conclusion entry of the owning worktree into the shadow and the transaction.
/// Why:
///   The shadow copy is what native commit reads;
/// the transaction copy is what landing compares against.
///  A worktree with nothing to conclude gets no copies directory at all:
/// its one `mkdir` was the largest stretch of a build in the records
/// (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "What each part of a commit's private repository is for,
/// and how many commits need it"),
///  and an absent directory is the same answer to the landing as an empty one.
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
    let copies: PathBuf = transaction_directory.join(CONCLUSION_COPY_DIRECTORY);
    // Store-held values in the order of the copied names, as the incumbent's JSON record lists them.
    let mut store_held: ObjectWriter = ObjectWriter::new();
    // The entries that are files, kept until it is known whether the copies directory is needed at all.
    let mut entries: Vec<(&str, Vec<u8>)> = Vec::new();
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
        let Some(bytes) = recovery_step(read_optional(git_dir.join(name).as_path()))? else {
            continue;
        };
        entries.push((name, bytes));
    }
    // The record names the conclusion values this backend's ref store holds. A files backend holds none, so the record
    // would say nothing there and its whole cost is a flush of the disk that every commit then waits for: it is not
    // written, and an absent one is that state (see `store_names_conclusions`).
    let record: Option<String> = if store_names_conclusions(ref_format) || !store_held.is_empty() {
        Some(store_held.finish())
    } else {
        None
    };
    let sequencer: PathBuf = git_dir.join(SEQUENCER_DIRECTORY);
    let sequencer_present: bool = recovery_step(path_present(sequencer.as_path()))?;
    if entries.is_empty() && record.is_none() && !sequencer_present {
        return Ok(());
    }
    io_step("creating the conclusion copies", create_private_directory(copies.as_path()))?;
    // What: The shadow's copies and the comparison copies are written without a flush of the disk.
    // Why:  The shadow's copy is read by this process's own `git commit` immediately after it is written, and the comparison
    //       copy by the landing this same command asks for; the service copies what the cleanup compares into the batch's
    //       journal before the branch moves (`conclusion_state_copy.rs`), so nothing recovers either from this directory. A
    //       copy a power cut never gave back keeps the worktree's own entry instead of removing it, so no content is lost.
    //       The flush was the whole cost of a conclusion commit on this host: eight client fsyncs of these throwaway files,
    //       2 to 53 s over native Git in ten rounds, five of ten over native plus five seconds
    //       (`doc/planning/cli-git-commit-latency-under-many-agents.md`, "A conclusion commit's copies wait for no flush").
    for (name, bytes) in entries {
        io_step(
            "copying a conclusion entry into the shadow",
            write_private_file_unsynced(shadow.join(name).as_path(), bytes.as_slice()),
        )?;
        io_step(
            "copying a conclusion entry for comparison",
            write_private_file_unsynced(copies.join(name).as_path(), bytes.as_slice()),
        )?;
    }
    if let Some(text) = record {
        io_step("writing the store-held record", write_store_record(copies.as_path(), text.as_str(), ref_format))?;
    }
    if sequencer_present {
        io_step(
            "copying the sequencer into the shadow",
            copy_tree(sequencer.as_path(), shadow.join(SEQUENCER_DIRECTORY).as_path()),
        )?;
        io_step(
            "copying the sequencer for comparison",
            copy_tree(sequencer.as_path(), copies.join(SEQUENCER_DIRECTORY).as_path()),
        )?;
    }
    return Ok(());
}

/// What:
///  Write the record that names the conclusion values a backend's ref store holds,
///  flushed only where a record that a power cut never gave back is damage.
/// Why:
///   A files backend holds no such value,
///  so an absent record and `{}` answer the same question there,
///  and the flush would make the commit wait for the disk for no answer
///  (`store_names_conclusions`).
///  A reftable backend keeps three pseudorefs in its store,
///  and a record missing there is read as damage,
///  so that one keeps its flush.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeStoreRecord(copies, text, format);
/// ```
fn write_store_record(copies: &Path, text: &str, format: RefFormat) -> std::io::Result<()> {
    let path: PathBuf = copies.join(STORE_RECORD_FILENAME);
    let bytes: String = format!("{text}\n");
    return if store_names_conclusions(format) {
        write_private_file(path.as_path(), bytes.as_bytes())
    } else {
        write_private_file_unsynced(path.as_path(), bytes.as_bytes())
    };
}

/// The heads a conclusion's parents come from,
/// in the order the preparation reads them:
/// a merge,
/// a cherry-pick and a revert.
pub const CONCLUSION_HEAD_NAMES: [&str; 3] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"];

/// What:
///  What the merge,
/// cherry-pick or revert state of the owning worktree names,
/// as one text that can be compared with a later reading.
/// Why:
///   A commit whose branch moved is made again over the state as it is now,
/// and the state must be the one it was prepared over:
/// another session that concluded or abandoned the operation meanwhile leaves nothing this commit could conclude,
/// so it is refused instead of becoming an ordinary commit of the same tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// conclusionStateText({ context, cwd, gitDir, refFormat }) // 'MERGE_HEAD=abc\nCHERRY_PICK_HEAD=\nREVERT_HEAD=\n'
/// ```
///
/// # Errors
///
/// `GitFailure` when an entry that exists cannot be read.
pub fn conclusion_state_text(context: &GitContext, cwd: &Path, git_dir: &Path, ref_format: RefFormat) -> Result<String, GitFailure> {
    // `mut` collects one line per head,
    // in the order a person's own `git status` names them.
    let mut text: String = String::new();
    for name in CONCLUSION_HEAD_NAMES {
        text.push_str(name);
        text.push('=');
        if is_store_held(ref_format, name) {
            text.push_str(owning_pseudoref(context, cwd, name).unwrap_or_default().as_str());
        } else {
            let bytes: Vec<u8> = recovery_step(read_optional(git_dir.join(name).as_path()))?.unwrap_or_default();
            text.push_str(String::from_utf8_lossy(bytes.as_slice()).as_ref());
        }
        text.push('\n');
    }
    return Ok(text);
}

/// What:
///  What the conclusion state named when a commit was prepared,
/// nothing when the commit concluded none.
/// Why:
///   A conversion from the kind to the state read happens in one place,
/// so a preparation and its replay always compare the same three heads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = theStateOfAConclusion({ context, cwd, capture }); // undefined for an ordinary commit and an amend
/// ```
///
/// # Errors
///
/// `GitFailure` when an entry that exists cannot be read.
pub fn the_state_of_a_conclusion(
    context: &GitContext,
    cwd: &Path,
    git_dir: &Path,
    ref_format: RefFormat,
    conclusion: crate::transaction_journal::Conclusion,
) -> Result<Option<String>, GitFailure> {
    return match conclusion {
        crate::transaction_journal::Conclusion::Merge | crate::transaction_journal::Conclusion::CherryPick | crate::transaction_journal::Conclusion::Revert => {
            Ok(Some(conclusion_state_text(context, cwd, git_dir, ref_format)?))
        }
        crate::transaction_journal::Conclusion::None | crate::transaction_journal::Conclusion::Amend => Ok(None),
    };
}

/// Conclusion copy controls stay out of the release executable.
#[cfg(test)]
#[path = "conclusion_copy_tests.rs"]
mod tests;

//! What:
//!  The smaller steps of a service preparation:
//! the changes a commit makes,
//! whether the configuration needs native Git,
//! the
//!  pathspec file the private index reads,
//! and the ending of a commit whose corrections removed every change.
//! Why:
//!  `service_prepare.rs` holds the preparation itself;
//! these depend on nothing in it,
//! so they are a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await changesOf(...); configNeedsNative(snapshot); await pathspecFileFor(...); await normalized(...)
//! ```

use crate::capture_invocation::InvocationCapture;
use crate::command_commit::CommitRegion;
use crate::private_index_request::{CaptureIndexes, indexed_request};
use crate::commit_conclusion::events_with_summary;
use crate::commit_no_change::{NO_CHANGE_CODE, NO_CHANGE_CORE_ID, no_change_message};
use crate::commit_transaction::{
    CommitInvocation, TransactionResult, TransactionStop, failed,
};
use crate::config_snapshot::ConfigSnapshot;
use crate::policy_events::PolicyEvent;
use crate::private_storage::write_private_file;
use crate::service_land_follow::{IndexPaths, update_index};
use crate::service_protocol::{ALLOWED_MODES, Change, MAX_CHANGES, is_object_name, is_plain_path};
use crate::transaction_git::{GitContext, run_git_checked};
use std::ffi::OsString;
use crate::transaction_journal::AddedPath;
use crate::worktree_completion::install_added_worktree_files;
use std::path::{Path, PathBuf};
use crate::service_prepare_ready_shapes::Prepared;

/// The all-zero object name of the repository's hash length,
/// standing for a branch that does not exist yet.
pub fn zero_object(empty_tree: &str) -> String {
    return "0".repeat(empty_tree.len());
}

/// The changes between the base and the private index:
/// path,
/// mode and object,
/// deletions as no object.
/// Nothing when the
///  embedded route cannot carry them.
pub fn changes_of(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, base: &str) -> Result<Option<Vec<Change>>, TransactionStop> {
    let arguments: Vec<String> = ["diff-index", "--cached", "-r", "--raw", "-z", "--no-renames", "--no-abbrev", base].map(String::from).to_vec();
    let output = run_git_checked(
        context,
        &indexed_request(cwd, &arguments, Some(indexes.commit_index.as_path()), Some(indexes.object_directory.as_path())),
    )?;
    // The listing is read as the bytes Git wrote: the header is text, and the name after it is a name and not text.
    let mut fields = output.stdout.split(ends_a_field).filter(is_not_empty_field);
    let mut changes: Vec<Change> = Vec::new();
    while let Some(raw_header) = fields.next() {
        let Some(named) = fields.next() else { return Ok(None) };
        let Ok(header) = std::str::from_utf8(raw_header) else { return Ok(None) };
        let parts: Vec<&str> = header.trim_start_matches(':').split(' ').collect();
        let [_, new_mode, _, new_name, status] = parts.as_slice() else { return Ok(None) };
        let deletion: bool = status.starts_with('D');
        let mode: u32 = if deletion { ALLOWED_MODES[0] } else { u32::from_str_radix(new_mode, 8).unwrap_or(0) };
        let path: PathBuf = PathBuf::from(crate::path_bytes::os_from_path_bytes(named));
        if !is_plain_path(path.as_path()) || !ALLOWED_MODES.contains(&mode) || (!deletion && !is_object_name(new_name)) {
            return Ok(None);
        }
        changes.push(Change { path, mode, blob: if deletion { None } else { Some((*new_name).to_string()) } });
        if changes.len() > MAX_CHANGES {
            return Ok(None);
        }
    }
    return Ok(Some(changes));
}

/// Whether a byte ends a field of Git's `-z` output.
fn ends_a_field(byte: &u8) -> bool {
    return *byte == b'\0';
}

/// What:
///  Whether a field of `git diff-index -z` output is not empty.
/// Why:
///  The output ends in a NUL,
/// so splitting at NUL leaves an empty field at the end that is no field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotEmptyField = (field: string) => field !== '';
/// ```
pub(crate) fn is_not_empty_field(field: &&[u8]) -> bool {
    return !field.is_empty();
}

/// Whether the repository's configuration signs commits:
/// `commit.gpgsign` is set to anything but a false word.
pub fn config_signs(snapshot: Option<&ConfigSnapshot>) -> bool {
    let Some(found) = snapshot else { return false };
    for entry in found.entries().iter() {
        let value: String = match entry.value.as_deref() {
            Some(bytes) => String::from_utf8_lossy(bytes).to_lowercase(),
            None => String::new(),
        };
        if entry.key == "commit.gpgsign" && !["false", "no", "off", "0"].contains(&value.as_str()) {
            return true;
        }
    }
    return false;
}

/// Whether the repository's configuration makes native `git commit` do something the service does not:
/// sign,
/// apply a
///  template or a clean-up mode,
/// use another message encoding or another comment character,
/// or change trailers.
pub fn config_needs_native(snapshot: Option<&ConfigSnapshot>) -> bool {
    let Some(found) = snapshot else { return true };
    for entry in found.entries().iter() {
        // A signing request is the one native key whose false words mean it does not ask for anything.
        if entry.key == "commit.gpgsign" {
            let value: String = match entry.value.as_deref() {
                Some(bytes) => String::from_utf8_lossy(bytes).to_lowercase(),
                None => String::new(),
            };
            if !["false", "no", "off", "0"].contains(&value.as_str()) {
                return true;
            }
            continue;
        }
        if crate::native_commit_configuration::is_native_commit_key(entry.key.as_str()) {
            return true;
        }
    }
    return false;
}

/// What:
///  The pathspec file the private index reads,
/// for a source named on the command line.
/// Why:
///   A regular file is read where it is named;
/// standard input was read once,
/// before the first preparation,
/// and is written
///  into each preparation's scratch directory,
/// because a second read would see nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pathspecFileFor(scratch, cwd, source, stdinBytes)
/// ```
pub(crate) fn pathspec_file_for(scratch: &Path, cwd: &Path, source: Option<&str>, standard_input: Option<&[u8]>) -> std::io::Result<Option<PathBuf>> {
    let Some(name) = source else { return Ok(None) };
    if name != "-" {
        let path: PathBuf = PathBuf::from(name);
        return Ok(Some(if path.is_absolute() { path } else { cwd.join(path) }));
    }
    let snapshot: PathBuf = scratch.join(crate::commit_index::PATHSPEC_INPUT_NAME);
    write_private_file(snapshot.as_path(), standard_input.unwrap_or_default())?;
    return Ok(Some(snapshot));
}

/// What:
///  The positional pathspecs of a command line,
/// as the text a request names them by.
/// Why:
///   A path that is not text is still a path,
///  and the request of an embedded commit carries its readable spelling,
///  so a name the service cannot carry sends the commit to Git
///  (`embedded_that_fits`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = pathspecTexts(pathspecs);
/// ```
pub(crate) fn pathspec_texts(pathspecs: &[OsString]) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for token in pathspecs {
        texts.push(token.to_string_lossy().into_owned());
    }
    return texts;
}

/// What:
///  The file the private index reads a pathspec list from,
/// or nothing when the line names none.
/// Why:
///   A pathspec file named `-` is read once,
/// before the first preparation,
/// and written into each preparation's scratch directory,
/// because a second read would see nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = thePathspecFile({ region, arguments, scratch, cwd, standardInput });
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the file cannot be written.
pub(crate) fn the_pathspec_file(
    region: &CommitRegion,
    arguments: &[OsString],
    scratch: &Path,
    cwd: &Path,
    standard_input: Option<&[u8]>,
) -> Result<Option<PathBuf>, TransactionStop> {
    let source: Option<String> = match region.pathspec_from_file {
        Some(value) => Some(String::from_utf8_lossy(crate::command_options_query::value_bytes(arguments, value)).into_owned()),
        None => None,
    };
    return match pathspec_file_for(scratch, cwd, source.as_deref(), standard_input) {
        Ok(made) => Ok(made),
        Err(error) => Err(crate::commit_transaction::failed(format!("reading the pathspec file failed: {error}"))),
    };
}

/// The ending of a commit whose corrections removed every change:
/// the index entries and the worktree copies of the
///  corrected files are reconciled with the base,
/// and the command says so and exits 1.
pub(crate) fn normalized(
    invocation: &CommitInvocation<'_>,
    capture: &InvocationCapture,
    base: &str,
    committed: &[Vec<u8>],
    records: &[AddedPath],
    settled: &(Vec<PolicyEvent>, u64, Vec<Vec<u8>>),
) -> Result<Prepared, TransactionStop> {
    let (events, passes, changed_paths) = settled;
    if let Err(code) = update_index(invocation.context, capture.layout.repository_root.as_path(), capture.layout.real_index_path.as_path(), base, IndexPaths::Named(committed)) {
        return Err(failed(format!("the index could not be reconciled with the corrected files: {}", code.word())));
    }
    let completion = match install_added_worktree_files(
        invocation.context,
        invocation.cwd,
        capture.layout.repository_root.as_path(),
        records,
        Some(capture.layout.object_directory.as_path()),
    ) {
        Ok(done) => done,
        Err(error) => return Err(TransactionStop::Message(error.0)),
    };
    let mut summary: Vec<PolicyEvent> = events_with_summary(events.clone(), *passes, changed_paths.as_slice());
    summary.push(PolicyEvent::CoreFinding {
        core_id: NO_CHANGE_CORE_ID,
        code: NO_CHANGE_CODE,
        message: no_change_message(completion.conflicted.as_slice()),
    });
    return Ok(Prepared::Ended(TransactionResult { events: summary, exit_code: 1, landed_oid: None, uses_lfs: false }));
}

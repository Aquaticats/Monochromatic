//! What:
//!  The append-only record of every command form that can discard uncommitted state: rejected, allowed because the
//!  worktree was clean, exempt, or run with the escape hatch.
//! Why:
//!  When uncommitted work vanishes, nothing said which command did it, and the person who lost it blamed the
//!  wrong one. One line per decision, with the time, process, parent process, directory and arguments, makes a
//!  loss attributable. Writing the record never changes the outcome of the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await appendFile(join(commonDir, 'cli-git-audit.jsonl'), JSON.stringify(entry) + '\n');
//! ```

use super::diagnostic_log::debug;
use super::iso_time::format_iso_milliseconds;
use super::json_record::{quote, string_array};
use super::service_protocol::RELEASE;
use super::rule_linked_worktree::GuardedCommand;
use super::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The file inside the common Git directory that collects the lines of every worktree of the repository.
pub const AUDIT_FILE: &str = "cli-git-audit.jsonl";

/// The size at which the file moves to `cli-git-audit.jsonl.1`, replacing an older copy.
const ROTATE_AT_BYTES: u64 = 1024 * 1024;

/// Module tag for diagnostics.
const TAG: &str = "discardAudit";

/// What:
///  What the policy decided for a guarded form.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A rejection, a run in a clean worktree and a run under the hatch are the three things a later reader asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AuditOutcome = 'rejected' | 'allowed' | 'exempt' | 'hatch';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditOutcome {
    /// The wrapper refused the command.
    Rejected,
    /// The worktree was clean (or the form only reads), so the command ran.
    Allowed,
    /// The repository lies under an allowlisted tool cache.
    Exempt,
    /// The person or agent passed `--no-enforce-worktree`.
    Hatch,
    /// The command took a side of conflicted files after the wrapper saved a copy of each.
    SavedConflict,
}

impl AuditOutcome {
    /// The word written to the record.
    fn word(self) -> &'static str {
        match self {
            AuditOutcome::Rejected => return "rejected",
            AuditOutcome::Allowed => return "allowed",
            AuditOutcome::Exempt => return "exempt",
            AuditOutcome::Hatch => return "hatch",
            AuditOutcome::SavedConflict => return "saved-conflict",
        }
    }
}

/// What:
///  The common Git directory and the worktree kind an identity names, or nothing outside a repository.
/// Why:
///   The record lives in the common directory so every worktree of the repository writes to one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function auditPlace(identity: WorktreeIdentity): { commonDir: string; kind: string } | undefined;
/// ```
pub fn audit_place(identity: &WorktreeIdentity) -> Option<(PathBuf, &'static str)> {
    match identity {
        WorktreeIdentity::OutsideWorktree => return None,
        WorktreeIdentity::BareRepository { common_dir, .. } => {
            return Some((common_dir.clone(), "bare"));
        }
        WorktreeIdentity::MainWorktree { common_dir, .. } => {
            return Some((common_dir.clone(), "main"));
        }
        WorktreeIdentity::LinkedWorktree { common_dir, .. } => {
            return Some((common_dir.clone(), "linked"));
        }
    }
}

/// The invocation's own Git directory an identity names, or nothing outside a repository.
pub fn audit_git_dir(identity: &WorktreeIdentity) -> Option<&Path> {
    return match identity {
        WorktreeIdentity::OutsideWorktree => None,
        WorktreeIdentity::BareRepository { git_dir, .. } | WorktreeIdentity::MainWorktree { git_dir, .. } | WorktreeIdentity::LinkedWorktree { git_dir, .. } => Some(git_dir.as_path()),
    };
}

/// What:
///  Everything one record states.
///  A `struct` is a record with named fields.
/// Why:
///   The values travel together to the line builder, and a struct keeps the call readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AuditRecord = { timeMillis: bigint; pid: number; parentPid: number; form: string; outcome: string; /* ... */ };
/// ```
pub struct AuditRecord<'record> {
    /// Milliseconds since the Unix epoch.
    pub time_millis: u128,
    /// The wrapper process.
    pub process: u32,
    /// The process that started it.
    pub parent: u32,
    /// The guarded form.
    pub form: GuardedCommand,
    /// What the policy decided.
    pub outcome: AuditOutcome,
    /// The common Git directory and the worktree kind.
    pub place: &'record (PathBuf, &'static str),
    /// The invocation's own Git directory, which names the worktree and its service instance.
    pub git_dir: &'record Path,
    /// The working directory of the invocation.
    pub directory: &'record Path,
    /// The whole argument list, global options included.
    pub arguments: &'record [OsString],
    /// Whether uncommitted state existed, when the policy asked.
    pub dirty: Option<bool>,
    /// Where the copies of conflicted files were saved, each as `path => place`, or `path => none` when there was
    ///  nothing to copy; empty for every other outcome.
    pub copies: &'record [String],
}

/// What:
///  One record as a line of JSON, ending in a newline.
/// Why:
///   One `write` call per line keeps lines from interleaving when several processes append at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function auditLine(record: AuditRecord): string;
/// ```
pub fn audit_line(record: &AuditRecord) -> String {
    let rendered: Vec<String> = record
        .arguments
        .iter()
        .map(|argument| { return argument.to_string_lossy().into_owned(); })
        .collect();
    let dirty: &str = match record.dirty {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    let copies: String = if record.copies.is_empty() {
        String::new()
    } else {
        format!(",\"copies\":{}", string_array(record.copies))
    };
    // A refusal is what a reader scanning the file looks for; the rest are decisions that let the command run.
    let level: &str = if record.outcome == AuditOutcome::Rejected { "warn" } else { "info" };
    let time: String = format_iso_milliseconds(i64::try_from(record.time_millis).unwrap_or(i64::MAX));
    return format!(
        "{{\"schemaVersion\":1,\"time\":{},\"level\":\"{level}\",\"timeMillis\":{},\"release\":{},\"pid\":{},\"parentPid\":{},\"worktree\":{},\"gitDir\":{},\"form\":{},\
         \"outcome\":{},\"dirty\":{dirty},\"cwd\":{},\"arguments\":{}{copies}}}\n",
        quote(&time),
        quote(&record.time_millis.to_string()),
        quote(RELEASE),
        record.process,
        record.parent,
        quote(record.place.1),
        quote(&record.git_dir.to_string_lossy()),
        quote(&format!("{:?}", record.form)),
        quote(record.outcome.word()),
        quote(&record.directory.to_string_lossy()),
        string_array(&rendered),
    );
}

/// The process that started this one, or zero where the platform does not say.
/// One body with a gated block per platform, so a mutation of the whole function is built and tested on every host.
fn parent_process() -> u32 {
    #[cfg(unix)]
    {
        return std::os::unix::process::parent_id();
    }
    #[cfg(not(unix))]
    {
        return 0;
    }
}

/// What:
///  Append one record to the audit file of the repository, best effort.
/// Why:
///   A command must run or be rejected the same way whether or not the record could be written, so a failure is
///  only a debug line. The directory is never created: a repository without a common Git directory has nothing to
///  protect, and a test location that does not exist stays untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordAudit({ identity, form, outcome, arguments, dirty });
/// ```
pub fn record_audit(
    identity: &WorktreeIdentity,
    form: GuardedCommand,
    outcome: AuditOutcome,
    arguments: &[OsString],
    dirty: Option<bool>,
) {
    record_audit_with_copies(identity, form, outcome, arguments, dirty, &[]);
}

/// What:
///  Append one record that also names the copies saved before the command ran.
/// Why:
///   A later reader looking for a file that vanished from a conflicted path finds where its last content went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordAudit({ identity, form, outcome, arguments, dirty, copies });
/// ```
pub fn record_audit_with_copies(
    identity: &WorktreeIdentity,
    form: GuardedCommand,
    outcome: AuditOutcome,
    arguments: &[OsString],
    dirty: Option<bool>,
    copies: &[String],
) {
    let Some(place) = audit_place(identity) else {
        return;
    };
    let target: PathBuf = place.0.join(AUDIT_FILE);
    let git_dir: &Path = audit_git_dir(identity).unwrap_or(place.0.as_path());
    let time_millis: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| { return elapsed.as_millis(); })
        .unwrap_or_default();
    let directory: PathBuf = std::env::current_dir().unwrap_or_default();
    let line: String = audit_line(&AuditRecord {
        time_millis,
        process: std::process::id(),
        parent: parent_process(),
        form,
        outcome,
        place: &place,
        git_dir,
        directory: directory.as_path(),
        arguments,
        dirty,
        copies,
    });
    if let Err(error) = append_line(target.as_path(), line.as_bytes()) {
        debug(TAG, &format!("the audit record could not be written to {}: {error}", target.display()));
    }
}

/// What:
///  Append bytes to the file with one write, moving a file past the size limit aside first.
/// Why:
///   A file that grows without bound would turn a safety record into a disk problem. No lock is taken, so the appender
///  guards the two ways a lock-free appender goes wrong: it moves the file aside only when the path still names the file it
///  opened (a writer that rotated a moment ago leaves a new, small file that a second rotation would put over the full
///  history), and when the file ends without a newline (a write a full disk or a signal cut short) it starts its own line
///  on a fresh one, so one torn line never swallows the next record. A writer that looks while another is half-way
///  through a write can add an empty line the same way; readers skip empty lines, and nothing is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendLine(path, bytes);
/// ```
pub fn append_line(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file: std::fs::File = open_for_append(target)?;
    if file.metadata()?.len() >= ROTATE_AT_BYTES {
        move_aside_if_still_named(target, &file)?;
        file = open_for_append(target)?;
    }
    let length: u64 = file.metadata()?.len();
    let mut line: Vec<u8> = Vec::with_capacity(bytes.len() + 1);
    if length > 0 && !ends_with_newline(&file, length) {
        line.push(b'\n');
    }
    line.extend_from_slice(bytes);
    return write_once(&mut file, line.as_slice());
}

/// What:
///  One `write` of the whole line, and an error if the system took only part of it.
/// Why:
///   `write_all` would go on with the rest after a short write, and another process may have appended in between, which
///  splits one record around another. A short write is a torn line; the next record starts on a fresh line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = await handle.write(line); if (written !== line.length) throw new Error('short write')
/// ```
fn write_once(file: &mut std::fs::File, line: &[u8]) -> std::io::Result<()> {
    loop {
        match file.write(line) {
            Ok(count) if count == line.len() => return Ok(()),
            Ok(count) => return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, format!("the system took {count} of {} bytes", line.len()))),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
}

/// What:
///  Whether the file at `target` can be opened to append, created when missing.
/// Why:
///   A pass that removes copies writes one line per removal; knowing before the first removal that the file cannot take it
///  keeps a directory from going without its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await (await open(target, 'a')).close();
/// ```
///
/// # Errors
/// The system's error when the file cannot be opened to append.
pub fn check_appendable(target: &Path) -> std::io::Result<()> {
    open_for_append(target)?;
    return Ok(());
}

/// The file opened to append, created when missing, and readable for the check of its last byte. A file the user may
/// write but not read is opened for appending alone: the check of the last byte is then skipped, never a record.
fn open_for_append(target: &Path) -> std::io::Result<std::fs::File> {
    return match std::fs::OpenOptions::new().read(true).append(true).create(true).open(target) {
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => std::fs::OpenOptions::new().append(true).create(true).open(target),
        other => other,
    };
}

/// Whether the last byte of the file, `length` long, is a newline; a byte that cannot be read counts as one, so a read
/// failure never adds a line.
fn ends_with_newline(file: &std::fs::File, length: u64) -> bool {
    use std::os::unix::fs::FileExt;
    let mut last: [u8; 1] = *b"\n";
    return file.read_exact_at(&mut last, length - 1).is_err() || last[0] == b'\n';
}

/// What:
///  Moves the file at `target` to `target.1` when the path still names `opened`.
/// Why:
///   Two writers can both see a full file; the second must not move the small file the first one started, which would
///  replace the full history in `.1` with a few lines. A path that names another file, or none, is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (sameFile(await stat(target), await opened.stat())) await rename(target, `${target}.1`)
/// ```
fn move_aside_if_still_named(target: &Path, opened: &std::fs::File) -> std::io::Result<()> {
    return move_aside_between(target, opened, &mut |_step| {});
}

/// The instants of a rotation at which another writer can act, named so a test can act there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RotationStep {
    /// The path was seen to name the opened file, and nothing has moved yet.
    BeforeMove,
}

/// The rotation with a callback at each instant another writer can act; `move_aside_if_still_named` passes one that does nothing.
fn move_aside_between(target: &Path, opened: &std::fs::File, at_step: &mut dyn FnMut(RotationStep)) -> std::io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let Ok(named) = std::fs::metadata(target) else {
        return Ok(());
    };
    let held: std::fs::Metadata = opened.metadata()?;
    if (named.dev(), named.ino()) != (held.dev(), held.ino()) {
        return Ok(());
    }
    at_step(RotationStep::BeforeMove);
    let mut aside: std::ffi::OsString = target.as_os_str().to_os_string();
    aside.push(".1");
    return std::fs::rename(target, aside);
}

/// The path, the line, the outcome words, and the limits.
#[cfg(test)]
#[path = "discard_audit_tests.rs"]
mod tests;

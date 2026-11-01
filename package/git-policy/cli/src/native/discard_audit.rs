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
use super::json_record::{quote, string_array};
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
}

impl AuditOutcome {
    /// The word written to the record.
    fn word(self) -> &'static str {
        match self {
            AuditOutcome::Rejected => return "rejected",
            AuditOutcome::Allowed => return "allowed",
            AuditOutcome::Exempt => return "exempt",
            AuditOutcome::Hatch => return "hatch",
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
    /// The working directory of the invocation.
    pub directory: &'record Path,
    /// The whole argument list, global options included.
    pub arguments: &'record [OsString],
    /// Whether uncommitted state existed, when the policy asked.
    pub dirty: Option<bool>,
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
    return format!(
        "{{\"schemaVersion\":1,\"timeMillis\":{},\"pid\":{},\"parentPid\":{},\"worktree\":{},\"form\":{},\
         \"outcome\":{},\"dirty\":{dirty},\"cwd\":{},\"arguments\":{}}}\n",
        quote(&record.time_millis.to_string()),
        record.process,
        record.parent,
        quote(record.place.1),
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
    let Some(place) = audit_place(identity) else {
        return;
    };
    let target: PathBuf = place.0.join(AUDIT_FILE);
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
        directory: directory.as_path(),
        arguments,
        dirty,
    });
    if let Err(error) = append_line(target.as_path(), line.as_bytes()) {
        debug(TAG, &format!("the audit record could not be written to {}: {error}", target.display()));
    }
}

/// What:
///  Append bytes to the file with one write, moving a file past the size limit aside first.
/// Why:
///   A file that grows without bound would turn a safety record into a disk problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendLine(path, bytes);
/// ```
pub fn append_line(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if std::fs::metadata(target).is_ok_and(|metadata| { return metadata.len() >= ROTATE_AT_BYTES; }) {
        let mut aside: std::ffi::OsString = target.as_os_str().to_os_string();
        aside.push(".1");
        std::fs::rename(target, aside)?;
    }
    let mut file: std::fs::File = std::fs::OpenOptions::new().append(true).create(true).open(target)?;
    return file.write_all(bytes);
}

/// The path, the line, the outcome words, and the limits.
#[cfg(test)]
#[path = "discard_audit_tests.rs"]
mod tests;

//! What:
//!  The append-only record of every command form that can discard uncommitted state:
//! rejected,
//! allowed because the
//!  worktree was clean,
//! exempt,
//! or run with the escape hatch.
//! Why:
//!  When uncommitted work vanishes,
//! nothing said which command did it,
//! and the person who lost it blamed the
//!  wrong one.
//! One line per decision,
//! with the time,
//! process,
//! parent process,
//! directory and arguments,
//! makes a
//!  loss attributable.
//! Writing the record never changes the outcome of the command.
//!  A full file moves aside to the next free generation,
//! and the generations are kept up to the bound every log shares,
//! except one that names a
//!  saved copy still on disk (`discard_audit_pins.rs`).
//! The appender,
//! the rotation and that bound are `log_lines_appended_without_a_lock.rs`
//! and
//!  `log_generations_kept_to_a_bound.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await appendFile(join(commonDir, 'monochromatic-git-policy-cli-audit.jsonl'), JSON.stringify(entry) + '\n');
//! ```

use crate::discard_audit_facts::AuditFact;
use crate::discard_audit_pins::names_a_copy_on_disk;
use crate::discard_audit_record::{AuditRecord, DecidedForm, audit_line};
use crate::diagnostic_log::debug;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
use crate::log_generations_kept_to_a_bound::{LOG_HISTORY_BYTES, LogRetention};
use crate::log_lines_appended_without_a_lock::append_line;
use crate::rule_linked_worktree::GuardedCommand;
use crate::wrapper_names_old_and_new::DISCARD_AUDIT_LOG;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{Path, PathBuf};

/// The file inside the common Git directory that collects the lines of every worktree of the repository.
pub const AUDIT_FILE: &str = DISCARD_AUDIT_LOG.current;

/// What the audit file's generations keep:
/// the bound every log of the wrapper shares,
/// and the line that keeps one past it.
/// Why:
///   An audit generation that names a saved copy still on disk is the only pointer to that copy,
/// and the owner
///  answered on 2026-11-10 (the question tool,
/// "Keep more,
/// capped (Recommended)") that such a line is never
///  dropped.
/// No other log's lines name anything outside the log,
/// so no other log pins a generation.
/// The measurement
///  behind the bound is beside `LOG_HISTORY_BYTES`.
pub const AUDIT_RETENTION: LogRetention = LogRetention { budget: LOG_HISTORY_BYTES, pins: names_a_copy_on_disk };

/// Module tag for diagnostics.
const TAG: &str = "discardAudit";

/// What:
///  What the policy decided for a guarded form.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A rejection,
/// a run in a clean worktree and a run under the hatch are the three things a later reader asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AuditOutcome = 'rejected' | 'allowed' | 'exempt' | 'hatch';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditOutcome {
    /// The wrapper refused the command.
    Rejected,
    /// The worktree was clean,
    /// the form only reads,
    /// or the only changes are those of the operation a discarded command undoes,
    /// so the command ran.
    Allowed,
    /// The repository lies under an allowlisted tool cache.
    Exempt,
    /// The person or agent passed `--no-enforce-worktree`.
    Hatch,
    /// The command took a side of conflicted files after the wrapper saved a copy of each.
    SavedConflict,
    /// The command would have deleted a nested Git repository,
    ///  and the wrapper moved it whole beside the audit log first.
    SavedNestedRepository,

    /// The command undid an operation that waited for its conflicts,
    /// after the wrapper saved a copy of the work that was not the operation's own.
    SavedAbort,
    /// The command dropped the commit or patch an operation stopped at and went on,
    /// after the wrapper saved a copy of the work that was not the operation's own.
    SavedSkip,
    /// The form is recorded and was never refused:
    ///  it runs as Git runs it,
    ///  by the owner's decision,
    ///  and this
    ///       line says what it took.
    Logged,
}

impl AuditOutcome {
    /// The word written to the record.
    pub fn word(self) -> &'static str {
        match self {
            AuditOutcome::Rejected => return "rejected",
            AuditOutcome::Allowed => return "allowed",
            AuditOutcome::Exempt => return "exempt",
            AuditOutcome::Hatch => return "hatch",
            AuditOutcome::SavedConflict => return "saved-conflict",
            AuditOutcome::SavedNestedRepository => return "saved-nested-repository",

            AuditOutcome::SavedAbort => return "saved-abort",
            AuditOutcome::SavedSkip => return "saved-skip",
            AuditOutcome::Logged => return "logged",
        }
    }
}

/// What:
///  The common Git directory and the worktree kind an identity names,
/// or nothing outside a repository.
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

/// The invocation's own Git directory an identity names,
/// or nothing outside a repository.
pub fn audit_git_dir(identity: &WorktreeIdentity) -> Option<&Path> {
    return match identity {
        WorktreeIdentity::OutsideWorktree => None,
        WorktreeIdentity::BareRepository { git_dir, .. } | WorktreeIdentity::MainWorktree { git_dir, .. } | WorktreeIdentity::LinkedWorktree { git_dir, .. } => Some(git_dir.as_path()),
    };
}

/// The process that started this one,
/// or zero where the platform does not say.
/// One body with a gated block per platform,
/// so a mutation of the whole function is built and tested on every host.
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
///  Append one record to the audit file of the repository,
/// best effort.
/// Why:
///   A command must run or be rejected the same way whether or not the record could be written,
/// so a failure is
///  only a debug line.
/// The directory is never created:
/// a repository without a common Git directory has nothing to
///  protect,
/// and a test location that does not exist stays untouched.
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
pub fn record_audit_with_facts(
    identity: &WorktreeIdentity,
    form: impl Into<DecidedForm>,
    outcome: AuditOutcome,
    arguments: &[OsString],
    dirty: Option<bool>,
    facts: &[AuditFact],
) {
    record_audit_with_copies_and_facts(identity, form, outcome, arguments, dirty, &[], facts);
}

/// What:
///  Append one record that also names the copies saved before the command ran,
/// and the facts the decision
///  read.
/// Why:
///   A reader looking for a file that vanished needs both where its last content went and what the
///  decision saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordAudit({ identity, form, outcome, arguments, dirty, copies, facts });
/// ```
pub fn record_audit_with_copies(
    identity: &WorktreeIdentity,
    form: GuardedCommand,
    outcome: AuditOutcome,
    arguments: &[OsString],
    dirty: Option<bool>,
    copies: &[String],
) {
    record_audit_with_copies_and_facts(identity, form, outcome, arguments, dirty, copies, &[]);
}

/// What:
///  Append one record from every part a decision produced.
/// Why:
///   One writer builds every line,
///  so no ending can skip the torn-line guard or the rotation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordAudit({ identity, form, outcome, arguments, dirty, copies, facts });
/// ```
pub fn record_audit_with_copies_and_facts(
    identity: &WorktreeIdentity,
    form: impl Into<DecidedForm>,
    outcome: AuditOutcome,
    arguments: &[OsString],
    dirty: Option<bool>,
    copies: &[String],
    facts: &[AuditFact],
) {
    let Some(place) = audit_place(identity) else {
        return;
    };
    let target: PathBuf = place.0.join(AUDIT_FILE);
    let git_dir: &Path = audit_git_dir(identity).unwrap_or(place.0.as_path());
    let time_millis: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_) => 0,
    };
    let directory: PathBuf = std::env::current_dir().unwrap_or_default();
    let line: String = audit_line(&AuditRecord {
        time_millis,
        process: std::process::id(),
        parent: parent_process(),
        form: form.into(),
        outcome,
        place: &place,
        git_dir,
        directory: directory.as_path(),
        arguments,
        dirty,
        copies,
        facts,
    });
    if let Err(error) = append_line(target.as_path(), line.as_bytes(), &AUDIT_RETENTION) {
        debug(TAG, &format!("the audit record could not be written to {}: {error}", target.display()));
    }
}

/// The path,
/// the line,
/// the outcome words,
/// and the limits.
#[cfg(test)]
#[path = "discard_audit_tests.rs"]
mod tests;

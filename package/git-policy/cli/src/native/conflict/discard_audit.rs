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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await appendFile(join(commonDir, 'monochromatic-git-policy-cli-audit.jsonl'), JSON.stringify(entry) + '\n');
//! ```

use crate::wrapper_names_old_and_new::DISCARD_AUDIT_LOG;
use crate::diagnostic_log::debug;
use generations::{Retention, install_generation, retention_of, trim_generations};
use crate::discard_audit_facts::AuditFact;
use crate::discard_audit_record::{AuditRecord, DecidedForm, audit_line};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
use crate::rule_linked_worktree::GuardedCommand;
use crate::worktree_identity::WorktreeIdentity;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The file inside the common Git directory that collects the lines of every worktree of the repository.
pub const AUDIT_FILE: &str = DISCARD_AUDIT_LOG.current;

/// The size at which a log file moves aside.
///
/// Measured on this host (read-only,
/// 2026-11-03T14:55Z):
/// `monochromatic-git-policy-cli-audit.jsonl` of the main repository held 322,900 bytes in
/// 950 lines from 2026-10-21T00:35:32Z to 2026-11-03T14:15:51Z,
/// about 23 KB a day with a daily range of 1,074 to 71,328
/// bytes,
/// so one mebibyte is about 45 days of audit lines at the mean rate and about 15 days at the busiest measured day.
/// `monochromatic-git-policy-cli-commit-timing.jsonl` held 760,605 bytes in 1,040 lines from 2026-11-02T09:01:37Z to 2026-11-03T14:55:02Z,
/// so it
/// reaches one mebibyte in about two days.
/// The commands and the window are in
/// `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Audit log rotation,
/// 1 MiB and one generation".
pub const ROTATE_AT_BYTES: u64 = 1024 * 1024;

/// The bytes of audit generations that name no saved copy still on disk,
/// kept beside the live file.
///
/// Derived from the measurement at `ROTATE_AT_BYTES`:
/// the busiest measured day was 211 lines,
/// and the lines written now are
/// about 489 bytes each (the newest day's 67 lines took 32,779 bytes),
/// so about 103 KB;
/// the saved copies that these lines
/// explain are kept for `DEFAULT_MAX_AGE_DAYS` (30) days,
/// and 30 such days are about 3.0 MiB.
/// Four generations hold that
/// with one to spare.
/// The coordinator picked this under the owner's delegation (pick 11,
/// option C),
/// open to the owner's veto.
/// It bounds only history that explains no saved copy,
/// because a generation that names a copy still on disk is never removed.
pub const AUDIT_HISTORY_BYTES: u64 = 4 * ROTATE_AT_BYTES;

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
    /// The worktree was clean (or the form only reads),
    /// so the command ran.
    Allowed,
    /// The repository lies under an allowlisted tool cache.
    Exempt,
    /// The person or agent passed `--no-enforce-worktree`.
    Hatch,
    /// The command took a side of conflicted files after the wrapper saved a copy of each.
    SavedConflict,
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
    if let Err(error) = append_line(target.as_path(), line.as_bytes()) {
        debug(TAG, &format!("the audit record could not be written to {}: {error}", target.display()));
    }
}

/// What:
///  Append bytes to the file with one write,
/// moving a file past the size limit aside first.
/// Why:
///   A file that grows without bound would turn a safety record into a disk problem.
/// No lock is taken,
/// so the appender
///  guards the three ways a lock-free appender goes wrong:
/// it moves the file aside only when what it took off the name is
///  the file it opened,
/// and gives any other file back (a writer that rotated a moment ago leaves a new,
/// small file that a
///  second rotation would put over the full history);
/// a rotation that cannot finish never costs the record;
/// and when the file ends without a newline (a write a full disk or a signal cut short) it starts its own line
///  on a fresh one,
/// so one torn line never swallows the next record.
/// A writer that looks while another is half-way
///  through a write can add an empty line the same way;
/// readers skip empty lines,
/// and nothing is lost.
///  Where the moved file goes depends on the file:
/// the audit file keeps a generation per rotation and trims them
///  (`discard_audit_generations.rs`),
/// and every other log (the timing logs and the push trigger log) keeps one,
/// `.1`,
///  which the next rotation replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendLine(path, bytes);
/// ```
pub fn append_line(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file: std::fs::File = open_for_append(target)?;
    if file.metadata()?.len() >= ROTATE_AT_BYTES {
        // A rotation that fails, or a name that cannot be opened again, leaves this writer on the file it has: the record is
        //  never what a failed housekeeping step costs.
        if let Err(error) = move_aside_if_still_named(target, &file) {
            debug(TAG, &format!("the audit file {} could not be moved aside: {error}", target.display()));
        } else {
            if retention_of(target) == Retention::Generations {
                // Best effort as well: a pass that cannot run leaves the generations as they are, which is more history, not less.
                if let Err(error) = trim_generations(target, AUDIT_HISTORY_BYTES) {
                    debug(TAG, &format!("the generations of {} could not be trimmed: {error}", target.display()));
                }
            }
            if let Ok(fresh) = open_for_append(target) {
                file = fresh;
            }
        }
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
///  One `write` of the whole line,
/// and an error if the system took only part of it.
/// Why:
///   `write_all` would go on with the rest after a short write,
/// and another process may have appended in between,
/// which
///  splits one record around another.
/// A short write is a torn line;
/// the next record starts on a fresh line.
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
///  Whether the file at `target` can be opened to append,
/// created when missing.
/// Why:
///   A pass that removes copies writes one line per removal;
/// knowing before the first removal that the file cannot take it
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

/// The file opened to append,
/// created when missing,
/// and readable for the check of its last byte.
/// A file the user may
/// write but not read is opened for appending alone:
/// the check of the last byte is then skipped,
/// never a record.
fn open_for_append(target: &Path) -> std::io::Result<std::fs::File> {
    return match std::fs::OpenOptions::new().read(true).append(true).create(true).open(target) {
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => std::fs::OpenOptions::new().append(true).create(true).open(target),
        other => other,
    };
}

/// Whether the last byte of the file,
/// `length` long,
/// is a newline;
/// a byte that cannot be read counts as one,
/// so a read
/// failure never adds a line.
fn ends_with_newline(file: &std::fs::File, length: u64) -> bool {
    use std::os::unix::fs::FileExt;
    let mut last: [u8; 1] = *b"\n";
    return file.read_exact_at(&mut last, length - 1).is_err() || last[0] == b'\n';
}

/// What:
///  Moves the file at `target` to `target.1` when the path still names `opened`.
/// Why:
///   Two writers can both see a full file;
/// the second must not move the small file the first one started,
/// which would
///  replace the full history in `.1` with a few lines.
/// A path that names another file,
/// or none,
/// is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (sameFile(await stat(target), await opened.stat())) await rename(target, `${target}.1`)
/// ```
fn move_aside_if_still_named(target: &Path, opened: &std::fs::File) -> std::io::Result<()> {
    return move_aside_between(target, opened, &mut NoObserver);
}

/// What:
///  The instants of a rotation at which another writer can act,
/// named so a test can act there.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The race is a matter of order,
/// so a test has to put another writer's whole rotation at exactly one instant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RotationStep = 'beforeMove' | 'afterMove';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RotationStep {
    /// The path was seen to name the opened file,
    /// and nothing has moved yet.
    BeforeMove,
    /// The name was taken from the path,
    /// and the file taken has not been looked at yet.
    AfterMove,
}

/// What:
///  Something told at each instant of a rotation at which another writer can act.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   A test acts as another writer at exactly one instant,
/// and the shipped rotation acts at none,
/// so a value stands where a
///  callback stood;
/// a closure of the right shape is an observer too,
/// so a test passes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RotationObserver { at(step: RotationStep): void; }
/// ```
trait RotationObserver {
    /// Called at one instant of the rotation.
    fn at(&mut self, step: RotationStep);
}

/// Every function-like value with the right shape is an observer:
/// `at` just calls it.
impl<Act: FnMut(RotationStep)> RotationObserver for Act {
    fn at(&mut self, step: RotationStep) {
        self(step);
    }
}

/// The observer of the shipped rotation,
/// which does nothing at any instant.
struct NoObserver;

impl RotationObserver for NoObserver {
    fn at(&mut self, _step: RotationStep) {}
}

/// What:
///  The path a rotation takes the file to before it knows what it took,
/// unique to this process and call.
/// Why:
///   The move that takes the file off its name has to land somewhere no other writer looks,
/// and two writers (or two threads
///  of one) must never pick the same place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${target}.moving-${process.pid}-${counter++}`
/// ```
fn private_name(target: &Path) -> PathBuf {
    // A counter that only goes up, shared by every thread of the process.
    static CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let call: u64 = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".moving-{}-{call}", std::process::id()));
    return PathBuf::from(name);
}

/// What:
///  Puts a file taken off `target` back at `target`,
/// or leaves it at its private name when something is there already.
/// Why:
///   A hard link fails when the name exists,
/// so a file a writer started in the meantime is never replaced.
/// A file that stays at
///  its private name loses nothing:
/// its lines are still on disk,
/// and writers that hold it open still write into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await link(taken, target); await unlink(taken); } catch (error) { if (error.code !== 'EEXIST') throw error; }
/// ```
fn give_back(taken: &Path, target: &Path) -> std::io::Result<()> {
    match std::fs::hard_link(taken, target) {
        Ok(()) => {
            if let Err(error) = std::fs::remove_file(taken) {
                debug(TAG, &format!("{} is back at its name but its second name {} stays: {error}", target.display(), taken.display()));
            }
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            debug(TAG, &format!("a newer audit file is at {}, so the file taken from it stays at {}", target.display(), taken.display()));
            return Ok(());
        }
        Err(error) => return Err(error),
    }
}

/// What:
///  The rotation,
/// with a callback at each instant another writer can act;
/// `move_aside_if_still_named` passes one that
///  does nothing.
/// Why:
///   The check of the path and the move cannot be one step,
/// so the move itself is the check:
/// the name is taken with one
///  rename to a private path (the writer that renames owns whatever it got,
/// with no lock),
/// then the file taken is compared
///  with the one opened.
/// The same file goes to `.1` (a timing log,
/// replacing the older `.1`) or to the next free generation
///  number (the audit file,
/// replacing nothing);
/// any other file is a newer file that a writer that rotated first started,
///  and it goes back to its name without replacing anything.
/// The rotation itself deletes nothing that a reader could
///  want except a timing log's older `.1`;
/// the removal of audit generations is `trim_generations`,
/// after the rotation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = await rename(target, privateName); // ENOENT: someone else took it, nothing to do
/// if (sameFile(await stat(taken), await opened.stat())) await rename(taken, `${target}.1`); else await giveBack(taken, target);
/// ```
fn move_aside_between(target: &Path, opened: &std::fs::File, at_step: &mut dyn RotationObserver) -> std::io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let Ok(named) = std::fs::metadata(target) else {
        return Ok(());
    };
    let held: std::fs::Metadata = opened.metadata()?;
    if (named.dev(), named.ino()) != (held.dev(), held.ino()) {
        return Ok(());
    }
    at_step.at(RotationStep::BeforeMove);
    let taken_at: PathBuf = private_name(target);
    match std::fs::rename(target, &taken_at) {
        Ok(()) => {}
        // Another writer took the name between the check and this move: nothing is left to rotate, and that is not a failure.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
    at_step.at(RotationStep::AfterMove);
    let taken: std::fs::Metadata = match std::fs::metadata(&taken_at) {
        Ok(taken) => taken,
        Err(error) => {
            give_back(taken_at.as_path(), target)?;
            return Err(error);
        }
    };
    if (taken.dev(), taken.ino()) != (held.dev(), held.ino()) {
        return give_back(taken_at.as_path(), target);
    }
    let placed: std::io::Result<()> = match retention_of(target) {
        Retention::OneGeneration => {
            let mut aside: std::ffi::OsString = target.as_os_str().to_os_string();
            aside.push(".1");
            std::fs::rename(&taken_at, aside)
        }
        Retention::Generations => install_generation(taken_at.as_path(), target),
    };
    if let Err(error) = placed {
        give_back(taken_at.as_path(), target)?;
        return Err(error);
    }
    return Ok(());
}

/// Where a rotated file goes,
/// and which generations of the audit file may be removed.
#[path = "discard_audit_generations.rs"]
mod generations;

/// The path,
/// the line,
/// the outcome words,
/// and the limits.
#[cfg(test)]
#[path = "discard_audit_tests.rs"]
mod tests;

/// The generations kept,
/// the bound on them,
/// and the timing logs' single generation are controlled in the generation test
/// module.
#[cfg(test)]
#[path = "discard_audit_generation_tests.rs"]
mod generation_tests;

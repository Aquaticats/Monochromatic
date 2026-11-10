//! What:
//!  The one appender every log of the wrapper writes through:
//! one `write` of a whole line or entry,
//! and the move that takes a full file off its name so a new one starts.
//! Why:
//!  Every log lives in a Git directory that several commands (and several worktrees) write at once,
//! and no lock may be taken,
//! so the appender
//!  guards the three ways a lock-free appender goes wrong:
//! it moves the file aside only when what it took off the name is
//!  the file it opened,
//! and gives any other file back (a writer that rotated a moment ago leaves a new,
//! small file that a
//!  second rotation would put over the full history);
//! a rotation that cannot finish never costs the line;
//! and when the file ends without a newline (a write a full disk or a signal cut short) a line starts its own
//!  line on a fresh one,
//! so one torn line never swallows the next record.
//! A writer that looks while another is half-way
//!  through a write can add an empty line the same way;
//! readers skip empty lines,
//! and nothing is lost.
//!  Where the moved file goes,
//! and which older files may be removed,
//! is `log_generations_kept_to_a_bound.rs`;
//! the caller says what its log keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! appendLine(path, line); appendEntry(privateLogPath, entry);
//! ```

use crate::diagnostic_log::debug;
use crate::log_generations_kept_to_a_bound::{LogRetention, install_generation, schedule_trim};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "appendedLogLine";

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

/// How one log's file is opened for the write that follows the rotation check.
type OpenLogFile = fn(&Path) -> std::io::Result<std::fs::File>;

/// What:
///  Append bytes to the file with one write,
/// moving a file past the size limit aside first.
/// Why:
///   A file that grows without bound would turn a safety record into a disk problem.
/// A rotation that fails,
/// or a name that cannot be opened again,
/// leaves this writer on the file it has:
/// the line is never what a failed housekeeping step costs.
/// The line starts on a fresh line when
///  the file ends without one,
/// so a torn line never swallows the next record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendLine(path, bytes, retention);
/// ```
///
/// # Errors
/// The system's error when the file cannot be opened or the write is short.
pub fn append_line(target: &Path, bytes: &[u8], retention: &LogRetention) -> std::io::Result<()> {
    return append_block(target, bytes, retention, open_beside_the_git_directory, true);
}

/// What:
///  Append one whole entry to a log the wrapper creates in its own folder,
/// as one write,
/// moving a file past the size limit aside first.
/// Why:
///   An entry carries its own separators and is written as it was composed:
/// the caller's block is what lands,
/// byte for byte.
/// The file is created private and a final link is refused,
/// because an entry carries a commit's subject
///  and the name is in a folder other users can reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(dirname(log), { recursive: true, mode: 0o700 }); appendEntry(log, entry, retention);
/// ```
///
/// # Errors
/// The system's error when the directory or the file cannot be created,
/// or the write is short.
pub fn append_entry(target: &Path, block: &[u8], retention: &LogRetention) -> std::io::Result<()> {
    if let Some(directory) = target.parent() {
        crate::private_storage::create_private_directory_all(directory)?;
    }
    return append_block(target, block, retention, open_private_log_file, false);
}

/// The file opened to append,
/// created when missing,
/// with one `write` of `block` after the rotation below.
/// A line's log is readable so that the check of its last byte can be made;
/// an entry's log is opened private and never followed through a link.
fn append_block(target: &Path, block: &[u8], retention: &LogRetention, open: OpenLogFile, starts_on_a_fresh_line: bool) -> std::io::Result<()> {
    let mut file: std::fs::File = open(target)?;
    if file.metadata()?.len() >= ROTATE_AT_BYTES {
        if let Err(error) = move_aside_if_still_named(target, &file) {
            debug(TAG, &format!("the log {} could not be moved aside: {error}", target.display()));
        } else {
            // Best effort as well, and on a thread of its own: a pass that cannot run leaves the generations as they are, which is more
            // history, not less, and the generation it reads costs a command nothing.
            schedule_trim(target, retention);
            if let Ok(fresh) = open(target) {
                file = fresh;
            }
        }
    }
    let length: u64 = file.metadata()?.len();
    let mut line: Vec<u8> = Vec::with_capacity(block.len() + 1);
    if starts_on_a_fresh_line && length > 0 && !ends_with_newline(&file, length) {
        line.push(b'\n');
    }
    line.extend_from_slice(block);
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
    open_beside_the_git_directory(target)?;
    return Ok(());
}

/// The file opened to append,
/// created when missing,
/// and readable for the check of its last byte.
/// A file the user may
/// write but not read is opened for appending alone:
/// the check of the last byte is then skipped,
/// never a record.
fn open_beside_the_git_directory(target: &Path) -> std::io::Result<std::fs::File> {
    return match std::fs::OpenOptions::new().read(true).append(true).create(true).open(target) {
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => std::fs::OpenOptions::new().append(true).create(true).open(target),
        other => other,
    };
}

/// The file of an entry's log opened to append,
/// created private and never through a final link,
/// because an entry carries a commit's subject.
fn open_private_log_file(target: &Path) -> std::io::Result<std::fs::File> {
    // `mut` allows the builder to be configured step by step.
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.append(true).create(true);
    #[cfg(unix)]
    {
        // The trait adds `.mode` and `.custom_flags` to open options on Unix.
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(crate::private_storage::PRIVATE_FILE_MODE).custom_flags(libc::O_NOFOLLOW);
    }
    return options.open(target);
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
///  Moves the file at `target` aside when the path still names `opened`.
/// Why:
///   Two writers can both see a full file;
/// the second must not take the small file the first one started,
/// which would
///  replace a rotation's history with a few lines.
/// A path that names another file,
/// or none,
/// is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (sameFile(await stat(target), await opened.stat())) await rotate(target)
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
pub enum RotationStep {
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
pub trait RotationObserver {
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
            debug(TAG, &format!("a newer log is at {}, so the file taken from it stays at {}", target.display(), taken.display()));
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
/// The same file goes to the next free generation number,
/// replacing nothing;
/// any other file is a newer file that a writer that rotated first started,
///  and it goes back to its name without replacing anything.
/// The rotation itself removes nothing:
/// the generations a log no longer needs are removed by `trim_generations`,
/// after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = await rename(target, privateName); // ENOENT: someone else took it, nothing to do
/// if (sameFile(await stat(taken), await opened.stat())) await installGeneration(taken, target); else await giveBack(taken, target);
/// ```
///
/// # Errors
/// The system's error when the file cannot be taken off its name or given a generation number.
pub fn move_aside_between(target: &Path, opened: &std::fs::File, at_step: &mut dyn RotationObserver) -> std::io::Result<()> {
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
    if let Err(error) = install_generation(taken_at.as_path(), target) {
        give_back(taken_at.as_path(), target)?;
        return Err(error);
    }
    return Ok(());
}

/// The appender,
/// the rotation steps,
/// the size limit and the opening modes are controlled in `log_lines_appended_without_a_lock_tests.rs`.
#[cfg(test)]
#[path = "log_lines_appended_without_a_lock_tests.rs"]
mod tests;

/// The generations the appender's rotations make,
/// the bound on them and the pass that removes the oldest are controlled in
/// `log_generations_kept_to_a_bound_tests.rs`.
#[cfg(test)]
#[path = "log_generations_kept_to_a_bound_tests.rs"]
mod generation_tests;

/// What a rotation costs on the path that appends a line,
/// measured for the record and not run as a control.
#[cfg(test)]
#[path = "log_rotation_cost_measurement_tests.rs"]
mod rotation_cost_measurement_tests;

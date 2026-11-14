//! What:
//! Publishing a new `toolchain.lock.json` safely:
//! record the revisions (content hashes) of the plan and the
//!           lock when they are read,
//! then,
//! holding an exclusive OS lock for the scope,
//! read them again and publish only
//!           if neither changed.
//! Why:
//! `update` can run for minutes on the network while a person edits the plan or another mpm-shadow rewrites
//!           the lock.
//! Writing over either would lose work,
//! so a changed revision refuses the publish and names the
//!           file;
//! the OS lock makes the re-read and the write one step as far as other mpm-shadow runs can tell.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // publish.ts
//! export function readRevisions(scope: Scope): Revisions
//! export function publishLock(scope: Scope, expected: Revisions, text: string): void  // throws when a revision moved
//! ```

/// Imports what this module uses from `std::fs`.
use std::fs::{self, File, OpenOptions};
/// Imports the write trait the holder's note is written with.
use std::io::Write;
/// Imports what this module uses from `std::path`.
use std::path::Path;
/// Imports the clock and the pause the bounded wait is measured with.
use std::time::{Duration, Instant};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `sha256_hex` from `super::hash`.
use super::hash::sha256_hex;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports the shared named helpers.
use super::util::{write_atomically as write_bytes_atomically, write_atomically_with_mode};

/// What:
/// The content hash of one file,
/// or `None` when the file does not exist.
/// Why:
/// A revision is "what the bytes were when I read them";
/// comparing two revisions says whether anyone wrote
///           the file in between,
/// whatever tool they used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Revision = string | undefined;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revision(
    /// The SHA-256 of the file's bytes as lowercase hex,
    /// or `None` when the file does not exist.
    pub Option<String>,
);

/// The revisions of the plan and the lock,
/// read together.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revisions {
    /// Revision of `toolchain.plan.jsonc`.
    pub plan: Revision,
    /// Revision of `toolchain.lock.json`.
    pub lock: Revision,
}

/// The bytes of a file,
/// or `None` when it does not exist;
/// any other read failure is an error naming the path.
pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, ToolchainError> {
    return match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ToolchainError::config(format!("cannot read {}: {error}", path.display()))),
    };
}

/// The revision of the bytes read from a file (`None` stays `None`).
pub fn revision_of(bytes: &Option<Vec<u8>>) -> Revision {
    return match bytes {
        Some(content) => Revision(Some(sha256_hex(content))),
        None => Revision(None),
    };
}

/// Reads both files and returns their revisions.
pub fn read_revisions(scope: &Scope) -> Result<Revisions, ToolchainError> {
    let plan: Option<Vec<u8>> = read_optional(&scope.plan_file)?;
    let lock: Option<Vec<u8>> = read_optional(&scope.lock_file)?;
    return Ok(Revisions { plan: revision_of(&plan), lock: revision_of(&lock) });
}

/// What:
/// How long a command waits for the scope's lock before it goes on without it, or refuses.
/// Why:
/// Nothing waits forever, so a holder that is stopped or hung must not stop another command.
/// The lock covers one checkout's link and launcher writes: the whole `install` measured on this machine takes about
/// 60 ms on a checkout with 174 declared bins (README, "The launcher"), and the one member of the held region that can take
/// long is the bounded check of the repository's link, which waits at most `repository_link_cause::CHECK_DEADLINE`
/// (10 s). The bound is smaller than that check's own bound on purpose: a waiter that gives up only loses this run's
/// retirement of the names the lock declares nowhere, which the next install does, and never any data.
pub const GUARD_WAIT: Duration = Duration::from_secs(5);

/// The pause between two attempts at the lock.
const GUARD_POLL: Duration = Duration::from_millis(20);

/// What:
/// The exclusive OS lock for one scope,
/// held for as long as the value lives.
/// Why:
/// Dropping the value closes the file,
/// which releases the lock,
/// so an early return or a panic cannot leave
///           the scope locked.
/// It also clears the note it left in the file, so a later holder's note is never a dead process's number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Guard { constructor(private file: FileHandle) {} [Symbol.dispose]() { this.file.close(); } }
/// ```
#[derive(Debug)]
pub struct Guard {
    /// The open file that holds the lock;
    /// kept open so the lock stays taken,
    /// and truncated on drop so the holder's note does not outlive it.
    file: File,
}

/// The methods of `Guard`.
impl Guard {
    /// What:
    /// Leaves this process's number in the guard file,
    /// for a command that cannot take the lock to name it.
    /// Why:
    /// The file is the one thing on disk that a waiter can read,
    /// and a note that cannot be written costs the note, never the lock.
    fn leave_a_note(&self) {
        let _ = self.file.set_len(0);
        let note: String = format!("{}\n", std::process::id());
        let mut borrowed: &File = &self.file;
        let _ = borrowed.write_all(note.as_bytes());
    }
}

/// Clears the holder's note; the lock itself is released when the file closes after this.
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.file.set_len(0);
    }
}

/// Opens (creating it and its directory) the scope's guard file.
fn open_guard_file(path: &Path) -> Result<File, ToolchainError> {
    if let Some(parent) = path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        return Err(ToolchainError::failure(format!("cannot create {} for the scope lock: {error}", parent.display())));
    }
    return match OpenOptions::new().create(true).truncate(false).write(true).open(path) {
        Ok(file) => Ok(file),
        Err(error) => Err(ToolchainError::failure(format!("cannot open the scope lock file {}: {error}", path.display()))),
    };
}

/// What:
/// Takes the scope's exclusive lock,
/// waiting no longer than `GUARD_WAIT`,
/// or `None` when another command holds it for that long.
/// Why:
/// The wait has a stated bound,
/// so a holder that is stopped or hung cannot stop this command for ever;
/// the caller says in words what it does without the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function acquireGuard(scope: Scope): Guard | undefined  // throws
/// ```
pub fn acquire_guard(scope: &Scope) -> Result<Option<Guard>, ToolchainError> {
    let deadline: Instant = Instant::now() + GUARD_WAIT;
    loop {
        match try_acquire_guard(scope)? {
            Some(guard) => return Ok(Some(guard)),
            None => {
                if Instant::now() >= deadline {
                    return Ok(None);
                }
                std::thread::sleep(GUARD_POLL);
            }
        }
    }
}

/// What:
/// Who holds the scope's lock,
/// as far as the guard file says.
/// Why:
/// A command that could not take the lock says so in words,
/// and the holder's note is what can name it;
/// a note that is absent or unreadable is said to be so rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function holderOf(scope: Scope): string
/// ```
pub fn holder_of(scope: &Scope) -> String {
    let text: String = fs::read_to_string(&scope.guard_file).unwrap_or_default();
    return match text.trim().parse::<u32>() {
        Ok(pid) => format!("another mpm-shadow (process {pid}, the last one that took it)"),
        Err(_) => "another command in this checkout (it left no note of its process)".to_string(),
    };
}

/// Takes the scope's exclusive lock without waiting;
/// `None` when another run holds it.
pub fn try_acquire_guard(scope: &Scope) -> Result<Option<Guard>, ToolchainError> {
    let file: File = open_guard_file(&scope.guard_file)?;
    return match file.try_lock() {
        Ok(()) => {
            let guard: Guard = Guard { file };
            guard.leave_a_note();
            Ok(Some(guard))
        }
        Err(std::fs::TryLockError::WouldBlock) => Ok(None),
        Err(std::fs::TryLockError::Error(error)) => Err(ToolchainError::failure(format!(
            "cannot test the scope lock {}: {error}",
            scope.guard_file.display()
        ))),
    };
}

/// The name of the file revision `kind` came from,
/// for the refusal message.
fn moved_files(expected: &Revisions, observed: &Revisions, scope: &Scope) -> Vec<String> {
    let mut moved: Vec<String> = Vec::new();
    if expected.plan != observed.plan {
        moved.push(scope.plan_file.display().to_string());
    }
    if expected.lock != observed.lock {
        moved.push(scope.lock_file.display().to_string());
    }
    return moved;
}

/// What:
/// Writes `text` to `target` through the toolchain's one atomic write.
/// Why:
/// A reader (a `jq` in CI,
/// another mpm-shadow) sees the old file or the new one,
/// never half of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeAtomically(target: string, text: string): void
/// ```
pub fn write_atomically(target: &Path, text: &str) -> Result<(), ToolchainError> {
    return write_bytes_atomically(target, text.as_bytes());
}

/// What:
/// Publishes `text` as the new lock if the plan and lock still have the revisions `expected`.
/// Why:
/// This is the one write path of the lock.
/// It takes the scope lock,
/// re-reads both revisions,
/// refuses when
///           either moved,
/// and writes atomically;
/// nothing else in the crate writes `toolchain.lock.json`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publishLock(scope: Scope, expected: Revisions, text: string): void
/// ```
pub fn publish_lock(scope: &Scope, expected: &Revisions, text: &str) -> Result<(), ToolchainError> {
    let Some(guard) = acquire_guard(scope)? else {
        return Err(ToolchainError::failure(format!(
            "{} is held by {}, and this command waited {} s for it, so {} was not written and nothing was changed; \
             run the command again when the other one has finished",
            scope.guard_file.display(),
            holder_of(scope),
            GUARD_WAIT.as_secs(),
            scope.lock_file.display()
        )));
    };
    let observed: Revisions = read_revisions(scope)?;
    let moved: Vec<String> = moved_files(expected, &observed, scope);
    if !moved.is_empty() {
        drop(guard);
        return Err(ToolchainError::drift(format!(
            "{} changed while this command was running, so the new lock was not written; \
             run the command again to start from the current files",
            moved.join(" and ")
        )));
    }
    if let Some(parent) = scope.lock_file.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", parent.display())));
    }
    write_atomically(&scope.lock_file, text)?;
    drop(guard);
    return Ok(());
}

/// What:
/// Writes `text` to `target` the way `write_atomically` does,
/// and gives the file the mode the old one had.
/// Why:
/// The plan is a file people own;
/// a rename replaces its inode,
/// so a plan kept readable by its owner alone would come back readable by everyone.
/// The mode is asked for at the temporary's creation and put on it before its first byte,
/// so neither the file that holds the plan's content nor the file a kill leaves behind is wider than the plan
/// (`util::write_atomically_with_mode`, "what a kill leaves, step by step").
/// A target that does not exist yet keeps the mode a new file gets,
/// and a target whose mode cannot be read is a failure rather than a silent widening.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeAtomicallyKeepingMode(target: string, text: string): void
/// ```
#[cfg(unix)]
fn write_atomically_keeping_mode(target: &Path, text: &str) -> Result<(), ToolchainError> {
    use std::os::unix::fs::PermissionsExt;
    let mode: Option<u32> = match fs::metadata(target) {
        Ok(metadata) => Some(metadata.permissions().mode()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(ToolchainError::failure(format!(
                "cannot read the mode of {}: {error}",
                target.display()
            )));
        }
    };
    return write_atomically_with_mode(target, text.as_bytes(), mode);
}

/// What:
/// `write_atomically_keeping_mode` where the platform has no mode to keep.
/// Why:
/// Windows and macOS are out of scope for now,
/// and a plan written there is an ordinary new file.
#[cfg(not(unix))]
fn write_atomically_keeping_mode(target: &Path, text: &str) -> Result<(), ToolchainError> {
    return write_atomically(target, text);
}

/// What:
/// Publishes `text` as the new plan if the plan still has the revision `expected`.
/// Why:
/// Only this command writes the plan,
/// and only to append an entry;
/// another mpm-shadow adding a tool at the same moment read the file before this run did,
/// so writing over it would drop that entry.
/// The scope lock makes the re-read and the write one step as far as other runs can tell,
/// and the file keeps the permissions it had,
/// because the global plan is readable by its owner alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publishPlan(scope: Scope, expected: Revision, text: string): void
/// ```
pub fn publish_plan(scope: &Scope, expected: &Revision, text: &str) -> Result<(), ToolchainError> {
    let guard: Guard = acquire_guard(scope)?;
    let observed: Revisions = read_revisions(scope)?;
    if expected != &observed.plan {
        drop(guard);
        return Err(ToolchainError::drift(format!(
            "{} changed while this command was running, so the new entry was not written; \
             run the command again to start from the current file",
            scope.plan_file.display()
        )));
    }
    let result: Result<(), ToolchainError> = write_atomically_keeping_mode(&scope.plan_file, text);
    drop(guard);
    return result;
}

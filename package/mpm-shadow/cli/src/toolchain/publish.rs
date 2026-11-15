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
use std::fs::{self, File};
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
/// Imports the one way this tool opens or creates a file it did not just make.
use super::ordinary_file::{create_lock_file, read_ordinary_text, read_ordinary_through_links};
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

/// What:
/// The bytes of a file,
/// or `None` when it does not exist;
/// any other read failure is an error naming the path.
/// Why:
/// The plan and the lock are files this run did not create,
/// so the read never waits (a named pipe here would stop an install for ever) and takes nothing but an ordinary file,
/// reached through a symbolic link when the path is one.
pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, ToolchainError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ToolchainError::config(format!("cannot read {}: {error}", path.display()))),
    };
    return match read_ordinary_through_links(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(reason) => Err(ToolchainError::config(reason)),
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
/// How long a publish waits for the scope's guard before it leaves its text to the next install.
/// Why:
/// Nothing waits forever, so a holder that is stopped or hung must not stop another command.
/// The guard covers exactly one region, and this is where its reason lives:
/// the read of the plan's and the lock's revisions, the comparison of them with the ones the caller read, and the
/// atomic write, so that two commands publishing the plan or the lock do not both pass the comparison and write over
/// each other's finished file, and so that the plan and the lock do not move between each other's publishes.
/// A publish that cannot take it inside this bound does not write (the ninth review reproduced two bounded publishers
/// erasing one another's finished plans); it answers `Published::Deferred` with the sentence that names the holder, and
/// the caller reports it and goes on, because a failed publish fails the install and the commit waiting for it, while a
/// deferred one is done by the next install.
/// The region is a read and a write of a few hundred kilobytes at most, so this bound has a hundredfold slack.
pub const GUARD_WAIT: Duration = Duration::from_secs(5);

/// What a publish did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Published {
    /// The file holds this run's text, or already did.
    Written,
    /// Another command holds the scope guard past the bound, so nothing was written, and the sentence says why.
    Deferred(String),
}

/// The methods of `Published`.
impl Published {
    /// The sentence for work this publish left to the next install, when there is one.
    pub fn deferred(&self) -> Option<&str> {
        return match self {
            Published::Written => None,
            Published::Deferred(sentence) => Some(sentence.as_str()),
        };
    }
}

/// What:
/// Who holds the scope's guard,
/// as far as the guard file says.
/// Why:
/// A command that could not take the guard says so in words,
/// and the holder's note is what can name it;
/// a note that is absent or unreadable is said to be so rather than guessed at.
pub fn holder_of(scope: &Scope) -> String {
    let text: String = read_ordinary_text(&scope.guard_file).unwrap_or_default();
    return match text.trim().parse::<u32>() {
        Ok(pid) => format!("another mpm-shadow (process {pid}, the last one that took it)"),
        Err(_) => "another command in this checkout (it left no note of its process)".to_string(),
    };
}

/// The sentence for a publish the guard's holder made this run leave to the next install.
fn deferred_sentence(scope: &Scope, target: &Path) -> String {
    return format!(
        "{} was not written: {} holds {}, and this command waited {} s for it; the next install writes it",
        target.display(),
        holder_of(scope),
        scope.guard_file.display(),
        GUARD_WAIT.as_secs()
    );
}

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
    return match create_lock_file(path) {
        Ok(file) => Ok(file),
        Err(reason) => Err(ToolchainError::failure(reason)),
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
/// It takes the scope guard,
/// re-reads both revisions,
/// refuses when
///           either moved,
/// and writes atomically under that guard;
/// nothing else in the crate writes `toolchain.lock.json`.
/// A guard it cannot take means nothing is written and the answer says so: writing without the guard is what would let
/// a second publisher erase the first one's finished lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publishLock(scope: Scope, expected: Revisions, text: string): Published
/// ```
pub fn publish_lock(scope: &Scope, expected: &Revisions, text: &str) -> Result<Published, ToolchainError> {
    let Some(guard) = acquire_guard(scope)? else {
        return Ok(Published::Deferred(deferred_sentence(scope, &scope.lock_file)));
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
    return Ok(Published::Written);
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
pub fn publish_plan(scope: &Scope, expected: &Revision, text: &str) -> Result<Published, ToolchainError> {
    let Some(guard) = acquire_guard(scope)? else {
        return Ok(Published::Deferred(deferred_sentence(scope, &scope.plan_file)));
    };
    let observed: Revisions = read_revisions(scope)?;
    #[cfg(test)]
    seam::after_the_revision_read();
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
    result?;
    return Ok(Published::Written);
}

/// The step a control uses to run a second publisher between this one's revision read and its write.
/// Why:
/// The ninth review placed a whole second publish there: with the guard held by a third command, both publishers read
/// one revision and wrote, and the later rename erased the earlier one. Compiled in tests only.
#[cfg(test)]
pub(crate) mod seam {
    /// Imports `Mutex` from `std`.
    use std::sync::Mutex;

    /// The hook a control sets.
    static AFTER_THE_REVISION_READ: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

    /// Sets (or with `None` clears) the hook run after the observed revisions are read.
    pub(crate) fn set_after_the_revision_read(hook: Option<Box<dyn Fn() + Send>>) {
        *AFTER_THE_REVISION_READ.lock().expect("the seam is not poisoned") = hook;
    }

    /// Runs the hook once, when there is one.
    pub(crate) fn after_the_revision_read() {
        let waiting: Option<Box<dyn Fn() + Send>> = AFTER_THE_REVISION_READ.lock().expect("the seam is not poisoned").take();
        if let Some(hook) = waiting {
            hook();
        }
    }
}

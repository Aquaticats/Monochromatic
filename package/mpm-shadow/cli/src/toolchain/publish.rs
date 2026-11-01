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
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `sha256_hex` from `super::hash`.
use super::hash::sha256_hex;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports the shared named helpers.
use super::util::file_name_os;

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
/// The exclusive OS lock for one scope,
/// held for as long as the value lives.
/// Why:
/// Dropping the value closes the file,
/// which releases the lock,
/// so an early return or a panic cannot leave
///           the scope locked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Guard { constructor(private file: FileHandle) {} [Symbol.dispose]() { this.file.close(); } }
/// ```
#[derive(Debug)]
pub struct Guard {
    /// The open file that holds the lock;
    /// kept only so it stays open.
    _file: File,
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

/// Takes the scope's exclusive lock,
/// waiting until another mpm-shadow run releases it.
pub fn acquire_guard(scope: &Scope) -> Result<Guard, ToolchainError> {
    let file: File = open_guard_file(&scope.guard_file)?;
    if let Err(error) = file.lock() {
        return Err(ToolchainError::failure(format!(
            "cannot take the scope lock {}: {error}",
            scope.guard_file.display()
        )));
    }
    return Ok(Guard { _file: file });
}

/// Takes the scope's exclusive lock without waiting;
/// `None` when another run holds it.
pub fn try_acquire_guard(scope: &Scope) -> Result<Option<Guard>, ToolchainError> {
    let file: File = open_guard_file(&scope.guard_file)?;
    return match file.try_lock() {
        Ok(()) => Ok(Some(Guard { _file: file })),
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
/// Writes `text` to a temporary file beside `target`,
/// then renames it over `target`.
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
    let mut temporary_name = file_name_os(target);
    temporary_name.push(format!(".{}.tmp", std::process::id()));
    let temporary: PathBuf = target.with_file_name(temporary_name);
    if let Err(error) = fs::write(&temporary, text) {
        return Err(ToolchainError::failure(format!("cannot write {}: {error}", temporary.display())));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        // Best effort: the temporary file is useless now, and a failed removal changes nothing for the caller.
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!("cannot replace {}: {error}", target.display())));
    }
    return Ok(());
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
    let guard: Guard = acquire_guard(scope)?;
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

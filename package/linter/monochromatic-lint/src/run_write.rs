//! What:
//!  Replace a file's contents atomically while keeping its permission bits.
//! Why:
//!  Opening the target for writing truncates it first,
//!  so a crash mid-write could leave an
//! empty file.
//!  Writing a complete sibling temporary file and renaming it over the target means
//! readers see either the old bytes or the new bytes,
//!  never a partial file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await writeFile(temp, contents, { mode }); await rename(temp, path);
//! ```

/// Import file handles and the write trait that provides `write_all`.
/// Import a counter that several threads can advance without a lock.
use std::{
    ffi::OsString,
    fs::{File, Metadata, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// What:
///  A process-wide sequence for temporary file names.
/// Why:
///  Process id plus sequence keeps concurrent workers,
///  and concurrent processes,
///  from choosing
/// the same temporary name in one directory.
///  `AtomicU64` (not a plain `u64`) because worker
/// threads increment it concurrently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let sequence = 0; // single-threaded in Node
/// ```
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// What:
///  A failed replacement,
///  naming the file and the operation that failed.
/// Why:
///  The caller reports that the original bytes are unchanged;
///  the cause must be visible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WriteError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteError {
    /// Affected path and failed operation.
    pub message: String,
}

/// Render the failure through the ordinary error interface.
impl std::fmt::Display for WriteError {
    /// Borrow the formatter only while writing the stored explanation.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// Mark the failure as a standard error for application error handling.
impl std::error::Error for WriteError {}

/// What:
///  Build a typed failure from an operation name,
///  a path and an I/O error.
/// Why:
///  Every step reports the same shape,
///  so the message always names its file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failure(operation: string, path: Path, error: Error): WriteError;
/// ```
fn failure(operation: &str, path: &Path, error: &std::io::Error) -> WriteError {
    return WriteError {
        message: format!("Cannot {operation} {}: {error}.", path.display()),
    };
}

/// What:
///  Create the temporary file exclusively,
///  readable and writable by its owner only.
/// Why:
///  `create_new` refuses an existing path,
///  so a leftover or foreign file is never overwritten
/// or followed.
///  The restrictive initial mode keeps a private file's new contents private during
/// the short time before the original permission bits are copied onto it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(temporary, 'wx', 0o600);
/// ```
fn create_temporary(temporary: &Path) -> Result<File, WriteError> {
    let mut options: OpenOptions = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        /// Import the extension trait that adds the Unix-only `mode` setting to `OpenOptions`.
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(temporary) {
        Ok(handle) => return Ok(handle),
        Err(error) => return Err(failure("create temporary file", temporary, &error)),
    }
}

/// What:
///  Write every byte,
///  copy the original permission bits,
///  and flush to disk.
/// Why:
///  Permissions are set on the open handle after writing,
///  so the result carries the original
/// mode exactly,
///  independent of the process umask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handle.writeFile(contents); await handle.chmod(mode); await handle.sync();
/// ```
fn fill_temporary(
    file: &mut File,
    temporary: &Path,
    contents: &[u8],
    original: &Metadata,
) -> Result<(), WriteError> {
    if let Err(error) = file.write_all(contents) {
        return Err(failure("write temporary file", temporary, &error));
    }
    if let Err(error) = file.set_permissions(original.permissions()) {
        return Err(failure(
            "set permissions on temporary file",
            temporary,
            &error,
        ));
    }
    if let Err(error) = file.sync_all() {
        return Err(failure("flush temporary file", temporary, &error));
    }
    return Ok(());
}

/// What:
///  Replace `path` with `contents` through a same-directory temporary file and a rename.
/// Why:
///  The rename is atomic on one filesystem.
///  A symbolic link is resolved first,
///  so the link
/// stays a link and its target file is the one replaced.
///  On any failure the temporary file is
/// removed and the original bytes are untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeAtomically(path: Path, contents: Uint8Array): Promise<void>;
/// ```
pub fn write_atomically(path: &Path, contents: &[u8]) -> Result<(), WriteError> {
    let target: PathBuf = match std::fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(error) => return Err(failure("resolve", path, &error)),
    };
    let original: Metadata = match std::fs::metadata(&target) {
        Ok(metadata) => metadata,
        Err(error) => return Err(failure("inspect", &target, &error)),
    };
    let (Some(directory), Some(name)) = (target.parent(), target.file_name()) else {
        return Err(WriteError {
            message: format!(
                "Cannot replace {}: it has no parent directory.",
                target.display()
            ),
        });
    };
    // Relaxed ordering still hands every caller a distinct number; no other memory depends on it.
    let sequence: u64 = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut temporary_name: OsString = OsString::from(".");
    temporary_name.push(name);
    temporary_name.push(format!(
        ".monochromatic-lint-{}-{sequence}.tmp",
        std::process::id()
    ));
    let temporary: PathBuf = directory.join(temporary_name);
    // A creation failure returns here: nothing was created, so there is nothing of ours to remove.
    let mut file: File = create_temporary(&temporary)?;
    let mut outcome: Result<(), WriteError> =
        fill_temporary(&mut file, &temporary, contents, &original);
    // Close the handle before renaming; some platforms refuse to rename an open file.
    drop(file);
    if outcome.is_ok()
        && let Err(error) = std::fs::rename(&temporary, &target)
    {
        outcome = Err(failure("rename temporary file over", &target, &error));
    }
    if let Err(first) = &outcome
        && let Err(cleanup) = std::fs::remove_file(&temporary)
    {
        return Err(WriteError {
            message: format!(
                "{} Its temporary file {} could not be removed: {cleanup}.",
                first.message,
                temporary.display()
            ),
        });
    }
    return outcome;
}

/// Mode,
///  symlink and failure-cleanup controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_write_tests.rs"]
mod tests;

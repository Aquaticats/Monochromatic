//! Unpack an embedded parser library into the private cache,
//!  exactly once per content.
//!
//! What:
//!  A parser library (a shared object) must be a file on disk,
//!  because the operating system's
//!       dynamic loader opens libraries by path.
//!  [`unpack`] writes one embedded library into
//!       `$XDG_CACHE_HOME/monochromatic-ide/runtime/<key>/grammars/<name>.so` and returns that path.
//! Why:
//!  The application is one executable;
//!  everything it needs at run time comes from inside it.
//!      Queries are read straight from the executable and never touch the disk;
//!  only libraries do.
//!
//! The rules that keep the cache safe:
//! - The cached file is compared byte for byte with the embedded bytes every time a library is
//!   loaded.
//!    A missing,
//!    shortened,
//!    or altered file is written again;
//!    it is never loaded as found.
//! - A write goes to a private file named after this process and a counter,
//!    then is renamed over the
//!   final name.
//!    A rename within one directory is atomic,
//!    so another process or thread opening the
//!   final name sees either the complete old file or the complete new one,
//!    never a partial write.
//!   Two first starts racing each other both write identical bytes;
//!    whichever rename lands last wins.
//! - Directories are created readable only by the user (mode 0700),
//!    files mode 0600.
//! - `<key>` is a digest of every embedded file,
//!    so builds with different files never share a cache.

/// Errors name the cache directory and the remedy.
use anyhow::Result;
/// Files,
///  the private file and directory modes,
///  the write counter,
///  and timing for logs.
use std::{
    fs,
    io::{ErrorKind, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

/// What:
///  A process-wide counter.
///  `AtomicU64` is a 64-bit integer that several threads may increment
///       without a lock (siblings:
///  `u64`,
///  a plain integer,
///  and `Mutex<u64>`,
///  one behind a lock).
/// Why:
///  Threads of one process share a process id,
///  so each private write file also takes a number
///      no other write in this process uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let nextWrite = 0; // JS has one thread per isolate, so no atomic is needed there
/// ```
static NEXT_WRITE: AtomicU64 = AtomicU64::new(0);

/// What:
///  Write `bytes` to a new private file beside `target`,
///  then rename it over `target`.
///       `&Path` is a borrowed path (sibling `PathBuf`,
///  an owned one).
/// Why:
///  Kept separate so a failed write can remove its private file in one place;
///  the retention
///      module writes its use marker through it as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceAtomically(directory: string, target: string, bytes: Uint8Array): void
/// ```
pub(super) fn replace_atomically(
    directory: &Path,
    target: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    // What: `fetch_add(1, Ordering::Relaxed)` returns the counter and adds one in a single step;
    //       `Relaxed` asks only for uniqueness, not for ordering with other memory.
    // Why: Every write in this process gets its own number.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const number = nextWrite++;
    // ```
    let number = NEXT_WRITE.fetch_add(1, Ordering::Relaxed);
    let name = match target.file_name() {
        Some(found) => found.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let private = directory.join(format!(".{name}.{}-{number}.partial", std::process::id()));
    // What: `OpenOptions` builds an open call: `create_new` fails if the file exists, `mode(0o600)`
    //       makes it readable and writable by the user only. `?` returns an I/O error to the caller.
    // Why: A leftover file from a crashed run is never reused or written through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const file = fs.openSync(privatePath, 'wx', 0o600);
    // ```
    let opened = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&private);
    // The bytes are written and the file renamed into place only while every earlier step succeeded.
    let written = match opened {
        Ok(mut file) => match file.write_all(bytes) {
            Ok(()) => fs::rename(&private, target),
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    };
    // What: `if let Err(error) = &written` runs the block only for a failed write; `&` lends the result.
    // Why: A failed write must not leave its private file behind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (failed) { try { fs.rmSync(privatePath); } catch (cleanup) { log.debug(cleanup); } }
    // ```
    if let Err(error) = &written {
        tracing::warn!(path = %private.display(), %error, "language parser write failed; removing its partial file");
        if let Err(cleanup) = fs::remove_file(&private) {
            tracing::debug!(path = %private.display(), error = %cleanup, "partial language parser file was not removed");
        }
    }
    return written;
}

/// What:
///  Return the path of a cached copy of `bytes` named `<name>.so` in `directory`,
///  writing it
///       first when the cached file is absent or differs.
///  `Result<PathBuf>` is the path or an error.
/// Why:
///  The dynamic loader needs a file;
///  the byte comparison is the integrity check of the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unpack(directory: string, name: string, bytes: Uint8Array): string // throws on failure
/// ```
pub fn unpack(directory: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf> {
    let started = Instant::now();
    let target = directory.join(format!("{name}.so"));
    // What: `match` on `fs::read(...)`: `Ok(found)` is the whole file, `Err(error)` an I/O failure;
    //       `if found == bytes` compares every byte.
    // Why: Only an identical file may be loaded; anything else is written again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let found: Uint8Array | undefined; try { found = fs.readFileSync(target); } catch (error) { ... }
    // ```
    match fs::read(&target) {
        Ok(found) if found == bytes => {
            tracing::debug!(path = %target.display(), bytes = bytes.len(), elapsed_us = started.elapsed().as_micros(), "cached language parser matches the embedded one");
            // `Ok(...)` wraps the path as the success value.
            return Ok(target);
        }
        Ok(found) => {
            tracing::warn!(path = %target.display(), found = found.len(), expected = bytes.len(), "cached language parser differs from the embedded one; writing it again");
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            tracing::debug!(path = %target.display(), "language parser is not cached yet");
        }
        Err(error) => {
            tracing::warn!(path = %target.display(), %error, "cached language parser cannot be read; writing it again");
        }
    }
    // What: `DirBuilder` creates the directory and missing parents (`recursive`) with mode 0700.
    //       `.context(...)` attaches a message to a failure, built only when it fails.
    // Why: Unpacked libraries are code the application loads; only the user should be able to replace them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fs.mkdirSync(directory, { recursive: true, mode: 0o700 });
    // ```
    if let Err(error) = fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)
    {
        return Err(anyhow::Error::new(error).context(remedy(directory, name)));
    }
    if let Err(error) = replace_atomically(directory, &target, bytes) {
        return Err(anyhow::Error::new(error).context(remedy(directory, name)));
    }
    tracing::info!(path = %target.display(), bytes = bytes.len(), elapsed_us = started.elapsed().as_micros(), "language parser unpacked into the private cache");
    return Ok(target);
}

/// What:
///  The message for a cache that cannot be written,
///  naming the directory and every way out.
/// Why:
///  The application shows this under the source;
///  the reader must know it is a cache problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remedy(directory: string, name: string): string
/// ```
fn remedy(directory: &Path, name: &str) -> String {
    return format!(
        "Cannot unpack the language parser {name} into the private cache {}. Highlighting needs that directory to be writable: free disk space or fix its permissions, or point XDG_CACHE_HOME at a writable directory, then restart the application. Source remains readable without coloring.",
        directory.display()
    );
}

/// First use,
///  reuse,
///  damage,
///  concurrency,
///  and an unwritable cache,
///  on disposable directories.
#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;

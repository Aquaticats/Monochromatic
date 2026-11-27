//! What:
//! Making names a file can be published under:
//! a directory chain whose every new entry is made durable,
//! an exclusively created file whose name nobody else holds,
//! the name part that keeps two calls apart,
//! and the error a directory full of temporary names answers with.
//! Why:
//! The replacement protocol publishes by exchanging a temporary file with its target,
//! so the temporary must live beside the target,
//! must never reuse another writer's name,
//! and the error of a full directory must say what it may hold.
//! Split out of `journal_replace.rs`;
//! the items are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal_replace_creates.ts: mkdir each missing ancestor and fsync it, open with O_EXCL, a unique suffix, the exhausted-name error
//! ```

/// What:
/// `use crate::journal_replace_preserve::sync_directory_and_parent;` imports the barrier that syncs a directory and then the directory that lists it.
/// Why:
/// A new directory is durable only when its own entry is on disk and so is the entry that names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { sync_directory_and_parent } from 'crate/journal_replace_preserve';
/// ```
use crate::journal_replace_preserve::sync_directory_and_parent;
/// What:
/// `use crate::journal_replace_unsynced::remember_unsynced;` imports the record of a directory whose creation was not made durable.
/// Why:
/// A directory whose sync failed is remembered,
/// so a later preservation knows it may not be durable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { remember_unsynced } from 'crate/journal_replace_unsynced';
/// ```
use crate::journal_replace_unsynced::remember_unsynced;
/// What:
/// `use std::fs::{self, File, OpenOptions};` imports the file-system module,
/// the open-file type and the builder that opens a file with chosen flags.
/// Why:
/// Creating a directory chain,
/// syncing a directory and creating a file exclusively all go through them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mkdirSync, openSync } from 'node:fs';
/// ```
use std::fs::{
    self,
    File,
    OpenOptions,
};
/// What:
/// `use std::io::{self, Write};` imports the error module and the trait that adds `write_all` to an open file.
/// Why:
/// The text of a new file is written through the descriptor that created it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node; writeSync is a free function
/// ```
use std::io::{
    self,
    Write,
};
/// What:
/// `use std::os::unix::fs::MetadataExt;` imports the trait that adds `ino` and `dev` to file metadata.
/// Why:
/// The identity of the created file is answered as its device and inode,
/// so a caller can tell its own file from another writer's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';
/// ```
use std::os::unix::fs::MetadataExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The calls take borrowed paths and answer the directories and files they created as owned ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::sync::atomic::{AtomicU64, Ordering};` imports a counter that threads can add to safely.
/// Why:
/// Temporary names combine the process id,
/// the clock and this counter,
/// so two replacements in one process never collide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let counter = 0; // single-threaded in Node
/// ```
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};
/// Counts the temporary names made by this process.
static TEMPORARY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What:
/// `fn sync_directory(directory: &Path) -> io::Result<()>` forces the directory entries of a directory to disk.
/// Why:
/// A rename,
/// a link or an unlink is durable only after the directory that holds the name is synchronized.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fd = openSync(directory, 'r'); fsyncSync(fd); closeSync(fd);
/// ```
pub fn sync_directory(directory: &Path) -> io::Result<()> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    return File::open(directory)?.sync_all();
}

/// What:
/// `fn parent_of(path: &Path) -> PathBuf` is the directory that holds a path (the current directory for a bare name).
/// Why:
/// The temporary file and the kept names are made beside the target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parentOf = (path: string): string => dirname(path);
/// ```
pub(crate) fn parent_of(path: &Path) -> PathBuf {
    // `match`: picks the arm whose pattern fits the value.
    return match path.parent() {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) if !found.as_os_str().is_empty() => found.to_path_buf(),
        // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
        // Why:      Joined or stored paths are owned; paths passed in are borrowed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        _ => PathBuf::from("."),
    };
}

/// What:
/// `pub fn file_name_text(path: &Path) -> String` is the last component of a path as text (empty when there is none).
/// Why:
/// The kept copies are named after the file they stand for,
/// and the messages name them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const fileNameText = (path: string): string => basename(path);
/// ```
pub fn file_name_text(path: &Path) -> String {
    return match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => String::new(),
    };
}

/// The most names tried when a kept file's name is taken,
/// and the most temporary names tried.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
/// Collisions were not counted;
/// the names carry the process id,
/// a nanosecond clock and a counter,
/// so one try is the normal case
/// and the cap only ends a search that something else is filling.
/// The number is a ceiling that a directory this tool does not fill cannot reach.
pub(crate) const MAX_NAME_ATTEMPTS: u32 = 10_000;

/// What:
/// `pub fn create_directories_with(path, sync_dir) -> io::Result<Vec<PathBuf>>` creates a directory and every missing ancestor,
/// syncing each new directory and the directory that now lists it with `sync_dir`,
/// and returns the directories it created.
/// Why:
/// Review finding (3b 6):
/// a file linked into a new directory is lost to a power failure when the new directory's own entry is
/// not durable;
/// `create_dir_all` alone makes no such promise.
/// A directory whose sync failed is remembered as unsynced (contract assumption b).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function createDirectoriesWith(path: string, syncDir: (p: string) => void): string[]  // mkdir each missing ancestor, fsync it and its parent
/// ```
pub fn create_directories_with(
    path: &Path,
    sync_dir: &dyn Fn(&Path) -> io::Result<()>,
) -> io::Result<Vec<PathBuf>> {
    let mut missing: Vec<PathBuf> = Vec::new();
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() || ancestor.exists() {
            break;
        }
        missing.push(ancestor.to_path_buf());
    }
    let mut created: Vec<PathBuf> = Vec::new();
    for directory in missing.iter().rev() {
        match fs::create_dir(directory) {
            Ok(()) => created.push(directory.clone()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        if let Err(error) = sync_directory_and_parent(sync_dir, directory) {
            remember_unsynced(directory);
            return Err(error);
        }
    }
    return Ok(created);
}

/// What:
/// `pub fn create_directories_synced(path: &Path) -> io::Result<()>` is `create_directories_with` with the real sync.
/// Why:
/// Most callers only need the directories to exist durably.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function createDirectoriesSynced(path: string): void
/// ```
pub fn create_directories_synced(path: &Path) -> io::Result<()> {
    return create_directories_with(path, &sync_directory).map(drop);
}

/// What:
/// `pub fn create_exclusive(directory, base, text, mode, next_name) -> io::Result<(PathBuf, u64, u64)>` writes `text` to a new file
/// that did not exist,
/// syncs it,
/// and returns its path,
/// device and inode.
/// Why:
/// An existing file of the chosen name is never truncated or reused:
/// the next name is tried,
/// up to `MAX_NAME_ATTEMPTS`,
/// then the call fails naming the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function createExclusive(directory: string, base: string, text: string, mode: number, nextName: () => string): [string, bigint, bigint]
/// ```
pub fn create_exclusive(
    directory: &Path,
    base: &str,
    text: &str,
    mode: u32,
    next_name: &mut dyn FnMut() -> String,
) -> io::Result<(PathBuf, u64, u64)> {
    /// Imports `std::os::unix::fs::OpenOptionsExt` for this file.
    use std::os::unix::fs::OpenOptionsExt;
    for _attempt in 0..MAX_NAME_ATTEMPTS {
        // Typed binding: `candidate: PathBuf`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let candidate: PathBuf = directory.join(format!(".{base}.shadow-{}.tmp", next_name()));
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&candidate`: lends the value without copying or giving it away.
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode & 0o777)
            .open(&candidate)
        {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(mut file) => {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                file.write_all(text.as_bytes())?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                #[cfg(test)]
                if crate::failpoints::fires(crate::failpoints::Point::PublicationFileSync) {
                    return Err(io::Error::other("injected file sync failure"));
                }
                file.sync_all()?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                let metadata = file.metadata()?;
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                return Ok((candidate, metadata.dev(), metadata.ino()));
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => return Err(error),
        }
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(io::Error::other(names_exhausted_message(directory, base)));
}

/// What:
/// `pub fn unique_suffix() -> String` is a name part that differs between calls and processes (pid,
/// clock,
/// counter).
/// Why:
/// Temporary names are tried with `create_new`,
/// so this only makes the first try succeed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const uniqueSuffix = (): string => `${process.pid}-${process.hrtime.bigint()}-${counter++}`;
/// ```
pub fn unique_suffix() -> String {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `nanos: u128`.
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(elapsed) => elapsed.as_nanos(),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => 0,
    };
    // Typed binding: `number: u64`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let number: u64 = TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!("{}-{nanos}-{number}", std::process::id());
}

/// What:
/// `pub(crate) fn create_temporary(target: &Path, text: &str, mode: u32) -> io::Result<(PathBuf, u64, u64)>` writes the replacement beside
/// the target.
/// Why:
/// The replacement is exchanged into place,
/// so it lives in the target's directory (the same file system).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createTemporary(target: string, text: string, mode: number): [string, bigint, bigint]
/// ```
pub(crate) fn create_temporary(
    target: &Path,
    text: &str,
    mode: u32,
) -> io::Result<(PathBuf, u64, u64)> {
    // Borrow `&parent_of`: lends the value without copying or giving it away.
    return create_exclusive(
        &parent_of(target),
        &file_name_text(target),
        text,
        mode,
        &mut unique_suffix,
    );
}

/// What:
/// `pub(crate) fn names_exhausted_message(directory: &Path, base: &str) -> String` is the error for a directory whose temporary names are used up.
/// Why:
/// The files may be the only names of another writer's data,
/// so the text says what keeps the data and what loses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesExhaustedMessage(directory: string, base: string): string
/// ```
pub(crate) fn names_exhausted_message(
    directory: &Path,
    base: &str,
) -> String {
    return format!(
        "{} holds {MAX_NAME_ATTEMPTS} temporary files named after {base}; those files can be the only names of data another writer produced, so keep them. Renaming one inside the same file system keeps its data; copying one elsewhere and deleting the original does not, because a program that still has the original open keeps writing to it. Close every program that may have one open before you move or remove any, then run the command again",
        directory.display()
    );
}

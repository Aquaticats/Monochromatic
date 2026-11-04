//! What:
//! The replacement of one file that keeps every file it displaces:
//! the new text is written to a uniquely named sibling file,
//! exchanged with the file in one step (`renameat2` with `RENAME_EXCHANGE`),
//! the displaced file is checked through an open descriptor,
//! and the exchange is undone when it was not the file expected.
//! Whatever is displaced is given a surviving name (a hard link) before any other name for it is removed.
//! Why:
//! The second review (2026-11-03) showed that a comparison before or after an exchange cannot make a replacement conditional against
//! writers that hold the file open or publish at any moment.
//! The contract here is therefore preservation,
//! not atomic conditional replacement:
//! no byte that was in the file when the exchange happened is ever left without a name,
//! the file a concurrent writer published is put back (or kept under a reported name),
//! and a filesystem that cannot exchange makes the restore fail visibly instead of falling back to a rename that overwrites.
//! Two concurrent publishers can still end up in the other order than they ran;
//! the loser is kept and reported.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-replace.ts
//! export function replaceIfHolds(path: string, expected: string, newText: string, between: (path: string) => void): ReplaceOutcome
//! ```

/// What:
/// `use std::fs::{self, File, OpenOptions};` imports the file-system module,
/// the open-file type and the builder that opens a file with chosen flags.
/// Why:
/// The replacement is created exclusively (`create_new`),
/// written,
/// synchronized and renamed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { openSync, writeSync, fsyncSync, linkSync, unlinkSync } from 'node:fs';
/// ```
use std::fs::{self, File, OpenOptions};
/// What:
/// `use std::io::{self, Write};` imports the error module and the trait that adds `write_all` to an open file.
/// Why:
/// The replacement text is written through the descriptor that created the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node; writeSync is a free function
/// ```
use std::io::{self, Write};
/// What:
/// `use std::os::unix::fs::MetadataExt;` imports the trait that adds `ino` and `dev` to file metadata.
/// Why:
/// The replacement's own inode is remembered so that the undo can tell it from another writer's file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';
/// ```
use std::os::unix::fs::MetadataExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// Paths are lent to the functions and the kept names are returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{Path, PathBuf};
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
use std::sync::atomic::{AtomicU64, Ordering};
/// What:
/// `use rustix::fs::{CWD, RenameFlags, renameat_with};` imports the safe wrapper of `renameat2`:
/// the flags,
/// and the current directory as the base of relative names.
/// Why:
/// The exchange needs `RENAME_EXCHANGE` and the put-back needs `RENAME_NOREPLACE`,
/// which the standard library does not expose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no Node API; the kernel call is renameat2
/// ```
use rustix::fs::{CWD, RenameFlags, renameat_with};
/// What:
/// `use rustix::io::Errno;` imports the operating-system error code type.
/// Why:
/// "The filesystem cannot exchange" and "the target vanished" are told apart by code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // error.code === 'ENOENT'
/// ```
use rustix::io::Errno;

/// What:
/// `use crate::file_owner::observe;` imports the read that separates absence from failure.
/// Why:
/// The first comparison must not take an unreadable file for an empty one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { observe } from 'crate/file_owner';
/// ```
use crate::file_owner::observe;

/// Counts the temporary names made by this process.
static TEMPORARY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What a compare-and-replace did to one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplaceOutcome {
    /// The file held what was expected and now holds the new text;
    /// the displaced file was kept under the name inside.
    Replaced(
        /// Where the displaced file was kept (always `Some` unless the target had vanished and nothing was displaced).
        Option<PathBuf>,
    ),
    /// The file held something else,
    /// or held it only for a moment and was put back:
    /// another writer's file is where it was.
    LeftAlone,
    /// The snapshot is live and the file it displaced is kept at this path
    /// (it was not exactly the expected one,
    /// or could only be kept under its temporary name).
    Displaced(
        /// Where the displaced file is kept.
        PathBuf,
    ),
    /// The exchange was undone:
    /// another writer's file is live and the replacement inode is kept at this path.
    Undone(
        /// Where the replacement is kept.
        PathBuf,
    ),
}

/// Where and under what suffix a displaced file is kept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeepAs {
    /// The directory the kept names are made in;
    /// `None` means next to the target.
    pub directory: Option<PathBuf>,
    /// The suffix after the target's file name (`.displaced`,
    /// `.refused`).
    pub suffix: String,
}

/// The constructors of the places a displaced file is kept.
impl KeepAs {
    /// What:
    /// `pub fn next_to_target(suffix: &str) -> KeepAs` keeps displaced files in the target's own directory.
    /// Why:
    /// A refused lockfile saved by a recovery sits beside the lockfile,
    /// where the person looks for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static nextToTarget(suffix: string): KeepAs
    /// ```
    pub fn next_to_target(suffix: &str) -> KeepAs {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        return KeepAs { directory: None, suffix: suffix.to_string() };
    }

    /// What:
    /// `pub fn in_directory(directory: &Path, suffix: &str) -> KeepAs` keeps displaced files in one directory
    /// (the quarantine under `.cache/cargo-shadow/`).
    /// Why:
    /// An ordinary rollback of this run's own output should not leave files in the working tree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static inDirectory(directory: string, suffix: string): KeepAs
    /// ```
    pub fn in_directory(directory: &Path, suffix: &str) -> KeepAs {
        // `Some`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return KeepAs { directory: Some(directory.to_path_buf()), suffix: suffix.to_string() };
    }
}

/// What:
/// `fn exchange(first: &Path, second: &Path) -> io::Result<()>` swaps two paths in one step (`renameat2` with `RENAME_EXCHANGE`).
/// Why:
/// After the call `first` names what `second` named and the other way round,
/// with no moment at which either path is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exchange(first: string, second: string): void  // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
/// ```
pub fn exchange_paths(first: &Path, second: &Path) -> io::Result<()> {
    return exchange(first, second);
}

/// The exchange primitive,
/// private to the protocol.
fn exchange(first: &Path, second: &Path) -> io::Result<()> {
    // `match`: picks the arm whose pattern fits the value.
    return match renameat_with(CWD, first, CWD, second, RenameFlags::EXCHANGE) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(()) => Ok(()),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(code) if code == Errno::INVAL || code == Errno::NOSYS || code == Errno::OPNOTSUPP => {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            Err(io::Error::new(io::ErrorKind::Unsupported, format!("this filesystem cannot exchange {} and {} atomically ({code})", first.display(), second.display())))
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(code) => Err(io::Error::from(code)),
    };
}

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
fn parent_of(path: &Path) -> PathBuf {
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
/// `pub fn no_hook(_path: &Path)` is the hook that does nothing.
/// Why:
/// The real callers have no gap to exercise;
/// a named function passes as `&dyn Fn(&Path)` where a closure would be flagged as avoidable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const noHook = (_path: string): void => {};
/// ```
pub fn no_hook(_path: &Path) {}

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

/// What:
/// `pub fn replace_if_holds(path, expected, new_text, between) -> io::Result<ReplaceOutcome>` puts `new_text` in `path` when `path`
/// holds `expected`,
/// keeping the displaced file next to the target as `<name>.displaced`.
/// Why:
/// The plain form for the callers and tests that have no keep directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHolds(path: string, expected: string, newText: string, between: (path: string) => void): ReplaceOutcome
/// ```
pub fn replace_if_holds(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    // Borrow `&KeepAs`: lends the value without copying or giving it away.
    return replace_core(path, expected, new_text, &KeepAs::next_to_target(".displaced"), between, &no_hook, &Ops::standard());
}

/// What:
/// `pub fn replace_if_holds_with_hooks(path, expected, new_text, between, after_exchange)` is `replace_if_holds` with a second hook
/// that runs right after the exchange,
/// before the displaced file is read.
/// Why:
/// The second gap is where a writer publishes between the swap and the swap back;
/// the hook lets a test publish one there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHoldsWithHooks(path, expected, newText, between, afterExchange): ReplaceOutcome
/// ```
pub fn replace_if_holds_with_hooks(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path), after_exchange: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    // Borrow `&KeepAs`: lends the value without copying or giving it away.
    return replace_core(path, expected, new_text, &KeepAs::next_to_target(".displaced"), between, after_exchange, &Ops::standard());
}

/// What:
/// `pub fn replace_keeping(path, expected, new_text, keep) -> io::Result<ReplaceOutcome>` is the production form:
/// the displaced file is kept as `keep` says.
/// Why:
/// Rollbacks keep their displaced output in the quarantine;
/// recoveries keep it beside the lockfile as `.refused`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceKeeping(path: string, expected: string, newText: string, keep: KeepAs): ReplaceOutcome
/// ```
pub fn replace_keeping(path: &Path, expected: &str, new_text: &str, keep: &KeepAs) -> io::Result<ReplaceOutcome> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return replace_core(path, expected, new_text, keep, &no_hook, &no_hook, &Ops::standard());
}

/// What:
/// `pub fn replace_keeping_with_hook(path, expected, new_text, keep, between)` is `replace_keeping` with the first hook.
/// Why:
/// The recovery tests publish a file exactly between the last comparison and the exchange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceKeepingWithHook(path, expected, newText, keep, between): ReplaceOutcome
/// ```
pub fn replace_keeping_with_hook(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, between: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return replace_core(path, expected, new_text, keep, between, &no_hook, &Ops::standard());
}

/// What:
/// `pub fn replace_with_swap(path, expected, new_text, keep, swap)` is the protocol with the swap primitive chosen by the caller.
/// Why:
/// A test passes a swap that fails with the error a filesystem without exchange gives,
/// to show that the restore fails visibly
/// and overwrites nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceWithSwap(path, expected, newText, keep, swap): ReplaceOutcome
/// ```
pub fn replace_with_swap(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, swap: &dyn Fn(&Path, &Path) -> io::Result<()>) -> io::Result<ReplaceOutcome> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return replace_core(path, expected, new_text, keep, &no_hook, &no_hook, &Ops { swap, ..Ops::standard() });
}

/// What:
/// `fn real_link(existing: &Path, new: &Path) -> io::Result<()>` is the hard link the protocol keeps displaced files with.
/// Why:
/// It is a named function so tests can swap in a failing one through `Seams`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const realLink = (existing: string, name: string): void => linkSync(existing, name);
/// ```
fn real_link(existing: &Path, new: &Path) -> io::Result<()> {
    return fs::hard_link(existing, new);
}

/// The points of the protocol where a test can intervene or inject a failure.
pub struct Seams<'a> {
    /// Runs after the first comparison and before the temporary file is created.
    pub between: &'a dyn Fn(&Path),
    /// Runs right after the first exchange,
    /// with the live path.
    pub after_exchange: &'a dyn Fn(&Path),
    /// The exchange itself.
    pub swap: &'a dyn Fn(&Path, &Path) -> io::Result<()>,
    /// The hard link that gives a displaced file a surviving name.
    pub link: &'a dyn Fn(&Path, &Path) -> io::Result<()>,
    /// The barrier that makes a file's data durable.
    pub sync_file: &'a dyn Fn(&Path) -> io::Result<()>,
    /// The barrier that makes a directory's entries durable.
    pub sync_dir: &'a dyn Fn(&Path) -> io::Result<()>,
}

/// The most names tried when a kept file's name is taken,
/// and the most temporary names tried.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open; not a measurement.
/// Collisions were not counted;
/// the names carry the process id,
/// a nanosecond clock and a counter,
/// so one try is the normal case
/// and the cap only ends a search that something else is filling.
/// The number is a ceiling that a directory this tool does not fill cannot reach.
const MAX_NAME_ATTEMPTS: u32 = 10_000;

/// The directories this process knows it left unsynchronized because a sync after their creation failed.
/// Contract assumption b: such a directory is not treated as durable merely because it exists now.
/// The registry lives as long as the process;
/// across processes a directory left unsynced is a pre-existing directory and assumption b applies to it.
static UNSYNCED: std::sync::Mutex<std::collections::BTreeSet<PathBuf>> = std::sync::Mutex::new(std::collections::BTreeSet::new());

/// What:
/// `fn remember_unsynced(directory: &Path)` records a directory whose creation was not made durable.
/// Why:
/// The walk of a later preservation continues through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rememberUnsynced(directory: string): void
/// ```
fn remember_unsynced(directory: &Path) {
    if let Ok(mut known) = UNSYNCED.lock() {
        known.insert(fs::canonicalize(directory).unwrap_or_else(|_| { return directory.to_path_buf(); }));
    }
}

/// What:
/// `fn forget_unsynced(directory: &Path)` removes a directory from the registry once its entry has been synchronized.
/// Why:
/// A later preservation may stop at it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetUnsynced(directory: string): void
/// ```
fn forget_unsynced(directory: &Path) {
    if let Ok(mut known) = UNSYNCED.lock() {
        known.remove(directory);
    }
}

/// What:
/// `fn is_unsynced(directory: &Path) -> bool` says whether this process knows it left the directory unsynchronized.
/// Why:
/// An unreadable registry counts as "unsynced": the safe answer continues the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isUnsynced(directory: string): boolean
/// ```
fn is_unsynced(directory: &Path) -> bool {
    return UNSYNCED.lock().map(|known| { return known.contains(directory); }).unwrap_or(true);
}

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
pub fn create_directories_with(path: &Path, sync_dir: &dyn Fn(&Path) -> io::Result<()>) -> io::Result<Vec<PathBuf>> {
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
        if let Err(error) = sync_dir(directory).and_then(|()| { return sync_dir(&parent_of(directory)); }) {
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
    return create_directories_with(path, &sync_directory).map(|_created| {});
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
pub fn create_exclusive(directory: &Path, base: &str, text: &str, mode: u32, next_name: &mut dyn FnMut() -> String) -> io::Result<(PathBuf, u64, u64)> {
    /// Imports `std::os::unix::fs::OpenOptionsExt` for this file.
    use std::os::unix::fs::OpenOptionsExt;
    for _attempt in 0..MAX_NAME_ATTEMPTS {
        // Typed binding: `candidate: PathBuf`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let candidate: PathBuf = directory.join(format!(".{base}.shadow-{}.tmp", next_name()));
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&candidate`: lends the value without copying or giving it away.
        match OpenOptions::new().write(true).create_new(true).mode(mode & 0o777).open(&candidate) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(mut file) => {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                file.write_all(text.as_bytes())?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
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
/// `fn create_temporary(target: &Path, text: &str, mode: u32) -> io::Result<(PathBuf, u64, u64)>` writes the replacement beside
/// the target.
/// Why:
/// The replacement is exchanged into place,
/// so it lives in the target's directory (the same file system).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createTemporary(target: string, text: string, mode: number): [string, bigint, bigint]
/// ```
fn create_temporary(target: &Path, text: &str, mode: u32) -> io::Result<(PathBuf, u64, u64)> {
    // Borrow `&parent_of`: lends the value without copying or giving it away.
    return create_exclusive(&parent_of(target), &file_name_text(target), text, mode, &mut unique_suffix);
}

/// The operations the protocol is built from,
/// so a test can fail or watch each one.
pub struct Ops<'a> {
    /// The exchange itself.
    pub swap: &'a dyn Fn(&Path, &Path) -> io::Result<()>,
    /// The hard link that gives a displaced file a surviving name.
    pub link: &'a dyn Fn(&Path, &Path) -> io::Result<()>,
    /// The barrier that makes a file's data durable.
    pub sync_file: &'a dyn Fn(&Path) -> io::Result<()>,
    /// The barrier that makes a directory's entries durable.
    pub sync_dir: &'a dyn Fn(&Path) -> io::Result<()>,
}

/// What:
/// `fn sync_file_data(path: &Path) -> io::Result<()>` makes a file's data durable.
/// Why:
/// It is the real data barrier of `Ops`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const syncFileData = (path: string): void => fsyncSync(openSync(path, 'r'));
/// ```
fn sync_file_data(path: &Path) -> io::Result<()> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    return File::open(path)?.sync_all();
}

/// The methods of the type above.
impl Ops<'static> {
    /// What:
    /// `fn standard() -> Ops<'static>` is the real exchange,
    /// link and barriers.
    /// Why:
    /// Production and most tests use them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static standard(): Ops
    /// ```
    fn standard() -> Ops<'static> {
        return Ops { swap: &exchange, link: &real_link, sync_file: &sync_file_data, sync_dir: &sync_directory };
    }
}

/// Which stage of preserving a file failed,
/// because the stages mean different things for the data.
enum PreserveFailure {
    /// The data barrier of the displaced file failed:
    /// nothing may be called settled.
    Data(
        /// The system error.
        io::Error,
    ),
    /// No name could be linked;
    /// the file keeps its temporary name,
    /// whose data was synced.
    Link(
        /// The system error.
        io::Error,
    ),
    /// The file was linked under this kept name,
    /// but a directory barrier failed;
    /// both names stay.
    Durability {
        /// The kept name.
        kept: PathBuf,
        /// The system error.
        error: io::Error,
    },
}

/// What:
/// `fn preserve(displaced, target, keep, ops) -> Result<PathBuf, PreserveFailure>` gives a file a surviving name:
/// its data is synced,
/// then it is hard-linked under the first free name in the kept directory,
/// or beside the target,
/// and the kept directory and its two parents are synced.
/// Why:
/// A hard link keeps the inode,
/// so a write through a descriptor opened earlier still lands in the kept file.
/// Review findings (4b 1,
/// 4b 2):
/// "exists" is not a durability certificate,
/// so the directory chain is synced every time;
/// a failed data barrier is its own failure and is never folded into a settled result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function preserve(displaced: string, target: string, keep: KeepAs): string  // throws a typed failure
/// ```
fn preserve(displaced: &Path, target: &Path, keep: &KeepAs, ops: &Ops) -> Result<PathBuf, PreserveFailure> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    (ops.sync_file)(displaced).map_err(PreserveFailure::Data)?;
    let base: String = file_name_text(target);
    let mut directories: Vec<PathBuf> = Vec::new();
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Borrow `&keep.directory`: lends the value without copying or giving it away.
    let mut created: Vec<PathBuf> = Vec::new();
    if let Some(chosen) = &keep.directory
        && let Ok(made) = create_directories_with(chosen, ops.sync_dir)
    {
        created = made;
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        directories.push(chosen.clone());
    }
    directories.push(parent_of(target));
    let mut last_error: io::Error = io::Error::other("no directory to keep the displaced file in");
    for directory in directories {
        for number in 0..MAX_NAME_ATTEMPTS {
            let name: String = if number == 0 { format!("{base}{}", keep.suffix) } else { format!("{base}{}.{number}", keep.suffix) };
            let candidate: PathBuf = directory.join(name);
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&candidate`: lends the value without copying or giving it away.
            match (ops.link)(displaced, &candidate) {
                Ok(()) => {
                    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
                    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // (a, b) => body
                    // ```
                    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
                    // Why:      They express one step of a pipeline without an index loop.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // list.map(f).filter(g)
                    // ```
                    // The entries of the kept copy are made durable from the kept directory up to the first directory that existed before
                    // this transaction, and the walk stops there (contract assumption b): a pre-existing directory and its path are
                    // assumed durable. A directory this call created, or one this process knows it left unsynchronized, is not.
                    // The path is resolved first, so a link or a relative path names the directories that really hold the entries.
                    // The walk fails, and preservation is not reported, when any barrier fails.
                    let durable_up_to_the_anchor = || -> io::Result<()> {
                        let resolved: PathBuf = fs::canonicalize(&directory)?;
                        let created_here: Vec<PathBuf> = created.iter().filter_map(|made| { return fs::canonicalize(made).ok(); }).collect();
                        let mut current: PathBuf = resolved;
                        let mut child: Option<PathBuf> = None;
                        loop {
                            (ops.sync_dir)(&current)?;
                            if let Some(synced_child) = child.take() {
                                forget_unsynced(&synced_child);
                            }
                            if !created_here.contains(&current) && !is_unsynced(&current) {
                                return Ok(());
                            }
                            let Some(parent) = current.parent() else {
                                return Ok(());
                            };
                            child = Some(current.clone());
                            current = parent.to_path_buf();
                        }
                    };
                    durable_up_to_the_anchor().map_err(|error| { return PreserveFailure::Durability { kept: candidate.clone(), error }; })?;
                    return Ok(candidate);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    last_error = error;
                    break;
                }
            }
        }
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    return Err(PreserveFailure::Link(last_error));
}

/// What:
/// `pub fn keep_name(displaced, target, keep) -> io::Result<PathBuf>` gives a file a surviving name under the kept directory
/// (or beside the target),
/// exactly as the replace protocol does for a displaced file.
/// Why:
/// Other code that removes a name (the stand-in lockfile) must first make sure the inode keeps another one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function keepName(displaced: string, target: string, keep: KeepAs): string
/// ```
pub fn keep_name(displaced: &Path, target: &Path, keep: &KeepAs) -> io::Result<PathBuf> {
    // Closure `|failure|`: an anonymous function, like a TS arrow function.
    // Borrow `&Ops`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    return preserve(displaced, target, keep, &Ops::standard()).map_err(|failure| {
        // `match`: picks the arm whose pattern fits the value.
        return match failure {
            PreserveFailure::Data(error) | PreserveFailure::Link(error) => error,
            // Macro `format!`: builds or prints text (or a list) from a template.
            PreserveFailure::Durability { kept, error } => io::Error::other(format!("kept as {} but not made durable: {error}", kept.display())),
        };
    });
}

/// What:
/// `fn keep_and_unlink(name, target, keep, ops) -> io::Result<Result<PathBuf, PathBuf>>` keeps the file at `name` under a
/// surviving name and then removes `name`;
/// the answer is the kept path,
/// or `name` itself when no name could be linked
/// (it is then left where it is,
/// with its data synced).
/// Why:
/// This is the only place the protocol removes a name,
/// and only after the inode has another durable one.
/// Every error says where the file is,
/// so the caller can report it (review finding 4b 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepAndUnlink(name: string, target: string, keep: KeepAs): { kept: string } | { left: string }  // throws with the paths in the message
/// ```
fn keep_and_unlink(name: &Path, target: &Path, keep: &KeepAs, ops: &Ops) -> io::Result<Result<PathBuf, PathBuf>> {
    let parent: PathBuf = parent_of(target);
    // `match`: picks the arm whose pattern fits the value.
    return match preserve(name, target, keep, ops) {
        Ok(kept) => {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Macro `format!`: builds or prints text (or a list) from a template.
            fs::remove_file(name).map_err(|error| { return io::Error::other(format!("{} is kept as {} but its temporary name {} could not be removed: {error}", target.display(), kept.display(), name.display())); })?;
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&parent`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Macro `format!`: builds or prints text (or a list) from a template.
            (ops.sync_dir)(&parent).map_err(|error| { return io::Error::other(format!("{} is kept as {} but {} could not be synced: {error}", target.display(), kept.display(), parent.display())); })?;
            Ok(Ok(kept))
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(PreserveFailure::Link(_)) => {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&parent`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Macro `format!`: builds or prints text (or a list) from a template.
            (ops.sync_dir)(&parent).map_err(|error| { return io::Error::other(format!("the displaced file stays as {} but {} could not be synced: {error}", name.display(), parent.display())); })?;
            Ok(Err(name.to_path_buf()))
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(PreserveFailure::Data(error)) => Err(io::Error::other(format!("the data of the displaced file could not be made durable, so nothing is settled; it stays as {}: {error}", name.display()))),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(PreserveFailure::Durability { kept, error }) => Err(io::Error::other(format!("the displaced file is kept as {} (and still as {}) but a directory could not be made durable: {error}", kept.display(), name.display()))),
    };
}

/// What:
/// `fn replace_core(...) -> io::Result<ReplaceOutcome>` is the protocol:
/// compare,
/// create an exclusive temporary,
/// exchange it with the target,
/// then keep whatever the exchange displaced.
/// Why:
/// The rule that makes the outcome independent of who wrote what:
/// an inode is unlinked only if this call created it and never
/// exposed it at the live path,
/// or after it has a durable second name.
/// A target that vanishes between the comparison and the exchange stays vanished (review finding 3b 10).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceCore(path: string, expected: string, newText: string, keep: KeepAs, hooks: Hooks, ops: Ops): ReplaceOutcome
/// ```
fn replace_core(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, between: &dyn Fn(&Path), after_exchange: &dyn Fn(&Path), ops: &Ops) -> io::Result<ReplaceOutcome> {
    let first = match observe(path)? {
        Some(found) => found,
        None => return Ok(ReplaceOutcome::LeftAlone),
    };
    if first.text != expected {
        return Ok(ReplaceOutcome::LeftAlone);
    }
    between(path);
    let (temporary, _device, _inode) = create_temporary(path, new_text, first.mode)?;
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&temporary`: lends the value without copying or giving it away.
    if let Err(error) = (ops.swap)(&temporary, path) {
        // The temporary was never exposed: it is this call's alone and may be removed.
        let _ = fs::remove_file(&temporary);
        if error.kind() == io::ErrorKind::NotFound {
            return Ok(ReplaceOutcome::LeftAlone);
        }
        return Err(error);
    }
    after_exchange(path);
    let displaced_expected: bool = match observe(&temporary) {
        Ok(Some(seen)) => seen.text == expected,
        _ => false,
    };
    if displaced_expected {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&temporary`: lends the value without copying or giving it away.
        return Ok(match keep_and_unlink(&temporary, path, keep, ops)? {
            Ok(kept) => ReplaceOutcome::Replaced(Some(kept)),
            Err(left) => ReplaceOutcome::Displaced(left),
        });
    }
    // Borrow `&temporary`: lends the value without copying or giving it away.
    if (ops.swap)(&temporary, path).is_err() {
        // The replacement stays live and the unexpected file is at the temporary name.
        return Ok(ReplaceOutcome::Displaced(match keep_and_unlink(&temporary, path, keep, ops)? {
            Ok(kept) => kept,
            Err(left) => left,
        }));
    }
    // The exchange is undone: the other writer's file is live again, and the replacement (live for a moment, possibly open
    // elsewhere) is kept, not deleted.
    return Ok(ReplaceOutcome::Undone(match keep_and_unlink(&temporary, path, keep, ops)? {
        Ok(kept) => kept,
        Err(left) => left,
    }));
}

/// What:
/// `pub fn replace_with_seams(path, expected, new_text, keep, seams) -> io::Result<ReplaceOutcome>` is the protocol with every
/// seam given.
/// Why:
/// Tests drive the schedules the reviews describe (a failing link,
/// a failing barrier) deterministically.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceWithSeams(path: string, expected: string, newText: string, keep: KeepAs, seams: Seams): ReplaceOutcome
/// ```
pub fn replace_with_seams(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, seams: &Seams) -> io::Result<ReplaceOutcome> {
    // Typed binding: `ops: Ops`.
    let ops: Ops = Ops { swap: seams.swap, link: seams.link, sync_file: seams.sync_file, sync_dir: seams.sync_dir };
    // Borrow `&ops`: lends the value without copying or giving it away.
    return replace_core(path, expected, new_text, keep, seams.between, seams.after_exchange, &ops);
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
pub(crate) fn names_exhausted_message(directory: &Path, base: &str) -> String {
    return format!(
        "{} holds {MAX_NAME_ATTEMPTS} temporary files named after {base}; those files can be the only names of data another writer produced, so keep them. Renaming one inside the same file system keeps its data; copying one elsewhere and deleting the original does not, because a program that still has the original open keeps writing to it. Close every program that may have one open before you move or remove any, then run the command again",
        directory.display()
    );
}

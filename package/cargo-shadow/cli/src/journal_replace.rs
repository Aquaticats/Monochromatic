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
    /// (it was not exactly the expected one, or could only be kept under its temporary name).
    Displaced(
        /// Where the displaced file is kept.
        PathBuf,
    ),
    /// The exchange was undone: another writer's file is live and the replacement inode is kept at this path.
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

/// The exchange primitive, private to the protocol.
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
    return replace_core(path, expected, new_text, &KeepAs::next_to_target(".displaced"), between, &no_hook, &exchange, &real_link);
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
    return replace_core(path, expected, new_text, &KeepAs::next_to_target(".displaced"), between, after_exchange, &exchange, &real_link);
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
    return replace_core(path, expected, new_text, keep, &no_hook, &no_hook, &exchange, &real_link);
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
    return replace_core(path, expected, new_text, keep, between, &no_hook, &exchange, &real_link);
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
    return replace_core(path, expected, new_text, keep, &no_hook, &no_hook, swap, &real_link);
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
    /// Runs right after the first exchange, with the live path.
    pub after_exchange: &'a dyn Fn(&Path),
    /// The exchange itself.
    pub swap: &'a dyn Fn(&Path, &Path) -> io::Result<()>,
    /// The hard link that gives a displaced file a surviving name.
    pub link: &'a dyn Fn(&Path, &Path) -> io::Result<()>,
}

/// What:
/// `pub fn replace_with_seams(path, expected, new_text, keep, seams) -> io::Result<ReplaceOutcome>` is the protocol with every
/// seam given.
/// Why:
/// Tests drive the schedules the reviews describe (a failing link, an exchange that fails) deterministically.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceWithSeams(path: string, expected: string, newText: string, keep: KeepAs, seams: Seams): ReplaceOutcome
/// ```
pub fn replace_with_seams(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, seams: &Seams) -> io::Result<ReplaceOutcome> {
    return replace_core(path, expected, new_text, keep, seams.between, seams.after_exchange, seams.swap, seams.link);
}

/// The most names tried when a kept file's name is taken, and the most temporary names tried.
/// A bound chosen for the owner's rule of 2026-11-03 ("nothing should ever wait forever or indefinitely"):
/// far above any real number of kept copies, so reaching it means something else is filling the directory.
const MAX_NAME_ATTEMPTS: u32 = 10_000;

/// What:
/// `pub fn create_directories_synced(path: &Path) -> io::Result<()>` creates a directory and every missing ancestor,
/// syncing each new directory and the directory that now lists it.
/// Why:
/// Review finding (3b 6): a file linked into a new directory is lost to a power failure when the new directory's own entry is
/// not durable; `create_dir_all` alone makes no such promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function createDirectoriesSynced(path: string): void  // mkdir each missing ancestor, fsync it and its parent
/// ```
pub fn create_directories_synced(path: &Path) -> io::Result<()> {
    let mut missing: Vec<PathBuf> = Vec::new();
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() || ancestor.exists() {
            break;
        }
        missing.push(ancestor.to_path_buf());
    }
    for directory in missing.iter().rev() {
        match fs::create_dir(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        sync_directory(directory)?;
        sync_directory(&parent_of(directory))?;
    }
    return Ok(());
}

/// What:
/// `pub fn create_exclusive(directory, base, text, mode, next_name) -> io::Result<(PathBuf, u64, u64)>` writes `text` to a new file
/// that did not exist, syncs it, and returns its path, device and inode.
/// Why:
/// An existing file of the chosen name is never truncated or reused: the next name is tried,
/// up to `MAX_NAME_ATTEMPTS`, then the call fails naming the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function createExclusive(directory: string, base: string, text: string, mode: number, nextName: () => string): [string, bigint, bigint]
/// ```
pub fn create_exclusive(directory: &Path, base: &str, text: &str, mode: u32, next_name: &mut dyn FnMut() -> String) -> io::Result<(PathBuf, u64, u64)> {
    use std::os::unix::fs::OpenOptionsExt;
    for _attempt in 0..MAX_NAME_ATTEMPTS {
        let candidate: PathBuf = directory.join(format!(".{base}.shadow-{}.tmp", next_name()));
        match OpenOptions::new().write(true).create_new(true).mode(mode & 0o777).open(&candidate) {
            Ok(mut file) => {
                file.write_all(text.as_bytes())?;
                file.sync_all()?;
                let metadata = file.metadata()?;
                return Ok((candidate, metadata.dev(), metadata.ino()));
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    return Err(io::Error::other(format!(
        "{} holds {MAX_NAME_ATTEMPTS} temporary files named after {base}; remove the .{base}.shadow-*.tmp files there and run the command again",
        directory.display()
    )));
}

/// What:
/// `pub fn unique_suffix() -> String` is a name part that differs between calls and processes (pid, clock, counter).
/// Why:
/// Temporary names are tried with `create_new`, so this only makes the first try succeed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const uniqueSuffix = (): string => `${process.pid}-${process.hrtime.bigint()}-${counter++}`;
/// ```
pub fn unique_suffix() -> String {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_) => 0,
    };
    let number: u64 = TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
    return format!("{}-{nanos}-{number}", std::process::id());
}

/// What:
/// `fn create_temporary(target: &Path, text: &str, mode: u32) -> io::Result<(PathBuf, u64, u64)>` writes the replacement beside
/// the target.
/// Why:
/// The replacement is exchanged into place, so it lives in the target's directory (the same file system).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createTemporary(target: string, text: string, mode: number): [string, bigint, bigint]
/// ```
fn create_temporary(target: &Path, text: &str, mode: u32) -> io::Result<(PathBuf, u64, u64)> {
    return create_exclusive(&parent_of(target), &file_name_text(target), text, mode, &mut unique_suffix);
}

/// What:
/// `fn preserve(displaced, target, keep, link) -> io::Result<PathBuf>` gives a file a surviving name:
/// its data is synced, then it is hard-linked under the first free name in the kept directory, or beside the target.
/// Why:
/// A hard link keeps the inode, so a write through a descriptor opened earlier still lands in the kept file.
/// Review finding (3b 6): the data is made durable before the name is relied on, new directories are synced with their parents,
/// and the name search is bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function preserve(displaced: string, target: string, keep: KeepAs): string  // fsync, then linkSync per candidate name
/// ```
fn preserve(displaced: &Path, target: &Path, keep: &KeepAs, link: &dyn Fn(&Path, &Path) -> io::Result<()>) -> io::Result<PathBuf> {
    File::open(displaced)?.sync_all()?;
    let base: String = file_name_text(target);
    let mut directories: Vec<PathBuf> = Vec::new();
    if let Some(chosen) = &keep.directory {
        match create_directories_synced(chosen) {
            Ok(()) => directories.push(chosen.clone()),
            Err(_) => {}
        }
    }
    directories.push(parent_of(target));
    let mut last_error: io::Error = io::Error::other("no directory to keep the displaced file in");
    for directory in directories {
        for number in 0..MAX_NAME_ATTEMPTS {
            let name: String = if number == 0 { format!("{base}{}", keep.suffix) } else { format!("{base}{}.{number}", keep.suffix) };
            let candidate: PathBuf = directory.join(name);
            match link(displaced, &candidate) {
                Ok(()) => {
                    sync_directory(&directory)?;
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
    return Err(last_error);
}

/// What:
/// `fn keep_and_unlink(name, target, keep, link) -> io::Result<Result<PathBuf, PathBuf>>` keeps the file at `name` under a
/// surviving name and then removes `name`; the answer is the kept path, or `name` itself when it could not be kept
/// (it is then left where it is).
/// Why:
/// This is the only place the protocol removes a name, and only after the inode has another name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepAndUnlink(name: string, target: string, keep: KeepAs): { kept: string } | { left: string }
/// ```
fn keep_and_unlink(name: &Path, target: &Path, keep: &KeepAs, link: &dyn Fn(&Path, &Path) -> io::Result<()>) -> io::Result<Result<PathBuf, PathBuf>> {
    let parent: PathBuf = parent_of(target);
    return match preserve(name, target, keep, link) {
        Ok(kept) => {
            fs::remove_file(name)?;
            sync_directory(&parent)?;
            Ok(Ok(kept))
        }
        Err(_) => {
            sync_directory(&parent)?;
            Ok(Err(name.to_path_buf()))
        }
    };
}

/// What:
/// `fn replace_core(...) -> io::Result<ReplaceOutcome>` is the protocol:
/// compare, create an exclusive temporary, exchange it with the target, then keep whatever the exchange displaced.
/// Why:
/// The rule that makes the outcome independent of who wrote what: an inode is unlinked only if this call created it and never
/// exposed it at the live path.
/// The replacement becomes visible by the exchange, so after it no branch unlinks any inode without first giving that inode
/// another name (review finding 3b 1: an editor can hold even the replacement open).
/// A target that vanishes between the comparison and the exchange stays vanished (review finding 3b 10).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceCore(path: string, expected: string, newText: string, keep: KeepAs, seams: Seams): ReplaceOutcome
/// ```
fn replace_core(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, between: &dyn Fn(&Path), after_exchange: &dyn Fn(&Path), swap: &dyn Fn(&Path, &Path) -> io::Result<()>, link: &dyn Fn(&Path, &Path) -> io::Result<()>) -> io::Result<ReplaceOutcome> {
    let first = match observe(path)? {
        Some(found) => found,
        None => return Ok(ReplaceOutcome::LeftAlone),
    };
    if first.text != expected {
        return Ok(ReplaceOutcome::LeftAlone);
    }
    between(path);
    let (temporary, _device, _inode) = create_temporary(path, new_text, first.mode)?;
    if let Err(error) = swap(&temporary, path) {
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
        return Ok(match keep_and_unlink(&temporary, path, keep, link)? {
            Ok(kept) => ReplaceOutcome::Replaced(Some(kept)),
            Err(left) => ReplaceOutcome::Displaced(left),
        });
    }
    if swap(&temporary, path).is_err() {
        // The replacement stays live and the unexpected file is at the temporary name.
        return Ok(ReplaceOutcome::Displaced(match keep_and_unlink(&temporary, path, keep, link)? {
            Ok(kept) => kept,
            Err(left) => left,
        }));
    }
    // The exchange is undone: the other writer's file is live again, and the replacement (live for a moment, possibly open
    // elsewhere) is kept, not deleted.
    return Ok(ReplaceOutcome::Undone(match keep_and_unlink(&temporary, path, keep, link)? {
        Ok(kept) => kept,
        Err(left) => left,
    }));
}

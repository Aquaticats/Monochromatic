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
//! The contract here is therefore preservation, not atomic conditional replacement:
//! no byte that was in the file when the exchange happened is ever left without a name,
//! the file a concurrent writer published is put back (or kept under a reported name),
//! and a filesystem that cannot exchange makes the restore fail visibly instead of falling back to a rename that overwrites.
//! Two concurrent publishers can still end up in the other order than they ran; the loser is kept and reported.
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
/// The replacement is created exclusively (`create_new`), written, synchronized and renamed.
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
/// Temporary names combine the process id, the clock and this counter, so two replacements in one process never collide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let counter = 0; // single-threaded in Node
/// ```
use std::sync::atomic::{AtomicU64, Ordering};
/// What:
/// `use rustix::fs::{CWD, RenameFlags, renameat_with};` imports the safe wrapper of `renameat2`:
/// the flags, and the current directory as the base of relative names.
/// Why:
/// The exchange needs `RENAME_EXCHANGE` and the put-back needs `RENAME_NOREPLACE`, which the standard library does not expose.
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
    /// The file held something else, or held it only for a moment and was put back:
    /// another writer's file is where it was.
    LeftAlone,
    /// Another writer published twice in the gap,
    /// so a file was displaced and could not be put back;
    /// it was kept under this name.
    Preserved(
        /// Where the displaced file was kept,
        /// as an owned path (`PathBuf`, not the borrowed `&Path`, because the value is returned to the caller).
        PathBuf,
    ),
}

/// Where and under what suffix a displaced file is kept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeepAs {
    /// The directory the kept names are made in;
    /// `None` means next to the target.
    pub directory: Option<PathBuf>,
    /// The suffix after the target's file name (`.displaced`, `.refused`).
    pub suffix: String,
}

impl KeepAs {
    /// What:
    /// `pub fn next_to_target(suffix: &str) -> KeepAs` keeps displaced files in the target's own directory.
    /// Why:
    /// A refused lockfile saved by a recovery sits beside the lockfile, where the person looks for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static nextToTarget(suffix: string): KeepAs
    /// ```
    pub fn next_to_target(suffix: &str) -> KeepAs {
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
        return KeepAs { directory: Some(directory.to_path_buf()), suffix: suffix.to_string() };
    }
}

/// What:
/// `fn exchange(first: &Path, second: &Path) -> io::Result<()>` swaps two paths in one step (`renameat2` with `RENAME_EXCHANGE`).
/// Why:
/// After the call `first` names what `second` named and the other way round, with no moment at which either path is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exchange(first: string, second: string): void  // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
/// ```
fn exchange(first: &Path, second: &Path) -> io::Result<()> {
    return match renameat_with(CWD, first, CWD, second, RenameFlags::EXCHANGE) {
        Ok(()) => Ok(()),
        Err(code) if code == Errno::INVAL || code == Errno::NOSYS || code == Errno::OPNOTSUPP => {
            Err(io::Error::new(io::ErrorKind::Unsupported, format!("this filesystem cannot exchange {} and {} atomically ({code})", first.display(), second.display())))
        }
        Err(code) => Err(io::Error::from(code)),
    };
}

/// What:
/// `fn sync_directory(directory: &Path) -> io::Result<()>` forces the directory entries of a directory to disk.
/// Why:
/// A rename, a link or an unlink is durable only after the directory that holds the name is synchronized.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fd = openSync(directory, 'r'); fsyncSync(fd); closeSync(fd);
/// ```
pub fn sync_directory(directory: &Path) -> io::Result<()> {
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
    return match path.parent() {
        Some(found) if !found.as_os_str().is_empty() => found.to_path_buf(),
        _ => PathBuf::from("."),
    };
}

/// What:
/// `fn create_temporary(target: &Path, text: &str, mode: u32) -> io::Result<(PathBuf, u64, u64)>` creates a new, uniquely named
/// sibling file holding `text`, synchronizes it, and returns its path with its device and inode numbers.
/// Why:
/// The name is never reused (`create_new` fails if it exists), so a stranded file from a crashed run, or a hard link to a preserved
/// file, can never be truncated by a later run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createTemporary(target: string, text: string, mode: number): [string, bigint, bigint]  // openSync(name, 'wx', mode)
/// ```
fn create_temporary(target: &Path, text: &str, mode: u32) -> io::Result<(PathBuf, u64, u64)> {
    use std::os::unix::fs::OpenOptionsExt;
    let base: String = file_name_text(target);
    let directory: PathBuf = parent_of(target);
    loop {
        let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|elapsed| { return elapsed.as_nanos(); }).unwrap_or(0);
        let number: u64 = TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate: PathBuf = directory.join(format!(".{base}.shadow-{}-{nanos}-{number}.tmp", std::process::id()));
        let opened = OpenOptions::new().write(true).create_new(true).mode(mode & 0o777).open(&candidate);
        match opened {
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
}

/// What:
/// `fn preserve(displaced: &Path, target: &Path, keep: &KeepAs) -> io::Result<PathBuf>` gives a file a surviving name:
/// a hard link to the first free `<name><suffix>`, `<name><suffix>.1`, ... in the keep directory (else beside the target).
/// Why:
/// The temporary name is removed only after this succeeds, so the inode, with any write an open descriptor still makes to it, keeps a name.
/// A hard link fails when the name exists, so two runs never share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function preserve(displaced: string, target: string, keep: KeepAs): string  // linkSync(displaced, name) per candidate name
/// ```
fn preserve(displaced: &Path, target: &Path, keep: &KeepAs) -> io::Result<PathBuf> {
    let base: String = file_name_text(target);
    let mut directories: Vec<PathBuf> = Vec::new();
    if let Some(chosen) = &keep.directory {
        fs::create_dir_all(chosen)?;
        directories.push(chosen.clone());
    }
    directories.push(parent_of(target));
    let mut last_error: io::Error = io::Error::other("no directory to keep the displaced file in");
    for directory in directories {
        let mut number: u32 = 0;
        loop {
            let name: String = if number == 0 { format!("{base}{}", keep.suffix) } else { format!("{base}{}.{number}", keep.suffix) };
            let candidate: PathBuf = directory.join(name);
            match fs::hard_link(displaced, &candidate) {
                Ok(()) => {
                    sync_directory(&directory)?;
                    return Ok(candidate);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => number += 1,
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
/// `pub fn no_hook(_path: &Path)` is the hook that does nothing.
/// Why:
/// The real callers have no gap to exercise; a named function passes as `&dyn Fn(&Path)` where a closure would be flagged as avoidable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const noHook = (_path: string): void => {};
/// ```
pub fn no_hook(_path: &Path) {}

/// What:
/// `pub fn file_name_text(path: &Path) -> String` is the last component of a path as text (empty when there is none).
/// Why:
/// The kept copies are named after the file they stand for, and the messages name them.
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
/// holds `expected`, keeping the displaced file next to the target as `<name>.displaced`.
/// Why:
/// The plain form for the callers and tests that have no keep directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHolds(path: string, expected: string, newText: string, between: (path: string) => void): ReplaceOutcome
/// ```
pub fn replace_if_holds(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    return replace_core(path, expected, new_text, &KeepAs::next_to_target(".displaced"), between, &no_hook, &exchange);
}

/// What:
/// `pub fn replace_if_holds_with_hooks(path, expected, new_text, between, after_exchange)` is `replace_if_holds` with a second hook
/// that runs right after the exchange, before the displaced file is read.
/// Why:
/// The second gap is where a writer publishes between the swap and the swap back; the hook lets a test publish one there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHoldsWithHooks(path, expected, newText, between, afterExchange): ReplaceOutcome
/// ```
pub fn replace_if_holds_with_hooks(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path), after_exchange: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    return replace_core(path, expected, new_text, &KeepAs::next_to_target(".displaced"), between, after_exchange, &exchange);
}

/// What:
/// `pub fn replace_keeping(path, expected, new_text, keep) -> io::Result<ReplaceOutcome>` is the production form:
/// the displaced file is kept as `keep` says.
/// Why:
/// Rollbacks keep their displaced output in the quarantine; recoveries keep it beside the lockfile as `.refused`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceKeeping(path: string, expected: string, newText: string, keep: KeepAs): ReplaceOutcome
/// ```
pub fn replace_keeping(path: &Path, expected: &str, new_text: &str, keep: &KeepAs) -> io::Result<ReplaceOutcome> {
    return replace_core(path, expected, new_text, keep, &no_hook, &no_hook, &exchange);
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
    return replace_core(path, expected, new_text, keep, between, &no_hook, &exchange);
}

/// What:
/// `pub fn replace_with_swap(path, expected, new_text, keep, swap)` is the protocol with the swap primitive chosen by the caller.
/// Why:
/// A test passes a swap that fails with the error a filesystem without exchange gives, to show that the restore fails visibly
/// and overwrites nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceWithSwap(path, expected, newText, keep, swap): ReplaceOutcome
/// ```
pub fn replace_with_swap(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, swap: &dyn Fn(&Path, &Path) -> io::Result<()>) -> io::Result<ReplaceOutcome> {
    return replace_core(path, expected, new_text, keep, &no_hook, &no_hook, swap);
}

/// What:
/// `fn replace_core(...) -> io::Result<ReplaceOutcome>` is the protocol:
/// observe, create the replacement, exchange, check the displaced file, then keep it or undo.
/// Why:
/// See the module header for the contract; every step that could drop data keeps a name first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceCore(path, expected, newText, keep, between, afterExchange): ReplaceOutcome
/// ```
fn replace_core(path: &Path, expected: &str, new_text: &str, keep: &KeepAs, between: &dyn Fn(&Path), after_exchange: &dyn Fn(&Path), swap: &dyn Fn(&Path, &Path) -> io::Result<()>) -> io::Result<ReplaceOutcome> {
    // An unreadable file is an error, never an empty one.
    let first = match observe(path)? {
        Some(found) => found,
        None => return Ok(ReplaceOutcome::LeftAlone),
    };
    if first.text != expected {
        return Ok(ReplaceOutcome::LeftAlone);
    }
    between(path);
    let (temporary, replacement_device, replacement_inode) = create_temporary(path, new_text, first.mode)?;
    let parent: PathBuf = parent_of(path);
    if let Err(error) = swap(&temporary, path) {
        // The target vanished: put the replacement there only if nothing has appeared, never over a file.
        if error.kind() == io::ErrorKind::NotFound {
            return match renameat_with(CWD, &temporary, CWD, path, RenameFlags::NOREPLACE) {
                Ok(()) => {
                    sync_directory(&parent)?;
                    Ok(ReplaceOutcome::Replaced(None))
                }
                Err(_) => {
                    let _ = fs::remove_file(&temporary);
                    Ok(ReplaceOutcome::LeftAlone)
                }
            };
        }
        // No exchange here: fail visibly. The replacement is this run's own file and is removed; the target is untouched.
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    after_exchange(path);
    // The temporary name now holds the file that was at `path`; it is read through an open descriptor.
    let displaced_expected: bool = match observe(&temporary) {
        Ok(Some(seen)) => seen.text == expected,
        _ => false,
    };
    if displaced_expected {
        return match preserve(&temporary, path, keep) {
            Ok(kept) => {
                fs::remove_file(&temporary)?;
                sync_directory(&parent)?;
                Ok(ReplaceOutcome::Replaced(Some(kept)))
            }
            // No name could be made: the displaced file stays at the temporary name, reported as kept there.
            Err(_) => Ok(ReplaceOutcome::Preserved(temporary)),
        };
    }
    // Another writer's file came out: swap back.
    if swap(&temporary, path).is_err() {
        return match preserve(&temporary, path, keep) {
            Ok(kept) => {
                fs::remove_file(&temporary)?;
                Ok(ReplaceOutcome::Preserved(kept))
            }
            Err(_) => Ok(ReplaceOutcome::Preserved(temporary)),
        };
    }
    // The temporary name holds what was at `path` after the first exchange: this run's replacement, unless yet another writer published.
    // The inode alone is not enough: a writer that opened the replacement in place keeps its inode but changes its text.
    let own: bool = match observe(&temporary) {
        Ok(Some(seen)) => seen.device == replacement_device && seen.inode == replacement_inode && seen.text == new_text,
        _ => false,
    };
    if own {
        fs::remove_file(&temporary)?;
        sync_directory(&parent)?;
        return Ok(ReplaceOutcome::LeftAlone);
    }
    return match preserve(&temporary, path, keep) {
        Ok(kept) => {
            fs::remove_file(&temporary)?;
            sync_directory(&parent)?;
            Ok(ReplaceOutcome::Preserved(kept))
        }
        Err(_) => Ok(ReplaceOutcome::Preserved(temporary)),
    };
}

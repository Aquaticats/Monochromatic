//! What:
//! The atomic compare-and-replace of one file:
//! the new text is exchanged with the file in one step (`renameat2` with `RENAME_EXCHANGE`),
//! the displaced file is read,
//! and the exchange is undone when it was not the file expected.
//! Why:
//! Reading a file and then renaming over it leaves a gap in which another writer can publish a lockfile that the rename destroys.
//! Here no writer's file is ever lost and no lock is taken;
//! the journal (`journal.rs`) re-exports these items because its restores and its recovery are the callers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-replace.ts
//! export function replaceIfHolds(path: string, expected: string, newText: string, between: (path: string) => void): ReplaceOutcome
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// The new text is written to a sibling file and the displaced file is read back with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports `io` from the standard library so this file can name it unqualified.
/// Why:
/// `io::Result` and `io::ErrorKind` are the error types of the file operations here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node; nothing is imported
/// ```
use std::io;
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is the borrowed path lent to the functions and `PathBuf` the owned path they return (the same pair as `&str` and `String`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{Path, PathBuf};
/// What:
/// `use rustix::fs::{CWD, RenameFlags, renameat_with};` imports the safe wrapper of `renameat2`:
/// the flags,
/// and the current directory as the base of relative names.
/// Why:
/// The exchange needs `RENAME_EXCHANGE`,
/// which the standard library does not expose;
/// `rustix` wraps it without `unsafe` code here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
/// ```
use rustix::fs::{CWD, RenameFlags, renameat_with};

/// What:
/// `use crate::journal::sibling_temp;` imports the function that names the temporary sibling file of a target.
/// Why:
/// The new text is written there before the exchange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { siblingTemp } from 'crate/journal';
/// ```
use crate::journal::sibling_temp;

/// What a compare-and-replace did to one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplaceOutcome {
    /// The file held what was expected and now holds the new text.
    Replaced,
    /// The file held something else,
    /// or held it only for a moment and was put back:
    /// another writer's file is where it was.
    LeftAlone,
    /// Another writer published twice in the gap,
    /// so a file was displaced and could not be put back;
    /// it was kept under this name.
    Preserved(
        /// Where the displaced file was kept,
        /// as an owned path (`PathBuf`,
        /// not the borrowed `&Path`,
        /// because the value is returned to the caller).
        PathBuf,
    ),
}

/// What:
/// `fn exchange(first, second) -> io::Result<()>` swaps two paths in one step (`renameat2` with `RENAME_EXCHANGE`,
/// through `rustix`'s safe wrapper).
/// Why:
/// After the call `first` names what `second` named and the other way round,
/// with no moment at which either path is missing,
/// so the file that was displaced can be read and,
/// if it was not the expected one,
/// swapped back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
/// function exchange(first: string, second: string): void
/// ```
fn exchange(first: &Path, second: &Path) -> io::Result<()> {
    return renameat_with(CWD, first, CWD, second, RenameFlags::EXCHANGE).map_err(io::Error::from);
}

/// What:
/// `fn preserve(displaced, target) -> io::Result<PathBuf>` moves a file to the first free `<name>.displaced`,
/// `<name>.displaced.1`,
/// ... next to the target,
/// without replacing anything.
/// Why:
/// A file that could not be put back must not be deleted;
/// a hard link fails when the name exists,
/// so two runs never share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function preserve(displaced: string, target: string): string  // linkSync(displaced, name) per candidate, then unlinkSync(displaced)
/// ```
fn preserve(displaced: &Path, target: &Path) -> io::Result<PathBuf> {
    let base: String = file_name_text(target);
    let mut number: u32 = 0;
    loop {
        let name: String = if number == 0 { format!("{base}.displaced") } else { format!("{base}.displaced.{number}") };
        let candidate: PathBuf = target.with_file_name(name);
        match fs::hard_link(displaced, &candidate) {
            Ok(()) => {
                fs::remove_file(displaced)?;
                return Ok(candidate);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => number += 1,
            Err(error) => return Err(error),
        }
    }
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
/// `pub fn replace_if_holds(path, expected, new_text, between) -> io::Result<ReplaceOutcome>` puts `new_text` in `path` only if
/// `path` holds `expected`,
/// comparing and replacing in one atomic step.
/// Why:
/// Review finding and delegated pick (2026-11-03):
/// reading a file and then renaming over it leaves a gap in which another writer
/// can publish a lockfile that the rename then destroys.
/// The new text is written to a sibling file and exchanged with `path`
/// atomically;
/// the file that came out is read,
/// and when it is not the expected one the exchange is undone,
/// so no writer's
/// file is ever lost and no lock is taken.
/// A filesystem that cannot exchange falls back to a second comparison right before
/// an ordinary rename.
/// `between` runs after the first comparison,
/// so a test can publish a file exactly in the gap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHolds(path: string, expected: string, newText: string, between: (path: string) => void): ReplaceOutcome
/// ```
pub fn replace_if_holds(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    return replace_if_holds_with_hooks(path, expected, new_text, between, &no_hook);
}

/// What:
/// `pub fn replace_if_holds_with_hooks(path, expected, new_text, between, after_exchange)` is `replace_if_holds` with a second
/// hook that runs right after the exchange,
/// before the displaced file is read.
/// Why:
/// The second gap is where a writer publishes a file between the swap and the swap back;
/// the hook lets a test publish one there,
/// to show that the file displaced twice is kept and not deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHoldsWithHooks(path, expected, newText, between, afterExchange): ReplaceOutcome
/// ```
pub fn replace_if_holds_with_hooks(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path), after_exchange: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    let first: String = fs::read_to_string(path).unwrap_or_default();
    if first != expected {
        return Ok(ReplaceOutcome::LeftAlone);
    }
    between(path);
    let temp: PathBuf = sibling_temp(path);
    fs::write(&temp, new_text)?;
    if exchange(&temp, path).is_err() {
        // No exchange here (the path vanished, or the filesystem cannot swap): compare again, then an ordinary rename.
        let again: String = fs::read_to_string(path).unwrap_or_default();
        if again != expected {
            let _ = fs::remove_file(&temp);
            return Ok(ReplaceOutcome::LeftAlone);
        }
        fs::rename(&temp, path)?;
        return Ok(ReplaceOutcome::Replaced);
    }
    after_exchange(path);
    // The temporary name now holds the file that was at `path`.
    let displaced: String = fs::read_to_string(&temp).unwrap_or_default();
    if displaced == expected {
        fs::remove_file(&temp)?;
        return Ok(ReplaceOutcome::Replaced);
    }
    // Another writer published in the gap: swap back, so their file is where it was.
    if exchange(&temp, path).is_err() {
        return Ok(ReplaceOutcome::Preserved(preserve(&temp, path)?));
    }
    let after_undo: String = fs::read_to_string(&temp).unwrap_or_default();
    if after_undo == new_text {
        fs::remove_file(&temp)?;
        return Ok(ReplaceOutcome::LeftAlone);
    }
    // Yet another writer published between the swap and the swap back; their file now sits at the temporary name.
    return Ok(ReplaceOutcome::Preserved(preserve(&temp, path)?));
}

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
use rustix::fs::{
    CWD,
    RenameFlags,
    renameat_with,
};
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
/// `use std::fs::{self, File};` imports the file-system module and the open-file type.
/// Why:
/// The displaced file is linked under a surviving name and a temporary that was never exposed is removed,
/// and the two barriers open the file or directory they sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { linkSync, unlinkSync, openSync } from 'node:fs';
/// ```
use std::fs::{
    self,
    File,
};
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// Every step of the protocol answers an `io::Result`,
/// and "this filesystem cannot exchange" is an `io::Error`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node
/// ```
use std::io;
/// What:
/// `use std::path::Path;` imports the borrowed path type from the standard library so this file can name it unqualified.
/// Why:
/// The protocol only lends the paths it is given;
/// the owned `PathBuf` of the same family is not named here,
/// because every name this file produces comes from the two modules split out of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;

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

/// The name a file is published under,
/// made durable and exclusive.
mod creates;
/// What one replacement answered and where the file it displaced is kept.
mod outcome_and_keep_rule;

/// The temporary file the replacement publishes (split out of this module).
use self::creates::create_temporary;
/// The error for a directory whose temporary names are used up (split out of this module).
#[cfg(test)]
pub(crate) use self::creates::names_exhausted_message;
/// The name cap and the borrowed parent of a path (split out of this module).
pub(crate) use self::creates::{
    MAX_NAME_ATTEMPTS,
    parent_of,
};
/// The directory chain and the exclusive file that the replacement is published from (split out of this module).
pub use self::creates::{
    create_directories_synced,
    create_directories_with,
    create_exclusive,
    file_name_text,
    sync_directory,
    unique_suffix,
};
/// What a compare-and-replace answered and where the displaced file is kept (split out of this module).
pub use self::outcome_and_keep_rule::{
    KeepAs,
    ReplaceOutcome,
};
/// Keeping the displaced file under a surviving name (split out of the journal replacement module).
pub(crate) use crate::journal_replace_preserve::keep_and_unlink;
/// The name a displaced file is kept under,
/// beside the target or in a quarantine (split out of the journal replacement module).
pub(crate) use crate::journal_replace_preserve::keep_name;
/// The registry of directories a creation left unsynchronized (split out of this module).
pub(crate) use crate::journal_replace_unsynced::{
    forget_unsynced,
    is_unsynced,
};
/// The two registry calls the busy-registry test drives (split out of this module).
#[cfg(test)]
pub(crate) use crate::journal_replace_unsynced::{
    hold_unsynced_registry_for_test,
    registry_calls_for_test,
};

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
pub fn exchange_paths(
    first: &Path,
    second: &Path,
) -> io::Result<()> {
    return exchange(first, second);
}

/// The exchange primitive,
/// private to the protocol.
fn exchange(
    first: &Path,
    second: &Path,
) -> io::Result<()> {
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
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "this filesystem cannot exchange {} and {} atomically ({code})",
                    first.display(),
                    second.display()
                ),
            ))
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(code) => Err(io::Error::from(code)),
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
pub fn replace_if_holds(
    path: &Path,
    expected: &str,
    new_text: &str,
    between: &dyn Fn(&Path),
) -> io::Result<ReplaceOutcome> {
    // Borrow `&KeepAs`: lends the value without copying or giving it away.
    return replace_core(
        path,
        expected,
        new_text,
        &KeepAs::next_to_target(".displaced"),
        between,
        &no_hook,
        &Ops::standard(),
    );
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
pub fn replace_if_holds_with_hooks(
    path: &Path,
    expected: &str,
    new_text: &str,
    between: &dyn Fn(&Path),
    after_exchange: &dyn Fn(&Path),
) -> io::Result<ReplaceOutcome> {
    // Borrow `&KeepAs`: lends the value without copying or giving it away.
    return replace_core(
        path,
        expected,
        new_text,
        &KeepAs::next_to_target(".displaced"),
        between,
        after_exchange,
        &Ops::standard(),
    );
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
pub fn replace_keeping(
    path: &Path,
    expected: &str,
    new_text: &str,
    keep: &KeepAs,
) -> io::Result<ReplaceOutcome> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return replace_core(
        path,
        expected,
        new_text,
        keep,
        &no_hook,
        &no_hook,
        &Ops::standard(),
    );
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
pub fn replace_keeping_with_hook(
    path: &Path,
    expected: &str,
    new_text: &str,
    keep: &KeepAs,
    between: &dyn Fn(&Path),
) -> io::Result<ReplaceOutcome> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return replace_core(
        path,
        expected,
        new_text,
        keep,
        between,
        &no_hook,
        &Ops::standard(),
    );
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
pub fn replace_with_swap(
    path: &Path,
    expected: &str,
    new_text: &str,
    keep: &KeepAs,
    swap: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<ReplaceOutcome> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return replace_core(
        path,
        expected,
        new_text,
        keep,
        &no_hook,
        &no_hook,
        &Ops {
            swap,
            ..Ops::standard()
        },
    );
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
fn real_link(
    existing: &Path,
    new: &Path,
) -> io::Result<()> {
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
    pub(crate) fn standard() -> Ops<'static> {
        return Ops {
            swap: &exchange,
            link: &real_link,
            sync_file: &sync_file_data,
            sync_dir: &sync_directory,
        };
    }
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
fn replace_core(
    path: &Path,
    expected: &str,
    new_text: &str,
    keep: &KeepAs,
    between: &dyn Fn(&Path),
    after_exchange: &dyn Fn(&Path),
    ops: &Ops,
) -> io::Result<ReplaceOutcome> {
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
        return Ok(ReplaceOutcome::Displaced(
            match keep_and_unlink(&temporary, path, keep, ops)? {
                Ok(kept) => kept,
                Err(left) => left,
            },
        ));
    }
    // The exchange is undone: the other writer's file is live again, and the replacement (live for a moment, possibly open
    // elsewhere) is kept, not deleted.
    return Ok(ReplaceOutcome::Undone(
        match keep_and_unlink(&temporary, path, keep, ops)? {
            Ok(kept) => kept,
            Err(left) => left,
        },
    ));
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
pub fn replace_with_seams(
    path: &Path,
    expected: &str,
    new_text: &str,
    keep: &KeepAs,
    seams: &Seams,
) -> io::Result<ReplaceOutcome> {
    // Typed binding: `ops: Ops`.
    let ops: Ops = Ops {
        swap: seams.swap,
        link: seams.link,
        sync_file: seams.sync_file,
        sync_dir: seams.sync_dir,
    };
    // Borrow `&ops`: lends the value without copying or giving it away.
    return replace_core(
        path,
        expected,
        new_text,
        keep,
        seams.between,
        seams.after_exchange,
        &ops,
    );
}

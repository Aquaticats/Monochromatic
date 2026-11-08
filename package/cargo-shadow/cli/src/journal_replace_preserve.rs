//! What:
//! How the displaced file is kept under a surviving name:
//! the choice of the name,
//! the link,
//! and the durability of the kept copy up to the first directory that existed before.
//! Why:
//! Split out of `journal_replace.rs`,
//! which replaces files with the displaced copy kept,
//! to keep that file under the code-line limit.
//! The items are re-exported from `journal_replace`,
//! so their paths there are unchanged.

/// The seams,
/// the keep rule,
/// and the helpers of the replacement that the keeping uses.
use crate::journal_replace::{KeepAs, MAX_NAME_ATTEMPTS, Ops, create_directories_with, file_name_text, forget_unsynced, is_unsynced, parent_of};
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
use std::fs::{self};
/// What:
/// `use std::io::{self, Write};` imports the error module and the trait that adds `write_all` to an open file.
/// Why:
/// The replacement text is written through the descriptor that created the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node; writeSync is a free function
/// ```
use std::io::{self};
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


/// Which stage of preserving a file failed,
/// because the stages mean different things for the data.
pub(crate) enum PreserveFailure {
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
pub(crate) fn preserve(displaced: &Path, target: &Path, keep: &KeepAs, ops: &Ops) -> Result<PathBuf, PreserveFailure> {
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
            #[cfg(test)]
            let link_result: io::Result<()> = if crate::failpoints::keep_link_of(target) {
                Err(io::Error::from_raw_os_error(rustix::io::Errno::MLINK.raw_os_error()))
            } else {
                (ops.link)(displaced, &candidate)
            };
            #[cfg(not(test))]
            let link_result: io::Result<()> = (ops.link)(displaced, &candidate);
            match link_result {
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
                    if let Err(error) = durable_up_to_the_anchor(&directory, &created, ops) {
                        return Err(PreserveFailure::Durability { kept: candidate.clone(), error });
                    }
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
    return preserve(displaced, target, keep, &Ops::standard()).map_err(preserve_failure_error);
}

/// What:
/// `fn preserve_failure_error(failure: PreserveFailure) -> io::Error` turns a failure to keep a file into the `io::Error` the caller reports.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn preserve_failure_error(failure: PreserveFailure) -> io::Error {
    return match failure {
        PreserveFailure::Data(error) | PreserveFailure::Link(error) => error,
        PreserveFailure::Durability { kept, error } => io::Error::other(format!("kept as {} but not made durable: {error}", kept.display())),
    };
}

/// What:
/// `fn durable_up_to_the_anchor(directory: &Path, created: &[PathBuf], ops: &Ops) -> io::Result<()>` makes the entries of the kept copy durable from the kept directory up to the first directory that existed before this transaction.
/// Why:
/// The walk stops there (contract assumption b):
/// a pre-existing directory and its path are assumed durable.
/// A directory this call created,
/// or one this process knows it left unsynchronized,
/// is not.
/// The path is resolved first,
/// so a link or a relative path names the directories that really hold the entries.
/// The walk fails,
/// and preservation is not reported,
/// when any barrier fails.
pub(crate) fn durable_up_to_the_anchor(directory: &Path, created: &[PathBuf], ops: &Ops) -> io::Result<()> {
    let resolved: PathBuf = fs::canonicalize(directory)?;
    let mut created_here: Vec<PathBuf> = Vec::new();
    for made in created {
        if let Ok(found) = fs::canonicalize(made) {
            created_here.push(found);
        }
    }
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
}

/// What:
/// `fn sync_directory_and_parent(sync_dir: &dyn Fn(&Path) -> io::Result<()>, directory: &Path) -> io::Result<()>` syncs a directory and then the directory that holds it.
/// Why:
/// A function replaces the closure that `and_then` ran after the first sync.
pub(crate) fn sync_directory_and_parent(sync_dir: &dyn Fn(&Path) -> io::Result<()>, directory: &Path) -> io::Result<()> {
    sync_dir(directory)?;
    return sync_dir(&parent_of(directory));
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
pub(crate) fn keep_and_unlink(name: &Path, target: &Path, keep: &KeepAs, ops: &Ops) -> io::Result<Result<PathBuf, PathBuf>> {
    let parent: PathBuf = parent_of(target);
    // `match`: picks the arm whose pattern fits the value.
    return match preserve(name, target, keep, ops) {
        Ok(kept) => {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Macro `format!`: builds or prints text (or a list) from a template.
            if let Err(error) = fs::remove_file(name) {
                return Err(io::Error::other(format!("{} is kept as {} but its temporary name {} could not be removed: {error}", target.display(), kept.display(), name.display())));
            }
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&parent`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Macro `format!`: builds or prints text (or a list) from a template.
            if let Err(error) = (ops.sync_dir)(&parent) {
                return Err(io::Error::other(format!("{} is kept as {} but {} could not be synced: {error}", target.display(), kept.display(), parent.display())));
            }
            Ok(Ok(kept))
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(PreserveFailure::Link(_)) => {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&parent`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Macro `format!`: builds or prints text (or a list) from a template.
            if let Err(error) = (ops.sync_dir)(&parent) {
                return Err(io::Error::other(format!("the displaced file stays as {} but {} could not be synced: {error}", name.display(), parent.display())));
            }
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

//! What:
//! The lockfile transaction's snapshots and its journal:
//! a durable record,
//! written before cargo is started,
//! that lets the next run finish what a killed run left,
//! and the restore that puts snapshots back without destroying anyone's file.
//! Why:
//! Owner rules:
//! "never discard user data" and "use as few locks as possible".
//! So there is no lock around the lockfile.
//! Ownership of a file is decided by the evidence in `file_owner.rs` (the change time of the file against the window of the child's run),
//! never by reading the file after the child returned;
//! a restore replaces a lockfile only when it still holds what this run's child wrote,
//! with the replacement protocol of `journal_replace.rs` (every displaced file keeps a name),
//! and anything else is left alone with a warning.
//! The journal under `.cache/cargo-shadow/journal/<pid>/` is written and synchronized before cargo mutates anything:
//! an owner record (process id and start time,
//! so a reused process id is not mistaken for the killed run),
//! the snapshots,
//! and a complete,
//! versioned index.
//! A recovering run first claims a journal by renaming its directory (only one run wins),
//! decides root by root whether the lockfile the killed run left passes the policy,
//! keeps it if it does and otherwise saves it beside the lockfile as `Cargo.lock.refused` while putting the snapshot back,
//! restores the manifests of the roots whose lockfile was not kept,
//! synchronizes,
//! and only then deletes the journal.
//! `cargo -Zunstable-options update --lockfile-path`,
//! which would resolve into a private file and publish it with one rename,
//! does not exist on the pinned nightly (checked 2026-11-03:
//! `cargo update --help` and `cargo check --help` list no such option).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal.ts
//! export function begin(root: string, snapshots: LockSnapshot[]): void
//! export function restoreIfUnchanged(root: string, snapshots: LockSnapshot[], produced: LockSnapshot[]): string[]
//! export function recover(root: string, isAlive: (pid: number, identity: string) => boolean, judge: Judge): string[]
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a list or value that can be changed through a shared reference.
/// Why:
/// The record of the lockfiles a run wrote grows through one while the transaction body holds a shared reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::fs;` imports the standard file-system module.
/// Why:
/// The journal is made of ordinary files and directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync, mkdirSync, renameSync, rmSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io::{self, Write};` imports the error module and the trait that adds `write_all` to an open file.
/// Why:
/// Journal files are written through a descriptor and synchronized before they count as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node
/// ```
use std::io::{self, Write};
/// What:
/// `use std::os::unix::ffi::{OsStrExt, OsStringExt};` imports the traits that turn an operating-system string into bytes and back.
/// Why:
/// A manifest path is stored as the exact bytes of the path,
/// so a path that is not UTF-8 survives a recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Buffer.from(path) / Buffer.toString()
/// ```
use std::os::unix::ffi::{OsStrExt, OsStringExt};
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// Paths are lent to the functions and the journal's own paths are returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `pub use crate::journal_replace::{...};` imports and re-exports the replacement protocol,
/// so callers keep writing `crate::journal::replace_if_holds`.
/// Why:
/// The file was split to stay under the line limit;
/// the callers and the tests are unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { KeepAs, ReplaceOutcome, replaceIfHolds } from './journal-replace.ts';
/// ```
pub use crate::journal_replace::{KeepAs, ReplaceOutcome, file_name_text, no_hook, replace_if_holds, replace_if_holds_with_hooks, replace_keeping, replace_keeping_with_hook, replace_with_swap, sync_directory};
/// Re-exports the recovery entry points,
/// which live in `journal_recover.rs`.
pub use crate::journal_recover::{recover, recover_with};

/// What:
/// `use crate::constants::LOCKFILE_NAME;` imports the lockfile's file name.
/// Why:
/// Lockfile paths are built from a crate root and this name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::LOCKFILE_NAME;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports the lockfile snapshot type and the reader of the lockfiles.
/// Why:
/// Snapshots are what the journal stores;
/// the live lockfiles are read for the recovery decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, readLocks } from 'crate/crates';
/// ```
use crate::crates::LockSnapshot;
/// What:
/// `use crate::file_owner::{Claim, Window, claim_output, observe, timed};` imports the ownership evidence.
/// Why:
/// Every lockfile written by a child is attributed with it before it is recorded as this run's output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Claim, Window, claimOutput, observe, timed } from 'crate/file_owner';
/// ```
use crate::file_owner::{Claim, Window, claim_output, observe, timed};
/// What:
/// `use crate::manifest_edit::ManifestEdit;` imports one manifest a `--breaking` run may rewrite.
/// Why:
/// The journal records the manifests with their root and their text before the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ManifestEdit } from 'crate/manifest_edit';
/// ```
use crate::manifest_edit::ManifestEdit;

/// Where journals live,
/// relative to the repository root.
pub const JOURNAL_DIR: &str = ".cache/cargo-shadow/journal";

/// Where files a rollback displaced are kept,
/// relative to the repository root.
pub const QUARANTINE_DIR: &str = ".cache/cargo-shadow/displaced";

/// The suffix appended to a lockfile name when a refused live lock is saved before the snapshot goes back.
pub const REFUSED_SUFFIX: &str = ".refused";

/// The file in a journal directory that names the crate roots,
/// one per line,
/// in the order of the snapshot files;
/// it is written last,
/// so a journal without it was never complete.
pub(crate) const INDEX_FILE: &str = "index.txt";

/// The file in a journal directory that says who wrote it:
/// the version line,
/// the process id and the process start time.
pub(crate) const OWNER_FILE: &str = "owner";

/// The file in a journal directory that names the manifests a `--breaking` run may rewrite.
/// Its first line is `v1`,
/// each following line is `<crate root>`,
/// a tab and the hexadecimal bytes of the path,
/// and its last line is `end <count>`,
/// so a truncated file is recognized.
pub(crate) const MANIFEST_INDEX_FILE: &str = "manifests.txt";

/// What:
/// `fn write_synced(path: &Path, text: &str) -> io::Result<()>` creates a file,
/// writes it and forces it to disk.
/// Why:
/// A journal record counts as written only when its bytes are durable,
/// before cargo mutates the originals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeSynced(path: string, text: string): void  // openSync, writeSync, fsyncSync
/// ```
fn write_synced(path: &Path, text: &str) -> io::Result<()> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `file: fs::File`.
    let mut file: fs::File = fs::File::create(path)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    file.write_all(text.as_bytes())?;
    return file.sync_all();
}

/// What:
/// `pub fn atomic_write(target: &Path, text: &str) -> io::Result<()>` publishes a file whole:
/// written and synchronized under a unique sibling name,
/// renamed over the target,
/// and the directory synchronized.
/// Why:
/// The journal index must appear complete or not at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function atomicWrite(target: string, text: string): void
/// ```
pub fn atomic_write(target: &Path, text: &str) -> io::Result<()> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `directory: PathBuf`.
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let directory: PathBuf = match target.parent() {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found.to_path_buf(),
        // `None`: the present or the absent value of an `Option`.
        None => PathBuf::from("."),
    };
    // Typed binding: `temp: PathBuf`.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    let temp: PathBuf = directory.join(format!(".{}.shadow-{}.tmp", file_name_text(target), crate::file_owner::clock_ns()));
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&temp`: lends the value without copying or giving it away.
    write_synced(&temp, text)?;
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Borrow `&temp`: lends the value without copying or giving it away.
    if let Err(error) = fs::rename(&temp, target) {
        let _ = fs::remove_file(&temp);
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(error);
    }
    // Borrow `&directory`: lends the value without copying or giving it away.
    return sync_directory(&directory);
}

/// What:
/// `fn snapshot_of<'a>(snapshots: &'a [LockSnapshot], crate_dir: &str) -> Option<&'a LockSnapshot>` finds the snapshot of one crate root.
/// Why:
/// The restore pairs each snapshot with the output recorded for the same root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshotOf = (snapshots: LockSnapshot[], crateDir: string) => snapshots.find((s) => s.crateDir === crateDir);
/// ```
fn snapshot_of<'a>(snapshots: &'a [LockSnapshot], crate_dir: &str) -> Option<&'a LockSnapshot> {
    for candidate in snapshots {
        if candidate.crate_dir == crate_dir {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
/// `pub fn restore_if_unchanged(root, snapshots, produced) -> io::Result<Vec<String>>` puts each snapshot back only when the live
/// lockfile still holds what this run's child wrote.
/// Why:
/// A live lockfile that differs from the recorded output was changed by someone else and is left alone with a warning;
/// a root with no recorded output was never written by this run (unknown ownership never authorizes a restore);
/// a lockfile that cannot be read is an error,
/// never an empty file.
/// The file the restore displaces is kept in the quarantine directory,
/// not deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchanged(root: string, snapshots: LockSnapshot[], produced: LockSnapshot[]): string[]
/// ```
pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>> {
    return restore_if_unchanged_with(root, snapshots, produced, &no_hook);
}

/// What:
/// `pub fn restore_if_unchanged_with(root, snapshots, produced, between)` is `restore_if_unchanged` with a hook that runs after a
/// lockfile was compared and before it is replaced.
/// Why:
/// The gap between the comparison and the replacement is where another writer can publish a lockfile;
/// the hook lets a test publish one exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchangedWith({ root, snapshots, produced, between }): string[]
/// ```
pub fn restore_if_unchanged_with(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot], between: &dyn Fn(&Path)) -> io::Result<Vec<String>> {
    let mut warnings: Vec<String> = Vec::new();
    for snapshot in snapshots {
        let path: PathBuf = root.join(&snapshot.crate_dir).join(LOCKFILE_NAME);
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&path`: lends the value without copying or giving it away.
        // Typed binding: `live: String`.
        let live: String = match observe(&path)? {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found.text,
            // `None`: the present or the absent value of an `Option`.
            None => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                warnings.push(format!("{}/{LOCKFILE_NAME} is missing; it was not restored", snapshot.crate_dir));
                continue;
            }
        };
        if live == snapshot.source {
            continue;
        }
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&snapshot.crate_dir`: lends the value without copying or giving it away.
        match snapshot_of(produced, &snapshot.crate_dir) {
            Some(found) if found.source == live => {}
            Some(_) => {
                warnings.push(format!(
                    "{}/{LOCKFILE_NAME} was changed by another process while the step ran; it was left as that process wrote it, not restored",
                    snapshot.crate_dir
                ));
                continue;
            }
            None => {
                warnings.push(format!(
                    "{}/{LOCKFILE_NAME} differs from the snapshot, but this run did not write it, so another process did; it was left as it is, not restored",
                    snapshot.crate_dir
                ));
                continue;
            }
        }
        // Borrow `&root.join`: lends the value without copying or giving it away.
        // Typed binding: `keep: KeepAs`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let keep: KeepAs = KeepAs::in_directory(&root.join(QUARANTINE_DIR), &format!(".{}.displaced", snapshot.crate_dir.replace('/', "__")));
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&path`: lends the value without copying or giving it away.
        match replace_keeping_with_hook(&path, &live, &snapshot.source, &keep, between)? {
            ReplaceOutcome::Replaced(_) => {}
            ReplaceOutcome::LeftAlone => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed by another process while the step ran; it was left as that process wrote it, not restored",
                snapshot.crate_dir
            )),
            ReplaceOutcome::Preserved(copy) | ReplaceOutcome::Displaced(copy) | ReplaceOutcome::Undone(copy) => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed twice by other processes while the step ran; the file displaced is kept as {}",
                snapshot.crate_dir,
                copy.display()
            )),
        }
    }
    return Ok(warnings);
}

/// What:
/// `pub fn run_recording<T>(root, crate_dir, produced, run) -> io::Result<(T, Claim)>` runs a child that may write one lockfile and
/// records the lockfile as this run's output when the evidence says the child wrote it.
/// Why:
/// This is the one place a lockfile is attributed to the run:
/// the live text before the child is the prior state,
/// the child runs inside a timed window,
/// and the file counts as the child's output only if it differs from the prior state and its change time lies in the window.
/// The output is recorded whether the child succeeded or failed,
/// because a child that wrote a lockfile and then exited with an error still left that lockfile behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runRecording<T>(root, crateDir, produced, run: () => T): [T, Claim]
/// ```
pub fn run_recording<T>(root: &Path, crate_dir: &str, produced: &RefCell<Vec<LockSnapshot>>, run: impl FnOnce() -> T) -> io::Result<(T, Claim)> {
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = root.join(crate_dir).join(LOCKFILE_NAME);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&path`: lends the value without copying or giving it away.
    // Typed binding: `prior: String`.
    let prior: String = match observe(&path)? {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found.text,
        // `None`: the present or the absent value of an `Option`.
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        None => String::new(),
    };
    let (result, window): (T, Window) = timed(run);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&path`: lends the value without copying or giving it away.
    // Typed binding: `claim: Claim`.
    let claim: Claim = claim_output(&path, &prior, window)?;
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&claim`: lends the value without copying or giving it away.
    // Typed binding: `Claim: :Ours(text)`.
    if let Claim::Ours(text) = &claim {
        // `let mut`: a binding that may change.
        let mut list = produced.borrow_mut();
        // Borrow `&mut *list`: lends the value without copying or giving it away.
        // Typed binding: `earlier: Vec<LockSnapshot>`.
        // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
        // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: T[] / Set / Map / number
        // ```
        let earlier: Vec<LockSnapshot> = std::mem::take(&mut *list);
        for known in earlier {
            if known.crate_dir != crate_dir {
                list.push(known);
            }
        }
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        list.push(LockSnapshot { crate_dir: crate_dir.to_string(), source: text.clone() });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok((result, claim));
}

/// What:
/// `fn journal_path(root: &Path, pid: u32) -> PathBuf` is where a process's journal lives.
/// Why:
/// The directory is named after the process id,
/// so a recovering run can tell whose it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journalPath = (root: string, pid: number): string => join(root, JOURNAL_DIR, String(pid));
/// ```
fn journal_path(root: &Path, pid: u32) -> PathBuf {
    return root.join(JOURNAL_DIR).join(pid.to_string());
}

/// What:
/// `pub fn process_identity(pid: u32) -> Option<String>` is the start time of a process (field 22 of `/proc/<pid>/stat`),
/// or `None` when the process does not exist.
/// Why:
/// A process id can be reused;
/// the pair of process id and start time cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processIdentity(pid: number): string | undefined
/// ```
pub fn process_identity(pid: u32) -> Option<String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
    // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compute(); } catch { return undefined; }
    // ```
    // Typed binding: `text: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let text: String = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The command name is in parentheses and may contain spaces; the fields after the last `)` are plain.
    let after: &str = &text[text.rfind(')')? + 1..];
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // Typed binding: `fields: Vec<&str>`.
    let fields: Vec<&str> = after.split_whitespace().collect();
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return fields.get(19).map(ToString::to_string);
}

/// What:
/// `pub fn proc_usable() -> bool` says whether `/proc` can be read for process identities.
/// Why:
/// In a chroot or a container `/proc` may be absent;
/// a process that cannot be looked up must be assumed alive,
/// or a running transaction would be "recovered" under its own feet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const procUsable = (): boolean => existsSync('/proc/self/stat');
/// ```
pub fn proc_usable() -> bool {
    return Path::new("/proc/self/stat").exists();
}

/// What:
/// `pub fn process_exists(pid: u32, recorded_identity: &str) -> bool` says whether the process that wrote a journal is still running.
/// Why:
/// Without a usable `/proc` the answer is yes (never recover what cannot be shown dead);
/// with it,
/// the process must exist and,
/// when the journal recorded a start time,
/// have that start time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processExists(pid: number, recordedIdentity: string): boolean
/// ```
pub fn process_exists(pid: u32, recorded_identity: &str) -> bool {
    if !proc_usable() {
        return true;
    }
    // `match`: picks the arm whose pattern fits the value.
    return match process_identity(pid) {
        // `Some`: the present or the absent value of an `Option`.
        Some(now) => recorded_identity.is_empty() || now == recorded_identity,
        // `None`: the present or the absent value of an `Option`.
        None => false,
    };
}

/// What:
/// `pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()>` writes this process's journal:
/// the owner record,
/// every snapshot,
/// and last the index,
/// each synchronized,
/// with the journal directories synchronized after.
/// Why:
/// Cargo must not be started until the originals are safely copied;
/// the index is the commit record,
/// so a journal without it is known to be incomplete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[]): void
/// ```
pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()> {
    let directory: PathBuf = journal_path(root, std::process::id());
    fs::create_dir_all(&directory)?;
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Typed binding: `identity: String`.
    let identity: String = process_identity(std::process::id()).unwrap_or_default();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_synced(&directory.join(OWNER_FILE), &format!("v1\n{}\n{identity}\n", std::process::id()))?;
    let mut index: String = String::new();
    for (position, snapshot) in snapshots.iter().enumerate() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&directory.join`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        write_synced(&directory.join(format!("{position}.lock")), &snapshot.source)?;
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    atomic_write(&directory.join(INDEX_FILE), &index)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory`: lends the value without copying or giving it away.
    sync_directory(&directory)?;
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let Some(parent) = directory.parent() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        sync_directory(parent)?;
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `fn encode_path(path: &Path) -> String` writes a path as the hexadecimal text of its bytes.
/// Why:
/// A lossy conversion would journal a different path than the one snapshotted when a directory name is not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const encodePath = (path: Buffer): string => path.toString('hex');
/// ```
fn encode_path(path: &Path) -> String {
    // `let mut`: a binding that may change.
    // Typed binding: `text: String`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    let mut text: String = String::new();
    for byte in path.as_os_str().as_bytes() {
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// What:
/// `fn decode_path(text: &str) -> Option<PathBuf>` reads back what `encode_path` wrote.
/// Why:
/// Recovery must find the exact manifest that was snapshotted;
/// an odd length or a non-hexadecimal digit is a corrupt record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decodePath = (text: string): Buffer | undefined => /^([0-9a-f]{2})*$/.test(text) ? Buffer.from(text, 'hex') : undefined;
/// ```
pub(crate) fn decode_path(text: &str) -> Option<PathBuf> {
    if text.len() % 2 != 0 {
        // `None`: the present or the absent value of an `Option`.
        return None;
    }
    // `let mut`: a binding that may change.
    // Typed binding: `bytes: Vec<u8>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut bytes: Vec<u8> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `position: usize`.
    let mut position: usize = 0;
    while position < text.len() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `pair: &str`.
        let pair: &str = text.get(position..position + 2)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
        bytes.push(u8::from_str_radix(pair, 16).ok()?);
        position += 2;
    }
    // `Some`: the present or the absent value of an `Option`.
    // What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
    // Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    return Some(PathBuf::from(std::ffi::OsString::from_vec(bytes)));
}

/// What:
/// `pub fn add_manifests(root: &Path, edits: &[ManifestEdit]) -> io::Result<()>` records the manifests a `--breaking` run may rewrite
/// in this process's journal,
/// each with its crate root and its text before the run.
/// Why:
/// A killed run must be able to put them back too,
/// root by root.
/// Every snapshot is synchronized before the versioned index is published,
/// and the index ends with a count,
/// so a truncated record is detected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addManifests(root: string, edits: ManifestEdit[]): void  // throws when the journal cannot be written
/// ```
pub fn add_manifests(root: &Path, edits: &[ManifestEdit]) -> io::Result<()> {
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = journal_path(root, std::process::id());
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `let mut`: a binding that may change.
    // Typed binding: `index: String`.
    let mut index: String = String::from("v1\n");
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    for (position, edit) in edits.iter().enumerate() {
        if edit.crate_dir.contains('\n') || edit.crate_dir.contains('\t') {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the crate root {:?} contains a line feed or tab, which the journal cannot list", edit.crate_dir)));
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&directory.join`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        write_synced(&directory.join(format!("m{position}.toml")), &edit.before)?;
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        index.push_str(&format!("{}\t{}\n", edit.crate_dir, encode_path(&edit.path)));
    }
    // Borrow `&format`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    index.push_str(&format!("end {}\n", edits.len()));
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    atomic_write(&directory.join(MANIFEST_INDEX_FILE), &index)?;
    // Borrow `&directory`: lends the value without copying or giving it away.
    return sync_directory(&directory);
}

/// What:
/// `pub fn finish(root: &Path)` deletes this process's journal and synchronizes the journal directory.
/// Why:
/// The deletion must be durable too,
/// or a completed journal could reappear after a power loss and be "recovered" over newer work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function finish(root: string): void
/// ```
pub fn finish(root: &Path) {
    let _ = fs::remove_dir_all(journal_path(root, std::process::id()));
    // Borrow `&root.join`: lends the value without copying or giving it away.
    // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
    // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void expr;   // or try { expr } catch {}
    // ```
    let _ = sync_directory(&root.join(JOURNAL_DIR));
}

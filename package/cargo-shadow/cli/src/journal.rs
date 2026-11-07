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
use std::os::unix::ffi::OsStringExt;
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
pub use crate::journal_replace::{create_directories_synced, create_directories_with, create_exclusive, unique_suffix, KeepAs, ReplaceOutcome, file_name_text, no_hook, replace_if_holds, replace_if_holds_with_hooks, replace_keeping, replace_keeping_with_hook, replace_with_swap, sync_directory};
/// Re-exports the recovery entry points,
/// which live in `journal_recover.rs`.
pub use crate::journal_recover::{RecoveryReport, recover, recover_report, recover_report_with, recover_with};
/// Re-exports the transaction handle,
/// which lives in `journal_transaction.rs`.
pub use crate::journal_transaction::{Transaction, begin};

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

/// The file in which the containment (a control group,
/// or in the weaker path a process group) of the children a transaction started is recorded,
/// one record per line.
pub(crate) const GROUPS_FILE: &str = "groups";

/// The file in which the mount id and inode of the journal directory are recorded when the journal is made,
/// so a recovery that sees the directory differently refuses (contract assumption c).
pub(crate) const VIEW_FILE: &str = "view";

/// The file by which a person tells a recovery that every process or worker of the run has stopped.
pub(crate) const ACKNOWLEDGED_FILE: &str = "acknowledged";

/// The first line of a well-formed acknowledgement.
pub(crate) const ACKNOWLEDGED_HEADER: &str = "acknowledged v1";

/// The number of kept files past which every run warns (count,
/// total size,
/// folder).
/// Owner answer of round 6,
/// "Warn past a count,
/// never refuse (Recommended)":
/// nothing is refused or deleted past it.
/// The value is the warning point carried over from round 5,
/// a delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// the rate at which files are kept has not been measured.
pub const KEPT_FILES_WARNING_COUNT: usize = 1_000;

/// The file whose exclusive lock a transaction,
/// or a recovery,
/// holds on the whole workspace while it lives.
/// One lock for all roots is the fewest locks that exclude every conflict (owner rule:
/// "Use as few locks as possible.").
pub const WORKSPACE_LOCK: &str = ".cache/cargo-shadow/lock";

/// The record written last that lists every file a recovery needs,
/// with its length.
pub(crate) const READY_FILE: &str = "ready";

/// The suffix of a journal directory whose transaction completed and whose removal may not have.
pub(crate) const DONE_SUFFIX: &str = ".done";

/// The file in a journal directory that names the manifests a `--breaking` run may rewrite.
/// Its first line is `v1`,
/// each following line is `<crate root>`,
/// a tab and the hexadecimal bytes of the path,
/// and its last line is `end <count>`,
/// so a truncated file is recognized.
pub(crate) const MANIFEST_INDEX_FILE: &str = "manifests.txt";

/// What:
/// `pub(crate) fn write_synced(path: &Path, text: &str) -> io::Result<()>` creates a file,
/// writes it and forces it to disk.
/// Why:
/// A journal record counts as written only when its bytes are durable,
/// before cargo mutates the originals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeSynced(path: string, text: string): void  // openSync, writeSync, fsyncSync
/// ```
pub(crate) fn write_synced(path: &Path, text: &str) -> io::Result<()> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `file: fs::File`.
    let mut file: fs::File = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    file.write_all(text.as_bytes())?;
    return file.sync_all();
}

#[cfg(test)]
thread_local! {
    /// A hook that runs inside `atomic_write` after the temporary file is whole and synced and before the rename publishes it,
    /// so a
    /// test can stop or kill a real publisher at that boundary.
    static BEFORE_PUBLISH: std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
/// `pub(crate) fn before_publish_for_test(hook: Option<Rc<dyn Fn()>>)` sets the hook that runs between the temporary file and the
/// rename of an atomic publication on this thread.
/// Why:
/// An interrupted publication is made by a real publisher that is killed there,
/// not built by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beforePublishForTest(hook?: () => void): void
/// ```
#[cfg(test)]
pub(crate) fn before_publish_for_test(hook: Option<std::rc::Rc<dyn Fn()>>) {
    BEFORE_PUBLISH.with(|slot| { *slot.borrow_mut() = hook; });
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
    let (temp, _device, _inode): (PathBuf, u64, u64) = create_exclusive(&directory, &file_name_text(target), text, 0o644, &mut unique_suffix)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&temp`: lends the value without copying or giving it away.
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
    #[cfg(test)]
    if let Some(hook) = BEFORE_PUBLISH.with(|slot| { return slot.borrow().clone(); }) {
        hook();
    }
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
pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> (Vec<String>, Option<io::Error>) {
    return restore_if_unchanged_with(root, snapshots, produced, &no_hook);
}

/// What:
/// `pub fn restore_if_unchanged_with(root, snapshots, produced, between) -> (Vec<String>, Option<io::Error>)` is
/// `restore_if_unchanged` with a hook that runs between the exchange and the link.
/// Why:
/// The warnings of every root come back beside the first failure,
/// and a failed root does not stop the others (round 6,
/// 5b 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchangedWith(root: string, snapshots: LockSnapshot[], produced: LockSnapshot[], between: Hook): [string[], Error | undefined]
/// ```
pub fn restore_if_unchanged_with(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot], between: &dyn Fn(&Path)) -> (Vec<String>, Option<io::Error>) {
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::Restore) {
        return (Vec::new(), Some(io::Error::other("injected restore failure")));
    }
    let mut warnings: Vec<String> = Vec::new();
    let mut first_failure: Option<io::Error> = None;
    // A root that fails does not stop the others: every root still holding unapproved output gets its chance, and every retained
    // path is reported (round 6, 5b 2).
    for snapshot in snapshots {
        if let Err(error) = restore_one(root, snapshot, produced, between, &mut warnings) {
            warnings.push(format!("{}/{LOCKFILE_NAME} could not be restored: {error}", snapshot.crate_dir));
            first_failure.get_or_insert(error);
        }
    }
    return (warnings, first_failure);
}

/// What:
/// `fn restore_one(root, snapshot, produced, between, warnings) -> io::Result<()>` puts one lockfile back when it still holds what the run produced.
/// Why:
/// One root's failure must not stop the restore of the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreOne(root: string, snapshot: LockSnapshot, produced: LockSnapshot[], between: Hook, warnings: string[]): void
/// ```
fn restore_one(root: &Path, snapshot: &LockSnapshot, produced: &[LockSnapshot], between: &dyn Fn(&Path), warnings: &mut Vec<String>) -> io::Result<()> {
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
                return Ok(());
            }
        };
        if live == snapshot.source {
            return Ok(());
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
                return Ok(());
            }
            None => {
                warnings.push(format!(
                    "{}/{LOCKFILE_NAME} differs from the snapshot, but this run did not write it, so another process did; it was left as it is, not restored",
                    snapshot.crate_dir
                ));
                return Ok(());
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
            // Macro `format!`: builds or prints text (or a list) from a template.
            ReplaceOutcome::Displaced(copy) => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was put back to its snapshot; the file the exchange displaced is kept as {}",
                snapshot.crate_dir,
                copy.display()
            )),
            // Macro `format!`: builds or prints text (or a list) from a template.
            ReplaceOutcome::Undone(copy) => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed again during the restore; the undo put the file it first displaced back, and the file the undo displaced is kept as {}",
                snapshot.crate_dir,
                copy.display()
            )),
        }
    return Ok(());
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
/// `pub fn make_durable(path: &Path) -> io::Result<()>` syncs a file and the directory that lists it;
/// an absent file is skipped.
/// Why:
/// Review finding (4b2 3),
/// and the owner's rule "Force at leaving":
/// what a command promises when it leaves is on disk before the
/// snapshots that could undo it are removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function makeDurable(path: string): void  // fsync the file, then its directory
/// ```
pub fn make_durable(path: &Path) -> io::Result<()> {
    // `match`: picks the arm whose pattern fits the value.
    match fs::File::open(path) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        Ok(file) => {
            #[cfg(test)]
            if crate::failpoints::fires(crate::failpoints::Point::OutputFileSync) {
                return Err(io::Error::other("injected failure to sync an output file"));
            }
            file.sync_all()?;
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => return Err(error),
    }
    // `match`: picks the arm whose pattern fits the value.
    let directory_synced: io::Result<()> = match path.parent() {
        // `Some`: the present or the absent value of an `Option`.
        Some(parent) if !parent.as_os_str().is_empty() => {
            #[cfg(test)]
            if crate::failpoints::fires(crate::failpoints::Point::OutputDirectorySync) {
                return Err(io::Error::other("injected failure to sync the directory of an output file"));
            }
            sync_directory(parent)
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        _ => Ok(()),
    };
    // A test build traces the barrier by the name of the file once both syncs returned, so a test of the ordinary `finish` or of a
    // recovery sees the real output barrier succeed, and not a callback of its own (certification, third and fourth passes, A4, V5).
    #[cfg(test)]
    if directory_synced.is_ok() {
        crate::bounded_run::trace(|| { return format!("durable {}", path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default()); });
    }
    return directory_synced;
}

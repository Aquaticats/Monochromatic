//! What:
//! Compare-before-replace for the lockfile transaction,
//! and a journal that lets the next run finish what a killed run left.
//! Why:
//! The install and the update snapshot every lockfile,
//! let cargo rewrite them,
//! check the new versions against the policy,
//! and
//! put the snapshot back on a refusal.
//! Owner rules:
//! "never discard user data" and "use as few locks as possible".
//! So there is no
//! lock:
//! a restore replaces a lockfile only if it still holds what this run produced (anything else was written by someone
//! else meanwhile and is left alone with a warning),
//! and the replacement is a write to a sibling file and an atomic rename.
//! A process killed between cargo's write and the restore leaves a journal directory with the snapshots;
//! the next run reads it,
//! checks the live lock against the policy,
//! keeps it if it passes,
//! and otherwise saves it beside the lockfile as
//! `Cargo.lock.refused` before putting the snapshot back,
//! so no lockfile content is ever thrown away.
//! `cargo -Zunstable-options update --lockfile-path`,
//! which would resolve into a private file and publish it with one rename,
//! does not exist on the pinned nightly (checked 2026-11-03:
//! `cargo update --help` and `cargo check --help` list no such option).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal.ts
//! export function restoreIfUnchanged({ root, snapshots, produced }): string[]  // warnings
//! export function begin / finish / recover
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `atomic_write`,
/// `restore_if_unchanged`,
/// `begin`,
/// `finish`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports `io` from the standard library so this file can name it unqualified.
/// Why:
/// `io` is used by `atomic_write`,
/// `restore_if_unchanged`,
/// `begin`,
/// `read_journal`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { io } from 'std';
/// ```
use std::cell::RefCell;
/// What:
/// `use std::io;` imports
/// `io`
/// so this file can name them unqualified.
/// Why:
/// `io::Result` and `io::Error` are the file-system error types of the journal functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { } from 'node:fs'; // errors are thrown
/// ```
use std::io;
/// What:
/// `use std::io::Write;` imports
/// `Write`
/// so this file can name them unqualified.
/// Why:
/// `Write` is the trait that adds `write_all` to an open file,
/// which `save_aside` uses to fill a file it just created exclusively.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node's fs.writeSync is a free function; no trait import is needed.
/// ```
use std::io::Write;
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is used by `sibling_temp`,
/// `atomic_write`,
/// `restore_if_unchanged`,
/// `journal_path`,
/// and others;
/// `PathBuf` is used by `sibling_temp`,
/// `atomic_write`,
/// `restore_if_unchanged`,
/// `journal_path`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};
/// What:
/// `pub use crate::journal_replace::{ReplaceOutcome, file_name_text, no_hook, replace_if_holds, replace_if_holds_with_hooks};` imports and re-exports
/// the atomic compare-and-replace,
/// so callers keep writing `crate::journal::replace_if_holds`.
/// Why:
/// The file was split to stay under the line limit;
/// the callers and the tests are unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { ReplaceOutcome, replaceIfHolds, replaceIfHoldsWithHooks } from './journal-replace.ts';
/// ```
pub use crate::journal_replace::{ReplaceOutcome, file_name_text, no_hook, replace_if_holds, replace_if_holds_with_hooks};

/// What:
/// `use crate::constants::LOCKFILE_NAME;` imports `LOCKFILE_NAME` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LOCKFILE_NAME` is used by `restore_if_unchanged`,
/// `recover`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::LOCKFILE_NAME;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports `LockSnapshot`,
/// `read_locks` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockSnapshot` is used by `restore_if_unchanged`,
/// `begin`,
/// `read_journal`,
/// `recover`;
/// `read_locks` is used by `recover`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{LockSnapshot, read_locks};
/// Imports `ManifestEdit`,
/// one manifest a `--breaking` run may rewrite.
use crate::manifest_edit::ManifestEdit;

/// Where journals live,
/// relative to the repository root.
pub const JOURNAL_DIR: &str = ".cache/cargo-shadow/journal";

/// The extension appended to a lockfile name when a refused live lock is saved before the snapshot goes back.
pub const REFUSED_SUFFIX: &str = ".refused";

/// The file in a journal directory that names the crate roots,
/// one per line,
/// in the order of the snapshot files.
const INDEX_FILE: &str = "index.txt";

/// The path of a sibling file used for the write half of an atomic replace.
pub(crate) fn sibling_temp(target: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = target.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    name.push(format!(".cargo-shadow-{}.tmp", std::process::id()));
    return target.with_file_name(name);
}

/// Writes `text` to `target` by writing a sibling file and renaming it over the target.
pub fn atomic_write(target: &Path, text: &str) -> io::Result<()> {
    let temp: PathBuf = sibling_temp(target);
    fs::write(&temp, text)?;
    let renamed: io::Result<()> = fs::rename(&temp, target);
    if renamed.is_err() {
        let _ = fs::remove_file(&temp);
    }
    return renamed;
}






/// What:
/// `fn snapshot_of(snapshots, crate_dir) -> Option<&LockSnapshot>` finds the snapshot of one crate root,
/// or `None`.
/// Why:
/// The restore needs the lockfile this run produced for a root,
/// to tell it from one a concurrent writer left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshotOf = (snapshots: Snapshot[], crateDir: string) => snapshots.find((s) => s.crateDir === crateDir);
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
/// `pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>>`
/// puts each snapshot back only when the live lockfile still holds what this run produced.
/// Why:
/// A live lockfile that differs from `produced` was changed by another process after cargo finished;
/// replacing it would discard
/// that change,
/// so it is left alone and named in a warning.
/// A root that `produced` does not list was never written by this run,
/// so unknown ownership never authorizes a restore.
/// One that already equals the snapshot needs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchanged({ root, snapshots, produced }): string[]
/// ```
pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return restore_if_unchanged_with(root, snapshots, produced, &no_hook);
}

/// What:
/// `pub fn restore_if_unchanged_with(root, snapshots, produced, between)` is `restore_if_unchanged` with a hook that runs
/// after a lockfile was compared and before it is replaced.
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
        let live: String = fs::read_to_string(&path).unwrap_or_default();
        if live == snapshot.source {
            continue;
        }
        // Review finding (unknown ownership): only a lockfile this run is known to have written may be replaced. A root the run
        // never recorded as produced, or one that holds something else than was recorded, belongs to somebody else.
        let expected: Option<&LockSnapshot> = snapshot_of(produced, &snapshot.crate_dir);
        // `match`: picks the arm whose pattern fits the value.
        match expected {
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            Some(found) if found.source == live => {}
            // `Some`: the present or the absent value of an `Option`.
            Some(_) => {
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      Text building and printing are written as one call instead of string concatenation.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                warnings.push(format!(
                    "{}/{LOCKFILE_NAME} was changed by another process while the step ran; it was left as that process wrote it, not restored",
                    snapshot.crate_dir
                ));
                continue;
            }
            // `None`: the present or the absent value of an `Option`.
            None => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                warnings.push(format!(
                    "{}/{LOCKFILE_NAME} differs from the snapshot, but this run did not write it, so another process did; it was left as it is, not restored",
                    snapshot.crate_dir
                ));
                continue;
            }
        }
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&path`: lends the value without copying or giving it away.
        match replace_if_holds(&path, &live, &snapshot.source, between)? {
            ReplaceOutcome::Replaced => {}
            // Macro `format!`: builds or prints text (or a list) from a template.
            ReplaceOutcome::LeftAlone => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed by another process while the step ran; it was left as that process wrote it, not restored",
                snapshot.crate_dir
            )),
            // Macro `format!`: builds or prints text (or a list) from a template.
            ReplaceOutcome::Preserved(copy) => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed twice by other processes while the step ran; the file displaced is kept as {}",
                snapshot.crate_dir,
                copy.display()
            )),
        }
    }
    return Ok(warnings);
}

/// What:
/// `pub fn note_produced(root, crate_dir, produced)` records the lockfile of one crate root as this run's own output,
/// right after a cargo call of this run succeeded there.
/// Why:
/// The rollback replaces a lockfile only when it still holds what this run produced;
/// recording each write as it happens means a failure halfway never has to guess the content from the file system,
/// where another writer's content would be indistinguishable from the run's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function noteProduced({ root, crateDir, produced }: { root: string; crateDir: string; produced: LockSnapshot[] }): void
/// ```
pub fn note_produced(root: &Path, crate_dir: &str, produced: &RefCell<Vec<LockSnapshot>>) {
    // A lockfile that cannot be read right now is not recorded; an unrecorded root is never restored.
    let source: String = match fs::read_to_string(root.join(crate_dir).join(LOCKFILE_NAME)) {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => return,
    };
    // `let mut`: a binding that may change.
    let mut list = produced.borrow_mut();
    // Borrow `&mut *list`: lends the value without copying or giving it away.
    // Typed binding: `earlier: Vec<LockSnapshot>`.
    let earlier: Vec<LockSnapshot> = std::mem::take(&mut *list);
    for known in earlier {
        if known.crate_dir != crate_dir {
            list.push(known);
        }
    }
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    list.push(LockSnapshot { crate_dir: crate_dir.to_string(), source });
}

/// What:
/// `fn save_aside(target, text) -> io::Result<PathBuf>` writes `text` to a new file next to `target`
/// named `target`,
/// then `target.1`,
/// `target.2`,
/// ... ,
/// the first name that does not exist yet,
/// and returns the path.
/// Why:
/// A refused lockfile is evidence the user may need;
/// a later refusal must never replace an earlier saved copy,
/// and `create_new` makes the claim of a name atomic,
/// so two runs never pick the same one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function saveAside(target: string, text: string): string  // writeFileSync(name, text, { flag: 'wx' }) per candidate name
/// ```
fn save_aside(target: &Path, text: &str) -> io::Result<PathBuf> {
    // Typed binding: `base: String`.
    let base: String = file_name_text(target);
    // `let mut`: a binding that may change.
    // Typed binding: `number: u32`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut number: u32 = 0;
    loop {
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // Typed binding: `name: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let name: String = if number == 0 { base.clone() } else { format!("{base}.{number}") };
        // Typed binding: `candidate: PathBuf`.
        // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
        // Why:      Joined or stored paths are owned; paths passed in are borrowed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let candidate: PathBuf = target.with_file_name(name);
        // `create_new(true)` fails with `AlreadyExists` instead of truncating, the exclusive-create flag `wx` of Node.
        let opened = fs::OpenOptions::new().write(true).create_new(true).open(&candidate);
        // `match`: picks the arm whose pattern fits the value.
        match opened {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(mut file) => {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                file.write_all(text.as_bytes())?;
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                return Ok(candidate);
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => number += 1,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => return Err(error),
        }
    }
}

/// Where this process's journal lives.
fn journal_path(root: &Path, pid: u32) -> PathBuf {
    return root.join(JOURNAL_DIR).join(pid.to_string());
}

/// The file in a journal directory that names the manifests a `--breaking` run may rewrite,
/// one absolute path per line,
/// in the order of the `m<position>.toml` files that hold their text before the run.
const MANIFEST_INDEX_FILE: &str = "manifests.txt";

/// What:
/// `pub fn add_manifests(root, edits) -> io::Result<()>` records the manifests a `--breaking` run may rewrite in this process's
/// journal,
/// with their text before the run.
/// Why:
/// Delegated pick (2026-11-03):
/// cargo's breaking update rewrites manifests as well as the lockfile,
/// so a killed run must be
/// able to put them back too.
/// A path with a line feed cannot be listed one per line,
/// so it stops the run before cargo runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addManifests(root: string, edits: ManifestEdit[]): void  // throws when the journal cannot be written
/// ```
pub fn add_manifests(root: &Path, edits: &[ManifestEdit]) -> io::Result<()> {
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = journal_path(root, std::process::id());
    // `let mut`: a binding that may change.
    // Typed binding: `index: String`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut index: String = String::new();
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    for (position, edit) in edits.iter().enumerate() {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Typed binding: `path_text: String`.
        let path_text: String = edit.path.to_string_lossy().to_string();
        if path_text.contains('\n') {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the manifest path {path_text:?} contains a line feed, which the journal cannot list")));
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&edit.before`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        fs::write(directory.join(format!("m{position}.toml")), &edit.before)?;
        // Borrow `&path_text`: lends the value without copying or giving it away.
        index.push_str(&path_text);
        index.push('\n');
    }
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    return atomic_write(&directory.join(MANIFEST_INDEX_FILE), &index);
}

/// What:
/// `pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()>` writes the journal of this process:
/// every snapshot
/// and the list of crate roots.
/// Why:
/// If this process is killed before `finish`,
/// the next run finds the snapshots here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[]): void
/// ```
pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()> {
    let directory: PathBuf = journal_path(root, std::process::id());
    fs::create_dir_all(&directory)?;
    let mut index: String = String::new();
    for (position, snapshot) in snapshots.iter().enumerate() {
        fs::write(directory.join(format!("{position}.lock")), &snapshot.source)?;
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    return atomic_write(&directory.join(INDEX_FILE), &index);
}

/// Removes this process's journal;
/// a missing journal is fine.
pub fn finish(root: &Path) {
    let _ = fs::remove_dir_all(journal_path(root, std::process::id()));
}

/// Reads the snapshots recorded in a journal directory.
fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>> {
    let index: String = fs::read_to_string(directory.join(INDEX_FILE))?;
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    for (position, crate_dir) in index.lines().enumerate() {
        let source: String = fs::read_to_string(directory.join(format!("{position}.lock")))?;
        snapshots.push(LockSnapshot { crate_dir: crate_dir.to_string(), source });
    }
    return Ok(snapshots);
}

/// Whether a process with this id exists.
/// It reads Linux `/proc`,
/// and the crate refuses to compile on other systems (`lib.rs`),
/// so a missing `/proc` cannot make a
/// running process look dead;
/// a reused id reads as alive,
/// which only delays a recovery.
pub fn process_exists(pid: u32) -> bool {
    return Path::new("/proc").join(pid.to_string()).exists();
}

/// What:
/// `fn recover_manifests(directory, between, messages) -> bool` puts back the manifests a killed `--breaking` run journaled,
/// saving the text it replaces as `<name>.refused` (or the first free `<name>.refused.N`),
/// and says whether all were settled.
/// Why:
/// A journaled manifest that differs from its text before the run was rewritten by the killed run (or edited since,
/// which the
/// saved copy keeps),
/// and a manifest that cannot be put back keeps the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverManifests(directory: string, between: (path: string) => void, messages: string[]): boolean
/// ```
fn recover_manifests(directory: &Path, between: &dyn Fn(&Path), messages: &mut Vec<String>) -> bool {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `index: String`.
    let index: String = match fs::read_to_string(directory.join(MANIFEST_INDEX_FILE)) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => return true,
    };
    // `let mut`: a binding that may change.
    // Typed binding: `all_settled: bool`.
    let mut all_settled: bool = true;
    for (position, path_text) in index.lines().enumerate() {
        // Typed binding: `path: PathBuf`.
        let path: PathBuf = PathBuf::from(path_text);
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `before: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let before: String = match fs::read_to_string(directory.join(format!("m{position}.toml"))) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => {
                all_settled = false;
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{path_text}: the journaled text of a manifest could not be read ({error}); its journal was kept"));
                continue;
            }
        };
        // Borrow `&path`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Typed binding: `live: String`.
        let live: String = fs::read_to_string(&path).unwrap_or_default();
        if live == before {
            continue;
        }
        // Borrow `&path.with_file_name`: lends the value without copying or giving it away.
        // Typed binding: `saved: io::Result<PathBuf>`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let saved: io::Result<PathBuf> = save_aside(&path.with_file_name(format!("{}{REFUSED_SUFFIX}", file_name_text(&path))), &live);
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `restored: io::Result<(PathBuf, ReplaceOutco...`.
        let restored: io::Result<(PathBuf, ReplaceOutcome)> = match saved {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&path`: lends the value without copying or giving it away.
            Ok(copy) => match replace_if_holds(&path, &live, &before, between) {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(outcome) => Ok((copy, outcome)),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(error) => Err(error),
            },
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => Err(error),
        };
        // `match`: picks the arm whose pattern fits the value.
        match restored {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok((copy, ReplaceOutcome::Replaced)) => messages.push(format!(
                "{path_text} was rewritten by a run that was killed and its lockfile was not kept; the earlier manifest was put back and the rewritten text is kept as {}",
                copy.display()
            )),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok((_, ReplaceOutcome::LeftAlone)) => messages.push(format!("{path_text} was changed by another process while a killed run was being recovered; it was left as that process wrote it")),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok((_, ReplaceOutcome::Preserved(kept))) => messages.push(format!("{path_text} was changed twice by other processes while a killed run was being recovered; the file displaced is kept as {}", kept.display())),
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => {
                all_settled = false;
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{path_text}: could not recover a killed run: {error}; its journal was kept"));
            }
        }
    }
    return all_settled;
}

/// What:
/// `pub fn recover(root, is_alive, judge) -> Vec<String>` finishes the journals of processes that no longer exist.
/// Why:
/// For each such journal:
/// a live lock equal to its snapshot needs nothing;
/// a live lock the `judge` finds acceptable (no policy
/// findings for the versions it added) is kept;
/// otherwise the live lock is saved as `Cargo.lock.refused` (or the first free `Cargo.lock.refused.N`,
/// so an earlier copy
/// is never replaced) and the snapshot goes back by atomic rename,
/// after the live lock was read again and found to be still the judged one.
/// The journal is removed only when every root it lists was settled;
/// an unreadable journal or lockfile and a failed step keep it,
/// because it holds the only copy of the original.
/// A message says what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recover({ root, isAlive, judge }): string[]
/// ```
pub fn recover(root: &Path, is_alive: &dyn Fn(u32) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>) -> Vec<String> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return recover_with(root, is_alive, judge, &no_hook);
}

/// What:
/// `pub fn recover_with(root, is_alive, judge, between) -> Vec<String>` is `recover` with a hook that runs after the last
/// comparison of a lockfile and before it is replaced.
/// Why:
/// The same gap as in `restore_if_unchanged_with`,
/// for the recovery of a killed run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverWith({ root, isAlive, judge, between }): string[]
/// ```
pub fn recover_with(root: &Path, is_alive: &dyn Fn(u32) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path)) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let entries = match fs::read_dir(root.join(JOURNAL_DIR)) {
        Ok(found) => found,
        Err(_) => return messages,
    };
    for listed in entries.flatten() {
        let pid: u32 = match listed.file_name().to_string_lossy().parse() {
            Ok(found) => found,
            Err(_) => continue,
        };
        if pid == std::process::id() || is_alive(pid) {
            continue;
        }
        let directory: PathBuf = listed.path();
        // Review finding (recovery destroys the remaining copies): the journal holds the only copy of the original lockfiles,
        // so it is deleted only when every root it lists was settled. Any unreadable part or failed step keeps it.
        let before: Vec<LockSnapshot> = match read_journal(&directory) {
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{}: the journal of a killed run could not be read ({error}); it was kept", directory.display()));
                continue;
            }
        };
        let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
        let live: Vec<LockSnapshot> = match read_locks(root, &dirs) {
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{}: a lockfile of a killed run could not be read ({error}); its journal was kept", directory.display()));
                continue;
            }
        };
        // `let mut`: a binding that may change.
        // Typed binding: `settled: bool`.
        let mut settled: bool = true;
        // Whether a lockfile this journal lists was kept because its new versions pass the policy: the manifest edits that go
        // with it are kept too.
        let mut kept_a_lock: bool = false;
        for (old, now) in before.iter().zip(live.iter()) {
            if old.source == now.source {
                continue;
            }
            let problems: Vec<String> = judge(std::slice::from_ref(old), std::slice::from_ref(now));
            let path: PathBuf = root.join(&old.crate_dir).join(LOCKFILE_NAME);
            if problems.is_empty() {
                messages.push(format!("{}/{LOCKFILE_NAME} was left by a run that was killed; its new versions pass the policy, so it was kept", old.crate_dir));
                kept_a_lock = true;
                continue;
            }
            // Review finding (compare-before-replace): the judgement above can take a long time, so the file is read again
            // right before it is replaced, and a lockfile that is no longer the judged one is another writer's and is left.
            let latest: String = fs::read_to_string(&path).unwrap_or_default();
            if latest != now.source {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!(
                    "{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored",
                    old.crate_dir
                ));
                continue;
            }
            // Borrow `&path.with_file_name`: lends the value without copying or giving it away.
            // Typed binding: `saved: io::Result<PathBuf>`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            let saved: io::Result<PathBuf> = save_aside(&path.with_file_name(format!("{LOCKFILE_NAME}{REFUSED_SUFFIX}")), &now.source);
            // `match`: picks the arm whose pattern fits the value.
            // Typed binding: `restored: io::Result<(PathBuf, ReplaceOutco...`.
            let restored: io::Result<(PathBuf, ReplaceOutcome)> = match saved {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // `match`: picks the arm whose pattern fits the value.
                // Borrow `&path`: lends the value without copying or giving it away.
                Ok(copy) => match replace_if_holds(&path, &now.source, &old.source, between) {
                    // `Ok(...)`: builds the success or the failure variant of a `Result`.
                    Ok(outcome) => Ok((copy, outcome)),
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    Err(error) => Err(error),
                },
                Err(error) => Err(error),
            };
            // `match`: picks the arm whose pattern fits the value.
            match restored {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                Ok((copy, ReplaceOutcome::Replaced)) => messages.push(format!(
                    "{}/{LOCKFILE_NAME} was left by a run that was killed and holds versions the policy refuses ({}); it was saved as {} and the earlier lockfile was put back",
                    old.crate_dir,
                    problems.join("; "),
                    // Borrow `&copy`: lends the value without copying or giving it away.
                    file_name_text(&copy)
                )),
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                Ok((copy, ReplaceOutcome::LeftAlone)) => messages.push(format!(
                    "{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored (the refused text is kept as {})",
                    old.crate_dir,
                    copy.display()
                )),
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                Ok((_, ReplaceOutcome::Preserved(kept))) => messages.push(format!(
                    "{}/{LOCKFILE_NAME} was changed twice by other processes while a killed run was being recovered; the file displaced is kept as {}",
                    old.crate_dir,
                    kept.display()
                )),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(error) => {
                    settled = false;
                    // Macro `format!`: builds or prints text (or a list) from a template.
                    messages.push(format!("{}/{LOCKFILE_NAME}: could not recover a killed run: {error}; its journal was kept", old.crate_dir));
                }
            }
        }
        // Delegated pick (2026-11-03): the manifests of a killed `--breaking` run go back unless its lockfile was kept.
        if !kept_a_lock && !recover_manifests(&directory, between, &mut messages) {
            settled = false;
        }
        if settled {
            // Borrow `&directory`: lends the value without copying or giving it away.
            // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
            // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // void expr;   // or try { expr } catch {}
            // ```
            let _ = fs::remove_dir_all(&directory);
        }
    }
    return messages;
}

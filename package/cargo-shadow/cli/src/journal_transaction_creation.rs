//! What:
//! Creates the journal of one transaction:
//! the workspace lock taken first,
//! the journal directory's unique name,
//! the path text its index holds,
//! and the ready record that lists every file with its length.
//! Why:
//! A journal is only trusted by a recovery when it is complete and never reused,
//! so the naming, the ready record and the workspace lock are the rules that make it trustworthy,
//! and they are read on their own here.
//! `journal_transaction.rs` keeps the type those rules fill in and the methods it offers a run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-transaction-creation.ts
//! export function takeWorkspace(root: string): File  // throws when another run holds it
//! export function tryLock(file: FileHandle): boolean
//! export function encodePath(path: Buffer): string
//! function journalName(): string
//! function writeReady(directory: string, listed: string[], manifests: boolean): void
//! ```

/// What:
/// `use std::fs::{self, File, OpenOptions};` imports the file system calls, an open file and the opener.
/// Why:
/// `write_ready` asks each listed file for its length, `take_workspace` opens and truncates the workspace lock,
/// and `try_lock` takes an open file by reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync, openSync, FileHandle } from 'node:fs';
/// ```
use std::fs::{
    self,
    File,
    OpenOptions,
};
/// What:
/// `use std::io::{self, Write};` imports the input/output error type and the write trait.
/// Why:
/// Every step of a creation answers an `io::Result`, and the owner line of the workspace lock is written with `write_all`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::{
    self,
    Write,
};
/// What:
/// `use std::os::unix::ffi::OsStrExt;` imports the reading of an operating-system string as bytes.
/// Why:
/// `encode_path` writes a path's bytes as hexadecimal, and a path need not be valid UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has no equivalent: a string is always text.
/// ```
use std::os::unix::ffi::OsStrExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and the owned file path.
/// Why:
/// `take_workspace` builds the lockfile's own path (`PathBuf`) and every function lends a path in (`&Path`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::sync::atomic::{AtomicU64, Ordering};` imports the counter that keeps two journals of one process apart.
/// Why:
/// Two journals of one process can share a nanosecond reading, and only a counter tells them apart without a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // JS runs one thread, so a plain `let counter = 0` does the same job.
/// ```
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// What:
/// `use rustix::fs::{FlockOperation, flock};` imports the lock call and its operation.
/// Why:
/// The workspace lock is tried without waiting, which is the `NonBlockingLockExclusive` operation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { flock, FlockOperation } from 'rustix/fs';
/// ```
use rustix::fs::{
    FlockOperation,
    flock,
};

/// What:
/// `use crate::journal::{READY_FILE, WORKSPACE_LOCK, atomic_write, create_directories_synced, sync_directory};` imports the journal's names and its durable writes.
/// Why:
/// The ready record is published by an atomic write and its directory is synced, and the workspace lock is created with its directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { READY_FILE, WORKSPACE_LOCK, atomicWrite, createDirectoriesSynced, syncDirectory } from 'crate/journal';
/// ```
use crate::journal::{
    READY_FILE,
    WORKSPACE_LOCK,
    atomic_write,
    create_directories_synced,
    sync_directory,
};
/// The most journal names tried when one exists already.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
/// The names carry a clock,
/// a process id and a counter,
/// so a second try is the exception;
/// the collision count was not measured.
pub(crate) const MAX_JOURNAL_NAME_ATTEMPTS: u32 = 100;

/// The counter that keeps two journals of one process apart.
static JOURNAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What:
/// `pub fn encode_path(path: &Path) -> String` writes the bytes of a path as hexadecimal.
/// Why:
/// A path is bytes,
/// not necessarily UTF-8;
/// the journal must give back exactly the path it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const encodePath = (path: Buffer): string => path.toString('hex');
/// ```
pub fn encode_path(path: &Path) -> String {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let mut text: String = String::new();
    for byte in path.as_os_str().as_bytes() {
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// What:
/// `pub(crate) fn journal_name() -> String` is a unique,
/// sortable name for a new journal (clock,
/// process id,
/// counter).
/// Why:
/// The name carries no meaning for liveness;
/// it only has to be new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journalName = (): string => `${hrtime}-${pid}-${counter++}`;
/// ```
pub(crate) fn journal_name() -> String {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // Typed binding: `nanos: u128`.
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        Ok(elapsed) => elapsed.as_nanos(),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => 0,
    };
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!(
        "{nanos:040}-{}-{}",
        std::process::id(),
        JOURNAL_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
}

/// What:
/// `pub(crate) fn write_ready(directory: &Path, listed: &[String], manifests: bool) -> io::Result<()>` writes the ready record:
/// every listed file with its length.
/// Why:
/// A recovery trusts a journal only if every listed file is present with exactly that length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeReady(directory: string, listed: string[], manifests: boolean): void
/// ```
pub(crate) fn write_ready(
    directory: &Path,
    listed: &[String],
    manifests: bool,
) -> io::Result<()> {
    // `let mut`: a binding that may change.
    // Typed binding: `text: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut text: String = format!("v1\nmanifests {}\n", u8::from(manifests));
    for name in listed {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // Typed binding: `length: u64`.
        let length: u64 = fs::metadata(directory.join(name))?.len();
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        text.push_str(&format!("{name}\t{length}\n"));
    }
    // Borrow `&format`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    text.push_str(&format!("end {}\n", listed.len()));
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    atomic_write(&directory.join(READY_FILE), &text)?;
    return sync_directory(directory);
}

/// What:
/// `pub fn try_lock(file: &File) -> io::Result<bool>` tries for the exclusive lock without waiting;
/// `Ok(false)` means someone holds it.
/// Why:
/// Nothing in cargo-shadow waits for a lock:
/// a held lock is an answer,
/// not a reason to block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function tryLock(file: number): boolean  // flock(LOCK_EX | LOCK_NB)
/// ```
pub fn try_lock(file: &File) -> io::Result<bool> {
    // `match`: picks the arm whose pattern fits the value.
    return match flock(file, FlockOperation::NonBlockingLockExclusive) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(()) => Ok(true),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(code) if code == rustix::io::Errno::WOULDBLOCK => Ok(false),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(code) => Err(io::Error::from(code)),
    };
}

/// What:
/// `pub fn take_workspace(root: &Path) -> io::Result<Result<File, String>>` takes the workspace lock without waiting.
/// `Ok(Err(message))` means another run holds it,
/// and the message names the holder and says what to do.
/// Why:
/// One lock for the whole workspace excludes every conflict between transactions and recoveries;
/// the file is opened for writing
/// (some file systems emulate `flock` with write locks),
/// never truncated by the contender.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function takeWorkspace(root: string): File | string  // the string is the message when it is held
/// ```
pub fn take_workspace(root: &Path) -> io::Result<Result<File, String>> {
    // Typed binding: `path: PathBuf`.
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let path: PathBuf = root.join(WORKSPACE_LOCK);
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    if let Some(parent) = path.parent() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        create_directories_synced(parent)?;
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&path`: lends the value without copying or giving it away.
    // Typed binding: `file: File`.
    let file: File = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&file`: lends the value without copying or giving it away.
    if try_lock(&file)? {
        // Borrow `&file`: lends the value without copying or giving it away.
        // `let mut`: a binding that may change.
        let mut owner = &file;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        file.set_len(0)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Macro `format!`: builds or prints text (or a list) from a template.
        owner.write_all(format!("{}\n", std::process::id()).as_bytes())?;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(Ok(file));
    }
    // Borrow `&path`: lends the value without copying or giving it away.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Typed binding: `holder: String`.
    let holder: String = fs::read_to_string(&path)
        .unwrap_or_default()
        .trim()
        .to_string();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Ok(Err(format!(
        "another cargo-shadow run (pid {}) is changing the lockfiles of this workspace; run the command again when it has finished",
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        if holder.is_empty() {
            "unknown".to_string()
        } else {
            holder
        }
    )));
}

#[cfg(test)]
thread_local! {
    /// A hook that runs after each step of the preparation of a journal in `begin`,
    /// with the step's name,
    /// so a test can stop or kill a
    /// real wrapper at each real boundary of the preparation.
    static BEGIN_STEP: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(&str)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
/// `pub(crate) fn begin_step_for_test(hook: Option<Rc<dyn Fn(&str)>>)` sets the hook that runs after each step of `begin` on this thread.
/// Why:
/// The cuts of V3/V4 are made by killing a real wrapper at each step,
/// not by building the files of each state by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beginStepForTest(hook?: (step: string) => void): void
/// ```
#[cfg(test)]
pub(crate) fn begin_step_for_test(hook: Option<std::rc::Rc<dyn Fn(&str)>>) {
    BEGIN_STEP.with(|slot| {
        *slot.borrow_mut() = hook;
    });
}

/// What:
/// `pub(crate) fn at_step(name: &str)` runs the test hook,
/// if any,
/// after a step of the preparation of a journal.
/// Why:
/// The production build has no hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function atStep(name: string): void
/// ```
pub(crate) fn at_step(name: &str) {
    #[cfg(test)]
    if let Some(hook) = BEGIN_STEP.with(|slot| {
        return slot.borrow().clone();
    }) {
        hook(name);
    }
    #[cfg(not(test))]
    let _ = name;
}

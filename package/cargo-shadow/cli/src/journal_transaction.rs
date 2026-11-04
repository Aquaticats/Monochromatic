//! What:
//! The transaction handle of a run:
//! a journal directory of lockfile and manifest snapshots,
//! kept alive by an exclusive lock
//! that the run holds for as long as it lives.
//! Why:
//! The reviews of 2026-11-03 showed that naming a journal by process id and deciding liveness from `/proc` fails open
//! (a reused id,
//! an unreadable `/proc`,
//! a recorded identity from another namespace).
//! The kernel releases a lock when its holder dies,
//! so "the lock can be taken" is the proof that the owner is gone,
//! and trying for it never waits (owner rule of 2026-11-03:
//! "nothing should ever wait forever or indefinitely").
//! Every journal has its own unique name and is created,
//! never reused;
//! a `ready` record,
//! written last,
//! lists every file with its
//! length,
//! so a recovery can tell a complete journal from a partial one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-transaction.ts
//! export function begin(root: string, snapshots: LockSnapshot[], expectsManifests: boolean): Transaction
//! export class Transaction { addManifests(edits: ManifestEdit[]): void; finish(): void }
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::fs::{self, File, OpenOptions}` for this file.
use std::fs::{self, File, OpenOptions};
// Imports `std::io`.
/// Imports `std::io` for this file.
use std::io;
// Imports `std::os::unix::ffi::OsStrExt`.
/// Imports `std::os::unix::ffi::OsStrExt` for this file.
use std::os::unix::ffi::OsStrExt;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};
// Imports `std::sync::atomic::{AtomicU64, Ordering}`.
/// Imports `std::sync::atomic::{AtomicU64, Ordering}` for this file.
use std::sync::atomic::{AtomicU64, Ordering};

// Imports `rustix::fs::{FlockOperation, flock}`.
/// Imports `rustix::fs::{FlockOperation, flock}` for this file.
use rustix::fs::{FlockOperation, flock};

// Imports `crate::crates::LockSnapshot`.
/// Imports `crate::crates::LockSnapshot` for this file.
use crate::crates::LockSnapshot;
// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, J...`.
/// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, LOCK_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, READY_FILE, atomic_write, create_directories_synced, sync_directory, write_synced}` for this file.
use crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, LOCK_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, READY_FILE, atomic_write, create_directories_synced, sync_directory, write_synced};
// Imports `crate::manifest_edit::ManifestEdit`.
/// Imports `crate::manifest_edit::ManifestEdit` for this file.
use crate::manifest_edit::ManifestEdit;

/// The most journal names tried when one exists already.
/// A bound chosen for the owner's rule of 2026-11-03 ("nothing should ever wait forever or indefinitely");
/// the names carry a
/// clock,
/// a process id and a counter,
/// so a second try is already unreachable in practice.
const MAX_JOURNAL_NAME_ATTEMPTS: u32 = 100;

/// The counter that keeps two journals of one process apart.
static JOURNAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A journal held by its owner.
/// Dropping it without calling `finish` is what a killed run looks like to a recovery:
/// the journal stays,
/// its lock is free.
pub struct Transaction {
    /// The journal directory.
    directory: PathBuf,
    /// The locked file;
    /// closing it (by drop or by death) releases the lock.
    _lock: File,
    /// The files already listed for the ready record.
    listed: std::cell::RefCell<Vec<String>>,
}

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
/// `fn journal_name() -> String` is a unique,
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
fn journal_name() -> String {
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
    return format!("{nanos:040}-{}-{}", std::process::id(), JOURNAL_COUNTER.fetch_add(1, Ordering::Relaxed));
}

/// What:
/// `pub fn try_lock(file: &File) -> io::Result<bool>` tries for the exclusive lock without waiting;
/// `Ok(false)` means someone holds it.
/// Why:
/// Nothing in cargo-shadow waits for a lock:
/// a held lock is an answer (alive,
/// or being recovered),
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
/// `fn write_ready(directory: &Path, listed: &[String], manifests: bool) -> io::Result<()>` writes the ready record:
/// every listed file with its length.
/// Why:
/// A recovery trusts a journal only if every listed file is present with exactly that length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeReady(directory: string, listed: string[], manifests: boolean): void
/// ```
fn write_ready(directory: &Path, listed: &[String], manifests: bool) -> io::Result<()> {
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
/// `pub fn begin(root, snapshots, expects_manifests) -> io::Result<Transaction>` creates a new journal directory,
/// locks it,
/// writes the snapshots and,
/// when no manifests are expected,
/// the ready record.
/// Why:
/// Nothing may be mutated without a complete recovery record.
/// The directory is created,
/// never reused (an existing name is an error and a new name is tried),
/// so an earlier journal's snapshots are never truncated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[], expectsManifests: boolean): Transaction
/// ```
pub fn begin(root: &Path, snapshots: &[LockSnapshot], expects_manifests: bool) -> io::Result<Transaction> {
    // Typed binding: `journals: PathBuf`.
    let journals: PathBuf = root.join(JOURNAL_DIR);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&journals`: lends the value without copying or giving it away.
    create_directories_synced(&journals)?;
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `directory: Option<PathBuf>`.
    let mut created: Option<PathBuf> = None;
    for _attempt in 0..MAX_JOURNAL_NAME_ATTEMPTS {
        // Typed binding: `candidate: PathBuf`.
        let candidate: PathBuf = journals.join(journal_name());
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&candidate`: lends the value without copying or giving it away.
        match fs::create_dir(&candidate) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(()) => {
                // `Some`: the present or the absent value of an `Option`.
                created = Some(candidate);
                break;
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => return Err(error),
        }
    }
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    let Some(directory) = created else {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(io::Error::other(format!("{} could not be given a new journal name; run the command again", journals.display())));
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Typed binding: `lock: File`.
    let lock: File = OpenOptions::new().write(true).create_new(true).open(directory.join(LOCK_FILE))?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&lock`: lends the value without copying or giving it away.
    if !try_lock(&lock)? {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(io::Error::other("a journal that was just created is locked by someone else"));
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_synced(&directory.join(OWNER_FILE), &format!("v2\n{}\n", std::process::id()))?;
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `index: String`.
    let mut index: String = String::from("v2\n");
    // `let mut`: a binding that may change.
    // Typed binding: `listed: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut listed: Vec<String> = Vec::new();
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    for (position, snapshot) in snapshots.iter().enumerate() {
        // Typed binding: `name: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let name: String = format!("{position}.lock");
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&directory.join`: lends the value without copying or giving it away.
        write_synced(&directory.join(&name), &snapshot.source)?;
        listed.push(name);
        // Borrow `&snapshot.crate_dir`: lends the value without copying or giving it away.
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    // Borrow `&format`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    index.push_str(&format!("end {}\n", snapshots.len()));
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    atomic_write(&directory.join(INDEX_FILE), &index)?;
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    listed.insert(0, INDEX_FILE.to_string());
    if !expects_manifests {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&directory`: lends the value without copying or giving it away.
        write_ready(&directory, &listed, false)?;
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory`: lends the value without copying or giving it away.
    sync_directory(&directory)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&journals`: lends the value without copying or giving it away.
    sync_directory(&journals)?;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    return Ok(Transaction { directory, _lock: lock, listed: std::cell::RefCell::new(listed) });
}

/// The methods of the type above.
impl Transaction {
    /// What:
    /// `pub fn add_manifests(&self, edits) -> io::Result<()>` journals the manifests a `--breaking` run may rewrite and writes the
    /// ready record.
    /// Why:
    /// Their original text is the only copy once a run rewrites them,
    /// so it is synced before the run starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// addManifests(edits: ManifestEdit[]): void
    /// ```
    pub fn add_manifests(&self, edits: &[ManifestEdit]) -> io::Result<()> {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `let mut`: a binding that may change.
        // Typed binding: `index: String`.
        let mut index: String = String::from("v1\n");
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // `let mut`: a binding that may change.
        // Typed binding: `listed: Vec<String>`.
        let mut listed: Vec<String> = self.listed.borrow().clone();
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        for (position, edit) in edits.iter().enumerate() {
            if edit.crate_dir.contains('\n') || edit.crate_dir.contains('\t') {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the crate root {:?} contains a line feed or tab, which the journal cannot list", edit.crate_dir)));
            }
            // Typed binding: `name: String`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            let name: String = format!("m{position}.toml");
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&self.directory.join`: lends the value without copying or giving it away.
            write_synced(&self.directory.join(&name), &edit.before)?;
            listed.push(name);
            // Borrow `&format`: lends the value without copying or giving it away.
            // Macro `format!`: builds or prints text (or a list) from a template.
            index.push_str(&format!("{}\t{}\n", edit.crate_dir, encode_path(&edit.path)));
        }
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        index.push_str(&format!("end {}\n", edits.len()));
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.directory.join`: lends the value without copying or giving it away.
        atomic_write(&self.directory.join(MANIFEST_INDEX_FILE), &index)?;
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        listed.push(MANIFEST_INDEX_FILE.to_string());
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.directory`: lends the value without copying or giving it away.
        write_ready(&self.directory, &listed, true)?;
        *self.listed.borrow_mut() = listed;
        // Borrow `&self.directory`: lends the value without copying or giving it away.
        return sync_directory(&self.directory);
    }

    /// What:
    /// `pub fn finish(self) -> io::Result<()>` settles the transaction:
    /// the journal is renamed to its `.done` name (the durable
    /// completion marker),
    /// then removed.
    /// Why:
    /// A removal that stops partway leaves a `.done` directory,
    /// which the next recovery finishes,
    /// and never a journal that looks
    /// incomplete.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finish(): void
    /// ```
    pub fn finish(self) -> io::Result<()> {
        return self.finish_with(&|_path| { return Ok(()); });
    }

    /// What:
    /// `pub fn finish_with(self, make_durable) -> io::Result<()>` is `finish` with the barrier that makes the final outputs durable
    /// given by the caller (tests record it).
    /// Why:
    /// The owner's rule for durability is "Force at leaving": what a command promises when it leaves is on disk before the
    /// snapshots that could undo it are removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finishWith(makeDurable: (path: string) => void): void
    /// ```
    pub fn finish_with(self, _make_durable: &dyn Fn(&Path) -> io::Result<()>) -> io::Result<()> {
        // Borrow `&self.directory`: lends the value without copying or giving it away.
        return mark_done_and_remove(&self.directory);
    }
}

/// What:
/// `pub fn mark_done_and_remove(directory: &Path) -> io::Result<()>` renames a journal to its `.done` name,
/// syncs the parent,
/// removes the directory and syncs the parent again.
/// Why:
/// The rename is the completion marker;
/// failures of the removal are reported,
/// not ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markDoneAndRemove(directory: string): void
/// ```
pub fn mark_done_and_remove(directory: &Path) -> io::Result<()> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `parent: PathBuf`.
    let parent: PathBuf = match directory.parent() {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found.to_path_buf(),
        // `None`: the present or the absent value of an `Option`.
        None => PathBuf::from("."),
    };
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `name: String`.
    let name: String = directory.file_name().map(|found| { return found.to_string_lossy().to_string(); }).unwrap_or_default();
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Typed binding: `done: PathBuf`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let done: PathBuf = if name.ends_with(DONE_SUFFIX) { directory.to_path_buf() } else { parent.join(format!("{name}{DONE_SUFFIX}")) };
    if done != directory {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&done`: lends the value without copying or giving it away.
        fs::rename(directory, &done)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&parent`: lends the value without copying or giving it away.
        sync_directory(&parent)?;
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&done`: lends the value without copying or giving it away.
    fs::remove_dir_all(&done)?;
    // Borrow `&parent`: lends the value without copying or giving it away.
    return sync_directory(&parent);
}

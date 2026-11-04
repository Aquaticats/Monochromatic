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
use std::io::{self, Write};
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
use crate::bounded_run::set_group_log;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `crate::constants::LOCKFILE_NAME` for this file.
use crate::constants::LOCKFILE_NAME;
// Imports `crate::journal::{DONE_SUFFIX, GROUPS_FILE, ...`.
/// Imports `crate::journal::{DONE_SUFFIX, GROUPS_FILE, INDEX_FILE, JOURNAL_DIR, MANIFEST_INDEX_FILE, KEPT_FILES_WARNING_COUNT, QUARANTINE_DIR, OWNER_FILE, READY_FILE, WORKSPACE_LOCK, atomic_write, create_directories_synced, make_durable, sync_directory, write_synced}` for this file.
use crate::journal::{DONE_SUFFIX, GROUPS_FILE, INDEX_FILE, JOURNAL_DIR, MANIFEST_INDEX_FILE, KEPT_FILES_WARNING_COUNT, QUARANTINE_DIR, OWNER_FILE, READY_FILE, WORKSPACE_LOCK, atomic_write, create_directories_synced, make_durable, sync_directory, write_synced};
// Imports `crate::manifest_edit::ManifestEdit`.
/// Imports `crate::manifest_edit::ManifestEdit` for this file.
use crate::manifest_edit::ManifestEdit;

/// The most journal names tried when one exists already.
/// Unmeasured:
/// the names carry a clock,
/// a process id and a counter,
/// so a second try is the exception;
/// what must be measured is the collision count.
/// The owner's rule of 2026-11-03 justifies that a bound exists,
/// not this number.
const MAX_JOURNAL_NAME_ATTEMPTS: u32 = 100;

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

/// A transaction held by its owner:
/// the workspace lock and the journal.
/// Dropping it without calling `finish` is what a killed run looks like to a recovery:
/// the journal stays,
/// the lock is free,
/// and the process groups of the children it started are recorded in the journal.
pub struct Transaction {
    /// The repository root.
    root: PathBuf,
    /// The journal directory.
    directory: PathBuf,
    /// The locked workspace file;
    /// closing it (by drop or by death) releases the lock.
    _workspace: File,
    /// The files already listed for the ready record.
    listed: std::cell::RefCell<Vec<String>>,
    /// The crate roots whose lockfiles this transaction covers.
    crate_dirs: Vec<String>,
    /// The manifests journaled for it.
    manifests: std::cell::RefCell<Vec<PathBuf>>,
    /// Whether the manifests were journaled already.
    manifests_added: std::cell::Cell<bool>,
    /// The notice that the kept folder is large, when it is.
    kept_notice: Option<String>,
}

/// The methods of the type above.
impl Drop for Transaction {
    /// What:
    /// `fn drop(&mut self)` stops recording child process groups for this thread.
    /// Why:
    /// The record belongs to this transaction's journal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        set_group_log(None);
    }
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
    let file: File = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&path)?;
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
    let holder: String = fs::read_to_string(&path).unwrap_or_default().trim().to_string();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Ok(Err(format!(
        "another cargo-shadow run (pid {}) is changing the lockfiles of this workspace; run the command again when it has finished",
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        if holder.is_empty() { "unknown".to_string() } else { holder }
    )));
}

/// What:
/// `pub fn begin(root, snapshots, expects_manifests) -> io::Result<Transaction>` takes the workspace lock,
/// creates a new journal
/// directory,
/// writes the snapshots and,
/// when no manifests are expected,
/// the ready record.
/// Why:
/// Nothing may be mutated without a complete recovery record,
/// and no other transaction or recovery may run meanwhile.
/// The directory is created,
/// never reused,
/// so an earlier journal's snapshots are never truncated.
/// The process group of every child this thread starts is recorded in the journal,
/// so a recovery can see a survivor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[], expectsManifests: boolean): Transaction
/// ```
pub fn begin(root: &Path, snapshots: &[LockSnapshot], expects_manifests: bool) -> io::Result<Transaction> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `workspace: File`.
    let workspace: File = match take_workspace(root)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(held) => held,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(message) => return Err(io::Error::other(message)),
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    refuse_over_cap(root)?;
    let journals: PathBuf = root.join(JOURNAL_DIR);
    create_directories_synced(&journals)?;
    let mut created: Option<PathBuf> = None;
    for _attempt in 0..MAX_JOURNAL_NAME_ATTEMPTS {
        let candidate: PathBuf = journals.join(journal_name());
        match fs::create_dir(&candidate) {
            Ok(()) => {
                created = Some(candidate);
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let Some(directory) = created else {
        return Err(io::Error::other(format!("{} could not be given a new journal name; run the command again", journals.display())));
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_synced(&directory.join(OWNER_FILE), &format!("v3\n{}\n", std::process::id()))?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&directory.join`: lends the value without copying or giving it away.
    write_synced(&directory.join(GROUPS_FILE), "")?;
    let mut index: String = String::from("v2\n");
    let mut listed: Vec<String> = Vec::new();
    for (position, snapshot) in snapshots.iter().enumerate() {
        let name: String = format!("{position}.lock");
        write_synced(&directory.join(&name), &snapshot.source)?;
        listed.push(name);
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    index.push_str(&format!("end {}\n", snapshots.len()));
    atomic_write(&directory.join(INDEX_FILE), &index)?;
    listed.insert(0, INDEX_FILE.to_string());
    if !expects_manifests {
        write_ready(&directory, &listed, false)?;
    }
    sync_directory(&directory)?;
    sync_directory(&journals)?;
    // `Some`: the present or the absent value of an `Option`.
    set_group_log(Some(directory.join(GROUPS_FILE)));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(Transaction {
        root: root.to_path_buf(),
        directory,
        _workspace: workspace,
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        listed: std::cell::RefCell::new(listed),
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        crate_dirs: snapshots.iter().map(LockSnapshot::crate_dir_text).collect(),
        // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
        manifests: std::cell::RefCell::new(Vec::new()),
        // Constructor `Cell::new(`: an empty owned collection or a boxed value.
        manifests_added: std::cell::Cell::new(false),
        kept_notice: None,
    });
}

/// The methods of the type above.
impl Transaction {
    /// What:
    /// `pub fn kept_notice(&self) -> Option<&str>` is the warning to print when the kept folder holds many files.
    /// Why:
    /// The owner decided to warn past a count and never to refuse.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// keptNotice(): string | undefined
    /// ```
    pub fn kept_notice(&self) -> Option<&str> {
        return self.kept_notice.as_deref();
    }

    /// What:
    /// `pub fn add_manifests(&self, edits) -> io::Result<()>` journals the manifests a `--breaking` run may rewrite and writes the
    /// ready record;
    /// it can be called once.
    /// Why:
    /// Their original text is the only copy once a run rewrites them,
    /// so it is synced before the run starts.
    /// The payload of a ready transaction is frozen:
    /// a second call would rewrite snapshots a recovery may rely on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// addManifests(edits: ManifestEdit[]): void
    /// ```
    pub fn add_manifests(&self, edits: &[ManifestEdit]) -> io::Result<()> {
        if self.manifests_added.replace(true) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(io::Error::other("the manifests of this transaction were journaled already; its recovery payload is not rewritten"));
        }
        let mut index: String = String::from("v1\n");
        let mut listed: Vec<String> = self.listed.borrow().clone();
        for (position, edit) in edits.iter().enumerate() {
            if edit.crate_dir.contains('\n') || edit.crate_dir.contains('\t') {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the crate root {:?} contains a line feed or tab, which the journal cannot list", edit.crate_dir)));
            }
            let name: String = format!("m{position}.toml");
            write_synced(&self.directory.join(&name), &edit.before)?;
            listed.push(name);
            index.push_str(&format!("{}\t{}\n", edit.crate_dir, encode_path(&edit.path)));
        }
        index.push_str(&format!("end {}\n", edits.len()));
        atomic_write(&self.directory.join(MANIFEST_INDEX_FILE), &index)?;
        listed.push(MANIFEST_INDEX_FILE.to_string());
        write_ready(&self.directory, &listed, true)?;
        *self.listed.borrow_mut() = listed;
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        // `.collect()`: runs the iterator and gathers the items into the bound collection type.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.map(`: transforms or selects items with a closure.
        *self.manifests.borrow_mut() = edits.iter().map(|edit| { return edit.path.clone(); }).collect();
        return sync_directory(&self.directory);
    }

    /// What:
    /// `pub fn finish(self) -> io::Result<()>` settles the transaction:
    /// the outputs are made durable,
    /// then the journal is renamed
    /// to its `.done` name (the durable completion marker) and removed.
    /// Why:
    /// The owner's rule is "Force at leaving":
    /// the lockfiles and manifests the run leaves are on disk before the snapshots that
    /// could undo them go.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finish(): void
    /// ```
    pub fn finish(self) -> io::Result<()> {
        // Borrow `&make_durable`: lends the value without copying or giving it away.
        return self.finish_with(&make_durable);
    }

    /// What:
    /// `pub fn finish_with(self, durable) -> io::Result<()>` is `finish` with the barrier that makes an output durable given by the
    /// caller (tests record it).
    /// Why:
    /// The order is the point:
    /// every output is made durable before the completion marker exists.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finishWith(makeDurable: (path: string) => void): void
    /// ```
    pub fn finish_with(self, durable: &dyn Fn(&Path) -> io::Result<()>) -> io::Result<()> {
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::Finish) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(io::Error::other("injected finish failure"));
        }
        // Borrow `&self.crate_dirs`: lends the value without copying or giving it away.
        for crate_dir in &self.crate_dirs {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&self.root.join`: lends the value without copying or giving it away.
            durable(&self.root.join(crate_dir).join(LOCKFILE_NAME))?;
        }
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        for manifest in self.manifests.borrow().iter() {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            durable(manifest)?;
        }
        return mark_done_and_remove(&self.directory);
    }
}

/// What:
/// `pub fn mark_done_and_remove(directory: &Path) -> io::Result<()>` renames a journal to its `.done` name,
/// syncs the parent,
/// removes the directory and syncs the parent again.
/// Why:
/// The rename is the completion marker,
/// and the parent is synced before any destructive cleanup even when the name already ends
/// in `.done` (review finding 4b2 5):
/// another run's rename may not be durable yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markDoneAndRemove(directory: string): void
/// ```
pub fn mark_done_and_remove(directory: &Path) -> io::Result<()> {
    let parent: PathBuf = match directory.parent() {
        Some(found) => found.to_path_buf(),
        None => PathBuf::from("."),
    };
    let name: String = directory.file_name().map(|found| { return found.to_string_lossy().to_string(); }).unwrap_or_default();
    let done: PathBuf = if name.ends_with(DONE_SUFFIX) { directory.to_path_buf() } else { parent.join(format!("{name}{DONE_SUFFIX}")) };
    if done != directory {
        fs::rename(directory, &done)?;
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&parent`: lends the value without copying or giving it away.
    sync_directory(&parent)?;
    fs::remove_dir_all(&done)?;
    return sync_directory(&parent);
}

/// What:
/// `fn refuse_over_cap(root: &Path) -> io::Result<()>` refuses a new transaction when the kept directory holds more than
/// `KEPT_FILES_WARNING_COUNT` files,
/// and lists them (path,
/// size,
/// age).
/// Why:
/// Review finding (4b 4):
/// kept files are other writers' data and are never deleted by this tool,
/// so growth is stopped here by
/// refusing to mutate further until a person has looked at them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refuseOverCap(root: string): void  // throws with the listing when over the cap
/// ```
fn refuse_over_cap(root: &Path) -> io::Result<()> {
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = root.join(QUARANTINE_DIR);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&directory`: lends the value without copying or giving it away.
    let Ok(listing) = fs::read_dir(&directory) else {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    };
    // `let mut`: a binding that may change.
    // Typed binding: `entries: Vec<(PathBuf, u64, u64)>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut entries: Vec<(PathBuf, u64, u64)> = Vec::new();
    for entry in listing.flatten() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        let metadata = entry.metadata()?;
        // Closure `|modified|`: an anonymous function, like a TS arrow function.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
        // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { return compute(); } catch { return undefined; }
        // ```
        // Adapter `.and_then(`: transforms or selects items with a closure.
        // Typed binding: `age: u64`.
        let age: u64 = metadata.modified().ok().and_then(|modified| { return modified.elapsed().ok(); }).map(|elapsed| { return elapsed.as_secs() / 86_400; }).unwrap_or(0);
        entries.push((entry.path(), metadata.len(), age));
    }
    if entries.len() <= KEPT_FILES_WARNING_COUNT {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }
    entries.sort();
    // Closure `|(path, size, age)|`: an anonymous function, like a TS arrow function.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `shown: Vec<String>`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let shown: Vec<String> = entries.iter().take(20).map(|(path, size, age)| { return format!("  {} ({size} bytes, {age} days old)", path.display()); }).collect();
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(io::Error::other(format!(
        "{} holds {} kept files (more than {KEPT_FILES_WARNING_COUNT}): files this tool moved aside instead of deleting them because they may be someone's work. Look at them, move or archive what you no longer need, then run the command again. The first {}:\n{}",
        directory.display(),
        entries.len(),
        shown.len(),
        shown.join("\n")
    )));
}

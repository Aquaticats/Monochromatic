//! What:
//!  The permanent binding of a request identity to the hash of its immutable content: `ids/<xx>/<id>` under a
//!  repository-owned root, created complete by a no-replace link and never overwritten or removed.
//! Why:
//!  "Durable acceptance needs no ref write" (`doc/planning/cli-git-commits-never-fail.md`) accepts a request only for the
//!  one client that created the binding; another content under the same identity is refused.
//!  The seventh review shows that `O_EXCL` creation makes the name visible before its content is written, so a client
//!  that dies in between strands the identity. A temporary file is written and synced first and linked into place, so a
//!  binding is never visible partial.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writeFileSync(tmp, hash + '\n'); fsyncSync(tmp); linkSync(tmp, final) /* throws EEXIST */; unlinkSync(tmp);
//! ```

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::commit_integrity::{is_identity, unhex};

/// What:
///  `enum Bound { ... }`: what `bind` found or did.
/// Why:
///  The caller proceeds only on `Created` (and may resume on `SameContent`); `DifferentContent` is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Bound = { kind: 'created' } | { kind: 'same-content' } | { kind: 'different-content'; recorded: string };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Bound {
    /// This call created the binding.
    Created,
    /// The binding existed and holds the same hash: a resend.
    SameContent,
    /// The binding exists and holds another hash (given): the request is refused.
    DifferentContent(String),
}

/// What:
///  `enum IdentityError { ... }`: why binding failed.
/// Why:
///  A binding that exists but is not a complete hash (written by an earlier implementation, or damaged) is reported, not
///  trusted and not replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class IdentityError extends Error { kind: 'invalid-identity' | 'invalid-hash' | 'unreadable-binding' | 'io'; }
/// ```
#[derive(Debug)]
pub enum IdentityError {
    /// The identity is not 32 lowercase hexadecimal characters.
    InvalidIdentity,
    /// The hash is not 64 lowercase hexadecimal characters.
    InvalidHash,
    /// The binding file exists and is not a complete hash; its path is given.
    UnreadableBinding(PathBuf),
    /// A file system call failed.
    Io(io::Error),
}

// What:     `impl From<A> for B` teaches the compiler to turn an `A` into a `B`, which is what `?` uses to convert an error on the way out.
// Why:      Let every file call below propagate its I/O error as this module's own error with `?`.
//
// In TS you'd write (pseudocode):
// ```ts
// class B { static from(a: A): B { ... } }
// ```
impl From<io::Error> for IdentityError {
    /// Lets `?` turn an `io::Error` into `IdentityError::Io` (as in `commit_ledger.rs`).
    fn from(error: io::Error) -> IdentityError {
        return IdentityError::Io(error);
    }
}

/// What:
///  The path `root/ids/<first two characters>/<id>` of an identity's binding.
/// Why:
///  Two characters of fan-out keep a directory small; the identity is validated so it can never escape `root`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bindingPath(root: string, id: string): string;
/// ```
pub fn binding_path(root: &Path, id: &str) -> Result<PathBuf, IdentityError> {
    if !is_identity(id) {
        return Err(IdentityError::InvalidIdentity);
    }
    return Ok(root.join("ids").join(&id[..2]).join(id));
}

/// A temporary name in `directory` that no other call shares: `.tmp-<16 random hex>`.
pub fn temporary_name(directory: &Path) -> io::Result<PathBuf> {
    let mut random: [u8; 8] = [0; 8];
    getrandom::fill(&mut random).map_err(|error| { return io::Error::other(error.to_string()); })?;
    // What:     `format!("...{}...", x)` builds an owned `String`, putting each argument where `{}` is (a standard macro, called like a function).
    // Why:      Build a label, a path or a line of text from parts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `...${x}...`
    // ```
    return Ok(directory.join(format!(".tmp-{}", super::commit_integrity::hex(&random))));
}

// What:     `thread_local!` declares a variable with one copy per thread (a macro); `RefCell<Option<Vec<PathBuf>>>` is a
//           list that can be changed through a shared reference, present only while a trace is on.
//           Siblings: a `static Mutex`, shared by all threads.
// Why:      A test must see which directories an operation synced and in what order. The trace is off by default, per thread,
//           so a test running beside others is not disturbed and production calls pay one cell check.
//
// In TS you'd write (pseudocode):
// ```ts
// let trace: string[] | undefined; // per worker thread
// ```
thread_local! {
    static TRACE: std::cell::RefCell<Option<Vec<PathBuf>>> = const { std::cell::RefCell::new(None) };
}

// What:     a second per-thread cell holding an optional plain function (`fn(&str)`), the fault point callback. `fn(&str)` is a
//           function pointer (TS: a function value); it carries no captured variable, so a test shares state with it through
//           statics (a `Barrier`), not through a closure.
// Why:      A race test must stop one writer exactly at the existence or link decision so the unsafe schedule always runs; the
//           callback is called by name at those decisions and is `None`, doing nothing, in production.
//
// In TS you'd write (pseudocode):
// ```ts
// let faultPoint: ((name: string) => void) | undefined; // per worker thread
// ```
thread_local! {
    static FAULT: std::cell::Cell<Option<fn(&str)>> = const { std::cell::Cell::new(None) };
}

/// Sets, on this thread, the function called at each named fault point of the durable-record components (`None` clears it).
pub fn set_fault_point(callback: Option<fn(&str)>) {
    FAULT.with(|cell| { cell.set(callback); });
}

/// Calls this thread's fault point callback, if any, with the name of the decision point reached; a no-op otherwise.
pub fn fault_point(name: &str) {
    // What:     `if let Some(x) = value { ... }` runs the block only when the `Option` (or `Result`) holds a value, naming it `x` inside; otherwise it is skipped.
    // Why:      Handle the present case without a full `match`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const x = value; ... }
    // ```
    if let Some(callback) = FAULT.with(|cell| { return cell.get(); }) {
        callback(name);
    }
}

/// Starts recording, on this thread, the directories that `sync_directory` syncs, in order.
pub fn start_sync_trace() {
    // What:     `Vec::new()` makes an empty growable list (TS: `[]`); the type on the left says what it will hold. Siblings: `[T; N]`, a fixed-size array, and `&[T]`, a borrowed view.
    // Why:      Collect results whose number is not known beforehand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found: T[] = [];
    // ```
    TRACE.with(|trace| { *trace.borrow_mut() = Some(Vec::new()); });
}

/// Stops recording and returns the directories synced since `start_sync_trace`, in order.
pub fn take_sync_trace() -> Vec<PathBuf> {
    return TRACE.with(|trace| { return trace.borrow_mut().take().unwrap_or_default(); });
}

/// Syncs a directory so entries made in it survive power loss.
pub fn sync_directory(directory: &Path) -> io::Result<()> {
    TRACE.with(|trace| {
        // What:     `Some(x)` is the present case of `Option` (TS: the value itself, as opposed to `undefined`); `None` is the absent case.
        // Why:      Return or store a value that may be missing without using a sentinel.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return x; // or undefined
        // ```
        if let Some(list) = trace.borrow_mut().as_mut() {
            list.push(directory.to_path_buf());
        }
    });
    return File::open(directory)?.sync_all();
}

/// The parent of `path`, with the empty parent of a bare relative name read as the current directory.
fn parent_of(path: &Path) -> Option<PathBuf> {
    let parent: &Path = path.parent()?;
    if parent.as_os_str().is_empty() {
        return Some(PathBuf::from("."));
    }
    return Some(parent.to_path_buf());
}

/// What:
///  Syncs `from` and each of its ancestors up to and including `anchor`, the directory the caller knows to be durable.
///  Both paths are made absolute first, so a relative path works and the empty parent of a bare name is never synced.
/// Why:
///  A name found already linked may be visible without being durable (its creator may have died before its directory sync),
///  and so may the names of the directories above it; every successful path makes the whole chain durable itself, through an
///  anchor, and never stops at the first directory that merely exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncChain(from: string, anchor: string): void;
/// ```
pub fn sync_chain(from: &Path, anchor: &Path) -> io::Result<()> {
    let absolute_anchor: PathBuf = std::path::absolute(anchor)?;
    let mut current: PathBuf = std::path::absolute(from)?;
    if !current.starts_with(&absolute_anchor) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the path is not below its durable anchor"));
    }
    loop {
        sync_directory(&current)?;
        if current == absolute_anchor {
            return Ok(());
        }
        // What:     `match value { pattern => ..., }` compares a value against patterns top to bottom and runs the arm that fits; the compiler checks that every case is covered.
        // Why:      Branch on the outcome of a call that can succeed or fail, or on which variant a value is.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (value.kind) { case ...: ...; }
        // ```
        match parent_of(&current) {
            Some(parent) => current = parent,
            None => return Ok(()),
        }
    }
}

/// What:
///  Creates `path` and every missing ancestor below `anchor`, then syncs the chain from `path` through `anchor` whether or not
///  this call created anything; returns the directories it created, outermost first. The anchor must exist and be durable.
/// Why:
///  Syncing a child directory does not make its name in the parent durable, and an ancestor that merely exists may be one another
///  creator made and never synced. The caller names the known-durable anchor and the whole chain below it is synced on every
///  successful path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createDirAllDurable(anchor: string, path: string): string[];
/// ```
pub fn create_dir_all_durable(anchor: &Path, path: &Path) -> io::Result<Vec<PathBuf>> {
    let absolute_path: PathBuf = std::path::absolute(path)?;
    let mut missing: Vec<PathBuf> = Vec::new();
    let mut current: PathBuf = absolute_path.clone();
    while !current.exists() {
        missing.push(current.clone());
        match parent_of(&current) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    missing.reverse();
    for directory in &missing {
        match fs::create_dir(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    sync_chain(&absolute_path, anchor)?;
    return Ok(missing);
}

/// What:
///  Binds `id` to `hash` for good, or reports what is already bound.
///  Writes `<hash>\n` to a temporary file, syncs it, links it to the binding path without replacing, removes the
///  temporary name and syncs the directory.
/// Why:
///  `hard_link` fails with `AlreadyExists` when the binding exists, so exactly one caller creates it, and what it
///  creates is complete. A leftover `.tmp-*` file after a crash is harmless and is never read as a binding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bind(root: string, id: string, hash: string): Bound;
/// ```
pub fn bind(root: &Path, id: &str, hash: &str) -> Result<Bound, IdentityError> {
    let path: PathBuf = binding_path(root, id)?;
    if hash.len() != 64 || unhex(hash).is_none() {
        return Err(IdentityError::InvalidHash);
    }
    let directory: &Path = path.parent().ok_or(IdentityError::InvalidIdentity)?;
    create_dir_all_durable(root, directory)?;
    let temporary: PathBuf = temporary_name(directory)?;
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(format!("{}\n", hash).as_bytes())?;
    file.sync_all()?;
    fault_point("bind:before-link");
    let linked: io::Result<()> = fs::hard_link(&temporary, &path);
    fs::remove_file(&temporary)?;
    match linked {
        Ok(()) => {
            sync_chain(directory, root)?;
            return Ok(Bound::Created);
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            // The creator may have died before syncing the name; this call makes the chain durable up to the root.
            sync_chain(directory, root)?;
            let recorded: String = read_binding(&path)?;
            if recorded == hash {
                return Ok(Bound::SameContent);
            }
            return Ok(Bound::DifferentContent(recorded));
        }
        Err(error) => return Err(IdentityError::Io(error)),
    }
}

/// What:
///  The hash an existing binding holds; an empty, short or non-hexadecimal file is `UnreadableBinding`.
/// Why:
///  Never trust or replace a binding that is not a complete hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readBinding(path: string): string;
/// ```
pub fn read_binding(path: &Path) -> Result<String, IdentityError> {
    let text: String = fs::read_to_string(path).map_err(|_| { return IdentityError::UnreadableBinding(path.to_path_buf()); })?;
    let hash: &str = text.strip_suffix('\n').unwrap_or("");
    if hash.len() != 64 || unhex(hash).is_none() {
        return Err(IdentityError::UnreadableBinding(path.to_path_buf()));
    }
    return Ok(hash.to_string());
}

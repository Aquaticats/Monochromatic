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

/// What:
///  The test seams of the durable-record components: a per-thread event log (every directory sync, and the file syncs and
///  truncations the ledger records) and a per-thread fault-point callback. They exist only with the Cargo feature `test-seams`;
///  without it `fault_point` and `record_event` are empty functions and nothing else is compiled.
/// Why:
///  A race or ordering test must stop a writer at a named decision point and read the order of the destructive and syncing
///  steps. Production builds carry no thread-local cell, no allocation and no callback, so the seam cannot change behavior or
///  panic there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // if (process.env.TEST_SEAMS) { events.push(name); faultPoint?.(name); }
/// ```
#[cfg(feature = "test-seams")]
mod seams {
    use std::cell::{Cell, RefCell};

    thread_local! {
        /// The event log of this thread while recording; `None` when off.
        pub static EVENTS: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
        /// The fault point callback of this thread.
        pub static FAULT: Cell<Option<fn(&str)>> = const { Cell::new(None) };
    }
}

/// Starts recording this thread's events: directory syncs, file syncs and truncations, in the order they are attempted and
/// completed (an event is recorded after its operation succeeded).
#[cfg(feature = "test-seams")]
pub fn start_events() {
    seams::EVENTS.with(|events| { *events.borrow_mut() = Some(Vec::new()); });
}

/// Stops recording and returns the events recorded since `start_events`.
#[cfg(feature = "test-seams")]
pub fn take_events() -> Vec<String> {
    return seams::EVENTS.with(|events| { return events.borrow_mut().take().unwrap_or_default(); });
}

/// Sets, on this thread, the function called at each named fault point (`None` clears it).
#[cfg(feature = "test-seams")]
pub fn set_fault_point(callback: Option<fn(&str)>) {
    seams::FAULT.with(|cell| { cell.set(callback); });
}

/// Records one event on this thread when a test is recording; an empty function in a production build.
pub fn record_event(name: &str) {
    #[cfg(feature = "test-seams")]
    seams::EVENTS.with(|events| {
        if let Some(list) = events.borrow_mut().as_mut() {
            list.push(name.to_string());
        }
    });
    #[cfg(not(feature = "test-seams"))]
    let _ = name;
}

/// Calls this thread's fault point callback with the name of the decision point reached; an empty function in a production build.
pub fn fault_point(name: &str) {
    #[cfg(feature = "test-seams")]
    if let Some(callback) = seams::FAULT.with(|cell| { return cell.get(); }) {
        callback(name);
    }
    #[cfg(not(feature = "test-seams"))]
    let _ = name;
}

/// Syncs a directory so entries made in it survive power loss. The event `dir:<path>` is recorded after the sync succeeded.
pub fn sync_directory(directory: &Path) -> io::Result<()> {
    File::open(directory)?.sync_all()?;
    record_event(&format!("dir:{}", directory.display()));
    return Ok(());
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
    let absolute_anchor: PathBuf = std::path::absolute(anchor)?;
    let absolute_path: PathBuf = std::path::absolute(path)?;
    // Containment is checked before anything is created: the path must lie below the anchor with no `..` component, and the
    // anchor must already exist (it is made by `init_root`, never silently here). Lexical only: symlinks below are the caller's.
    let escapes: bool = absolute_path.components().any(|part| { return part == std::path::Component::ParentDir; });
    if escapes || !absolute_path.starts_with(&absolute_anchor) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the path is not below its durable anchor"));
    }
    if !absolute_anchor.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "the durable anchor does not exist; initialize the root first"));
    }
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
    sync_chain(&absolute_path, &absolute_anchor)?;
    return Ok(missing);
}

/// What:
///  The initialized-root constructor: makes `root` a directory (creating it if missing, accepting it if an interrupted creator
///  left it) and makes its name durable by syncing `root` and then `known_durable_parent`, which the caller knows to be durable
///  by other means. Returns the absolute root to hand to `bind`, `write_spool`, `publish` and `append` as their anchor.
/// Why:
///  Syncing a directory does not make its own name durable, and an existence check cannot show that an earlier creator ever
///  synced; the root is the one place where that has to be established through a parent that is durable independently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initRoot(knownDurableParent: string, root: string): string;
/// ```
pub fn init_root(known_durable_parent: &Path, root: &Path) -> io::Result<PathBuf> {
    let absolute_parent: PathBuf = std::path::absolute(known_durable_parent)?;
    let absolute_root: PathBuf = std::path::absolute(root)?;
    if parent_of(&absolute_root).as_deref() != Some(absolute_parent.as_path()) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the root must be a direct child of the known-durable parent"));
    }
    match fs::create_dir(&absolute_root) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists && absolute_root.is_dir() => {}
        Err(error) => return Err(error),
    }
    sync_directory(&absolute_root)?;
    sync_directory(&absolute_parent)?;
    return Ok(absolute_root);
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

//! What:
//!  The shared durable-file helpers of the commit protocol's record components: a unique temporary name, the directory sync chain
//!  through a known-durable anchor, durable directory creation, the initialized root, and the test seams (an event log and fault points).
//! Why:
//!  Every record component (the inbox, the state log, the ledger, the spool) needs the same few operations and the same seams, so a test
//!  can count syncs and stop a writer at a named decision point. The permanent identity binding that this module once held is removed:
//!  revision 13 names an identity by the hash of the request's content, so no binding file exists.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const tmp = join(dir, `.tmp-${randomHex(8)}`); fsyncSync(fd); fsyncDirectory(dir); /* seams: events.push(name) */
//! ```

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

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
        /// A forcing error to inject on this thread: how many syncs succeed first, and the error number of the one that fails.
        pub static SYNC_FAILURE: Cell<Option<(u32, i32)>> = const { Cell::new(None) };
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

/// Makes the sync after the next `after` successful ones on this thread fail once with the error number `errno` (`None` clears it): a disk
/// that answers a forcing with `EIO`, `ENOSPC` or `EROFS`. Both file syncs and directory syncs of the durable-record components count.
#[cfg(feature = "test-seams")]
pub fn inject_sync_error(failure: Option<(u32, i32)>) {
    seams::SYNC_FAILURE.with(|cell| { cell.set(failure); });
}

/// The error this sync must fail with, if a test asked for one at this point; the injection fires once.
fn injected_sync_error() -> Option<io::Error> {
    #[cfg(feature = "test-seams")]
    return seams::SYNC_FAILURE.with(|cell| {
        return match cell.get() {
            Some((0, errno)) => {
                cell.set(None);
                Some(io::Error::from_raw_os_error(errno))
            }
            Some((left, errno)) => {
                cell.set(Some((left - 1, errno)));
                None
            }
            None => None,
        };
    });
    #[cfg(not(feature = "test-seams"))]
    return None;
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

/// Records one event built only when a test is recording; the argument is never built in a production build.
pub fn record_event_lazy(build: impl FnOnce() -> String) {
    #[cfg(feature = "test-seams")]
    record_event(&build());
    #[cfg(not(feature = "test-seams"))]
    let _ = build;
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

/// Syncs a file's data and records the event `file-sync:<path>` (test seam) after it succeeded; every file sync of the durable-record
/// components goes through here, so a test can count them.
pub fn sync_file(file: &File, path: &Path) -> io::Result<()> {
    if let Some(error) = injected_sync_error() {
        return Err(error);
    }
    file.sync_all()?;
    record_event_lazy(|| { return format!("file-sync:{}", path.display()); });
    return Ok(());
}

/// Syncs a directory so entries made in it survive power loss. The event `dir:<path>` is recorded after the sync succeeded.
pub fn sync_directory(directory: &Path) -> io::Result<()> {
    if let Some(error) = injected_sync_error() {
        return Err(error);
    }
    File::open(directory)?.sync_all()?;
    record_event_lazy(|| { return format!("dir:{}", directory.display()); });
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
///  by other means. Returns the absolute root to hand to `write_spool`, `accept` and the ledger's `append` as their anchor.
/// Why:
///  Syncing a directory does not make its own name durable, and an existence check cannot show that an earlier creator ever
///  synced; the root is the one place where that has to be established through a parent that is durable independently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initRoot(knownDurableParent: string, root: string): string;
/// ```
pub fn init_root(known_durable_parent: &Path, root: &Path) -> io::Result<PathBuf> {
    // The same lexical rule as descendant creation: a `..` spelling is refused before anything is created.
    let dotdot = |path: &Path| { return path.components().any(|part| { return matches!(part, std::path::Component::ParentDir); }); };
    if dotdot(known_durable_parent) || dotdot(root) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "a root path with a parent-directory component is refused"));
    }
    let absolute_parent: PathBuf = std::path::absolute(known_durable_parent)?;
    let absolute_root: PathBuf = std::path::absolute(root)?;
    if parent_of(&absolute_root).as_deref() != Some(absolute_parent.as_path()) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the root must be a direct child of the known-durable parent"));
    }
    // A root that is a symlink would put the protocol's state under a target whose ancestry the known-durable parent does not make
    // durable; it is refused (and so is anything that is not a plain directory).
    if let Ok(metadata) = fs::symlink_metadata(&absolute_root)
        && (metadata.file_type().is_symlink() || !metadata.is_dir())
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the root is a symlink or not a directory"));
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

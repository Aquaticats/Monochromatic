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
    return Ok(directory.join(format!(".tmp-{}", super::commit_integrity::hex(&random))));
}

// What:     `thread_local!` declares a variable with one copy per thread (a macro); `Cell<usize>` is a number that can be
//           changed through a shared reference. Siblings: a `static AtomicUsize`, shared by all threads.
// Why:      Tests need to see that a code path really asked for a directory sync, and a counter per thread is not disturbed by
//           the tests that run beside it.
//
// In TS you'd write (pseudocode):
// ```ts
// let syncs = 0; // per worker thread
// ```
thread_local! {
    static SYNCS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The number of directory syncs this thread has made, for tests that must see a path ask for one.
pub fn directory_sync_count() -> usize {
    return SYNCS.with(|count| { return count.get(); });
}

/// Syncs a directory so entries made in it survive power loss.
pub fn sync_directory(directory: &Path) -> io::Result<()> {
    SYNCS.with(|count| { count.set(count.get() + 1); });
    return File::open(directory)?.sync_all();
}

/// What:
///  Syncs `from` and each of its ancestors up to and including `to`.
/// Why:
///  A name found already linked may be visible without being durable (its creator may have died before its directory sync),
///  and so may the names of the directories above it; a resend or a recovery makes the whole chain durable itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncChain(from: string, to: string): void;
/// ```
pub fn sync_chain(from: &Path, to: &Path) -> io::Result<()> {
    let mut current: PathBuf = from.to_path_buf();
    loop {
        sync_directory(&current)?;
        if current == to || !current.starts_with(to) {
            return Ok(());
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Ok(()),
        }
    }
}

/// What:
///  Creates `path` and every missing ancestor, syncing each created directory and the parent that holds its name; returns the
///  directories it created, outermost first.
/// Why:
///  Syncing a child directory does not make its name in the parent durable, so after a power loss an acknowledged inbox or
///  binding directory could be absent. One helper makes the whole new hierarchy durable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createDirAllDurable(path: string): string[];
/// ```
pub fn create_dir_all_durable(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut missing: Vec<PathBuf> = Vec::new();
    let mut current: PathBuf = path.to_path_buf();
    while !current.exists() {
        missing.push(current.clone());
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
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
        sync_directory(directory)?;
        if let Some(parent) = directory.parent() {
            sync_directory(parent)?;
        }
    }
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
    create_dir_all_durable(directory)?;
    let temporary: PathBuf = temporary_name(directory)?;
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(format!("{}\n", hash).as_bytes())?;
    file.sync_all()?;
    let linked: io::Result<()> = fs::hard_link(&temporary, &path);
    fs::remove_file(&temporary)?;
    match linked {
        Ok(()) => {
            sync_directory(directory)?;
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

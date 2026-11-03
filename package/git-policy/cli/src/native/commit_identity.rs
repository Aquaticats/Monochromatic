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

/// Syncs a directory so entries made in it survive power loss.
pub fn sync_directory(directory: &Path) -> io::Result<()> {
    return File::open(directory)?.sync_all();
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
    fs::create_dir_all(directory)?;
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

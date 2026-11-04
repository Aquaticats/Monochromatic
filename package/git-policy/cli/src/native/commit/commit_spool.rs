//! What:
//!  The spool of a request: the captured bytes of each changed path, stored under `spool/<id>/<n>` with a manifest that
//!  is linked into place last, and a verifier that checks every file against the manifest and against the request.
//! Why:
//!  The spool is the source of truth for a request until its ledger line exists (`doc/planning/cli-git-commits-never-
//!  fail.md`, "Retention lifecycle and the spool closure"). Step 1 builds the component with no reclamation: this module
//!  has no function that deletes or shortens a spool file, and every file is linked into place only complete, so a file
//!  that exists is a file that was synced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const [n, bytes] of files.entries()) writeAtomic(`spool/${id}/${n}`, bytes); writeAtomic(manifest);
//! ```

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::commit_identity::{create_dir_all_durable, fault_point, record_event_lazy, sync_chain, sync_directory, sync_file, temporary_name};
use super::commit_integrity::{is_identity, sha256_hex};
use super::commit_request::Request;
use super::commit_state_log::read_regular;

/// What:
///  `enum SpoolError { ... }`: why a spool operation failed.
/// Why:
///  A missing manifest (`Incomplete`) is a spool a crash left half made, to be completed by the same client's resend;
///  `Mismatch` names the first file or manifest field that disagrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SpoolError extends Error { kind: 'invalid-identity' | 'incomplete' | 'mismatch' | 'conflict' | 'io'; }
/// ```
#[derive(Debug)]
pub enum SpoolError {
    /// The identity is not valid.
    InvalidIdentity,
    /// The spool has no manifest yet.
    Incomplete,
    /// A file, the manifest or the request disagrees (what, given).
    Mismatch(String),
    /// A spool for the identity exists with other content.
    Conflict,
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
impl From<io::Error> for SpoolError {
    /// Lets `?` turn an `io::Error` into `SpoolError::Io`.
    fn from(error: io::Error) -> SpoolError {
        return SpoolError::Io(error);
    }
}

/// The directory of an identity's spool: `root/spool/<id>`.
pub fn spool_directory(root: &Path, id: &str) -> Result<PathBuf, SpoolError> {
    if !is_identity(id) {
        // What:     `Err(e)` is the failure case of `Result` (TS: throwing `e`).
        // Why:      Report a failure to the caller as a value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // throw e;
        // ```
        return Err(SpoolError::InvalidIdentity);
    }
    // What:     `Ok(x)` is the success case of `Result` (TS: returning `x`); `Err(e)` is the failure case (TS: throwing `e`).
    // Why:      Return a value from a function that can also fail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return x;
    // ```
    return Ok(root.join("spool").join(id));
}

/// The manifest body for `files`: one `file <n> <length> <sha256>` line per file.
fn manifest_body(files: &[Vec<u8>]) -> String {
    let mut body: String = String::new();
    // What:     `.iter()` walks a list by reference; `.iter_mut()` lends each entry for change; `.into_iter()` consumes the list. The chained method (`any`, `all`, `map`, `find`) takes a closure.
    // Why:      Ask a question of every entry in one expression.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.some((x) => ...)
    // ```
    for (number, bytes) in files.iter().enumerate() {
        // What:     `format!("...{}...", x)` builds an owned `String`, putting each argument where `{}` is (a standard macro, called like a function).
        // Why:      Build a label, a path or a line of text from parts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `...${x}...`
        // ```
        body.push_str(&format!("file {} {} {}\n", number, bytes.len(), sha256_hex(bytes)));
    }
    return body;
}

/// What:
///  Writes `bytes` to `target` complete or not at all: a synced temporary file linked to `target` without replacing.
///  When `target` exists the call succeeds only if its bytes are identical.
/// Why:
///  A file that exists in a spool was synced whole, so a resend after a crash can skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkComplete(target: string, bytes: Uint8Array): void;
/// ```
fn link_complete(target: &Path, bytes: &[u8]) -> Result<(), SpoolError> {
    let directory: &Path = target.parent().ok_or(SpoolError::InvalidIdentity)?;
    // A resend finds the complete final file first and writes no duplicate (a full disk must not fail it); the name may have been linked
    // by a writer that died before syncing the directory, so the directory is synced before the answer.
    match read_regular(target) {
        Ok(existing) => {
            if existing == bytes {
                sync_directory(directory)?;
                return Ok(());
            }
            return Err(SpoolError::Conflict);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(SpoolError::Io(error)),
    }
    fault_point("spool:before-temporary");
    let temporary: PathBuf = temporary_name(directory)?;
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(bytes)?;
    sync_file(&file, &temporary)?;
    fault_point("spool:before-link");
    let linked: io::Result<()> = fs::hard_link(&temporary, target);
    // A failed cleanup after a successful link does not make the file absent; only the stale temporary name remains.
    let cleanup: io::Result<()> = fs::remove_file(&temporary);
    if let Err(cleanup_error) = &cleanup {
        record_event_lazy(|| { return format!("spool:cleanup-failed:{:?}", cleanup_error.kind()); });
    }
    match linked {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            // The name may have been linked by a writer that died before syncing the directory.
            sync_directory(directory)?;
            if read_regular(target)? == bytes {
                // What:     `return Ok(x)` / `return Err(e)` ends the function with a success or a failure value.
                // Why:      Hand the result to the caller.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return x; // or throw
                // ```
                return Ok(());
            }
            return Err(SpoolError::Conflict);
        }
        Err(error) => return Err(SpoolError::Io(error)),
    }
}

/// What:
///  Writes the spool of `id`: entry `n` of `files` is the new bytes of change `n` (file `n`), entry `n` of `old_files` the bytes
///  of that path at the captured base (file `n.old`), then the manifest, syncing the directory between.
///  Safe to call again with the same bytes after a crash; other bytes under the same identity give `Conflict`.
/// Why:
///  The base side of the three-way replay is part of the spool, so a replay needs no object of the base (a prune after a branch
///  rewrite cannot remove it). The manifest is linked last, so a spool with a manifest is complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeSpool(root: string, id: string, files: Uint8Array[], oldFiles: Uint8Array[]): void;
/// ```
pub fn write_spool(root: &Path, id: &str, files: &[Vec<u8>], old_files: &[Vec<u8>]) -> Result<(), SpoolError> {
    if files.len() != old_files.len() {
        return Err(SpoolError::Mismatch("file counts".to_string()));
    }
    let directory: PathBuf = spool_directory(root, id)?;
    create_dir_all_durable(root, &directory)?;
    for (number, bytes) in files.iter().enumerate() {
        link_complete(&directory.join(number.to_string()), bytes)?;
    }
    for (number, bytes) in old_files.iter().enumerate() {
        link_complete(&directory.join(format!("{}.old", number)), bytes)?;
    }
    sync_directory(&directory)?;
    let mut body: String = manifest_body(files);
    for (number, bytes) in old_files.iter().enumerate() {
        body.push_str(&format!("old {} {} {}\n", number, bytes.len(), sha256_hex(bytes)));
    }
    let manifest: String = format!("{}end {}\n", body, sha256_hex(body.as_bytes()));
    link_complete(&directory.join("manifest"), manifest.as_bytes())?;
    // Every successful path, a repeated write included, makes the chain through the caller's durable root durable itself.
    sync_chain(&directory, root)?;
    return Ok(());
}

/// What:
///  Checks the spool of `id` against its manifest and returns the SHA-256 of each new file, in order, then of each old file.
/// Why:
///  An integrity check that reads every byte: the manifest's own end hash, then each file's length and hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verifySpool(root: string, id: string): string[];
/// ```
pub fn verify_spool(root: &Path, id: &str) -> Result<Vec<String>, SpoolError> {
    let directory: PathBuf = spool_directory(root, id)?;
    let manifest: String = match read_regular(directory.join("manifest")).and_then(|bytes| { return String::from_utf8(bytes).map_err(|error| { return io::Error::new(io::ErrorKind::InvalidData, error); }); }) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(SpoolError::Incomplete),
        Err(error) => return Err(SpoolError::Io(error)),
    };
    let end_at: usize = manifest.rfind("end ").ok_or(SpoolError::Mismatch("manifest end".to_string()))?;
    let (body, end) = manifest.split_at(end_at);
    if end != format!("end {}\n", sha256_hex(body.as_bytes())) {
        return Err(SpoolError::Mismatch("manifest hash".to_string()));
    }
    // The manifest is canonical: `file 0` to `file n-1` in order, then `old 0` to `old n-1` in order, numeric indices only, so a
    // name read from it can never leave the spool directory and no record can be missing, duplicated or reordered.
    // Exact bytes: LF endings only, `end` at a line boundary, and the whole manifest equal to what the writer emits.
    if body.contains('\r') || !body.is_empty() && !body.ends_with('\n') {
        return Err(SpoolError::Mismatch("manifest line endings".to_string()));
    }
    let lines: Vec<&str> = body.lines().collect();
    if lines.len() % 2 != 0 {
        return Err(SpoolError::Mismatch("manifest record count".to_string()));
    }
    let count: usize = lines.len() / 2;
    // What:     `Vec::new()` makes an empty growable list (TS: `[]`); the type on the left says what it will hold. Siblings: `[T; N]`, a fixed-size array, and `&[T]`, a borrowed view.
    // Why:      Collect results whose number is not known beforehand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found: T[] = [];
    // ```
    let mut hashes: Vec<String> = Vec::new();
    for (position, line) in lines.iter().enumerate() {
        let kind: &str = if position < count { "file" } else { "old" };
        let number: usize = position % count.max(1);
        let parts: Vec<&str> = line.split(' ').collect();
        let canonical: bool = parts.len() == 4 && parts[0] == kind && parts[1] == number.to_string();
        if !canonical {
            return Err(SpoolError::Mismatch(format!("manifest record {} is not {} {}", position, kind, number)));
        }
        let name: String = if kind == "old" { format!("{}.old", number) } else { number.to_string() };
        let bytes: Vec<u8> = read_regular(directory.join(&name))?;
        // What:     `|x| expr` is a closure (an unnamed function; TS: `(x) => expr`). It is passed to a method that calls it.
        // Why:      Say in one place what to do for each element or error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (x) => expr
        // ```
        if bytes.len().to_string() != parts[2] || sha256_hex(&bytes) != parts[3] {
            return Err(SpoolError::Mismatch(format!("file {}", name)));
        }
        hashes.push(parts[3].to_string());
    }
    return Ok(hashes);
}

/// What:
///  Checks the spool of `request.id` against the request: one new and one old file per change, each hashing to what the change
///  records (a deletion's new file and an absent old entry's file are empty).
/// Why:
///  The record promises what the spool must hold; this is the check that the promise is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verifyAgainstRequest(root: string, request: Request): void;
/// ```
pub fn verify_against_request(root: &Path, request: &Request) -> Result<(), SpoolError> {
    let hashes: Vec<String> = verify_spool(root, &request.id)?;
    let count: usize = request.changes.len();
    if hashes.len() != count * 2 {
        return Err(SpoolError::Mismatch("file count".to_string()));
    }
    for (number, change) in request.changes.iter().enumerate() {
        let expected_new: String = if change.mode == "000000" { sha256_hex(&[]) } else { change.sha256.clone() };
        let expected_old: String = if change.old_mode == "000000" { sha256_hex(&[]) } else { change.old_sha256.clone() };
        if hashes[number] != expected_new || hashes[count + number] != expected_old {
            return Err(SpoolError::Mismatch(format!("change {}", number)));
        }
    }
    return Ok(());
}

/// What:
///  `struct ClosureEntry { ... }`: one path of a request with both sides of its replay, read from the spool alone.
///  `Option<(String, Vec<u8>)>` is the mode and bytes, or `None` for an absent side.
/// Why:
///  A replay after the base objects are gone (a branch rewrite and a prune) still has the base side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClosureEntry = { path: string; old?: [string, Uint8Array]; new?: [string, Uint8Array] };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct ClosureEntry {
    /// The repository-relative path.
    pub path: Vec<u8>,
    /// Mode and bytes at the captured base, `None` when the base has no such path.
    pub old: Option<(String, Vec<u8>)>,
    /// Mode and bytes of the captured result, `None` for a deletion.
    pub new: Option<(String, Vec<u8>)>,
}

/// What:
///  The replay closure of `request`, after `verify_against_request` passed: both sides of every change, with no read of any
///  Git object.
/// Why:
///  This is what makes the spool self-contained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readClosure(root: string, request: Request): ClosureEntry[];
/// ```
pub fn read_closure(root: &Path, request: &Request) -> Result<Vec<ClosureEntry>, SpoolError> {
    verify_against_request(root, request)?;
    let directory: PathBuf = spool_directory(root, &request.id)?;
    let mut entries: Vec<ClosureEntry> = Vec::new();
    for (number, change) in request.changes.iter().enumerate() {
        // The bytes returned are the bytes hashed here against what the request promises, not a positional vector read earlier.
        let new: Option<(String, Vec<u8>)> = if change.mode == "000000" {
            None
        } else {
            let bytes: Vec<u8> = read_regular(directory.join(number.to_string()))?;
            if sha256_hex(&bytes) != change.sha256 {
                return Err(SpoolError::Mismatch(format!("new bytes of change {}", number)));
            }
            // What:     `Some(x)` is the present case of `Option` (TS: the value itself, as opposed to `undefined`); `None` is the absent case.
            // Why:      Return or store a value that may be missing without using a sentinel.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return x; // or undefined
            // ```
            Some((change.mode.clone(), bytes))
        };
        let old: Option<(String, Vec<u8>)> = if change.old_mode == "000000" {
            None
        } else {
            let bytes: Vec<u8> = read_regular(directory.join(format!("{}.old", number)))?;
            if sha256_hex(&bytes) != change.old_sha256 {
                return Err(SpoolError::Mismatch(format!("old bytes of change {}", number)));
            }
            Some((change.old_mode.clone(), bytes))
        };
        entries.push(ClosureEntry { path: change.path.clone(), old, new });
    }
    return Ok(entries);
}

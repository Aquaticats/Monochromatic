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

use super::commit_identity::{create_dir_all_durable, sync_directory, temporary_name};
use super::commit_integrity::{is_identity, sha256_hex};
use super::commit_request::Request;

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

impl From<io::Error> for SpoolError {
    /// Lets `?` turn an `io::Error` into `SpoolError::Io`.
    fn from(error: io::Error) -> SpoolError {
        return SpoolError::Io(error);
    }
}

/// The directory of an identity's spool: `root/spool/<id>`.
pub fn spool_directory(root: &Path, id: &str) -> Result<PathBuf, SpoolError> {
    if !is_identity(id) {
        return Err(SpoolError::InvalidIdentity);
    }
    return Ok(root.join("spool").join(id));
}

/// The manifest body for `files`: one `file <n> <length> <sha256>` line per file.
fn manifest_body(files: &[Vec<u8>]) -> String {
    let mut body: String = String::new();
    for (number, bytes) in files.iter().enumerate() {
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
    let temporary: PathBuf = temporary_name(directory)?;
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    let linked: io::Result<()> = fs::hard_link(&temporary, target);
    fs::remove_file(&temporary)?;
    match linked {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            // The name may have been linked by a writer that died before syncing the directory.
            sync_directory(directory)?;
            if fs::read(target)? == bytes {
                return Ok(());
            }
            return Err(SpoolError::Conflict);
        }
        Err(error) => return Err(SpoolError::Io(error)),
    }
}

/// What:
///  Writes the spool of `id` holding `files` (entry `n` is file `n`), then the manifest, syncing the directory between.
///  Safe to call again with the same files after a crash; other files under the same identity give `Conflict`.
/// Why:
///  The manifest is linked last, so a spool with a manifest is complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeSpool(root: string, id: string, files: Uint8Array[]): void;
/// ```
pub fn write_spool(root: &Path, id: &str, files: &[Vec<u8>]) -> Result<(), SpoolError> {
    let directory: PathBuf = spool_directory(root, id)?;
    create_dir_all_durable(&directory)?;
    for (number, bytes) in files.iter().enumerate() {
        link_complete(&directory.join(number.to_string()), bytes)?;
    }
    sync_directory(&directory)?;
    let body: String = manifest_body(files);
    let manifest: String = format!("{}end {}\n", body, sha256_hex(body.as_bytes()));
    link_complete(&directory.join("manifest"), manifest.as_bytes())?;
    sync_directory(&directory)?;
    if let Some(parent) = directory.parent() {
        sync_directory(parent)?;
    }
    return Ok(());
}

/// What:
///  Checks the spool of `id` against its manifest and returns the SHA-256 of each file, in order.
/// Why:
///  An integrity check that reads every byte: the manifest's own end hash, then each file's length and hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verifySpool(root: string, id: string): string[];
/// ```
pub fn verify_spool(root: &Path, id: &str) -> Result<Vec<String>, SpoolError> {
    let directory: PathBuf = spool_directory(root, id)?;
    let manifest: String = match fs::read_to_string(directory.join("manifest")) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(SpoolError::Incomplete),
        Err(error) => return Err(SpoolError::Io(error)),
    };
    let end_at: usize = manifest.rfind("end ").ok_or(SpoolError::Mismatch("manifest end".to_string()))?;
    let (body, end) = manifest.split_at(end_at);
    if end != format!("end {}\n", sha256_hex(body.as_bytes())) {
        return Err(SpoolError::Mismatch("manifest hash".to_string()));
    }
    let mut hashes: Vec<String> = Vec::new();
    for (number, line) in body.lines().enumerate() {
        let parts: Vec<&str> = line.split(' ').collect();
        let well_formed: bool = parts.len() == 4 && parts[0] == "file" && parts[1] == number.to_string();
        if !well_formed {
            return Err(SpoolError::Mismatch(format!("manifest line {}", number)));
        }
        let bytes: Vec<u8> = fs::read(directory.join(number.to_string()))?;
        if bytes.len().to_string() != parts[2] || sha256_hex(&bytes) != parts[3] {
            return Err(SpoolError::Mismatch(format!("file {}", number)));
        }
        hashes.push(parts[3].to_string());
    }
    return Ok(hashes);
}

/// What:
///  Checks the spool of `request.id` against the request: one file per change, each non-deletion file hashing to the
///  hash the change records, each deletion file empty.
/// Why:
///  The record promises what the spool must hold; this is the check that the promise is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verifyAgainstRequest(root: string, request: Request): void;
/// ```
pub fn verify_against_request(root: &Path, request: &Request) -> Result<(), SpoolError> {
    let hashes: Vec<String> = verify_spool(root, &request.id)?;
    if hashes.len() != request.changes.len() {
        return Err(SpoolError::Mismatch("file count".to_string()));
    }
    for (number, change) in request.changes.iter().enumerate() {
        let expected: String = if change.mode == "000000" { sha256_hex(&[]) } else { change.sha256.clone() };
        if hashes[number] != expected {
            return Err(SpoolError::Mismatch(format!("change {}", number)));
        }
    }
    return Ok(());
}

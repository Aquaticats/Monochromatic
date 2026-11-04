//! What:
//!  A request as one checksummed file in `inbox/`: its record, every captured side and one checksum, written under a temporary name
//!  in `inbox/tmp/` and published by a no-replace link, with no sync; the departure barrier that forces only what a leaving command
//!  promises; and the identity that makes one identity name one content.
//! Why:
//!  Rule V (revision 12): the waiting path of a plain commit waits for no sync of the wrapper's. The revision 11 path (a binding file,
//!  a spool directory with a manifest, the record, each with its file and directory syncs) made 22 sync calls to acceptance for a
//!  one-path commit. Directories are created once, when the lane is initialized, and no per-commit step creates or syncs one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writeFile(tmp, bytes); link(tmp, `inbox/${id}`); unlink(tmp); // no fsync until a departure
//! ```

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::commit_identity::{create_dir_all_durable, fault_point, record_event, sync_directory, sync_file, temporary_name};
use super::commit_integrity::sha256_hex;
use super::commit_request::{Request, RequestError, decode, encode};
use super::commit_state_log::sync_newest_log;

/// What:
///  `enum InboxError { ... }`: why a request cannot be encoded, decoded or accepted.
/// Why:
///  A damaged file is reported as damaged (and treated as never written), not repaired in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class InboxError extends Error { kind: 'request' | 'invalid' | 'damaged' | 'conflict' | 'io'; }
/// ```
#[derive(Debug)]
pub enum InboxError {
    /// The request record is invalid or damaged.
    Request(RequestError),
    /// The identity or a side does not match what the record promises (the reason is given).
    Invalid(&'static str),
    /// The file is torn or damaged (its checksum or its structure fails).
    Damaged,
    /// The identity names a file with other bytes.
    Conflict,
    /// A file system call failed.
    Io(io::Error),
}

impl From<io::Error> for InboxError {
    /// Lets `?` turn an `io::Error` into `InboxError::Io`.
    fn from(error: io::Error) -> InboxError {
        return InboxError::Io(error);
    }
}

impl From<RequestError> for InboxError {
    /// Lets `?` turn a record error into `InboxError::Request`.
    fn from(error: RequestError) -> InboxError {
        return InboxError::Request(error);
    }
}

/// What this call did with `inbox/<id>`.
#[derive(Debug, Eq, PartialEq)]
pub enum Accepted {
    /// This call linked the file into place.
    Created,
    /// A file with the same bytes was already there (a resend).
    AlreadyPresent,
    /// This call linked the file into place and then could not remove its temporary name (review 5 item 3.6): the request is accepted
    /// and published; only a stale name remains in `inbox/tmp/`, which recovery ignores. The kind says why the removal failed.
    CreatedTemporaryLeft(io::ErrorKind),
}

/// What:
///  The identity of `request` under `instance`: the first 32 hexadecimal characters of the SHA-256 of the instance and the request's
///  canonical encoding with a zero identity.
/// Why:
///  An identity that is the hash of the content can name only one content, so no binding file is needed to reject one identity with
///  two contents; the instance in the hash keeps lanes apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function identityOf(instance: string, request: Request): string;
/// ```
pub fn identity_of(instance: &str, request: &Request) -> Result<String, RequestError> {
    let mut zeroed: Request = request.clone();
    zeroed.id = "0".repeat(32);
    let mut material: Vec<u8> = instance.as_bytes().to_vec();
    material.push(0);
    material.extend_from_slice(&encode(&zeroed)?);
    return Ok(sha256_hex(&material)[..32].to_string());
}

/// `request` with its identity set to `identity_of(instance, request)`.
pub fn with_identity(instance: &str, mut request: Request) -> Result<Request, RequestError> {
    request.id = identity_of(instance, &request)?;
    return Ok(request);
}

/// Appends one section: `<kind> <length>\n<bytes>\n`.
fn push_section(out: &mut Vec<u8>, kind: &str, bytes: &[u8]) {
    out.extend_from_slice(format!("{} {}\n", kind, bytes.len()).as_bytes());
    out.extend_from_slice(bytes);
    out.push(b'\n');
}

/// What:
///  The bytes of the inbox file: `cli-git-inbox 1`, the record, each new side, each old side, then `end <sha256 of all before>`.
///  Each side must hash to what the record promises (an absent side is the empty file).
/// Why:
///  One file, one checksum: whatever a power loss leaves of it either verifies whole or is treated as never written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encodeInboxFile(request: Request, news: Uint8Array[], olds: Uint8Array[]): Uint8Array;
/// ```
pub fn encode_inbox_file(request: &Request, new_files: &[Vec<u8>], old_files: &[Vec<u8>]) -> Result<Vec<u8>, InboxError> {
    let count: usize = request.changes.len();
    if new_files.len() != count || old_files.len() != count {
        return Err(InboxError::Invalid("side counts"));
    }
    for (number, change) in request.changes.iter().enumerate() {
        let new_ok: bool = if change.mode == "000000" { new_files[number].is_empty() } else { sha256_hex(&new_files[number]) == change.sha256 };
        let old_ok: bool = if change.old_mode == "000000" { old_files[number].is_empty() } else { sha256_hex(&old_files[number]) == change.old_sha256 };
        if !new_ok || !old_ok {
            return Err(InboxError::Invalid("side does not match the record"));
        }
    }
    let mut out: Vec<u8> = b"cli-git-inbox 1\n".to_vec();
    push_section(&mut out, "record", &encode(request)?);
    for bytes in new_files {
        push_section(&mut out, "new", bytes);
    }
    for bytes in old_files {
        push_section(&mut out, "old", bytes);
    }
    let checksum: String = sha256_hex(&out);
    out.extend_from_slice(format!("end {}\n", checksum).as_bytes());
    return Ok(out);
}

/// Reads one section `<kind> <length>\n<bytes>\n` at `at`; returns its bytes and the position after it.
fn read_section<'a>(bytes: &'a [u8], at: usize, kind: &str) -> Option<(&'a [u8], usize)> {
    let line_end: usize = at + bytes.get(at..)?.iter().position(|byte| { return *byte == b'\n'; })?;
    let line: &str = std::str::from_utf8(&bytes[at..line_end]).ok()?;
    let length: usize = line.strip_prefix(kind)?.strip_prefix(' ')?.parse().ok()?;
    let start: usize = line_end + 1;
    let end: usize = start.checked_add(length)?;
    if bytes.get(end) != Some(&b'\n') {
        return None;
    }
    return Some((&bytes[start..end], end + 1));
}

/// What:
///  The request and both sides an inbox file holds, or `Damaged` when the checksum, the structure or any promise of the record
///  fails; the decoded file re-encodes to exactly the input.
/// Why:
///  After a power loss a recovery reads files that may be partial: a file that does not verify whole was never accepted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeInboxFile(bytes: Uint8Array): [Request, Uint8Array[], Uint8Array[]];
/// ```
pub fn decode_inbox_file(bytes: &[u8]) -> Result<(Request, Vec<Vec<u8>>, Vec<Vec<u8>>), InboxError> {
    let header: &[u8] = b"cli-git-inbox 1\n";
    if !bytes.starts_with(header) {
        return Err(InboxError::Damaged);
    }
    let (record, mut at) = read_section(bytes, header.len(), "record").ok_or(InboxError::Damaged)?;
    let request: Request = decode(record).map_err(|_| { return InboxError::Damaged; })?;
    let mut new_files: Vec<Vec<u8>> = Vec::new();
    let mut old_files: Vec<Vec<u8>> = Vec::new();
    for _ in 0..request.changes.len() {
        let (side, next) = read_section(bytes, at, "new").ok_or(InboxError::Damaged)?;
        new_files.push(side.to_vec());
        at = next;
    }
    for _ in 0..request.changes.len() {
        let (side, next) = read_section(bytes, at, "old").ok_or(InboxError::Damaged)?;
        old_files.push(side.to_vec());
        at = next;
    }
    let again: Vec<u8> = encode_inbox_file(&request, &new_files, &old_files).map_err(|_| { return InboxError::Damaged; })?;
    if again != bytes {
        return Err(InboxError::Damaged);
    }
    return Ok((request, new_files, old_files));
}

/// What:
///  Creates the lane's directories once (`inbox/`, `inbox/tmp/`, `state/`, `queue/`) below `lane`, making them durable through the
///  known-durable `anchor`, and returns the absolute lane directory.
/// Why:
///  Rule V: no per-commit step creates a directory or syncs an ancestor; all of that is done here, once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initLaneDirectories(anchor: string, lane: string): string;
/// ```
pub fn init_lane_directories(anchor: &Path, lane: &Path) -> io::Result<PathBuf> {
    for sub in ["inbox/tmp", "state", "queue"] {
        create_dir_all_durable(anchor, &lane.join(sub))?;
    }
    return std::path::absolute(lane);
}

/// What:
///  Accepts a request: verifies its identity and sides, writes the file under `inbox/tmp/`, links it to `inbox/<id>` without
///  replacing, removes the temporary name. Makes no sync. A file already in place with the same bytes is a resend (answered at once,
///  nothing written); other bytes under the identity are a conflict.
/// Why:
///  Visible is enough for the death of any process; durability is a departure barrier's or the background syncer's business.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function accept(inbox: string, instance: string, request: Request, news: Uint8Array[], olds: Uint8Array[]): 'created' | 'already-present';
/// ```
pub fn accept(inbox: &Path, instance: &str, request: &Request, new_files: &[Vec<u8>], old_files: &[Vec<u8>]) -> Result<Accepted, InboxError> {
    if request.id != identity_of(instance, request)? {
        return Err(InboxError::Invalid("identity is not the hash of the content and the instance"));
    }
    let bytes: Vec<u8> = encode_inbox_file(request, new_files, old_files)?;
    let target: PathBuf = inbox.join(&request.id);
    // Only absence lets acceptance go on: another read failure leaves the request's status unknown, and the caller must not treat it as
    // "not accepted" (an accepted request may already be in place).
    match fs::read(&target) {
        Ok(existing) => {
            if existing == bytes {
                return Ok(Accepted::AlreadyPresent);
            }
            return Err(InboxError::Conflict);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(InboxError::Io(error)),
    }
    let temporary: PathBuf = temporary_name(&inbox.join("tmp"))?;
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(&bytes)?;
    fault_point("accept:before-link");
    let linked: io::Result<()> = fs::hard_link(&temporary, &target);
    fault_point("accept:after-link");
    // A failed cleanup after a successful link must not look like a request that was never accepted: the caller would run native Git
    // beside a queued request. The outcome of the link decides the answer; the cleanup failure rides along or is recorded.
    let cleanup: io::Result<()> = fs::remove_file(&temporary);
    match linked {
        Ok(()) => {
            return Ok(match cleanup {
                Ok(()) => Accepted::Created,
                Err(error) => Accepted::CreatedTemporaryLeft(error.kind()),
            });
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            if let Err(cleanup_error) = cleanup {
                record_event(&format!("accept:cleanup-failed:{:?}", cleanup_error.kind()));
            }
            if fs::read(&target)? == bytes {
                return Ok(Accepted::AlreadyPresent);
            }
            return Err(InboxError::Conflict);
        }
        Err(error) => return Err(InboxError::Io(error)),
    }
}

/// The directory that holds `path`; a bare relative name has the empty parent, which is the current directory.
fn parent_directory(path: &Path) -> &Path {
    return match path.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => directory,
        _ => Path::new("."),
    };
}

/// What:
///  The departure barrier: forces to disk what a leaving command promises, in the order of the dependency chain: each promised request
///  file, then the `inbox/` directory (its entries), then the newest state file.
/// Why:
///  Before a process stops being responsible for a request that is not published (W1, an interrupt, a handover), the request must
///  survive a power loss; a command that waits until its request is published has no departure and waits for none of this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function depart(inbox: string, ids: string[], stateDir: string): void;
/// ```
pub fn depart(inbox: &Path, ids: &[String], state_dir: &Path) -> io::Result<()> {
    return depart_closure(inbox, ids, None, state_dir, None);
}

/// What:
///  The departure barrier with its whole closure, in the order of the dependency chain: each promised request file, the `inbox/`
///  directory, the order file (when the exit promises an adoption) and its directory, the newest state log file and the state
///  directory (the state's name), then the ledger file (when it promises a terminal outcome) and its directory.
/// Why:
///  A saving exit promises everything it depends on: a synced file under a directory entry that a power loss can remove promises nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function departClosure(inbox: string, ids: string[], order: string | undefined, stateDir: string, ledger: string | undefined): void;
/// ```
pub fn depart_closure(inbox: &Path, ids: &[String], order: Option<&Path>, state_dir: &Path, ledger: Option<&Path>) -> io::Result<()> {
    for id in ids {
        let path: PathBuf = inbox.join(id);
        let file: File = File::open(&path)?;
        sync_file(&file, &path)?;
    }
    sync_directory(inbox)?;
    if let Some(path) = order {
        sync_file(&File::open(path)?, path)?;
        sync_directory(parent_directory(path))?;
    }
    sync_newest_log(state_dir)?;
    if let Some(path) = ledger {
        sync_file(&File::open(path)?, path)?;
        sync_directory(parent_directory(path))?;
    }
    return Ok(());
}

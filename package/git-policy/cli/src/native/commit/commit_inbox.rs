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

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::commit_identity::{create_dir_all_durable, fault_point, record_event_lazy, sync_chain, sync_directory, sync_file, temporary_name};
use super::commit_integrity::{is_identity, sha256_hex};
use super::commit_request::{Request, RequestError, decode, encode};
use super::commit_state_log::{ClosureEntry, EntryKind, Prefix, force_item, select, state_covers, sync_prefix};

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
    /// The identity names a file with other bytes that are a valid record: a different request under one identity.
    Conflict,
    /// The identity names a file that does not decode (a torn or damaged final file): its bytes are the only evidence, so nothing was
    /// overwritten; `quarantine_damaged` moves it aside and the resend may then be accepted.
    DamagedFinal,
    /// The file carries a request format this build does not read; its bytes are left alone.
    Unsupported,
    /// The request may already be in place and its status could not be read (the link found the name and the read failed): the caller
    /// must not run native Git beside it until the status is known.
    StatusUnknown(io::Error),
    /// A file system call failed before any request could have been exposed.
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
            return Err(if decode_inbox_file(&existing).is_err() { InboxError::DamagedFinal } else { InboxError::Conflict });
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
                record_event_lazy(|| { return format!("accept:cleanup-failed:{:?}", cleanup_error.kind()); });
            }
            let existing: Vec<u8> = fs::read(&target).map_err(InboxError::StatusUnknown)?;
            if existing == bytes {
                return Ok(Accepted::AlreadyPresent);
            }
            return Err(if decode_inbox_file(&existing).is_err() { InboxError::DamagedFinal } else { InboxError::Conflict });
        }
        Err(error) => return Err(InboxError::Io(error)),
    }
}

/// What:
///  The only authority that may repair a damaged final: a non-blocking exclusive lock on `<inbox>/repair.lock`, held for the value's life.
///  `quarantine` moves a final that does not decode to `inbox/damaged/<unique>` after checking that the object it moves is the object it
///  inspected, keeps its bytes, and forces the archived entry, its directory and the inbox directory before it returns. A file with an
///  unsupported version, a name that is not an identity, or a file that decodes is never moved.
/// Why:
///  Repair is a destructive mutation of the active namespace. Two repairers, or a repairer and a resend that takes the freed name, must not
///  be able to move an accepted request, and the evidence a repair keeps is the only copy of the torn bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class InboxRepairer { static acquire(inbox: string): InboxRepairer; quarantine(id: string): string; }
/// ```
#[derive(Debug)]
pub struct InboxRepairer {
    /// The locked file; dropping the repairer releases the lock.
    _lock: File,
    /// The absolute inbox directory.
    inbox: PathBuf,
}

impl InboxRepairer {
    /// Takes the repair lock without waiting; `WouldBlock` (as `InboxError::Io`) when another repairer holds it. A foreign special file
    /// at the lock path is refused without being opened.
    pub fn acquire(inbox: &Path) -> Result<InboxRepairer, InboxError> {
        use std::os::unix::fs::OpenOptionsExt;
        let absolute: PathBuf = std::path::absolute(inbox)?;
        let path: PathBuf = absolute.join("repair.lock");
        if let Ok(metadata) = fs::symlink_metadata(&path)
            && !metadata.file_type().is_file()
        {
            return Err(InboxError::Invalid("the repair lock path is not a regular file"));
        }
        let lock: File = OpenOptions::new().create(true).write(true).truncate(false).custom_flags(libc::O_NONBLOCK).open(&path)?;
        match lock.try_lock() {
            Ok(()) => return Ok(InboxRepairer { _lock: lock, inbox: absolute }),
            Err(std::fs::TryLockError::WouldBlock) => return Err(InboxError::Io(io::Error::new(io::ErrorKind::WouldBlock, "another repairer holds the inbox"))),
            Err(std::fs::TryLockError::Error(error)) => return Err(InboxError::Io(error)),
        }
    }

    /// Moves the damaged final of `id` aside, keeping its bytes; returns the archived path.
    pub fn quarantine(&mut self, id: &str) -> Result<PathBuf, InboxError> {
        use std::io::Read;
        use std::os::unix::fs::MetadataExt;
        if !is_identity(id) {
            return Err(InboxError::Invalid("the name is not a request identity"));
        }
        let target: PathBuf = self.inbox.join(id);
        // The inspected object is held open, so its inode cannot be freed and reused while the repair decides.
        let mut inspected: File = File::open(&target)?;
        let before = inspected.metadata()?;
        let mut existing: Vec<u8> = Vec::new();
        inspected.read_to_end(&mut existing)?;
        if existing.starts_with(b"cli-git-inbox ") && !existing.starts_with(b"cli-git-inbox 1\n") {
            return Err(InboxError::Unsupported);
        }
        if decode_inbox_file(&existing).is_ok() {
            return Err(InboxError::Invalid("the file decodes; it is a request and is not moved"));
        }
        fault_point("inbox:after-inspect");
        let now = fs::symlink_metadata(&target)?;
        if now.dev() != before.dev() || now.ino() != before.ino() {
            return Err(InboxError::Invalid("the object at the name is not the one that was inspected; nothing was moved"));
        }
        let directory: PathBuf = self.inbox.join("damaged");
        match fs::create_dir(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(InboxError::Io(error)),
        }
        let moved: PathBuf = temporary_name(&directory)?;
        fs::rename(&target, &moved)?;
        // The archive obligation: the evidence, its directory entry and the inbox directory are forced here, on the repair path.
        sync_file(&File::open(&moved)?, &moved)?;
        sync_directory(&directory)?;
        sync_directory(&self.inbox)?;
        return Ok(moved);
    }
}

/// The directory that holds `path`; a bare relative name has the empty parent, which is the current directory.
fn parent_directory(path: &Path) -> &Path {
    return match path.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => directory,
        _ => Path::new("."),
    };
}

/// What a leaving command promised, which decides what its departure must force.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    /// W1 withdrew the request: the stop receipt (the mark and its directory) is the promise that no attempt starts later, with the
    /// captured request as the saved copy.
    Withdrawn,
    /// The unresolved COMMITTING exit: the stop receipt naming the attempt in flight, and the captured request.
    Unresolved,
    /// A terminal outcome (conflicted, rewritten-away, found by W1): the ledger line and its directory entry are the stop.
    Terminal,
    /// A request left with the service: the captured request, the order file that adopts it, and the state.
    Handover,
    /// Captured intent only: the request files and the state, with no stop and no terminal line.
    Saved,
}

/// Why a departure was not built or did not cover its promise.
#[derive(Debug)]
pub enum DepartureError {
    /// The disposition's promise has no file to carry it (the reason names the missing part).
    MissingPromise(&'static str),
    /// The state prefix that was forced does not reach the record that names the saved item.
    StateNotCovering {
        /// The generation the promise needs.
        needed: u64,
        /// The generation the forced prefix reaches.
        forced: u64,
    },
    /// A file given for the disposition is not the one that carries its promise (the reason names it).
    NotThePromise(&'static str),
    /// At least one forcing returned an error. Every other save was still attempted, and nothing is acknowledged.
    Incomplete {
        /// Whether a stop receipt was among the failures (the one failure that leaves a request publishable after the error).
        stop_failed: bool,
        /// What failed, in the order attempted: the file and the error.
        failures: Vec<(String, io::Error)>,
    },
    /// A file system call failed before any forcing began.
    Io(io::Error),
}

impl From<io::Error> for DepartureError {
    /// Lets `?` turn an `io::Error` into `DepartureError::Io`.
    fn from(error: io::Error) -> DepartureError {
        return DepartureError::Io(error);
    }
}

/// What a completed departure acknowledges: which disposition it was bound to, whether its stop receipts were forced first, which files
/// it forced, and the state prefix it forced, bound to the content of its last record (the promise covers that record and its retained set
/// and no more).
#[derive(Debug)]
pub struct Acknowledgment {
    /// The disposition the departure was bound to.
    pub disposition: Disposition,
    /// Every stop receipt was forced first and alone.
    pub stop_forced: bool,
    /// Every file forced, in order.
    pub forced: Vec<PathBuf>,
    /// The state prefix forced.
    pub state: Prefix,
}

impl Acknowledgment {
    /// Whether the state recovery would select now still carries what this acknowledgment promised: the same record, or a successor that
    /// carries every retained item of it (completion of a checkpoint, a later append). A substitute that drops support is not coverage.
    pub fn still_covers(&self, state_dir: &Path) -> bool {
        return state_covers(state_dir, &self.state).unwrap_or(false);
    }
}

/// What:
///  One exit's departure, bound to its `Disposition`: the stop receipts, one per departing request, forced first and alone, with the chain of
///  their directories; the request files and the `inbox/` directory; the saved content the exit names; the order file, the state prefix with
///  every item that prefix retains, and the ledger. `run` refuses a disposition whose promise has no file or whose files do not carry the
///  promise for each request, attempts every save even after an error, checks that the forced state prefix reaches the record that names the
///  saved item, and returns an `Acknowledgment` of exactly what it forced.
/// Why:
///  A flush list assembled by the caller can omit the one file that carries the prohibition on later attempts. Decision 14 makes the
///  barrier run to completion; this makes its contents depend on the promise and not on the caller's care.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function departure(disposition, inbox, ids, stateDir, { stops, order, ledger, closure, anchor, namesRecord }): Acknowledgment;
/// ```
pub struct Departure<'a> {
    disposition: Disposition,
    inbox: &'a Path,
    ids: &'a [String],
    state_dir: &'a Path,
    order: Option<&'a Path>,
    ledger: Option<&'a Path>,
    stops: Vec<&'a Path>,
    closure: &'a [&'a Path],
    anchor: Option<&'a Path>,
    names_generation: Option<u64>,
}

/// The identity a forced item is accounted under: its path, and the length and checksum of what was forced.
fn identity_of_entry(entry: &ClosureEntry) -> (PathBuf, u64, String) {
    return (entry.path().to_path_buf(), entry.length(), entry.sha256().to_string());
}

/// What a departure has forced and what failed, so that one failure does not end the other saves.
struct Saves {
    forced: Vec<PathBuf>,
    failures: Vec<(String, io::Error)>,
}

impl Saves {
    /// Records the outcome of forcing `path`; returns whether it succeeded.
    fn attempt(&mut self, label: &str, result: io::Result<()>, path: &Path) -> bool {
        match result {
            Ok(()) => {
                self.forced.push(path.to_path_buf());
                return true;
            }
            Err(error) => {
                self.failures.push((format!("{} {}", label, path.display()), error));
                return false;
            }
        }
    }
}

/// How many times a departure re-forces the live closure when the state grows while it is forced. Owner decision 14 makes the departure
/// run to completion; the bound only turns a writer that appends without end into a named refusal instead of a hang. Four passes cover a
/// writer that appends once per pass with margin; no workload measurement exists yet (a measured value belongs in the notes).
const LIVE_FORCE_PASSES: usize = 4;

impl<'a> Departure<'a> {
    /// A departure of `disposition` for the requests `ids` in `inbox`, with the lane's state in `state_dir`.
    pub fn new(disposition: Disposition, inbox: &'a Path, ids: &'a [String], state_dir: &'a Path) -> Departure<'a> {
        return Departure { disposition, inbox, ids, state_dir, order: None, ledger: None, stops: Vec::new(), closure: &[], anchor: None, names_generation: None };
    }

    /// A stop receipt (a mark file). A withdrawal needs one per departing request.
    pub fn with_stop(mut self, stop: &'a Path) -> Departure<'a> {
        self.stops.push(stop);
        return self;
    }

    /// The order file an adoption is promised in.
    pub fn with_order(mut self, order: &'a Path) -> Departure<'a> {
        self.order = Some(order);
        return self;
    }

    /// The ledger file a terminal outcome is promised in.
    pub fn with_ledger(mut self, ledger: &'a Path) -> Departure<'a> {
        self.ledger = Some(ledger);
        return self;
    }

    /// Saved content the exit names.
    pub fn with_closure(mut self, closure: &'a [&'a Path]) -> Departure<'a> {
        self.closure = closure;
        return self;
    }

    /// A known-durable ancestor through which each parent chain is synced.
    pub fn with_anchor(mut self, anchor: &'a Path) -> Departure<'a> {
        self.anchor = Some(anchor);
        return self;
    }

    /// The generation of the state record that names the saved item: the forced prefix must reach it.
    pub fn naming_generation(mut self, generation: u64) -> Departure<'a> {
        self.names_generation = Some(generation);
        return self;
    }

    /// The lane directory: the parent of the inbox.
    fn lane(&self) -> &Path {
        return parent_directory(self.inbox);
    }

    /// Refuses a disposition whose promise has no file.
    fn validate(&self) -> Result<(), DepartureError> {
        let missing: Option<&'static str> = match self.disposition {
            Disposition::Withdrawn | Disposition::Unresolved if self.stops.is_empty() => Some("a withdrawal needs its stop receipt"),
            Disposition::Terminal if self.ledger.is_none() => Some("a terminal outcome needs its ledger line"),
            Disposition::Withdrawn | Disposition::Unresolved | Disposition::Handover | Disposition::Saved if self.ids.is_empty() => Some("the exit names no request"),
            Disposition::Handover if self.order.is_none() => Some("a handover needs the order file that adopts it"),
            _ => None,
        };
        if let Some(reason) = missing {
            return Err(DepartureError::MissingPromise(reason));
        }
        return self.validate_promise_files();
    }

    /// Checks that each file given for the promise is the file that carries it for each departing request, not merely a file: a withdrawal
    /// has the mark `decision/<id>` of every departing request (and no other mark), a terminal outcome has its line for each request, and a
    /// handover's order file adopts each request.
    fn validate_promise_files(&self) -> Result<(), DepartureError> {
        if matches!(self.disposition, Disposition::Withdrawn | Disposition::Unresolved) || !self.stops.is_empty() {
            let decision: PathBuf = self.lane().join("decision");
            for stop in &self.stops {
                let belongs: bool = stop.parent() == Some(decision.as_path()) && stop.file_name().and_then(|name| { return name.to_str(); }).is_some_and(|name| { return self.ids.iter().any(|id| { return id == name; }); });
                if !belongs {
                    return Err(DepartureError::NotThePromise("a stop receipt is not the decision mark of a departing request in this lane"));
                }
            }
            for id in self.ids {
                if !self.stops.iter().any(|stop| { return *stop == decision.join(id); }) {
                    return Err(DepartureError::NotThePromise("a departing request has no stop receipt of its own"));
                }
            }
        }
        if self.disposition == Disposition::Terminal
            && let Some(ledger) = self.ledger
        {
            for id in self.ids {
                let line = super::commit_ledger::lookup(ledger, id).map_err(|_| { return DepartureError::NotThePromise("the ledger cannot be read as a ledger"); })?;
                if line.is_none() {
                    return Err(DepartureError::NotThePromise("the ledger has no terminal line for a departing request"));
                }
            }
        }
        if self.disposition == Disposition::Handover
            && let Some(order) = self.order
        {
            let text: Vec<u8> = fs::read(order)?;
            for id in self.ids {
                let wanted: String = format!("adopt {}", id);
                if !text.split(|byte| { return *byte == b'\n'; }).any(|line| { return line == wanted.as_bytes(); }) {
                    return Err(DepartureError::NotThePromise("the order file has no adoption line for a departing request"));
                }
            }
        }
        return Ok(());
    }

    /// Runs the barrier to completion and acknowledges what it forced. Every save is attempted even after an error: a returned error is an
    /// outside fault, it does not excuse the saves that could still succeed, and nothing is acknowledged when any failed.
    pub fn run(&self) -> Result<Acknowledgment, DepartureError> {
        self.validate()?;
        let mut saves: Saves = Saves { forced: Vec::new(), failures: Vec::new() };
        let mut stop_failed: bool = false;
        for stop in &self.stops {
            let result: io::Result<()> = self.force_with_name(stop);
            if !saves.attempt("stop receipt", result, stop) {
                stop_failed = true;
            }
        }
        for id in self.ids {
            let path: PathBuf = self.inbox.join(id);
            let result: io::Result<()> = File::open(&path).and_then(|file| { return sync_file(&file, &path); });
            saves.attempt("request", result, &path);
        }
        if let Err(error) = sync_directory(self.inbox) {
            saves.failures.push((format!("directory {}", self.inbox.display()), error));
        }
        for saved in self.closure {
            let result: io::Result<()> = self.force_with_name(saved);
            saves.attempt("saved content", result, saved);
        }
        if let Some(path) = self.order {
            let result: io::Result<()> = self.force_with_name(path);
            saves.attempt("order file", result, path);
        }
        let state: Option<Prefix> = self.force_state_with_items(&mut saves.forced, &mut saves.failures);
        let mut not_covering: Option<(u64, u64)> = None;
        if let (Some(prefix), Some(needed)) = (&state, self.names_generation)
            && prefix.generation < needed
        {
            not_covering = Some((needed, prefix.generation));
        }
        if let Some(path) = self.ledger {
            let result: io::Result<()> = self.force_with_name(path);
            saves.attempt("ledger", result, path);
        }
        let Saves { forced, failures } = saves;
        if !failures.is_empty() {
            return Err(DepartureError::Incomplete { stop_failed, failures });
        }
        if let Some((needed, reached)) = not_covering {
            return Err(DepartureError::StateNotCovering { needed, forced: reached });
        }
        let Some(prefix) = state else {
            return Err(DepartureError::NotThePromise("the state could not be forced"));
        };
        return Ok(Acknowledgment { disposition: self.disposition, stop_forced: !self.stops.is_empty(), forced, state: prefix });
    }

    /// Forces the state prefix together with every item that prefix retains (envelopes, saved content, ledger and order prefixes): an
    /// acknowledged state is a recoverable promise only when its whole closure is durable, not just the departing requests. Failures are
    /// recorded and the next item is still attempted.
    fn force_state_with_items(&self, forced: &mut Vec<PathBuf>, failures: &mut Vec<(String, io::Error)>) -> Option<Prefix> {
        let lane: &Path = self.lane();
        // What was forced is accounted by identity (the path, and for an append-only file the extent of the prefix and its checksum), not
        // by path: a ledger forced at one length is not forced at the longer length a later state record retains.
        let mut identities: BTreeSet<(PathBuf, u64, String)> = BTreeSet::new();
        // Each pass forces the items of the state recovery selects now; a record appended meanwhile can retain more, so the prefix the sync
        // captured is checked against what was forced and another pass runs when it names more. A fixed pass count bounds the wait.
        for _ in 0..LIVE_FORCE_PASSES {
            match select(self.state_dir) {
                Ok(Some(selected)) => self.force_each(lane, &selected.items, &mut identities, forced, failures),
                Ok(None) => {}
                Err(error) => {
                    failures.push((format!("state {}", self.state_dir.display()), io::Error::other(format!("{:?}", error))));
                    return None;
                }
            }
            fault_point("departure:after-items-forced");
            let prefix: Prefix = match sync_prefix(self.state_dir) {
                Ok(prefix) => prefix,
                Err(error) => {
                    failures.push((format!("state {}", self.state_dir.display()), error));
                    return None;
                }
            };
            if prefix.items.iter().all(|entry| { return identities.contains(&identity_of_entry(entry)); }) {
                return Some(prefix);
            }
            self.force_each(lane, &prefix.items, &mut identities, forced, failures);
        }
        failures.push((format!("state {}", self.state_dir.display()), io::Error::other("the state kept retaining new items while it was forced")));
        return None;
    }

    /// Forces each item whose identity is not yet forced, recording failures, then the inbox directory when an envelope was among them.
    fn force_each(&self, lane: &Path, items: &[ClosureEntry], identities: &mut BTreeSet<(PathBuf, u64, String)>, forced: &mut Vec<PathBuf>, failures: &mut Vec<(String, io::Error)>) {
        let mut any_envelope: bool = false;
        for entry in items {
            let path: PathBuf = lane.join(entry.path());
            if identities.contains(&identity_of_entry(entry)) {
                continue;
            }
            match force_item(lane, entry) {
                Ok(()) => {
                    any_envelope = any_envelope || entry.kind() == EntryKind::InboxEnvelope;
                    identities.insert(identity_of_entry(entry));
                    if !forced.contains(&path) {
                        forced.push(path);
                    }
                }
                Err(error) => failures.push((format!("retained item {}", path.display()), error)),
            }
        }
        if any_envelope && let Err(error) = sync_directory(self.inbox) {
            failures.push((format!("directory {}", self.inbox.display()), error));
        }
    }

    /// Forces a file, its directory, and (with an anchor) the chain above the directory.
    fn force_with_name(&self, path: &Path) -> io::Result<()> {
        sync_file(&File::open(path)?, path)?;
        let parent: &Path = parent_directory(path);
        sync_directory(parent)?;
        if let Some(anchor) = self.anchor {
            sync_chain(parent, anchor)?;
        }
        return Ok(());
    }
}

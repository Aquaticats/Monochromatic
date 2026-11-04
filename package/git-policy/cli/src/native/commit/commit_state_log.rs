//! What:
//!  The lane's state document as a hash-chained append-only log, `state.log.<k>`, with checkpoint rotation: a record holds the whole
//!  document and the hash of its predecessor; recovery takes the longest prefix that validates; a rotation starts a new file whose first
//!  record is a full document, and the old file is removed only after the new file and its directory entry are synced.
//! Why:
//!  Revision 13 (`doc/planning/cli-git-commits-never-fail.md`, rule V and rule S): two unsynced overwrites of an alternating pair can tear
//!  both generations. A log is never overwritten in place; unsynced write-back can persist a later record while an earlier one is
//!  missing, so a checksum is not enough and every record names its predecessor's hash. An append makes no sync (it is visible, which is
//!  all a process death needs); the syncer or a departure makes it durable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // appendFileSync(newestLog, record(g, prevHash, payload)); // no fsync; recovery = longest valid hash-chained prefix
//! ```

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::commit_identity::{fault_point, record_event_lazy, sync_chain, sync_directory, sync_file};
use super::commit_integrity::sha256_hex;

/// The chain hash before the first record of a file.
fn genesis() -> String {
    return "0".repeat(64);
}

/// The path of log file number `number` in `dir`.
fn log_path(dir: &Path, number: u64) -> PathBuf {
    return dir.join(format!("state.log.{}", number));
}

/// What the state log could not give: a damaged log is not the same as no log, and a read failure is neither.
#[derive(Debug)]
pub enum StateError {
    /// A log file exists with bytes and no valid first record, and no older file holds a valid state.
    Damaged,
    /// Two valid files hold the same generation with different documents: neither is the root.
    Conflict,
    /// A file or the directory could not be read or written.
    Io(io::Error),
}

impl From<io::Error> for StateError {
    fn from(error: io::Error) -> Self {
        return StateError::Io(error);
    }
}

/// The numbers of the log files in `dir`, ascending; a directory that does not exist has none, any other listing failure is an error.
/// The error as an `io::Error`, for the writer's methods.
fn to_io(error: StateError) -> io::Error {
    return match error {
        StateError::Io(inner) => inner,
        StateError::Damaged => io::Error::new(io::ErrorKind::InvalidData, "the state log is damaged; its bytes are kept"),
        StateError::Conflict => io::Error::new(io::ErrorKind::InvalidData, "two state files hold one generation with different documents"),
    };
}

fn numbers(dir: &Path) -> io::Result<Vec<u64>> {
    let mut found: Vec<u64> = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(found),
        Err(error) => return Err(error),
    };
    for listed in entries {
        let entry = listed?;
        if let Some(number) = entry.file_name().to_str().and_then(|name| { return name.strip_prefix("state.log."); }).and_then(|n| { return n.parse::<u64>().ok(); }) {
            found.push(number);
        }
    }
    found.sort();
    return Ok(found);
}

/// The highest log file number in `dir`, if any.
pub fn rotation_number(dir: &Path) -> io::Result<Option<u64>> {
    return Ok(numbers(dir)?.last().copied());
}

/// One record: `rec <generation> <previous hash> <length>\n<payload>\nhash <hash>\n`, the hash over the previous hash, the header line and
/// the payload.
fn encode_record(generation: u64, previous: &str, payload: &[u8]) -> (Vec<u8>, String) {
    let header: String = format!("rec {} {} {}\n", generation, previous, payload.len());
    let mut material: Vec<u8> = previous.as_bytes().to_vec();
    material.extend_from_slice(header.as_bytes());
    material.extend_from_slice(payload);
    let hash: String = sha256_hex(&material);
    let mut bytes: Vec<u8> = header.into_bytes();
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(format!("\nhash {}\n", hash).as_bytes());
    return (bytes, hash);
}

/// The first line of a checkpoint's first record: a payload that starts with it is a replacement-root authority.
const AUTHORITY_MAGIC: &str = "cli-git-authority 2\n";

/// The first line of a record that declares live requests: its document and its live set.
const BODY_MAGIC: &str = "cli-git-body 1\n";

/// What kind of content a closure entry is. The kind is a claim that validation checks against the content: whatever a caller declares, a
/// file whose bytes are an inbox envelope is validated as one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// Immutable retained content: its length and checksum must match, always. Completion makes it read-only, so a legitimate writer cannot
    /// silently change what a root depends on.
    Opaque,
    /// An inbox request envelope under `inbox/<id>`: its length and checksum must match and it must decode with the identity of its name.
    InboxEnvelope,
    /// An append-only file (a ledger, the order file): the identified prefix must still be there; later lines are allowed.
    AppendPrefix,
}

/// One entry of a checkpoint's live closure: a file of the lane, bound to the identity of its content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureEntry {
    /// The path relative to the lane directory (the parent of the state directory).
    pub path: PathBuf,
    /// How the content is validated.
    pub kind: EntryKind,
    /// The content's length in bytes (for an append-only entry, the length of the identified prefix).
    pub length: u64,
    /// The SHA-256 of the content, hexadecimal (for an append-only entry, of the identified prefix).
    pub sha256: String,
}

/// Whether bytes are (or claim to be) an inbox envelope.
fn looks_like_envelope(bytes: &[u8]) -> bool {
    return bytes.starts_with(b"cli-git-inbox ");
}

impl ClosureEntry {
    /// Captures the identity of the file at `path` (under `lane`) by reading it. The kind is derived from the content as well as
    /// declared: an envelope is captured as an envelope however it was declared, and a torn one is refused.
    pub fn capture(lane: &Path, path: &Path, declared: EntryKind) -> io::Result<ClosureEntry> {
        let relative: PathBuf = path.strip_prefix(lane).map(PathBuf::from).map_err(|_| { return io::Error::new(io::ErrorKind::InvalidInput, "a closure entry lies under the lane"); })?;
        if relative.components().any(|part| { return !matches!(part, std::path::Component::Normal(_)); }) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a closure entry path has only normal components"));
        }
        let bytes: Vec<u8> = fs::read(path)?;
        let kind: EntryKind = if looks_like_envelope(&bytes) || relative.starts_with("inbox") { EntryKind::InboxEnvelope } else { declared };
        let entry: ClosureEntry = ClosureEntry { path: relative, kind, length: bytes.len() as u64, sha256: sha256_hex(&bytes) };
        validate_entry(lane, &entry)?;
        return Ok(entry);
    }
}

/// Reads the entry's file and checks its identity and its kind.
fn validate_entry(lane: &Path, entry: &ClosureEntry) -> io::Result<()> {
    let bytes: Vec<u8> = fs::read(lane.join(&entry.path))?;
    let identity_ok: bool = match entry.kind {
        EntryKind::AppendPrefix => usize::try_from(entry.length).ok().and_then(|length| { return bytes.get(..length); }).is_some_and(|prefix| { return sha256_hex(prefix) == entry.sha256; }),
        EntryKind::Opaque | EntryKind::InboxEnvelope => bytes.len() as u64 == entry.length && sha256_hex(&bytes) == entry.sha256,
    };
    if !identity_ok {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "a closure entry's content is not the content that was captured"));
    }
    if entry.kind != EntryKind::AppendPrefix && (entry.kind == EntryKind::InboxEnvelope || looks_like_envelope(&bytes)) {
        let decoded = super::commit_inbox::decode_inbox_file(&bytes).map_err(|_| { return io::Error::new(io::ErrorKind::InvalidData, "a closure entry is not a valid request envelope"); })?;
        let named: Option<&str> = entry.path.file_name().and_then(|name| { return name.to_str(); });
        if entry.path.starts_with("inbox") && named != Some(decoded.0.id.as_str()) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "an envelope's name is not its identity"));
        }
    }
    return Ok(());
}

/// Whether `id` names a live request whose envelope is there and decodes.
fn live_request_present(lane: &Path, id: &str) -> bool {
    if !super::commit_integrity::is_identity(id) {
        return false;
    }
    return fs::read(lane.join("inbox").join(id)).ok().and_then(|bytes| { return super::commit_inbox::decode_inbox_file(&bytes).ok(); }).is_some_and(|decoded| { return decoded.0.id == id; });
}

/// A record's payload, decoded.
enum Payload {
    /// A raw document with no live requests.
    Plain(Vec<u8>),
    /// A document that declares its live requests.
    Body { live: Vec<String>, document: Vec<u8> },
    /// A replacement-root authority.
    Authority(Authority),
    /// Starts like a supported format and does not parse: ineligible, never read as a plain document.
    Damaged,
}

/// A decoded replacement-root authority: its document, its closure (the live envelopes and the saved content) and the segments it
/// authorizes for reclamation.
struct Authority {
    document: Vec<u8>,
    closure: Vec<ClosureEntry>,
    reclaims: Vec<u64>,
}

/// The live requests a closure names: the envelopes directly under `inbox/`, by identity.
fn live_of(closure: &[ClosureEntry]) -> Vec<String> {
    return closure
        .iter()
        .filter(|entry| { return entry.kind == EntryKind::InboxEnvelope && entry.path.parent() == Some(Path::new("inbox")); })
        .filter_map(|entry| { return entry.path.file_name().and_then(|name| { return name.to_str().map(String::from); }); })
        .collect();
}

fn kind_word(kind: EntryKind) -> &'static str {
    return match kind {
        EntryKind::Opaque => "opaque",
        EntryKind::InboxEnvelope => "envelope",
        EntryKind::AppendPrefix => "prefix",
    };
}

fn encode_authority(document: &[u8], closure: &[ClosureEntry], reclaims: &[u64]) -> Vec<u8> {
    let mut text: String = String::from(AUTHORITY_MAGIC);
    for entry in closure {
        text.push_str(&format!("entry {} {} {} {}\n", kind_word(entry.kind), entry.length, entry.sha256, super::commit_integrity::hex(entry.path.as_os_str().as_encoded_bytes())));
    }
    let numbers: Vec<String> = reclaims.iter().map(|number| { return number.to_string(); }).collect();
    text.push_str(&format!("reclaims {}\ndocument {}\n", numbers.join(","), document.len()));
    let mut bytes: Vec<u8> = text.into_bytes();
    bytes.extend_from_slice(document);
    return bytes;
}

fn encode_body(live: &[String], document: &[u8]) -> Vec<u8> {
    if live.is_empty() {
        return document.to_vec();
    }
    let mut text: String = String::from(BODY_MAGIC);
    for id in live {
        text.push_str(&format!("live {}\n", id));
    }
    text.push_str(&format!("document {}\n", document.len()));
    let mut bytes: Vec<u8> = text.into_bytes();
    bytes.extend_from_slice(document);
    return bytes;
}

fn parse_payload(payload: &[u8]) -> Payload {
    if let Some(body) = payload.strip_prefix(AUTHORITY_MAGIC.as_bytes()) {
        return parse_authority(body).map_or(Payload::Damaged, Payload::Authority);
    }
    if let Some(body) = payload.strip_prefix(BODY_MAGIC.as_bytes()) {
        return parse_body(body).map_or(Payload::Damaged, |(live, document)| { return Payload::Body { live, document }; });
    }
    if payload.starts_with(b"cli-git-authority ") || payload.starts_with(b"cli-git-body ") {
        return Payload::Damaged;
    }
    return Payload::Plain(payload.to_vec());
}

/// Splits `body` into lines up to and including the `document <len>` line, returning the lines and the document bytes.
fn header_lines(body: &[u8]) -> Option<(Vec<&str>, Vec<u8>)> {
    let mut at: usize = 0;
    let mut lines: Vec<&str> = Vec::new();
    loop {
        let end: usize = at + body.get(at..)?.iter().position(|byte| { return *byte == b'\n'; })?;
        let line: &str = std::str::from_utf8(&body[at..end]).ok()?;
        at = end + 1;
        if let Some(rest) = line.strip_prefix("document ") {
            let length: usize = rest.parse().ok()?;
            let document: &[u8] = body.get(at..at.checked_add(length)?)?;
            return Some((lines, document.to_vec()));
        }
        lines.push(line);
    }
}

fn parse_body(body: &[u8]) -> Option<(Vec<String>, Vec<u8>)> {
    let (lines, document) = header_lines(body)?;
    let live: Vec<String> = lines.iter().map(|line| { return line.strip_prefix("live ").map(String::from); }).collect::<Option<Vec<String>>>()?;
    return Some((live, document));
}

fn parse_authority(body: &[u8]) -> Option<Authority> {
    use std::os::unix::ffi::OsStringExt;
    let (lines, document) = header_lines(body)?;
    let mut closure: Vec<ClosureEntry> = Vec::new();
    let mut reclaims: Vec<u64> = Vec::new();
    for line in lines {
        if let Some(rest) = line.strip_prefix("entry ") {
            let parts: Vec<&str> = rest.split(' ').collect();
            if parts.len() != 4 {
                return None;
            }
            let kind: EntryKind = match parts[0] {
                "envelope" => EntryKind::InboxEnvelope,
                "opaque" => EntryKind::Opaque,
                "prefix" => EntryKind::AppendPrefix,
                _ => return None,
            };
            let raw: Vec<u8> = super::commit_integrity::unhex(parts[3])?;
            closure.push(ClosureEntry { path: PathBuf::from(std::ffi::OsString::from_vec(raw)), kind, length: parts[1].parse().ok()?, sha256: parts[2].to_string() });
        } else {
            let rest: &str = line.strip_prefix("reclaims ")?;
            for number in rest.split(',').filter(|piece| { return !piece.is_empty(); }) {
                reclaims.push(number.parse().ok()?);
            }
        }
    }
    return Some(Authority { document, closure, reclaims });
}

/// One record of a log file's valid chain.
struct Rec {
    generation: u64,
    payload: Vec<u8>,
    hash: String,
    end: usize,
}

/// Every record of the longest valid hash chain of `bytes`, in order.
fn records(bytes: &[u8]) -> Vec<Rec> {
    let mut previous: String = genesis();
    let mut at: usize = 0;
    let mut found: Vec<Rec> = Vec::new();
    while at < bytes.len() {
        let Some(offset) = bytes[at..].iter().position(|byte| { return *byte == b'\n'; }) else { break };
        let line_end: usize = at + offset;
        let Ok(header) = std::str::from_utf8(&bytes[at..line_end]) else { break };
        let parts: Vec<&str> = header.split(' ').collect();
        let ok_shape: bool = parts.len() == 4 && parts[0] == "rec" && parts[2] == previous;
        let parsed = if ok_shape { (parts[1].parse::<u64>().ok(), parts[3].parse::<usize>().ok()) } else { (None, None) };
        let (Some(generation), Some(length)) = parsed else { break };
        let start: usize = line_end + 1;
        // A damaged header can name a length near `usize::MAX`; the sums are checked so it ends the chain instead of overflowing.
        let Some(end) = start.checked_add(length) else { break };
        let Some(trailer_end) = end.checked_add(1 + 5 + 64 + 1) else { break };
        if trailer_end > bytes.len() {
            break;
        }
        let payload: &[u8] = &bytes[start..end];
        let (expected, hash) = encode_record(generation, &previous, payload);
        if bytes[at..trailer_end] != expected[..] {
            break;
        }
        found.push(Rec { generation, payload: payload.to_vec(), hash: hash.clone(), end: trailer_end });
        previous = hash;
        at = trailer_end;
    }
    return found;
}

/// The file a recovery uses and what it holds: its number, the generation and document of its last eligible record, the chain hash there,
/// the length of the eligible prefix and the live requests that record declares.
#[derive(Debug)]
pub struct Selected {
    /// The log file number.
    pub number: u64,
    /// The generation of the last eligible record.
    pub generation: u64,
    /// The document of the last eligible record.
    pub payload: Vec<u8>,
    /// The chain hash after the last eligible record.
    pub hash: String,
    /// The length of the eligible prefix in bytes.
    pub length: usize,
    /// Whether this is the highest-numbered file (otherwise a newer candidate exists that does not validate).
    pub highest: bool,
    /// The live requests the selected record declares.
    pub live: Vec<String>,
    /// Records follow the eligible prefix in the same file and were dropped because a dependency of theirs is missing.
    pub dropped_suffix: bool,
}

/// The eligible part of a file: its records up to the first one whose dependencies do not validate, with the decoded document of the last.
fn eligible(lane: &Path, bytes: &[u8]) -> Option<(Vec<Rec>, usize, Vec<u8>, Vec<String>)> {
    let all: Vec<Rec> = records(bytes);
    let first: &Rec = all.first()?;
    let mut retained_ok: bool = true;
    if let Payload::Authority(authority) = parse_payload(&first.payload) {
        retained_ok = authority.closure.iter().all(|entry| { return validate_entry(lane, entry).is_ok(); });
    } else if matches!(parse_payload(&first.payload), Payload::Damaged) {
        retained_ok = false;
    }
    if !retained_ok {
        return None;
    }
    let mut count: usize = 0;
    let mut last_document: Vec<u8> = Vec::new();
    let mut last_live: Vec<String> = Vec::new();
    for record in &all {
        let (record_live, record_document): (Vec<String>, Vec<u8>) = match parse_payload(&record.payload) {
            Payload::Plain(plain) => (Vec::new(), plain),
            Payload::Body { live, document } => (live, document),
            Payload::Authority(authority) => (live_of(&authority.closure), authority.document),
            Payload::Damaged => break,
        };
        if !record_live.iter().all(|id| { return live_request_present(lane, id); }) {
            break;
        }
        last_document = record_document;
        last_live = record_live;
        count += 1;
    }
    if count == 0 {
        return None;
    }
    return Some((all, count, last_document, last_live));
}

/// The file recovery selects: the file whose last eligible record has the newest generation (a tie goes to the higher file when the
/// documents agree). A record is eligible when its own live requests are present and, for a checkpoint segment, its retained closure
/// validates; the first ineligible record ends the file's prefix, so a later document cannot outrun the dependencies it declares. No log
/// file is `Ok(None)`; files with bytes and no eligible record anywhere are `Damaged`; a file that cannot be read is `Io`.
pub fn select(dir: &Path) -> Result<Option<Selected>, StateError> {
    let all_numbers: Vec<u64> = numbers(dir)?;
    let highest: Option<u64> = all_numbers.last().copied();
    let lane: PathBuf = lane_of(dir);
    let mut saw_bytes: bool = false;
    let mut best: Option<Selected> = None;
    for number in all_numbers.into_iter().rev() {
        let bytes: Vec<u8> = fs::read(log_path(dir, number))?;
        saw_bytes = saw_bytes || !bytes.is_empty();
        let Some((all, count, document, live)) = eligible(&lane, &bytes) else { continue };
        let last: &Rec = &all[count - 1];
        let candidate: Selected = Selected {
            number,
            generation: last.generation,
            payload: document,
            hash: last.hash.clone(),
            length: last.end,
            highest: Some(number) == highest,
            live,
            dropped_suffix: count < all.len(),
        };
        match &best {
            // The newest generation wins, and a tie goes to the higher file when the documents agree; a lower file holding a newer
            // generation than a higher one means the higher one is a rollback and is not the root.
            Some(chosen) if candidate.generation == chosen.generation && candidate.payload != chosen.payload => return Err(StateError::Conflict),
            Some(chosen) if candidate.generation <= chosen.generation => {}
            _ => best = Some(candidate),
        }
    }
    if best.is_none() && saw_bytes {
        return Err(StateError::Damaged);
    }
    return Ok(best);
}

/// The lane directory a state directory belongs to: the closure's paths are relative to it, and it is the durable anchor of their ancestry.
fn lane_of(dir: &Path) -> PathBuf {
    return dir.parent().map_or_else(|| { return PathBuf::from("."); }, PathBuf::from);
}

/// The newest valid state (see `select`): its generation and document.
pub fn read(dir: &Path) -> Result<Option<(u64, Vec<u8>)>, StateError> {
    return Ok(select(dir)?.map(|selected| { return (selected.generation, selected.payload); }));
}

/// Checks the non-drop obligation: every request live in the previous record is live in the new one or retired, and a retirement is
/// verified against the ledger, whose line for a request is its terminal outcome (nothing is promised saved after it).
fn check_non_drop(previous: &[String], live: &[String], retired: &[String], ledger: Option<&Path>) -> io::Result<()> {
    for id in previous {
        if live.contains(id) {
            continue;
        }
        let discharged: bool = retired.contains(id) && ledger.is_some_and(|path| { return super::commit_ledger::lookup(path, id).ok().flatten().is_some(); });
        if !discharged {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "the new document drops a request that is live in the previous one and is not discharged by the ledger"));
        }
    }
    return Ok(());
}

/// Appends a record that declares `live` requests to the newest log file, chained to the file's last eligible record, with no sync. The
/// live requests are validated now, none of the previous record's may be dropped unless the ledger discharges it, and a file with a
/// dropped suffix is not appended to (it needs repair first). Private: the only way in is the `StateWriter` that holds the state's lock.
fn append(dir: &Path, generation: u64, document: &[u8], live: &[String], retired: &[String], ledger: Option<&Path>) -> io::Result<()> {
    let selected: Option<Selected> = select(dir).map_err(to_io)?;
    let lane: PathBuf = lane_of(dir);
    let (number, previous, keep): (u64, String, usize) = match &selected {
        Some(chosen) => {
            if generation <= chosen.generation {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "the generation must increase"));
            }
            if !chosen.highest {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "a newer checkpoint candidate does not validate; complete or abandon it first"));
            }
            if chosen.dropped_suffix {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "records after the eligible prefix were dropped for a missing dependency; repair first"));
            }
            check_non_drop(&chosen.live, live, retired, ledger)?;
            (chosen.number, chosen.hash.clone(), chosen.length)
        }
        None => {
            check_non_drop(&[], live, retired, ledger)?;
            (1, genesis(), 0)
        }
    };
    if let Some(missing) = live.iter().find(|id| { return !live_request_present(&lane, id); }) {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("live request {} is not present as a valid envelope", missing)));
    }
    let path: PathBuf = log_path(dir, number);
    let (record, _) = encode_record(generation, &previous, &encode_body(live, document));
    let mut file: File = OpenOptions::new().create(true).write(true).truncate(false).open(&path)?;
    if keep as u64 != file.metadata()?.len() {
        file.set_len(keep as u64)?;
    }
    io::Seek::seek(&mut file, io::SeekFrom::Start(keep as u64))?;
    file.write_all(&record)?;
    return Ok(());
}

/// Syncs the file recovery selects and its directory; see `sync_prefix`, which also says what it acknowledges.
pub fn sync_newest_log(dir: &Path) -> io::Result<()> {
    sync_prefix(dir)?;
    return Ok(());
}

/// The prefix of a log that a sync made durable: the file, the byte length, the last generation it holds and the live requests that record
/// declares (the closure a departure must also have forced for the acknowledgment to be a recoverable promise).
#[derive(Debug, Eq, PartialEq)]
pub struct Prefix {
    /// The log file number.
    pub number: u64,
    /// The bytes of the file that were part of the eligible prefix when the sync began.
    pub length: u64,
    /// The generation of the last record in that prefix.
    pub generation: u64,
    /// The live requests of that record.
    pub live: Vec<String>,
}

/// Syncs the file that recovery would select (not the highest filename), then its directory, and returns the prefix captured before the
/// sync began: records appended while the sync ran are not claimed by it.
pub fn sync_prefix(dir: &Path) -> io::Result<Prefix> {
    let selected: Selected = select(dir).map_err(to_io)?.ok_or_else(|| { return io::Error::new(io::ErrorKind::NotFound, "no state log"); })?;
    // The prefix is fixed here; records appended while the sync runs are not claimed by it.
    fault_point("state:after-capture");
    let path: PathBuf = log_path(dir, selected.number);
    let file: File = File::open(&path)?;
    sync_file(&file, &path)?;
    sync_directory(dir)?;
    return Ok(Prefix { number: selected.number, length: selected.length as u64, generation: selected.generation, live: selected.live });
}

/// A checkpoint candidate the writer started: its file number, generation and chain hash, so completion acts on exactly this file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The new segment's number.
    pub number: u64,
    /// The generation of its first record.
    pub generation: u64,
    /// The chain hash after its first record.
    pub hash: String,
}

fn begin_checkpoint(dir: &Path, generation: u64, document: &[u8], closure: &[ClosureEntry], retired: &[String], ledger: Option<&Path>) -> io::Result<Candidate> {
    let selected: Option<Selected> = select(dir).map_err(to_io)?;
    let lane: PathBuf = lane_of(dir);
    let existing: Vec<u64> = numbers(dir)?;
    // The closure is validated now, by reading each entry, and may not contain a file the authority will delete (aliases included).
    let canonical_state: PathBuf = fs::canonicalize(dir)?;
    for entry in closure {
        validate_entry(&lane, entry)?;
        if fs::canonicalize(lane.join(&entry.path))?.starts_with(&canonical_state) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a closure entry lies in the state directory the checkpoint reclaims"));
        }
    }
    let live: Vec<String> = live_of(closure);
    if let Some(chosen) = &selected {
        if generation < chosen.generation {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a checkpoint must not hold an older generation than the log"));
        }
        if generation == chosen.generation && document != chosen.payload.as_slice() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a checkpoint at the log's generation must hold the log's document"));
        }
        if !chosen.highest {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "an earlier checkpoint candidate does not validate"));
        }
        // Non-drop: the document is the replay of the predecessor's valid prefix, so every request live there is live here or discharged.
        check_non_drop(&chosen.live, &live, retired, ledger)?;
    }
    let next: u64 = existing
        .last()
        .map_or(Some(1), |number| { return number.checked_add(1); })
        .ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the segment numbers are exhausted"); })?;
    let (record, hash) = encode_record(generation, &genesis(), &encode_authority(document, closure, &existing));
    let mut file: File = OpenOptions::new().create_new(true).write(true).open(log_path(dir, next))?;
    file.write_all(&record)?;
    return Ok(Candidate { number: next, generation, hash });
}

/// Completes exactly `candidate`: it must still be the recovery root with the identity it was started with (so recovery's validation of
/// its whole closure passes now); every closure entry is forced to disk with its complete name chain through the lane directory (the
/// durable anchor) and immutable retained content is made read-only; the candidate and its directory entry are made durable; and only
/// then are the segments the authority names removed. Anything else leaves every older file in place.
fn complete_checkpoint(dir: &Path, candidate: &Candidate) -> io::Result<()> {
    let all: Vec<u64> = numbers(dir)?;
    let selected: Selected = select(dir).map_err(to_io)?.ok_or_else(|| { return io::Error::new(io::ErrorKind::NotFound, "no state log"); })?;
    let same: bool = selected.number == candidate.number && selected.generation == candidate.generation && selected.hash == candidate.hash;
    if !same || all.last() != Some(&candidate.number) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "the candidate is not the recovery root with the identity it was started with; the older files are kept"));
    }
    let bytes: Vec<u8> = fs::read(log_path(dir, candidate.number))?;
    let first: Rec = records(&bytes).into_iter().next().ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the candidate has no valid first record"); })?;
    let Payload::Authority(authority) = parse_payload(&first.payload) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "the candidate is not a replacement-root authority"));
    };
    for number in all.iter().filter(|number| { return **number != candidate.number; }) {
        let older: Vec<u8> = fs::read(log_path(dir, *number))?;
        if records(&older).last().is_some_and(|last| { return last.generation > selected.generation; }) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "an older file holds a newer generation than the candidate; it is kept"));
        }
    }
    // The new root replaces old log history, not the external content its closure names: that content is durable, with its ancestry,
    // before the candidate is made durable and before any predecessor is removed.
    let lane: PathBuf = lane_of(dir);
    let canonical_state: PathBuf = fs::canonicalize(dir)?;
    for entry in &authority.closure {
        let path: PathBuf = lane.join(&entry.path);
        if fs::canonicalize(&path)?.starts_with(&canonical_state) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a closure entry lies in the state directory the checkpoint reclaims"));
        }
        let file: File = File::open(&path)?;
        sync_file(&file, &path)?;
        sync_chain(path.parent().filter(|parent| { return !parent.as_os_str().is_empty(); }).unwrap_or(Path::new(".")), &lane)?;
        if entry.kind == EntryKind::Opaque {
            // The lifetime contract of immutable retained content is enforced: a legitimate writer meets a refusal instead of silently
            // invalidating the only remaining root.
            use std::os::unix::fs::PermissionsExt;
            let mode: u32 = fs::metadata(&path)?.permissions().mode();
            fs::set_permissions(&path, fs::Permissions::from_mode(mode & !0o222))?;
        }
    }
    let path: PathBuf = log_path(dir, candidate.number);
    let file: File = File::open(&path)?;
    sync_file(&file, &path)?;
    sync_directory(dir)?;
    for number in all.into_iter().filter(|number| { return *number != candidate.number && authority.reclaims.contains(number); }) {
        let old: PathBuf = log_path(dir, number);
        fs::remove_file(&old)?;
        record_event_lazy(|| { return format!("remove:{}", old.display()); });
    }
    return Ok(());
}

/// The single writer of a state directory, proved by a non-blocking exclusive lock on `state.writer` that lives as long as the value.
/// Appends and checkpoints take `&mut self`, so one owner cannot interleave them, a second owner is refused at once, and no free function
/// of this module can mutate the log: this is the only way. The directory is stored absolute, so a later change of the working directory
/// cannot point the writer at a directory its lock does not cover.
#[derive(Debug)]
pub struct StateWriter {
    /// The locked file; dropping the writer releases the lock.
    _lock: File,
    /// The absolute state directory.
    dir: PathBuf,
    /// The ledger that verifies a retirement (a request leaving the live set), when the owner has one.
    ledger: Option<PathBuf>,
}

impl StateWriter {
    /// Takes the writer lock without waiting; `WouldBlock` when another writer holds it. The lock path must be a regular file: a
    /// foreign special file there is refused without being opened for a blocking write.
    pub fn acquire(dir: &Path) -> io::Result<StateWriter> {
        use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
        let absolute: PathBuf = std::path::absolute(dir)?;
        let path: PathBuf = absolute.join("state.writer");
        if let Ok(metadata) = fs::symlink_metadata(&path)
            && !metadata.file_type().is_file()
        {
            let kind = if metadata.file_type().is_fifo() { "a FIFO" } else { "not a regular file" };
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the state writer lock path is {}", kind)));
        }
        let lock: File = OpenOptions::new().create(true).write(true).truncate(false).custom_flags(libc::O_NONBLOCK).open(&path)?;
        match lock.try_lock() {
            Ok(()) => return Ok(StateWriter { _lock: lock, dir: absolute, ledger: None }),
            Err(std::fs::TryLockError::WouldBlock) => return Err(io::Error::new(io::ErrorKind::WouldBlock, "another writer holds the state")),
            Err(std::fs::TryLockError::Error(error)) => return Err(error),
        }
    }

    /// Names the ledger that verifies retirements.
    pub fn with_ledger(mut self, ledger: &Path) -> StateWriter {
        self.ledger = Some(ledger.to_path_buf());
        return self;
    }

    /// Appends a document that declares no live request (valid only while the previous record declares none).
    pub fn append(&mut self, generation: u64, document: &[u8]) -> io::Result<()> {
        return append(&self.dir, generation, document, &[], &[], self.ledger.as_deref());
    }

    /// Appends a document that declares its live requests; each must be present as a valid envelope, none of the previous record's may be
    /// dropped unless it is in `retired` and the ledger holds its terminal line.
    pub fn append_live(&mut self, generation: u64, document: &[u8], live: &[String], retired: &[String]) -> io::Result<()> {
        return append(&self.dir, generation, document, live, retired, self.ledger.as_deref());
    }

    /// Starts a checkpoint: writes `document` and its validated live `closure` as the first record of the next segment and returns the
    /// candidate to complete. The candidate is not a root until recovery's validation of the closure passes.
    pub fn begin_checkpoint(&mut self, generation: u64, document: &[u8], closure: &[ClosureEntry]) -> io::Result<Candidate> {
        return begin_checkpoint(&self.dir, generation, document, closure, &[], self.ledger.as_deref());
    }

    /// As `begin_checkpoint`, with the requests that the ledger shows retired since the previous record.
    pub fn begin_checkpoint_retiring(&mut self, generation: u64, document: &[u8], closure: &[ClosureEntry], retired: &[String]) -> io::Result<Candidate> {
        return begin_checkpoint(&self.dir, generation, document, closure, retired, self.ledger.as_deref());
    }

    /// Completes exactly `candidate` after forcing its closure with the ancestry; the segments it authorizes are removed last, and every
    /// older file stays on any refusal.
    pub fn complete_checkpoint(&mut self, candidate: &Candidate) -> io::Result<()> {
        return complete_checkpoint(&self.dir, candidate);
    }

    /// Seals the current segment and replaces it by a checkpoint in one owner step; future appends go to the new segment.
    pub fn checkpoint(&mut self, generation: u64, document: &[u8], closure: &[ClosureEntry]) -> io::Result<u64> {
        let candidate: Candidate = begin_checkpoint(&self.dir, generation, document, closure, &[], self.ledger.as_deref())?;
        complete_checkpoint(&self.dir, &candidate)?;
        return Ok(candidate.number);
    }
}

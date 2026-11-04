//! What:
//!  The lane's state document as a hash-chained append-only log, `state.log.<k>`, with checkpoint rotation: a record holds the whole
//!  document and the hash of its predecessor; recovery takes the longest prefix that validates; a rotation starts a new file whose first
//!  record is a full document, and the old file is removed only after the new file and its directory entry are synced.
//! Premise of the lane: saved content is immutable because `create_saved_content` is its only way in (written, forced and made
//!  read-only before it has its final name); the only way to write it afterwards is a deliberate chmod by the account that owns the lane,
//!  and an owner who chmods and rewrites its own files defeats every scheme, this one included. The checksum name still detects it.
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

pub use super::commit_lane_state::{Consumption, ConsumptionKind, LaneState, Marker, RequestPhase, RequestState, RESUME_MAX};
use std::collections::BTreeSet;
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
const AUTHORITY_MAGIC: &str = "cli-git-authority 3\n";

/// The first line of a record that retains items: its document and its complete retained set.
const BODY_MAGIC: &str = "cli-git-body 2\n";

/// The directory of the lane that holds request envelopes, one file per identity.
const INBOX: &str = "inbox";

/// The directory of the lane that holds decision marks, empty files named by the request identity and a suffix.
const DECISION: &str = "decision";

/// The suffixes of a decision mark: a withdrawal, and a withdrawal during which the attempt in flight was the one the owner had recorded.
const MARK_SUFFIXES: [&str; 2] = [".withdraw", ".withdraw-unresolved"];

/// The lane's adoption order, relative to the lane: an append-only file whose extent a checked document relies on.
pub const LANE_ORDER: &str = "order";

/// The request identity a decision mark name carries: the name must be an identity followed by one of the mark suffixes.
fn mark_identity(name: &str) -> Option<&str> {
    return MARK_SUFFIXES
        .iter()
        .find_map(|suffix| { return name.strip_suffix(suffix); })
        .filter(|identity| { return super::commit_integrity::is_identity(identity); });
}

/// The lane's authoritative ledger, relative to the lane: the only append-only file whose lines discharge a retained item. A line in any
/// other retained file, however well formed, is not evidence that a request or saved content was settled.
pub const LANE_LEDGER: &str = "ledger";

/// The line prefix with which a document names a file of the lane that it depends on: `cli-git-needs <hex of the relative path>`.
const NEEDS_PREFIX: &str = "cli-git-needs ";

/// The line a document carries to name `path` (relative to the lane) as a dependency; the line ends with a newline.
pub fn needs_line(path: &Path) -> String {
    return format!("{}{}\n", NEEDS_PREFIX, super::commit_integrity::hex(path.as_os_str().as_encoded_bytes()));
}

/// The lane-relative paths `document` names as dependencies through its `needs` lines; `None` when a line starts as one and does not decode,
/// so a malformed obligation is never read as no obligation.
fn document_needs(document: &[u8]) -> Option<Vec<PathBuf>> {
    use std::os::unix::ffi::OsStringExt;
    let mut needs: Vec<PathBuf> = Vec::new();
    for line in document.split(|byte| { return *byte == b'\n'; }) {
        let Some(rest) = line.strip_prefix(NEEDS_PREFIX.as_bytes()) else { continue };
        let raw: Vec<u8> = super::commit_integrity::unhex(std::str::from_utf8(rest).ok()?)?;
        needs.push(PathBuf::from(std::ffi::OsString::from_vec(raw)));
    }
    return Some(needs);
}

/// Whether the retained `items` cover every dependency `document` names.
fn document_is_covered(lane: &Path, document: &[u8], items: &[ClosureEntry]) -> bool {
    let needs_covered: bool = document_needs(document).is_some_and(|needs| { return needs.iter().all(|need| { return items.iter().any(|entry| { return entry.path == *need; }); }); });
    return needs_covered && checked_document_is_covered(lane, document, items);
}

/// Whether `items` hold every reference of a document in the checked representation: each request, each saved content, and a ledger prefix
/// at least as long as the extent it relies on. A document that starts as a checked document and does not parse is not covered (a malformed
/// reference is never no reference); any other document has no checked reference to check.
fn checked_document_is_covered(lane: &Path, document: &[u8], items: &[ClosureEntry]) -> bool {
    if !document.starts_with(LANE_DOCUMENT_MAGIC.as_bytes()) {
        return true;
    }
    let Some(parsed) = LaneDocument::parse(document) else { return false };
    let held = |directory: &str, name: &str| -> bool { return items.iter().any(|entry| { return entry.path == Path::new(directory).join(name); }); };
    let extent_held = |file: &str, length: Option<u64>| -> bool { return length.is_none_or(|wanted| { return items.iter().any(|entry| { return entry.path == Path::new(file) && entry.length >= wanted; }); }); };
    return parsed.requests.iter().all(|id| { return held(INBOX, id); })
        && parsed.saved.iter().all(|name| { return held(SAVED, name); })
        && parsed.marks.iter().all(|name| { return held(DECISION, name); })
        && extent_held(LANE_LEDGER, parsed.ledger_length)
        && extent_held(LANE_ORDER, parsed.order_length)
        && positions_are_adopt_lines(lane, &parsed);
}

/// Whether each request position of `document` is the start of that request's adopt line in the order prefix the document relies on: a line
/// boundary (the start of the prefix or the byte after a newline) whose line begins `adopt <identity> `. The check reads the order file's
/// bytes, so it holds only for the file the closure's own identity check also validates.
fn positions_are_adopt_lines(lane: &Path, document: &LaneDocument) -> bool {
    let positioned: Vec<(&String, u64)> = document.state.requests.iter().filter_map(|request| { return request.position.map(|at| { return (&request.id, at); }); }).collect();
    if positioned.is_empty() {
        return true;
    }
    let Some(length) = document.order_length.and_then(|length| { return usize::try_from(length).ok(); }) else { return false };
    let Ok(bytes) = fs::read(lane.join(LANE_ORDER)) else { return false };
    let Some(prefix) = bytes.get(..length) else { return false };
    return positioned.iter().all(|(id, at)| {
        let Ok(start) = usize::try_from(*at) else { return false };
        let boundary: bool = start == 0 || prefix.get(start - 1) == Some(&b'\n');
        let wanted: String = format!("adopt {} ", id);
        return boundary && prefix.get(start..).is_some_and(|rest| { return rest.starts_with(wanted.as_bytes()); });
    });
}

/// What:
///  Checks that `closure` holds everything the lane holds that a reclaiming checkpoint could orphan: every request envelope in `inbox/`,
///  every saved content in `saved/` and the lane ledger up to its length now, each unless a ledger line inside the closure discharges it.
/// Why:
///  A reclaiming checkpoint removes its predecessor, the only record that still retains these. What the lane holds is the state the
///  component can see without trusting a document or a caller's list, so the obligations are derived from it. Over-approximation is the
///  safe side: a checkpoint that retains more than its document needs is correct. The ledger extent is the length at this moment, so a
///  prefix captured before later lines were appended is refused and the caller captures again. Dependencies that are not in these places
///  (an order file, say) are declared by the document's `needs` lines and checked as before; the component cannot derive them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkLaneObligations(lane: string, closure: ClosureEntry[]): void; // inbox/*, saved/*, ledger length; or discharged
/// ```
fn check_lane_obligations(lane: &Path, closure: &[ClosureEntry]) -> io::Result<()> {
    let refuse = |what: &str| -> io::Error { return io::Error::new(io::ErrorKind::InvalidInput, format!("the closure does not hold {} that the lane holds", what)); };
    for (directory, kind) in [(INBOX, EntryKind::InboxEnvelope), (SAVED, EntryKind::Opaque), (DECISION, EntryKind::Mark)] {
        let entries = match fs::read_dir(lane.join(directory)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        for listed in entries {
            let name = listed?.file_name();
            let Some(text) = name.to_str() else { continue };
            let obligation: bool = match kind {
                EntryKind::InboxEnvelope => super::commit_integrity::is_identity(text),
                EntryKind::Mark => mark_identity(text).is_some(),
                _ => text.len() == 64 && super::commit_integrity::unhex(text).is_some(),
            };
            if !obligation {
                continue;
            }
            let path: PathBuf = Path::new(directory).join(text);
            let held: bool = closure.iter().any(|entry| { return entry.path == path; });
            let key: String = match kind {
                EntryKind::Opaque => text.chars().take(32).collect(),
                EntryKind::Mark => mark_identity(text).unwrap_or(text).to_string(),
                _ => text.to_string(),
            };
            if !held && !item_is_discharged(lane, closure, kind, &key) {
                return Err(refuse(match kind {
                    EntryKind::Opaque => "saved content",
                    EntryKind::Mark => "a decision mark",
                    _ => "a request",
                }));
            }
        }
    }
    for (file, what) in [(LANE_LEDGER, "the ledger up to its length now (capture it again)"), (LANE_ORDER, "the order file up to its length now (capture it again)")] {
        match fs::metadata(lane.join(file)) {
            Ok(metadata) => {
                let held: bool = closure.iter().any(|entry| { return entry.path == Path::new(file) && entry.length == metadata.len(); });
                if !held {
                    return Err(refuse(what));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

/// What:
///  Captures everything the lane holds that a reclaiming checkpoint must retain: every request envelope in `inbox/`, every saved content in
///  `saved/`, every decision mark and the ledger and the order file at their length now, each as a validated closure entry. Names that are
///  not an obligation (a temporary file, a stray name) are skipped, as `check_lane_obligations` skips them.
/// Why:
///  The planned-shutdown seal is a reclaiming checkpoint of the lane as it is. It builds its closure from the lane itself, by the same scan
///  that the checkpoint then uses to refuse a closure that omits something, so the two cannot disagree about what the lane holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function captureLaneClosure(lane: string): ClosureEntry[]; // inbox/*, saved/*, decision/*, ledger, order
/// ```
pub fn capture_lane_closure(lane: &Path) -> io::Result<Vec<ClosureEntry>> {
    let mut closure: Vec<ClosureEntry> = Vec::new();
    // Saved content whose `disposed` line is in the ledger is never captured again: a root that retained it once more would be invalidated by
    // the unlink that finishes the disposal (review of step 2).
    let disposed: BTreeSet<String> = match fs::read(lane.join(LANE_LEDGER)) {
        Ok(bytes) => super::commit_ledger::complete_entries(&bytes).map_or_else(BTreeSet::new, |entries| { return entries.into_iter().filter(|entry| { return entry.outcome == "disposed"; }).map(|entry| { return entry.id; }).collect(); }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => BTreeSet::new(),
        Err(error) => return Err(error),
    };
    for directory in [INBOX, SAVED, DECISION] {
        let entries = match fs::read_dir(lane.join(directory)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let mut names: Vec<String> = Vec::new();
        for listed in entries {
            if let Some(text) = listed?.file_name().to_str() {
                names.push(text.to_string());
            }
        }
        names.sort();
        for text in names {
            let path: PathBuf = lane.join(directory).join(&text);
            match directory {
                INBOX if super::commit_integrity::is_identity(&text) => closure.push(ClosureEntry::capture(lane, &path, EntryKind::InboxEnvelope)?),
                SAVED if text.len() == 64 && super::commit_integrity::unhex(&text).is_some() && !disposed.contains(&text.chars().take(32).collect::<String>()) => closure.push(ClosureEntry::capture_sealed_saved(lane, &path)?),
                DECISION if mark_identity(&text).is_some() => closure.push(ClosureEntry::capture(lane, &path, EntryKind::Mark)?),
                _ => {}
            }
        }
    }
    for file in [LANE_LEDGER, LANE_ORDER] {
        let path: PathBuf = lane.join(file);
        match fs::metadata(&path) {
            Ok(_) => closure.push(ClosureEntry::capture(lane, &path, EntryKind::AppendPrefix)?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    return Ok(closure);
}

/// Refuses a retained set that names one path twice: the set is a map from paths to identities, and two entries for a path would let the
/// later one decide.
fn reject_duplicate_paths(items: &[ClosureEntry]) -> io::Result<()> {
    let paths: BTreeSet<&Path> = items.iter().map(|entry| { return entry.path.as_path(); }).collect();
    if paths.len() != items.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the retained set names a path twice"));
    }
    return Ok(());
}

/// The first line of a document in the checked representation.
const LANE_DOCUMENT_MAGIC: &str = "cli-git-lane-document 1\n";

/// What:
///  The checked representation of a state document, in two halves. The references are the files of the lane the document depends on: the
///  requests it adopts, the saved content it holds, the decision marks of its live requests, and the extents of the lane ledger and the
///  adoption order it relies on. The state is everything with no file behind it (`LaneState`): the marker, the logical head and tail, the
///  state of every live request and the consumption map. Only a document in this representation can be checkpointed with reclamation; any
///  other document is opaque to the component and its checkpoints never remove a predecessor.
/// Why:
///  A document's dependencies cannot be read out of opaque bytes (a dependency outside the places the component knows has no line to omit
///  and no scan that finds it). A representation that cannot express such a dependency cannot hide one. The split is the classification of
///  the design's state document (`doc/planning/cli-git-commits-never-fail.md`, "State document, the revision counter and the marker"):
///  a request file, a saved content, a mark and the two append-only files are retained by the closure and checked against the lane; the
///  marker, head, tail, request states and consumption map are superseded by the next generation, so a reclaiming checkpoint compares only
///  the references and does not require a newer document to contain the older state.
///  What is not retained: the spool and the attempt directories of a request. The state log does not delete them, so no reclamation can
///  orphan them; the request's own protocol step creates and removes them, and a later step that wants them kept names them as references.
///  A mark is discharged when a ledger line for its request lies inside the new closure: the design ends a request by publishing its
///  outcome to the ledger (T0) before it leaves the document, and the withdrawal mark has no further effect after that (rule V, stop
///  disposition).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LaneDocument = { requests: string[]; saved: string[]; marks: string[]; ledgerLength: number | null; orderLength: number | null; state: LaneState };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LaneDocument {
    /// Request identities, sorted.
    requests: Vec<String>,
    /// Saved content names (the checksum), sorted.
    saved: Vec<String>,
    /// Decision mark names (`<identity>.withdraw` and `<identity>.withdraw-unresolved`), sorted.
    marks: Vec<String>,
    /// The length of the lane ledger the document relies on.
    ledger_length: Option<u64>,
    /// The length of the adoption order the document relies on.
    order_length: Option<u64>,
    /// The state with no file behind it.
    state: LaneState,
}

impl LaneDocument {
    /// The document that names exactly what `closure` holds, with no state; `None` when the closure holds an append-only file that is neither
    /// the lane ledger nor the adoption order, which this representation cannot express.
    pub fn from_closure(closure: &[ClosureEntry]) -> Option<LaneDocument> {
        let mut document: LaneDocument = LaneDocument::default();
        // Two captures of one path would make the document depend on the order of the inputs; a closure names a path once.
        let paths: BTreeSet<&Path> = closure.iter().map(|entry| { return entry.path.as_path(); }).collect();
        if paths.len() != closure.len() {
            return None;
        }
        for entry in closure {
            let name: String = entry.path.file_name().and_then(|name| { return name.to_str(); })?.to_string();
            match entry.kind {
                EntryKind::InboxEnvelope => document.requests.push(name),
                EntryKind::Opaque => document.saved.push(name),
                EntryKind::Mark => document.marks.push(name),
                EntryKind::AppendPrefix if entry.path == Path::new(LANE_LEDGER) => document.ledger_length = Some(entry.length),
                EntryKind::AppendPrefix if entry.path == Path::new(LANE_ORDER) => document.order_length = Some(entry.length),
                EntryKind::AppendPrefix => return None,
            }
        }
        document.requests.sort();
        document.saved.sort();
        document.marks.sort();
        return Some(document);
    }

    /// The saved content names (checksums) the document holds, sorted.
    pub fn saved(&self) -> &[String] {
        return &self.saved;
    }

    /// This document with `state` in place of its state; the references stay.
    pub fn with_state(mut self, state: LaneState) -> LaneDocument {
        self.state = state;
        return self;
    }

    /// The state with no file behind it.
    pub fn state(&self) -> &LaneState {
        return &self.state;
    }

    /// The references alone: the document a closure names. A reclaiming checkpoint compares these with its closure and nothing else.
    pub fn references(&self) -> LaneDocument {
        return LaneDocument { state: LaneState::default(), ..self.clone() };
    }

    /// The document's bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut text: String = String::from(LANE_DOCUMENT_MAGIC);
        for request in &self.requests {
            text.push_str(&format!("request {}\n", request));
        }
        for saved in &self.saved {
            text.push_str(&format!("saved {}\n", saved));
        }
        for mark in &self.marks {
            text.push_str(&format!("mark {}\n", mark));
        }
        if let Some(length) = self.ledger_length {
            text.push_str(&format!("ledger {}\n", length));
        }
        if let Some(length) = self.order_length {
            text.push_str(&format!("order {}\n", length));
        }
        self.state.encode_into(&mut text);
        return text.into_bytes();
    }

    /// Reads `bytes` as a document in this representation; `None` for anything else (an opaque document, extra lines, a repeated line, lines
    /// out of order, a state that names a request the document does not adopt, a mark whose request it does not adopt). A document is read
    /// only if encoding what was read gives back exactly these bytes, so no variant of the bytes is a second spelling of the same document.
    pub fn parse(bytes: &[u8]) -> Option<LaneDocument> {
        let text: &str = std::str::from_utf8(bytes).ok()?.strip_prefix(LANE_DOCUMENT_MAGIC)?;
        let mut document: LaneDocument = LaneDocument::default();
        for line in text.lines() {
            let (key, value): (&str, &str) = line.split_once(' ')?;
            match key {
                "request" => document.requests.push(value.to_string()),
                "saved" => document.saved.push(value.to_string()),
                "mark" => document.marks.push(value.to_string()),
                "ledger" if document.ledger_length.is_none() => document.ledger_length = Some(value.parse().ok()?),
                "order" if document.order_length.is_none() => document.order_length = Some(value.parse().ok()?),
                _ => document.state.apply_line(key, value)?,
            }
        }
        let strictly_sorted = |names: &[String]| -> bool { return names.windows(2).all(|pair| { return pair[0] < pair[1]; }); };
        let marks_ok: bool = document.marks.iter().all(|name| { return mark_identity(name).is_some_and(|identity| { return document.requests.binary_search(&identity.to_string()).is_ok(); }); });
        let canonical: bool = strictly_sorted(&document.requests) && strictly_sorted(&document.saved) && strictly_sorted(&document.marks) && marks_ok;
        let consistent: bool = document.state.is_consistent(&document.requests, document.order_length);
        return if canonical && consistent && document.encode() == bytes { Some(document) } else { None };
    }
}

/// The directory of the lane that holds immutable saved content, one file per content, named by the SHA-256 of its bytes.
const SAVED: &str = "saved";

/// What kind of content a retained item is. The kind follows from where the file lives and from nothing a caller says, so a torn envelope
/// cannot be retained as a prefix and a mutable file cannot be retained as immutable content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// Immutable saved content under `saved/`, named by the SHA-256 of its bytes: a legitimate writer cannot change it without changing
    /// its name, and nothing ever holds a writable descriptor to it after creation. Completion also makes it read-only.
    Opaque,
    /// An inbox request envelope, `inbox/<id>`: its length and checksum must match and it must decode with the identity of its name.
    InboxEnvelope,
    /// An empty decision mark, `decision/<identity>.withdraw` or `decision/<identity>.withdraw-unresolved`: its existence is its content, so
    /// the length is zero and the name must carry a request identity and a mark suffix.
    Mark,
    /// An append-only file anywhere else (a ledger, the order file): the identified prefix must still be there; later lines are allowed.
    AppendPrefix,
}

/// One retained item of a document: a file of the lane, bound to the identity of its content. Fields are private; the only constructor
/// derives the kind from the path and validates the content, and decoding validates the same rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureEntry {
    path: PathBuf,
    kind: EntryKind,
    length: u64,
    sha256: String,
}

impl ClosureEntry {
    /// The path relative to the lane directory.
    pub fn path(&self) -> &Path {
        return &self.path;
    }

    /// How the content is validated.
    pub fn kind(&self) -> EntryKind {
        return self.kind;
    }

    /// The content's length in bytes (for an append-only entry, the length of the identified prefix).
    pub fn length(&self) -> u64 {
        return self.length;
    }

    /// The SHA-256 of the content (for an append-only entry, of the identified prefix), hexadecimal.
    pub fn sha256(&self) -> &str {
        return &self.sha256;
    }

    /// Captures the identity of the file at `path` (under `lane`) by reading it. `declared` is the caller's belief about the kind; the kind
    /// is derived from the path, and a different belief is refused instead of overridden, so a mistaken caller learns it.
    pub fn capture(lane: &Path, path: &Path, declared: EntryKind) -> io::Result<ClosureEntry> {
        let relative: PathBuf = path.strip_prefix(lane).map(PathBuf::from).map_err(|_| { return io::Error::new(io::ErrorKind::InvalidInput, "a closure entry lies under the lane"); })?;
        let kind: EntryKind = kind_of_path(&relative)?;
        if kind != declared {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "the declared kind is not the kind the path has: envelopes live in inbox/, immutable content in saved/<sha256>, anything else is an append-only prefix"));
        }
        if kind == EntryKind::Opaque {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "saved content entries come from create_saved_content only: an existing file, read-only or not, may have a writer that still holds a descriptor"));
        }
        return Self::build(lane, relative, kind, path);
    }

    /// Captures saved content that this component created earlier, for a later closure (the planned-shutdown seal retains what the lane holds):
    /// the file under `saved/` must be named by its checksum, read-only and, where the kernel grants a lease, not open for writing. The premise
    /// is the one of the lane: `saved/` is written only through `create_saved_content`, so a read-only file there has no writer, and a
    /// privileged writer that replaces it is detected by the checksum name and not prevented.
    pub fn capture_sealed_saved(lane: &Path, path: &Path) -> io::Result<ClosureEntry> {
        let relative: PathBuf = path.strip_prefix(lane).map(PathBuf::from).map_err(|_| { return io::Error::new(io::ErrorKind::InvalidInput, "a closure entry lies under the lane"); })?;
        if kind_of_path(&relative)? != EntryKind::Opaque {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "only a file under saved/ is saved content"));
        }
        check_saved_content_sealed(path)?;
        return Self::build(lane, relative, EntryKind::Opaque, path);
    }

    /// Reads the file at `path` and builds its validated entry; the callers decide whether the file may be admitted at all.
    fn build(lane: &Path, relative: PathBuf, kind: EntryKind, path: &Path) -> io::Result<ClosureEntry> {
        let bytes: Vec<u8> = fs::read(path)?;
        let entry: ClosureEntry = ClosureEntry { path: relative, kind, length: bytes.len() as u64, sha256: sha256_hex(&bytes) };
        validate_entry(lane, &entry)?;
        return Ok(entry);
    }
}

/// The error number a test makes the read lease call fail with (a file system that refuses leases); 0 is none.
#[cfg(feature = "test-seams")]
static LEASE_ERRNO: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Makes the read lease check see `errno` (`None` clears it): the control of a file system that grants no lease.
#[cfg(feature = "test-seams")]
pub fn inject_lease_errno(errno: Option<i32>) {
    LEASE_ERRNO.store(errno.unwrap_or(0), std::sync::atomic::Ordering::SeqCst);
}

/// The injected lease error number, if a test set one; always `None` without the test seams.
fn injected_lease_errno() -> Option<i32> {
    #[cfg(feature = "test-seams")]
    return Some(LEASE_ERRNO.load(std::sync::atomic::Ordering::SeqCst)).filter(|errno| { return *errno != 0; });
    #[cfg(not(feature = "test-seams"))]
    return None;
}

/// The lane's private staging directory for saved content on a file system without `O_TMPFILE`: mode 0700, so no other account reaches
/// the staged file between its creation and its sealing.
const SAVED_STAGING: &str = "saved-staging";

/// Whether a test made `O_TMPFILE` unavailable (a file system that does not support it).
#[cfg(feature = "test-seams")]
static NO_TMPFILE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Makes the creation of saved content behave as on a file system without `O_TMPFILE` (`false` clears it).
#[cfg(feature = "test-seams")]
pub fn inject_no_tmpfile(unavailable: bool) {
    NO_TMPFILE.store(unavailable, std::sync::atomic::Ordering::SeqCst);
}

/// Whether `O_TMPFILE` is to be skipped; always false without the test seams.
fn tmpfile_unavailable() -> bool {
    #[cfg(feature = "test-seams")]
    return NO_TMPFILE.load(std::sync::atomic::Ordering::SeqCst);
    #[cfg(not(feature = "test-seams"))]
    return false;
}

/// Writes `bytes` to a file that has no name yet, with `O_TMPFILE` in `directory`; `None` when the file system or the kernel does not
/// support it (the caller falls back to the staging directory).
fn unnamed_file(directory: &Path) -> io::Result<Option<File>> {
    use std::os::fd::FromRawFd;
    use std::os::unix::ffi::OsStrExt;
    if tmpfile_unavailable() {
        return Ok(None);
    }
    let path = std::ffi::CString::new(directory.as_os_str().as_bytes()).map_err(|_| { return io::Error::new(io::ErrorKind::InvalidInput, "a path with a NUL byte"); })?;
    // SAFETY: `open` takes a NUL-terminated path, flags and a mode; the returned descriptor is owned by the `File` made from it below.
    let descriptor: libc::c_int = unsafe { libc::open(path.as_ptr(), libc::O_TMPFILE | libc::O_WRONLY | libc::O_CLOEXEC, 0o600 as libc::c_uint) };
    if descriptor < 0 {
        let error: io::Error = io::Error::last_os_error();
        return match error.raw_os_error() {
            Some(libc::EOPNOTSUPP | libc::EISDIR | libc::EINVAL) => Ok(None),
            _ => Err(error),
        };
    }
    // SAFETY: `descriptor` was just returned by `open` and nothing else owns it.
    return Ok(Some(unsafe { File::from_raw_fd(descriptor) }));
}

/// Gives the unnamed `file` its final `name` without replacing anything: `linkat` through `/proc/self/fd`; `Ok(false)` when that route
/// is unavailable.
fn link_unnamed(file: &File, name: &Path) -> io::Result<bool> {
    use std::os::fd::AsRawFd;
    use std::os::unix::ffi::OsStrExt;
    let source = std::ffi::CString::new(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(|_| { return io::Error::other("a descriptor path"); })?;
    let target = std::ffi::CString::new(name.as_os_str().as_bytes()).map_err(|_| { return io::Error::new(io::ErrorKind::InvalidInput, "a path with a NUL byte"); })?;
    // SAFETY: both arguments are NUL-terminated paths; `AT_SYMLINK_FOLLOW` makes `linkat` follow the `/proc` link to the open file.
    let status: libc::c_int = unsafe { libc::linkat(libc::AT_FDCWD, source.as_ptr(), libc::AT_FDCWD, target.as_ptr(), libc::AT_SYMLINK_FOLLOW) };
    if status == 0 {
        return Ok(true);
    }
    let error: io::Error = io::Error::last_os_error();
    return match error.raw_os_error() {
        Some(libc::EEXIST) => Err(error),
        Some(libc::ENOENT | libc::EPERM | libc::EOPNOTSUPP) => Ok(false),
        _ => Err(error),
    };
}

/// Writes, forces and seals `file` (mode 0444 before it has its final name).
fn fill_and_seal(file: &mut File, bytes: &[u8]) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.write_all(bytes)?;
    file.sync_all()?;
    file.set_permissions(fs::Permissions::from_mode(0o444))?;
    return Ok(());
}

/// What:
///  Creates saved content from `bytes` as `lane/saved/<sha256 of bytes>` and returns its entry, the only way an `Opaque` entry exists.
///  The file is written, forced and made read-only (0444) under a private name in the lane's 0700 `saved-staging` directory (unnamed
///  until complete where `O_TMPFILE` exists), then renamed over the final name, the directories are forced up to the lane, and the only
///  descriptor is closed before the function returns.
/// Why:
///  A checksum name detects a change; it does not stop a writer. Saved content enters the store only through here, from bytes and never as
///  an existing file, and an existing file at the name is replaced and never adopted (its creator is not known, and mode 0444 is not
///  provenance), so no other descriptor to what the lane retains can be writable: the only way to write it afterwards is a deliberate
///  same-user `chmod`, the premise of the lane (as for every file the lane keeps).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createSavedContent(lane: string, bytes: Uint8Array): ClosureEntry; // tmpfile -> write -> fsync -> chmod 444 -> link -> fsync dir
/// ```
pub fn create_saved_content(lane: &Path, bytes: &[u8]) -> io::Result<ClosureEntry> {
    let directory: PathBuf = lane.join(SAVED);
    fs::create_dir_all(&directory)?;
    let name: PathBuf = directory.join(sha256_hex(bytes));
    let staging: PathBuf = ensure_staging(lane)?;
    let temporary: PathBuf = super::commit_identity::temporary_name(&staging)?;
    let linked: bool = match unnamed_file(&staging)? {
        Some(mut file) => {
            fill_and_seal(&mut file, bytes)?;
            link_unnamed(&file, &temporary)?
        }
        None => false,
    };
    if !linked {
        stage_file(&temporary, bytes)?;
    }
    // The fresh inode takes the name over whatever was there: an existing file at the name is never adopted, because its creator is not
    // known and may still hold a writable descriptor. A holder of the replaced inode writes into a file nothing refers to.
    if let Err(error) = fs::rename(&temporary, &name) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    sync_chain(&directory, lane)?;
    sync_directory(&staging)?;
    let relative: PathBuf = PathBuf::from(SAVED).join(sha256_hex(bytes));
    return ClosureEntry::build(lane, relative, EntryKind::Opaque, &name);
}

/// The lane's private staging directory (0700), created if missing.
fn ensure_staging(lane: &Path) -> io::Result<PathBuf> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let staging: PathBuf = lane.join(SAVED_STAGING);
    match fs::DirBuilder::new().mode(0o700).create(&staging) {
        Ok(()) => sync_chain(&staging, lane)?,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            if fs::metadata(&staging)?.permissions().mode() & 0o077 != 0 {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "the saved-content staging directory is not private to its owner"));
            }
        }
        Err(error) => return Err(error),
    }
    return Ok(staging);
}

/// The fallback of `create_saved_content` without `O_TMPFILE`: a new file at the private `temporary` name, written, forced and sealed.
fn stage_file(temporary: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file: File = OpenOptions::new().create_new(true).write(true).mode(0o600).open(temporary)?;
    let outcome: io::Result<()> = fill_and_seal(&mut file, bytes);
    drop(file);
    if outcome.is_err() {
        let _ = fs::remove_file(temporary);
    }
    return outcome;
}

/// What:
///  Checks that the saved content at `path` is immutable by construction: read-only, which only `create_saved_content` leaves it (a file
///  that exists already may have a writer that still holds a descriptor, and is refused). Where the kernel grants a read lease it also
///  refuses content that some process holds open for writing; a file system that refuses leases is not a refusal.
/// Why:
///  The lease is an extra check, never a requirement: the owner's rule is that the wrapper works where a facility is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkSavedContentSealed(path: string): void; // mode has no write bit; where a lease is granted, no writer is open
/// ```
fn check_saved_content_sealed(path: &Path) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::PermissionsExt;
    if fs::metadata(path)?.permissions().mode() & 0o222 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "saved content enters the store only through create_saved_content; an existing writable file is not saved content"));
    }
    let file: File = File::open(path)?;
    // SAFETY: `fcntl` with `F_SETLEASE` takes a descriptor and an integer argument and touches no memory.
    let granted: libc::c_int = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLEASE, libc::F_RDLCK) };
    let refused_with: Option<i32> = injected_lease_errno().or_else(|| { return if granted != 0 { io::Error::last_os_error().raw_os_error() } else { None }; });
    match refused_with {
        Some(libc::EAGAIN) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "saved content is still open for writing by some process; it is not immutable")),
        // Any other refusal means this file system or account grants no lease: the creator-enforced mode is the protection.
        Some(_) => {}
        None => {
            // SAFETY: as above; releasing the lease taken for the check.
            unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLEASE, libc::F_UNLCK) };
        }
    }
    return Ok(());
}

/// The kind a relative path has, refusing paths that are not plain relative names.
fn kind_of_path(relative: &Path) -> io::Result<EntryKind> {
    if relative.components().any(|part| { return !matches!(part, std::path::Component::Normal(_)); }) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "a closure entry path has only normal components"));
    }
    let mut parts = relative.components();
    let first: Option<&std::ffi::OsStr> = parts.next().map(|part| { return part.as_os_str(); });
    let is_direct_child: bool = relative.components().count() == 2;
    return Ok(match first.and_then(|name| { return name.to_str(); }) {
        Some(INBOX) if is_direct_child => EntryKind::InboxEnvelope,
        Some(INBOX) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "inbox/ holds envelopes directly")),
        Some(SAVED) if is_direct_child => EntryKind::Opaque,
        Some(SAVED) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "saved/ holds content directly")),
        Some(DECISION) if is_direct_child => EntryKind::Mark,
        Some(DECISION) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "decision/ holds marks directly")),
        _ => EntryKind::AppendPrefix,
    });
}

/// Reads the entry's file and checks its identity and its kind. The kind is re-derived from the path, so a decoded entry cannot claim a
/// kind its path does not have.
fn validate_entry(lane: &Path, entry: &ClosureEntry) -> io::Result<()> {
    if kind_of_path(&entry.path)? != entry.kind {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "a closure entry claims a kind its path does not have"));
    }
    let bytes: Vec<u8> = fs::read(lane.join(&entry.path))?;
    let identity_ok: bool = match entry.kind {
        EntryKind::AppendPrefix => usize::try_from(entry.length).ok().and_then(|length| { return bytes.get(..length); }).is_some_and(|prefix| { return sha256_hex(prefix) == entry.sha256; }),
        EntryKind::Opaque | EntryKind::InboxEnvelope => bytes.len() as u64 == entry.length && sha256_hex(&bytes) == entry.sha256,
        EntryKind::Mark => bytes.is_empty() && entry.length == 0 && sha256_hex(&bytes) == entry.sha256,
    };
    if !identity_ok {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "a closure entry's content is not the content that was captured"));
    }
    let named: Option<&str> = entry.path.file_name().and_then(|name| { return name.to_str(); });
    match entry.kind {
        EntryKind::InboxEnvelope => {
            let decoded = super::commit_inbox::decode_inbox_file(&bytes).map_err(|_| { return io::Error::new(io::ErrorKind::InvalidData, "a closure entry is not a valid request envelope"); })?;
            if named != Some(decoded.0.id.as_str()) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "an envelope's name is not its identity"));
            }
        }
        EntryKind::Opaque => {
            if named != Some(entry.sha256.as_str()) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "saved content is not named by its checksum"));
            }
        }
        EntryKind::Mark => {
            if !named.is_some_and(|name| { return mark_identity(name).is_some(); }) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "a mark's name is not a request identity and a mark suffix"));
            }
        }
        EntryKind::AppendPrefix => {}
    }
    return Ok(());
}

/// A record's payload, decoded.
enum Payload {
    /// A raw document that retains nothing.
    Plain(Vec<u8>),
    /// A document with its complete retained set.
    Body { items: Vec<ClosureEntry>, document: Vec<u8> },
    /// A replacement-root authority: its document, its retained set and the segments it authorizes for reclamation.
    Authority(Authority),
    /// Starts like a supported format and does not parse: ineligible, never read as a plain document.
    Damaged,
}

/// A decoded replacement-root authority.
struct Authority {
    document: Vec<u8>,
    items: Vec<ClosureEntry>,
    reclaims: Vec<u64>,
    /// The chain hashes of the records of the predecessor segment this authority replaces (the records an acknowledgment may have named).
    lineage: Vec<String>,
}

fn kind_word(kind: EntryKind) -> &'static str {
    return match kind {
        EntryKind::Opaque => "saved",
        EntryKind::InboxEnvelope => "request",
        EntryKind::Mark => "mark",
        EntryKind::AppendPrefix => "prefix",
    };
}

fn item_lines(items: &[ClosureEntry]) -> String {
    let mut text: String = String::new();
    for entry in items {
        text.push_str(&format!("item {} {} {} {}\n", kind_word(entry.kind), entry.length, entry.sha256, super::commit_integrity::hex(entry.path.as_os_str().as_encoded_bytes())));
    }
    return text;
}

fn encode_authority(document: &[u8], items: &[ClosureEntry], reclaims: &[u64], lineage: &[String]) -> Vec<u8> {
    let mut text: String = String::from(AUTHORITY_MAGIC);
    text.push_str(&item_lines(items));
    let numbers: Vec<String> = reclaims.iter().map(|number| { return number.to_string(); }).collect();
    text.push_str(&format!("reclaims {}\nlineage {}\ndocument {}\n", numbers.join(","), lineage.join(","), document.len()));
    let mut bytes: Vec<u8> = text.into_bytes();
    bytes.extend_from_slice(document);
    return bytes;
}

fn encode_body(items: &[ClosureEntry], document: &[u8]) -> Vec<u8> {
    if items.is_empty() {
        return document.to_vec();
    }
    let mut text: String = String::from(BODY_MAGIC);
    text.push_str(&item_lines(items));
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
        return parse_body(body).map_or(Payload::Damaged, |(items, document)| { return Payload::Body { items, document }; });
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

fn parse_item(rest: &str) -> Option<ClosureEntry> {
    use std::os::unix::ffi::OsStringExt;
    let parts: Vec<&str> = rest.split(' ').collect();
    if parts.len() != 4 {
        return None;
    }
    let kind: EntryKind = match parts[0] {
        "request" => EntryKind::InboxEnvelope,
        "saved" => EntryKind::Opaque,
        "mark" => EntryKind::Mark,
        "prefix" => EntryKind::AppendPrefix,
        _ => return None,
    };
    let raw: Vec<u8> = super::commit_integrity::unhex(parts[3])?;
    return Some(ClosureEntry { path: PathBuf::from(std::ffi::OsString::from_vec(raw)), kind, length: parts[1].parse().ok()?, sha256: parts[2].to_string() });
}

fn parse_body(body: &[u8]) -> Option<(Vec<ClosureEntry>, Vec<u8>)> {
    let (lines, document) = header_lines(body)?;
    let items: Vec<ClosureEntry> = lines.iter().map(|line| { return line.strip_prefix("item ").and_then(parse_item); }).collect::<Option<Vec<ClosureEntry>>>()?;
    if reject_duplicate_paths(&items).is_err() {
        return None;
    }
    return Some((items, document));
}

fn parse_authority(body: &[u8]) -> Option<Authority> {
    let (lines, document) = header_lines(body)?;
    let mut items: Vec<ClosureEntry> = Vec::new();
    let mut reclaims: Vec<u64> = Vec::new();
    let mut lineage: Vec<String> = Vec::new();
    for line in lines {
        if let Some(rest) = line.strip_prefix("item ") {
            items.push(parse_item(rest)?);
        } else if let Some(rest) = line.strip_prefix("lineage ") {
            lineage = rest.split(',').filter(|piece| { return !piece.is_empty(); }).map(str::to_string).collect();
        } else {
            let rest: &str = line.strip_prefix("reclaims ")?;
            for number in rest.split(',').filter(|piece| { return !piece.is_empty(); }) {
                reclaims.push(number.parse().ok()?);
            }
        }
    }
    if reject_duplicate_paths(&items).is_err() {
        return None;
    }
    return Some(Authority { document, items, reclaims, lineage });
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
/// the length of the eligible prefix and the complete retained set that record declares.
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
    /// The complete retained set the selected record declares.
    pub items: Vec<ClosureEntry>,
    /// Records follow the eligible prefix in the same file and were dropped because a dependency of theirs is missing.
    pub dropped_suffix: bool,
}

/// The eligible part of a file: the chain of its records up to the first whose retained set does not follow from its predecessor's by
/// non-drop (every item kept, or discharged by a ledger prefix inside the new set), and within that chain the last record whose own
/// retained set validates now. Every record carries its complete set, so a record's own validity never leans on another record's closure;
/// the chain rule is on the declared sets only, which is what lets recovery reach a retirement record whose predecessor's item a disposal
/// has since removed, and keeps a record that dropped an undischarged item from outrunning the dependencies its predecessor declared.
fn eligible(lane: &Path, bytes: &[u8]) -> Option<(Vec<Rec>, usize, Vec<u8>, Vec<ClosureEntry>)> {
    let all: Vec<Rec> = records(bytes);
    if all.is_empty() {
        return None;
    }
    let mut count: usize = 0;
    let mut last_document: Vec<u8> = Vec::new();
    let mut last_items: Vec<ClosureEntry> = Vec::new();
    let mut previous: Option<Vec<ClosureEntry>> = None;
    for (index, record) in all.iter().enumerate() {
        let (items, document): (Vec<ClosureEntry>, Vec<u8>) = match parse_payload(&record.payload) {
            Payload::Plain(plain) => (Vec::new(), plain),
            Payload::Body { items, document } => (items, document),
            Payload::Authority(authority) => (authority.items, authority.document),
            Payload::Damaged => break,
        };
        if let Some(before) = &previous
            && check_non_drop(lane, before, &items, None).is_err()
        {
            break;
        }
        if document_is_covered(lane, &document, &items) && items.iter().all(|entry| { return validate_entry(lane, entry).is_ok(); }) {
            last_document = document;
            last_items = items.clone();
            count = index + 1;
        }
        previous = Some(items);
    }
    if count == 0 {
        return None;
    }
    return Some((all, count, last_document, last_items));
}

/// The file in the state directory that records the newest completed checkpoint: `cli-git-established <number> <generation> <hash>`.
const ESTABLISHED: &str = "state.established";

/// What the establishment record names: the segment, the generation of its first record and the chain hash after it.
struct Established {
    number: u64,
    generation: u64,
    hash: String,
}

/// What:
///  The newest completed checkpoint, as `state.established` names it: `Ok(None)` when no checkpoint ever completed (or the state predates
///  the record), `Err` when the record exists and does not parse (the hash must be 64 lowercase hexadecimal characters).
/// Why:
///  Recovery never falls back past an established root. Without the record a damaged established root with an older usable segment looks
///  like a candidate that never completed, and the older state would be selected silently. The record binds the identity (segment and hash),
///  not only a generation: a checkpoint at the same generation as an older segment, or a valid substitute at the same number, is not it.
fn established_record(dir: &Path) -> Result<Option<Established>, StateError> {
    let text: String = match fs::read_to_string(dir.join(ESTABLISHED)) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(StateError::Io(error)),
    };
    let fields: Vec<&str> = text.strip_suffix('\n').ok_or(StateError::Damaged)?.split(' ').collect();
    let [magic, number, generation, hash] = fields[..] else { return Err(StateError::Damaged) };
    let hash_ok: bool = hash.len() == 64 && super::commit_integrity::unhex(hash).is_some();
    if magic != "cli-git-established" || !hash_ok {
        return Err(StateError::Damaged);
    }
    let (Ok(number), Ok(generation)) = (number.parse::<u64>(), generation.parse::<u64>()) else { return Err(StateError::Damaged) };
    return Ok(Some(Established { number, generation, hash: hash.to_string() }));
}

/// Whether the log, as `chosen` was selected from it, honors the established checkpoint: nothing older than its segment is the root, and
/// when its segment is the root, its first record is the one the record names.
fn honors_establishment(dir: &Path, established: &Established, chosen: Option<&Selected>) -> Result<bool, StateError> {
    let Some(chosen) = chosen else { return Ok(false) };
    if chosen.number < established.number || chosen.generation < established.generation {
        return Ok(false);
    }
    if chosen.number > established.number {
        return Ok(true);
    }
    let bytes: Vec<u8> = fs::read(log_path(dir, established.number))?;
    return Ok(records(&bytes).first().is_some_and(|first| { return first.generation == established.generation && first.hash == established.hash; }));
}

/// Records, durably, that the checkpoint `candidate` is established: written to a temporary name, forced, renamed over the record and the
/// directory forced. Called by completion after the candidate and its directory entry are durable and before any predecessor is removed.
fn establish(dir: &Path, candidate: &Candidate) -> io::Result<()> {
    let temporary: PathBuf = dir.join("state.established.tmp");
    let mut file: File = OpenOptions::new().create(true).write(true).truncate(true).open(&temporary)?;
    file.write_all(format!("cli-git-established {} {} {}\n", candidate.number, candidate.generation, candidate.hash).as_bytes())?;
    sync_file(&file, &temporary)?;
    fs::rename(&temporary, dir.join(ESTABLISHED))?;
    return sync_directory(dir);
}

/// The file recovery selects: the file whose last eligible record has the newest generation (a tie goes to the higher file when the documents
/// agree). A record is eligible when its whole retained set validates; the first ineligible record ends the file's prefix, so a later document
/// cannot outrun the dependencies it declares. No log file is `Ok(None)`; files with bytes and no eligible record anywhere are `Damaged`; a
/// file that cannot be read is `Io`.
pub fn select(dir: &Path) -> Result<Option<Selected>, StateError> {
    let all_numbers: Vec<u64> = numbers(dir)?;
    let highest: Option<u64> = all_numbers.last().copied();
    let lane: PathBuf = lane_of(dir);
    let mut saw_bytes: bool = false;
    let mut best: Option<Selected> = None;
    for number in all_numbers.into_iter().rev() {
        let bytes: Vec<u8> = fs::read(log_path(dir, number))?;
        saw_bytes = saw_bytes || !bytes.is_empty();
        let Some((all, count, document, items)) = eligible(&lane, &bytes) else { continue };
        let last: &Rec = &all[count - 1];
        let candidate: Selected = Selected {
            number,
            generation: last.generation,
            payload: document,
            hash: last.hash.clone(),
            length: last.end,
            highest: Some(number) == highest,
            items,
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
    // An established root that no longer validates is damaged, not a candidate that never completed: nothing older is selected in its place.
    if let Some(established) = established_record(dir)?
        && !honors_establishment(dir, &established, best.as_ref())?
    {
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

/// Whether the lane's authoritative ledger, as identified by the entry in `items` that names it, holds a line for `id` (with `outcome` when
/// given) inside the identified prefix. The entry's prefix must still be the file's prefix; no other retained file counts.
fn ledger_prefix_has(lane: &Path, items: &[ClosureEntry], id: &str, outcome: Option<&str>) -> bool {
    return items.iter().filter(|entry| { return entry.kind == EntryKind::AppendPrefix && entry.path == Path::new(LANE_LEDGER); }).any(|entry| {
        if validate_entry(lane, entry).is_err() {
            return false;
        }
        let Ok(bytes) = fs::read(lane.join(&entry.path)) else { return false };
        let Some(prefix) = usize::try_from(entry.length).ok().and_then(|length| { return bytes.get(..length); }) else { return false };
        return super::commit_ledger::complete_entries(prefix).is_some_and(|lines| { return lines.iter().any(|line| { return line.id == id && outcome.is_none_or(|wanted| { return line.outcome == wanted; }); }); });
    });
}

/// The outcomes that promise nothing saved: a request that ends this way no longer needs its envelope. Any other terminal outcome
/// (`withdrawn`, `conflicted`, `unresolved`, `rewritten-away`) promises the request's saved inputs, and the envelope is one of them: it
/// stays until a `disposed` line for the request is in the ledger (the design, "Retention lifecycle and the spool closure").
const SAVED_FREE_OUTCOMES: [&str; 4] = ["published", "refused", "already", "superseded"];

/// Whether the ledger prefix held in `items` discharges the request envelope of `id`: a line with an outcome that promises nothing saved,
/// or a `disposed` line for the request.
fn request_is_discharged(lane: &Path, items: &[ClosureEntry], id: &str) -> bool {
    return SAVED_FREE_OUTCOMES.iter().any(|outcome| { return ledger_prefix_has(lane, items, id, Some(outcome)); }) || ledger_prefix_has(lane, items, id, Some("disposed"));
}

/// Whether the ledger prefix held in `items` discharges `old`, the item of a kind, under its key: the rule of its kind.
fn item_is_discharged(lane: &Path, items: &[ClosureEntry], kind: EntryKind, key: &str) -> bool {
    return match kind {
        EntryKind::InboxEnvelope => request_is_discharged(lane, items, key),
        EntryKind::Opaque => ledger_prefix_has(lane, items, key, Some("disposed")),
        EntryKind::Mark => ledger_prefix_has(lane, items, key, None),
        EntryKind::AppendPrefix => false,
    };
}

/// The key under which a retained item is discharged: a request by its identity, saved content by the first characters of its name.
fn retirement_key(entry: &ClosureEntry) -> Option<String> {
    let name: &str = entry.path.file_name().and_then(|name| { return name.to_str(); })?;
    return match entry.kind {
        EntryKind::InboxEnvelope => Some(name.to_string()),
        EntryKind::Opaque => Some(name.chars().take(32).collect()),
        EntryKind::Mark => mark_identity(name).map(str::to_string),
        EntryKind::AppendPrefix => None,
    };
}

/// Checks the non-drop obligation: every item of the previous record is in the new set (an append-only prefix may have grown, and its
/// identified prefix must still be the file's prefix) or is discharged. A request is discharged by a ledger line for it in a prefix item of
/// the new set, so the proof is part of what completion forces; saved content only by a `disposed` line. A retirement the caller names but the
/// new set cannot prove is refused. `retired` is the writer's stated intent; recovery passes `None` and accepts any drop the new set's own
/// ledger prefix proves.
fn check_non_drop(lane: &Path, previous: &[ClosureEntry], new: &[ClosureEntry], retired: Option<&[String]>) -> io::Result<()> {
    for old in previous {
        let kept: bool = new.iter().any(|entry| {
            return entry.path == old.path
                && entry.kind == old.kind
                && match old.kind {
                    EntryKind::AppendPrefix => {
                        entry.length >= old.length
                            && fs::read(lane.join(&entry.path)).ok().and_then(|bytes| { return usize::try_from(old.length).ok().and_then(|length| { return bytes.get(..length).map(sha256_hex); }); }).is_some_and(|hash| { return hash == old.sha256; })
                    }
                    EntryKind::Opaque | EntryKind::InboxEnvelope | EntryKind::Mark => entry.length == old.length && entry.sha256 == old.sha256,
                };
        });
        if kept {
            continue;
        }
        let discharged: bool = retirement_key(old).is_some_and(|key| {
            return retired.is_none_or(|names| { return names.contains(&key); }) && item_is_discharged(lane, new, old.kind, &key);
        });
        if !discharged {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "the new document drops a retained item that is not discharged by a ledger line inside its own retained set"));
        }
    }
    return Ok(());
}

/// Appends a record that retains `items` to the newest log file, chained to the file's last eligible record, with no sync. Every item is
/// validated now, none of the previous record's may be dropped unless the new set proves its discharge, and a file with a dropped suffix is
/// not appended to (it needs repair first). Private: the only way in is the `StateWriter` that holds the state's lock.
fn append(dir: &Path, generation: u64, document: &[u8], items: &[ClosureEntry], retired: &[String]) -> io::Result<()> {
    reject_duplicate_paths(items)?;
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
            check_non_drop(&lane, &chosen.items, items, Some(retired))?;
            (chosen.number, chosen.hash.clone(), chosen.length)
        }
        None => (1, genesis(), 0),
    };
    if !document_is_covered(&lane, document, items) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the document names a dependency that the retained set does not cover"));
    }
    for entry in items {
        validate_entry(&lane, entry)?;
    }
    let path: PathBuf = log_path(dir, number);
    let (record, _) = encode_record(generation, &previous, &encode_body(items, document));
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

/// The prefix of a log that a sync made durable: the file, the byte length, the last generation it holds, the chain hash of that record, and
/// the complete retained set that record declares (what a departure must also have forced for the acknowledgment to be a recoverable promise).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prefix {
    /// The log file number.
    pub number: u64,
    /// The bytes of the file that were part of the eligible prefix when the sync began.
    pub length: u64,
    /// The generation of the last record in that prefix.
    pub generation: u64,
    /// The chain hash after that record: the content the acknowledgment is bound to.
    pub hash: String,
    /// The retained set of that record.
    pub items: Vec<ClosureEntry>,
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
    // The state directory's own entry lives in the lane directory, which an empty closure would otherwise never force.
    sync_directory(&lane_of(dir))?;
    return Ok(Prefix { number: selected.number, length: selected.length as u64, generation: selected.generation, hash: selected.hash, items: selected.items });
}

/// Whether the state recovery would select now still carries what `prefix` promised: the same record (or a later record of the same chain),
/// or the segment of a checkpoint whose authority names the promised record among those it replaced and that carries every item of the
/// promised one. Items alone are not succession: a different document that retains the same files does not cover the promise, and support
/// that was dropped or whose content no longer validates is not coverage.
pub fn state_covers(dir: &Path, prefix: &Prefix) -> Result<bool, StateError> {
    let Some(current) = select(dir)? else { return Ok(false) };
    fault_point("state:covers-after-select");
    if current.generation < prefix.generation {
        return Ok(false);
    }
    if current.number == prefix.number && current.length as u64 >= prefix.length {
        let bytes: Vec<u8> = fs::read(log_path(dir, current.number))?;
        let same_record: bool = records(&bytes).iter().any(|record| { return record.end as u64 == prefix.length && record.hash == prefix.hash; });
        if same_record {
            return Ok(true);
        }
    }
    // Across segments the successor must descend from the promised record: the first record of the current segment is a replacement
    // authority that names the promised record's hash among the records it replaced, and nothing the promise retained was dropped except
    // by a discharge its own ledger prefix proves.
    let bytes: Vec<u8> = fs::read(log_path(dir, current.number))?;
    let descends: bool = records(&bytes).first().is_some_and(|first| {
        return matches!(parse_payload(&first.payload), Payload::Authority(authority) if authority.lineage.contains(&prefix.hash));
    });
    if !descends {
        return Ok(false);
    }
    let lane: PathBuf = lane_of(dir);
    return Ok(check_non_drop(&lane, &prefix.items, &current.items, None).is_ok());
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

/// Whether every record of segment `number` is a document in the checked representation whose references the new checked document
/// contains (each request and saved content kept in `closure` or discharged by a ledger line inside it, the ledger no longer than the new
/// one). Only then is the state the segment promised shown to be preserved, and only then may the segment be reclaimed.
fn segment_is_contained(dir: &Path, number: u64, lane: &Path, closure: &[ClosureEntry], new: &LaneDocument) -> io::Result<bool> {
    let bytes: Vec<u8> = fs::read(log_path(dir, number))?;
    for record in records(&bytes) {
        let document: Vec<u8> = match parse_payload(&record.payload) {
            Payload::Plain(plain) => plain,
            Payload::Body { document, .. } => document,
            Payload::Authority(authority) => authority.document,
            Payload::Damaged => return Ok(false),
        };
        let Some(old) = LaneDocument::parse(&document) else { return Ok(false) };
        let requests_kept: bool = old.requests.iter().all(|id| { return new.requests.contains(id) || request_is_discharged(lane, closure, id); });
        let saved_kept: bool = old.saved.iter().all(|name| {
            let key: String = name.chars().take(32).collect();
            return new.saved.contains(name) || ledger_prefix_has(lane, closure, &key, Some("disposed"));
        });
        let marks_kept: bool = old.marks.iter().all(|name| {
            return new.marks.contains(name) || mark_identity(name).is_some_and(|identity| { return ledger_prefix_has(lane, closure, identity, None); });
        });
        let extent_kept = |older: Option<u64>, newer: Option<u64>| -> bool { return older.is_none_or(|length| { return newer.is_some_and(|longer| { return longer >= length; }); }); };
        let ledger_kept: bool = extent_kept(old.ledger_length, new.ledger_length);
        let order_kept: bool = extent_kept(old.order_length, new.order_length);
        if !(requests_kept && saved_kept && marks_kept && ledger_kept && order_kept) {
            return Ok(false);
        }
    }
    return Ok(true);
}

fn begin_checkpoint(dir: &Path, generation: u64, document: &[u8], closure: &[ClosureEntry], retired: &[String], reclaiming: bool) -> io::Result<Candidate> {
    reject_duplicate_paths(closure)?;
    let selected: Option<Selected> = select(dir).map_err(to_io)?;
    let lane: PathBuf = lane_of(dir);
    let existing: Vec<u64> = numbers(dir)?;
    // The closure is validated now, by reading each entry, and may not contain a file the authority will delete (aliases included).
    let canonical_state: PathBuf = fs::canonicalize(dir)?;
    if !document_is_covered(&lane, document, closure) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the document names a dependency that the closure does not cover"));
    }
    let checked: Option<LaneDocument> = if reclaiming {
        let parsed: LaneDocument = LaneDocument::parse(document).ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidInput, "a reclaiming checkpoint needs a document in the checked representation"); })?;
        if LaneDocument::from_closure(closure).as_ref() != Some(&parsed.references()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a reclaiming checkpoint's document must name exactly what its closure holds, and the closure may hold only what the representation expresses"));
        }
        check_lane_obligations(&lane, closure)?;
        Some(parsed)
    } else {
        None
    };
    for entry in closure {
        validate_entry(&lane, entry)?;
        if fs::canonicalize(lane.join(&entry.path))?.starts_with(&canonical_state) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a closure entry lies in the state directory the checkpoint reclaims"));
        }
    }
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
        // Non-drop: the document is the replay of the predecessor's valid prefix, so every item retained there is retained here or discharged
        // by a ledger prefix that is itself part of this closure (and so is forced at completion).
        check_non_drop(&lane, &chosen.items, closure, Some(retired))?;
    }
    let next: u64 = existing
        .last()
        .map_or(Some(1), |number| { return number.checked_add(1); })
        .ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the segment numbers are exhausted"); })?;
    // A segment is reclaimed only when the checkpoint's document is shown to contain the state every record of it promised; an opaque
    // checkpoint, or a segment with an opaque or uncontained record, reclaims nothing. The lineage names the records of the reclaimed
    // segments, and so only records shown to be contained: an acknowledgment of any of them is covered, any other is not.
    let mut reclaims: Vec<u64> = Vec::new();
    let mut lineage: Vec<String> = Vec::new();
    if let Some(new) = &checked {
        for number in &existing {
            if segment_is_contained(dir, *number, &lane, closure, new)? {
                reclaims.push(*number);
                let reclaimed: Vec<Rec> = records(&fs::read(log_path(dir, *number))?);
                // A checkpoint carries the lineage of the authorities it replaces, so an acknowledgment taken before an earlier seal is still
                // covered after a later one.
                for record in &reclaimed {
                    if let Payload::Authority(earlier) = parse_payload(&record.payload) {
                        lineage.extend(earlier.lineage);
                    }
                }
                lineage.extend(reclaimed.into_iter().map(|record| { return record.hash; }));
            }
        }
    }
    let (record, hash) = encode_record(generation, &genesis(), &encode_authority(document, closure, &reclaims, &lineage));
    let mut file: File = OpenOptions::new().create_new(true).write(true).open(log_path(dir, next))?;
    file.write_all(&record)?;
    return Ok(Candidate { number: next, generation, hash });
}

/// Completes exactly `candidate`: it must still be the recovery root with the identity it was started with (so recovery's validation of
/// its whole closure passes now); every closure entry is forced to disk with its complete name chain through the lane directory (the
/// durable anchor) and saved content is made read-only; the candidate and its directory entry are made durable; and only then are the
/// segments the authority names removed. Anything else leaves every older file in place.
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
    for entry in &selected.items {
        if fs::canonicalize(lane.join(&entry.path))?.starts_with(&canonical_state) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a closure entry lies in the state directory the checkpoint reclaims"));
        }
    }
    force_items(&lane, &selected.items)?;
    let path: PathBuf = log_path(dir, candidate.number);
    let file: File = File::open(&path)?;
    sync_file(&file, &path)?;
    sync_directory(dir)?;
    sync_directory(&lane)?;
    establish(dir, candidate)?;
    for number in all.into_iter().filter(|number| { return *number != candidate.number && authority.reclaims.contains(number); }) {
        let old: PathBuf = log_path(dir, number);
        fs::remove_file(&old)?;
        record_event_lazy(|| { return format!("remove:{}", old.display()); });
    }
    return Ok(());
}

/// Forces every retained item with its name chain through the lane directory, and makes saved content read-only (defense in depth: its name
/// is already its checksum).
pub fn force_items(lane: &Path, items: &[ClosureEntry]) -> io::Result<()> {
    for entry in items {
        force_item(lane, entry)?;
    }
    return Ok(());
}

/// Forces one retained item with its name chain through the lane directory, and checks that saved content is still sealed.
pub fn force_item(lane: &Path, entry: &ClosureEntry) -> io::Result<()> {
    let path: PathBuf = lane.join(&entry.path);
    let file: File = File::open(&path)?;
    sync_file(&file, &path)?;
    sync_chain(path.parent().filter(|parent| { return !parent.as_os_str().is_empty(); }).unwrap_or(Path::new(".")), lane)?;
    if entry.kind == EntryKind::Opaque {
        check_saved_content_sealed(&path)?;
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
            Ok(()) => return Ok(StateWriter { _lock: lock, dir: absolute }),
            Err(std::fs::TryLockError::WouldBlock) => return Err(io::Error::new(io::ErrorKind::WouldBlock, "another writer holds the state")),
            Err(std::fs::TryLockError::Error(error)) => return Err(error),
        }
    }

    /// Appends a document that retains nothing (valid only while the previous record retains nothing).
    pub fn append(&mut self, generation: u64, document: &[u8]) -> io::Result<()> {
        return append(&self.dir, generation, document, &[], &[]);
    }

    /// Appends a document with its complete retained set (every item the document depends on, not only the new ones). Each item is
    /// validated now; none of the previous record's may be dropped.
    pub fn append_with(&mut self, generation: u64, document: &[u8], items: &[ClosureEntry]) -> io::Result<()> {
        return append(&self.dir, generation, document, items, &[]);
    }

    /// As `append_with`, retiring the requests (by identity) and disposing the saved content (by name prefix) that a ledger prefix inside
    /// `items` proves discharged.
    pub fn append_retiring(&mut self, generation: u64, document: &[u8], items: &[ClosureEntry], retired: &[String]) -> io::Result<()> {
        return append(&self.dir, generation, document, items, retired);
    }

    /// Starts a checkpoint of an opaque document, which never reclaims a predecessor (the component cannot derive an opaque document's
    /// dependencies): writes `document` and its validated retained set as the first record of the next segment and returns
    /// the candidate to complete. The candidate is not a root until recovery's validation of the set passes.
    pub fn begin_checkpoint(&mut self, generation: u64, document: &[u8], closure: &[ClosureEntry]) -> io::Result<Candidate> {
        return begin_checkpoint(&self.dir, generation, document, closure, &[], false);
    }

    /// As `begin_checkpoint`, retiring requests and disposing saved content that a ledger prefix inside `closure` proves discharged.
    pub fn begin_checkpoint_retiring(&mut self, generation: u64, document: &[u8], closure: &[ClosureEntry], retired: &[String]) -> io::Result<Candidate> {
        return begin_checkpoint(&self.dir, generation, document, closure, retired, false);
    }

    /// Completes exactly `candidate` after forcing its closure with the ancestry; the segments it authorizes are removed last, and every
    /// older file stays on any refusal.
    pub fn complete_checkpoint(&mut self, candidate: &Candidate) -> io::Result<()> {
        return complete_checkpoint(&self.dir, candidate);
    }

    /// Starts a reclaiming checkpoint: `document` is in the checked representation, names exactly what `closure` holds, and the closure holds
    /// everything the lane holds. Segments whose every record is a checked document that `document` contains are reclaimed at completion;
    /// every other segment stays.
    pub fn begin_checkpoint_reclaiming(&mut self, generation: u64, document: &LaneDocument, closure: &[ClosureEntry]) -> io::Result<Candidate> {
        return begin_checkpoint(&self.dir, generation, &document.encode(), closure, &[], true);
    }

    /// Starts a reclaiming checkpoint that also retires: `retired` names the request identities (a request's mark is retired with it) and the
    /// saved-content names the writer says are discharged, and each must be proven by a ledger line inside `closure`. Without it a reclaiming
    /// checkpoint could never drop a request, a mark or saved content, because the non-drop check accepts only what the writer names.
    pub fn begin_checkpoint_reclaiming_retiring(&mut self, generation: u64, document: &LaneDocument, closure: &[ClosureEntry], retired: &[String]) -> io::Result<Candidate> {
        return begin_checkpoint(&self.dir, generation, &document.encode(), closure, retired, true);
    }

    /// A reclaiming checkpoint in one owner step.
    pub fn checkpoint_reclaiming(&mut self, generation: u64, document: &LaneDocument, closure: &[ClosureEntry]) -> io::Result<u64> {
        let candidate: Candidate = begin_checkpoint(&self.dir, generation, &document.encode(), closure, &[], true)?;
        complete_checkpoint(&self.dir, &candidate)?;
        return Ok(candidate.number);
    }

    /// Seals the current segment and replaces it by a checkpoint in one owner step; future appends go to the new segment.
    pub fn checkpoint(&mut self, generation: u64, document: &[u8], closure: &[ClosureEntry]) -> io::Result<u64> {
        let candidate: Candidate = begin_checkpoint(&self.dir, generation, document, closure, &[], false)?;
        complete_checkpoint(&self.dir, &candidate)?;
        return Ok(candidate.number);
    }
}

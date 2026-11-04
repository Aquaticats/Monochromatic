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

use super::commit_identity::{fault_point, record_event, sync_directory, sync_file};
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
    /// A file or the directory could not be read or written.
    Io(io::Error),
}

impl From<io::Error> for StateError {
    fn from(error: io::Error) -> Self {
        return StateError::Io(error);
    }
}

/// The numbers of the log files in `dir`, ascending; a directory that does not exist has none, any other listing failure is an error.
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

/// The longest valid prefix of a log file's bytes: the generation and payload of its last valid record, its chain hash and the length of
/// the valid prefix; `None` when not even the first record validates.
fn valid_prefix(bytes: &[u8]) -> Option<(u64, Vec<u8>, String, usize)> {
    let mut previous: String = genesis();
    let mut at: usize = 0;
    let mut last: Option<(u64, Vec<u8>, String, usize)> = None;
    while at < bytes.len() {
        let Some(offset) = bytes[at..].iter().position(|byte| { return *byte == b'\n'; }) else { break };
        let line_end: usize = at + offset;
        let Ok(header) = std::str::from_utf8(&bytes[at..line_end]) else { break };
        let parts: Vec<&str> = header.split(' ').collect();
        let ok_shape: bool = parts.len() == 4 && parts[0] == "rec" && parts[2] == previous;
        let parsed = if ok_shape { (parts[1].parse::<u64>().ok(), parts[3].parse::<usize>().ok()) } else { (None, None) };
        let (Some(generation), Some(length)) = parsed else { break };
        let start: usize = line_end + 1;
        // A damaged header can name a length near `usize::MAX`; the sums are checked so it ends the prefix instead of overflowing.
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
        last = Some((generation, payload.to_vec(), hash.clone(), trailer_end));
        previous = hash;
        at = trailer_end;
    }
    return last;
}

/// The file a recovery uses and what it holds: its number, the generation and payload of its last valid record, the chain hash there and
/// the length of its valid prefix.
#[derive(Debug)]
pub struct Selected {
    /// The log file number.
    pub number: u64,
    /// The generation of the last valid record.
    pub generation: u64,
    /// The payload of the last valid record.
    pub payload: Vec<u8>,
    /// The chain hash after the last valid record.
    pub hash: String,
    /// The length of the valid prefix in bytes.
    pub length: usize,
    /// Whether this is the highest-numbered file (otherwise a newer candidate exists that does not validate).
    pub highest: bool,
}

/// The file recovery selects: the highest-numbered file whose first record validates, read as its longest valid prefix. No log file is
/// `Ok(None)`; files with bytes and no valid record anywhere are `Damaged`; a file that cannot be read is `Io`, never a silent fall back.
pub fn select(dir: &Path) -> Result<Option<Selected>, StateError> {
    let all: Vec<u64> = numbers(dir)?;
    let highest: Option<u64> = all.last().copied();
    let mut saw_bytes: bool = false;
    let mut best: Option<Selected> = None;
    for number in all.into_iter().rev() {
        let bytes: Vec<u8> = fs::read(log_path(dir, number))?;
        saw_bytes = saw_bytes || !bytes.is_empty();
        if let Some((generation, payload, hash, length)) = valid_prefix(&bytes) {
            // The newest generation wins, and a tie goes to the higher file; a lower file holding a newer generation than a higher one
            // means the higher one is a rollback and is not the root.
            if best.as_ref().is_none_or(|chosen| { return generation > chosen.generation; }) {
                best = Some(Selected { number, generation, payload, hash, length, highest: Some(number) == highest });
            }
        }
    }
    if best.is_none() && saw_bytes {
        return Err(StateError::Damaged);
    }
    return Ok(best);
}

/// The newest valid state (see `select`): its generation and payload.
pub fn read(dir: &Path) -> Result<Option<(u64, Vec<u8>)>, StateError> {
    return Ok(select(dir)?.map(|selected| { return (selected.generation, selected.payload); }));
}

/// What:
///  Appends a record holding the whole document to the newest log file (the first file when there is none), chained to the file's last
///  valid record, with no sync. Bytes after the file's valid prefix (a torn record) are cut first so the chain stays contiguous.
/// Why:
///  The waiting path of a plain commit makes no sync; the log is the only copy of the state, so it is never overwritten in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function append(dir: string, generation: number, payload: Uint8Array): void;
/// ```
pub fn append(dir: &Path, generation: u64, payload: &[u8]) -> io::Result<()> {
    let selected: Option<Selected> = select(dir).map_err(|error| {
        return match error {
            StateError::Io(inner) => inner,
            StateError::Damaged => io::Error::new(io::ErrorKind::InvalidData, "the state log is damaged; its bytes are kept"),
        };
    })?;
    let (number, previous, keep): (u64, String, usize) = match &selected {
        Some(chosen) => {
            if generation <= chosen.generation {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "the generation must increase"));
            }
            if !chosen.highest {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "a newer checkpoint candidate does not validate; complete or abandon it first"));
            }
            (chosen.number, chosen.hash.clone(), chosen.length)
        }
        None => (1, genesis(), 0),
    };
    let path: PathBuf = log_path(dir, number);
    let (record, _) = encode_record(generation, &previous, payload);
    let mut file: File = OpenOptions::new().create(true).write(true).truncate(false).open(&path)?;
    if keep as u64 != file.metadata()?.len() {
        // Bytes after the valid prefix are a torn record; they are cut so the chain stays contiguous.
        file.set_len(keep as u64)?;
    }
    io::Seek::seek(&mut file, io::SeekFrom::Start(keep as u64))?;
    file.write_all(&record)?;
    return Ok(());
}

/// What:
///  Syncs the newest log file and then the directory that holds its name (a departure barrier's step for the state).
/// Why:
///  A synced file whose name is not durable can vanish with its directory entry; the state's name is part of what a departure forces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncNewestLog(dir: string): void;
/// ```
pub fn sync_newest_log(dir: &Path) -> io::Result<()> {
    sync_prefix(dir)?;
    return Ok(());
}

/// The prefix of a log that a sync made durable: the file, the byte length and the last generation it holds.
#[derive(Debug, Eq, PartialEq)]
pub struct Prefix {
    /// The log file number.
    pub number: u64,
    /// The bytes of the file that were part of the valid prefix when the sync began.
    pub length: u64,
    /// The generation of the last record in that prefix.
    pub generation: u64,
}

/// What:
///  Syncs the file that recovery would select (not the highest filename), then its directory, and returns the prefix that was captured
///  before the sync began: records appended while the sync ran are not claimed.
/// Why:
///  A promise names a state, and a flush that reports "everything visible when I returned" would claim bytes it never covered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncPrefix(dir: string): { number: number; length: number; generation: number };
/// ```
pub fn sync_prefix(dir: &Path) -> io::Result<Prefix> {
    let selected: Selected = select(dir)
        .map_err(|error| {
            return match error {
                StateError::Io(inner) => inner,
                StateError::Damaged => io::Error::new(io::ErrorKind::InvalidData, "the state log is damaged"),
            };
        })?
        .ok_or_else(|| { return io::Error::new(io::ErrorKind::NotFound, "no state log"); })?;
    // The prefix is fixed here; records appended while the sync runs are not claimed by it.
    fault_point("state:after-capture");
    let path: PathBuf = log_path(dir, selected.number);
    let file: File = File::open(&path)?;
    sync_file(&file, &path)?;
    sync_directory(dir)?;
    return Ok(Prefix { number: selected.number, length: selected.length as u64, generation: selected.generation });
}

/// What:
///  Starts a checkpoint: a new log file `state.log.<k+1>` whose first record is a full document; returns its number. The old file stays.
/// Why:
///  A rotation never destroys the only valid log; until the new file is durable the previous file is the fallback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rotate(dir: string, generation: number, document: Uint8Array): number;
/// ```
pub fn rotate(dir: &Path, generation: u64, document: &[u8]) -> io::Result<u64> {
    let selected: Option<Selected> = select(dir).map_err(|error| {
        return match error {
            StateError::Io(inner) => inner,
            StateError::Damaged => io::Error::new(io::ErrorKind::InvalidData, "the state log is damaged"),
        };
    })?;
    if let Some(chosen) = &selected {
        if generation < chosen.generation {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a checkpoint must not hold an older generation than the log"));
        }
        if !chosen.highest {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "an earlier checkpoint candidate does not validate"));
        }
    }
    let next: u64 = rotation_number(dir)?.map_or(1, |number| { return number + 1; });
    let (record, _) = encode_record(generation, &genesis(), document);
    let mut file: File = OpenOptions::new().create_new(true).write(true).open(log_path(dir, next))?;
    file.write_all(&record)?;
    return Ok(next);
}

/// What:
///  Finishes a rotation: syncs the new file, then the directory (its entry), and only then removes the older files.
/// Why:
///  Recovery falls back to the previous file when the newest does not validate, so the previous file may go only after the new one is D.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function completeRotation(dir: string): void;
/// ```
pub fn complete_rotation(dir: &Path) -> io::Result<()> {
    let all: Vec<u64> = numbers(dir)?;
    let Some(newest) = all.last().copied() else { return Ok(()) };
    // The candidate must itself be the recovery root: it validates, and no older file holds a newer generation. Otherwise nothing is
    // deleted, so the older valid roots stay.
    let selected: Selected = select(dir)
        .map_err(|error| {
            return match error {
                StateError::Io(inner) => inner,
                StateError::Damaged => io::Error::new(io::ErrorKind::InvalidData, "the state log is damaged"),
            };
        })?
        .ok_or_else(|| { return io::Error::new(io::ErrorKind::NotFound, "no state log"); })?;
    if selected.number != newest {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "the checkpoint candidate does not validate; the older files are kept"));
    }
    for number in all.iter().filter(|number| { return **number != newest; }) {
        let older: Vec<u8> = fs::read(log_path(dir, *number))?;
        if valid_prefix(&older).is_some_and(|(generation, _, _, _)| { return generation > selected.generation; }) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "an older file holds a newer generation than the candidate; it is kept"));
        }
    }
    let path: PathBuf = log_path(dir, newest);
    let file: File = File::open(&path)?;
    sync_file(&file, &path)?;
    sync_directory(dir)?;
    for number in all.into_iter().filter(|number| { return *number != newest; }) {
        let old: PathBuf = log_path(dir, number);
        fs::remove_file(&old)?;
        record_event(&format!("remove:{}", old.display()));
    }
    return Ok(());
}

/// What:
///  The single writer of a state directory, proved by a non-blocking exclusive lock on `state.writer` that lives as long as the value.
///  Appends and checkpoints take `&mut self`, so one owner cannot interleave them, and a second owner is refused at once.
/// Why:
///  An append that selected a segment, a checkpoint that snapshotted it and a completion that deletes it must not interleave: a
///  transition appended to the old segment after the snapshot is lost when the old segment goes (the cutover race).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StateWriter { static acquire(dir: string): StateWriter; append(g: number, p: Uint8Array): void; checkpoint(g: number, d: Uint8Array): number; }
/// ```
#[derive(Debug)]
pub struct StateWriter {
    /// The locked file; dropping the writer releases the lock.
    _lock: File,
    /// The state directory.
    dir: PathBuf,
}

impl StateWriter {
    /// Takes the writer lock without waiting; `WouldBlock` when another writer holds it.
    pub fn acquire(dir: &Path) -> io::Result<StateWriter> {
        let lock: File = OpenOptions::new().create(true).write(true).truncate(false).open(dir.join("state.writer"))?;
        match lock.try_lock() {
            Ok(()) => return Ok(StateWriter { _lock: lock, dir: dir.to_path_buf() }),
            Err(std::fs::TryLockError::WouldBlock) => return Err(io::Error::new(io::ErrorKind::WouldBlock, "another writer holds the state")),
            Err(std::fs::TryLockError::Error(error)) => return Err(error),
        }
    }

    /// Appends a record as `append` does, as the only writer.
    pub fn append(&mut self, generation: u64, payload: &[u8]) -> io::Result<()> {
        return append(&self.dir, generation, payload);
    }

    /// Seals the current segment and replaces it by a checkpoint in one owner step: the document is written as the first record of the
    /// next segment, validated, made durable, and only then are older segments removed. Future appends go to the new segment.
    pub fn checkpoint(&mut self, generation: u64, document: &[u8]) -> io::Result<u64> {
        let number: u64 = rotate(&self.dir, generation, document)?;
        complete_rotation(&self.dir)?;
        return Ok(number);
    }
}

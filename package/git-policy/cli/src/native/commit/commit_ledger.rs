//! What:
//!  The permanent ledger of retired requests: an append-only file with one self-checking line per request, a tail
//!  repair that truncates only a torn last append (keeping the removed bytes beside it), and a lookup by identity.
//! Why:
//!  "Retirement and permanent deduplication" (`doc/planning/cli-git-commits-never-fail.md`) answers a resend from this
//!  ledger after any delay, so the file has no horizon, no rotation and no deletion; this module has no function that
//!  removes or rewrites a line.
//!  A damaged line that is not the last is reported, never truncated: the lines behind it are good data.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // appendFileSync(path, `${crc} ${id} ${outcome} ${commit} ${time}\n`); fsync(fd);
//! ```

// What:     `use std::...` names items of Rust's standard library; the braces import several at once.
//           `fs` is the file-system module, `io` the input/output module, `Path` a borrowed file path.
// Why:      The ledger reads, appends and truncates a file and reports failures as I/O errors.
//
// In TS you'd write (pseudocode):
// ```ts
// import { openSync, writeSync, fsyncSync, ftruncateSync } from 'node:fs';
// ```
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use super::commit_identity::{create_dir_all_durable, fault_point, record_event_lazy, sync_chain, sync_directory, sync_file};
use super::commit_integrity::{crc32, is_identity, is_object_name};

/// What:
///  `struct LedgerEntry { ... }`: the four fields of a terminal line.
///  Each `String` is an owned text (sibling `&str`, borrowed).
/// Why:
///  A line is `<identity> <outcome> <commit or -> <time>`; the owned texts outlive the file read that produced them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LedgerEntry = { id: string; outcome: string; commit: string; time: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerEntry {
    /// The request identity, 32 lowercase hexadecimal characters.
    pub id: String,
    /// The outcome word: lowercase ASCII letters and hyphens, at most 32.
    pub outcome: String,
    /// The commit object name, or `-` for an outcome with no commit.
    pub commit: String,
    /// The time, an ISO 8601 text with an offset and no spaces.
    pub time: String,
}

/// What:
///  `enum LedgerError { ... }`: why a ledger operation failed (TS: a union of error shapes).
/// Why:
///  A custom error type and not a bare `io::Error`, so a caller can tell a damaged middle line (data worth keeping)
///  from an I/O failure and from a field that cannot be written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LedgerError extends Error { kind: 'io' | 'invalid-field' | 'damaged-line'; }
/// ```
#[derive(Debug)]
pub enum LedgerError {
    /// A file system call failed.
    Io(io::Error),
    /// A field cannot be written as a line (empty, a space, a newline, or not the right shape).
    InvalidField(&'static str),
    /// A line that is not the last one fails its check; the lines behind it are intact and nothing was truncated.
    DamagedLine(usize),
    /// Another process holds the exclusive owner lock of this ledger.
    OwnedElsewhere,
}

impl From<io::Error> for LedgerError {
    /// What:
    ///  `impl From<io::Error> for LedgerError` lets `?` turn an `io::Error` into a `LedgerError::Io` on its own.
    /// Why:
    ///  Every file call below can fail with an I/O error and is written with `?`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // catch (e) { throw new LedgerError('io', e) }
    /// ```
    fn from(error: io::Error) -> LedgerError {
        return LedgerError::Io(error);
    }
}

/// The longest `time` field a line may carry (an ISO 8601 text with an offset is 25 bytes; the bound leaves room and keeps every line
/// inside `TAIL_WINDOW`).
pub const TIME_MAX: usize = 64;

/// The bytes a clean append reads from the end of the ledger.
pub const TAIL_WINDOW: u64 = 512;

/// The longest valid line: checksum, identity, outcome (32), commit (64), time (`TIME_MAX`), five separators and the newline.
pub const MAX_LINE_BYTES: usize = 8 + 1 + 32 + 1 + 32 + 1 + 64 + 1 + TIME_MAX + 1;

/// Whether `text` is a field of the line: non-empty, no space, no control character.
fn plain_field(text: &str) -> bool {
    return !text.is_empty() && text.chars().all(|c| { return c.is_ascii_graphic(); });
}

/// What:
///  The bytes of one ledger line for `entry`: `<crc32 as 8 hex> <id> <outcome> <commit> <time>\n`, the CRC over the
///  text after its own space.
/// Why:
///  One line, one append, one check; the fields are validated so a line always parses back to the same entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encodeLine(entry: LedgerEntry): Uint8Array;
/// ```
pub fn encode_line(entry: &LedgerEntry) -> Result<Vec<u8>, LedgerError> {
    if !is_identity(&entry.id) {
        return Err(LedgerError::InvalidField("id"));
    }
    let outcome_ok: bool = !entry.outcome.is_empty()
        && entry.outcome.len() <= 32
        && entry.outcome.chars().all(|c| { return c.is_ascii_lowercase() || c == '-'; });
    if !outcome_ok {
        return Err(LedgerError::InvalidField("outcome"));
    }
    if entry.commit != "-" && !is_object_name(&entry.commit) {
        return Err(LedgerError::InvalidField("commit"));
    }
    if !plain_field(&entry.time) || entry.time.len() > TIME_MAX {
        return Err(LedgerError::InvalidField("time"));
    }
    let body: String = format!("{} {} {} {}", entry.id, entry.outcome, entry.commit, entry.time);
    let line: String = format!("{:08x} {}\n", crc32(body.as_bytes()), body);
    return Ok(line.into_bytes());
}

/// What:
///  The entry a line (without its newline) spells, or `None` when the line fails its check or its shape.
/// Why:
///  The one reader of the format; tail repair, lookup and the tests all go through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLine(line: Uint8Array): LedgerEntry | undefined;
/// ```
pub fn parse_line(line: &[u8]) -> Option<LedgerEntry> {
    let text: &str = std::str::from_utf8(line).ok()?;
    let (checksum, body) = text.split_once(' ')?;
    if checksum.len() != 8 || format!("{:08x}", crc32(body.as_bytes())) != checksum {
        return None;
    }
    let fields: Vec<&str> = body.split(' ').collect();
    if fields.len() != 4 {
        return None;
    }
    let entry: LedgerEntry =
        LedgerEntry { id: fields[0].to_string(), outcome: fields[1].to_string(), commit: fields[2].to_string(), time: fields[3].to_string() };
    // Only lines this module could have written count as valid: re-encoding must give the same bytes.
    let again: Vec<u8> = encode_line(&entry).ok()?;
    let mut expected: Vec<u8> = line.to_vec();
    expected.push(b'\n');
    if again != expected {
        return None;
    }
    return Some(entry);
}

/// What:
///  Splits `bytes` into its complete lines and classifies the end: the length of the valid prefix, and the number of
///  the first damaged line when a damaged line is followed by a valid one (a damaged middle).
///  `(usize, Option<usize>)` is a pair: a length and, maybe, a line number.
/// Why:
///  Only a damaged tail may be truncated; a damaged middle is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(bytes: Uint8Array): [number, number | undefined];
/// ```
fn scan(bytes: &[u8]) -> (usize, Option<usize>) {
    let mut valid_end: usize = 0;
    let mut first_damaged: Option<usize> = None;
    let mut damaged_then_valid: Option<usize> = None;
    let mut start: usize = 0;
    let mut number: usize = 0;
    while start < bytes.len() {
        let newline: Option<usize> = bytes[start..].iter().position(|b| { return *b == b'\n'; });
        let complete: bool = newline.is_some();
        let end: usize = start + newline.map_or(bytes.len() - start, |at| { return at; });
        let valid: bool = complete && parse_line(&bytes[start..end]).is_some();
        if valid {
            if first_damaged.is_some() && damaged_then_valid.is_none() {
                damaged_then_valid = first_damaged;
            }
            valid_end = end + 1;
        } else if first_damaged.is_none() {
            first_damaged = Some(number);
        }
        number += 1;
        start = end + 1;
    }
    return (valid_end, damaged_then_valid);
}

/// What:
///  The result of a tail repair: how many bytes stay and how many were cut off.
/// Why:
///  A caller (and a test) can tell a clean file from a torn one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Repair = { kept: number; cut: number };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Repair {
    /// The length of the valid prefix, now the length of the file.
    pub kept: u64,
    /// The number of bytes cut from the end and saved beside the ledger.
    pub cut: u64,
}

/// The path of the file that keeps the bytes cut by tail repairs of `ledger`.
fn torn_path(ledger: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = ledger.as_os_str().to_os_string();
    name.push(".torn");
    return PathBuf::from(name);
}

/// What:
///  Truncates `ledger` to its valid prefix when only the tail is torn, after appending the cut bytes to
///  `<ledger>.torn` and syncing it; returns what stayed and what was cut. A missing file is an empty ledger.
/// Why:
///  Ignoring a torn tail and appending behind it is not enough: the new line would follow garbage. Cutting without
///  keeping the bytes would discard data, so they are saved first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repairTail(path: string): Repair;
/// ```
fn repair_tail(ledger: &Path) -> Result<Repair, LedgerError> {
    let mut bytes: Vec<u8> = Vec::new();
    match File::open(ledger) {
        Ok(mut file) => {
            file.read_to_end(&mut bytes)?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(Repair { kept: 0, cut: 0 });
        }
        Err(error) => {
            return Err(LedgerError::Io(error));
        }
    }
    record_event_lazy(|| { return format!("read-bytes:{}", bytes.len()); });
    let (valid_end, damaged_middle) = scan(&bytes);
    if let Some(number) = damaged_middle {
        return Err(LedgerError::DamagedLine(number));
    }
    let cut: usize = bytes.len() - valid_end;
    if cut > 0 {
        let mut saved: File = OpenOptions::new().create(true).append(true).open(torn_path(ledger))?;
        saved.write_all(&bytes[valid_end..])?;
        sync_file(&saved, &torn_path(ledger))?;
        sync_parent(&torn_path(ledger))?;
        fault_point("ledger:before-truncate");
        let file: File = OpenOptions::new().write(true).open(ledger)?;
        file.set_len(valid_end as u64)?;
        record_event_lazy(|| { return format!("truncate:{}", ledger.display()); });
        sync_file(&file, ledger)?;
    }
    return Ok(Repair { kept: valid_end as u64, cut: cut as u64 });
}

/// What:
///  `Some(repair)` when the ledger is empty or its last line is a complete, valid line (read from the last bytes only); `None` when
///  the tail needs the full scan.
/// Why:
///  Every append used to read and parse the whole ledger; the ledger grows without end, so the cost of a commit would grow with
///  its history. The tail window is a few hundred bytes (a line is about a hundred and thirty).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanTail(path: string): Repair | undefined;
/// ```
fn clean_tail(ledger: &Path) -> Result<Option<Repair>, LedgerError> {
    let mut file: File = match File::open(ledger) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Some(Repair { kept: 0, cut: 0 })),
        Err(error) => return Err(LedgerError::Io(error)),
    };
    let length: u64 = file.metadata()?.len();
    if length == 0 {
        return Ok(Some(Repair { kept: 0, cut: 0 }));
    }
    let window: u64 = length.min(TAIL_WINDOW);
    file.seek(SeekFrom::Start(length - window))?;
    let mut tail: Vec<u8> = vec![0; window as usize];
    file.read_exact(&mut tail)?;
    record_event_lazy(|| { return format!("read-bytes:{}", tail.len()); });
    if tail.last() != Some(&b'\n') {
        return Ok(None);
    }
    let body: &[u8] = &tail[..tail.len() - 1];
    let line_start: usize = match body.iter().rposition(|byte| { return *byte == b'\n'; }) {
        Some(at) => at + 1,
        None if window == length => 0,
        None => return Ok(None),
    };
    if parse_line(&body[line_start..]).is_some() {
        return Ok(Some(Repair { kept: length, cut: 0 }));
    }
    return Ok(None);
}

/// Syncs the directory that holds `path`, so a newly created name survives power loss.
fn sync_parent(path: &Path) -> io::Result<()> {
    // A bare relative name has the empty parent, which is the current directory.
    let parent: PathBuf = match path.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => directory.to_path_buf(),
        _ => PathBuf::from("."),
    };
    return sync_directory(&parent);
}

/// What:
///  The path of a lane's own ledger: `<common dir>/ledgers/<instance>/ledger`.
/// Why:
///  Revision 9 gives every lane its own ledger with exactly one appender, so no repair or append of one lane can truncate a record
///  of another; a lane repairs only this path, and a reader repairs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function laneLedgerPath(common: string, instance: string): string;
/// ```
pub fn lane_ledger_path(common_dir: &Path, instance: &str) -> Result<PathBuf, LedgerError> {
    if !is_identity(instance) {
        return Err(LedgerError::InvalidField("instance"));
    }
    return Ok(common_dir.join("ledgers").join(instance).join("ledger"));
}

/// What:
///  Appends one line for `entry` to `ledger` after repairing a torn tail, and syncs the file (and its directory when the
///  file is new). An entry whose identity already has a line is still appended: the first line stays the answer.
/// Why:
///  Retirement writes this line before it releases anything, so it must be durable when this returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function append(path: string, entry: LedgerEntry): void;
/// ```
fn append(anchor: &Path, ledger: &Path, entry: &LedgerEntry) -> Result<(), LedgerError> {
    let line: Vec<u8> = encode_line(entry)?;
    // The parent chain through the anchor is made durable before a repair can destroy anything: the saved tail must not sit under a
    // directory name that a power loss could remove. A bare relative file name has the current directory as its parent.
    let parent: PathBuf = match ledger.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => directory.to_path_buf(),
        _ => PathBuf::from("."),
    };
    create_dir_all_durable(anchor, &parent)?;
    // The commit's waiting path reads only the last bytes of a clean ledger; the full scan (which also finds a damaged middle) runs
    // only when the last line is not a complete valid line. A damaged middle is found by `repair_tail` and `lookup`, not by every append.
    if clean_tail(ledger)?.is_none() {
        repair_tail(ledger)?;
    }
    let mut file: File = OpenOptions::new().create(true).append(true).open(ledger)?;
    file.write_all(&line)?;
    sync_file(&file, ledger)?;
    // Always, through the caller's durable anchor: the file or a directory above it may exist because a writer that died created
    // it before syncing.
    sync_chain(&parent, anchor)?;
    return Ok(());
}

/// What:
///  The first entry for `id` in `ledger`, or `None`; a torn tail is ignored (not cut), a damaged middle is an error.
/// Why:
///  A resend of an identity is answered from the first line ever written for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lookup(path: string, id: string): LedgerEntry | undefined;
/// ```
pub fn lookup(ledger: &Path, id: &str) -> Result<Option<LedgerEntry>, LedgerError> {
    let bytes: Vec<u8> = match fs::read(ledger) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(LedgerError::Io(error)),
    };
    let (valid_end, damaged_middle) = scan(&bytes);
    if let Some(number) = damaged_middle {
        return Err(LedgerError::DamagedLine(number));
    }
    for line in bytes[..valid_end].split(|b| { return *b == b'\n'; }) {
        // `if let Some(entry) = parse_line(line) && entry.id == id` runs the block when the line parses and names `id`.
        if let Some(entry) = parse_line(line)
            && entry.id == id
        {
            return Ok(Some(entry));
        }
    }
    return Ok(None);
}

/// Reads the whole ledger once and refuses a damaged middle; a torn tail is allowed (an append repairs it) and a missing file is empty.
fn validate_history(ledger: &Path) -> Result<(), LedgerError> {
    let bytes: Vec<u8> = match fs::read(ledger) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(LedgerError::Io(error)),
    };
    let (_, damaged_middle) = scan(&bytes);
    if let Some(number) = damaged_middle {
        return Err(LedgerError::DamagedLine(number));
    }
    return Ok(());
}

/// What:
///  `struct LedgerOwner { ... }`: the exclusive owner of one lane's ledger, proved by a non-blocking exclusive lock on `<ledger>.owner`
///  that lives as long as the value. Only an owner may repair or append through it.
/// Why:
///  Repair truncates and an append writes behind the end; two processes doing either on one ledger can truncate each other's record.
///  The contract "exactly one appender" is enforced by the lock, not by convention, and a second acquirer is refused at once, never
///  made to wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LedgerOwner { static acquire(ledger: string): LedgerOwner; /* throws OwnedElsewhere */ }
/// ```
#[derive(Debug)]
pub struct LedgerOwner {
    /// The locked file; closing it (dropping the owner) releases the lock.
    _lock: File,
    /// The ledger this owner owns, absolute.
    ledger: PathBuf,
}

impl LedgerOwner {
    /// Takes the ledger's owner lock without waiting; `OwnedElsewhere` when another process (or another owner value) holds it.
    pub fn acquire(ledger: &Path) -> Result<LedgerOwner, LedgerError> {
        let mut name: std::ffi::OsString = ledger.as_os_str().to_os_string();
        name.push(".owner");
        let lock_path: PathBuf = PathBuf::from(name);
        // A foreign FIFO at the lock path would block the open before `try_lock` could refuse; only a regular file (or nothing) is opened.
        if let Ok(metadata) = fs::symlink_metadata(&lock_path)
            && !metadata.file_type().is_file()
        {
            return Err(LedgerError::InvalidField("the owner lock path is not a regular file"));
        }
        let lock: File = std::os::unix::fs::OpenOptionsExt::custom_flags(OpenOptions::new().create(true).write(true).truncate(false), libc::O_NONBLOCK)
            .open(lock_path)?;
        match lock.try_lock() {
            Ok(()) => {
                // The one full validation of this owner's lifetime: every append behind it reads only the tail, which is sound because
                // this owner is the only appender and the history it appends to has no damaged middle.
                validate_history(ledger)?;
                return Ok(LedgerOwner { _lock: lock, ledger: std::path::absolute(ledger)? });
            }
            Err(std::fs::TryLockError::WouldBlock) => return Err(LedgerError::OwnedElsewhere),
            Err(std::fs::TryLockError::Error(error)) => return Err(LedgerError::Io(error)),
        }
    }

    /// Makes the ledger's parent chain durable through `anchor` (a fresh lane has none yet), then takes the owner lock.
    pub fn acquire_in(anchor: &Path, ledger: &Path) -> Result<LedgerOwner, LedgerError> {
        let parent: PathBuf = match ledger.parent() {
            Some(directory) if !directory.as_os_str().is_empty() => directory.to_path_buf(),
            _ => PathBuf::from("."),
        };
        create_dir_all_durable(anchor, &parent)?;
        return LedgerOwner::acquire(ledger);
    }

    /// Whether `ledger` is the ledger this owner owns; an owner of one ledger may not touch another.
    fn owns(&self, ledger: &Path) -> Result<(), LedgerError> {
        if std::path::absolute(ledger)? != self.ledger {
            return Err(LedgerError::InvalidField("another owner's ledger"));
        }
        return Ok(());
    }

    /// Repairs the torn tail of `ledger` after making its parent chain durable through `anchor` (so the saved bytes cannot sit under a
    /// name a power loss could remove).
    pub fn repair_tail(&mut self, anchor: &Path, ledger: &Path) -> Result<Repair, LedgerError> {
        self.owns(ledger)?;
        let parent: PathBuf = match ledger.parent() {
            Some(directory) if !directory.as_os_str().is_empty() => directory.to_path_buf(),
            _ => PathBuf::from("."),
        };
        create_dir_all_durable(anchor, &parent)?;
        return repair_tail(ledger);
    }

    /// Appends one line as `append` does, as the ledger's owner.
    pub fn append(&mut self, anchor: &Path, ledger: &Path, entry: &LedgerEntry) -> Result<(), LedgerError> {
        self.owns(ledger)?;
        return append(anchor, ledger, entry);
    }
}

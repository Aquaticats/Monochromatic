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
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

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
    if !plain_field(&entry.time) {
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
pub fn repair_tail(ledger: &Path) -> Result<Repair, LedgerError> {
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
    let (valid_end, damaged_middle) = scan(&bytes);
    if let Some(number) = damaged_middle {
        return Err(LedgerError::DamagedLine(number));
    }
    let cut: usize = bytes.len() - valid_end;
    if cut > 0 {
        let mut saved: File = OpenOptions::new().create(true).append(true).open(torn_path(ledger))?;
        saved.write_all(&bytes[valid_end..])?;
        saved.sync_all()?;
        let file: File = OpenOptions::new().write(true).open(ledger)?;
        file.set_len(valid_end as u64)?;
        file.sync_all()?;
    }
    return Ok(Repair { kept: valid_end as u64, cut: cut as u64 });
}

/// Syncs the directory that holds `path`, so a newly created name survives power loss.
fn sync_parent(path: &Path) -> io::Result<()> {
    let parent: &Path = path.parent().unwrap_or_else(|| { return Path::new("."); });
    return File::open(parent)?.sync_all();
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
pub fn append(ledger: &Path, entry: &LedgerEntry) -> Result<(), LedgerError> {
    let line: Vec<u8> = encode_line(entry)?;
    repair_tail(ledger)?;
    let existed: bool = ledger.exists();
    let mut file: File = OpenOptions::new().create(true).append(true).open(ledger)?;
    file.write_all(&line)?;
    file.sync_all()?;
    if !existed {
        sync_parent(ledger)?;
    }
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
        if let Some(entry) = parse_line(line) {
            if entry.id == id {
                return Ok(Some(entry));
            }
        }
    }
    return Ok(None);
}

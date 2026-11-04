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

use super::commit_identity::{record_event, sync_directory, sync_file};
use super::commit_integrity::sha256_hex;

/// The chain hash before the first record of a file.
fn genesis() -> String {
    return "0".repeat(64);
}

/// The path of log file number `number` in `dir`.
fn log_path(dir: &Path, number: u64) -> PathBuf {
    return dir.join(format!("state.log.{}", number));
}

/// The numbers of the log files in `dir`, ascending.
fn numbers(dir: &Path) -> Vec<u64> {
    let mut found: Vec<u64> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(number) = entry.file_name().to_str().and_then(|name| { return name.strip_prefix("state.log."); }).and_then(|n| { return n.parse::<u64>().ok(); }) {
                found.push(number);
            }
        }
    }
    found.sort();
    return found;
}

/// The highest log file number in `dir`, if any.
pub fn rotation_number(dir: &Path) -> Option<u64> {
    return numbers(dir).last().copied();
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
        let end: usize = start + length;
        let trailer_end: usize = end + 1 + 5 + 64 + 1;
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

/// The newest valid state: the highest-numbered file whose first record validates, read as its longest valid prefix; a newest file that
/// does not validate falls back to the previous one (it exists until the new one is durable).
pub fn read(dir: &Path) -> Option<(u64, Vec<u8>)> {
    for number in numbers(dir).into_iter().rev() {
        let bytes: Vec<u8> = match fs::read(log_path(dir, number)) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if let Some((generation, payload, _, _)) = valid_prefix(&bytes) {
            return Some((generation, payload));
        }
    }
    return None;
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
    let number: u64 = rotation_number(dir).unwrap_or(1);
    let path: PathBuf = log_path(dir, number);
    let existing: Vec<u8> = fs::read(&path).unwrap_or_default();
    let (previous, keep): (String, usize) = match valid_prefix(&existing) {
        Some((_, _, hash, length)) => (hash, length),
        None => (genesis(), 0),
    };
    let (record, _) = encode_record(generation, &previous, payload);
    let mut file: File = OpenOptions::new().create(true).write(true).truncate(false).open(&path)?;
    if keep < existing.len() {
        file.set_len(keep as u64)?;
    }
    io::Seek::seek(&mut file, io::SeekFrom::Start(keep as u64))?;
    file.write_all(&record)?;
    return Ok(());
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
    let next: u64 = rotation_number(dir).map_or(1, |number| { return number + 1; });
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
    let all: Vec<u64> = numbers(dir);
    let Some(newest) = all.last().copied() else { return Ok(()) };
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

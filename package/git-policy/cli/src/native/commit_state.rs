//! What:
//!  The lane's state document as a double-buffered pair of files, `state.a` and `state.b`, each with its generation and a checksum
//!  over its whole content; a write goes to the file that does not hold the newest valid generation, in place, with no sync.
//! Why:
//!  Rule V (revision 12, `doc/planning/cli-git-commits-never-fail.md`): a transition's records are visible (they survive the death of
//!  any process) without a sync, and a power loss that lands the new file only in part must leave the older generation valid. A
//!  rename over the only copy would need a sync before it to be safe; the pair does not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writeFileSync(olderOf(['state.a', 'state.b']), header(g) + payload + checksum); // no fsync
//! ```

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::commit_identity::sync_file;
use super::commit_integrity::sha256_hex;

/// The two file names of the pair.
const NAMES: [&str; 2] = ["state.a", "state.b"];

/// The bytes of a state file: `cli-git-state 1 <generation>\n`, the payload, then `\nend <sha256 of everything before>\n`.
fn encode(generation: u64, payload: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = format!("cli-git-state 1 {}\n", generation).into_bytes();
    bytes.extend_from_slice(payload);
    let checksum: String = sha256_hex(&bytes);
    bytes.extend_from_slice(format!("\nend {}\n", checksum).as_bytes());
    return bytes;
}

/// The generation and payload a state file's bytes hold, or `None` when it is torn, damaged or not a state file.
fn decode(bytes: &[u8]) -> Option<(u64, Vec<u8>)> {
    const TRAILER: usize = 5 + 64 + 1;
    if bytes.len() < TRAILER {
        return None;
    }
    let (body, trailer) = bytes.split_at(bytes.len() - TRAILER);
    if trailer != format!("\nend {}\n", sha256_hex(body)).as_bytes() {
        return None;
    }
    let newline: usize = body.iter().position(|byte| { return *byte == b'\n'; })?;
    let header: &str = std::str::from_utf8(&body[..newline]).ok()?;
    let generation: u64 = header.strip_prefix("cli-git-state 1 ")?.parse().ok()?;
    return Some((generation, body[newline + 1..].to_vec()));
}

/// Both files of the pair as `(path, valid content)`.
fn read_pair(dir: &Path) -> Vec<(PathBuf, Option<(u64, Vec<u8>)>)> {
    return NAMES.iter().map(|name| {
        let path: PathBuf = dir.join(name);
        let content: Option<(u64, Vec<u8>)> = fs::read(&path).ok().and_then(|bytes| { return decode(&bytes); });
        return (path, content);
    }).collect();
}

/// The path and content of the newest valid file of the pair, if any.
fn newest(dir: &Path) -> Option<(PathBuf, u64, Vec<u8>)> {
    let mut best: Option<(PathBuf, u64, Vec<u8>)> = None;
    for (path, content) in read_pair(dir) {
        if let Some((generation, payload)) = content
            && best.as_ref().is_none_or(|(_, newest_generation, _)| { return generation > *newest_generation; })
        {
            best = Some((path, generation, payload));
        }
    }
    return best;
}

/// The newest valid generation and its payload, or `None` when neither file is valid.
pub fn read_state(dir: &Path) -> Option<(u64, Vec<u8>)> {
    return newest(dir).map(|(_, generation, payload)| { return (generation, payload); });
}

/// What:
///  Writes `payload` as `generation` into the file of the pair that does not hold the newest valid generation (`state.a` when neither
///  is valid), in place and without a sync; the newest valid file is never touched.
/// Why:
///  A transition's write costs no wait; a power loss in the middle of it leaves the other file, the older generation, valid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeState(dir: string, generation: number, payload: Uint8Array): void;
/// ```
pub fn write_state(dir: &Path, generation: u64, payload: &[u8]) -> io::Result<()> {
    let target: PathBuf = match newest(dir) {
        Some((path, _, _)) if path == dir.join(NAMES[0]) => dir.join(NAMES[1]),
        _ => dir.join(NAMES[0]),
    };
    let mut file: File = OpenOptions::new().create(true).write(true).truncate(true).open(&target)?;
    file.write_all(&encode(generation, payload))?;
    return Ok(());
}

/// Syncs the file that holds the newest valid generation (a departure barrier's step); an error when neither file is valid.
pub fn sync_newest(dir: &Path) -> io::Result<()> {
    let (path, _, _) = newest(dir).ok_or_else(|| { return io::Error::new(io::ErrorKind::NotFound, "no valid state file"); })?;
    let file: File = File::open(&path)?;
    return sync_file(&file, &path);
}

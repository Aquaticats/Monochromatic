//! What:
//!  The history of `monochromatic-git-policy-cli-audit.jsonl`:
//! where a rotated file goes,
//! which generations may be removed,
//! and the pass that
//!  removes them.
//! Why:
//!  The audit lines name the saved copies of conflicted files,
//! and with a single `.1` the second rotation dropped the lines
//!  that named copies still on disk.
//! Each rotation now keeps its file under a new number;
//! a pass after the rotation removes the
//!  oldest generations that name no copy still on disk until those fit the bound `AUDIT_HISTORY_BYTES`;
//! a generation that
//!  names a copy on disk is never removed (the delegated pick 11,
//! option C,
//! in
//!  `doc/planning/cli-git-shipped-limits-audit.md`).
//! No lock is taken:
//! a number is claimed with a hard link,
//! which fails when
//!  the name exists,
//! so two writers cannot take one number,
//! and a removal that another pass did first is not an error.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = await rename(target, privateName);
//! await installGeneration(taken, target); // link to `${target}.${n}`, n + 1 on EEXIST, then unlink the private name
//! await trimGenerations(target, AUDIT_HISTORY_BYTES);
//! ```

use super::AUDIT_FILE;
use super::TAG;
use crate::diagnostic_log::debug;
use std::ffi::OsStr;
use std::io::BufRead;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// What:
///  How many rotated files a log keeps.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The audit file explains saved copies and keeps a history;
/// the timing logs and the push trigger log are diagnostics that
///  the next rotation may replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Retention = 'oneGeneration' | 'generations';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Retention {
    /// The rotated file is `<name>.1`,
    /// and the next rotation replaces it.
    OneGeneration,
    /// The rotated file gets the next free number,
    /// and the pass in `trim_generations` decides what may go.
    Generations,
}

/// The retention of the log at `target`:
/// the audit file keeps generations,
/// every other file one.
pub(super) fn retention_of(target: &Path) -> Retention {
    if target.file_name() == Some(OsStr::new(AUDIT_FILE)) {
        return Retention::Generations;
    }
    return Retention::OneGeneration;
}

/// The path of generation `number` of the log at `target`:
/// the name with `.` and the number after it.
fn numbered(target: &Path, number: u64) -> PathBuf {
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".{number}"));
    return PathBuf::from(name);
}

/// What:
///  A regular file of the log's directory that belongs to the log at `target`.
///  A `struct` is a record with named fields.
/// Why:
///   The pass needs each file's number,
/// size and identity together,
/// and a struct keeps the listing and the choice
///  of what to remove readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Piece = { path: string; number: number | undefined; length: number; identity: [bigint, bigint] };
/// ```
struct Piece {
    /// The path.
    path: PathBuf,
    /// The generation number,
    /// or nothing for a file at a private name.
    number: Option<u64>,
    /// Bytes.
    length: u64,
    /// The device and inode,
    /// which say that two names are one file.
    identity: (u64, u64),
}

/// What:
///  The regular files of the directory of `target` named `<target>.<digits>` (generations) or `<target>.moving-...` (files a
///  rotation took off the name and has not placed).
/// Why:
///   One read of the directory serves both the choice of the next number and the pass that trims.
/// A directory,
/// a link or
///  anything else with such a name is not a log file and is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pieces = (await readdir(dirname(target))).filter((name) => /^audit\.(\d+|moving-.*)$/.test(name));
/// ```
fn list_pieces(target: &Path) -> std::io::Result<Vec<Piece>> {
    let directory: &Path = target.parent().unwrap_or(Path::new("."));
    let stem: String = match target.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let mut pieces: Vec<Piece> = Vec::new();
    for listed in std::fs::read_dir(directory)? {
        let entry: std::fs::DirEntry = listed?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(after_stem) = name.strip_prefix(stem.as_str()) else { continue };
        let Some(rest) = after_stem.strip_prefix('.') else { continue };
        let number: Option<u64> = if !rest.is_empty() && rest.as_bytes().iter().all(u8::is_ascii_digit) { rest.parse::<u64>().ok() } else { None };
        if number.is_none() && !rest.starts_with("moving-") {
            continue;
        }
        // A file that vanished since the listing was another pass's removal.
        let Ok(found) = std::fs::symlink_metadata(entry.path()) else { continue };
        if !found.is_file() {
            continue;
        }
        pieces.push(Piece { path: entry.path(), number, length: found.len(), identity: (found.dev(), found.ino()) });
    }
    return Ok(pieces);
}

/// What:
///  Gives the file a rotation took off `target` (now at `taken`) the next free generation number,
/// and removes the private name.
/// Why:
///   A hard link fails when the name exists,
/// so each writer claims a number nobody holds and never replaces another's
///  generation;
/// on a failure each tries the number after.
/// A failure means another writer's success,
/// so the loop ends.
/// A kill
///  between the link and the removal leaves two names for one file,
/// which the next `trim_generations` pass removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let n = highest + 1; ; n++) { try { await link(taken, `${target}.${n}`); break; } catch (e) { if (e.code !== 'EEXIST') throw e; } }
/// await unlink(taken);
/// ```
///
/// # Errors
/// The system's error when the directory cannot be read or a link cannot be made for a reason other than a taken name.
pub(super) fn install_generation(taken: &Path, target: &Path) -> std::io::Result<()> {
    let mut highest: u64 = 0;
    for piece in list_pieces(target)? {
        if let Some(number) = piece.number {
            highest = highest.max(number);
        }
    }
    let mut number: u64 = highest.saturating_add(1);
    loop {
        match std::fs::hard_link(taken, numbered(target, number)) {
            Ok(()) => break,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => number = number.saturating_add(1),
            Err(error) => return Err(error),
        }
    }
    if let Err(error) = std::fs::remove_file(taken) {
        debug(TAG, &format!("generation {number} of {} is in place but its private name {} stays: {error}", target.display(), taken.display()));
    }
    return Ok(());
}

/// The words a saved-conflict line carries,
/// which no other line has.
const SAVED_CONFLICT_MARK: &[u8] = b"\"outcome\":\"saved-conflict\"";

/// The characters that stand for a byte the line's writer could not show as text:
/// a place that has one cannot be looked up.
const REPLACEMENT: char = '\u{FFFD}';

/// The generation number of a piece.
/// `sort_by_key` takes this function by path.
fn piece_number(piece: &&Piece) -> Option<u64> {
    return piece.number;
}

/// Whether a line holds the mark of a `saved-conflict` record anywhere in it.
fn holds_saved_conflict_mark(line: &[u8]) -> bool {
    for window in line.windows(SAVED_CONFLICT_MARK.len()) {
        if window == SAVED_CONFLICT_MARK {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a `saved-conflict` line names a saved copy that exists,
/// or might.
/// Why:
///   The line lists `<path> => <place>`;
/// the path is the person's and may hold ` => `,
/// so every text after such a separator is
///  tried as a place.
/// A line the reader cannot parse,
/// and a place the writer could not show as text,
/// count as naming a copy:
///  the generation is kept,
/// because a wrong answer in that direction costs disk and the other direction costs the only
///  explanation of a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// line.copies.some((c) => c.split(' => ').slice(1).some((_, i) => existsSync(c.split(' => ').slice(i + 1).join(' => '))))
/// ```
fn line_names_a_copy(line: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(line) else {
        return true;
    };
    let Some(copies) = value.get("copies").and_then(serde_json::Value::as_array) else {
        return false;
    };
    for entry in copies {
        let Some(copy) = entry.as_str() else {
            continue;
        };
        for (index, _) in copy.match_indices(" => ") {
            let place: &str = &copy[index + " => ".len()..];
            if place == "none" {
                continue;
            }
            if place.contains(REPLACEMENT) || std::fs::symlink_metadata(place).is_ok() {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  Whether a generation holds a `saved-conflict` line that names a copy still on disk,
/// read in one pass over the file.
/// Why:
///   Only that line says where a copy went;
/// a pruned directory's line records a removal and explains nothing that is still
///  there.
/// One line at a time is held,
/// so the memory is the longest line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const line of readLines(path)) if (line.includes('"outcome":"saved-conflict"') && namesACopy(line)) return true;
/// return false;
/// ```
///
/// # Errors
/// The system's error when the file cannot be opened or read.
fn names_a_copy_on_disk(path: &Path) -> std::io::Result<bool> {
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut line: Vec<u8> = Vec::new();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            return Ok(false);
        }
        if holds_saved_conflict_mark(line.as_slice()) && line_names_a_copy(line.as_slice()) {
            return Ok(true);
        }
    }
}

/// What:
///  Removes the oldest generations that name no copy on disk until the others fit `budget` bytes,
/// and the second name of any
///  file that has a number.
/// Why:
///   The budget counts what explains nothing on disk:
/// generations without a copy,
/// and files at a private name (a killed
///  rotation's,
/// which are never removed because their lines are in no other place).
/// A generation that names a copy on disk,
/// or
///  cannot be read to see,
/// is outside the budget and stays.
/// A removal another pass did first is the same result,
/// and the pass
///  never waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const g of oldestFirst(generations)) { if (total <= budget) break; if (!(await namesACopyOnDisk(g))) { await unlink(g); total -= g.size; } }
/// ```
///
/// # Errors
/// The system's error when the directory cannot be read.
pub(super) fn trim_generations(target: &Path, budget: u64) -> std::io::Result<()> {
    let pieces: Vec<Piece> = list_pieces(target)?;
    let mut generations: Vec<&Piece> = Vec::new();
    for piece in &pieces {
        if piece.number.is_some() {
            generations.push(piece);
        }
    }
    generations.sort_by_key(piece_number);
    let mut total: u64 = 0;
    for piece in &pieces {
        if piece.number.is_some() {
            continue;
        }
        let mut numbered_too: bool = false;
        for generation in &generations {
            if generation.identity == piece.identity {
                numbered_too = true;
                break;
            }
        }
        if numbered_too {
            // The same file under its generation number: a rotation was killed between its link and its removal.
            if let Err(error) = std::fs::remove_file(&piece.path) {
                debug(TAG, &format!("the second name {} of a generation could not be removed: {error}", piece.path.display()));
            }
        } else {
            total = total.saturating_add(piece.length);
        }
    }
    let mut candidates: Vec<&Piece> = Vec::new();
    for generation in generations {
        match names_a_copy_on_disk(generation.path.as_path()) {
            Ok(false) => {
                total = total.saturating_add(generation.length);
                candidates.push(generation);
            }
            Ok(true) => {}
            Err(error) => debug(TAG, &format!("{} was kept: it could not be read to see which copies it names: {error}", generation.path.display())),
        }
    }
    for candidate in candidates {
        if total <= budget {
            break;
        }
        match std::fs::remove_file(&candidate.path) {
            Ok(()) => total = total.saturating_sub(candidate.length),
            // Another pass removed it first.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => total = total.saturating_sub(candidate.length),
            Err(error) => debug(TAG, &format!("{} was kept: it could not be removed: {error}", candidate.path.display())),
        }
    }
    return Ok(());
}

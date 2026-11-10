//! What:
//!  The history of each log of the wrapper:
//! where a rotated file goes,
//! which generations may be removed,
//! and the pass that
//!  removes them.
//! Why:
//!  Every log keeps numbered generations,
//! so a reader of a log reads its history and not the few lines that came
//!  after the last rotation.
//! The owner answered on 2026-11-10 (the question tool,
//! "Keep more,
//! capped (Recommended)") that each log keeps up
//!  to `LOG_HISTORY_BYTES`,
//! and that an audit line naming a saved copy that still exists is never dropped.
//! A log that pins its
//!  generations says so through `LogRetention::pins`,
//! and its pinned generations stay past the bound
//!  (`discard_audit_pins.rs`).
//! No lock is taken:
//! a number is claimed with a hard link,
//! which fails when
//!  the name exists,
//! so two writers cannot take one number,
//! and a removal that another pass did first is not an error.
//! A file a
//!  killed rotation left at its private name is given the next number by the same pass,
//! because readers go by number.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = await rename(target, privateName);
//! await installGeneration(taken, target); // link to `${target}.${n}`, n + 1 on EEXIST, then unlink the private name
//! await trimGenerations(target, retention);
//! ```

use crate::diagnostic_log::debug;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "logGenerations";

/// The bytes every log's generations may take together,
/// the live file left aside.
///
/// The owner's answer of 2026-11-10 in the question tool ("Keep more,
/// capped (Recommended)") set it at 32 MiB per log,
/// with an audit generation that names a saved copy still on disk kept past it.
/// Measured on this host on 2026-11-10T14:31Z:
/// the commit timing log held 1,143,526 bytes in
/// 229 records from 2026-11-10T09:08:07Z to 2026-11-10T14:31:41Z,
/// about 0.21 MiB an hour,
/// so 32 MiB
/// is about six days of that log at the pace of the day the answer was given.
/// The audit log held 782,685 bytes in 1,619 lines
/// from 2026-10-21 to 2026-11-10,
/// about 42 KB a day,
/// so the bound is the whole history of an ordinary repository and only a
///  repository that discards work continuously reaches it.
/// The measurement and the earlier four mebibytes are in
/// `doc/planning/cli-git-shipped-limits-audit.md`,
/// "The audit and timing logs' history".
pub const LOG_HISTORY_BYTES: u64 = 32 * 1024 * 1024;

/// What a log keeps when a rotation makes a generation.
///  A `struct` is a record with named fields.
/// Why:
///   Every log is bounded by the same owner-chosen total;
/// only the audit log's generations can explain a
///  saved copy that still exists,
/// so only its `pins` reads a generation before dropping it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LogRetention = { budget: number; pins: (path: string) => boolean };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LogRetention {
    /// The bytes the generations may take together.
    pub budget: u64,
    /// Why one generation must stay,
    /// whatever its size:
    /// `Ok(true)` keeps it,
    /// `Ok(false)` lets it go,
    /// and an error keeps it,
    /// because a wrong answer in that direction costs disk and the
    ///  other costs what a reader needs.
    pub pins: fn(&Path) -> std::io::Result<bool>,
}

/// A log whose generations explain nothing beyond their own lines.
pub const UNPINNED_LOG: LogRetention = LogRetention { budget: LOG_HISTORY_BYTES, pins: pins_nothing };

/// The `pins` of a log whose generations name no saved copy.
fn pins_nothing(_generation: &Path) -> std::io::Result<bool> {
    return Ok(false);
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
/// for (let n = highest + 1; ; n++) { try { await link(taken, `${target}.${n}`); break; } catch (e) { if (e.code !== 'EEXIST' || !isFile(`${target}.${n}`)) throw e; } }
/// await unlink(taken);
/// ```
/// Those words end the numbering: a name that exists but is not a regular file (a
///  directory a person left there, a socket) will never become free,
/// so the rotation reports it instead of counting up for
///  ever.
///
/// # Errors
/// The system's error when the directory cannot be read or a link cannot be made for a reason other than a taken name.
pub fn install_generation(taken: &Path, target: &Path) -> std::io::Result<()> {
    let mut highest: u64 = 0;
    for piece in list_pieces(target)? {
        if let Some(number) = piece.number {
            highest = highest.max(number);
        }
    }
    let mut number: u64 = highest.saturating_add(1);
    loop {
        let candidate: PathBuf = numbered(target, number);
        match std::fs::hard_link(taken, &candidate) {
            Ok(()) => break,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && std::fs::symlink_metadata(&candidate).is_ok_and(|found| { return found.is_file(); }) => number = number.saturating_add(1),
            Err(error) => return Err(error),
        }
    }
    if let Err(error) = std::fs::remove_file(taken) {
        debug(TAG, &format!("generation {number} of {} is in place but its private name {} stays: {error}", target.display(), taken.display()));
    }
    return Ok(());
}

/// The generation number of a piece.
/// `sort_by_key` takes this function by path.
fn piece_number(piece: &&Piece) -> Option<u64> {
    return piece.number;
}

/// What:
///  Removes the second name of any file that has two,
/// gives a file left at a private name by a killed rotation the next
///  generation number,
/// and removes the oldest generations until the others fit the log's budget.
/// Why:
///   Readers go by number,
/// so a file that stays at a private name is read by nobody:
/// its lines are on disk and in no
///  reader's sight.
/// The pass gives it a number by the hard link that gives a rotation its own,
/// so two passes cannot take one
///  number.
/// A writer that is not killed but between its take and its install finds its private name gone and reports an error
///  that costs nothing:
/// the line it was appending went to the file it held,
/// which is now the generation.
/// If two passes did
///  give one file two numbers,
/// the higher name goes,
/// which loses no line.
/// The oldest goes first,
/// and a generation the log's `pins` keeps is
///  skipped:
/// such a generation stays past the budget,
/// because its lines explain something still on disk.
/// A generation is read only when
///  the budget is already exceeded and only until the others fit,
/// so a log under its budget costs one listing and no read
///  at all.
/// A removal another pass did first is the same result,
/// and the pass never waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const second of secondNames(pieces)) await unlink(second);
/// for (const orphan of orphans(pieces)) await installGeneration(orphan, target);
/// for (const g of oldestFirst(generations)) { if (total <= budget) break; if (!retention.pins(g)) { await unlink(g); total -= g.size; } }
/// ```
///
/// # Errors
/// The system's error when the directory cannot be read.
pub fn trim_generations(target: &Path, retention: &LogRetention) -> std::io::Result<()> {
    let mut pieces: Vec<Piece> = list_pieces(target)?;
    if settle_names(target, pieces.as_slice()) {
        pieces = list_pieces(target)?;
    }
    let mut generations: Vec<&Piece> = Vec::new();
    let mut total: u64 = 0;
    for piece in &pieces {
        // A file that could not be given a number stays where it is, and its bytes count against the bound like a generation's.
        total = total.saturating_add(piece.length);
        if piece.number.is_some() {
            generations.push(piece);
        }
    }
    if total > retention.budget {
        drop_oldest_generations(generations.as_mut_slice(), total, retention);
    }
    return Ok(());
}

/// What:
///  Removes the oldest generations of `generations` until their bytes fit `retention.budget`,
/// reading a generation only
///  to ask whether the log's `pins` keeps it,
/// and never the newest,
/// the file the rotation that asked for the pass has just made.
/// Why:
///   A log under its budget needs no read at all,
/// and the read of a generation that pins costs disk once,
///  where every generation read on every rotation would cost the whole history each time.
/// The sort is by number,
/// which is the order the rotations
///  happened in.
/// Removing the newest would mean a log whose oldest generations all pin loses each new generation as it is made:
/// the
///  newest history would be the first to go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// generations.sort((a, b) => a.number - b.number); for (const g of generations.slice(0, -1)) { if (total <= budget) return; if (retention.pins(g.path)) continue; await unlink(g.path); total -= g.length; }
/// ```
fn drop_oldest_generations(generations: &mut [&Piece], mut total: u64, retention: &LogRetention) {
    generations.sort_by_key(piece_number);
    let removable: usize = generations.len().saturating_sub(1);
    for generation in generations.iter().take(removable) {
        if total <= retention.budget {
            return;
        }
        match (retention.pins)(generation.path.as_path()) {
            Ok(false) => {}
            Ok(true) => continue,
            Err(error) => {
                debug(TAG, &format!("{} was kept: it could not be read to see which copies it names: {error}", generation.path.display()));
                continue;
            }
        }
        match std::fs::remove_file(&generation.path) {
            Ok(()) => total = total.saturating_sub(generation.length),
            // Another pass removed it first.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => total = total.saturating_sub(generation.length),
            Err(error) => debug(TAG, &format!("{} was kept: it could not be removed: {error}", generation.path.display())),
        }
    }
}

/// What:
///  Whether `pieces` holds a file that `file` is a second name of:
/// a numbered piece with the same device and inode,
/// which a
///  lower number or a numbered name already stands for.
/// Why:
///   A second name keeps the file's bytes and says nothing new,
/// so it may go;
/// the first name,
/// the lowest number,
/// stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSecondName = (file, pieces) => pieces.some((p) => sameFile(p, file) && p.number !== undefined && (file.number === undefined || p.number < file.number));
/// ```
fn is_second_name(file: &Piece, pieces: &[Piece]) -> bool {
    for other in pieces {
        let Some(other_number) = other.number else { continue };
        if other.identity != file.identity {
            continue;
        }
        match file.number {
            None => return true,
            Some(number) if other_number < number => return true,
            Some(_) => {}
        }
    }
    return false;
}

/// What:
///  Removes the second names of files and gives each file left at a private name its generation number;
/// whether the listing
///  changed.
/// Why:
///   The two repairs of a killed rotation.
/// A kill between the link and the removal leaves a private name beside the number;
/// a
///  kill after the take leaves a private name alone.
/// Either way the work is one hard link or one removal,
/// and a failure is a
///  debug line,
/// never an error:
/// the file stays and the next pass tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const p of pieces) if (isSecondName(p)) await unlink(p.path); else if (p.number === undefined) await installGeneration(p.path, target);
/// ```
fn settle_names(target: &Path, pieces: &[Piece]) -> bool {
    let mut changed: bool = false;
    for piece in pieces {
        if is_second_name(piece, pieces) {
            // The same file under its generation number: a rotation was killed between its link and its removal, or two passes
            // gave one file two numbers.
            match std::fs::remove_file(&piece.path) {
                Ok(()) => changed = true,
                Err(error) => debug(TAG, &format!("the second name {} of a generation could not be removed: {error}", piece.path.display())),
            }
        } else if piece.number.is_none() {
            // A file a rotation took off the name and never placed: its lines are in no reader's sight until it has a number.
            match install_generation(piece.path.as_path(), target) {
                Ok(()) => changed = true,
                Err(error) => debug(TAG, &format!("{} could not be given a generation number: {error}", piece.path.display())),
            }
        }
    }
    return changed;
}

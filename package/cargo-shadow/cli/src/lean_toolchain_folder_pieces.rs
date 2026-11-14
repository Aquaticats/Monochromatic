//! What:
//! The removal of one toolchain folder,
//! one piece at a time:
//! the drive it is measured on,
//! the claim that takes a folder out of the place a build is given,
//! and what a stopped removal leaves for the next pass.
//! Why:
//! A removal of a folder of many shards on this machine took 19 minutes and 27 seconds and met a drive whose readings
//! ran to fifteen seconds
//! (2026-11-13,
//! `keep/delete-the-build-folders-of-two-old-toolchains-shard-by-shard-and-stop-on-a-slow-drive.ts` in that day's run
//! folder),
//! so a pass removes one piece,
//! measures the drive,
//! and stops when the drive says to.
//! Everything here is one folder's business:
//! which folder is chosen and when is `lean_toolchain_folders.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function removePieces(entry: string, policy: PiecePolicy, volume: Volume, probe: string, deadline: Deadline, dryRun: boolean, stopUnderBound: boolean): Pieces
//! ```

/// Files,
/// paths and the length of a call.
use std::{fs, io, path::{Path, PathBuf}, process::Command, time::{Duration, Instant}};

/// One bounded child run,
/// for asking for the idle disk class.
use crate::bounded_run::{Bound, RunError, output_within};
/// The deadline a removal obeys.
use crate::lean_identity::Deadline;
/// The one way these tools remove an entry.
use crate::lean_removal::{Refusal, delete_claimed, item_of, modes_named};
/// The reading of the volume's free space the rest of a lean run already takes,
/// and the lines a removal writes.
use crate::lean_run_report::{free_bytes, gib};
/// The final component of a path as text.
use crate::lean_text::name_of;

/// The free space and the two drive bounds a removal of pieces obeys.
#[derive(Clone, Copy, Debug)]
pub struct PiecePolicy {
    /// The volume that holds the folders keeps this much free:
    /// a removal of a folder's pieces stops once the volume has it back,
    /// when the caller lets it stop there.
    pub free_space_bound_bytes: u64,
    /// The longest one pass may spend removing folders.
    pub pass_limit: Duration,
    /// One drive reading over this is a slow drive.
    pub slow_reading: Duration,
    /// This many slow readings in a row stop the pass.
    pub slow_readings_in_a_row: u32,
}

/// The directory under the base that holds the folders a removal started on.
/// Why:
/// A folder a pass was stopped in the middle of is recognisable as being removed by where it is,
/// the place the shadow hands a build no longer names it,
/// and a later pass finishes it from here.
pub const RETIRING_DIRECTORY_NAME: &str = ".cargo-shadow-retiring";

/// The file a drive reading is written to and synced,
/// at the base's own top level.
/// Why:
/// A reading has to reach the drive that holds the folders,
/// and the base's top level is on that drive while it holds no tree.
pub const READING_FILE_NAME: &str = ".cargo-shadow-drive-reading";

/// How many bytes one drive reading writes before its barrier.
/// Why:
/// The barrier is the measurement and not the size:
/// the bytes exist so the write cannot be answered out of the page cache alone,
/// and no figure here is read from their count.
const READING_BYTES: usize = 1 << 20;

/// How long one `ionice` call may take.
/// Why:
/// It sets a priority and reads nothing,
/// so a slow one is dropped rather than waited for.
const IDLE_CLASS_LIMIT: Duration = Duration::from_secs(5);


/// What a reading of the drive is,
/// so a test can give the pass a drive of its own.
/// Why:
/// The free space of the volume and the time a write and its barrier take are the only two facts the pass reads from
/// the machine beyond the folders themselves,
/// and neither can be given to a test by the real machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Volume { freeBytes(base: string): number; reading(probe: string): number }
/// ```
pub trait Volume {
    /// The free bytes an unprivileged user can still write on the volume that holds this path.
    fn free_bytes(&self, base: &Path) -> io::Result<u64>;
    /// One write-and-sync reading on the volume that holds this file.
    fn reading(&self, probe: &Path) -> io::Result<Duration>;
}

/// The volume of the machine the pass runs on.
pub struct RealVolume;

/// What:
/// `impl Volume for RealVolume` measures the real volume.
/// Why:
/// The free space is the one `statvfs` the rest of a lean run already takes,
/// and the reading is a write of `READING_BYTES` to the base's own top level followed by its barrier,
/// which is the reading the hand deletion of 2026-11-13 took before each shard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const realVolume: Volume = { freeBytes: (base) => statvfsSync(base).bavail * statvfsSync(base).frsize, reading: (probe) => { const start = performance.now(); const fd = openSync(probe, 'w'); writeSync(fd, Buffer.alloc(READING_BYTES)); fsyncSync(fd); closeSync(fd); return performance.now() - start; } };
/// ```
impl Volume for RealVolume {
    /// The free space of the volume.
    fn free_bytes(&self, base: &Path) -> io::Result<u64> {
        return match free_bytes(base) {
            Some(found) => Ok(found),
            None => Err(io::Error::other(format!("statvfs of {} gave no answer", base.display()))),
        };
    }

    /// One write and one barrier on the drive.
    fn reading(&self, probe: &Path) -> io::Result<Duration> {
        /// Writes the reading's bytes.
        use std::io::Write;
        let started: Instant = Instant::now();
        let mut file: fs::File = fs::File::create(probe)?;
        file.write_all(&vec![0_u8; READING_BYTES])?;
        file.sync_all()?;
        return Ok(started.elapsed());
    }
}

/// What stopped the removal of one folder's pieces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stop {
    /// Every piece is gone.
    Finished,
    /// The volume has its free space back.
    UnderBound,
    /// The drive answered slowly as often in a row as the bound allows.
    SlowDrive,
    /// The drive could not be read,
    /// so the pass cannot tell whether it is slow and stops.
    ReadingFailed,
    /// The pass's own time bound is gone.
    Deadline,
    /// The pass was told to stop after a piece,
    /// by one of its own tests.
    Interrupted,
}

/// What the removal of one folder's pieces did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pieces {
    /// How many pieces the folder or the earlier pass's claim holds.
    pub total: usize,
    /// How many of them went.
    pub gone: usize,
    /// The disk bytes that went.
    pub bytes: u64,
    /// Why the removal of pieces stopped.
    pub stopped: Option<Stop>,
    /// One reason for each part that stayed after a piece was not removed whole.
    pub notes: Vec<String>,
    /// The directories whose owner access a removal widened,
    /// so a line says a mode was changed.
    pub modes: Vec<PathBuf>,
}


/// What:
/// `pub fn retiring_directory(base: &Path) -> PathBuf` is where a started removal waits for the pass that finishes it.
/// Why:
/// The tree pass's own trash and this place are apart:
/// the tree pass deletes what it finds in its trash in one walk,
/// and a folder of a toolchain goes piece by piece.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const retiringDirectory = (base: string): string => join(base, RETIRING_DIRECTORY_NAME);
/// ```
pub fn retiring_directory(base: &Path) -> PathBuf {
    return base.join(RETIRING_DIRECTORY_NAME);
}

/// What:
/// `pub fn folder_name_of(claimed: &Path) -> String` is the toolchain folder a claim in the retiring directory was made from.
/// Why:
/// A claim's name is the process,
/// the clock and a serial before the folder's own name,
/// and the line a later pass reports names the folder it finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const folderNameOf = (claimed: string): string => basename(claimed).split('-').slice(3).join('-');
/// ```
pub fn folder_name_of(claimed: &Path) -> String {
    let name: String = name_of(claimed);
    let mut parts = name.splitn(4, '-');
    for _ in 0..3 {
        if parts.next().is_none() {
            return name.clone();
        }
    }
    return match parts.next() {
        Some(rest) if !rest.is_empty() => rest.to_string(),
        _ => name,
    };
}

/// What:
/// `pub fn children_of(directory: &Path) -> Vec<PathBuf>` lists the entries of a directory in one order.
/// Why:
/// A folder's pieces are its shards,
/// and a pass stopped between two of them continues in the same order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const childrenOf = (directory: string): string[] => readdirSync(directory).sort();
/// ```
pub fn children_of(directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            found.push(entry.path());
        }
    }
    found.sort();
    return found;
}


/// What:
/// `fn slow_reading(policy: &PiecePolicy, volume: &dyn Volume, probe: &Path, slow: &mut u32) -> Option<Stop>` takes one drive
/// reading and says whether the pass must stop.
/// Why:
/// A drive that answers slowly is the machine telling the pass to wait,
/// so readings over the bound in a row end it and one alone does not:
/// the hand deletion of 2026-11-13 took six readings over five seconds out of 512 and never two in a row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function slowReading(policy: PiecePolicy, volume: Volume, probe: string, slow: {count: number}): Stop | undefined
/// ```
fn slow_reading(policy: &PiecePolicy, volume: &dyn Volume, probe: &Path, slow: &mut u32) -> Option<Stop> {
    let reading: Duration = match volume.reading(probe) {
        Ok(found) => found,
        Err(_error) => return Some(Stop::ReadingFailed),
    };
    if reading <= policy.slow_reading {
        *slow = 0;
        return None;
    }
    *slow += 1;
    if *slow >= policy.slow_readings_in_a_row {
        return Some(Stop::SlowDrive);
    }
    return None;
}

/// What:
/// `pub(crate) fn remove_pieces(entry: &Path, policy: &PiecePolicy, volume: &dyn Volume, probe: &Path, deadline: &Deadline, dry_run: bool, stop_under_bound: bool) -> Pieces`
/// removes the entries of one folder one at a time,
/// with a drive reading before the first and after each,
/// and stops at a slow drive or at the deadline,
/// and at the volume having its free space back when the removal may stop there.
/// Why:
/// A shard is the piece:
/// the hand deletion of two toolchain folders on 2026-11-13 removed 512 shards in 19 minutes and 27 seconds,
/// about 2.3 s each,
/// so a pass that stops between two loses nothing and a pass that is killed keeps what it finished.
/// A folder an earlier pass claimed is finished whole:
/// the volume having its space back does not leave half a folder behind,
/// because nothing builds in what is being removed and the pieces that stay keep holding their blocks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removePieces(entry: string, policy: PiecePolicy, volume: Volume, probe: string, deadline: Deadline, dryRun: boolean, stopUnderBound: boolean): Pieces
/// ```
pub(crate) fn remove_pieces(entry: &Path, policy: &PiecePolicy, volume: &dyn Volume, probe: &Path, deadline: &Deadline, dry_run: bool, stop_under_bound: bool) -> Pieces {
    let pieces: Vec<PathBuf> = children_of(entry);
    let mut removed: Pieces = Pieces { total: pieces.len(), ..Pieces::default() };
    let mut slow: u32 = 0;
    for (index, piece) in pieces.iter().enumerate() {
        if deadline.passed() {
            removed.stopped = Some(Stop::Deadline);
            return removed;
        }
        if index == 0
            && let Some(stopped) = slow_reading(policy, volume, probe, &mut slow)
        {
            removed.stopped = Some(stopped);
            return removed;
        }
        if !dry_run {
            let item = match item_of(piece) {
                Ok(found) => found,
                Err(error) => {
                    removed.notes.push(format!("{} could not be read: {error}", piece.display()));
                    continue;
                }
            };
            match delete_claimed(piece, item.identity) {
                Ok(deleted) => {
                    removed.bytes += deleted.bytes;
                    removed.modes.extend(deleted.modes_added);
                }
                Err(Refusal::Gone(_)) => {}
                Err(Refusal::PartlyRemoved { deleted, .. }) => {
                    removed.bytes += deleted.bytes;
                    removed.modes.extend(deleted.modes_added);
                    removed.notes.extend(deleted.remaining);
                }
                Err(refusal) => {
                    removed.notes.push(refusal.to_string());
                    continue;
                }
            }
            if let Some(stopped) = slow_reading(policy, volume, probe, &mut slow) {
                removed.stopped = Some(stopped);
                return removed;
            }
        }
        removed.gone += 1;
        if stop_under_bound
            && index + 1 < pieces.len()
            && let Ok(free) = volume.free_bytes(entry)
            && free >= policy.free_space_bound_bytes
        {
            removed.stopped = Some(Stop::UnderBound);
            return removed;
        }
        if interrupted_after(index + 1) {
            removed.stopped = Some(Stop::Interrupted);
            return removed;
        }
    }
    removed.stopped = Some(Stop::Finished);
    return removed;
}

/// What:
/// `pub(crate) fn piece_line(what: &str, removed: &Pieces, entry: &Path) -> String` is the one line a removal of one folder earned.
/// Why:
/// It says what went,
/// what stayed,
/// whether a mode had to change,
/// and where the next pass continues.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pieceLine = (what: string, removed: Pieces, entry: string): string => `${what}: ${removed.gone} of ${removed.total} piece(s) gone (${gib(removed.bytes)} by disk blocks)${modesNamed(removed.modes)}${removed.notes.length ? `; what stayed: ${removed.notes.join('; ')}` : ''}${removed.stopped === 'Finished' ? '' : `; the next pass finishes it in ${entry}`}`;
/// ```
pub(crate) fn piece_line(what: &str, removed: &Pieces, entry: &Path) -> String {
    let mut line: String = format!("{what}: {} of {} piece(s) gone ({} by disk blocks){}", removed.gone, removed.total, gib(removed.bytes), modes_named(&removed.modes));
    if !removed.notes.is_empty() {
        line.push_str(&format!("; what stayed: {}", removed.notes.join("; ")));
    }
    if removed.stopped != Some(Stop::Finished) {
        line.push_str(&format!("; the next pass finishes it at {}", entry.display()));
    }
    return line;
}

/// What:
/// `pub(crate) fn stop_line(stopped: Option<Stop>, policy: &PiecePolicy) -> Option<String>` is the line a pass earns when it stops removing
/// before the volume has its free space back.
/// Why:
/// A stop at a slow drive,
/// at an unreadable drive or at the pass's own bound is a fact the next pass needs,
/// and the volume having its space back is not a stop worth a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopLine(stopped: Stop | undefined, policy: PiecePolicy): string | undefined
/// ```
pub(crate) fn stop_line(stopped: Option<Stop>, policy: &PiecePolicy) -> Option<String> {
    return match stopped {
        Some(Stop::SlowDrive) => Some(format!(
            "the toolchain-folder removal stopped: the drive answered over {} ms {} times in a row; the next pass continues",
            policy.slow_reading.as_millis(),
            policy.slow_readings_in_a_row
        )),
        Some(Stop::ReadingFailed) => Some(String::from("the toolchain-folder removal stopped: the drive could not be read, so the pass cannot tell whether it is slow")),
        Some(Stop::Deadline) => Some(format!("the toolchain-folder removal stopped at its own bound of {} s; the next pass continues", policy.pass_limit.as_secs())),
        Some(Stop::Interrupted) => Some(String::from("the toolchain-folder removal stopped on a fault point; the next pass continues")),
        Some(Stop::Finished) | Some(Stop::UnderBound) | None => None,
    };
}

/// What:
/// `pub(crate) fn ask_for_idle_disk_priority(own_pid: i32) -> String` puts this process in the idle disk class and answers the line that
/// says what came of it.
/// Why:
/// The removal is the only thing these tools do that writes this much to a drive a person and every other run also use,
/// and the idle class is the lowest one the kernel offers,
/// so it is asked for rather than inherited:
/// the installed service sets it too and a pass run by hand does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const askForIdleDiskPriority = (ownPid: number): string => { execFileSync('ionice', ['--class', 'idle', '--pid', String(ownPid)]); return 'the pass asked for the idle disk class (ionice --class idle) and got it'; };
/// ```
pub(crate) fn ask_for_idle_disk_priority(own_pid: i32) -> String {
    let bound: Bound = Bound { limit: IDLE_CLASS_LIMIT, what: String::from("`ionice --class idle`"), rerun: String::from("cargo-shadow lean run") };
    return match output_within(Command::new("ionice").args(["--class", "idle", "--pid"]).arg(own_pid.to_string()), None, &bound) {
        Ok(found) if found.status.success() => String::from("the pass asked for the idle disk class (ionice --class idle) and got it"),
        Ok(found) => format!("the pass asked for the idle disk class and did not get it: {}", String::from_utf8_lossy(&found.stderr).trim()),
        Err(RunError::TimedOut { what, limit, .. }) => format!("{what} did not finish within {} s, so the pass runs at the disk priority it inherited", limit.as_secs()),
        Err(other) => format!("the pass could not ask for the idle disk class ({other:?}), so it runs at the disk priority it inherited"),
    };
}

/// Whether this pass was told to stop after a piece.
/// Why:
/// The control of a removal stopped between two pieces needs the stop to be certain,
/// and a switch of its own in the test build is what the rest of this crate uses for that (`failpoints.rs`).
#[cfg(test)]
pub fn interrupted_after(pieces: usize) -> bool {
    return STOP_AFTER_PIECES.with(|slot| { return slot.get() == Some(pieces); });
}

/// A pass in the ordinary build stops after no piece.
#[cfg(not(test))]
pub fn interrupted_after(_pieces: usize) -> bool {
    return false;
}

thread_local! {
    /// The one piece count at which this thread's next pass reports being stopped,
    /// for the control of a removal killed between two pieces.
    #[cfg(test)]
    static STOP_AFTER_PIECES: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// What:
/// `pub fn arm_stop_after_pieces(pieces: usize)` makes the next pass on this thread stop after exactly that many pieces.
/// Why:
/// A removal killed between two pieces is a state the next pass has to finish,
/// and a test cannot kill a thread in the middle of a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function armStopAfterPieces(pieces: number): void
/// ```
#[cfg(test)]
pub fn arm_stop_after_pieces(pieces: usize) {
    STOP_AFTER_PIECES.with(|slot| { slot.set(Some(pieces)); });
}

/// What:
/// `pub fn disarm_stop_after_pieces()` forgets the stop a test armed.
/// Why:
/// The pass that finishes the removal must run the real loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function disarmStopAfterPieces(): void
/// ```
#[cfg(test)]
pub fn disarm_stop_after_pieces() {
    STOP_AFTER_PIECES.with(|slot| { slot.set(None); });
}

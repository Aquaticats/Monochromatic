//! What:
//! Mechanism G:
//! the removal of the build folders of toolchains nothing builds with any more,
//! as whole folders and not one tree at a time,
//! chosen by the folder's own modification time while the volume that holds the shared build directory is short of
//! free space.
//! Why:
//! A toolchain's folder is the unit in which a nightly's work becomes waste:
//! a nightly update dirties every unit of a build folder in place,
//! and a worktree that stops existing leaves its folder behind
//! (`doc/decision/cargo-build-dir-host-cache.md`).
//! Deciding about a tree costs a walk of that tree,
//! and the tree pass kept all 62,259 trees of this machine's base on every pass of 2026-11-13
//! (`journalctl --user --unit=cargo-shadow-lean.service`;
//! the probe in the run's own `keep/what-lean-decides-for-the-real-trees-2026-11-13.log`),
//! so the folder is decided from one look at the folder itself:
//! its own modification time, which the shadow sets on every build it hands to that toolchain
//! (`lean_keyed_folder.rs`),
//! and one reading of the volume's free space.
//! The volume lost about 36 GiB an hour on 2026-11-13 and a full drive ended twenty agent runs on 2026-11-12,
//! so the pass asks for the idle disk class,
//! removes one piece of a folder at a time,
//! and stops when the drive answers slowly.
//! A removal that is stopped is finished by a later pass:
//! the folder is renamed into `RETIRING_DIRECTORY_NAME` in one atomic step,
//! so the place the shadow hands a build no longer holds it and a build that meets it makes a fresh folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function retireToolchainFolders(base: string, keepToolchain: string | undefined, limits: Limits, volume: Volume, environment: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): Outcome
//! ```

/// Files,
/// paths,
/// the system clock and the length of a pass.
use std::{fs, io, os::unix::fs::MetadataExt, path::{Path, PathBuf}, process::Command, time::{Duration, Instant}};

/// One bounded child run, for asking for the idle disk class.
use crate::bounded_run::{Bound, RunError, output_within};
/// The user's build base holds the toolchain folders under this name.
use crate::lean_keyed_folder::KEYED_DIRECTORY;
/// The proof that nothing uses a folder.
use crate::lean_guard::GuardEnvironment;
/// The deadline every pass obeys and the clock ages are measured against.
use crate::lean_identity::{Deadline, Identity, now_seconds};
/// The bounds of this mechanism and what each rests on.
use crate::lean_policy::{
    DRIVE_SLOW_READINGS_IN_A_ROW,
    DRIVE_SLOW_READING_MILLIS,
    TOOLCHAIN_FOLDER_AGE_SECONDS,
    TOOLCHAIN_FOLDER_FREE_SPACE_BOUND_BYTES,
    TOOLCHAIN_FOLDER_PASS_SECONDS,
};
/// The first process that uses something under a folder.
use crate::lean_proc_scan::{ScanReport, scan_processes};
/// The one way these tools remove an entry.
use crate::lean_removal::{Refusal, claim_into_trash, delete_claimed, item_of, modes_named};
/// The reading of the volume's free space the run already has,
/// the lines of a removal and their figures.
use crate::lean_run_report::{emit, free_bytes, gib};
/// Directory entries and names.
use crate::lean_text::{is_directory, name_of};

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

/// The free space and the four bounds this mechanism ships.
#[derive(Clone, Copy, Debug)]
pub struct ToolchainFolderLimits {
    /// The volume that holds the base is under this bound in free bytes:
    /// folders are removed until it has this much free again.
    pub free_space_bound_bytes: u64,
    /// A folder whose own modification time is younger than this is never removed.
    pub age_seconds: i64,
    /// The longest one pass may spend removing folders.
    pub pass_limit: Duration,
    /// One drive reading over this is a slow drive.
    pub slow_reading: Duration,
    /// This many slow readings in a row stop the pass.
    pub slow_readings_in_a_row: u32,
}

/// The shipped bounds.
impl Default for ToolchainFolderLimits {
    /// What:
    /// `fn default() -> ToolchainFolderLimits` is what `cargo-shadow lean run` ships when nothing sets a bound.
    /// Why:
    /// Every value is one of the numbers of `lean_policy.rs`,
    /// and each can be set from the command line by whoever runs it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const DEFAULT_LIMITS: Limits = { freeSpaceBoundBytes: TOOLCHAIN_FOLDER_FREE_SPACE_BOUND_BYTES, ageSeconds: TOOLCHAIN_FOLDER_AGE_SECONDS, passLimit: TOOLCHAIN_FOLDER_PASS_SECONDS, slowReading: DRIVE_SLOW_READING_MILLIS, slowReadingsInARow: DRIVE_SLOW_READINGS_IN_A_ROW };
    /// ```
    fn default() -> ToolchainFolderLimits {
        return ToolchainFolderLimits {
            free_space_bound_bytes: TOOLCHAIN_FOLDER_FREE_SPACE_BOUND_BYTES,
            age_seconds: TOOLCHAIN_FOLDER_AGE_SECONDS,
            pass_limit: Duration::from_secs(TOOLCHAIN_FOLDER_PASS_SECONDS),
            slow_reading: Duration::from_millis(DRIVE_SLOW_READING_MILLIS),
            slow_readings_in_a_row: DRIVE_SLOW_READINGS_IN_A_ROW,
        };
    }
}

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
    /// The pass was told to stop after a piece, by one of its own tests.
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

/// A folder the pass may remove.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The folder.
    pub path: PathBuf,
    /// Its name under `toolchain/`.
    pub name: String,
    /// Its own modification time.
    pub modified: i64,
    /// Its identity when it was listed.
    pub identity: Identity,
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
/// `pub fn candidates(base: &Path) -> (Vec<Candidate>, Vec<String>)` lists the toolchain folders of the base,
/// the one modified longest ago first,
/// with one line for each folder it cannot judge.
/// Why:
/// The listing is one level deep and each folder costs one `lstat`:
/// a walk below a folder is what takes hours here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function candidates(base: string): [Candidate[], string[]]
/// ```
pub fn candidates(base: &Path) -> (Vec<Candidate>, Vec<String>) {
    let mut found: Vec<Candidate> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let Ok(entries) = fs::read_dir(base.join(KEYED_DIRECTORY)) else { return (found, notes) };
    for entry in entries.flatten() {
        if !is_directory(&entry) {
            continue;
        }
        let path: PathBuf = entry.path();
        let name: String = name_of(&path);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => found.push(Candidate { path, name, modified: metadata.mtime(), identity: Identity { device: metadata.dev(), inode: metadata.ino() } }),
            Err(error) => notes.push(format!("kept toolchain folder {}: its own time and identity could not be read: {error}", path.display())),
        }
    }
    found.sort_by(oldest_first);
    return (found, notes);
}

/// What:
/// `fn oldest_first(left: &Candidate, right: &Candidate) -> std::cmp::Ordering` orders folders by their own modification
/// time and then by name.
/// Why:
/// The folder modified longest ago goes first,
/// and two folders of one time keep one order between passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oldestFirst = (left: Candidate, right: Candidate): number => left.modified - right.modified || left.name.localeCompare(right.name);
/// ```
fn oldest_first(left: &Candidate, right: &Candidate) -> std::cmp::Ordering {
    return left.modified.cmp(&right.modified).then_with(|| { return left.name.cmp(&right.name); });
}

/// What:
/// `fn keep_reason(candidate: &Candidate, keep_toolchain: Option<&str>, limits: &ToolchainFolderLimits, now: i64, scan: Option<&ScanReport>) -> Option<String>`
/// is why the folder is kept,
/// or nothing when it may go.
/// Why:
/// Every reason is one line,
/// and a folder the pass cannot tell about is kept:
/// the order is the build this pass runs under,
/// then the folder's age,
/// then every process of the machine.
/// The names a checkout pins are deliberately not a reason:
/// 53 of this machine's worktrees still pinned `nightly-2026-11-10` on 2026-11-13 while none of them built,
/// and that pin is what kept its 1.17 TB of build folders.
/// The scan of the processes comes from the caller because it is the whole cost of a pass that finds work:
/// one scan covers every folder the pass judges,
/// so the proof costs one scan however many folders it covers
/// (measured 2026-11-13 on this machine: 0.19 s for a scan of one folder and 0.24 s of nine,
/// beside 0.02 s for a pass whose volume had its space back).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepReason(candidate: Candidate, keepToolchain: string | undefined, limits: ToolchainFolderLimits, now: number, scan: ScanReport | undefined): string | undefined
/// ```
fn keep_reason(candidate: &Candidate, keep_toolchain: Option<&str>, limits: &ToolchainFolderLimits, now: i64, scan: Option<&ScanReport>) -> Option<String> {
    if keep_toolchain == Some(candidate.name.as_str()) {
        return Some(String::from("the build this pass runs under hands its work to this toolchain, so the folder is in use"));
    }
    let changed: i64 = now - candidate.modified;
    if changed <= limits.age_seconds {
        return Some(format!("changed {changed} s ago, within the {} s age", limits.age_seconds));
    }
    let found: &ScanReport = scan?;
    if let Some(used) = found.blocks(&candidate.path) {
        return Some(format!("pid {} uses it as {}", used.pid, used.how));
    }
    return None;
}

/// What:
/// `fn scan_of(environment: &GuardEnvironment, folders: &[Candidate]) -> ScanReport` is one scan of every process for a use of
/// anything under any of these folders.
/// Why:
/// The proof of "not in use" is the same proof for every folder a pass judges,
/// so the pass pays for it once and judges every folder against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scanOf = (environment: GuardEnvironment, folders: Candidate[]): ScanReport => scanProcesses(environment.procRoot, folders.map((one) => one.path), [], environment.ownPid);
/// ```
fn scan_of(environment: &GuardEnvironment, folders: &[Candidate]) -> ScanReport {
    let mut roots: Vec<&Path> = Vec::with_capacity(folders.len());
    for folder in folders {
        roots.push(folder.path.as_path());
    }
    return scan_processes(&environment.proc_root, &roots, &[], environment.own_pid);
}

/// What:
/// `pub(crate) fn changed_since(candidate: &Candidate) -> Option<String>` says whether the place no longer holds the folder that was
/// judged,
/// and why.
/// Why:
/// A build that starts between the judgement and the claim shows itself in the folder's identity or in its time,
/// and the claim must not carry away a folder that has just been built into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changedSince = (candidate: Candidate): string | undefined => { const found = lstatSync(candidate.path); if (found.ino !== candidate.identity.inode || found.dev !== candidate.identity.device) return 'another entry is at its place now'; if (Math.floor(found.mtimeMs / 1000) !== candidate.modified) return 'a build wrote in it while the pass judged it'; return undefined; };
/// ```
pub(crate) fn changed_since(candidate: &Candidate) -> Option<String> {
    let metadata: fs::Metadata = match fs::symlink_metadata(&candidate.path) {
        Ok(found) => found,
        Err(error) => return Some(format!("its place cannot be read now: {error}")),
    };
    if metadata.dev() != candidate.identity.device || metadata.ino() != candidate.identity.inode {
        return Some(String::from("another entry is at its place now"));
    }
    if metadata.mtime() != candidate.modified {
        return Some(String::from("a build wrote in it while the pass judged it"));
    }
    return None;
}

/// What:
/// `fn slow_reading(limits: &ToolchainFolderLimits, volume: &dyn Volume, probe: &Path, slow: &mut u32) -> Option<Stop>` takes one drive
/// reading and says whether the pass must stop.
/// Why:
/// A drive that answers slowly is the machine telling the pass to wait,
/// so readings over the bound in a row end it and one alone does not:
/// the hand deletion of 2026-11-13 took six readings over five seconds out of 512 and never two in a row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function slowReading(limits: ToolchainFolderLimits, volume: Volume, probe: string, slow: {count: number}): Stop | undefined
/// ```
fn slow_reading(limits: &ToolchainFolderLimits, volume: &dyn Volume, probe: &Path, slow: &mut u32) -> Option<Stop> {
    let reading: Duration = match volume.reading(probe) {
        Ok(found) => found,
        Err(_error) => return Some(Stop::ReadingFailed),
    };
    if reading <= limits.slow_reading {
        *slow = 0;
        return None;
    }
    *slow += 1;
    if *slow >= limits.slow_readings_in_a_row {
        return Some(Stop::SlowDrive);
    }
    return None;
}

/// What:
/// `fn remove_pieces(entry: &Path, limits: &ToolchainFolderLimits, volume: &dyn Volume, probe: &Path, deadline: &Deadline, dry_run: bool, stop_under_bound: bool) -> Pieces`
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
/// function removePieces(entry: string, limits: Limits, volume: Volume, probe: string, deadline: Deadline, dryRun: boolean, stopUnderBound: boolean): Pieces
/// ```
fn remove_pieces(entry: &Path, limits: &ToolchainFolderLimits, volume: &dyn Volume, probe: &Path, deadline: &Deadline, dry_run: bool, stop_under_bound: bool) -> Pieces {
    let pieces: Vec<PathBuf> = children_of(entry);
    let mut removed: Pieces = Pieces { total: pieces.len(), ..Pieces::default() };
    let mut slow: u32 = 0;
    for (index, piece) in pieces.iter().enumerate() {
        if deadline.passed() {
            removed.stopped = Some(Stop::Deadline);
            return removed;
        }
        if index == 0
            && let Some(stopped) = slow_reading(limits, volume, probe, &mut slow)
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
            if let Some(stopped) = slow_reading(limits, volume, probe, &mut slow) {
                removed.stopped = Some(stopped);
                return removed;
            }
        }
        removed.gone += 1;
        if stop_under_bound
            && index + 1 < pieces.len()
            && let Ok(free) = volume.free_bytes(entry)
            && free >= limits.free_space_bound_bytes
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
/// `fn piece_line(what: &str, removed: &Pieces, entry: &Path) -> String` is the one line a removal of one folder earned.
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
fn piece_line(what: &str, removed: &Pieces, entry: &Path) -> String {
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
/// `fn stop_line(stopped: Option<Stop>, limits: &ToolchainFolderLimits) -> Option<String>` is the line a pass earns when it stops removing
/// before the volume has its free space back.
/// Why:
/// A stop at a slow drive,
/// at an unreadable drive or at the pass's own bound is a fact the next pass needs,
/// and the volume having its space back is not a stop worth a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopLine(stopped: Stop | undefined, limits: Limits): string | undefined
/// ```
fn stop_line(stopped: Option<Stop>, limits: &ToolchainFolderLimits) -> Option<String> {
    return match stopped {
        Some(Stop::SlowDrive) => Some(format!(
            "the toolchain-folder removal stopped: the drive answered over {} ms {} times in a row; the next pass continues",
            limits.slow_reading.as_millis(),
            limits.slow_readings_in_a_row
        )),
        Some(Stop::ReadingFailed) => Some(String::from("the toolchain-folder removal stopped: the drive could not be read, so the pass cannot tell whether it is slow")),
        Some(Stop::Deadline) => Some(format!("the toolchain-folder removal stopped at its own bound of {} s; the next pass continues", limits.pass_limit.as_secs())),
        Some(Stop::Interrupted) => Some(String::from("the toolchain-folder removal stopped on a fault point; the next pass continues")),
        Some(Stop::Finished) | Some(Stop::UnderBound) | None => None,
    };
}

/// What:
/// `fn ask_for_idle_disk_priority(own_pid: i32) -> String` puts this process in the idle disk class and answers the line that
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
fn ask_for_idle_disk_priority(own_pid: i32) -> String {
    let bound: Bound = Bound { limit: IDLE_CLASS_LIMIT, what: String::from("`ionice --class idle`"), rerun: String::from("cargo-shadow lean run") };
    return match output_within(Command::new("ionice").args(["--class", "idle", "--pid"]).arg(own_pid.to_string()), None, &bound) {
        Ok(found) if found.status.success() => String::from("the pass asked for the idle disk class (ionice --class idle) and got it"),
        Ok(found) => format!("the pass asked for the idle disk class and did not get it: {}", String::from_utf8_lossy(&found.stderr).trim()),
        Err(RunError::TimedOut { what, limit, .. }) => format!("{what} did not finish within {} s, so the pass runs at the disk priority it inherited", limit.as_secs()),
        Err(other) => format!("the pass could not ask for the idle disk class ({other:?}), so it runs at the disk priority it inherited"),
    };
}

/// What the removal of toolchain folders did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Outcome {
    /// One line per folder removed,
    /// per folder kept,
    /// and per reason the pass stopped.
    pub lines: Vec<String>,
    /// Folders removed whole.
    pub removed: usize,
    /// Folders kept.
    pub kept: usize,
    /// The disk bytes the removal freed.
    pub bytes: u64,
    /// Where the last removal stopped,
    /// when the pass was stopped in the middle of one.
    pub stopped_in: Option<PathBuf>,
}

/// What:
/// `pub fn retire_toolchain_folders(base: &Path, keep_toolchain: Option<&str>, limits: &ToolchainFolderLimits, volume: &dyn Volume, environment: &GuardEnvironment, now: i64, dry_run: bool, deadline: &Deadline, log: &mut Vec<String>) -> Outcome`
/// finishes the removals an earlier pass started and then removes the folders the volume's free space asks for,
/// oldest modification first.
/// Why:
/// It is mechanism G.
/// A pass that removed nothing and kept nothing says nothing,
/// and a folder a pass was stopped in the middle of is finished before anything else,
/// whatever the free space is now:
/// it is already committed to removal and no build is given its place any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireToolchainFolders(base: string, keepToolchain: string | undefined, limits: Limits, volume: Volume, environment: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): Outcome
/// ```
pub fn retire_toolchain_folders(
    base: &Path,
    keep_toolchain: Option<&str>,
    limits: &ToolchainFolderLimits,
    volume: &dyn Volume,
    environment: &GuardEnvironment,
    now: i64,
    dry_run: bool,
    deadline: &Deadline,
    log: &mut Vec<String>,
) -> Outcome {
    let retiring: PathBuf = retiring_directory(base);
    let probe: PathBuf = base.join(READING_FILE_NAME);
    let pass_deadline: Deadline = Deadline::after(limits.pass_limit.min(deadline.remaining()));
    let mut outcome: Outcome = Outcome::default();

    // An earlier pass claimed a folder and was stopped between two of its pieces.
    let mut serial: u64 = 0;
    for claimed in children_of(&retiring) {
        if !matches!(fs::symlink_metadata(&claimed), Ok(metadata) if metadata.is_dir()) {
            continue;
        }
        let name: String = folder_name_of(&claimed);
        let verb: &str = if dry_run { "would finish" } else { "finished" };
        let removable: Pieces = remove_pieces(&claimed, limits, volume, &probe, &pass_deadline, dry_run, false);
        let what: String = format!("{verb} the removal of toolchain folder {name} that an earlier pass started");
        let line: String = piece_line(&what, &removable, &claimed);
        outcome.lines.push(line.clone());
        emit(log, line);
        outcome.bytes += removable.bytes;
        if removable.stopped == Some(Stop::Finished) {
            if !dry_run {
                let _ = fs::remove_dir(&claimed);
            }
            outcome.removed += 1;
        } else {
            outcome.stopped_in = Some(claimed.clone());
        }
        if let Some(text) = stop_line(removable.stopped, limits) {
            outcome.lines.push(text.clone());
            emit(log, text);
        }
        if outcome.stopped_in.is_some() {
            return outcome;
        }
        serial += 1;
    }

    let free: u64 = match volume.free_bytes(base) {
        Ok(found) => found,
        Err(error) => {
            emit(log, format!("toolchain folders: the free space of {} could not be read ({error}), so no folder was judged", base.display()));
            return outcome;
        }
    };
    if free >= limits.free_space_bound_bytes {
        return outcome;
    }
    let (listed, unreadable): (Vec<Candidate>, Vec<String>) = candidates(base);
    outcome.lines.extend(unreadable);
    // The scan of every process is taken at most once, and only when a folder is old enough to be judged by it.
    let mut scanned: Option<ScanReport> = None;
    for candidate in listed.iter() {
        if pass_deadline.passed() {
            let text: String = format!("toolchain folders: the pass's own bound of {} s is gone; the next pass continues", limits.pass_limit.as_secs());
            outcome.lines.push(text.clone());
            emit(log, text);
            return outcome;
        }
        // A folder old enough to go is judged against a scan of the machine, which the first such folder pays for.
        if now - candidate.modified > limits.age_seconds && scanned.is_none() {
            scanned = Some(scan_of(environment, &listed));
        }
        if let Some(why) = keep_reason(candidate, keep_toolchain, limits, now, scanned.as_ref()) {
            outcome.kept += 1;
            outcome.lines.push(format!("kept toolchain folder {}: {why}", candidate.path.display()));
            continue;
        }
        let what: String = format!("removed toolchain folder {} (its own last change was {} s ago)", candidate.path.display(), now - candidate.modified);
        if dry_run {
            let count: usize = children_of(&candidate.path).len();
            let line: String = format!(
                "would remove toolchain folder {}: {count} piece(s); its own last change was {} s ago, no process uses it, and the volume has {} free of the {} byte bound",
                candidate.path.display(),
                now - candidate.modified,
                gib(free),
                gib(limits.free_space_bound_bytes)
            );
            outcome.removed += 1;
            outcome.lines.push(line.clone());
            emit(log, line);
            continue;
        }
        if let Some(changed) = changed_since(candidate) {
            outcome.kept += 1;
            outcome.lines.push(format!("kept toolchain folder {}: {changed}", candidate.path.display()));
            continue;
        }
        let item = match item_of(&candidate.path) {
            Ok(found) => found,
            Err(error) => {
                outcome.kept += 1;
                outcome.lines.push(format!("kept toolchain folder {}: it could not be read: {error}", candidate.path.display()));
                continue;
            }
        };
        let claimed: PathBuf = match claim_into_trash(&item, &retiring, serial) {
            Ok(found) => found,
            Err(refusal) => {
                outcome.kept += 1;
                outcome.lines.push(format!("kept toolchain folder {}: {refusal}", candidate.path.display()));
                continue;
            }
        };
        serial += 1;
        let removable: Pieces = remove_pieces(&claimed, limits, volume, &probe, &pass_deadline, dry_run, true);
        let line: String = piece_line(&what, &removable, &claimed);
        outcome.lines.push(line.clone());
        emit(log, line);
        outcome.bytes += removable.bytes;
        if removable.stopped == Some(Stop::Finished) {
            let _ = fs::remove_dir(&claimed);
            outcome.removed += 1;
        } else {
            outcome.stopped_in = Some(claimed.clone());
        }
        if let Some(text) = stop_line(removable.stopped, limits) {
            outcome.lines.push(text.clone());
            emit(log, text);
        }
        if outcome.stopped_in.is_some() {
            return outcome;
        }
        // The free space that decides whether another folder goes is read again after each folder.
        match volume.free_bytes(base) {
            Ok(found) if found >= limits.free_space_bound_bytes => return outcome,
            _ => {}
        }
    }
    return outcome;
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

/// What:
/// `pub fn pieces_of(folder: &Path) -> Vec<String>` is the names of a folder's pieces.
/// Why:
/// A test says which shards of a fixture are gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const piecesOf = (folder: string): string[] => readdirSync(folder).sort();
/// ```
pub fn pieces_of(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for piece in children_of(folder) {
        names.push(name_of(&piece));
    }
    return names;
}

/// What:
/// `pub fn identity_of_candidate(path: &Path) -> Option<Candidate>` is a folder as the pass judges it,
/// or nothing when it cannot be read.
/// Why:
/// The control of a folder a build changes between the judgement and the claim needs the judgement itself,
/// and a folder a build is in is one the pass read once and must not carry away on that reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const identityOfCandidate = (path: string): Candidate | undefined => { try { const found = lstatSync(path); return { path, name: basename(path), modified: Math.floor(found.mtimeMs / 1000), identity: { device: found.dev, inode: found.ino } }; } catch { return undefined; } };
/// ```
pub fn identity_of_candidate(path: &Path) -> Option<Candidate> {
    let metadata: fs::Metadata = fs::symlink_metadata(path).ok()?;
    return Some(Candidate { path: path.to_path_buf(), name: name_of(path), modified: metadata.mtime(), identity: Identity { device: metadata.dev(), inode: metadata.ino() } });
}

/// What:
/// `pub fn run_toolchain_folders(base: &Path, dry_run: bool, limits: &ToolchainFolderLimits, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String>`
/// is mechanism G on the real machine.
/// Why:
/// It reads the clock,
/// the real volume,
/// and the toolchain this process itself builds with when its environment names one,
/// and it asks for the idle disk class before it removes anything,
/// so this function is the only part of the mechanism that touches the machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runToolchainFolders(base: string, dryRun: boolean, limits: Limits, deadline: Deadline, environment: GuardEnvironment): string[]
/// ```
pub fn run_toolchain_folders(base: &Path, dry_run: bool, limits: &ToolchainFolderLimits, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String> {
    let keep: Option<String> = std::env::var("RUSTUP_TOOLCHAIN").ok().filter(|name| { return !name.is_empty(); });
    let mut log: Vec<String> = Vec::new();
    if !dry_run
        && let Ok(free) = RealVolume.free_bytes(base)
        && free < limits.free_space_bound_bytes
    {
        let text: String = ask_for_idle_disk_priority(environment.own_pid);
        emit(&mut log, text);
    }
    let outcome: Outcome = retire_toolchain_folders(base, keep.as_deref(), limits, &RealVolume, environment, now_seconds(), dry_run, deadline, &mut log);
    return outcome.lines;
}

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
//! the probe in that day's run folder,
//! `keep/what-lean-decides-for-the-real-trees-2026-11-13.log`),
//! so the folder is decided from one look at the folder itself:
//! its own modification time,
//! which the shadow sets on every build it hands to that toolchain
//! (`lean_keyed_folder.rs`),
//! and one reading of the volume's free space.
//! The volume lost about 36 GiB an hour on 2026-11-13 and a full drive ended twenty agent runs on 2026-11-12,
//! so the pass asks for the idle disk class,
//! removes one piece of a folder at a time
//! (`lean_toolchain_folder_pieces.rs`),
//! and stops when the drive answers slowly.
//! A removal that is stopped is finished by a later pass:
//! the folder is renamed into `RETIRING_DIRECTORY_NAME` in one atomic step,
//! so the place the shadow hands a build no longer holds it,
//! and a build that meets it makes a fresh folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function retireToolchainFolders(base: string, keepToolchain: string | undefined, limits: ToolchainFolderLimits, volume: Volume, environment: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): Outcome
//! ```

/// Files,
/// paths and the system clock.
use std::{fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}, time::Duration};

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
/// The scan of the processes that is the proof of "not in use".
use crate::lean_proc_scan::{ScanReport, scan_processes};
/// The claim that takes a folder out of the place a build is given.
use crate::lean_removal::{claim_into_trash, item_of};
/// The lines a removal writes.
use crate::lean_run_report::{emit, gib};
/// Directory entries and names.
use crate::lean_text::{is_directory, name_of};
/// The removal of one folder's pieces,
/// and the drive it is measured on.
use crate::lean_toolchain_folder_pieces::{
    PiecePolicy,
    Volume,
    READING_FILE_NAME,
    Pieces,
    RealVolume,
    Stop,
    ask_for_idle_disk_priority,
    children_of,
    folder_name_of,
    piece_line,
    remove_pieces,
    retiring_directory,
    stop_line,
};

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
    let by_time: std::cmp::Ordering = left.modified.cmp(&right.modified);
    return match by_time {
        std::cmp::Ordering::Equal => left.name.cmp(&right.name),
        decided => decided,
    };
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
/// (measured 2026-11-13 on this machine:
/// 0.19 s for a scan of one folder and 0.24 s of nine,
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
    let policy: PiecePolicy = PiecePolicy {
        free_space_bound_bytes: limits.free_space_bound_bytes,
        pass_limit: limits.pass_limit,
        slow_reading: limits.slow_reading,
        slow_readings_in_a_row: limits.slow_readings_in_a_row,
    };
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
        let removable: Pieces = remove_pieces(&claimed, &policy, volume, &probe, &pass_deadline, dry_run, false);
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
        if let Some(text) = stop_line(removable.stopped, &policy) {
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
        let removable: Pieces = remove_pieces(&claimed, &policy, volume, &probe, &pass_deadline, dry_run, true);
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
        if let Some(text) = stop_line(removable.stopped, &policy) {
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
    // An empty `RUSTUP_TOOLCHAIN` means the variable is unset, not that a build runs with a toolchain of no name.
    let keep: Option<String> = match std::env::var("RUSTUP_TOOLCHAIN") {
        Ok(name) if !name.is_empty() => Some(name),
        _other => None,
    };
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

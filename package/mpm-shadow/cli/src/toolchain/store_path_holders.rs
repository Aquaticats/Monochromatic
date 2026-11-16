//! What:
//! Which live processes of this user hold a path under an mpm-shadow store:
//! the program a process runs,
//! the directory it works in,
//! and every file it has mapped or open.
//! Why:
//! The cleanup never removes a version a running process uses.
//! A process whose program, working directory or mapping names a version keeps it,
//! and a process the kernel refuses to describe is named in the reason a version stays,
//! because a version that may be in use is not removed.
//! The scan reads the process table and nothing else:
//! no process of another user is read (the kernel refuses those anyway),
//! and no file a process holds is opened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // store-path-holders.ts
//! export function holdersOf(storeRoots: string[], deadline: Date): StoreHolders
//! export function heldReason(holders: StoreHolders, versionDirectory: string): string | undefined
//! export function opaqueReason(holders: StoreHolders, toolDirectoryName: string): string | undefined
//! ```

/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Duration`,
/// `Instant` from `std::time`.
use std::time::{Duration, Instant, SystemTime};

/// How a process holds a store path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HoldKind {
    /// The process was started from it.
    Runs,
    /// The process works inside it.
    WorksInside,
    /// The process has it open or mapped.
    HasOpen,
}

/// The methods of `HoldKind`.
impl HoldKind {
    /// The words a reason names this hold with.
    pub fn words(self) -> &'static str {
        return match self {
            HoldKind::Runs => "runs",
            HoldKind::WorksInside => "works inside",
            HoldKind::HasOpen => "has open",
        };
    }
}

/// One store path a live process holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeldPath {
    /// The path, as the process table resolved it.
    pub path: PathBuf,
    /// How the process holds it.
    pub kind: HoldKind,
    /// The process.
    pub pid: u32,
}

/// A process of this user whose program the kernel will not name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpaqueProcess {
    /// The process.
    pub pid: u32,
    /// The name the process calls itself (`/proc/<pid>/comm`),
    /// which is the only thing the kernel still says about it.
    pub comm: String,
    /// When the process started, when the kernel still says it,
    /// which is what tells a process that may have re-executed itself into a version from one that started before the
    /// version was installed.
    pub started: Option<SystemTime>,
}

/// What a scan of the process table found.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StoreHolders {
    /// Every store path a process of this user holds.
    pub held: Vec<HeldPath>,
    /// Processes of this user the kernel refuses to describe.
    pub opaque: Vec<OpaqueProcess>,
    /// How many processes the scan read.
    pub processes: usize,
    /// Why the scan could not be completed,
    /// when it could not.
    /// A cleanup that cannot ask the process table keeps every version.
    pub incomplete: Option<String>,
}

/// What:
/// The store path one line of `/proc/<pid>/maps` names,
/// with the ` (deleted)` mark of an unlinked file removed.
/// Why:
/// A mapping line is `range perms offset device inode path`;
/// the path is the rest after those five fields and may hold spaces,
/// so the fields are counted, not split.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function mappedPath(line: string): string | undefined
/// ```
pub fn mapped_path(line: &str) -> Option<&str> {
    let mut rest: &str = line;
    // Five fields: address range, permissions, offset, device, inode.
    for _ in 0..5 {
        let start: usize = rest.find(|character: char| return !character.is_whitespace())?;
        rest = &rest[start..];
        let end: usize = rest.find(|character: char| return character.is_whitespace()).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    let path: &str = rest.trim_start();
    if path.is_empty() {
        return None;
    }
    return Some(path.strip_suffix(" (deleted)").unwrap_or(path));
}

/// How many processes are read between two reads of the clock.
const DEADLINE_CHECK_EVERY: usize = 64;

/// Whether `path` is `store_root` or lies below it.
fn is_under(path: &Path, store_roots: &[PathBuf]) -> bool {
    for store_root in store_roots {
        if path.starts_with(store_root) {
            return true;
        }
    }
    return false;
}

/// The canonical form of every store root,
/// so a path the process table resolved compares equal.
fn canonical_roots(store_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for store_root in store_roots {
        match std::fs::canonicalize(store_root) {
            Ok(canonical) => roots.push(canonical),
            // A store that cannot be resolved is compared by the path it was given as.
            Err(_) => roots.push(store_root.clone()),
        }
    }
    return roots;
}

/// What:
/// Every store path a live process of this user holds,
/// read from the process table.
/// Why:
/// This is the only place that answers "is a running process using this version".
/// A read the kernel refuses is recorded as an opaque process instead of silently counting as "not held":
/// the version it may run is then kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function holdersOf(storeRoots: string[], deadline: Date): StoreHolders
/// ```
#[cfg(target_os = "linux")]
pub fn holders_of(store_roots: &[PathBuf], deadline: Instant) -> StoreHolders {
    use std::os::unix::fs::MetadataExt;

    let roots: Vec<PathBuf> = canonical_roots(store_roots);
    let mut holders: StoreHolders = StoreHolders::default();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        holders.incomplete = Some("the process table at /proc cannot be read".to_string());
        return holders;
    };
    let Ok(self_metadata) = std::fs::metadata("/proc/self") else {
        holders.incomplete = Some("the user id of this process cannot be read from /proc/self".to_string());
        return holders;
    };
    let self_uid: u32 = self_metadata.uid();
    let mut pids: Vec<u32> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if let Ok(pid) = name.parse::<u32>() {
            pids.push(pid);
        }
    }
    pids.sort_unstable();
    for (index, pid) in pids.iter().enumerate() {
        // What: the deadline is checked once per block of processes.
        // Why: reading a clock per process costs more than the process table it guards.
        if index % DEADLINE_CHECK_EVERY == 0 && Instant::now() >= deadline {
            holders.incomplete = Some(format!(
                "the scan of the process table ran out of its time limit after {} of {} processes",
                index,
                pids.len()
            ));
            return holders;
        }
        let process_directory: PathBuf = PathBuf::from(format!("/proc/{pid}"));
        let Ok(metadata) = std::fs::metadata(&process_directory) else {
            // The process ended between the listing and this read.
            continue;
        };
        if metadata.uid() != self_uid {
            // Another user's process: the kernel hides its program and mappings from this user, and a store under
            // this user's home is not what such a process runs.
            continue;
        }
        holders.processes += 1;
        let mut denied: bool = false;
        for (file, kind) in [("exe", HoldKind::Runs), ("cwd", HoldKind::WorksInside)] {
            let link: PathBuf = process_directory.join(file);
            match std::fs::read_link(&link) {
                Ok(target) => {
                    if is_under(&target, &roots) {
                        holders.held.push(HeldPath { path: target, kind, pid: *pid });
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => denied = true,
                Err(_) => {}
            }
        }
        // What: the descriptors are read by their links only.
        // Why: an open file is not a mapping, and a program that holds a version's file open (`tail -f`, a build
        // watching a file) uses that version. The descriptor's own record (`fdinfo`) is never read.
        match std::fs::read_dir(process_directory.join("fd")) {
            Ok(descriptors) => {
                for descriptor in descriptors.flatten() {
                    if let Ok(target) = std::fs::read_link(descriptor.path())
                        && is_under(&target, &roots)
                    {
                        holders.held.push(HeldPath { path: target, kind: HoldKind::HasOpen, pid: *pid });
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => denied = true,
            Err(_) => {}
        }
        match std::fs::read_to_string(process_directory.join("maps")) {
            Ok(maps) => {
                for line in maps.lines() {
                    let Some(mapped) = mapped_path(line) else {
                        continue;
                    };
                    let mapped_path: PathBuf = PathBuf::from(mapped);
                    if is_under(&mapped_path, &roots) {
                        holders.held.push(HeldPath { path: mapped_path, kind: HoldKind::HasOpen, pid: *pid });
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => denied = true,
            Err(_) => {}
        }
        if denied {
            let comm: String = std::fs::read_to_string(process_directory.join("comm"))
                .unwrap_or_default()
                .trim()
                .to_string();
            holders.opaque.push(OpaqueProcess { pid: *pid, comm, started: process_start_time(*pid) });
        }
    }
    return holders;
}

/// Tells the cleanup that it cannot tell which versions a process uses (no process table on this platform).
#[cfg(not(target_os = "linux"))]
pub fn holders_of(_store_roots: &[PathBuf], _deadline: Instant) -> StoreHolders {
    let mut holders: StoreHolders = StoreHolders::default();
    holders.incomplete =
        Some("this platform keeps no process table at /proc, so no process can be asked what it runs".to_string());
    return holders;
}

/// What:
/// The reason `version_directory` stays because a live process holds something below it,
/// or `None` when no process does.
/// Why:
/// One sentence names the hold and a process,
/// so a person can look at what kept the version (or end it).
/// The process table spells a program's path with every symbolic link resolved,
/// and a store root spelled through `$HOME` need not be that spelling
/// (on this machine `/home` is a link to `/var/home`),
/// so the directory is compared both as it was given and as it resolves;
/// without the second spelling a store under a linked home would match no holder at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function heldReason(holders: StoreHolders, versionDirectory: string): string | undefined
/// ```
pub fn held_reason(holders: &StoreHolders, version_directory: &Path) -> Option<String> {
    // What: the directory's own spelling, and the one every link in it resolved, when it resolves.
    // Why: a holder's path comes from the kernel, which resolves links, and the caller's path comes from `$HOME`.
    let physical: Option<PathBuf> = std::fs::canonicalize(version_directory).ok();
    let is_below = |path: &Path| -> bool {
        if path.starts_with(version_directory) {
            return true;
        }
        return physical.as_ref().is_some_and(|spelling| return path.starts_with(spelling));
    };
    for held in &holders.held {
        if !is_below(&held.path) {
            continue;
        }
        let others: usize = holders.held.iter().filter(|other| return is_below(&other.path)).count();
        let more: String = if others > 1 { format!(" and {} more process(es)", others - 1) } else { String::new() };
        return Some(format!("pid {} {} {}{more}", held.pid, held.kind.words(), held.path.display()));
    }
    return None;
}

/// What:
/// When the version at `directory` was installed, as the filesystem last changed its record of that directory,
/// or `None` when the record cannot be read.
/// Why:
/// The write time of a version directory is not the install time: a published archive is unpacked with `tar`, which
/// restores the times the archive recorded, and a version unpacked from an old release then looks installed long
/// before it was. The change time is written by the filesystem itself whenever the directory or its entries change,
/// so it is when the install last touched it, and `tar` putting a write time back is itself such a change.
#[cfg(unix)]
pub fn installed_at(directory: &Path) -> Option<SystemTime> {
    use std::os::unix::fs::MetadataExt;

    let metadata = std::fs::symlink_metadata(directory).ok()?;
    let seconds: i64 = metadata.ctime();
    let nanos: i64 = metadata.ctime_nsec();
    let since_epoch: Duration = Duration::new(seconds.unsigned_abs(), u32::try_from(nanos.unsigned_abs()).ok()?);
    return Some(if seconds < 0 { SystemTime::UNIX_EPOCH - since_epoch } else { SystemTime::UNIX_EPOCH + since_epoch });
}

/// What:
/// When the version at `directory` was installed, on a platform without a change time.
/// Why:
/// The age rule's only other answer there is the write time, which is the best the platform says.
#[cfg(not(unix))]
pub fn installed_at(directory: &Path) -> Option<SystemTime> {
    return std::fs::symlink_metadata(directory).ok()?.modified().ok();
}

/// What:
/// Why a process of this account that the kernel refuses to describe may run the version installed at
/// `installed`, or `None` when no such process started after it.
/// Why:
/// A process that hides what it runs is a process nobody can check:
/// it may hold any file of the store and work in any version,
/// and its name is not evidence either way, because a wrapper that holds tokens hides its program the same way and
/// calls itself anything.
/// The owner's decision of 2026-11-15 (recorded in `doc/handover/cli-git-rust-implementation.md`) is the age rule:
/// a hidden process that started after a version was installed keeps that version,
/// and one that started before it is taken not to run it.
/// The gap that accepts: such a process re-executing itself into the version later is possible and not seen.
/// The other answer, keeping every version for every hidden process until it ends, was measured on this machine at
/// one such process per session (the desktop's own), so it would have kept every version for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function opaqueHolderReason(holders: StoreHolders, versionWritten: Date): string | undefined
/// ```
pub fn opaque_holder_reason(holders: &StoreHolders, installed: Option<SystemTime>) -> Option<String> {
    // What: a version whose own write time cannot be read, which is a version that is not there, is kept by nothing.
    // Why: the age rule compares two times, and a version that is not there is not something a process runs;
    // what a lock names and what a readable hold shows still keep it.
    let installed_at: SystemTime = installed?;
    // What: a process whose start the kernel will not say keeps the version, and the line says it could not tell.
    // Why: the age rule is the only evidence there is about such a process, and a guess against it would be an
    // unmeasured claim about what may be running.
    let unknown: Option<&OpaqueProcess> = holders.opaque.iter().find(|process| return process.started.is_none());
    if let Some(process) = unknown {
        return Some(format!(
            "pid {} hides what it runs, and when it started cannot be read, so a process of this account may run this version",
            process.pid
        ));
    }
    // What: a start the kernel counts is read one tick later than it happened, so a version written in that last tick
    // before a process started counts as written after it, and the version is kept.
    // Why: that direction of error costs a version a few milliseconds too young; the other direction would take one a
    // process had just started using.
    let started_after: Vec<&OpaqueProcess> = holders
        .opaque
        .iter()
        .filter(|process| return process.started.is_some_and(|started| return started > installed_at))
        .collect();
    let first: &&OpaqueProcess = started_after.first()?;
    let others: usize = started_after.len() - 1;
    let more: String = if others > 0 { format!(" and {others} more") } else { String::new() };
    return Some(format!(
        "pid {}{more} hides what it runs and started after this version was installed, so a process of this account may run it",
        first.pid
    ));
}

/// What:
/// When the process `pid` started, or `None` when the kernel will not say.
/// Why:
/// The age rule needs the start of a process whose program and folders cannot be read,
/// and `/proc/<pid>/stat` still gives it: its twenty-second field counts clock ticks since boot, and `/proc/uptime`
/// says how long the machine has been up.
/// The count of ticks per second is taken as 100, which is what Linux uses on every architecture this repository
/// builds for; a machine with another rate would misdate the start by that factor, which the rule's boundary case
/// (a process started within a second of the install) already accepts.
fn process_start_time(pid: u32) -> Option<SystemTime> {
    let stat: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let uptime: f64 = std::fs::read_to_string("/proc/uptime")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    // The name the process calls itself can hold spaces and brackets, so the fields after it are counted from the last
    // closing bracket, not from the first space.
    let after_name: &str = stat.rsplit_once(')')?.1;
    let mut fields = after_name.split_whitespace();
    // The third field of the line is the state letter, and the twenty-second is the start; the fields after the name
    // start at the third.
    // What: the tick count is rounded up by one, so the start read is never earlier than the start that happened.
    // Why: the count has a tick of a hundredth of a second, and a start read earlier than it was would take a version
    // that the process had just begun to use.
    let start_ticks: f64 = fields.nth(19)?.parse::<f64>().ok()? + 1.0;
    let started_seconds: f64 = uptime - start_ticks / 100.0;
    let now: SystemTime = SystemTime::now();
    if started_seconds >= 0.0 {
        return now.checked_sub(Duration::from_secs_f64(started_seconds));
    }
    // The process started after this reading of the clock (a machine whose uptime and clock disagree): the start is
    // now.
    return Some(now);
}

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
use std::time::Instant;

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
            holders.opaque.push(OpaqueProcess { pid: *pid, comm });
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function heldReason(holders: StoreHolders, versionDirectory: string): string | undefined
/// ```
pub fn held_reason(holders: &StoreHolders, version_directory: &Path) -> Option<String> {
    for held in &holders.held {
        if !held.path.starts_with(version_directory) {
            continue;
        }
        let others: usize = holders.held.iter().filter(|other| return other.path.starts_with(version_directory)).count();
        let more: String = if others > 1 { format!(" and {} more process(es)", others - 1) } else { String::new() };
        return Some(format!("pid {} {} {}{more}", held.pid, held.kind.words(), held.path.display()));
    }
    return None;
}

/// The letters and digits of `text`, lowercased,
/// for comparing a process's name with a tool's directory name.
fn letters_of(text: &str) -> String {
    return text.chars().filter(|character| return character.is_alphanumeric()).flat_map(char::to_lowercase).collect();
}

/// The shortest process name that is compared with a tool's directory name.
/// Shorter names are ordinary words (`sh`, `ps`, `go`) and would match by accident.
const SHORTEST_COMPARED_NAME: usize = 5;

/// What:
/// The reason every version of the tool in `tool_directory_name` stays because a process of this user called itself
/// something that names this tool and the kernel will not say which file it runs,
/// or `None` when no such process exists.
/// Why:
/// A wrapper that holds tokens calls `prctl` to hide its program from every other process,
/// so a version it runs cannot be found by its path;
/// its name is all that is left,
/// and a name that matches the tool is taken as evidence that the version may be in use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function opaqueReason(holders: StoreHolders, toolDirectoryName: string): string | undefined
/// ```
pub fn opaque_reason(holders: &StoreHolders, tool_directory_name: &str) -> Option<String> {
    let tool_letters: String = letters_of(tool_directory_name);
    for opaque in &holders.opaque {
        let name: String = letters_of(&opaque.comm);
        if name.len() < SHORTEST_COMPARED_NAME {
            continue;
        }
        if tool_letters.contains(&name) {
            return Some(format!(
                "pid {} calls itself {} and the kernel will not say which file it runs, so it may run this version",
                opaque.pid, opaque.comm
            ));
        }
    }
    return None;
}

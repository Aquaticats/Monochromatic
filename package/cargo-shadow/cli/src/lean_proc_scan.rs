//! What:
//! The scan of `/proc` that finds every process using something under a path:
//! as its working directory,
//! its program,
//! an open file or a mapped file.
//! Why:
//! The second proof of "not in use" beside the build lock:
//! a removal's log line records how many processes were scanned and that none used the entry.
//! A process in another mount namespace (a container) sees the same files under other names,
//! so for those the scan compares device and inode of what they hold with the identities it was given.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function scanProcesses(procRoot: string, roots: string[], identities: Identity[]): ScanReport
//! ```

/// Files, directories and links,
/// and the device and inode of an open entry.
use std::{fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// Device and inode of what a foreign-namespace process holds.
use crate::lean_identity::{Identity, identity_of};

/// The suffix `/proc` appends to the link of a file that was unlinked while open.
const DELETED_SUFFIX: &str = " (deleted)";

/// One process using one path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessUse {
    /// The process id.
    pub pid: i32,
    /// How it uses the path: `cwd`, `exe`, `fd`, `map` or `identity`.
    pub how: &'static str,
    /// The path as the process shows it (for `identity`,
    /// the root that matched).
    pub path: PathBuf,
}

/// What the scan saw.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScanReport {
    /// Processes whose entries were read.
    pub scanned: usize,
    /// Processes whose entries could not be read (another user's),
    /// which cannot be this user's builds.
    pub unreadable: usize,
    /// Processes in another mount namespace,
    /// judged by identity and by the build lock.
    pub foreign_namespace: usize,
    /// Every use found under a root.
    pub uses: Vec<ProcessUse>,
}

impl ScanReport {
    /// The first use at or below `candidate`,
    /// which blocks its removal.
    pub fn blocks(&self, candidate: &Path) -> Option<&ProcessUse> {
        for used in &self.uses {
            if used.path.starts_with(candidate) {
                return Some(used);
            }
        }
        return None;
    }

    /// The proof text for a log line.
    pub fn describe(&self) -> String {
        return format!(
            "/proc scan: {} processes read, {} unreadable (other users), {} in other mount namespaces judged by identity, {} uses under the roots",
            self.scanned,
            self.unreadable,
            self.foreign_namespace,
            self.uses.len()
        );
    }
}

/// What:
/// `fn without_deleted(path: PathBuf) -> PathBuf` drops the ` (deleted)` marker `/proc` adds to a link.
/// Why:
/// A mapped file that was unlinked still names the removed entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutDeleted = (p: string) => p.endsWith(' (deleted)') ? p.slice(0, -10) : p;
/// ```
fn without_deleted(path: PathBuf) -> PathBuf {
    let text: String = path.to_string_lossy().to_string();
    return match text.strip_suffix(DELETED_SUFFIX) {
        Some(clean) => PathBuf::from(clean),
        None => path,
    };
}

/// What:
/// `fn map_path(line: &str) -> Option<PathBuf>` is the file a `/proc/<pid>/maps` line maps,
/// when it maps one.
/// Why:
/// The path is everything after the five fixed fields,
/// and it may contain spaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mapPath = (line: string): string | undefined => { const rest = line.split(/\s+/).slice(5).join(' '); return rest.startsWith('/') ? rest : undefined; };
/// ```
fn map_path(line: &str) -> Option<PathBuf> {
    let mut rest: &str = line;
    for _ in 0..5 {
        rest = rest.trim_start();
        let end: usize = rest.find(char::is_whitespace)?;
        rest = &rest[end..];
    }
    let path: &str = rest.trim_start();
    if !path.starts_with('/') {
        return None;
    }
    return Some(without_deleted(PathBuf::from(path)));
}

/// What:
/// `fn note_if_under(report: &mut ScanReport, pid: i32, how: &'static str, path: PathBuf, roots: &[&Path])` records a use
/// when the path is at or below any root.
/// Why:
/// Every kind of use goes through one test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteIfUnder(report: ScanReport, pid: number, how: string, path: string, roots: string[]): void
/// ```
fn note_if_under(report: &mut ScanReport, pid: i32, how: &'static str, path: PathBuf, roots: &[&Path]) {
    let clean: PathBuf = without_deleted(path);
    for root in roots {
        if clean.starts_with(root) {
            report.uses.push(ProcessUse { pid, how, path: clean });
            return;
        }
    }
}

/// What:
/// `fn scan_foreign(report: &mut ScanReport, pid: i32, entry: &Path, identities: &[Identity])` compares what a process in
/// another mount namespace holds with the given identities.
/// Why:
/// Its paths are spelled in its own namespace (`/work/target`), so only device and inode can match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanForeign(report: ScanReport, pid: number, entry: string, identities: Identity[]): void
/// ```
fn scan_foreign(report: &mut ScanReport, pid: i32, entry: &Path, identities: &[Identity]) {
    let mut held: Vec<PathBuf> = vec![entry.join("cwd")];
    if let Ok(descriptors) = fs::read_dir(entry.join("fd")) {
        for descriptor in descriptors.flatten() {
            held.push(descriptor.path());
        }
    }
    for link in held {
        let Ok(metadata) = fs::metadata(&link) else { continue };
        let found: Identity = Identity { device: metadata.dev(), inode: metadata.ino() };
        if identities.contains(&found) {
            report.uses.push(ProcessUse { pid, how: "identity", path: link });
        }
    }
}

/// What:
/// `pub fn scan_processes(proc_root: &Path, roots: &[&Path], identities: &[Identity], skip_pid: i32) -> ScanReport` reads
/// every process of the procfs at `proc_root`.
/// Why:
/// It finds the processes whose working directory, program, open files or mapped files are under any root,
/// and the container processes that hold an entry with one of the identities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function scanProcesses(procRoot: string, roots: string[], identities: Identity[], skipPid: number): ScanReport
/// ```
pub fn scan_processes(proc_root: &Path, roots: &[&Path], identities: &[Identity], skip_pid: i32) -> ScanReport {
    let mut report: ScanReport = ScanReport::default();
    let own_namespace: Option<PathBuf> = fs::read_link(proc_root.join("self/ns/mnt")).ok();
    let Ok(entries) = fs::read_dir(proc_root) else { return report };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let Ok(pid) = name.parse::<i32>() else { continue };
        if pid == skip_pid {
            continue;
        }
        let path: PathBuf = entry.path();
        // A process that cannot show its working directory is another user's (or is gone): it is counted, not judged.
        if fs::read_link(path.join("cwd")).is_err() {
            report.unreadable += 1;
            continue;
        }
        report.scanned += 1;
        let namespace: Option<PathBuf> = fs::read_link(path.join("ns/mnt")).ok();
        if own_namespace.is_some() && namespace.is_some() && namespace != own_namespace {
            report.foreign_namespace += 1;
            scan_foreign(&mut report, pid, &path, identities);
            continue;
        }
        if let Ok(target) = fs::read_link(path.join("cwd")) {
            note_if_under(&mut report, pid, "cwd", target, roots);
        }
        if let Ok(target) = fs::read_link(path.join("exe")) {
            note_if_under(&mut report, pid, "exe", target, roots);
        }
        if let Ok(descriptors) = fs::read_dir(path.join("fd")) {
            for descriptor in descriptors.flatten() {
                if let Ok(target) = fs::read_link(descriptor.path()) {
                    note_if_under(&mut report, pid, "fd", target, roots);
                }
            }
        }
        if let Ok(maps) = fs::read_to_string(path.join("maps")) {
            for line in maps.lines() {
                if let Some(mapped) = map_path(line) {
                    note_if_under(&mut report, pid, "map", mapped, roots);
                }
            }
        }
    }
    return report;
}

/// What:
/// `pub fn identities_of_existing(paths: &[PathBuf]) -> Vec<Identity>` reads the identity of every path that exists.
/// Why:
/// The scan of container processes needs the lock files' and directories' identities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const identitiesOfExisting = (paths: string[]): Identity[] => paths.flatMap((p) => { try { return [identityOf(p)]; } catch { return []; } });
/// ```
pub fn identities_of_existing(paths: &[PathBuf]) -> Vec<Identity> {
    let mut found: Vec<Identity> = Vec::new();
    for path in paths {
        if let Ok(identity) = identity_of(path) {
            found.push(identity);
        }
    }
    return found;
}

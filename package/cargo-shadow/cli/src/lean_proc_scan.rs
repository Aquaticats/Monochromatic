//! What:
//! The scan of `/proc` that finds every process using something under a path:
//! as its working directory,
//! its program,
//! an open file or a mapped file.
//! Why:
//! The second proof of "not in use" beside the build lock:
//! a removal's log line records how many processes were scanned and that none used the entry.
//! A process in another mount namespace (a container) sees the same files under other names,
//! so for those the scan compares device and inode of what they hold with the identities it was given,
//! and it compares the paths that namespace spells the way the root is spelled here,
//! which covers a bind mount of the root at the root's own path.
//! A root is compared both as it is spelled and with its symbolic links resolved,
//! because `/proc` names a held file the way the kernel holds it:
//! no link is left in that name,
//! so a build base reached through a linked home directory would match none of them.
//! A process whose entries cannot be read is classified instead of skipped:
//! a kernel thread and an ended process hold no file,
//! and a process of another user cannot hold one that the path denies that user,
//! so what is left is the uncertainty the removal's proof line names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function scanProcesses(procRoot: string, roots: string[], identities: Identity[]): ScanReport
//! ```

/// Files,
/// directories and links,
/// and the device and inode of an open entry.
use std::{fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// Device and inode of what a foreign-namespace process holds.
use crate::lean_identity::{Identity, identity_of};

/// The suffix `/proc` appends to the link of a file that was unlinked while open.
const DELETED_SUFFIX: &str = " (deleted)";

/// How a use found in another mount namespace by device and inode is named.
/// Why:
/// Such a use holds one of the identities the scan was given,
/// which are this root's build locks,
/// so it is a use of the whole root and not of one path under it.
const FOREIGN_IDENTITY_HOW: &str = "identity";

/// One process using one path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessUse {
    /// The process id.
    pub pid: i32,
    /// How it uses the path:
    /// `cwd`,
    /// `exe`,
    /// `fd`,
    /// `map`,
    /// `identity` or `uninspectable`.
    pub how: &'static str,
    /// The path as the process shows it (for `identity`,
    /// the root that matched;
    /// for `uninspectable`,
    /// the root that process could reach).
    pub path: PathBuf,
}

/// What the scan saw.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScanReport {
    /// Processes whose entries were read.
    pub scanned: usize,
    /// Processes whose entries could not be read and that hold no file:
    /// kernel threads (which have no descriptor table) and ended processes.
    pub inert: usize,
    /// Live processes whose entries could not be read.
    /// A kernel thread's and an ended process's absence from this count is what the scan proves;
    /// what stays here is what the kernel hides from a plain user.
    pub unreadable: usize,
    /// Of `unreadable`,
    /// the processes of another user that could reach a scanned root through the path's permissions.
    /// They are named so a reader sees which part of the uncertainty belongs to another user's access to the path.
    pub unreadable_reaching: usize,
    /// Processes in another mount namespace,
    /// judged by identity and by the build lock.
    pub foreign_namespace: usize,
    /// Every use found under a root.
    pub uses: Vec<ProcessUse>,
}

/// Reading a scan report.
impl ScanReport {
    /// The first use at or below `candidate`,
    /// which blocks its removal,
    /// and any use found by identity in another mount namespace,
    /// which blocks every entry of the root it was matched against.
    /// Why the path is compared both ways:
    /// it is tried as spelled and then with its symbolic links resolved.
    /// `/proc` names a held file the way the kernel holds it (`with_symlinks_resolved`),
    /// so a root reached through a link matches only the resolved spelling.
    /// Why only the uses it saw:
    /// a process whose entries could not be read is named by `describe` and is not judged here;
    /// `doc/handover/cargo-shadow-lean-build-folders.md` records that limit and the measurement under it.
    pub fn blocks(&self, candidate: &Path) -> Option<&ProcessUse> {
        for used in &self.uses {
            if used.how == FOREIGN_IDENTITY_HOW || used.path.starts_with(candidate) {
                return Some(used);
            }
        }
        let resolved: PathBuf = with_symlinks_resolved(candidate);
        if resolved == candidate {
            return None;
        }
        for used in &self.uses {
            if used.path.starts_with(&resolved) {
                return Some(used);
            }
        }
        return None;
    }

    /// The proof text for a log line.
    pub fn describe(&self) -> String {
        return format!(
            "/proc scan: {} processes read, {} kernel threads and ended processes (which hold no file), {} uninspectable ({} of another user reaching a root, {} of this user or of root that no plain user may look into), {} in other mount namespaces judged by identity, {} uses under the roots",
            self.scanned,
            self.inert,
            self.unreadable,
            self.unreadable_reaching,
            self.unreadable.saturating_sub(self.unreadable_reaching),
            self.foreign_namespace,
            self.uses.len()
        );
    }
}

/// What:
/// `fn with_symlinks_resolved(path: &Path) -> PathBuf` is the path with every symbolic link in it resolved,
/// and the path unchanged when it cannot be resolved.
/// Why:
/// `/proc` names the file a process holds the way the kernel holds it,
/// and that name has no symbolic link left in it,
/// so a root spelled through one,
/// a build base under a linked home directory say,
/// would match none of the uses and the proof would say "no use" about a file a process is running.
/// A path that cannot be resolved is compared as it is spelled,
/// which is the answer for an entry that is gone already.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const withSymlinksResolved = (p: string): string => { try { return realpathSync(p); } catch { return p; } };
/// ```
fn with_symlinks_resolved(path: &Path) -> PathBuf {
    return match fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(_) => path.to_path_buf(),
    };
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
fn note_if_under(report: &mut ScanReport, pid: i32, how: &'static str, path: PathBuf, roots: &[PathBuf]) {
    let clean: PathBuf = without_deleted(path);
    for root in roots {
        if clean.starts_with(root) {
            report.uses.push(ProcessUse { pid, how, path: clean });
            return;
        }
    }
}

/// What:
/// `fn scan_foreign(report: &mut ScanReport, pid: i32, entry: &Path, identities: &[Identity], roots: &[PathBuf])` compares what
/// a process in another mount namespace holds with the given identities and with the roots' spellings.
/// Why:
/// Its paths are spelled in its own namespace (`/work/target`),
/// so device and inode are what can match;
/// the program it runs and the files it has mapped are held files like any other,
/// so they are read too,
/// and a bind mount of a root at that root's own path spells them the way the root is spelled here,
/// which the path comparison catches.
/// A mount of the root at another path stays a named limit of this scan:
/// nothing here can translate one namespace's spelling into another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanForeign(report: ScanReport, pid: number, entry: string, identities: Identity[], roots: string[]): void
/// ```
fn scan_foreign(report: &mut ScanReport, pid: i32, entry: &Path, identities: &[Identity], roots: &[PathBuf]) {
    let mut held: Vec<(&'static str, PathBuf)> = vec![("cwd", entry.join("cwd")), ("exe", entry.join("exe"))];
    if let Ok(descriptors) = fs::read_dir(entry.join("fd")) {
        for descriptor in descriptors.flatten() {
            held.push(("fd", descriptor.path()));
        }
    }
    for (how, link) in held {
        let Ok(metadata) = fs::metadata(&link) else { continue };
        let found: Identity = Identity { device: metadata.dev(), inode: metadata.ino() };
        if identities.contains(&found) {
            report.uses.push(ProcessUse { pid, how: FOREIGN_IDENTITY_HOW, path: link.clone() });
        }
        if let Ok(target) = fs::read_link(&link) {
            note_if_under(report, pid, how, target, roots);
        }
    }
    if let Ok(maps) = fs::read_to_string(entry.join("maps")) {
        for line in maps.lines() {
            if let Some(mapped) = map_path(line) {
                note_if_under(report, pid, "map", mapped, roots);
            }
        }
    }
}

/// What:
/// `fn status_field(status: &str, name: &str) -> Option<&str>` is one field of `/proc/<pid>/status`.
/// Why:
/// The status of another user's process is readable where its descriptors are not,
/// and it says as whom that process runs and whether it has any file at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statusField = (status: string, name: string): string | undefined => status.split('\n').find((line) => line.startsWith(`${name}:`))?.slice(name.length + 1).trim();
/// ```
fn status_field<'a>(status: &'a str, name: &str) -> Option<&'a str> {
    for line in status.lines() {
        let Some(rest) = line.strip_prefix(name) else { continue };
        let Some(value) = rest.strip_prefix(':') else { continue };
        return Some(value.trim());
    }
    return None;
}

/// What:
/// `fn holds_no_file(status: &str) -> bool` says whether a process whose entries cannot be read still cannot hold any file:
/// it has ended,
/// or it is a kernel thread,
/// which has no descriptor table.
/// Why:
/// Neither can hold a record,
/// so neither is uncertainty a removal has to carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsNoFile = (status: string): boolean => (statusField(status, 'State') ?? '').startsWith('Z') || statusField(status, 'Kthread') === '1';
/// ```
fn holds_no_file(status: &str) -> bool {
    let state: &str = status_field(status, "State").unwrap_or("");
    if state.starts_with('Z') {
        return true;
    }
    return status_field(status, "Kthread") == Some("1");
}

/// What:
/// `fn real_uid_of(field: &str) -> Option<u32>` is the real uid of a `/proc/<pid>/status` `Uid:` field,
/// which lists the real,
/// the effective,
/// the saved and the file-system uid in that order.
/// Why:
/// The real uid says as whom the process started,
/// which is the owner whose access decides whether it could reach a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const realUidOf = (field: string): number | undefined => { const first = field.split(/\s+/u)[0]; return first === undefined ? undefined : Number(first); };
/// ```
fn real_uid_of(field: &str) -> Option<u32> {
    let first: &str = field.split_whitespace().next()?;
    return first.parse::<u32>().ok();
}

/// What:
/// `pub fn other_uids_can_reach(root: &Path) -> bool` says whether a process of another user could open something under `root`.
/// Why:
/// The kernel hides another user's descriptors,
/// so the only fact left about such a process is whether it could hold the entry at all:
/// traversal needs search permission on every directory from `/` down,
/// and one directory that grants its group and every other user nothing stops every other user.
/// Group membership is unknown here,
/// so a group bit on every directory counts as reachable,
/// which is the reading that keeps more entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const otherUidsCanReach = (root: string): boolean => !root.split('/').reduce<string[]>((all, _, index, parts) => [...all, parts.slice(0, index + 1).join('/') || '/'], []).some((one) => (statSync(one).mode & 0o077) === 0);
/// ```
pub fn other_uids_can_reach(root: &Path) -> bool {
    for ancestor in root.ancestors() {
        let Ok(metadata) = fs::metadata(ancestor) else { return true };
        if metadata.mode() & 0o077 == 0 {
            return false;
        }
    }
    return true;
}

/// What:
/// `fn note_without_entries(report: &mut ScanReport, proc_root: &Path, pid: i32, roots: &[&Path])` classifies a process
/// whose working directory could not be read,
/// instead of counting every one of them as a possible holder.
/// Why:
/// A kernel thread and an ended process hold no file,
/// so they are not uncertainty at all;
/// what is left is named,
/// and of that part the processes of another user are counted once more when the path lets that user in.
/// None of them blocks a removal:
/// the kernel shows a plain user neither their descriptors nor their mappings,
/// and refusing every removal because another user's daemon exists would stop the sweep where the build base is
/// world-searchable (measured 2026-11-08 in `doc/handover/cargo-shadow-lean-build-folders.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteWithoutEntries(report: ScanReport, procRoot: string, pid: number, roots: string[]): void
/// ```
fn note_without_entries(report: &mut ScanReport, proc_root: &Path, pid: i32, roots: &[PathBuf]) {
    let Ok(status) = fs::read_to_string(proc_root.join(pid.to_string()).join("status")) else {
        // The process ended between the listing and this read, so it holds nothing.
        report.inert += 1;
        return;
    };
    if holds_no_file(&status) {
        report.inert += 1;
        return;
    }
    report.unreadable += 1;
    let Some(uid) = status_field(&status, "Uid").and_then(real_uid_of) else {
        return;
    };
    // This user's own process can reach its own files whenever the owner bits allow it, and root reaches everything;
    // the kernel hides both from a plain user, so neither is a holder this scan can rule out.
    if uid == rustix::process::geteuid().as_raw() || uid == 0 {
        return;
    }
    for root in roots {
        if other_uids_can_reach(root) {
            report.unreadable_reaching += 1;
            return;
        }
    }
}

/// What:
/// `pub fn scan_processes(proc_root: &Path, roots: &[&Path], identities: &[Identity], skip_pid: i32) -> ScanReport` reads
/// every process of the procfs at `proc_root`.
/// Why:
/// It finds the processes whose working directory,
/// program,
/// open files or mapped files are under any root,
/// and the container processes that hold an entry with one of the identities or hold a file a root spells.
/// Each root is compared as it is spelled and with its symbolic links resolved,
/// because `/proc` names a held file without a link in it.
/// A process whose entries cannot be read is classified rather than counted as another user's:
/// see `note_without_entries`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function scanProcesses(procRoot: string, roots: string[], identities: Identity[], skipPid: number): ScanReport
/// ```
pub fn scan_processes(proc_root: &Path, roots: &[&Path], identities: &[Identity], skip_pid: i32) -> ScanReport {
    let mut report: ScanReport = ScanReport::default();
    let own_namespace: Option<PathBuf> = fs::read_link(proc_root.join("self/ns/mnt")).ok();
    // Both spellings of every root: the one a caller spelled, and the one the kernel holds.
    let mut compared: Vec<PathBuf> = Vec::with_capacity(roots.len() * 2);
    for root in roots {
        compared.push(root.to_path_buf());
        let resolved: PathBuf = with_symlinks_resolved(root);
        if resolved != *root {
            compared.push(resolved);
        }
    }
    let Ok(entries) = fs::read_dir(proc_root) else { return report };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let Ok(pid) = name.parse::<i32>() else { continue };
        if pid == skip_pid {
            continue;
        }
        let path: PathBuf = entry.path();
        // A process that cannot show its working directory is classified: it either cannot hold a file or it is
        // the uncertainty the removal's proof line names.
        if fs::read_link(path.join("cwd")).is_err() {
            note_without_entries(&mut report, proc_root, pid, &compared);
            continue;
        }
        report.scanned += 1;
        let namespace: Option<PathBuf> = fs::read_link(path.join("ns/mnt")).ok();
        if own_namespace.is_some() && namespace.is_some() && namespace != own_namespace {
            report.foreign_namespace += 1;
            scan_foreign(&mut report, pid, &path, identities, &compared);
            continue;
        }
        if let Ok(target) = fs::read_link(path.join("cwd")) {
            note_if_under(&mut report, pid, "cwd", target, &compared);
        }
        if let Ok(target) = fs::read_link(path.join("exe")) {
            note_if_under(&mut report, pid, "exe", target, &compared);
        }
        if let Ok(descriptors) = fs::read_dir(path.join("fd")) {
            for descriptor in descriptors.flatten() {
                if let Ok(target) = fs::read_link(descriptor.path()) {
                    note_if_under(&mut report, pid, "fd", target, &compared);
                }
            }
        }
        if let Ok(maps) = fs::read_to_string(path.join("maps")) {
            for line in maps.lines() {
                if let Some(mapped) = map_path(line) {
                    note_if_under(&mut report, pid, "map", mapped, &compared);
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

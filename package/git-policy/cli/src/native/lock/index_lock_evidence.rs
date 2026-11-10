//! What:
//!  Evidence about a foreign `index.lock` and the verdict it supports:
//!  proven alive,
//!  dead,
//!       or evidence-free.
//! Why:
//!  A live owner is proven by a process holding the lock open (device and inode),
//!  or by Git's
//!      `core.lockfilePid` file naming a running process that is Git.
//!  A number no process of this table has proves nothing:
//!  Git writes its own number into that file and no PID namespace,
//!  and a number means a process only inside the namespace that wrote it,
//!  so a live Git of another PID namespace writes a number this table has never had
//!  (cross-vendor review of 2026-11-09,
//!  finding 2).
//!  Everything else is
//!      evidence-free;
//!  an absent open holder never proves abandonment,
//!  because native
//!      `git commit` keeps `index.lock` without a descriptor through its hooks and editor.
//!  No verdict rests on the wall clock (audit of 2026-11-09,
//! finding 3):
//!  a process's start is derived from the wall clock while the lock's change time is stored,
//!  so a step of the clock moves the two against each other and the same evidence would then answer
//!  "the number was reused" or "the holder runs" by the clock alone.
//!  A start still read from the process table is worded beside the evidence and never decides.
//!  The sentences this evidence is reported in live in `index_lock_evidence_words.rs`,
//!  which builds them from what this file read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const evidence = await gatherIndexLockEvidence({ realIndexPath }); classifyIndexLock(evidence);
//! ```

/// Open-holder scans.
use crate::index_lock_holders::{HolderEvidence, scan_platform_holders};
/// Whether a process's command name is Git's.
use crate::git_command_name::names_git_command;
/// Process start times.
use crate::process_start::{ProcessStart, resolve_process_start};
/// The sentences about a live holder.
pub use crate::index_lock_holder_words::holder_way_out;
/// Git's PID file path.
use crate::recovery_files::lock_pid_path;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// The end of a wait and the sentences the evidence is reported in.
pub use crate::index_lock_evidence_words::{
    DEAD_OWNER_LOCK_LEFT, WaitEnd, blocker_words, ceiling_line, dead_owner_diagnostic, proven_holder_line, proven_holder_progress_line, unproven_removal_advice, unproven_wait_line,
};

/// Longest PID file text quoted in evidence,
///  in UTF-16 code units for ASCII text.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const QUOTED_TEXT_LIMIT: usize = 64;

/// What:
///  A lock's identity and change time,
///  read without following links.
/// Why:
///   `LockFileMetadata`;
///  the change time is whole milliseconds,
///  as `Number(stat.ctimeMs)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockFileMetadata = { device: bigint; inode: bigint; ctimeMs: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockMetadata {
    /// Device number.
    pub device: u64,
    /// Inode number.
    pub inode: u64,
    /// Status change time in whole milliseconds since the epoch.
    pub ctime_ms: i64,
}

/// What:
///  The process a PID file names,
///  now:
///  whether it runs,
///  when this host's clock puts its start,
///  and what it is called.
/// Why:
///   `PidProcess`.
/// The start is read from `/proc/<pid>/stat` and `/proc/uptime`,
///  so it moves with the wall clock,
///  and the command name is what tells a Git that writes its own number from a later program that took the number.
///  Only the name is read,
///  never the arguments or the environment,
///  which can hold a token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PidProcess = { start: ProcessStart; command?: string };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PidProcess {
    /// Whether the process runs,
    ///  and when this host's clock puts its start.
    pub start: ProcessStart,
    /// The command name the kernel reports,
    ///  when it could be read.
    pub command: Option<String>,
}

impl PidProcess {
    /// Whether this process table has no process with the number (or has a zombie with it).
    /// A number means a process only inside the PID namespace that wrote it,
    /// so this is no proof that the owner is gone (see the module description).
    pub fn gone(&self) -> bool {
        return matches!(self.start, ProcessStart::Missing);
    }

    /// Whether a process runs that is named like Git:
    /// Git writes its own number into the PID file,
    /// so such a process is the holder.
    pub fn names_git(&self) -> bool {
        if !matches!(self.start, ProcessStart::Running { .. }) {
            return false;
        }
        return self.command.as_deref().is_some_and(names_git_command);
    }

    /// Whether this host's clock puts the process's start after the lock's change time,
    /// which a reuse of the number would explain,
    /// and which a clock that moved explains as well.
    /// Only worded:
    /// no verdict rests on it (audit of 2026-11-09,
    /// finding 3).
    pub fn starts_after(&self, lock: &LockMetadata) -> bool {
        return match self.start {
            ProcessStart::Running { started_at_ms, resolution_ms } => started_at_ms > (lock.ctime_ms + resolution_ms) as f64,
            ProcessStart::Missing | ProcessStart::Unknown(_) => false,
        };
    }
}

/// What:
///  Git's `core.lockfilePid` file beside the lock.
/// Why:
///   `PidFileEvidence`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PidFileEvidence = { kind: 'absent' } | { kind: 'malformed'; text } | { kind: 'unreadable'; reason } | { kind: 'owner'; pid; owner };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum PidFile {
    /// No PID file exists.
    Absent,
    /// The file does not hold `pid <n>`;
    ///  the first characters of its text.
    Malformed(
        /// The first characters of the text.
        String,
    ),
    /// The file exists but could not be read.
    Unreadable(
        /// The reason the file could not be read.
        String,
    ),
    /// The file names a PID.
    Owner(
        /// The PID the file names.
        i64,
        /// What the process named by the PID is now.
        PidProcess,
    ),
}

/// What:
///  The evidence of one attempt.
/// Why:
///   `IndexLockEvidence`;
///  the holder scan is skipped once the PID file proves a live owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLockEvidence = { lockPath; lock; pidFile; holders? };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    /// Lock path.
    pub lock_path: PathBuf,
    /// Lock identity and change time.
    pub lock: LockMetadata,
    /// PID file evidence.
    pub pid_file: PidFile,
    /// Open-holder scan,
    ///  absent when the PID file proved a live owner.
    pub holders: Option<HolderEvidence>,
}

/// What:
///  The classification of a foreign lock's owner.
/// Why:
///   `IndexLockVerdict`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLockVerdict = { kind: 'proven-alive'; pid; source; command? } | { kind: 'dead'; pid } | { kind: 'evidence-free' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// A live owner is proven;
    ///  `true` when by an open descriptor,
    ///  `false` by the PID file.
    ProvenAlive {
        /// Owner PID.
        pid: i64,
        /// Whether an open descriptor proved it (otherwise the PID file).
        by_descriptor: bool,
        /// Owner command name when readable.
        command: Option<String>,
    },
    /// The PID file names a number this process table does not have,
    ///  which is what a Git that was killed leaves and what a live Git of another PID namespace leaves too.
    ///  A service that commits waits for it,
    ///  and a merge may clear the lock only where its own proof of disuse holds
    ///  (`landing/service_index_unheld_removal.rs`,
    /// `Disuse`).
    Dead(
        /// The PID of the number this table does not have.
        i64,
    ),
    /// Nothing proves a live owner or a dead one.
    EvidenceFree,
}

/// What:
///  A lock's metadata without following links,
///  or nothing when it does not exist.
/// Why:
///   `readLockMetadata`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readLockMetadata(lockPath: string): Promise<LockFileMetadata | typeof LOCK_ABSENT>;
/// ```
pub fn read_lock_metadata(lock_path: &Path) -> std::io::Result<Option<LockMetadata>> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(lock_path) {
        Ok(found) => found,
        Err(error) if is_missing(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    #[cfg(unix)]
    {
        /// The Unix metadata accessors,
        ///  for the device,
        ///  inode and change time.
        use std::os::unix::fs::MetadataExt;
        return Ok(Some(LockMetadata {
            device: metadata.dev(),
            inode: metadata.ino(),
            ctime_ms: metadata.ctime() * 1000 + metadata.ctime_nsec() / 1_000_000,
        }));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let changed: i64 = match metadata.modified() {
            Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
                Ok(elapsed) => i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX),
                Err(_) => 0,
            },
            Err(_) => 0,
        };
        return Ok(Some(LockMetadata { device: 0, inode: 0, ctime_ms: changed }));
    }
}

/// What:
///  Whether an error means the path does not exist.
/// Why:
///   `isMissing`:
///  `ENOENT` and `ENOTDIR`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.code === 'ENOENT' || error.code === 'ENOTDIR'
/// ```
fn is_missing(error: &std::io::Error) -> bool {
    return matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory);
}

/// What:
///  The PID in Git's `pid <n>` text,
///  as `read_lock_pid` reads it:
///  trailing whitespace
///       trimmed,
///  then `pid ` and decimal digits of a positive safe integer.
/// Why:
///   `parsePidFileText`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parsePidFileText('pid 42\n') // 42
/// ```
pub fn parse_pid_file_text(text: &str) -> Option<i64> {
    let trimmed: &str = crate::js_text::trim_javascript_end(text);
    let digits: &str = trimmed.strip_prefix("pid ")?;
    return crate::js_text::decimal_attempt(digits);
}

/// What:
///  The process a PID names,
///  as this host's process table reports it:
///  its start and its command name.
/// Why:
///   The production read behind every PID file's evidence,
///  and the only place the wall clock enters it,
///  where it is worded and never decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ start: resolveProcessStart(pid), command: readFileSync(`/proc/${pid}/comm`, 'utf8').trim() })
/// ```
pub fn host_pid_process(pid: i64) -> PidProcess {
    let command: Option<String> = match std::fs::read_to_string(format!("/proc/{pid}/comm")) {
        Ok(text) => {
            let name: String = text.trim().to_owned();
            if name.is_empty() { None } else { Some(name) }
        }
        Err(_) => None,
    };
    return PidProcess { start: resolve_process_start(pid), command };
}

/// What:
///  The first characters of a text,
///  as `slice(0, 64)` keeps them.
/// Why:
///   A malformed PID file is quoted,
///  not dumped whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.slice(0, 64)
/// ```
fn quoted_prefix(text: &str) -> String {
    let mut prefix: String = String::new();
    let mut units: usize = 0;
    for character in text.chars() {
        units += character.len_utf16();
        if units > QUOTED_TEXT_LIMIT {
            break;
        }
        prefix.push(character);
    }
    return prefix;
}

/// What:
///  The PID file beside a lock and the process it names.
/// Why:
///   `readPidFileEvidence`;
///  the text is read with replacement characters as `'utf8'` does,
///  and what the number is now comes from the caller's read of the process table,
///  so a control decides it without one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readPidFileEvidence({ pidPath, readProcess }): Promise<PidFileEvidence>;
/// ```
pub fn read_pid_file_evidence(pid_path: &Path, resolve: fn(i64) -> PidProcess) -> PidFile {
    let text: String = match std::fs::read(pid_path) {
        Ok(bytes) => String::from_utf8_lossy(bytes.as_slice()).into_owned(),
        Err(error) if is_missing(&error) => return PidFile::Absent,
        Err(error) => return PidFile::Unreadable(error.to_string()),
    };
    let Some(pid) = parse_pid_file_text(text.as_str()) else {
        return PidFile::Malformed(quoted_prefix(text.as_str()));
    };
    return PidFile::Owner(pid, resolve(pid));
}

/// What:
///  Classify gathered evidence.
/// Why:
///   `classifyIndexLock`:
///  an open holder first,
///  then a descriptor the scan could not identify,
///  then a PID file naming a running Git,
///  then one naming a number this table does not have,
///  and anything else is evidence-free.
///  That last verdict says what the table holds and no more:
///  a Git of another PID namespace writes a number this table has never had,
///  and holds the lock through its editor with no descriptor open,
///  so a command that may not clear the lock waits for it instead
///  (cross-vendor review of 2026-11-09,
/// finding 2).
///  The command name decides and the clock never does:
///  a Git writes its own number into the PID file,
///  so a running process named like Git is the holder whatever the wall clock says about its start,
///  and a running program that is not named like Git may be the writer under another name or a later program that took the number,
///  which no time can tell apart (audit of 2026-11-09,
/// finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyIndexLock(evidence: IndexLockEvidence): IndexLockVerdict;
/// ```
pub fn classify_index_lock(evidence: &Evidence) -> Verdict {
    if let Some(scan) = &evidence.holders
        && let Some(holder) = scan.holders.first()
    {
        return Verdict::ProvenAlive {
            pid: holder.pid,
            by_descriptor: true,
            command: holder.command.clone(),
        };
    }
    // A holder the scan could not identify may be alive, so a pid file's absent number is no proof the owner is
    // gone: the commit waits to its ceiling, as it does for a live holder (cross-vendor review of 2026-11-09, finding 4).
    match &evidence.pid_file {
        PidFile::Owner(pid, process) if process.names_git() => {
            return Verdict::ProvenAlive {
                pid: *pid,
                by_descriptor: false,
                command: process.command.clone(),
            };
        }
        // A number this table does not have is read as dead only where the scan identified every descriptor that names
        // the lock: Git records no PID namespace, so the writer may be a live Git of another one holding the lock through
        // its editor with no descriptor open (cross-vendor review of 2026-11-09, findings 2 and 4). A committing service
        // waits for such an owner instead of acting on this verdict.
        PidFile::Owner(pid, process) if process.gone() && !evidence.holders.as_ref().is_some_and(HolderEvidence::unresolved) => {
            return Verdict::Dead(*pid);
        }
        _ => return Verdict::EvidenceFree,
    }
}

/// What:
///  Gather one attempt's evidence with the given start-time source and holder scan.
/// Why:
///   `gatherIndexLockEvidence`;
///  nothing when the lock is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function gatherIndexLockEvidence({ realIndexPath, sources }): Promise<IndexLockEvidence | typeof LOCK_ABSENT>;
/// ```
pub fn gather_with(
    real_index: &Path,
    resolve: fn(i64) -> PidProcess,
    scan: fn(&Path, u64, u64) -> HolderEvidence,
) -> std::io::Result<Option<Evidence>> {
    let mut name: std::ffi::OsString = real_index.as_os_str().to_owned();
    name.push(".lock");
    let lock_path: PathBuf = PathBuf::from(name);
    let Some(lock) = read_lock_metadata(lock_path.as_path())? else {
        return Ok(None);
    };
    let pid_file: PidFile = read_pid_file_evidence(lock_pid_path(real_index).as_path(), resolve);
    if let PidFile::Owner(pid, process) = &pid_file
        && process.names_git()
    {
        crate::diagnostic_log::debug(
            "gatherIndexLockEvidence",
            format!("PID file proves live owner {pid}").as_str(),
        );
        return Ok(Some(Evidence { lock_path, lock, pid_file, holders: None }));
    }
    let holders: HolderEvidence = scan(lock_path.as_path(), lock.device, lock.inode);
    return Ok(Some(Evidence {
        lock_path,
        lock,
        pid_file,
        holders: Some(holders),
    }));
}

/// What:
///  Gather one attempt's evidence with the host's sources.
/// Why:
///   The production gather.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await gatherIndexLockEvidence({ realIndexPath })
/// ```
pub fn gather_index_lock_evidence(real_index: &Path) -> std::io::Result<Option<Evidence>> {
    return gather_with(real_index, host_pid_process, scan_platform_holders);
}

/// Evidence controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_evidence_tests.rs"]
mod tests;

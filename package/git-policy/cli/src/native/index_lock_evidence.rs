//! What:
//!  Evidence about a foreign `index.lock` and the verdict it supports:
//!  proven alive,
//!  dead,
//!       or evidence-free.
//! Why:
//!  A live owner is proven by a process holding the lock open (device and inode),
//!  or by Git's
//!      `core.lockfilePid` file naming a running process that started no later than the lock
//!      changed.
//!  A PID file naming a gone process proves a dead owner.
//!  Everything else is
//!      evidence-free;
//!  an absent open holder never proves abandonment,
//!  because native
//!      `git commit` keeps `index.lock` without a descriptor through its hooks and editor
//!      (`src/index-lock/index-lock-evidence.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const evidence = await gatherIndexLockEvidence({ realIndexPath }); classifyIndexLock(evidence);
//! ```

/// Open-holder scans.
use super::index_lock_holders::{HolderEvidence, scan_platform_holders};
/// The variable that sets how long the wait for a live holder lasts.
use super::live_lock::LIVE_WAIT_VARIABLE;
/// ISO timestamps for diagnostics.
use super::iso_time::format_iso_milliseconds;
/// Record writers.
use super::json_record::quote;
/// Process start times.
use super::process_start::{ProcessStart, resolve_process_start};
/// Git's PID file path.
use super::recovery_files::lock_pid_path;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Longest PID file text quoted in evidence,
///  in UTF-16 code units for ASCII text.
const QUOTED_TEXT_LIMIT: usize = 64;

/// Partial-evidence reasons quoted before the rest are summarized.
const QUOTED_REASON_LIMIT: usize = 3;

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
///  now.
/// Why:
///   `PidOwnerEvidence`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PidOwnerEvidence = { state: 'missing' } | { state: 'started-before-lock'; startedAtMs } | ...;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum PidOwner {
    /// No such process runs,
    ///  or it is a zombie.
    Missing,
    /// A process runs that started no later than the lock changed.
    StartedBeforeLock(
        /// Start time in milliseconds since the epoch.
        f64,
    ),
    /// A process runs that started after the lock changed:
    ///  the PID was reused.
    StartedAfterLock(
        /// Start time in milliseconds since the epoch.
        f64,
    ),
    /// A process runs whose start time is unreadable.
    StartUnknown(
        /// Why the start time could not be read.
        String,
    ),
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
        PidOwner,
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
    /// The PID file names a process that no longer runs.
    Dead(
        /// The PID of the process that no longer runs.
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
    let trimmed: &str = super::js_text::trim_javascript_end(text);
    let digits: &str = trimmed.strip_prefix("pid ")?;
    return super::recovery_evidence::decimal_attempt(digits);
}

/// What:
///  The owner evidence of a PID's start against the lock's change time.
/// Why:
///   `ownerEvidence`:
///  started no later than the change plus the start's resolution.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerEvidence({ start, lock }): PidOwnerEvidence;
/// ```
pub fn owner_evidence(start: ProcessStart, lock: &LockMetadata) -> PidOwner {
    match start {
        ProcessStart::Missing => return PidOwner::Missing,
        ProcessStart::Unknown(reason) => return PidOwner::StartUnknown(reason),
        ProcessStart::Running { started_at_ms, resolution_ms } => {
            if started_at_ms <= (lock.ctime_ms + resolution_ms) as f64 {
                return PidOwner::StartedBeforeLock(started_at_ms);
            }
            return PidOwner::StartedAfterLock(started_at_ms);
        }
    }
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
///  the text is read with replacement characters as `'utf8'` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readPidFileEvidence({ pidPath, lock, resolveStart }): Promise<PidFileEvidence>;
/// ```
pub fn read_pid_file_evidence(pid_path: &Path, lock: &LockMetadata, resolve: fn(i64) -> ProcessStart) -> PidFile {
    let text: String = match std::fs::read(pid_path) {
        Ok(bytes) => String::from_utf8_lossy(bytes.as_slice()).into_owned(),
        Err(error) if is_missing(&error) => return PidFile::Absent,
        Err(error) => return PidFile::Unreadable(error.to_string()),
    };
    let Some(pid) = parse_pid_file_text(text.as_str()) else {
        return PidFile::Malformed(quoted_prefix(text.as_str()));
    };
    return PidFile::Owner(pid, owner_evidence(resolve(pid), lock));
}

/// What:
///  Classify gathered evidence.
/// Why:
///   `classifyIndexLock`:
///  an open holder first,
///  then a PID file of a process that started
///       before the lock,
///  then one naming a gone process.
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
    match &evidence.pid_file {
        PidFile::Owner(pid, PidOwner::StartedBeforeLock(_)) => {
            return Verdict::ProvenAlive {
                pid: *pid,
                by_descriptor: false,
                command: None,
            };
        }
        PidFile::Owner(pid, PidOwner::Missing) => return Verdict::Dead(*pid),
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
    resolve: fn(i64) -> ProcessStart,
    scan: fn(&Path, u64, u64) -> HolderEvidence,
) -> std::io::Result<Option<Evidence>> {
    let mut name: std::ffi::OsString = real_index.as_os_str().to_owned();
    name.push(".lock");
    let lock_path: PathBuf = PathBuf::from(name);
    let Some(lock) = read_lock_metadata(lock_path.as_path())? else {
        return Ok(None);
    };
    let pid_file: PidFile = read_pid_file_evidence(lock_pid_path(real_index).as_path(), &lock, resolve);
    if let PidFile::Owner(pid, PidOwner::StartedBeforeLock(_)) = &pid_file {
        super::diagnostic_log::debug(
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
    return gather_with(real_index, resolve_process_start, scan_platform_holders);
}

/// What:
///  The PID file evidence as prose.
/// Why:
///   `describePidFile`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describePidFile(pidFile: PidFileEvidence): string;
/// ```
pub fn describe_pid_file(pid_file: &PidFile) -> String {
    match pid_file {
        PidFile::Absent => return String::from("no PID file (the owner did not run with core.lockfilePid=true)"),
        PidFile::Malformed(text) => return format!("a malformed PID file {}", quote(text.as_str())),
        PidFile::Unreadable(reason) => return format!("an unreadable PID file ({reason})"),
        PidFile::Owner(pid, owner) => {
            let named: String = format!("PID file names PID {pid}");
            match owner {
                PidOwner::Missing => return format!("{named}, which no longer runs"),
                PidOwner::StartedAfterLock(started) => {
                    return format!(
                        "{named}, now a process started at {}, after the lock changed, so the PID was reused",
                        format_iso_milliseconds(*started as i64)
                    );
                }
                PidOwner::StartUnknown(reason) => {
                    return format!("{named}, which runs but whose start time is unreadable ({reason})");
                }
                PidOwner::StartedBeforeLock(_) => return format!("{named}, which runs and started before the lock changed"),
            }
        }
    }
}

/// What:
///  The holder scan as prose.
/// Why:
///   `describeHolders`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeHolders(evidence: IndexLockEvidence): string;
/// ```
pub fn describe_holders(evidence: &Evidence) -> String {
    let Some(scan) = &evidence.holders else {
        return String::from("no open-holder scan");
    };
    let found: String = if scan.holders.is_empty() {
        String::from("no process holding it open")
    } else {
        let mut names: Vec<String> = Vec::new();
        for holder in &scan.holders {
            names.push(holder_name(holder.pid, holder.command.as_deref()));
        }
        names.join(", ")
    };
    let partial: String = if scan.partial.is_empty() {
        String::new()
    } else {
        let quoted: &[String] = &scan.partial[..scan.partial.len().min(QUOTED_REASON_LIMIT)];
        format!(
            "; {} partial-evidence note(s): {}{}",
            scan.partial.len(),
            quoted.join("; "),
            if scan.partial.len() > quoted.len() { "; ..." } else { "" }
        )
    };
    return format!("{} scan found {found}{partial}", scan.method);
}

/// What:
///  `PID <n>` with the command in parentheses when known.
/// Why:
///   Shared by the evidence and the waiting line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `PID ${pid}${command === undefined ? '' : ` (${command})`}`
/// ```
fn holder_name(pid: i64, command: Option<&str>) -> String {
    match command {
        Some(name) => return format!("PID {pid} ({name})"),
        None => return format!("PID {pid}"),
    }
}

/// What:
///  The evidence as one line of prose.
/// Why:
///   `describeIndexLockEvidence`,
///  quoted in the unproven-owner failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeIndexLockEvidence(evidence: IndexLockEvidence): string;
/// ```
pub fn describe_index_lock_evidence(evidence: &Evidence) -> String {
    return format!(
        "{} (device {}, inode {}, changed {}); {}; {}",
        evidence.lock_path.display(),
        evidence.lock.device,
        evidence.lock.inode,
        format_iso_milliseconds(evidence.lock.ctime_ms),
        describe_pid_file(&evidence.pid_file),
        describe_holders(evidence)
    );
}

/// What:
///  The one standard-error line naming a proven-alive holder.
/// Why:
///   `provenHolderLine`:
///  the person learns what the wait is for and where it gives up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderLine({ verdict, lockPath, limitSeconds }): string;
/// ```
pub fn proven_holder_line(
    pid: i64,
    by_descriptor: bool,
    command: Option<&str>,
    lock_path: &Path,
    limit_seconds: u64,
) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "cli-git: waiting for {}, which holds {} (proven by {proof}); giving up after {limit_seconds} seconds \
         ({LIVE_WAIT_VARIABLE} changes this), leaving the lock in place.\n",
        holder_name(pid, command),
        lock_path.display()
    );
}

/// What:
///  The line that repeats while a proven-alive holder keeps the lock.
/// Why:
///   Each repeat names the holder, the proof, how long the wait has lasted and where it gives up,
///  so a reader of the last line alone knows what is going on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderProgressLine({ verdict, lockPath, waitedSeconds, limitSeconds }): string;
/// ```
pub fn proven_holder_progress_line(
    pid: i64,
    by_descriptor: bool,
    command: Option<&str>,
    lock_path: &Path,
    waited_seconds: u64,
    limit_seconds: u64,
) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "cli-git: still waiting for {}, which holds {} (proven by {proof}) after {waited_seconds} seconds; giving up \
         after {limit_seconds} seconds ({LIVE_WAIT_VARIABLE} changes this), leaving the lock in place.\n",
        holder_name(pid, command),
        lock_path.display()
    );
}

/// Evidence controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_evidence_tests.rs"]
mod tests;

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
use crate::index_lock_holders::{HolderEvidence, scan_platform_holders};
/// Process start times.
use crate::process_start::{ProcessStart, resolve_process_start};
/// Git's PID file path.
use crate::recovery_files::lock_pid_path;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Longest PID file text quoted in evidence,
///  in UTF-16 code units for ASCII text.
///  Exempt from a citation under `LQM` (class: display cap): it limits how much text a message, log field or record
///  names, and no command result or duration depends on it. Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation, open to the owner's veto.
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
    let trimmed: &str = crate::js_text::trim_javascript_end(text);
    let digits: &str = trimmed.strip_prefix("pid ")?;
    return crate::js_text::decimal_attempt(digits);
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
    return gather_with(real_index, resolve_process_start, scan_platform_holders);
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
///  The one standard-error line naming a proven-alive holder.
/// Why:
///   `provenHolderLine`:
///  the person learns what the wait is for and that it lasts as long as the holder holds the lock; a holder proven alive
///  will let go, so no time is named at which the wait ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderLine({ verdict, lockPath }): string;
/// ```
pub fn proven_holder_line(pid: i64, by_descriptor: bool, command: Option<&str>, lock_path: &Path) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "monochromatic-git-policy-cli: waiting for {}, which holds {} (proven by {proof}); this waits for as long as that process holds the lock \
         and leaves the lock in place.\n",
        holder_name(pid, command),
        lock_path.display()
    );
}

/// What:
///  The line that repeats while a proven-alive holder keeps the lock.
/// Why:
///   Each repeat names the holder, the proof and how long the wait has lasted, so a reader of the last line alone knows what
///  is going on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderProgressLine({ verdict, lockPath, waitedSeconds }): string;
/// ```
pub fn proven_holder_progress_line(pid: i64, by_descriptor: bool, command: Option<&str>, lock_path: &Path, waited_seconds: u64) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "monochromatic-git-policy-cli: still waiting for {}, which holds {} (proven by {proof}) after {waited_seconds} seconds; this waits for as \
         long as that process holds the lock and leaves the lock in place.\n",
        holder_name(pid, command),
        lock_path.display()
    );
}

/// What:
///  The program name of a process, from its `comm`, when it can be read.
/// Why:
///   A holder proven by Git's PID file has no command in the evidence, and the person needs the program to know what to end.
///  Only the name is read, never the arguments or the environment, which can hold a token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(`/proc/${pid}/comm`, 'utf8').trim()
/// ```
fn program_of(pid: i64) -> Option<String> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    let name: &str = text.trim();
    return if name.is_empty() { None } else { Some(name.to_string()) };
}

/// What:
///  The sentence that names a live holder by its number, program and start time, and the way out if it is stuck.
/// Why:
///   On 2026-11-07 four commits waited a combined hour for a lock held by a live `git commit` and were told only that the owner
///  "cannot be proven alive". A person who reads PID, program, start time and the command that ends the process can decide in
///  seconds; the wrapper itself never ends a process or removes a lock whose holder runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holderWayOut({ pid, command }): string; // 'The holder is PID 5 (git), started 2026-11-07T18:04:20.000Z; ...'
/// ```
pub fn holder_way_out(pid: i64, command: Option<&str>) -> String {
    let program: String = match command.map(str::to_string).or_else(|| program_of(pid)) {
        Some(name) => name,
        None => String::from("program unknown"),
    };
    let started: String = match resolve_process_start(pid) {
        ProcessStart::Running { started_at_ms, .. } => format!("started {}", crate::iso_time::format_iso_milliseconds(started_at_ms as i64)),
        ProcessStart::Missing => String::from("start time unknown, it has just ended"),
        ProcessStart::Unknown(_) => String::from("start time unknown"),
    };
    return format!(
        "The holder is PID {pid} ({program}), {started}; if it is stuck, `kill {pid}` ends it and Git removes its lock, and a commit still waiting then lands.\n"
    );
}

/// What:
///  The reasons a scan could not see everything, as one short text: every reason when there are few, otherwise their number, the
///  first of them and the one that says the scan ran out of time.
/// Why:
///   On 2026-11-07 the line named every process the account could not look at (hundreds of rootless-container processes the
///  account's descriptors do not reach), which buried the one fact a person needs: the holder may be among them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// partialSummary(['PID 4: Permission denied', ...]) // '412 reasons, the first: PID 4: Permission denied'
/// ```
fn partial_summary(reasons: &[String]) -> String {
    const SHOWN_IN_FULL: usize = 3;
    if reasons.len() <= SHOWN_IN_FULL {
        return reasons.join("; ");
    }
    let mut text: String = format!("{} processes could not be looked at, the first of them: {}", reasons.len(), reasons[0]);
    if let Some(cut) = reasons.iter().find(|reason| { return reason.contains("time bound"); }) {
        text.push_str("; ");
        text.push_str(cut.as_str());
    }
    return text;
}

/// What the evidence says about the lock's owner, in words that fit after "found".
fn evidence_words(evidence: &Evidence) -> String {
    let descriptors: String = match &evidence.holders {
        Some(scan) if scan.partial.is_empty() => String::from("no process has the lock open"),
        Some(scan) => format!("no process the scan could see has the lock open (the scan could not see everything: {})", partial_summary(scan.partial.as_slice())),
        None => String::from("the open descriptors were not scanned"),
    };
    let pid_file: String = match &evidence.pid_file {
        PidFile::Absent => String::from("no Git lock PID file beside it (`core.lockfilePid` is off for the program that made it, or it is not Git)"),
        PidFile::Malformed(text) => format!("a Git lock PID file whose text is not `pid <number>` ({text:?})"),
        PidFile::Unreadable(reason) => format!("a Git lock PID file that could not be read ({reason})"),
        PidFile::Owner(pid, PidOwner::StartedAfterLock(_)) => format!("a Git lock PID file naming PID {pid}, which started after the lock was made (the number was reused)"),
        PidFile::Owner(pid, PidOwner::StartUnknown(reason)) => format!("a Git lock PID file naming PID {pid}, whose start time could not be read ({reason})"),
        PidFile::Owner(pid, PidOwner::Missing) => format!("a Git lock PID file naming PID {pid}, which no longer runs"),
        PidFile::Owner(pid, PidOwner::StartedBeforeLock(_)) => format!("a Git lock PID file naming PID {pid}, which runs"),
    };
    return format!("{descriptors}, and {pid_file}");
}

/// What:
///  The line that says a lock's owner cannot be proven alive, what is known about it, and what a person can do.
/// Why:
///   An owner nothing proves alive is waited for like one proven alive, with no bound, because the person's own `git commit`
///  with its editor open holds the lock exactly so; the line says what was looked at and names the file to remove if no program
///  is using the repository, which is the only case the wait cannot end by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unprovenWaitLine({ lockPath, evidence }): string;
/// ```
pub fn unproven_wait_line(lock_path: &Path, evidence: Option<&Evidence>) -> String {
    let known: String = match evidence {
        Some(found) => format!(" ({})", evidence_words(found)),
        None => String::from(" (the evidence about its owner could not be read)"),
    };
    return format!(
        "monochromatic-git-policy-cli: waiting for {lock}, whose owner cannot be proven alive{known}; this waits for as long as the lock stays there. \
         If no program is using this repository, the lock was left by a program that was killed, and removing the file {lock} \
         lets the command run.\n",
        lock = lock_path.display()
    );
}

/// What:
///  The line that names who still holds a lock when the wait for it reaches its ceiling, and what becomes of the command.
/// Why:
///   The owner's rule is that nothing waits for ever: at the bound the command stops and says what blocked it, with the evidence
///  there is, and what to run again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ceilingLine({ lockPath, blocker, waitedSeconds, consequence }): string;
/// ```
pub fn ceiling_line(lock_path: &Path, blocker: &str, waited_seconds: u64, consequence: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {lock} is still held after {waited_seconds} seconds by {blocker}, so {consequence}; nothing was changed and the lock was \
         left in place. Run the command again once that program is done.\n",
        lock = lock_path.display()
    );
}

/// What:
///  The sentence that names the lock file and the fix when a wait ended at its ceiling on an owner nothing proves alive.
/// Why:
///   A lock without its marker is what plain Git leaves when it is killed holding it, and what a kill of the service leaves on a
///  filesystem that places no file whole; nothing here removes it, because it may be a program that is still at work, so the
///  person who knows no program is using the repository needs the file's name and the command to clear it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unprovenRemovalAdvice(lockPath: string): string;
/// ```
pub fn unproven_removal_advice(lock_path: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: nothing proves a program is still using {lock}. If no program is using this repository, the lock was left by a program that was killed, \
         and removing the file {lock} lets the command run.\n",
        lock = lock_path.display()
    );
}

/// What:
///  The words for the owner of a lock that is held, from the evidence: a process proven alive, or an owner nothing proves alive
///  with what was looked at.
/// Why:
///   The line at the ceiling names the blocker, whatever the proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blockerWords(evidence, verdict): string;
/// ```
pub fn blocker_words(evidence: &Evidence) -> String {
    return match classify_index_lock(evidence) {
        Verdict::ProvenAlive { pid, by_descriptor, command } => {
            let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
            format!("{} (proven by {proof})", holder_name(pid, command.as_deref()))
        }
        Verdict::Dead(pid) => format!("{}, which no longer runs", holder_name(pid, None)),
        Verdict::EvidenceFree => format!("an owner nothing proves alive ({})", evidence_words(evidence)),
    };
}

/// What:
///  The diagnostic for a lock whose Git lock PID file names a process that no longer runs.
/// Why:
///   That owner can never let go, so there is nothing to wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadOwnerDiagnostic({ lockPath, pid, consequence }): string;
/// ```
pub fn dead_owner_diagnostic(lock_path: &Path, pid: i64, consequence: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {lock} was left by PID {pid}, which no longer runs, so nothing will release it. {consequence}; nothing was \
         changed and the lock was left in place. If no other program is using this repository, removing the file {lock} lets \
         the command run.\n",
        lock = lock_path.display()
    );
}

/// Evidence controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_evidence_tests.rs"]
mod tests;

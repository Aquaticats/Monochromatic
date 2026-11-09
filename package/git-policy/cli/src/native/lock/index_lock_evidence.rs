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
/// The sentences about a live holder and the summary of what a scan could not see.
pub use crate::index_lock_holder_words::holder_way_out;
use crate::index_lock_holder_words::partial_summary;
/// Git's PID file path.
use crate::recovery_files::lock_pid_path;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// How long a span lasts.
use std::time::Duration;

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
///  The end of a wait,
/// as a notice line about it says it:
/// the longest the wait lasts,
/// and what becomes of the command when that time is up.
/// Why:
///   The owner's rule is that nothing waits for ever (2026-11-03:
/// "nothing should ever wait forever or
///  indefinitely"),
/// so a line that describes a wait says when the wait ends and what happens then;
/// the ceiling comes from the constant
///  the wait itself runs on,
/// so the line and the wait cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WaitEnd { within: Duration; ending: string }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct WaitEnd<'a> {
    /// The longest the wait lasts.
    pub within: Duration,
    /// What becomes of the command when that time is up.
    pub ending: &'a str,
}

/// What:
///  The one standard-error line naming a proven-alive holder.
/// Why:
///   `provenHolderLine`:
///  the person learns what the wait is for and which process holds the lock;
/// the line names the end of the wait
///  as well,
/// because a holder proven alive is waited for until `end.within` and not for as long as it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderLine({ verdict, lockPath, end }): string;
/// ```
pub fn proven_holder_line(pid: i64, by_descriptor: bool, command: Option<&str>, lock_path: &Path, end: WaitEnd<'_>) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "monochromatic-git-policy-cli: waiting for {}, which holds {} (proven by {proof}); this waits until that process lets go, for at most {} seconds; \
         after that, {}, and the lock is left in place.\n",
        holder_name(pid, command),
        lock_path.display(),
        end.within.as_secs(),
        end.ending
    );
}

/// What:
///  The line that repeats while a proven-alive holder keeps the lock.
/// Why:
///   Each repeat names the holder,
/// the proof and how long the wait has lasted,
/// so a reader of the last line alone knows what
///  is going on,
/// and the repeat names the end of the wait too,
/// so the last line alone also says how long the wait can last and what happens then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderProgressLine({ verdict, lockPath, waitedSeconds, end }): string;
/// ```
pub fn proven_holder_progress_line(pid: i64, by_descriptor: bool, command: Option<&str>, lock_path: &Path, waited_seconds: u64, end: WaitEnd<'_>) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "monochromatic-git-policy-cli: still waiting for {}, which holds {} (proven by {proof}) after {waited_seconds} seconds; this waits until that process \
         lets go, for at most {} seconds in all; after that, {}, and the lock is left in place.\n",
        holder_name(pid, command),
        lock_path.display(),
        end.within.as_secs(),
        end.ending
    );
}

/// What the evidence says about the lock's owner,
/// in words that fit after "found".
fn evidence_words(evidence: &Evidence) -> String {
    let descriptors: String = match &evidence.holders {
        Some(scan) if !scan.unresolvable.is_empty() => format!(
            "a descriptor that names the lock could not be read ({})",
            partial_summary(scan.unresolvable.as_slice())
        ),
        Some(scan) if scan.partial.is_empty() => String::from("no process has the lock open"),
        Some(scan) => format!("no process the scan could see has the lock open (the scan could not see everything: {})", partial_summary(scan.partial.as_slice())),
        None => String::from("the open descriptors were not scanned"),
    };
    let pid_file: String = match &evidence.pid_file {
        PidFile::Absent => String::from("no Git lock PID file beside it (`core.lockfilePid` is off for the program that made it, or it is not Git)"),
        PidFile::Malformed(text) => format!("a Git lock PID file whose text is not `pid <number>` ({text:?})"),
        PidFile::Unreadable(reason) => format!("a Git lock PID file that could not be read ({reason})"),
        PidFile::Owner(pid, process) if process.gone() => format!(
            "a Git lock PID file naming PID {pid}, which this process table does not have (a Git of another PID namespace writes a number this table has never had, \
             and it holds the lock through its editor with no descriptor open)"
        ),
        PidFile::Owner(pid, process) if process.starts_after(&evidence.lock) => format!(
            "a Git lock PID file naming PID {pid} ({}), which runs, and which this host's clock puts after the lock was made, which a reused number would explain and so would a clock that moved",
            process.command.as_deref().unwrap_or("command name unknown")
        ),
        PidFile::Owner(pid, process) => format!("a Git lock PID file naming PID {pid} ({}), which runs", process.command.as_deref().unwrap_or("command name unknown")),
    };
    return format!("{descriptors}, and {pid_file}");
}

/// What:
///  The line that says a lock's owner cannot be proven alive,
/// what is known about it,
/// how long the wait lasts,
/// what becomes of the command then,
/// and what a person can do.
/// Why:
///   An owner nothing proves alive is waited for like one proven alive,
/// until the wait's ceiling (`MERGE_LOCK_CEILING` in `index_holder_wait.rs`,
/// `LANDING_LOCK_CEILING` in `service_index_attempts.rs`),
/// because the person's own `git commit`
///  with its editor open holds the lock exactly so;
/// the line says what was looked at and names the file to remove if no program
///  is using the repository,
/// which is the only case the wait cannot end by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unprovenWaitLine({ lockPath, evidence, end }): string;
/// ```
pub fn unproven_wait_line(lock_path: &Path, evidence: Option<&Evidence>, end: WaitEnd<'_>) -> String {
    let known: String = match evidence {
        Some(found) => format!(" ({})", evidence_words(found)),
        None => String::from(" (the evidence about its owner could not be read)"),
    };
    return format!(
        "monochromatic-git-policy-cli: waiting for {lock}, whose owner cannot be proven alive{known}; this waits until the lock goes, for at most {} seconds; \
         after that, {}. If no program is using this repository, the lock was left by a program that was killed, and removing the file {lock} \
         lets the command run.\n",
        end.within.as_secs(),
        end.ending,
        lock = lock_path.display()
    );
}

/// What:
///  The line that names who still holds a lock when the wait for it reaches its ceiling,
/// and what becomes of the command.
/// Why:
///   The owner's rule is that nothing waits for ever:
/// at the bound the command stops and says what blocked it,
/// with the evidence
///  there is,
/// and what to run again.
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
///   A lock without its marker is what plain Git leaves when it is killed holding it,
/// and what a kill of the service leaves on a
///  filesystem that places no file whole;
/// nothing here removes it,
/// because it may be a program that is still at work,
/// so the
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
///  The words for the owner of a lock that is held,
/// from the evidence:
/// a process proven alive,
/// or an owner nothing proves alive
///  with what was looked at.
/// Why:
///   The line at the ceiling names the blocker,
/// whatever the proof.
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
        Verdict::Dead(pid) => format!("{} (a number this process table does not have)", holder_name(pid, None)),
        Verdict::EvidenceFree => format!("an owner nothing proves alive ({})", evidence_words(evidence)),
    };
}

/// The sentence that ends a dead owner's line when the lock is left where it is,
/// which is what every command but a merge does.
pub const DEAD_OWNER_LOCK_LEFT: &str = "nothing was changed and the lock was left in place. If no other program is using this repository, removing the file lets the command run";

/// What:
///  The diagnostic for a lock whose Git lock PID file names a number this process table does not have,
///  ending in what the command does about it.
/// Why:
///   That owner can never let go,
/// so there is nothing to wait for;
/// a commit service and a merge each say what they do
///  instead,
/// and a removal that only some commands attempt must not be promised by the others.
///  The number is said for what it is:
///  a Git of another PID namespace writes one this table has never had,
///  so the diagnostic claims no proof that the owner is gone
///  (cross-vendor review of 2026-11-09,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadOwnerDiagnostic({ lockPath, pid, consequence, ending }): string;
/// ```
pub fn dead_owner_diagnostic(lock_path: &Path, pid: i64, consequence: &str, ending: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {lock} holds the PID number {pid}, which this process table does not have (a Git of another PID namespace writes a number this table \
         has never had), so no process here will release it. {consequence}; {ending}.\n",
        lock = lock_path.display()
    );
}

/// Evidence controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_evidence_tests.rs"]
mod tests;

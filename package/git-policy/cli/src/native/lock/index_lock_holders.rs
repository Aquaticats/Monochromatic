//! What:
//!  Processes holding a lock file open,
//!  found by device and inode:
//!  `/proc/<pid>/fd` on
//!       Linux,
//!  `lsof` on macOS,
//!  the Restart Manager on Windows.
//! Why:
//!  An open descriptor proves a live owner.
//!  Matching by the device number and the inode the file has,
//!  never by path text,
//!      means a renamed path cannot fake a holder,
//!      a bind mount cannot hide one,
//!  and two files of two btrfs subvolumes are told apart,
//!  which the mount number the kernel writes for a descriptor cannot do (audit of 2026-11-09,
//!  finding 4).
//!  Processes the scan
//!      cannot read are partial evidence,
//!  and they are reported as such rather than counted as absent.
//!  A process another account owns is the one such case a removal may act through
//!  (`OTHER_ACCOUNTS_PROCESS_REASON`):
//!  it cannot write into a file of this account's,
//!  which is why the commit service's lock removal tolerates it
//!  (the owner's decision of 2026-11-07,
//! "Service removes it").
//!  A process of this account whose descriptors the kernel refuses may hold the file,
//!  and a caller that would unlink it must keep it
//!  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
//! its first finding).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const evidence = await scanPlatformHolders({ lockPath, lock });
//! ```

/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::Path;
/// `PathBuf` is an owned filesystem path,
///  used by the Unix proc scan.
#[cfg(unix)]
use std::path::PathBuf;

/// How a descriptor's open file is read:
/// the kernel's copy of the descriptor in a judgement,
/// or a control's own read of a link for a table the kernel does not know.
#[cfg(unix)]
pub type OpenedFileReader = fn(&Path, &std::ffi::OsStr) -> std::io::Result<crate::open_file_of_a_descriptor::OpenedFile>;

/// The incumbent's Restart Manager script,
///  byte for byte.
pub const RESTART_MANAGER_SCRIPT: &str = include_str!("index_lock_restart_manager.ps1");

/// Partial-evidence note every unprivileged macOS scan carries.
pub const SILENT_DENIAL_NOTE: &str = "lsof omits other users' processes on macOS without reporting it";

/// What:
///  One process holding the lock open.
/// Why:
///   `LockHolderProcess`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockHolderProcess = { pid: number; command?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Holder {
    /// Holder process ID.
    pub pid: i64,
    /// Holder command name when readable.
    pub command: Option<String>,
}

/// What:
///  The scan mechanism,
///  its holders,
///  and what it could not see.
/// Why:
///   `LockHolderEvidence`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockHolderEvidence = { method: 'proc-fd' | 'lsof' | 'restart-manager' | 'unsupported'; holders; partial };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderEvidence {
    /// Scan mechanism name.
    pub method: &'static str,
    /// Processes holding the lock open.
    pub holders: Vec<Holder>,
    /// Reasons the scan could not see everything.
    pub partial: Vec<String>,
    /// Reasons a descriptor whose record names the lock could not be identified,
    /// because the kernel refused to copy it into this process:
    /// a removal that acts on a scan with one of these is acting on a scan that proved nothing.
    pub unresolvable: Vec<String>,
}

impl HolderEvidence {
    /// Whether the scan reported a descriptor whose record named the lock and which it could not identify,
    /// which is a holder that may be alive.
    pub fn unresolved(&self) -> bool {
        return !self.unresolvable.is_empty();
    }
}

/// What:
///  Whether a directory name is a decimal PID.
/// Why:
///   `/proc` also holds `self`,
///  `sys` and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.length > 0 && isAsciiDigits(name)
/// ```
#[cfg(unix)]
fn is_pid_name(name: &str) -> bool {
    return !name.is_empty() && name.bytes().all(is_digit);
}

/// What:
///  Whether a byte is an ASCII digit.
/// Why:
///   A named predicate keeps the scans free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte >= 0x30 && byte <= 0x39
/// ```
#[cfg(unix)]
fn is_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
///  Whether an error means the process exited during the scan.
/// Why:
///   `ENOENT` and `ESRCH` are expected races,
///  never partial evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['ENOENT', 'ESRCH'].includes(error.code)
/// ```
#[cfg(unix)]
fn exited(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::NotFound {
        return true;
    }
    #[cfg(unix)]
    {
        return error.raw_os_error() == Some(libc::ESRCH);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// What:
///  Whether one process holds the file open,
///  and why its descriptors were unreadable.
/// Why:
///   `scanProcess`:
///  the first unreadable descriptor's reason is kept,
///  and the identity test is the one the scan of a process tree uses,
///  so one file has one test everywhere:
///  the device number and the inode,
///  against a descriptor's record as the cheap test and its own open link as the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanProcess({ procRoot, pid, device, inode }): Promise<{ holds: boolean; partial?: string }>;
/// ```
#[cfg(unix)]
fn scan_process(proc_root: &Path, pid: &str, device: u64, inode: u64, opened: OpenedFileReader) -> (bool, Option<String>, Option<String>) {
    let process: PathBuf = proc_root.join(pid);
    let listing: std::fs::ReadDir = match std::fs::read_dir(process.join("fd")) {
        Ok(found) => found,
        Err(error) if exited(&error) => return (false, None, None),
        // A process of this account whose descriptors the kernel refuses may hold the file, so its reason is kept as one a
        //  removal does not act through; another account's process cannot write into a file of this account's, and that reason
        //  is the one the commit service's lock removal tolerates.
        Err(error) if crate::index_lock_holders_accounts::another_account_owns(process.as_path()) => {
            return (false, Some(format!("{}{pid}: {error}", crate::index_lock_holders_accounts::OTHER_ACCOUNTS_PROCESS_REASON)), None);
        }
        Err(error) => return (false, Some(format!("PID {pid}: {error}")), None),
    };
    let mut holds: bool = false;
    let mut partial: Option<String> = None;
    let mut unresolvable: Option<String> = None;
    for item in listing {
        let Ok(entry) = item else {
            continue;
        };
        match crate::process_use::descriptor_is_file_with(process.as_path(), entry.file_name().as_os_str(), device, inode, opened) {
            Ok(true) => holds = true,
            Ok(false) => {}
            // The record of this descriptor could not be read: the process's descriptors are what the scan could not look
            // at, which the service tolerates as it always has (the owner, 2026-11-07, "Service removes it").
            Err(crate::process_use::Unread::Process(reason)) => {
                if partial.is_none() {
                    partial = Some(format!("PID {pid} fd {}: {reason}", entry.file_name().to_string_lossy()));
                }
            }
            // The record named the file, and the file could not be identified: the process may hold it.
            Err(crate::process_use::Unread::File(reason)) => {
                if unresolvable.is_none() {
                    unresolvable = Some(format!("PID {pid} fd {}: {reason}", entry.file_name().to_string_lossy()));
                }
            }
        }
    }
    return (holds, partial, unresolvable);
}

/// What:
///  Every process other than this one holding a file open,
///  by device and inode,
///  through a
///       proc root.
/// Why:
///   `scanProcFdHolders`;
///  processes are scanned one after another (the incumbent runs 16 at
///       once;
///  the result is the same set).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanProcFdHolders({ device, inode, procRoot, selfPid }): Promise<LockHolderEvidence>;
/// ```
#[cfg(unix)]
pub fn scan_proc_fd_holders(proc_root: &Path, device: u64, inode: u64, self_pid: u32) -> HolderEvidence {
    return scan_proc_fd_holders_until(proc_root, device, inode, self_pid, None);
}

/// What:
///  `scan_proc_fd_holders` with the reader of a descriptor's open file given.
/// Why:
///   A control's `/proc` is a directory of links and its numbers are not processes the kernel copies a descriptor for,
///  so the control reads the file from the link,
///  which is the way the scan no longer reads it in a judgement
///  (`crate::process_use::descriptor_is_file_with`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanProcFdHoldersWith({ procRoot, device, inode, selfPid, opened });
/// ```
#[cfg(unix)]
pub fn scan_proc_fd_holders_with(proc_root: &Path, device: u64, inode: u64, self_pid: u32, opened: OpenedFileReader) -> HolderEvidence {
    return scan_proc_fd_holders_stopping_with(proc_root, device, inode, self_pid, ScanLimit::default(), opened);
}

/// What:
///  `scan_proc_fd_holders`,
/// stopping at `deadline` when one is given.
/// Why:
///   The commit service scans while commits queue behind it,
/// and a host with many processes must not hold them up beyond a bound:
/// the processes not reached are said in the evidence's partial reasons,
/// and the holders found so far are kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await scanProcFdHolders({ device, inode, procRoot, selfPid, deadline });
/// ```
#[cfg(unix)]
pub fn scan_proc_fd_holders_until(proc_root: &Path, device: u64, inode: u64, self_pid: u32, deadline: Option<std::time::Instant>) -> HolderEvidence {
    return scan_proc_fd_holders_stopping(proc_root, device, inode, self_pid, ScanLimit { deadline, processes: None });
}

/// The sort key that puts the highest process number first;
/// a name that is not a number sorts last.
#[cfg(unix)]
fn newest_first(entry: &(u64, String)) -> std::cmp::Reverse<u64> {
    return std::cmp::Reverse(entry.0);
}

/// Where a scan of the processes ends early:
/// at a time,
/// after a number of processes,
/// or both;
/// neither means the whole walk.
#[cfg(unix)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ScanLimit {
    /// The moment after which no further process is looked at.
    pub deadline: Option<std::time::Instant>,
    /// How many processes are looked at at most,
    /// so a control can cut a scan without racing a clock.
    pub processes: Option<usize>,
}

/// Whether a scan that has looked at `reached` processes must stop now.
#[cfg(unix)]
fn limit_reached(limit: &ScanLimit, reached: usize) -> bool {
    if let Some(most) = limit.processes
        && reached >= most
    {
        return true;
    }
    return limit.deadline.is_some_and(past);
}

/// Whether a moment is behind now.
#[cfg(unix)]
fn past(moment: std::time::Instant) -> bool {
    return std::time::Instant::now() >= moment;
}

/// What:
///  `scan_proc_fd_holders_until` with its limit given whole:
/// a time,
/// a number of processes,
/// or both.
/// Why:
///   A scan that reached the limit ends,
/// and the evidence says how far it got.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await scanProcFdHolders({ device, inode, procRoot, selfPid, limit: { processes: 5 } });
/// ```
#[cfg(unix)]
pub fn scan_proc_fd_holders_stopping(proc_root: &Path, device: u64, inode: u64, self_pid: u32, limit: ScanLimit) -> HolderEvidence {
    return scan_proc_fd_holders_stopping_with(proc_root, device, inode, self_pid, limit, crate::open_file_of_a_descriptor::opened_file_of);
}

/// What:
///  `scan_proc_fd_holders_stopping` with the reader of a descriptor's open file given.
/// Why:
///   The same seam as `scan_proc_fd_holders_with`,
///  for a control that also cuts the scan short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanProcFdHoldersStoppingWith({ procRoot, device, inode, selfPid, limit, opened });
/// ```
#[cfg(unix)]
pub fn scan_proc_fd_holders_stopping_with(proc_root: &Path, device: u64, inode: u64, self_pid: u32, limit: ScanLimit, opened: OpenedFileReader) -> HolderEvidence {
    let mut evidence: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
    let listing: std::fs::ReadDir = match std::fs::read_dir(proc_root) {
        Ok(found) => found,
        Err(error) => {
            evidence.partial.push(format!("{}: {error}", proc_root.display()));
            return evidence;
        }
    };
    let own: String = self_pid.to_string();
    let mut pids: Vec<String> = Vec::new();
    for item in listing.flatten() {
        let name: String = item.file_name().to_string_lossy().into_owned();
        if is_pid_name(name.as_str()) && name != own {
            pids.push(name);
        }
    }
    // Newest number first: the holder of a lock made a moment ago is usually a recent process, so a scan that is cut short has
    //  looked at the likeliest ones. Process numbers wrap, so this is a hedge and the bound (`SERVICE_SCAN_BUDGET`) is the fix.
    let mut numbered: Vec<(u64, String)> = Vec::new();
    for name in pids.drain(..) {
        numbered.push((name.parse::<u64>().unwrap_or(0), name));
    }
    numbered.sort_by_key(newest_first);
    for (_number, name) in numbered {
        pids.push(name);
    }
    for (reached, pid) in pids.iter().enumerate() {
        if limit_reached(&limit, reached) {
            evidence.partial.push(format!("stopped at the time bound after {reached} of {} processes", pids.len()));
            break;
        }
        let (holds, partial, unresolvable) = scan_process(proc_root, pid.as_str(), device, inode, opened);
        if holds && let Ok(number) = pid.parse::<i64>() {
            let command: Option<String> = std::fs::read_to_string(proc_root.join(pid).join("comm"))
                .ok()
                .map(trimmed);
            evidence.holders.push(Holder { pid: number, command });
        }
        if let Some(reason) = partial {
            evidence.partial.push(reason);
        }
        if let Some(reason) = unresolvable {
            evidence.unresolvable.push(reason);
        }
    }
    crate::diagnostic_log::debug(
        "scanProcFdHolders",
        format!(
            "scanned {} processes: {} holders, {} unreadable",
            pids.len(),
            evidence.holders.len(),
            evidence.partial.len()
        )
        .as_str(),
    );
    return evidence;
}

/// What:
///  Text without surrounding whitespace.
/// Why:
///   A named function keeps `map` free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (text: string) => text.trim()
/// ```
#[cfg(unix)]
fn trimmed(text: String) -> String {
    return String::from(crate::js_text::trim_javascript(text.as_str()));
}

/// How long the commit service's scan for the processes that have a lock open may take.
///
/// Measured on 2026-11-07 on this host at load average 37 to 52:
/// one walk of every process's open descriptors (list each
///  `/proc/<pid>/fd` and `stat` each link,
/// which the walk measured here does) over 1,000 processes and 54,500 descriptors took 313,
/// 360 and 398 ms.
/// The scan does less of that work now:
/// it resolves a descriptor's link only where the record already names the lock's inode number,
/// so this walk remains the upper bound.
///  The earlier budget,
/// 250 ms,
/// was measured on 2026-11-03 over 858 processes with `ls` (0.10 s) and was shorter than a whole walk
///  on today's host,
/// so a live holder past the cut read as an owner nothing proves alive.
/// Two seconds is the smallest whole number
///  of seconds that is at least five times the longest walk (1.99 s);
/// a scan that still runs out returns what it found,
/// newest
///  process first,
/// and the next look begins again.
pub const SERVICE_SCAN_BUDGET: std::time::Duration = std::time::Duration::from_secs(2);

/// The scan for open holders that the commit service makes:
/// the host's,
/// bounded in time where the host's is `/proc`.
pub fn scan_platform_holders_bounded(lock_path: &Path, device: u64, inode: u64) -> HolderEvidence {
    #[cfg(target_os = "linux")]
    {
        let _ = lock_path;
        return scan_proc_fd_holders_until(Path::new("/proc"), device, inode, std::process::id(), Some(std::time::Instant::now() + SERVICE_SCAN_BUDGET));
    }
    #[cfg(not(target_os = "linux"))]
    {
        return scan_platform_holders(lock_path, device, inode);
    }
}

/// What:
///  Scan for open holders with the host's mechanism.
/// Why:
///   `scanPlatformHolders`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanPlatformHolders({ lockPath, lock }): Promise<LockHolderEvidence>;
/// ```
pub fn scan_platform_holders(lock_path: &Path, device: u64, inode: u64) -> HolderEvidence {
    #[cfg(target_os = "linux")]
    {
        let _ = lock_path;
        return scan_proc_fd_holders(Path::new("/proc"), device, inode, std::process::id());
    }
    #[cfg(target_os = "macos")]
    {
        return scan_lsof_holders(lock_path, device, inode);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(windows)]
    {
        let _ = (device, inode);
        return scan_restart_manager_holders(lock_path);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        let _ = (lock_path, device, inode);
        return HolderEvidence {
            method: "unsupported",
            holders: Vec::new(),
            partial: vec![String::from("no open-holder scan on this platform")],
            unresolvable: Vec::new(),
        };
    }
}

/// What:
///  Holders through `lsof -n -P -w -F pcDi -- <lock>`;
///  exit 1 with output means "no match".
/// Why:
///   `scanLsofHolders`;
///  an unprivileged scan always notes that macOS hides other users.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanLsofHolders({ lockPath, device, inode }): Promise<LockHolderEvidence>;
/// ```
#[cfg(target_os = "macos")]
fn scan_lsof_holders(lock_path: &Path, device: u64, inode: u64) -> HolderEvidence {
    let output: std::io::Result<std::process::Output> = std::process::Command::new("lsof")
        .args(["-n", "-P", "-w", "-F", "pcDi", "--"])
        .arg(lock_path)
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output();
    let stdout: String = match output {
        Ok(finished) if finished.status.success() || finished.status.code() == Some(1) => {
            String::from_utf8_lossy(finished.stdout.as_slice()).into_owned()
        }
        Ok(finished) => {
            return HolderEvidence {
                method: "lsof",
                holders: Vec::new(),
                partial: vec![format!("lsof failed: exit {:?}", finished.status.code())],
                unresolvable: Vec::new(),
            };
        }
        Err(error) => {
            return HolderEvidence {
                method: "lsof",
                holders: Vec::new(),
                partial: vec![format!("lsof failed: {error}")],
                unresolvable: Vec::new(),
            };
        }
    };
    // SAFETY: `getuid` has no preconditions.
    let root: bool = unsafe { libc::getuid() } == 0;
    return HolderEvidence {
        method: "lsof",
        holders: matching_holders(parse_lsof_fields(stdout.as_str()).as_slice(), device, inode),
        partial: if root { Vec::new() } else { vec![String::from(SILENT_DENIAL_NOTE)] },
        unresolvable: Vec::new(),
    };
}

/// What:
///  Holders through the Restart Manager script,
///  the path passed in `CLI_GIT_RM_TARGET`.
/// Why:
///   `scanRestartManagerHolders`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanRestartManagerHolders(lockPath: string): Promise<LockHolderEvidence>;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn scan_restart_manager_holders(lock_path: &Path) -> HolderEvidence {
    let output: std::io::Result<std::process::Output> = std::process::Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", RESTART_MANAGER_SCRIPT])
        .env("CLI_GIT_RM_TARGET", lock_path)
        .stdin(std::process::Stdio::null())
        .output();
    match output {
        Ok(finished) if finished.status.success() => {
            return HolderEvidence {
                method: "restart-manager",
                holders: parse_restart_manager_output(String::from_utf8_lossy(finished.stdout.as_slice()).as_ref()),
                partial: Vec::new(),
                unresolvable: Vec::new(),
            };
        }
        Ok(finished) => {
            return HolderEvidence {
                method: "restart-manager",
                holders: Vec::new(),
                partial: vec![format!("Restart Manager query failed: exit {:?}", finished.status.code())],
                unresolvable: Vec::new(),
            };
        }
        Err(error) => {
            return HolderEvidence {
                method: "restart-manager",
                holders: Vec::new(),
                partial: vec![format!("Restart Manager query failed: {error}")],
                unresolvable: Vec::new(),
            };
        }
    }
}

/// Holders found by the parsers,
///  re-exported for the scans and their controls.
pub use crate::index_lock_holder_parsers::{LsofProcess, matching_holders, parse_lsof_fields, parse_lsof_number, parse_restart_manager_output};

/// Holder-scan controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_holders_tests.rs"]
mod tests;

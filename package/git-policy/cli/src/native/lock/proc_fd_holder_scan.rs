//! What:
//!  The `/proc/<pid>/fd` scan:
//!  every process of a proc root other than this one,
//!  checked for an open descriptor on one file,
//!  newest process number first,
//!  and cut short at a moment or after a number of processes when a caller says so.
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
//!  The cut exists because the commit service scans while commits queue behind it,
//!  and the processes it did not reach are said in the evidence rather than dropped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const evidence = scanProcFdHolders({ procRoot: '/proc', device, inode, selfPid });
//! ```

/// `Path` is a borrowed filesystem path and `PathBuf` its owned sibling.
use std::path::{
    Path,
    PathBuf,
};

/// The record shape every scan reports,
///  and the reader of a descriptor's open file.
use crate::holder_scan_evidence::{
    Holder,
    HolderEvidence,
    OpenedFileReader,
};
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
pub(crate) fn is_pid_name(name: &str) -> bool {
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
pub(crate) fn is_digit(byte: u8) -> bool {
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
fn scan_process(
    proc_root: &Path,
    pid: &str,
    device: u64,
    inode: u64,
    opened: OpenedFileReader,
) -> (bool, Option<String>, Option<String>) {
    let process: PathBuf = proc_root.join(pid);
    let listing: std::fs::ReadDir = match std::fs::read_dir(process.join("fd")) {
        Ok(found) => found,
        Err(error) if exited(&error) => return (false, None, None),
        // A process of this account whose descriptors the kernel refuses may hold the file, so its reason is kept as one a
        //  removal does not act through; another account's process cannot write into a file of this account's, and that reason
        //  is the one the commit service's lock removal tolerates.
        Err(error)
            if crate::index_lock_holders_accounts::another_account_owns(process.as_path()) =>
        {
            return (
                false,
                Some(format!(
                    "{}{pid}: {error}",
                    crate::index_lock_holders_accounts::OTHER_ACCOUNTS_PROCESS_REASON
                )),
                None,
            );
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
        match crate::process_use::descriptor_is_file_with(
            process.as_path(),
            entry.file_name().as_os_str(),
            device,
            inode,
            opened,
        ) {
            Ok(true) => holds = true,
            Ok(false) => {}
            // The record of this descriptor could not be read: the process's descriptors are what the scan could not look
            // at, which the service tolerates as it always has (the owner, 2026-11-07, "Service removes it").
            Err(crate::process_use::Unread::Process(reason)) => {
                if partial.is_none() {
                    partial = Some(format!(
                        "PID {pid} fd {}: {reason}",
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
            // The record named the file, and the file could not be identified: the process may hold it.
            Err(crate::process_use::Unread::File(reason)) => {
                if unresolvable.is_none() {
                    unresolvable = Some(format!(
                        "PID {pid} fd {}: {reason}",
                        entry.file_name().to_string_lossy()
                    ));
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
pub fn scan_proc_fd_holders(
    proc_root: &Path,
    device: u64,
    inode: u64,
    self_pid: u32,
) -> HolderEvidence {
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
pub fn scan_proc_fd_holders_with(
    proc_root: &Path,
    device: u64,
    inode: u64,
    self_pid: u32,
    opened: OpenedFileReader,
) -> HolderEvidence {
    return scan_proc_fd_holders_stopping_with(
        proc_root,
        device,
        inode,
        self_pid,
        ScanLimit::default(),
        opened,
    );
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
pub fn scan_proc_fd_holders_until(
    proc_root: &Path,
    device: u64,
    inode: u64,
    self_pid: u32,
    deadline: Option<std::time::Instant>,
) -> HolderEvidence {
    return scan_proc_fd_holders_stopping(
        proc_root,
        device,
        inode,
        self_pid,
        ScanLimit {
            deadline,
            processes: None,
        },
    );
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
fn limit_reached(
    limit: &ScanLimit,
    reached: usize,
) -> bool {
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
pub fn scan_proc_fd_holders_stopping(
    proc_root: &Path,
    device: u64,
    inode: u64,
    self_pid: u32,
    limit: ScanLimit,
) -> HolderEvidence {
    return scan_proc_fd_holders_stopping_with(
        proc_root,
        device,
        inode,
        self_pid,
        limit,
        crate::open_file_of_a_descriptor::opened_file_of,
    );
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
pub fn scan_proc_fd_holders_stopping_with(
    proc_root: &Path,
    device: u64,
    inode: u64,
    self_pid: u32,
    limit: ScanLimit,
    opened: OpenedFileReader,
) -> HolderEvidence {
    let mut evidence: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
        unresolvable: Vec::new(),
    };
    let listing: std::fs::ReadDir = match std::fs::read_dir(proc_root) {
        Ok(found) => found,
        Err(error) => {
            evidence
                .partial
                .push(format!("{}: {error}", proc_root.display()));
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
            evidence.partial.push(format!(
                "stopped at the time bound after {reached} of {} processes",
                pids.len()
            ));
            break;
        }
        let (holds, partial, unresolvable) =
            scan_process(proc_root, pid.as_str(), device, inode, opened);
        if holds && let Ok(number) = pid.parse::<i64>() {
            let command: Option<String> = std::fs::read_to_string(proc_root.join(pid).join("comm"))
                .ok()
                .map(trimmed);
            evidence.holders.push(Holder {
                pid: number,
                command,
            });
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

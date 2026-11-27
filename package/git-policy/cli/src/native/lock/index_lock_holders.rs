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

/// `Path` is a borrowed filesystem path,
/// and not the owned `PathBuf`.
use std::path::Path;

/// The records one scan reports and the seam a control reads a descriptor through,
///  re-exported so a caller of this module names them here.
pub use crate::holder_scan_evidence::{
    Holder,
    HolderEvidence,
    OpenedFileReader,
};

/// The `/proc` scan and the limit that cuts it short,
///  re-exported so a caller of this module names them here.
#[cfg(unix)]
pub use crate::proc_fd_holder_scan::{
    ScanLimit,
    scan_proc_fd_holders,
    scan_proc_fd_holders_stopping,
    scan_proc_fd_holders_stopping_with,
    scan_proc_fd_holders_until,
    scan_proc_fd_holders_with,
};

/// The incumbent's Restart Manager script,
///  byte for byte.
pub const RESTART_MANAGER_SCRIPT: &str = include_str!("index_lock_restart_manager.ps1");

/// Partial-evidence note every unprivileged macOS scan carries.
pub const SILENT_DENIAL_NOTE: &str =
    "lsof omits other users' processes on macOS without reporting it";

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
pub fn scan_platform_holders_bounded(
    lock_path: &Path,
    device: u64,
    inode: u64,
) -> HolderEvidence {
    #[cfg(target_os = "linux")]
    {
        let _ = lock_path;
        return scan_proc_fd_holders_until(
            Path::new("/proc"),
            device,
            inode,
            std::process::id(),
            Some(std::time::Instant::now() + SERVICE_SCAN_BUDGET),
        );
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
pub fn scan_platform_holders(
    lock_path: &Path,
    device: u64,
    inode: u64,
) -> HolderEvidence {
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
fn scan_lsof_holders(
    lock_path: &Path,
    device: u64,
    inode: u64,
) -> HolderEvidence {
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
        partial: if root {
            Vec::new()
        } else {
            vec![String::from(SILENT_DENIAL_NOTE)]
        },
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
    let output: std::io::Result<std::process::Output> =
        std::process::Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                RESTART_MANAGER_SCRIPT,
            ])
            .env("CLI_GIT_RM_TARGET", lock_path)
            .stdin(std::process::Stdio::null())
            .output();
    match output {
        Ok(finished) if finished.status.success() => {
            return HolderEvidence {
                method: "restart-manager",
                holders: parse_restart_manager_output(
                    String::from_utf8_lossy(finished.stdout.as_slice()).as_ref(),
                ),
                partial: Vec::new(),
                unresolvable: Vec::new(),
            };
        }
        Ok(finished) => {
            return HolderEvidence {
                method: "restart-manager",
                holders: Vec::new(),
                partial: vec![format!(
                    "Restart Manager query failed: exit {:?}",
                    finished.status.code()
                )],
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
pub use crate::index_lock_holder_parsers::{
    LsofProcess,
    matching_holders,
    parse_lsof_fields,
    parse_lsof_number,
    parse_restart_manager_output,
};

/// Holder-scan controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_holders_tests.rs"]
mod tests;

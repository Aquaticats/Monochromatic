//! What:
//!  Processes holding a lock file open,
//!  found by device and inode:
//!  `/proc/<pid>/fd` on
//!       Linux,
//!  `lsof` on macOS,
//!  the Restart Manager on Windows.
//! Why:
//!  An open descriptor proves a live owner.
//!  Matching by device and inode,
//!  never by path text,
//!      means bind mounts and renamed paths cannot fake or hide a holder.
//!  Processes the scan
//!      cannot read are partial evidence (`src/index-lock/index-lock-holders-linux.ts`,
//!      `-darwin.ts`,
//!  `-win32.ts`).
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
///  The device and inode a path names after following links,
///  as numbers.
/// Why:
///   A descriptor link resolves to the open file itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { dev, ino } = await stat(path, { bigint: true });
/// ```
#[cfg(unix)]
fn followed_identity(path: &Path) -> std::io::Result<(u64, u64)> {
    /// The Unix metadata accessors,
    ///  for the device and inode.
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = std::fs::metadata(path)?;
    return Ok((metadata.dev(), metadata.ino()));
}

/// What:
///  Whether one process holds the file open,
///  and why its descriptors were unreadable.
/// Why:
///   `scanProcess`:
///  the first unreadable descriptor's reason is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanProcess({ procRoot, pid, device, inode }): Promise<{ holds: boolean; partial?: string }>;
/// ```
#[cfg(unix)]
fn scan_process(proc_root: &Path, pid: &str, device: u64, inode: u64) -> (bool, Option<String>) {
    let fd_directory: PathBuf = proc_root.join(pid).join("fd");
    let listing: std::fs::ReadDir = match std::fs::read_dir(&fd_directory) {
        Ok(found) => found,
        Err(error) if exited(&error) => return (false, None),
        Err(error) => return (false, Some(format!("PID {pid}: {error}"))),
    };
    let mut holds: bool = false;
    let mut partial: Option<String> = None;
    for item in listing {
        let Ok(entry) = item else {
            continue;
        };
        match followed_identity(entry.path().as_path()) {
            Ok((found_device, found_inode)) => {
                if found_device == device && found_inode == inode {
                    holds = true;
                }
            }
            Err(error) if exited(&error) => {}
            Err(error) => {
                if partial.is_none() {
                    partial = Some(format!(
                        "PID {pid} fd {}: {error}",
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
        }
    }
    return (holds, partial);
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
    let mut evidence: HolderEvidence = HolderEvidence {
        method: "proc-fd",
        holders: Vec::new(),
        partial: Vec::new(),
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
    for pid in &pids {
        let (holds, partial) = scan_process(proc_root, pid.as_str(), device, inode);
        if holds && let Ok(number) = pid.parse::<i64>() {
            let command: Option<String> = std::fs::read_to_string(proc_root.join(pid).join("comm"))
                .ok()
                .map(trimmed);
            evidence.holders.push(Holder { pid: number, command });
        }
        if let Some(reason) = partial {
            evidence.partial.push(reason);
        }
    }
    super::diagnostic_log::debug(
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
    return String::from(super::js_text::trim_javascript(text.as_str()));
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
            };
        }
        Err(error) => {
            return HolderEvidence {
                method: "lsof",
                holders: Vec::new(),
                partial: vec![format!("lsof failed: {error}")],
            };
        }
    };
    // SAFETY: `getuid` has no preconditions.
    let root: bool = unsafe { libc::getuid() } == 0;
    return HolderEvidence {
        method: "lsof",
        holders: matching_holders(parse_lsof_fields(stdout.as_str()).as_slice(), device, inode),
        partial: if root { Vec::new() } else { vec![String::from(SILENT_DENIAL_NOTE)] },
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
            };
        }
        Ok(finished) => {
            return HolderEvidence {
                method: "restart-manager",
                holders: Vec::new(),
                partial: vec![format!("Restart Manager query failed: exit {:?}", finished.status.code())],
            };
        }
        Err(error) => {
            return HolderEvidence {
                method: "restart-manager",
                holders: Vec::new(),
                partial: vec![format!("Restart Manager query failed: {error}")],
            };
        }
    }
}

/// Holders found by the parsers,
///  re-exported for the scans and their controls.
pub use super::index_lock_holder_parsers::{LsofProcess, matching_holders, parse_lsof_fields, parse_lsof_number, parse_restart_manager_output};

/// Holder-scan controls stay out of the release executable.
#[cfg(test)]
#[path = "index_lock_holders_tests.rs"]
mod tests;

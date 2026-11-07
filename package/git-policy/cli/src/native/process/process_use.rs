//! What:
//!  The processes of this host that use a directory or a file:
//!  a working directory or an open descriptor inside the directory,
//!  or an open descriptor on the file.
//! Why:
//!  Recovery may treat a directory or a lock as abandoned only when no process can still be using it.
//!  A process that holds a path open is the one proof of use that survives a lost record,
//!  so the scan reads it from `/proc` and says what it could not read instead of guessing.
//!  Only processes of the scanning account are reported as unreadable:
//!  another account's process cannot have opened a private file this account created,
//!  and every unprivileged scan meets some of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const users = await processesUsing({ target: { inside: directory }, selfPid: process.pid });
//! ```

/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The root of the process table on Linux.
pub const PROC_ROOT: &str = "/proc";

/// What:
///  What a scan looks for.
/// Why:
///   A directory is matched by path because a record inside it may be opened under any name;
///  a file is matched by device and inode because a lock has more than one name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UseTarget = { inside: string } | { file: { device: bigint; inode: bigint } };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UseTarget<'a> {
    /// A working directory or open descriptor whose resolved path lies under this canonical directory.
    Inside(&'a Path),
    /// An open descriptor that resolves to this device and inode.
    File {
        /// Device number of the file.
        device: u64,
        /// Inode number of the file.
        inode: u64,
    },
}

/// One process found using the target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessUser {
    /// The process number.
    pub pid: i64,
    /// The command name when readable.
    pub command: Option<String>,
}

/// What:
///  The processes found using a target and the reasons the scan could not see everything.
/// Why:
///   A caller that cannot rely on a scan with no process table refuses when `unavailable` is set;
///  `unreadable` lists single processes the scan could not see into,
/// which a caller may or may not tolerate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UseScan = { users: ProcessUser[]; unreadable: string[]; unavailable?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UseScan {
    /// Processes using the target.
    pub users: Vec<ProcessUser>,
    /// Reasons some process of the scanning account could not be read.
    pub unreadable: Vec<String>,
    /// Why the process table itself could not be listed,
    /// so that no process was scanned.
    pub unavailable: Option<String>,
}

/// What:
///  Whether an error means the process exited during the scan.
/// Why:
///   `ENOENT` and `ESRCH` are expected races,
/// never unreadable evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['ENOENT', 'ESRCH'].includes(error.code)
/// ```
fn exited(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH);
}

/// What:
///  The account this process runs as.
/// Why:
///   Unreadable processes of other accounts are not evidence against a private file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.geteuid()
/// ```
#[cfg(unix)]
fn effective_user() -> u32 {
    // SAFETY: `geteuid` reads the process credentials and cannot fail.
    return unsafe { libc::geteuid() };
}

/// What:
///  Whether a path belongs to the account that runs this process.
/// Why:
///   A process directory in `/proc` belongs to the account of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await stat(path)).uid === process.geteuid()
/// ```
#[cfg(unix)]
fn belongs_to_this_account(path: &Path) -> bool {
    /// The Unix metadata accessors,
    /// for the owning account.
    use std::os::unix::fs::MetadataExt;
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    return metadata.uid() == effective_user();
}

/// What:
///  Without accounts,
/// no process is reported as unreadable evidence.
/// Why:
///   Without `/proc` the scan has already reported that it could not read the process table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// false
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn belongs_to_this_account(_path: &Path) -> bool {
    return false;
}

/// What:
///  Whether a descriptor link resolves to the file with this device and inode.
/// Why:
///   A lock has more than one name,
/// so only the identity of the open file says whether it is the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { dev, ino } = await stat(link, { bigint: true });
/// ```
#[cfg(unix)]
fn same_file(link: &Path, device: u64, inode: u64) -> std::io::Result<bool> {
    /// The Unix metadata accessors,
    /// for the device and inode.
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = std::fs::metadata(link)?;
    return Ok(metadata.dev() == device && metadata.ino() == inode);
}

/// What:
///  Without device and inode numbers,
/// no descriptor is recognized as the file.
/// Why:
///   The caller then finds no user and relies on its other proofs;
/// the crate does not build for Windows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// false
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn same_file(_link: &Path, _device: u64, _inode: u64) -> std::io::Result<bool> {
    return Ok(false);
}

/// What:
///  Whether a resolved link or descriptor matches the target.
/// Why:
///   One place decides a match for the working directory and for each descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// target.inside ? resolved.startsWith(target.inside) : sameFile(resolved, target.file)
/// ```
fn matches_target(link: &Path, target: &UseTarget<'_>) -> std::io::Result<bool> {
    match target {
        UseTarget::Inside(directory) => {
            let resolved: PathBuf = std::fs::read_link(link)?;
            return Ok(resolved.starts_with(directory));
        }
        UseTarget::File { device, inode } => return same_file(link, *device, *inode),
    }
}

/// What:
///  Whether one process uses the target,
///  and why its links were unreadable.
/// Why:
///   The first unreadable link is kept as the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function scanProcess({ procRoot, pid, target }): Promise<{ uses: boolean; unreadable?: string }>;
/// ```
fn scan_process(proc_root: &Path, pid: &str, target: &UseTarget<'_>) -> (bool, Option<String>) {
    let process: PathBuf = proc_root.join(pid);
    let mut uses: bool = false;
    let mut unreadable: Option<String> = None;
    if matches!(target, UseTarget::Inside(_)) {
        match matches_target(process.join("cwd").as_path(), target) {
            Ok(found) => uses = found,
            Err(error) if exited(&error) => {}
            Err(error) => unreadable = Some(format!("PID {pid} cwd: {error}")),
        }
    }
    let listing: std::fs::ReadDir = match std::fs::read_dir(process.join("fd")) {
        Ok(found) => found,
        Err(error) if exited(&error) => return (uses, unreadable),
        Err(error) => return (uses, unreadable.or(Some(format!("PID {pid} fd: {error}")))),
    };
    for item in listing.flatten() {
        match matches_target(item.path().as_path(), target) {
            Ok(true) => uses = true,
            Ok(false) => {}
            Err(error) if exited(&error) => {}
            Err(error) => {
                if unreadable.is_none() {
                    unreadable = Some(format!("PID {pid} fd {}: {error}", item.file_name().to_string_lossy()));
                }
            }
        }
    }
    return (uses, unreadable);
}

/// What:
///  Every process other than `self_pid` that uses the target,
/// through a proc root.
/// Why:
///   Recovery runs this only after a record is missing or a lock is unproven,
///  so the scan costs nothing on a commit that finds its state whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function processesUsing({ procRoot, target, selfPid }): Promise<UseScan>;
/// ```
pub fn processes_using_through(proc_root: &Path, target: &UseTarget<'_>, self_pid: u32) -> UseScan {
    let mut scan: UseScan = UseScan::default();
    let listing: std::fs::ReadDir = match std::fs::read_dir(proc_root) {
        Ok(found) => found,
        Err(error) => {
            scan.unavailable = Some(format!("{}: {error}", proc_root.display()));
            return scan;
        }
    };
    let own: String = self_pid.to_string();
    for item in listing.flatten() {
        let name: String = item.file_name().to_string_lossy().into_owned();
        let Ok(number) = name.parse::<i64>() else {
            continue;
        };
        if name == own {
            continue;
        }
        let (uses, unreadable) = scan_process(proc_root, name.as_str(), target);
        if uses {
            // A `comm` file that cannot be read leaves the command unknown, as the `.ok()` before did.
            let command: Option<String> = match std::fs::read_to_string(item.path().join("comm")) {
                Ok(text) => Some(String::from(text.trim())),
                Err(_) => None,
            };
            scan.users.push(ProcessUser { pid: number, command });
        }
        if let Some(reason) = unreadable
            && belongs_to_this_account(item.path().as_path())
        {
            scan.unreadable.push(reason);
        }
    }
    return scan;
}

/// What:
///  Every other process that uses the target on this host.
/// Why:
///   The production scan of `processes_using_through`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await processesUsing({ target })
/// ```
pub fn processes_using(target: &UseTarget<'_>) -> UseScan {
    return processes_using_through(Path::new(PROC_ROOT), target, std::process::id());
}

/// Process-use controls stay out of the release executable.
#[cfg(test)]
#[path = "process_use_tests.rs"]
mod tests;

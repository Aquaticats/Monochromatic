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
/// type UseTarget = { inside: string } | { file: { mount: bigint; inode: bigint } };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UseTarget<'a> {
    /// A working directory or open descriptor whose resolved path lies under this canonical directory.
    Inside(&'a Path),
    /// An open descriptor on this file,
    /// named by the mount it was reached through and its inode.
    File {
        /// The mount number the kernel gives the mount the file is reached through,
        /// which is what `/proc/<pid>/fdinfo` reports for a descriptor.
        mount: u64,
        /// Inode number of the file on that mount.
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
///  The mount number the kernel gives the mount a path is reached through.
/// Why:
///   `/proc/<pid>/fdinfo` names a descriptor's mount by that number and its file by the inode,
/// so a descriptor of another process is recognised without resolving it on its own file system.
/// `stat` on the `/proc/<pid>/fd/<n>` link resolves the file on its disk,
/// and waits there as long as that disk does:
///  on 2026-11-08 a stalled device held eight controls for over two minutes,
///  while their scans read other processes' descriptors.
/// Zero is "not known",
/// and matches no descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = statx(path, { mask: STATX_MNT_ID }).stx_mnt_id ?? 0;
/// ```
pub fn mount_of(path: &Path) -> u64 {
    let Ok(found) = rustix::fs::statx(rustix::fs::CWD, path, rustix::fs::AtFlags::SYMLINK_NOFOLLOW, rustix::fs::StatxFlags::MNT_ID) else {
        return 0;
    };
    return found.stx_mnt_id;
}

/// What:
///  Whether the descriptor `fdinfo` describes is the file with this inode on this mount.
/// Why:
///   The mount number and the inode together name the open file,
/// and both are in the text the kernel writes for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { mnt_id, ino } = parseFdinfo(await readFile(fdinfo, 'utf8')); return ino === inode && mnt_id === mount;
/// ```
fn descriptor_is_the_file(fdinfo: &Path, mount: u64, inode: u64) -> std::io::Result<bool> {
    let text: String = std::fs::read_to_string(fdinfo)?;
    let mut reached: Option<u64> = None;
    let mut held: Option<u64> = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("mnt_id:") {
            reached = value.trim().parse::<u64>().ok();
        } else if let Some(value) = line.strip_prefix("ino:") {
            held = value.trim().parse::<u64>().ok();
        }
    }
    return Ok(held == Some(inode) && reached == Some(mount) && mount != 0);
}

/// What:
///  The target of one scan,
/// prepared once:
/// the directory to resolve against,
/// or the mounts and the inode of the file to read descriptors for.
/// Why:
///   The mounts are read once for a scan and not once for each descriptor of each process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prepared = { inside: string } | { file: { mount: bigint; inode: bigint } };
/// ```
enum Prepared<'a> {
    /// A directory a working directory or a descriptor is resolved inside.
    Inside(&'a Path),
    /// A file's mount and inode.
    File {
        /// The mount number the file is reached through.
        mount: u64,
        /// The file's inode on that mount.
        inode: u64,
    },
}

/// What:
///  Whether a resolved working directory lies inside the target directory.
/// Why:
///   The working directory is a link to a path,
/// and reading it resolves nothing on the file system it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readlink(link).startsWith(directory);
/// ```
fn working_directory_is_inside(link: &Path, directory: &Path) -> std::io::Result<bool> {
    let resolved: PathBuf = std::fs::read_link(link)?;
    return Ok(resolved.starts_with(directory));
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
fn scan_process(proc_root: &Path, pid: &str, prepared: &Prepared<'_>) -> (bool, Option<String>) {
    let process: PathBuf = proc_root.join(pid);
    let mut uses: bool = false;
    let mut unreadable: Option<String> = None;
    let mut described: PathBuf = process.join("fdinfo");
    if let Prepared::Inside(directory) = prepared {
        match working_directory_is_inside(process.join("cwd").as_path(), directory) {
            Ok(found) => uses = found,
            Err(error) if exited(&error) => {}
            Err(error) => unreadable = Some(format!("PID {pid} cwd: {error}")),
        }
        described = process.join("fd");
    }
    let listing: std::fs::ReadDir = match std::fs::read_dir(described.as_path()) {
        Ok(found) => found,
        Err(error) if exited(&error) => return (uses, unreadable),
        Err(error) => return (uses, unreadable.or(Some(format!("PID {pid} fd: {error}")))),
    };
    for item in listing.flatten() {
        let answer: std::io::Result<bool> = match prepared {
            Prepared::Inside(directory) => working_directory_is_inside(item.path().as_path(), directory),
            Prepared::File { mount, inode } => descriptor_is_the_file(item.path().as_path(), *mount, *inode),
        };
        match answer {
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
    // A file whose file system the mount table does not name cannot be recognised in any process: the caller is told
    //  so rather than being given a scan that found nothing.
    let prepared: Prepared<'_> = match target {
        UseTarget::Inside(directory) => Prepared::Inside(directory),
        UseTarget::File { mount, inode } => Prepared::File { mount: *mount, inode: *inode },
    };
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
        let (uses, unreadable) = scan_process(proc_root, name.as_str(), &prepared);
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

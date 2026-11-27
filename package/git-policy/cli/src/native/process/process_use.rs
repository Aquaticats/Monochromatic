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
//!
//! Files beside this one,
//!  and why the walk is not one file:
//!  `process_use_targets_and_answers.rs` says what a scan is asked for and what it answers,
//!  and `descriptor_of_a_process_that_may_hold_the_file.rs` asks whether one descriptor is the file.
//!  `rust/max-lines` splits a file by role,
//!  and the walk of the table is the role that reads one process after another.

pub use super::descriptor_of_a_process_that_may_hold_the_file::{
    descriptor_is_file,
    descriptor_is_file_with,
    mount_of,
    mounted_descriptor_is_file,
};
pub use super::process_use_targets_and_answers::{
    PROC_ROOT,
    PROCESS_SCAN_CEILING,
    ProcessUser,
    Unread,
    UseScan,
    UseTarget,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Instant;

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
    return error.kind() == std::io::ErrorKind::NotFound
        || error.raw_os_error() == Some(libc::ESRCH);
}

/// What:
///  Whether a moment is behind now.
/// Why:
///   A scan checks the moment it was given before each process and each descriptor,
///  and a moment that has come ends the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Instant.now() >= moment
/// ```
fn past(moment: Instant) -> bool {
    return Instant::now() >= moment;
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
///  The target of one scan,
/// prepared once:
/// the directory to resolve against,
/// or the device number and the inode of the file to read descriptors for.
/// Why:
///   The two numbers are read once for a scan and not once for each descriptor of each process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prepared = { inside: string } | { file: { device: bigint; inode: bigint } };
/// ```
enum Prepared<'a> {
    /// A directory a working directory or a descriptor is resolved inside.
    Inside(&'a Path),
    /// A file's device number and inode.
    File {
        /// The device number of the file system that holds it.
        device: u64,
        /// The file's inode on that device.
        inode: u64,
    },
    /// A file's mount number and inode.
    Mounted {
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
fn working_directory_is_inside(
    link: &Path,
    directory: &Path,
) -> std::io::Result<bool> {
    let resolved: PathBuf = std::fs::read_link(link)?;
    return Ok(resolved.starts_with(directory));
}

/// What:
///  Whether one process uses the target,
///  why its descriptor directory could not be read,
///  which of its descriptors were named by a record and could not be identified,
///  and whether the moment the scan ends came before it finished.
/// Why:
///   The first reason of each kind is kept:
///  a refused directory is a process the scan could not look at,
///  and a refused descriptor is a file the kernel would not hand over,
///  which a caller treats as one that may be held.
///  A process the scan stopped in is a process whose descriptors past the stop were not looked at,
///  so the scan as a whole proved nothing and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Findings = { uses: boolean; unreadable?: string; unresolved?: string; stopped: boolean };
/// ```
struct Findings {
    /// A descriptor or a working directory of the process is the target.
    uses: bool,
    /// Why something of the process could not be read.
    unreadable: Option<String>,
    /// Why a descriptor that may be the target could not be identified.
    unresolved: Option<String>,
    /// The scan's deadline came before this process was looked at whole.
    stopped: bool,
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
fn scan_process(
    proc_root: &Path,
    pid: &str,
    prepared: &Prepared<'_>,
    deadline: Option<Instant>,
) -> Findings {
    let process: PathBuf = proc_root.join(pid);
    let mut found: Findings = Findings {
        uses: false,
        unreadable: None,
        unresolved: None,
        stopped: false,
    };
    let mut described: PathBuf = process.join("fdinfo");
    if let Prepared::Inside(directory) = prepared {
        match working_directory_is_inside(process.join("cwd").as_path(), directory) {
            Ok(inside) => found.uses = inside,
            Err(error) if exited(&error) => {}
            Err(error) => found.unreadable = Some(format!("PID {pid} cwd: {error}")),
        }
        described = process.join("fd");
    }
    let listing: std::fs::ReadDir = match std::fs::read_dir(described.as_path()) {
        Ok(listed) => listed,
        Err(error) if exited(&error) => return found,
        Err(error) => {
            found.unreadable = Some(format!("PID {pid} fd: {error}"));
            return found;
        }
    };
    for item in listing.flatten() {
        // A record read cannot be interrupted once it has begun, so the deadline is taken before each descriptor: everything
        //  the scan does between two of them is bounded by it.
        if deadline.is_some_and(past) {
            found.stopped = true;
            return found;
        }
        let answer: Result<bool, Unread> = match prepared {
            // A working directory that cannot be resolved is a path this scan could not look at, which a caller reads as an
            // unreadable process rather than as a file it could not identify.
            Prepared::Inside(directory) => {
                match working_directory_is_inside(item.path().as_path(), directory) {
                    Ok(inside) => Ok(inside),
                    Err(error) if exited(&error) => Ok(false),
                    Err(error) => Err(Unread::Process(format!("{error}"))),
                }
            }
            Prepared::File {
                device,
                inode,
            } => {
                let name: std::ffi::OsString = item.file_name();
                named_unread(
                    pid,
                    name.as_os_str(),
                    descriptor_is_file(process.as_path(), name.as_os_str(), *device, *inode),
                )
            }
            Prepared::Mounted {
                mount,
                inode,
            } => {
                let name: std::ffi::OsString = item.file_name();
                named_unread(
                    pid,
                    name.as_os_str(),
                    mounted_descriptor_is_file(process.as_path(), name.as_os_str(), *mount, *inode),
                )
            }
        };
        match answer {
            Ok(true) => found.uses = true,
            Ok(false) => {}
            Err(Unread::Process(reason)) => {
                if found.unreadable.is_none() {
                    found.unreadable = Some(reason);
                }
            }
            Err(Unread::File(reason)) => {
                if found.unresolved.is_none() {
                    found.unresolved = Some(reason);
                }
            }
        }
    }
    return found;
}

/// What:
///  An unread reason with the process and descriptor it is about.
/// Why:
///   The reasons are matched by their `PID <number>` prefix,
///  and a descriptor's own failure names which descriptor it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namedUnread(pid, fd, outcome);
/// ```
fn named_unread(
    pid: &str,
    fd: &std::ffi::OsStr,
    outcome: Result<bool, Unread>,
) -> Result<bool, Unread> {
    let where_: String = format!("PID {pid} fd {}", fd.to_string_lossy());
    return match outcome {
        Ok(found) => Ok(found),
        Err(Unread::Process(reason)) => Err(Unread::Process(format!("{where_}: {reason}"))),
        Err(Unread::File(reason)) => Err(Unread::File(format!("{where_}: {reason}"))),
    };
}

/// What:
///  Every process other than `self_pid` that uses the target,
/// through a proc root,
/// until the moment `deadline` names.
/// Why:
///   A record read cannot be interrupted once it has begun,
///  and a host holds as many descriptors as its programs decide to:
///  so the walk of the table is what carries a bound,
///  and a walk that reached it says so instead of reporting a table it did not finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function processesUsingUntil({ procRoot, target, selfPid, deadline }): Promise<UseScan>;
/// ```
pub fn processes_using_until(
    proc_root: &Path,
    target: &UseTarget<'_>,
    self_pid: u32,
    deadline: Option<Instant>,
) -> UseScan {
    let mut scan: UseScan = UseScan::default();
    // A file is named by the device number and the inode it has, so no mount table is read and no descriptor is
    //  resolved except the ones whose record already names the file's inode number.
    let prepared: Prepared<'_> = match target {
        UseTarget::Inside(directory) => Prepared::Inside(directory),
        UseTarget::File {
            device,
            inode,
        } => Prepared::File {
            device: *device,
            inode: *inode,
        },
        UseTarget::Mounted {
            mount,
            inode,
        } => Prepared::Mounted {
            mount: *mount,
            inode: *inode,
        },
    };
    let listing: std::fs::ReadDir = match std::fs::read_dir(proc_root) {
        Ok(listed) => listed,
        Err(error) => {
            scan.unavailable = Some(format!("{}: {error}", proc_root.display()));
            return scan;
        }
    };
    let own: String = self_pid.to_string();
    let mut reached: usize = 0;
    for item in listing.flatten() {
        let name: String = item.file_name().to_string_lossy().into_owned();
        let Ok(number) = name.parse::<i64>() else {
            continue;
        };
        if name == own {
            continue;
        }
        if deadline.is_some_and(past) {
            scan.unavailable = Some(stopped_at_the_bound(reached));
            return scan;
        }
        reached += 1;
        let found: Findings = scan_process(proc_root, name.as_str(), &prepared, deadline);
        if found.uses {
            // A `comm` file that cannot be read leaves the command unknown, as the `.ok()` before did.
            let command: Option<String> = match std::fs::read_to_string(item.path().join("comm")) {
                Ok(text) => Some(String::from(text.trim())),
                Err(_) => None,
            };
            scan.users.push(ProcessUser {
                pid: number,
                command,
            });
        }
        if let Some(reason) = found.unresolved
            && belongs_to_this_account(item.path().as_path())
        {
            scan.unresolved.push(reason);
        }
        if let Some(reason) = found.unreadable
            && belongs_to_this_account(item.path().as_path())
        {
            scan.unreadable.push(reason);
        }
        if found.stopped {
            scan.unavailable = Some(stopped_at_the_bound(reached));
            return scan;
        }
    }
    return scan;
}

/// What:
///  Why a scan of the process table ended at its moment rather than at the end of the table.
/// Why:
///   The processes past the stop were not looked at,
///  so the scan proves nothing about a file they may hold,
///  and a caller that would remove or move something waits for the processed table instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the scan of the process table stopped at its time bound after ${reached} processes, so the processes it did not reach may hold the target`;
/// ```
fn stopped_at_the_bound(reached: usize) -> String {
    return format!(
        "the scan of the process table stopped at its time bound after {reached} processes, so a process it did not reach may hold the target"
    );
}

/// What:
///  Every process other than `self_pid` that uses the target,
/// through a proc root,
/// within `PROCESS_SCAN_CEILING`.
/// Why:
///   The production scan:
///  every command that gives up on a record or a lock reads the table this way,
///  and none of them may outlast its bound because a program of the account holds many descriptors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function processesUsingThrough({ procRoot, target, selfPid }): Promise<UseScan>;
/// ```
pub fn processes_using_through(
    proc_root: &Path,
    target: &UseTarget<'_>,
    self_pid: u32,
) -> UseScan {
    return processes_using_until(
        proc_root,
        target,
        self_pid,
        Some(Instant::now() + PROCESS_SCAN_CEILING),
    );
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

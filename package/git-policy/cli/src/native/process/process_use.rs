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
/// `Instant` is a moment on the monotonic clock,
/// which a scan's bound names.
use std::time::Instant;

/// The root of the process table on Linux.
pub const PROC_ROOT: &str = "/proc";

/// How long one scan of this host's processes may take.
///
/// Measured on 2026-11-07 on this host at load average 37 to 52:
/// one walk of every process's open descriptors (list each `/proc/<pid>/fd` and `stat` each link) over 1,000 processes
/// and 54,500 descriptors took 313,
/// 360 and 398 ms,
/// and the commit service's scan carries the same bound for the same measurement (`SERVICE_SCAN_BUDGET`).
/// A scan that reaches it returns what it found and says that it did not finish,
/// which every caller reads as a table it may not act on:
/// two seconds is the smallest whole number of seconds that is at least five times the longest walk.
/// Why a whole table and not one descriptor:
///  a read of a descriptor's record cannot be interrupted once it has begun,
///  so what bounds a single read is the gate before it (`descriptor_record.rs`) and what bounds the scan is this.
pub const PROCESS_SCAN_CEILING: std::time::Duration = std::time::Duration::from_secs(2);

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
    /// An open descriptor on this file,
    /// named by the device number and the inode the file has.
    File {
        /// The device number of the file system that holds the file (`st_dev`).
        device: u64,
        /// Inode number of the file on that device.
        inode: u64,
    },
    /// An open descriptor on a file reached through this mount,
    /// named as `/proc/<pid>/fdinfo` names it.
    /// Why:
    ///   A pipe and another anonymous file have no device a descriptor could be read for through the gate that
    ///   keeps a stalled server out (`open_file_of_a_descriptor.rs`),
    ///   and the mount number and the inode in the descriptor's own record name the open file without resolving it.
    Mounted {
        /// The mount number the kernel gives the mount the file is reached through.
        mount: u64,
        /// The file's inode on that mount.
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
    /// Reasons some process of the scanning account could not be read at all.
    pub unreadable: Vec<String>,
    /// Reasons a descriptor whose record names the target could not be identified,
    /// because the kernel refused to copy it into this process:
    /// the process may hold the file,
    /// which is what a scan that did not finish proves nothing means.
    pub unresolved: Vec<String>,
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
///  Whether the descriptor `fd` of the process whose table directory is `process` is the file with this device number and inode.
/// Why:
///   A device number and an inode name one file on a machine,
/// which is what `stat(2)` says of `st_dev` and `st_ino`.
/// The pair the kernel writes for a descriptor,
/// its mount number and its inode,
/// is neither unique nor complete (audit of 2026-11-09,
/// finding 4):
/// two files of two btrfs subvolumes reached through one mount have the same inode number and the same mount number,
/// so a process holding one of them read as the holder of the other,
/// and a bind mount gives the same file another mount number,
/// so a holder that reached it that way was not found at all.
/// The record's inode number is the cheap test that comes first:
/// only a descriptor whose record already names the file's inode number is resolved,
/// and the answer comes from a copy of the descriptor itself (`opened_file_of`),
/// never from the path of its link,
/// because walking that path asks the file's own file system to revalidate its directory entry and a stalled one holds it
/// for ever (cross-vendor review of 2026-11-09,
/// finding 3;
/// `doc/troubleshooting/fuse-descriptor-attributes.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { ino } = descriptorRecord({ process, fd, targeted: 'real-file' });
/// const opened = ino === inode ? openedFileOf({ process, fd }) : undefined;
/// opened?.device === device && opened?.inode === inode;
/// ```
#[cfg(unix)]
pub fn descriptor_is_file(process: &Path, fd: &std::ffi::OsStr, device: u64, inode: u64) -> Result<bool, Unread> {
    return descriptor_is_file_with(process, fd, device, inode, crate::open_file_of_a_descriptor::opened_file_of);
}

/// What:
///  Why a descriptor could not be identified.
/// Why:
///   Something of the process could not be read at all (its descriptor's record,
/// or its working directory),
///  which a caller may tolerate as it always did;
///  a record that named the file whose file cannot then be identified is a file that may be held,
///  which a caller must not read as "not the file".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unread = { kind: 'process'; reason: string } | { kind: 'file'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Unread {
    /// Something of the process could not be read.
    Process(String),
    /// The record was read and named the file,
    /// and the file itself could not be identified.
    File(String),
}

/// What:
///  `descriptor_is_file` with the reader of a descriptor's open file given:
///  the kernel's copy of it in a judgement,
///  and a control's own reader for a table the kernel does not know.
/// Why:
///   A control's `/proc` is a directory of links,
///  so the file a descriptor has open there can only be read from its link,
///  while the production read copies the descriptor and never walks the path
///  (`src/native/process/open_file_of_a_descriptor.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// descriptorIsFileWith({ process, fd, device, inode, opened });
/// ```
#[cfg(unix)]
pub fn descriptor_is_file_with(
    process: &Path,
    fd: &std::ffi::OsStr,
    device: u64,
    inode: u64,
    opened: fn(&Path, &std::ffi::OsStr) -> std::io::Result<crate::open_file_of_a_descriptor::OpenedFile>,
) -> Result<bool, Unread> {
    let text: String = match crate::descriptor_record::descriptor_record(process, fd, crate::descriptor_record::Targeted::RealFile) {
        // A pipe, a socket or a kernel anonymous inode cannot be a file of a file system, its record is never opened, and
        //  a descriptor that closed while the scan walked the directory is open in nothing.
        crate::descriptor_record::Record::NotTheTarget | crate::descriptor_record::Record::Gone => return Ok(false),
        crate::descriptor_record::Record::Text(text) => text,
        // A record past the bound is one this scan did not read: the descriptor may hold the file, so nothing is concluded.
        crate::descriptor_record::Record::BeyondBound => {
            return Err(Unread::File(format!(
                "its record is longer than the {} bytes this scan takes, so the descriptor may hold the file",
                crate::descriptor_record::RECORD_BYTES_AT_MOST
            )));
        }
        crate::descriptor_record::Record::Unreadable { reason, .. } => return Err(Unread::Process(reason)),
        // The kind of a descriptor whose link text cannot be read is unknown, so it may be the file.
        crate::descriptor_record::Record::KindUnknown { reason, .. } => {
            return Err(Unread::File(format!("the text of its link cannot be read ({reason}), so the descriptor may hold the file")));
        }
    };
    let held: Option<u64> = crate::descriptor_record::inode_in(text.as_str());
    if held != Some(inode) {
        return Ok(false);
    }
    let file: crate::open_file_of_a_descriptor::OpenedFile = match opened(process, fd) {
        Ok(found) => found,
        Err(error) => return Err(Unread::File(format!("{error}"))),
    };
    return Ok(file.device == device && file.inode == inode);
}

/// What:
///  The mount number the kernel gives the mount a path is reached through.
/// Why:
///   `/proc/<pid>/fdinfo` names a descriptor's mount by that number and its file by the inode,
/// so a descriptor of another process is recognised without resolving it on its own file system,
/// which waits there as long as that file system does
/// (`open_file_of_a_descriptor.rs`).
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
///  Whether the descriptor `fd` of the process whose table directory is `process` is the file with this inode on this
///  mount,
///  as the descriptor's own record names it.
/// Why:
///   The mount number and the inode are both in the text the kernel writes for the descriptor,
///  so this asks no file system anything,
///  and it answers for a pipe or another anonymous file that the gate refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { mnt_id, ino } = descriptorRecord({ process, fd, targeted: 'anonymous-file' }); ino === inode && mnt_id === mount;
/// ```
pub fn mounted_descriptor_is_file(process: &Path, fd: &std::ffi::OsStr, mount: u64, inode: u64) -> Result<bool, Unread> {
    let text: String = match crate::descriptor_record::descriptor_record(process, fd, crate::descriptor_record::Targeted::AnonymousFile) {
        // A kernel anonymous inode is not a pipe or another named anonymous file, its record is never opened, and a
        //  descriptor that closed while the scan walked the directory is open in nothing.
        crate::descriptor_record::Record::NotTheTarget | crate::descriptor_record::Record::Gone => return Ok(false),
        crate::descriptor_record::Record::Text(text) => text,
        crate::descriptor_record::Record::BeyondBound => {
            return Err(Unread::File(format!(
                "its record is longer than the {} bytes this scan takes, so the descriptor may hold the file",
                crate::descriptor_record::RECORD_BYTES_AT_MOST
            )));
        }
        crate::descriptor_record::Record::Unreadable { reason, .. } => return Err(Unread::Process(reason)),
        crate::descriptor_record::Record::KindUnknown { reason, .. } => {
            return Err(Unread::File(format!("the text of its link cannot be read ({reason}), so the descriptor may hold the file")));
        }
    };
    let reached: Option<u64> = crate::descriptor_record::mount_id_in(text.as_str());
    let held: Option<u64> = crate::descriptor_record::inode_in(text.as_str());
    return Ok(held == Some(inode) && reached == Some(mount) && mount != 0);
}

/// What:
///  The same on a platform that has no `/proc` to read a descriptor's record from.
/// Why:
///   The scan of another process's descriptors is Linux's,
///  and a platform without it answers "not this file" rather than promising something it cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.platform !== 'linux') return false;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn descriptor_is_file(process: &Path, fd: &std::ffi::OsStr, device: u64, inode: u64) -> Result<bool, Unread> {
    let _ = (process, fd, device, inode);
    return Ok(false);
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
fn working_directory_is_inside(link: &Path, directory: &Path) -> std::io::Result<bool> {
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
fn scan_process(proc_root: &Path, pid: &str, prepared: &Prepared<'_>, deadline: Option<Instant>) -> Findings {
    let process: PathBuf = proc_root.join(pid);
    let mut found: Findings = Findings { uses: false, unreadable: None, unresolved: None, stopped: false };
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
            Prepared::Inside(directory) => match working_directory_is_inside(item.path().as_path(), directory) {
                Ok(inside) => Ok(inside),
                Err(error) if exited(&error) => Ok(false),
                Err(error) => Err(Unread::Process(format!("{error}"))),
            },
            Prepared::File { device, inode } => {
                let name: std::ffi::OsString = item.file_name();
                named_unread(pid, name.as_os_str(), descriptor_is_file(process.as_path(), name.as_os_str(), *device, *inode))
            }
            Prepared::Mounted { mount, inode } => {
                let name: std::ffi::OsString = item.file_name();
                named_unread(pid, name.as_os_str(), mounted_descriptor_is_file(process.as_path(), name.as_os_str(), *mount, *inode))
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
fn named_unread(pid: &str, fd: &std::ffi::OsStr, outcome: Result<bool, Unread>) -> Result<bool, Unread> {
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
pub fn processes_using_until(proc_root: &Path, target: &UseTarget<'_>, self_pid: u32, deadline: Option<Instant>) -> UseScan {
    let mut scan: UseScan = UseScan::default();
    // A file is named by the device number and the inode it has, so no mount table is read and no descriptor is
    //  resolved except the ones whose record already names the file's inode number.
    let prepared: Prepared<'_> = match target {
        UseTarget::Inside(directory) => Prepared::Inside(directory),
        UseTarget::File { device, inode } => Prepared::File { device: *device, inode: *inode },
        UseTarget::Mounted { mount, inode } => Prepared::Mounted { mount: *mount, inode: *inode },
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
            scan.users.push(ProcessUser { pid: number, command });
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
pub fn processes_using_through(proc_root: &Path, target: &UseTarget<'_>, self_pid: u32) -> UseScan {
    return processes_using_until(proc_root, target, self_pid, Some(Instant::now() + PROCESS_SCAN_CEILING));
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

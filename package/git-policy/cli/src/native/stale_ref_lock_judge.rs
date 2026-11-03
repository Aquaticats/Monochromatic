//! What:
//!  How one reference lock is judged: too new, held, stale on a proof, or unproven.
//! Why:
//!  Git writes no holder into a lock, so only a few facts prove a holder gone; everything else is left alone. The order of
//!  the checks and the reason for each are in `SPEC.md`, "Reference locks left by a Git that was killed".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const examined = judge.examine(lockPath); // { verdict, pidFile, size, born, identity }
//! ```

use super::fs_mount_options::read_mount_facts;
use super::pid_namespace::current_pid_namespace;
use super::process_start::{ProcessStart, linux_start, now_ms};
use super::process_use::{UseScan, UseTarget, processes_using_through};
use super::stale_ref_lock_found::PidFile;
use super::stopped_git_locks::pid_named_by;
use super::stale_ref_lock_walk::pid_file_beside;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How old a lock must be before it is judged: long enough for a live Git to have filled and renamed a lock it just made.
pub const MARGIN: Duration = Duration::from_secs(2);

/// The kernel's name for the initial PID namespace, where every process of the machine has the number it is seen by.
const INITIAL_PID_NAMESPACE: &str = "pid:[4026531836]";

/// The file system types a lock may sit on: disks and memory of this machine. A network or user-space file system can be
///  written by a Git on another machine, whose process numbers mean nothing here.
const LOCAL_FILE_SYSTEMS: [&str; 8] = ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs"];

/// The command name of Git, and of its helpers (`git-remote-https`), as the kernel reports it.
const GIT_COMMAND_PREFIX: &str = "git";

/// Why a lock may be removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Proof {
    /// The process the pid file names is gone.
    PidGone(i64),
    /// The process number now belongs to a program that is not Git.
    PidReused {
        /// The number the pid file names.
        pid: i64,
        /// The command name of the process that has it now.
        command: String,
    },
    /// The lock was born before the current boot of the machine.
    BeforeBoot,
}

/// What a judgement concluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// Younger than the margin, or not a regular file: left, and not mentioned.
    TooNew,
    /// A running process holds it; the text names the process and how that is known.
    Held(String),
    /// Nothing proves the holder gone; the text says why.
    Unproven(String),
    /// Removable, on this proof.
    Stale(Proof),
}

/// The evidence a judgement may use, and the limits of what it can see.
#[derive(Clone, Debug)]
pub struct Judge {
    /// The process table.
    pub proc_root: PathBuf,
    /// How old a lock must be before it is judged at all.
    pub margin: Duration,
    /// Whether the process numbers read here mean the same processes as in the pid files.
    pub pid_view_complete: bool,
    /// When the machine booted, where known.
    pub boot: Option<SystemTime>,
    /// The file system types a lock may sit on.
    pub local_file_systems: Vec<String>,
}

/// A lock's identity at the moment it was judged: the removal checks that it is still this file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Identity {
    /// Device number.
    pub device: u64,
    /// Inode number; a freed inode may be reused at once, so it is never compared alone.
    pub inode: u64,
    /// Size in bytes.
    pub size: u64,
    /// Birth time in nanoseconds since the epoch, where the file system keeps it.
    pub born_ns: Option<u128>,
    /// Modification time in nanoseconds since the epoch.
    pub modified_ns: i128,
}

/// The identity of the file at `path`, without following a link.
pub fn identity_of(path: &Path) -> Option<Identity> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    let born_ns: Option<u128> = metadata
        .created()
        .ok()
        .and_then(|born| { return born.duration_since(SystemTime::UNIX_EPOCH).ok(); })
        .map(|elapsed| { return elapsed.as_nanos(); });
    let modified_ns: i128 = i128::from(metadata.mtime()) * 1_000_000_000 + i128::from(metadata.mtime_nsec());
    return Some(Identity { device: metadata.dev(), inode: metadata.ino(), size: metadata.len(), born_ns, modified_ns });
}

/// Everything one judgement found out.
#[derive(Clone, Debug)]
pub struct Examined {
    /// The conclusion.
    pub verdict: Verdict,
    /// What the pid file beside the lock says.
    pub pid_file: PidFile,
    /// The lock's identity when it was judged.
    pub identity: Identity,
    /// When the lock was born, where known.
    pub born: Option<SystemTime>,
}

impl Judge {
    /// The judge of this process: the real process table, the real boot time, and a complete view of the process numbers
    ///  only where this process runs in the initial PID namespace.
    pub fn here() -> Self {
        return Self {
            proc_root: PathBuf::from("/proc"),
            margin: MARGIN,
            pid_view_complete: current_pid_namespace().as_deref() == Some(INITIAL_PID_NAMESPACE),
            boot: boot_time(),
            local_file_systems: LOCAL_FILE_SYSTEMS.iter().map(|name| { return String::from(*name); }).collect(),
        };
    }

    /// Whether the file system holding `path` is on the local list; the reason when it is not or cannot be told.
    fn local_check(&self, path: &Path) -> Result<(), String> {
        let facts = read_mount_facts(path)?;
        let Some(mount) = facts else {
            return Err(String::from("the file system that holds it is not in the mount table"));
        };
        if self.local_file_systems.contains(&mount.filesystem_type) {
            return Ok(());
        }
        return Err(format!("it sits on a `{}` file system, which a Git on another machine may share", mount.filesystem_type));
    }

    /// The pid file beside `lock`, as evidence about this lock's holder.
    fn pid_file_state(lock: &Path, lock_born: Option<SystemTime>) -> PidFile {
        let pid_path: PathBuf = pid_file_beside(lock);
        let Some(pid) = pid_named_by(pid_path.as_path()) else {
            return PidFile::Absent;
        };
        let pid_born: Option<SystemTime> = std::fs::symlink_metadata(pid_path.as_path()).ok().and_then(|metadata| { return metadata.created().ok(); });
        return match (lock_born, pid_born) {
            // Git creates the pid file after the lock, so a pid file born earlier belongs to an earlier holder.
            (Some(lock_at), Some(pid_at)) if pid_at >= lock_at => PidFile::Names(i64::from(pid)),
            _ => PidFile::Older(i64::from(pid)),
        };
    }

    /// What the pid file's process is now: gone, a program that is not Git, a Git, or not known.
    fn judge_pid(&self, pid: i64) -> Verdict {
        if !self.pid_view_complete {
            return Verdict::Unproven(format!(
                "the pid file beside it names process {pid}, but this process runs in a PID namespace of its own and cannot see every process of the machine, so it cannot tell whether {pid} is gone"
            ));
        }
        return match linux_start(pid, self.proc_root.as_path(), now_ms()) {
            ProcessStart::Missing => Verdict::Stale(Proof::PidGone(pid)),
            ProcessStart::Unknown(reason) => Verdict::Unproven(format!("the pid file beside it names process {pid}, which could not be looked at: {reason}")),
            ProcessStart::Running { .. } => {
                let command: String = std::fs::read_to_string(self.proc_root.join(pid.to_string()).join("comm")).map(|text| { return text.trim().to_owned(); }).unwrap_or_default();
                if command.starts_with(GIT_COMMAND_PREFIX) {
                    Verdict::Held(format!("process {pid} ({command}), which the pid file beside it names, is running"))
                } else {
                    Verdict::Stale(Proof::PidReused { pid, command })
                }
            }
        };
    }

    /// Whether the machine booted after the lock was last written, and the lock born.
    fn before_boot(&self, born: Option<SystemTime>, identity: &Identity) -> bool {
        let Some(boot) = self.boot else {
            return false;
        };
        let modified: SystemTime = nanoseconds_to_time(identity.modified_ns);
        let slack: Duration = self.margin.max(super::stopped_git_locks::COARSE_CLOCK_SLACK);
        return born.is_some_and(|at| { return at + slack < boot; }) && modified + slack < boot;
    }

    /// The first process of this account that the scan could not look into and that is a Git (or whose name cannot be read
    ///  back): a program that is not Git, such as the user's service manager, cannot hold a Git lock. The reason text of the
    ///  scan starts `PID <number>`.
    fn unreadable_git(&self, scan: &UseScan) -> Option<String> {
        for reason in &scan.unreadable {
            let number: Option<&str> = reason.strip_prefix("PID ").and_then(|rest| { return rest.split(' ').next(); });
            let command: Option<String> = number.and_then(|pid| { return std::fs::read_to_string(self.proc_root.join(pid).join("comm")).ok(); });
            match (number, command) {
                (Some(_), None) => {}
                (_, Some(name)) if !name.trim().starts_with(GIT_COMMAND_PREFIX) => {}
                _ => return Some(reason.clone()),
            }
        }
        return None;
    }

    /// Judge the lock at `lock`; nothing when it is not a regular file any more.
    pub fn examine(&self, lock: &Path) -> Option<Examined> {
        let identity: Identity = identity_of(lock)?;
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(lock).ok()?;
        let born: Option<SystemTime> = metadata.created().ok();
        let pid_file: PidFile = Self::pid_file_state(lock, born);
        let finish = |verdict: Verdict| { return Some(Examined { verdict, pid_file: pid_file.clone(), identity, born }); };
        if !metadata.is_file() {
            return finish(Verdict::TooNew);
        }
        let newest: SystemTime = born.map_or(nanoseconds_to_time(identity.modified_ns), |at| { return at.max(nanoseconds_to_time(identity.modified_ns)); });
        if SystemTime::now().duration_since(newest).unwrap_or_default() < self.margin {
            return finish(Verdict::TooNew);
        }
        if let Err(reason) = self.local_check(lock) {
            return finish(Verdict::Unproven(reason));
        }
        let scan: UseScan = processes_using_through(self.proc_root.as_path(), &UseTarget::File { device: identity.device, inode: identity.inode }, std::process::id());
        if let Some(reason) = scan.unavailable {
            return finish(Verdict::Unproven(format!("the process table could not be listed: {reason}")));
        }
        if let Some(user) = scan.users.first() {
            return finish(Verdict::Held(format!("process {} ({}) has the file open", user.pid, user.command.clone().unwrap_or_default())));
        }
        if let Some(reason) = self.unreadable_git(&scan) {
            return finish(Verdict::Unproven(format!("a Git process of this account could not be looked into ({reason}), so it may hold the lock")));
        }
        if let PidFile::Names(pid) = pid_file {
            return finish(self.judge_pid(pid));
        }
        if self.before_boot(born, &identity) {
            return finish(Verdict::Stale(Proof::BeforeBoot));
        }
        let why: String = match pid_file {
            PidFile::Older(pid) => format!("the pid file beside it (process {pid}) is older than the lock, so it belongs to an earlier holder"),
            _ => String::from("no pid file beside it names a holder (Git writes one only under `core.lockfilePid`), and no process has it open"),
        };
        return finish(Verdict::Unproven(why));
    }
}

/// A time given as nanoseconds since the epoch.
fn nanoseconds_to_time(nanoseconds: i128) -> SystemTime {
    let magnitude: u64 = u64::try_from(nanoseconds.unsigned_abs()).unwrap_or(u64::MAX);
    return if nanoseconds >= 0 { SystemTime::UNIX_EPOCH + Duration::from_nanos(magnitude) } else { SystemTime::UNIX_EPOCH - Duration::from_nanos(magnitude) };
}

/// When the machine booted: the `btime` line of `/proc/stat`, in whole seconds since the epoch.
fn boot_time() -> Option<SystemTime> {
    let text: String = std::fs::read_to_string("/proc/stat").ok()?;
    let seconds: u64 = text.lines().find_map(|line| { return line.strip_prefix("btime ")?.trim().parse::<u64>().ok(); })?;
    return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds));
}

//! What:
//!  How one reference lock is judged:
//! too new,
//! held,
//! stale on a proof,
//! or unproven.
//! Why:
//!  Git writes no holder into a lock,
//! so only a few facts prove a holder gone;
//! everything else is left alone.
//! The order of
//!  the checks and the reason for each are in `SPEC.md`,
//! "Reference locks left by a Git that was killed".
//! A time stamp is not
//!  among the proofs:
//! a correction of the wall clock moves every time that a lock and the boot are compared by (finding S-2 of
//!  the cross-vendor review of 2026-11-03T11:33:50-05:00).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const examined = judge.examine(lockPath); // { verdict, pidFile, size, born, identity }
//! ```

use crate::fs_mount_options::read_mount_facts;
use crate::pid_namespace::current_pid_namespace;
use crate::process_start::{ProcessStart, linux_start, now_ms};
use crate::process_use::{UseScan, UseTarget, processes_using_through};
use crate::stale_ref_lock_found::PidFile;
pub use crate::stale_ref_lock_own_ended::{ClaimedLock, OwnEnded};
use crate::stale_ref_lock_nested_numbers::nested_holders_of;
use crate::stopped_git_locks::pid_named_by;
use crate::stale_ref_lock_walk::pid_file_beside;
pub use crate::stale_ref_lock_identity::{Identity, identity_of};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The longest step of a landing that updates a reference,
/// in a live service:
/// 3,438 ms,
/// the largest `ref_ms` of 1,302 `answered`
///  entries of the service journal (`journalctl --user SYSLOG_IDENTIFIER=cli-git-service`,
/// 2026-11-02T09:01:38Z to
///  2026-11-03T15:11:58Z;
/// 1 entry above 2,000 ms and 2 at or above 1,000 ms;
/// median 2 ms,
/// 99th percentile 64 ms).
/// The step starts
///  Git,
/// takes the reference lock,
/// fills it and renames it,
/// so a lock a live Git holds is gone within it.
/// Real Git on a
///  throwaway repository,
/// traced with `strace`,
/// held a lock for at most 2.8 ms (7,700 updates,
/// load average 3 to 14),
/// so this
///  is the loaded tail,
/// not the common case.
pub const LONGEST_LIVE_REF_STEP: Duration = Duration::from_millis(3438);

/// How many times the longest live reference step the margin allows:
/// twice,
/// because the longest of a sample understates the
///  tail (one entry of 1,302 reached 3,438 ms,
/// so the next sample may reach more).
/// Option B of the question on the margin,
/// "twice
///  the sample maximum,
/// seven seconds",
/// picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups").
pub const MARGIN_MULTIPLE: u64 = 2;

/// How old a lock must be before it is judged:
/// long enough for a live Git to have filled and renamed a lock it just made.
/// The
///  whole seconds at or above `MARGIN_MULTIPLE` times the longest step measured in `LONGEST_LIVE_REF_STEP` (6.876 s,
/// so 7 s),
///  because a lock younger than the longest live hold may still be live.
/// A stale lock is left that long before it is cleared,
///  which is a few seconds of a failing update,
/// against a live lock removed (`doc/planning/cli-git-shipped-limits-audit.md`,
///  "Margins for removing left-behind locks").
pub const MARGIN: Duration = Duration::from_secs((LONGEST_LIVE_REF_STEP.as_millis() as u64 * MARGIN_MULTIPLE).div_ceil(1000));

/// The kernel's name for the initial PID namespace,
/// where every process of the machine has the number it is seen by.
const INITIAL_PID_NAMESPACE: &str = "pid:[4026531836]";

/// The mount point of the process table the judge reads.
const PROC_MOUNT_POINT: &str = "/proc";

/// The start of the mount option that hides other accounts' processes from this account.
const HIDEPID_OPTION: &str = "hidepid=";

/// The values of `hidepid` that hide nothing.
const HIDEPID_OFF: [&str; 2] = ["0", "off"];

/// The file system types a lock may sit on:
/// disks and memory of this machine.
/// A network or user-space file system can be
///  written by a Git on another machine,
/// whose process numbers mean nothing here.
const LOCAL_FILE_SYSTEMS: [&str; 8] = ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs"];

/// How far a file's birth may precede the clock reading taken before a child started,
/// for the file to count as born after
///  the child started.
/// The kernel stamps files from a coarse clock that lags the one `SystemTime` reads by up to one tick.
///  Measured on this host (`CONFIG_HZ=1000`,
/// so a tick is 1 ms):
/// a new file's birth time lagged a clock read taken just before
///  the create by at most 1.684 ms and 1.729 ms in two runs of 10,000 creates each,
/// so 50 ms is about thirty times the
///  longest lag.
/// Kept;
/// recorded in `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Margins for removing left-behind locks"
///  (the coordinator's pick under the owner's delegation,
/// open to the owner's veto).
pub const COARSE_CLOCK_SLACK: Duration = Duration::from_millis(50);

/// How far after the lock's birth a process must have started to count as a later user of the same number,
/// beyond the
///  clock's resolution.
/// Measured:
/// the start time derived from `/proc/<pid>/stat`,
/// `/proc/uptime` and the clock read together
///  exceeded the clock read just before the process was spawned by at most 2.331 ms and 2.329 ms in two runs of 1,000 spawns,
/// so
///  100 ms is over forty times the longest error beyond the 20 ms resolution.
/// Kept;
/// the coordinator's pick under the owner's
///  delegation,
/// open to the owner's veto.
const REUSE_SLACK_MS: f64 = 100.0;

/// The command name of Git,
/// and of its helpers (`git-remote-https`),
/// as the kernel reports it.
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
    /// The staging file `packed-refs.new` has no lock beside it,
    /// and Git creates it only while holding that lock.
    OrphanStaging,
    /// A member of the tree this wrapper stopped was seen holding the lock,
    /// by the pid file beside it,
    /// before the stop signalled it,
    /// and that member has ended;
    /// the lock is the same file.
    ClaimedBeforeStop(i64),
}

/// What a judgement concluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// Younger than the margin,
    /// or not a regular file:
    /// left,
    /// and not mentioned.
    TooNew,
    /// A running process holds it;
    /// the text names the process and how that is known.
    Held(String),
    /// Nothing proves the holder gone;
    /// the text says why.
    Unproven(String),
    /// Removable,
    /// on this proof.
    Stale(Proof),
}

/// The evidence a judgement may use,
/// and the limits of what it can see.
#[derive(Clone, Debug)]
pub struct Judge {
    /// The process table.
    pub proc_root: PathBuf,
    /// How old a lock must be before it is judged at all.
    pub margin: Duration,
    /// Whether the process numbers read here mean the same processes as in the pid files.
    pub pid_view_complete: bool,
    /// When the machine booted by the wall clock,
    /// where known.
    /// Used only to say that a lock looks older than the boot,
    ///  never as a proof:
    /// a correction of the clock moves it.
    pub boot: Option<SystemTime>,
    /// The file system types a lock may sit on.
    pub local_file_systems: Vec<String>,
    /// The ended processes of a child this wrapper stopped,
    /// where the judgement follows a stop.
    pub own_ended: Option<OwnEnded>,
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
    /// When the lock was born,
    /// where known.
    pub born: Option<SystemTime>,
}

impl Judge {
    /// The judge of this process:
    /// the real process table,
    /// the real boot time,
    /// and a complete view of the process numbers
    ///  only where this process runs in the initial PID namespace and its process table hides no process.
    pub fn here() -> Self {
        let mut local_file_systems: Vec<String> = Vec::new();
        for name in LOCAL_FILE_SYSTEMS {
            local_file_systems.push(String::from(name));
        }
        return Self {
            proc_root: PathBuf::from(PROC_MOUNT_POINT),
            margin: MARGIN,
            pid_view_complete: current_pid_namespace().as_deref() == Some(INITIAL_PID_NAMESPACE) && proc_hides_no_process(),
            boot: boot_time(),
            local_file_systems,
            own_ended: None,
        };
    }

    /// Whether the file system holding `path` is on the local list;
    /// the reason when it is not or cannot be told.
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

    /// The pid file beside `lock`,
    /// as evidence about this lock's holder.
    pub(crate) fn pid_file_state(lock: &Path, lock_born: Option<SystemTime>) -> PidFile {
        let pid_path: PathBuf = pid_file_beside(lock);
        let Some(pid) = pid_named_by(pid_path.as_path()) else {
            return PidFile::Absent;
        };
        let pid_born: Option<SystemTime> = match std::fs::symlink_metadata(pid_path.as_path()) {
            Ok(metadata) => metadata.created().ok(),
            Err(_) => None,
        };
        return match (lock_born, pid_born) {
            // Git creates the pid file after the lock, so a pid file born earlier belongs to an earlier holder.
            (Some(lock_at), Some(pid_at)) if pid_at >= lock_at => PidFile::Names(i64::from(pid)),
            _ => PidFile::Older(i64::from(pid)),
        };
    }

    /// What the pid file's process is now:
    /// gone,
    /// a program that is not Git,
    /// a Git,
    /// or not known.
    ///  A number that is gone or reused here proves the writer gone only when no process of the table has that number
    ///  inside a PID namespace of its own,
    /// because the pid file holds the number its writer had in the writer's namespace.
    fn judge_pid(&self, pid: i64, lock_born: Option<SystemTime>) -> Verdict {
        if !self.pid_view_complete {
            if self.names_ended_own_child(pid, lock_born) {
                return self.judge_own_ended_pid(pid);
            }
            return Verdict::Unproven(format!(
                "the pid file beside it names process {pid}, but this process runs in a PID namespace of its own, or cannot see every process of the machine, so it cannot tell whether {pid} is gone"
            ));
        }
        let verdict: Verdict = self.judge_listed_pid(pid, lock_born);
        if !matches!(verdict, Verdict::Stale(_)) {
            return verdict;
        }
        return match nested_holders_of(self.proc_root.as_path(), pid) {
            Ok(holders) if holders.is_empty() => verdict,
            Ok(holders) => Verdict::Unproven(format!(
                "the pid file beside it names process {pid}, and process {} is process {pid} of a PID namespace of its own, so the writer may be that process",
                holders.first().copied().unwrap_or_default()
            )),
            Err(reason) => Verdict::Unproven(format!(
                "the pid file beside it names process {pid}, and the processes could not be searched for one that is process {pid} of a PID namespace of its own: {reason}"
            )),
        };
    }

    /// Whether the pid file names a process of the tree this wrapper stopped and saw end,
    /// and the lock was born after that
    ///  child started:
    /// then the number is the wrapper's own namespace's,
    /// and it was the child's.
    fn names_ended_own_child(&self, pid: i64, lock_born: Option<SystemTime>) -> bool {
        let Some(own) = self.own_ended.as_ref() else {
            return false;
        };
        if !own.pids.contains(&pid) {
            return false;
        }
        return match lock_born {
            Some(born) => born + COARSE_CLOCK_SLACK >= own.since,
            None => false,
        };
    }

    /// The verdict for a number of a process this wrapper saw end,
    /// when the table is not known to be the machine's:
    /// it is
    ///  gone if the table shows nothing with that number now,
    /// and anything running with it is left alone.
    fn judge_own_ended_pid(&self, pid: i64) -> Verdict {
        return match linux_start(pid, self.proc_root.as_path(), now_ms()) {
            ProcessStart::Missing => Verdict::Stale(Proof::PidGone(pid)),
            ProcessStart::Unknown(reason) => Verdict::Unproven(format!("the pid file beside it names process {pid}, which could not be looked at: {reason}")),
            ProcessStart::Running { .. } => Verdict::Held(format!("process {pid}, which the pid file beside it names, ended with the child this wrapper stopped, and a process has the number again")),
        };
    }

    /// What the pid file's process is in the process table as listed:
    /// gone,
    /// a program that is not Git,
    /// a Git,
    /// or not known.
    fn judge_listed_pid(&self, pid: i64, lock_born: Option<SystemTime>) -> Verdict {
        return match linux_start(pid, self.proc_root.as_path(), now_ms()) {
            ProcessStart::Missing => Verdict::Stale(Proof::PidGone(pid)),
            ProcessStart::Unknown(reason) => Verdict::Unproven(format!("the pid file beside it names process {pid}, which could not be looked at: {reason}")),
            ProcessStart::Running { started_at_ms, resolution_ms } => {
                let command: String = match std::fs::read_to_string(self.proc_root.join(pid.to_string()).join("comm")) {
                    Ok(text) => text.trim().to_owned(),
                    Err(_) => String::new(),
                };
                // The number was reused only if the program is not Git AND it started after the lock was made; when the two
                //  disagree (a Git under another name, a clock that moved) the process is taken for the holder.
                let born_ms: Option<f64> = match lock_born {
                    Some(at) => match at.duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(elapsed) => Some(elapsed.as_secs_f64() * 1000.0),
                        Err(_) => None,
                    },
                    None => None,
                };
                let started_after: bool = match born_ms {
                    Some(born) => started_at_ms > born + resolution_ms as f64 + REUSE_SLACK_MS,
                    None => false,
                };
                if command.starts_with(GIT_COMMAND_PREFIX) || !started_after {
                    Verdict::Held(format!("process {pid} ({command}), which the pid file beside it names, is running"))
                } else {
                    Verdict::Stale(Proof::PidReused { pid, command })
                }
            }
        };
    }

    /// Whether the lock looks older than the boot by the wall clock.
    /// Only words the notice:
    /// a forward correction of the clock
    ///  during the boot makes a lock that a live Git holds look older than the boot,
    /// so no lock is removed on it.
    fn looks_older_than_boot(&self, born: Option<SystemTime>, identity: &Identity) -> bool {
        let Some(boot) = self.boot else {
            return false;
        };
        let modified: SystemTime = nanoseconds_to_time(identity.modified_ns);
        return match born {
            Some(at) => at < boot && modified < boot,
            None => false,
        };
    }

    /// The first process of this account that the scan could not look into and that is a Git (or whose name cannot be read
    ///  back):
    /// a program that is not Git,
    /// such as the user's service manager,
    /// cannot hold a Git lock.
    /// The reason text of the
    ///  scan starts `PID <number>`.
    fn unreadable_git(&self, scan: &UseScan) -> Option<String> {
        for reason in &scan.unreadable {
            let number: Option<&str> = match reason.strip_prefix("PID ") {
                Some(rest) => rest.split(' ').next(),
                None => None,
            };
            let command: Option<String> = match number {
                Some(pid) => std::fs::read_to_string(self.proc_root.join(pid).join("comm")).ok(),
                None => None,
            };
            match (number, command) {
                (Some(_), None) => {}
                (_, Some(name)) if !name.trim().starts_with(GIT_COMMAND_PREFIX) => {}
                _ => return Some(reason.clone()),
            }
        }
        return None;
    }

    /// Judge the staging file `packed-refs.new` at `staging`,
    /// which has no lock beside it:
    /// old enough,
    /// on a local disk,
    /// and
    ///  open in no process.
    /// The caller has checked that the lock is absent.
    pub fn examine_staging(&self, staging: &Path) -> Option<Examined> {
        let identity: Identity = identity_of(staging)?;
        let born: Option<SystemTime> = std::fs::symlink_metadata(staging).ok()?.created().ok();
        let newest: SystemTime = newest_time(born, &identity);
        if SystemTime::now().duration_since(newest).unwrap_or_default() < self.margin {
            return verdict_with(Verdict::TooNew, PidFile::Absent, identity, born);
        }
        if let Err(reason) = self.local_check(staging) {
            return verdict_with(Verdict::Unproven(reason), PidFile::Absent, identity, born);
        }
        let scan: UseScan = processes_using_through(self.proc_root.as_path(), &UseTarget::File { device: identity.device, inode: identity.inode }, std::process::id());
        if let Some(reason) = scan.unavailable.as_ref() {
            return verdict_with(Verdict::Unproven(format!("the process table could not be listed: {reason}")), PidFile::Absent, identity, born);
        }
        if let Some(user) = scan.users.first() {
            return verdict_with(Verdict::Held(format!("process {} ({}) has the file open", user.pid, user.command.clone().unwrap_or_default())), PidFile::Absent, identity, born);
        }
        return verdict_with(Verdict::Stale(Proof::OrphanStaging), PidFile::Absent, identity, born);
    }

    /// Judge the lock at `lock`;
    /// nothing when it is not a regular file any more.
    pub fn examine(&self, lock: &Path) -> Option<Examined> {
        let identity: Identity = identity_of(lock)?;
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(lock).ok()?;
        let born: Option<SystemTime> = metadata.created().ok();
        let pid_file: PidFile = Self::pid_file_state(lock, born);

        if !metadata.is_file() {
            return verdict_with(Verdict::TooNew, pid_file.clone(), identity, born);
        }
        let newest: SystemTime = newest_time(born, &identity);
        // A lock whose pid file names a process this wrapper stopped and saw end needs no margin: its holder is known to be gone.
        let claimed: Option<i64> = match self.own_ended.as_ref() {
            Some(own) => own.holder_of_claimed(lock, &identity),
            None => None,
        };
        let own: bool = claimed.is_some() || matches!(&pid_file, PidFile::Names(pid) if self.names_ended_own_child(*pid, born));
        if !own && SystemTime::now().duration_since(newest).unwrap_or_default() < self.margin {
            return verdict_with(Verdict::TooNew, pid_file.clone(), identity, born);
        }
        if let Err(reason) = self.local_check(lock) {
            return verdict_with(Verdict::Unproven(reason), pid_file.clone(), identity, born);
        }
        let scan: UseScan = processes_using_through(self.proc_root.as_path(), &UseTarget::File { device: identity.device, inode: identity.inode }, std::process::id());
        if let Some(reason) = scan.unavailable {
            return verdict_with(Verdict::Unproven(format!("the process table could not be listed: {reason}")), pid_file.clone(), identity, born);
        }
        if let Some(user) = scan.users.first() {
            return verdict_with(Verdict::Held(format!("process {} ({}) has the file open", user.pid, user.command.clone().unwrap_or_default())), pid_file.clone(), identity, born);
        }
        // A holder the wrapper itself saw hold this very file, and then saw end, needs no inference about the processes it cannot look into.
        if let Some(pid) = claimed {
            return verdict_with(Verdict::Stale(Proof::ClaimedBeforeStop(pid)), pid_file.clone(), identity, born);
        }
        if let Some(reason) = self.unreadable_git(&scan) {
            return verdict_with(Verdict::Unproven(format!("a Git process of this account could not be looked into ({reason}), so it may hold the lock")), pid_file.clone(), identity, born);
        }
        if let PidFile::Names(pid) = pid_file {
            return verdict_with(self.judge_pid(pid, born), pid_file.clone(), identity, born);
        }
        let mut why: String = match pid_file {
            PidFile::Older(pid) => format!("the pid file beside it (process {pid}) is older than the lock, so it belongs to an earlier holder"),
            _ => String::from("no pid file beside it names a holder (Git writes one only under `core.lockfilePid`), and no process has it open"),
        };
        if self.looks_older_than_boot(born, &identity) {
            why.push_str("; its times are earlier than the start of this boot, which a reboot would explain, but so would a correction of the clock, so the times alone prove nothing");
        }
        return verdict_with(Verdict::Unproven(why), pid_file.clone(), identity, born);
    }
}

/// Whether the mount of `/proc` shows this account every process:
/// it has no `hidepid` option,
/// or one that hides nothing.
///  A table that hides processes lists fewer than exist,
/// so a number missing from it is no proof.
/// Where the mount cannot be
///  read the answer is no.
fn proc_hides_no_process() -> bool {
    let Ok(Some(mount)) = read_mount_facts(Path::new(PROC_MOUNT_POINT)) else {
        return false;
    };
    for option in mount.mount_options.iter().chain(mount.super_options.iter()) {
        if let Some(value) = option.strip_prefix(HIDEPID_OPTION)
            && !HIDEPID_OFF.contains(&value)
        {
            return false;
        }
    }
    return true;
}

/// The later of the lock's birth (when known) and its last modification.
fn newest_time(born: Option<SystemTime>, identity: &Identity) -> SystemTime {
    let modified: SystemTime = nanoseconds_to_time(identity.modified_ns);
    return match born {
        Some(at) => at.max(modified),
        None => modified,
    };
}

/// The result of judging one file:
/// the verdict with the facts gathered on the way.
fn verdict_with(verdict: Verdict, pid_file: PidFile, identity: Identity, born: Option<SystemTime>) -> Option<Examined> {
    return Some(Examined { verdict, pid_file, identity, born });
}

/// A time given as nanoseconds since the epoch.
fn nanoseconds_to_time(nanoseconds: i128) -> SystemTime {
    let magnitude: u64 = u64::try_from(nanoseconds.unsigned_abs()).unwrap_or(u64::MAX);
    return if nanoseconds >= 0 { SystemTime::UNIX_EPOCH + Duration::from_nanos(magnitude) } else { SystemTime::UNIX_EPOCH - Duration::from_nanos(magnitude) };
}

/// When the machine booted:
/// the `btime` line of `/proc/stat`,
/// in whole seconds since the epoch.
fn boot_time() -> Option<SystemTime> {
    let text: String = std::fs::read_to_string("/proc/stat").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("btime ")
            && let Ok(seconds) = rest.trim().parse::<u64>()
        {
            return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds));
        }
    }
    return None;
}

/// Controls on the margins stay out of the release executable.
#[cfg(test)]
#[path = "stale_ref_lock_judge_tests.rs"]
mod tests;

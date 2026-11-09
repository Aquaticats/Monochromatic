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
use crate::stale_ref_lock_judge_environment::{INITIAL_PID_NAMESPACE, PROC_MOUNT_POINT, boot_time, nanoseconds_to_time, newest_time, proc_hides_no_process};
use crate::process_start::{ProcessStart, linux_start, now_ms};
use crate::process_use::{ProcessUser, UseScan, UseTarget, processes_using_through};
use crate::git_command_name::names_git_command;
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

/// The file system types a lock may sit on:
/// disks and memory of this machine.
/// A network or user-space file system can be
///  written by a Git on another machine,
/// whose process numbers mean nothing here.
const LOCAL_FILE_SYSTEMS: [&str; 8] = ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs"];

/// Why a lock may be removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Proof {
    /// The process the pid file names is gone.
    PidGone(i64),
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
    /// The process numbers a scan for open holders may consider,
    /// where a caller judges one fixture and nothing else.
    /// Every process of the machine is evidence about a lock,
    /// so a control that plants its own would depend on every other program's,
    /// a Git the account cannot look into vetoing the clearance
    /// whatever repository it works in.
    /// `None` is every process of the table,
    /// which is what the wrapper itself reads.
    pub scan_scope: Option<Vec<i64>>,
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
            scan_scope: None,
        };
    }

    /// The processes that may use `identity`'s file,
    /// among every process of the table or only among the numbers this judge was given.
    /// A file is looked for by the device number and the inode it has,
    /// which a descriptor of any process carries whatever mount it was reached through.
    /// A reason about a process outside the scope is dropped with the process it names:
    /// what the scan could not read of another program cannot be a holder this judgement is about.
    fn users_of(&self, identity: &Identity) -> UseScan {
        let target: UseTarget = UseTarget::File { device: identity.device, inode: identity.inode };
        let mut scan: UseScan = processes_using_through(self.proc_root.as_path(), &target, std::process::id());
        let Some(scope) = self.scan_scope.as_ref() else {
            return scan;
        };
        let mut users: Vec<ProcessUser> = Vec::new();
        for user in scan.users {
            if scope.contains(&user.pid) {
                users.push(user);
            }
        }
        scan.users = users;
        let mut unreadable: Vec<String> = Vec::new();
        for reason in scan.unreadable {
            let Some(pid) = pid_of_reason(reason.as_str()) else {
                continue;
            };
            if scope.contains(&pid) {
                unreadable.push(reason);
            }
        }
        scan.unreadable = unreadable;
        return scan;
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
            return Verdict::Unproven(self.incomplete_view_reason(pid));
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

    /// Why a pid file proves nothing when the process numbers read here are not known to be the machine's.
    /// The writer wrote the number of its own PID namespace, and Git records no namespace,
    /// so a Git of a sibling namespace that shares the repository may have the same number.
    /// A lock born after a child this wrapper stopped had started is no more that child's than the sibling's:
    /// a matching number and a birth order do not say who wrote the file
    /// (cross-vendor review of 2026-11-09,
    /// finding 2).
    /// The one proof the wrapper holds itself is a lock it saw a member of a stopped tree hold,
    /// pinned by the file's own identity (`Proof::ClaimedBeforeStop`).
    fn incomplete_view_reason(&self, pid: i64) -> String {
        let only_a_number: String = format!(
            "the pid file beside it names process {pid}, but this process runs in a PID namespace of its own, or cannot see every process of the machine, so it cannot tell whether {pid} is gone"
        );
        let Some(own) = self.own_ended.as_ref() else {
            return only_a_number;
        };
        if !own.pids.contains(&pid) {
            return only_a_number;
        }
        return format!(
            "{only_a_number}; that number is one a tree this wrapper stopped had, and a Git of another PID namespace that shares this repository may have the same number, so the number alone does not say which writer made the lock"
        );
    }

    /// What the pid file's process is in the process table as listed:
    /// gone,
    /// a Git,
    /// a program that is not Git,
    /// or not known.
    /// A number that a running program has proves the writer gone only when that program cannot be the writer,
    /// and its command name does not say that:
    /// Git may run under another name,
    /// and the one sign that would tell the two apart is when the process started,
    /// which this host derives from the wall clock,
    /// so a correction of the clock moves it against the lock's birth and the same evidence would answer both ways
    /// (cross-vendor review of 2026-11-09,
    /// finding 1).
    /// The start is therefore said and never decides:
    /// a program that is not named like Git is not proven the holder,
    /// and not proven a later user of the number either.
    fn judge_listed_pid(&self, pid: i64, lock_born: Option<SystemTime>) -> Verdict {
        return match linux_start(pid, self.proc_root.as_path(), now_ms()) {
            ProcessStart::Missing => Verdict::Stale(Proof::PidGone(pid)),
            ProcessStart::Unknown(reason) => Verdict::Unproven(format!("the pid file beside it names process {pid}, which could not be looked at: {reason}")),
            ProcessStart::Running { started_at_ms, resolution_ms } => {
                let command: String = match std::fs::read_to_string(self.proc_root.join(pid.to_string()).join("comm")) {
                    Ok(text) => text.trim().to_owned(),
                    Err(_) => String::new(),
                };
                if names_git_command(command.as_str()) {
                    return Verdict::Held(format!("process {pid} ({command}), which the pid file beside it names, is running"));
                }
                let named: String = if command.is_empty() {
                    format!("process {pid}")
                } else {
                    format!("process {pid} (`{command}`)")
                };
                let moved: &str = if self.starts_after(lock_born, started_at_ms, resolution_ms) {
                    ", which this host's clock puts after the lock was made, which a later program that took the number would explain and so would a correction of the clock"
                } else {
                    ""
                };
                Verdict::Unproven(format!(
                    "the pid file beside it names {named}, which is running and is not named like Git{moved}; a Git running under another name looks the same, so nothing here proves the writer gone"
                ))
            }
        };
    }

    /// Whether this host's clock puts the process's start after the lock's birth,
    /// beyond the clock's resolution.
    /// Only worded:
    /// no verdict rests on it,
    /// because a correction of the clock moves the derived start and nothing else.
    fn starts_after(&self, lock_born: Option<SystemTime>, started_at_ms: f64, resolution_ms: i64) -> bool {
        let Some(at) = lock_born else {
            return false;
        };
        let Ok(elapsed) = at.duration_since(SystemTime::UNIX_EPOCH) else {
            return false;
        };
        return started_at_ms > elapsed.as_secs_f64() * 1000.0 + resolution_ms as f64;
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
            let Some(pid) = pid_of_reason(reason.as_str()) else {
                return Some(reason.clone());
            };
            let command: Option<String> = std::fs::read_to_string(self.proc_root.join(pid.to_string()).join("comm")).ok();
            match command {
                None => {}
                Some(name) if !names_git_command(name.trim()) => {}
                Some(_) => return Some(reason.clone()),
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
        let scan: UseScan = self.users_of(&identity);
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
        let own: bool = claimed.is_some();
        if !own && SystemTime::now().duration_since(newest).unwrap_or_default() < self.margin {
            return verdict_with(Verdict::TooNew, pid_file.clone(), identity, born);
        }
        if let Err(reason) = self.local_check(lock) {
            return verdict_with(Verdict::Unproven(reason), pid_file.clone(), identity, born);
        }
        let scan: UseScan = self.users_of(&identity);
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

/// The process number an unreadable process's reason starts with,
/// as the scan words it (`PID <number> ...`).
fn pid_of_reason(reason: &str) -> Option<i64> {
    let rest: &str = reason.strip_prefix("PID ")?;
    return rest.split(' ').next()?.parse::<i64>().ok();
}

/// The result of judging one file:
/// the verdict with the facts gathered on the way.
fn verdict_with(verdict: Verdict, pid_file: PidFile, identity: Identity, born: Option<SystemTime>) -> Option<Examined> {
    return Some(Examined { verdict, pid_file, identity, born });
}

/// Controls on the margins stay out of the release executable.
#[cfg(test)]
#[path = "stale_ref_lock_judge_tests.rs"]
mod tests;

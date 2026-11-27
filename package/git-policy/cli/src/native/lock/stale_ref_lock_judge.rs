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
//! Neither is a pid file:
//!  Git writes it with a process number and no namespace,
//! and only the birth order of the two files could tie that number to this lock,
//! which a step of the clock turns around,
//! so the pid file is read for the words of the notice and decides nothing
//! (cross-vendor review of 2026-11-09,
//! finding 3).
//! The only removal left here is the staging file of the packed references.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const examined = judge.examine(lockPath); // { verdict, pidFile, size, born, identity }
//! ```

use crate::fs_mount_options::read_mount_facts;
use crate::git_command_name::names_git_command;
use crate::process_use::{
    ProcessUser,
    UseScan,
    UseTarget,
    processes_using_through,
};
use crate::stale_ref_lock_found::PidFile;
/// What a judgement concluded and why a lock may be removed,
/// re-exported so a caller of this module names them here.
pub use crate::stale_ref_lock_found::{
    Proof,
    Verdict,
};
pub use crate::stale_ref_lock_identity::{
    Identity,
    identity_of,
};
use crate::stale_ref_lock_judge_environment::{
    PROC_MOUNT_POINT,
    boot_time,
    newest_time,
};
use crate::stale_ref_lock_judge_words::{
    looks_older_than_boot,
    process_state_words,
};
use crate::stale_ref_lock_names::{
    pid_file_beside,
    pid_named_by,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    SystemTime,
};

/// The file system types a lock may sit on,
/// which the abandoned-lock proof reads as well.
pub(crate) use crate::stale_ref_lock_judge_limits::LOCAL_FILE_SYSTEMS;
/// The shipped limits a judgement rests on,
/// re-exported so a caller of this module names them here.
pub use crate::stale_ref_lock_judge_limits::{
    LONGEST_LIVE_REF_STEP,
    MARGIN,
    MARGIN_MULTIPLE,
};

/// The evidence a judgement may use,
/// and the limits of what it can see.
#[derive(Clone, Debug)]
pub struct Judge {
    /// The process table.
    pub proc_root: PathBuf,
    /// How old a lock must be before it is judged at all.
    pub margin: Duration,
    /// When the machine booted by the wall clock,
    /// where known.
    /// Used only to say that a lock looks older than the boot,
    ///  never as a proof:
    /// a correction of the clock moves it.
    pub boot: Option<SystemTime>,
    /// The file system types a lock may sit on.
    pub local_file_systems: Vec<String>,
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

/// What:
///  What one PID file says about the lock beside it,
///  from the two births alone.
/// Why:
///   Git creates the PID file after it takes the lock,
///  so a PID file born strictly later is this lock's,
///  and one born strictly earlier is an earlier holder's.
///  A file system may record the two in one tick of its clock,
///  or record no birth time at all,
///  and then the order is not proven:
///  an equal time is never read as "later",
/// and a missing one is never read as "earlier"
///  (`AGENTS.md`,
/// rule `NHP`;
///  a review of `d189162e3e` found the equal case removing a live Git's editor-phase lock,
///  which this file measured at one tick of this volume's clock).
///  Everything else is that unproven state,
///  which a caller that would remove a lock reads as "keep it".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pidFileStanding(lockBorn, pidBorn, pid) // later ? Names : earlier ? Older : Unproven
/// ```
pub(crate) fn pid_file_standing(
    lock_born: Option<SystemTime>,
    pid_born: Option<SystemTime>,
    pid: i64,
) -> PidFile {
    return match (lock_born, pid_born) {
        (Some(lock_at), Some(pid_at)) if pid_at > lock_at => PidFile::Names(pid),
        (Some(lock_at), Some(pid_at)) if pid_at < lock_at => PidFile::Older(pid),
        _ => PidFile::Unproven(pid),
    };
}

impl Judge {
    /// The judge of this process:
    /// the real process table,
    /// the real boot time,
    /// and the filesystems this boot mounts.
    pub fn here() -> Self {
        let mut local_file_systems: Vec<String> = Vec::new();
        for &name in LOCAL_FILE_SYSTEMS {
            local_file_systems.push(String::from(name));
        }
        return Self {
            proc_root: PathBuf::from(PROC_MOUNT_POINT),
            margin: MARGIN,
            boot: boot_time(),
            local_file_systems,
            scan_scope: None,
        };
    }

    /// The processes that may use `identity`'s file,
    /// among every process of the table or only among the numbers this judge was given.
    /// A file is looked for by the device number and the inode it has,
    /// which a descriptor of any process carries whatever mount it was reached through.
    /// A reason about a process outside the scope is dropped with the process it names:
    /// what the scan could not read of another program cannot be a holder this judgement is about.
    pub(crate) fn users_of(
        &self,
        identity: &Identity,
    ) -> UseScan {
        let target: UseTarget = UseTarget::File {
            device: identity.device,
            inode: identity.inode,
        };
        let mut scan: UseScan =
            processes_using_through(self.proc_root.as_path(), &target, std::process::id());
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
        let mut unresolved: Vec<String> = Vec::new();
        for reason in scan.unresolved {
            let Some(pid) = pid_of_reason(reason.as_str()) else {
                continue;
            };
            if scope.contains(&pid) {
                unresolved.push(reason);
            }
        }
        scan.unresolved = unresolved;
        return scan;
    }

    /// Whether the file system holding `path` is on the local list;
    /// the reason when it is not or cannot be told.
    pub(crate) fn local_check(
        &self,
        path: &Path,
    ) -> Result<(), String> {
        let facts = read_mount_facts(path)?;
        let Some(mount) = facts else {
            return Err(String::from(
                "the file system that holds it is not in the mount table",
            ));
        };
        if self.local_file_systems.contains(&mount.filesystem_type) {
            return Ok(());
        }
        return Err(format!(
            "it sits on a `{}` file system, which a Git on another machine may share",
            mount.filesystem_type
        ));
    }

    /// The pid file beside `lock`,
    /// as evidence about this lock's holder.
    pub(crate) fn pid_file_state(
        lock: &Path,
        lock_born: Option<SystemTime>,
    ) -> PidFile {
        let pid_path: PathBuf = pid_file_beside(lock);
        let Some(pid) = pid_named_by(pid_path.as_path()) else {
            return PidFile::Absent;
        };
        return pid_file_standing(
            lock_born,
            crate::lock_birth_times::birth_time_of(pid_path.as_path()),
            i64::from(pid),
        );
    }

    /// The first process of this account that the scan could not look into and that is a Git (or whose name cannot be read
    ///  back):
    /// a program that is not Git,
    /// such as the user's service manager,
    /// cannot hold a Git lock.
    /// The reason text of the
    ///  scan starts `PID <number>`.
    fn unreadable_git(
        &self,
        scan: &UseScan,
    ) -> Option<String> {
        for reason in &scan.unreadable {
            let Some(pid) = pid_of_reason(reason.as_str()) else {
                return Some(reason.clone());
            };
            let command: Option<String> =
                std::fs::read_to_string(self.proc_root.join(pid.to_string()).join("comm")).ok();
            match command {
                None => {}
                Some(name) if !names_git_command(name.trim()) => {}
                Some(_) => return Some(reason.clone()),
            }
        }
        return None;
    }

    /// Judge the lock at `lock`;
    /// nothing when it is not a regular file any more.
    pub fn examine(
        &self,
        lock: &Path,
    ) -> Option<Examined> {
        let identity: Identity = identity_of(lock)?;
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(lock).ok()?;
        let born: Option<SystemTime> = crate::lock_birth_times::birth_time_of(lock);
        let pid_file: PidFile = Self::pid_file_state(lock, born);

        if !metadata.is_file() {
            return verdict_with(Verdict::TooNew, pid_file.clone(), identity, born);
        }
        let newest: SystemTime = newest_time(born, &identity);
        if SystemTime::now().duration_since(newest).unwrap_or_default() < self.margin {
            return verdict_with(Verdict::TooNew, pid_file.clone(), identity, born);
        }
        if let Err(reason) = self.local_check(lock) {
            return verdict_with(Verdict::Unproven(reason), pid_file.clone(), identity, born);
        }
        let scan: UseScan = self.users_of(&identity);
        if let Some(reason) = scan.unavailable {
            return verdict_with(
                Verdict::Unproven(format!("the process table could not be listed: {reason}")),
                pid_file.clone(),
                identity,
                born,
            );
        }
        if let Some(user) = scan.users.first() {
            return verdict_with(
                Verdict::Held(format!(
                    "process {} ({}) has the file open",
                    user.pid,
                    user.command.clone().unwrap_or_default()
                )),
                pid_file.clone(),
                identity,
                born,
            );
        }
        if let Some(reason) = self.unreadable_git(&scan) {
            return verdict_with(
                Verdict::Unproven(format!(
                    "a Git process of this account could not be looked into ({reason}), so it may hold the lock"
                )),
                pid_file.clone(),
                identity,
                born,
            );
        }
        if let Some(reason) = scan.unresolved.first() {
            return verdict_with(
                Verdict::Unproven(format!(
                    "a descriptor whose record names this file could not be copied out of the process that holds it ({reason}), so that process may hold the lock"
                )),
                pid_file.clone(),
                identity,
                born,
            );
        }
        // A pid file is read for these words and decides nothing: its number names a process of the writer's own PID
        // namespace, Git records no namespace, and only the birth order of the two files could tie the number to this lock,
        // which a correction of the clock turns around (cross-vendor review of 2026-11-09, finding 3).
        let mut why: String = match pid_file {
            PidFile::Names(pid) => format!(
                "a pid file beside it names process {pid}, which {}, and nothing ties that number to this lock: Git writes a number and no namespace, and only the times of the two files could tie them, which a correction of the clock turns around",
                process_state_words(self.proc_root.as_path(), pid)
            ),
            PidFile::Older(pid) => format!(
                "the pid file beside it names process {pid}, which {}, and its times are earlier than the lock's, which fits an earlier holder as well as a correction of the clock, so it proves nothing",
                process_state_words(self.proc_root.as_path(), pid)
            ),
            PidFile::Unproven(pid) => format!(
                "a pid file beside it names process {pid}, which {}, and nothing proves it was written for this lock: its own birth time and the lock's cannot be told apart, or this file system records no birth time for one of them",
                process_state_words(self.proc_root.as_path(), pid)
            ),
            PidFile::Absent => String::from(
                "no pid file beside it names a holder (Git writes one only under `core.lockfilePid`), and no process has it open",
            ),
        };
        if looks_older_than_boot(self.boot, born, &identity) {
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
pub(crate) fn verdict_with(
    verdict: Verdict,
    pid_file: PidFile,
    identity: Identity,
    born: Option<SystemTime>,
) -> Option<Examined> {
    return Some(Examined {
        verdict,
        pid_file,
        identity,
        born,
    });
}

/// Controls on the margins stay out of the release executable.
#[cfg(test)]
#[path = "stale_ref_lock_judge_tests.rs"]
mod tests;

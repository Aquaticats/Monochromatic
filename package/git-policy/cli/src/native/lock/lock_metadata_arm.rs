//! What:
//!  A control's arm over what a read of the abandoned lock's PID file reports,
//!  so each term the removal's last look compares can be made to differ on its own.
//! Why:
//!  Two of the three terms cannot be presented by a fixture on this host:
//!  a device number comes from the mount and a fixture has one,
//!  and a change time in whole milliseconds is the kernel's to give,
//!  so an in-place write shares the judged millisecond on this host's ZFS in 957 of 1000 pairs
//!  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
//!  cause L).
//!  A control that had to wait for that would be the timing lottery issue 6342 was filed about.
//!  The arm answers one term differently for the second read of one path and records the plain time otherwise,
//!  which is the shape `lock_birth_times.rs` uses for a coarse birth time:
//!  it is armed by a test on its own thread and inert in the shipped executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! withMetadataArm({ path: pidFile, reported: { device: 'another', inode: 'same', ctime: 'same' } }, () => clearOne(lock, commonDir));
//! ```

/// `LockMetadata` is what a removal compares:
/// a device,
/// an inode and a change time in whole milliseconds.
use crate::index_lock_evidence::LockMetadata;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// How long a tick the arm floors change times to lasts.
use std::time::Duration;

/// What:
///  Which term of a later read the arm answers differently,
/// Why:
///  A control names one term at a time,
///  so a run that makes two terms differ says nothing about which one the removal needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Changed = 'device' | 'change-time';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Changed {
    /// Another device number.
    Device,
    /// An earlier change time,
    /// one millisecond back.
    ChangeTime,
}

/// What:
///  The answer a control arms for the reads of one path.
/// Why:
///  The removal reads the PID file's metadata twice:
///  the judged file before the lock's unlink and the file at that name after it.
///  Everything but the armed term stays the file system's own answer,
///  and the bytes of the file are never touched by the arm,
/// so a control that also writes them writes a state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Arm = { path: string; coarseChangeTime?: Duration; everyInodeIs?: number; later?: Changed };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MetadataArm {
    /// The one path whose reads the arm touches.
    pub(crate) path: PathBuf,
    /// A tick every read of that path has its change time floored to,
    ///  which is a file system whose tick holds two writes.
    pub(crate) coarse_change_time: Option<Duration>,
    /// A number every read of that path reports instead of the inode's,
    ///  which is a file system that hands the freed number to the next file.
    pub(crate) every_inode_is: Option<u64>,
    /// The term a later read of that path reports differently.
    pub(crate) later: Option<Changed>,
}

thread_local! {
    /// The arm of the running thread,
    /// and how many reads of its path it has answered.
    static ARM: std::cell::RefCell<Option<(MetadataArm, usize)>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Run `body` with this arm over the reads of its path,
///  and put the thread's own arm back however the body ends.
/// Why:
///  A control that left an arm set would change the next control of the same process,
///  which is the defect `lock_birth_times.rs` avoids the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { arm.set(it); body(); } finally { arm.clear(); }
/// ```
pub(crate) fn with_arm<T>(arm: MetadataArm, body: impl FnOnce() -> T) -> T {
    let held: Option<(MetadataArm, usize)> = ARM.with(|cell| { return cell.replace(Some((arm, 0))); });
    let ended: T = body();
    ARM.with(|cell| { *cell.borrow_mut() = held; });
    return ended;
}

/// What:
///  The metadata of the file at `path`,
///  with the running thread's arm answering for it when the arm names that path.
/// Why:
///  One function answers the removal's two reads of the PID file,
///  so a control can say which term differs without touching what the file system records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const striped = armed(path, statx(path)); // the arm's term changed on the later read
/// ```
pub(crate) fn armed(path: &Path, found: Option<LockMetadata>) -> Option<LockMetadata> {
    return ARM.with(|cell| {
        let mut held = cell.borrow_mut();
        let Some((arm, reads)) = held.as_mut() else {
            return found;
        };
        if arm.path != path {
            return found;
        }
        *reads += 1;
        let later: bool = *reads > 1;
        let mut answered: LockMetadata = match found {
            Some(recorded) => recorded,
            None => return None,
        };
        if let Some(tick) = arm.coarse_change_time {
            let millis: i64 = i64::try_from(tick.as_millis()).unwrap_or(i64::MAX);
            if millis > 0 {
                answered.ctime_ms = answered.ctime_ms / millis * millis;
            }
        }
        if let Some(number) = arm.every_inode_is {
            answered.inode = number;
        }
        if later {
            match arm.later {
                Some(Changed::Device) => answered.device = answered.device.wrapping_add(1),
                Some(Changed::ChangeTime) => { answered.ctime_ms -= 1; }
                None => {}
            }
        }
        return Some(answered);
    });
}

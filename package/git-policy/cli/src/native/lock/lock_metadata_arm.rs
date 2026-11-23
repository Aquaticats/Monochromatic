//! What:
//!  A control's arm over what a read of the abandoned lock's PID file reports,
//!  so each term the removal's last look compares can be made to differ on its own.
//! Why:
//!  A device number comes from the mount and a fixture has one,
//!  a change time in whole milliseconds is the kernel's to give,
//!  and a file system may hand a freed inode number to the next file,
//!  so a control that leaned on any of the three would be the timing and file-system lottery issue 6342 was filed
//!  about.
//!  The arm answers one term for the named path's reads,
//!  which is the shape `lock_birth_times.rs` uses for a coarse birth time,
//!  and it tells the judged read from the removal's last look by the phase a control's hook moves it to,
//!  so a judged read taken after the lock's name went is answered as a later one.
//!  It is compiled out of the shipped executable:
//!  `lock/mod.rs` declares this module under `#[cfg(test)]`
//!  and the one call site in `clear_one` (`pid_file_metadata`) is under `#[cfg(test)]` too,
//!  so a release build contains neither it nor its symbols.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! withMetadataArm({ path: pidFile, reported: { device: 'another', inode: 'same', changeTime: 'the same' } }, () => clearOne(lock, commonDir));
//! ```

/// `LockMetadata` is what a removal compares:
/// a device,
/// an inode and a change time in whole milliseconds.
use crate::index_lock_evidence::LockMetadata;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Which term a later read of the armed path answers differently.
/// Why:
///  A control names one term at a time,
///  so a run that makes two terms differ says nothing about which one the removal needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Changed = 'device' | 'inode' | 'change-time';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Changed {
    /// Another device number.
    Device,
    /// Another inode number.
    Inode,
    /// An earlier change time,
    /// one millisecond back.
    ChangeTime,
}

/// What:
///  The answer a control arms for the reads of one path.
/// Why:
///  The removal reads the PID file's metadata twice,
///  once as the judged file before the lock's unlink and once as the file at that name after it,
///  and which of the two a read is comes from the phase the removal is in.
///  Everything the arm does not name stays the file system's own answer,
///  and the bytes of the file are never touched by the arm,
///  so a control that writes them writes a state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Arm = { path: string; everyReadReportsChangeTime?: number; everyInodeIs?: number; later?: Changed };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MetadataArm {
    /// The one path whose reads the arm touches.
    pub(crate) path: PathBuf,
    /// A change time every read of that path reports in place of the file system's,
    ///  so no clock decides what a control holds.
    pub(crate) every_read_reports_change_time: Option<i64>,
    /// A number every read of that path reports in place of the inode's,
    ///  which is a file system that hands the freed number to the next file.
    pub(crate) every_inode_is: Option<u64>,
    /// The term a read taken after the lock's name went reports differently.
    pub(crate) later: Option<Changed>,
}

/// What:
///  How far the removal the running thread's arm answers has gone.
/// Why:
///  The removal reads the armed path once as the file it judges, before the lock's name goes,
///  and once at its last look, after it.
///  A count of reads would answer a judged read taken after the lock's name went as the first one,
///  so a removal that read the file in the wrong place would still compare it against an armed difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Phase = 'judging' | 'at-the-last-look';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    /// The lock's name stands,
    /// so a read here is the file the removal judges.
    Judging,
    /// The lock's name has gone,
    /// so every read from here is the removal's last look.
    AtTheLastLook,
}

thread_local! {
    /// The arm of the running thread,
    /// and how far the removal it answers has gone.
    static ARM: std::cell::RefCell<Option<(MetadataArm, Phase)>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  The arm this thread had before `with_arm` took it,
///  put back when it is dropped.
/// Why:
///  A control that panics leaves its arm set without this,
///  and every later control of the same thread would read armed metadata.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PutBack { #held; drop() { arm.set(this.#held); } }
/// ```
struct PutBackWhenDropped(Option<(MetadataArm, Phase)>);

impl Drop for PutBackWhenDropped {
    fn drop(&mut self) {
        let held: Option<(MetadataArm, Phase)> = self.0.take();
        ARM.with(|cell| { *cell.borrow_mut() = held; });
    }
}

/// What:
///  Run `body` with this arm over the reads of its path,
///  and put the thread's own arm back when the body returns or panics.
/// Why:
///  A control that left an arm set would change the next control of the same process,
///  and a control that panics inside the body is the case worth holding:
///  a drop guard puts the arm back on the way out of the unwind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { arm.set(it); body(); } finally { arm.putBack(); }
/// ```
pub(crate) fn with_arm<T>(arm: MetadataArm, body: impl FnOnce() -> T) -> T {
    let put_back: PutBackWhenDropped = PutBackWhenDropped(ARM.with(|cell| { return cell.replace(Some((arm, Phase::Judging))); }));
    let ended: T = body();
    drop(put_back);
    return ended;
}

/// What:
///  Say that the removal whose reads this thread's arm answers has unlinked the lock's name.
/// Why:
///  A control's hook runs at that instant,
///  and from here every read of the armed path is the removal's last look:
///  a judged read taken after the lock's name went is answered as a later one,
///  so a removal that reads the file in the wrong place fails its control instead of passing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// arm.setPhase('at-the-last-look');
/// ```
pub(crate) fn the_locks_name_is_gone() {
    return ARM.with(|cell| {
        let mut held = cell.borrow_mut();
        let Some((_arm, phase)) = held.as_mut() else {
            return;
        };
        *phase = Phase::AtTheLastLook;
    });
}

/// What:
///  The metadata of the file at `path`,
///  with the running thread's arm answering for it when the arm names that path.
/// Why:
///  One function answers the removal's reads of the PID file,
///  so a control can say which term differs without touching what the file system records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const striped = armed(path, statx(path)); // the arm's term changed on the later read
/// ```
pub(crate) fn armed(path: &Path, found: Option<LockMetadata>) -> Option<LockMetadata> {
    return ARM.with(|cell| {
        let held = cell.borrow();
        let Some((arm, phase)) = held.as_ref() else {
            return found;
        };
        if arm.path != path {
            return found;
        }
        let later: bool = *phase == Phase::AtTheLastLook;
        let mut answered: LockMetadata = match found {
            Some(recorded) => recorded,
            None => return None,
        };
        if let Some(change_time) = arm.every_read_reports_change_time {
            answered.ctime_ms = change_time;
        }
        if let Some(number) = arm.every_inode_is {
            answered.inode = number;
        }
        if later {
            match arm.later {
                Some(Changed::Device) => answered.device = answered.device.wrapping_add(1),
                Some(Changed::Inode) => answered.inode = answered.inode.wrapping_add(1),
                Some(Changed::ChangeTime) => { answered.ctime_ms -= 1; }
                None => {}
            }
        }
        return Some(answered);
    });
}

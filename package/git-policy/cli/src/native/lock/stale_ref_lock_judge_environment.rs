//! What:
//!  What one judgement reads of the machine it runs on:
//! when the machine booted,
//! what its process table shows this account,
//! and the time a file carries.
//! Why:
//!  These readings decide what a judgement may conclude,
//! not what it is given,
//! so they sit beside the judge rather than in it:
//! a lock older than the boot and a table that hides processes both turn a
//!  proof into words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const boot = bootTime(); const hides = procHidesNoProcess(); const newest = newestTime(born, identity);
//! ```

use crate::fs_mount_options::read_mount_facts;
use crate::stale_ref_lock_identity::Identity;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// The kernel's name for the initial PID namespace,
/// where every process of the machine has the number it is seen by.
pub const INITIAL_PID_NAMESPACE: &str = "pid:[4026531836]";

/// The mount point of the process table the judge reads.
pub const PROC_MOUNT_POINT: &str = "/proc";

/// The start of the mount option that hides other accounts' processes from this account.
const HIDEPID_OPTION: &str = "hidepid=";

/// The values of `hidepid` that hide nothing.
const HIDEPID_OFF: [&str; 2] = ["0", "off"];

/// Whether the mount of `/proc` shows this account every process:
/// it has no `hidepid` option,
/// or one that hides nothing.
///  A table that hides processes lists fewer than exist,
/// so a number missing from it is no proof.
/// Where the mount cannot be
///  read the answer is no.
pub fn proc_hides_no_process() -> bool {
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
pub fn newest_time(born: Option<SystemTime>, identity: &Identity) -> SystemTime {
    let modified: SystemTime = nanoseconds_to_time(identity.modified_ns);
    return match born {
        Some(at) => at.max(modified),
        None => modified,
    };
}

/// A time given as nanoseconds since the epoch.
pub fn nanoseconds_to_time(nanoseconds: i128) -> SystemTime {
    let magnitude: u64 = u64::try_from(nanoseconds.unsigned_abs()).unwrap_or(u64::MAX);
    return if nanoseconds >= 0 { SystemTime::UNIX_EPOCH + Duration::from_nanos(magnitude) } else { SystemTime::UNIX_EPOCH - Duration::from_nanos(magnitude) };
}

/// When the machine booted:
/// the `btime` line of `/proc/stat`,
/// in whole seconds since the epoch.
pub fn boot_time() -> Option<SystemTime> {
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

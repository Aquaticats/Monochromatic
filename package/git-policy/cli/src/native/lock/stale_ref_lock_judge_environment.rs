//! What:
//!  What one judgement reads of the machine it runs on:
//! when the machine booted,
//! and the time a file carries.
//! Why:
//!  These readings decide what a judgement may conclude,
//! not what it is given,
//! so they sit beside the judge rather than in it:
//! a lock older than the boot turns a proof into words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const boot = bootTime(); const newest = newestTime(born, identity);
//! ```

use crate::stale_ref_lock_identity::Identity;
use std::path::Path;
use std::time::{
    Duration,
    SystemTime,
};

/// The kernel's name for the initial PID namespace,
/// where every process of the machine has the number it is seen by.
pub const INITIAL_PID_NAMESPACE: &str = "pid:[4026531836]";

/// The mount point of the process table the judge reads.
pub const PROC_MOUNT_POINT: &str = "/proc";

/// The later of the lock's birth (when known) and its last modification.
pub fn newest_time(
    born: Option<SystemTime>,
    identity: &Identity,
) -> SystemTime {
    let modified: SystemTime = nanoseconds_to_time(identity.modified_ns);
    return match born {
        Some(at) => at.max(modified),
        None => modified,
    };
}

/// A time given as nanoseconds since the epoch.
pub fn nanoseconds_to_time(nanoseconds: i128) -> SystemTime {
    let magnitude: u64 = u64::try_from(nanoseconds.unsigned_abs()).unwrap_or(u64::MAX);
    return if nanoseconds >= 0 {
        SystemTime::UNIX_EPOCH + Duration::from_nanos(magnitude)
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_nanos(magnitude)
    };
}

/// How many bytes of the system's own counters this reads.
/// Why:
///  `/proc/stat` holds one line per processor,
///  so its length is the number of processors the machine has:
///  this host's was 4,639 bytes on 2026-11-12,
///  and 4 MiB leaves room for thousands of them.
const SYSTEM_STAT_BYTES_AT_MOST: u64 = 4 << 20;

/// When the machine booted:
/// the `btime` line of `/proc/stat`,
/// in whole seconds since the epoch.
pub fn boot_time() -> Option<SystemTime> {
    let text: String =
        crate::plain_file::read_plain_text_at(Path::new("/proc/stat"), SYSTEM_STAT_BYTES_AT_MOST)
            .ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("btime ")
            && let Ok(seconds) = rest.trim().parse::<u64>()
        {
            return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds));
        }
    }
    return None;
}

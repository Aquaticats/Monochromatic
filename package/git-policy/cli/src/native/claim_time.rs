//! What:
//!  A reading of the clocks a claim's lease is judged by: the boot the host is in, the host's monotonic clock, and the wall clock.
//!  Two readings of one boot are compared by the monotonic clock; a reading of an earlier boot is past at once; the wall
//!  clock judges only readings that carry no boot ID.
//! Why:
//!  A lease must run out when its holder stopped, not when somebody sets the date. The wall clock steps (a time sync, a person,
//!  a virtual machine resuming), and a step forward made a live holder look an hour stale while a step back kept a dead
//!  holder's claim alive. `CLOCK_MONOTONIC` is one clock for every process of a host, never steps, and does not count a
//!  suspended host's sleep, so a holder and a recoverer on one boot agree on how long a claim went unrenewed. It restarts at
//!  every boot, so a reading carries the boot ID (`/proc/sys/kernel/random/boot_id`). A restart ends the holder, so a claim of an
//!  earlier boot has run out at once, and a wall clock set back at boot cannot strand it; where the platform has no boot ID,
//!  only the wall clock is left to compare and it is used as the fallback.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stamp = { bootId: readBootId(), monotonicMillis: hrtimeMillis(), wallSeconds: Date.now() / 1000 };
//! const pastBoot = (now, then) => now.bootId !== '' && then.bootId !== '' && now.bootId !== then.bootId;
//! const elapsed = (now, then) => now.bootId !== '' && now.bootId === then.bootId ? now.monotonicMillis - then.monotonicMillis : (now.wallSeconds - then.wallSeconds) * 1000;
//! ```

use super::service_replay::capture_stamp;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Separates the three parts of a reading when it is written into a claim's file name.
const FIELD_SEPARATOR: char = '~';

/// The length of a boot ID without its dashes.
const BOOT_ID_LENGTH: usize = 32;

/// What:
///  One reading of the three clocks.
/// Why:
///   A claim carries the reading taken when it was last renewed; whoever judges the claim takes a reading of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimTime = { bootId: string; monotonicMillis: number; wallSeconds: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimTime {
    /// The boot the monotonic reading belongs to; empty where the host has no boot ID or no monotonic clock.
    boot_id: String,
    /// Milliseconds of `CLOCK_MONOTONIC`.
    monotonic_millis: u64,
    /// Seconds since the Unix epoch of the wall clock.
    wall_seconds: u64,
}

/// The boot ID of this host's current boot, without dashes, or nothing when it cannot be read.
fn current_boot_id() -> &'static str {
    static BOOT_ID: OnceLock<String> = OnceLock::new();
    return BOOT_ID
        .get_or_init(|| {
            let Ok(text) = std::fs::read_to_string("/proc/sys/kernel/random/boot_id") else {
                return String::new();
            };
            let compact: String = text.trim().chars().filter(|character| { return *character != '-'; }).collect();
            return if compact.len() == BOOT_ID_LENGTH && compact.bytes().all(|byte| { return byte.is_ascii_hexdigit(); }) { compact } else { String::new() };
        })
        .as_str();
}

/// Methods over a reading.
impl ClaimTime {
    /// A reading of the clocks now.
    pub fn read() -> ClaimTime {
        let monotonic_millis: u64 = capture_stamp() / 1_000_000;
        // Without a monotonic clock the boot ID would pair with a meaningless number, so the reading has none.
        let boot_id: String = if monotonic_millis == 0 { String::new() } else { String::from(current_boot_id()) };
        let wall_seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| { return elapsed.as_secs(); });
        return ClaimTime { boot_id, monotonic_millis, wall_seconds };
    }

    /// A reading of the given parts, which is how a test says what the clocks showed.
    pub fn from_parts(boot_id: &str, monotonic: Duration, wall: Duration) -> ClaimTime {
        return ClaimTime {
            boot_id: boot_id.to_owned(),
            monotonic_millis: u64::try_from(monotonic.as_millis()).unwrap_or(u64::MAX),
            wall_seconds: wall.as_secs(),
        };
    }

    /// The reading `by` later on both clocks, which is time passing.
    pub fn advanced(&self, by: Duration) -> ClaimTime {
        return ClaimTime {
            boot_id: self.boot_id.clone(),
            monotonic_millis: self.monotonic_millis.saturating_add(u64::try_from(by.as_millis()).unwrap_or(u64::MAX)),
            wall_seconds: self.wall_seconds.saturating_add(by.as_secs()),
        };
    }

    /// The reading `by` earlier on both clocks, which is how a test makes a claim whose holder last renewed that long ago.
    pub fn rewound(&self, by: Duration) -> ClaimTime {
        return ClaimTime {
            boot_id: self.boot_id.clone(),
            monotonic_millis: self.monotonic_millis.saturating_sub(u64::try_from(by.as_millis()).unwrap_or(u64::MAX)),
            wall_seconds: self.wall_seconds.saturating_sub(by.as_secs()),
        };
    }

    /// The reading with the wall clock set `by` forward and the monotonic clock untouched, which is a clock step.
    pub fn wall_stepped_forward(&self, by: Duration) -> ClaimTime {
        return ClaimTime { wall_seconds: self.wall_seconds.saturating_add(by.as_secs()), ..self.clone() };
    }

    /// The reading with the wall clock set `by` back and the monotonic clock untouched, which is a clock step.
    pub fn wall_stepped_back(&self, by: Duration) -> ClaimTime {
        return ClaimTime { wall_seconds: self.wall_seconds.saturating_sub(by.as_secs()), ..self.clone() };
    }

    /// The reading in another boot, which is a restart of the host.
    pub fn in_boot(&self, boot_id: &str) -> ClaimTime {
        return ClaimTime { boot_id: boot_id.to_owned(), ..self.clone() };
    }

    /// The reading a wall-clock time stands for where there is nothing else, such as the modification time of a claim an
    /// earlier release stamped.
    pub fn from_wall(wall: SystemTime) -> ClaimTime {
        return ClaimTime {
            boot_id: String::new(),
            monotonic_millis: 0,
            wall_seconds: wall.duration_since(UNIX_EPOCH).map_or(0, |elapsed| { return elapsed.as_secs(); }),
        };
    }

    /// How long ago `earlier` was, by the clock the two readings share: the monotonic clock within one boot, the wall clock
    /// where either has no boot ID. A reading from the future of this one counts as no time at all. Readings of two different
    /// boots share no clock that says how long the earlier holder lived, so a judge asks `is_past_the_boot_of` first.
    pub fn elapsed_since(&self, earlier: &ClaimTime) -> Duration {
        if !self.boot_id.is_empty() && self.boot_id == earlier.boot_id {
            return Duration::from_millis(self.monotonic_millis.saturating_sub(earlier.monotonic_millis));
        }
        return Duration::from_secs(self.wall_seconds.saturating_sub(earlier.wall_seconds));
    }

    /// What:
    ///  Whether `earlier` was read in a boot this reading is not in, so whoever took it is gone.
    /// Why:
    ///   A restart ends every process of the host, so the holder of an earlier boot's claim is dead and waiting out its
    ///  lease on the wall clock only strands the claim when the clock was set back at boot. Without a boot ID on either
    ///  side there is no boot to compare, and the answer is no.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// now.bootId !== '' && then.bootId !== '' && now.bootId !== then.bootId
    /// ```
    pub fn is_past_the_boot_of(&self, earlier: &ClaimTime) -> bool {
        return !self.boot_id.is_empty() && !earlier.boot_id.is_empty() && self.boot_id != earlier.boot_id;
    }

    /// The reading as the three file-name parts `boot~monotonic~wall`.
    pub fn encode(&self) -> String {
        return format!("{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}", self.boot_id, self.monotonic_millis, self.wall_seconds);
    }

    /// The reading `encode` wrote, or nothing for any other text.
    pub fn decode(text: &str) -> Option<ClaimTime> {
        let mut parts = text.split(FIELD_SEPARATOR);
        let boot_id: &str = parts.next()?;
        let monotonic_millis: u64 = parts.next()?.parse().ok()?;
        let wall_seconds: u64 = parts.next()?.parse().ok()?;
        if parts.next().is_some() || !(boot_id.is_empty() || (boot_id.len() == BOOT_ID_LENGTH && boot_id.bytes().all(|byte| { return byte.is_ascii_hexdigit(); }))) {
            return None;
        }
        return Some(ClaimTime { boot_id: boot_id.to_owned(), monotonic_millis, wall_seconds });
    }
}

/// Clock controls stay out of the release executable.
#[cfg(test)]
#[path = "claim_time_tests.rs"]
mod tests;

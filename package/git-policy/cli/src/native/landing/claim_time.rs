//! What:
//!  A reading of the clocks a claim's lease is judged by: the machine and the boot the host is in, the host's monotonic clock,
//!  and the wall clock. Two readings of one boot are compared by the monotonic clock; a reading of an earlier boot of the same
//!  machine is past at once; the wall clock judges every other pair.
//! Why:
//!  A lease must run out when its holder stopped, not when somebody sets the date. The wall clock steps (a time sync, a person,
//!  a virtual machine resuming), and a step forward made a live holder look an hour stale while a step back kept a dead
//!  holder's claim alive. `CLOCK_MONOTONIC` is one clock for every process of a host, never steps, and does not count a
//!  suspended host's sleep, so a holder and a recoverer on one boot agree on how long a claim went unrenewed. It restarts at
//!  every boot, so a reading carries the boot ID (`/proc/sys/kernel/random/boot_id`). A restart ends the holder, so a claim of an
//!  earlier boot has run out at once, and a wall clock set back at boot cannot strand it. That holds only when both readings
//!  are of one machine: two hosts that share a work directory have unrelated boot IDs, and the other host's boot is not the
//!  end of its holder. So a reading also carries the machine ID (`/etc/machine-id`), and a different boot ID is evidence of a
//!  restart only beside an equal, known machine ID. Equal non-empty boot IDs still compare by the monotonic clock. Where either
//!  reading has no boot ID, only the wall clock is left to compare; where either has no machine ID, a different boot ID proves
//!  no restart, so the wall clock judges that pair as the fallback. Machines cloned with one machine ID cannot be told
//!  apart; their claims are judged as one machine's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stamp = { machineId: readMachineId(), bootId: readBootId(), monotonicMillis: hrtimeMillis(), wallSeconds: Date.now() / 1000 };
//! const pastBoot = (now, then) => now.machineId !== '' && now.machineId === then.machineId
//!   && now.bootId !== '' && then.bootId !== '' && now.bootId !== then.bootId;
//! const elapsed = (now, then) => now.bootId !== '' && now.bootId === then.bootId ? now.monotonicMillis - then.monotonicMillis : (now.wallSeconds - then.wallSeconds) * 1000;
//! ```

use crate::service_replay::capture_stamp;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Separates the parts of a reading when it is written into a claim's file name.
const FIELD_SEPARATOR: char = '~';

/// The length of a boot ID without its dashes, and of a machine ID.
const BOOT_ID_LENGTH: usize = 32;

/// Where a machine ID is kept, in the order they are tried (`/var/lib/dbus/machine-id` is the older place and often a link to the first).
const MACHINE_ID_PATHS: [&str; 2] = ["/etc/machine-id", "/var/lib/dbus/machine-id"];

/// The number of parts in a reading's text before the machine ID was added, and after.
const PARTS_WITHOUT_MACHINE: usize = 3;
const PARTS_WITH_MACHINE: usize = 4;

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
    /// The machine the reading was taken on; empty where the host has none or the reading predates machine IDs.
    machine_id: String,
    /// The boot the monotonic reading belongs to; empty where the host has no boot ID or no monotonic clock.
    boot_id: String,
    /// Milliseconds of `CLOCK_MONOTONIC`.
    monotonic_millis: u64,
    /// Seconds since the Unix epoch of the wall clock.
    wall_seconds: u64,
}

/// Whether `text` is 32 hexadecimal digits, which is what a boot ID without dashes and a machine ID are.
fn is_compact_hex_id(text: &str) -> bool {
    return text.len() == BOOT_ID_LENGTH && text.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
}

/// Whether `text` is a machine ID: 32 lowercase hexadecimal digits that are not all zero.
pub(crate) fn is_machine_id(text: &str) -> bool {
    return is_compact_hex_id(text);
}

/// The boot ID of this host's current boot, without dashes, or nothing when it cannot be read.
pub(crate) fn current_boot_id() -> &'static str {
    static BOOT_ID: OnceLock<String> = OnceLock::new();
    return BOOT_ID
        .get_or_init(|| {
            let Ok(text) = std::fs::read_to_string("/proc/sys/kernel/random/boot_id") else {
                return String::new();
            };
            let compact: String = text.trim().chars().filter(|character| { return *character != '-'; }).collect();
            return if is_compact_hex_id(compact.as_str()) { compact } else { String::new() };
        })
        .as_str();
}

/// What:
///  The first machine ID that `paths` hold, or nothing when none holds one.
/// Why:
///   A file that is missing, empty or not 32 hexadecimal digits (a fresh image holds the text `uninitialized`, a container
///  may have no file) is no evidence of a machine, and the next place may still hold one. Taking the text as an ID would
///  make every such host look like one machine.
fn machine_id_from(paths: &[&Path]) -> String {
    for path in paths {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if is_compact_hex_id(text.trim()) {
            return text.trim().to_owned();
        }
    }
    return String::new();
}

/// The machine ID of this host, read once, or nothing when it cannot be read.
pub(crate) fn current_machine_id() -> &'static str {
    static MACHINE_ID: OnceLock<String> = OnceLock::new();
    return MACHINE_ID
        .get_or_init(|| {
            let paths: Vec<&Path> = MACHINE_ID_PATHS.iter().map(|path| { return Path::new(*path); }).collect();
            return machine_id_from(paths.as_slice());
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
        return ClaimTime { machine_id: String::from(current_machine_id()), boot_id, monotonic_millis, wall_seconds };
    }

    /// A reading of the given parts on a machine with no ID, which is how a test says what the clocks showed.
    pub fn from_parts(boot_id: &str, monotonic: Duration, wall: Duration) -> ClaimTime {
        return ClaimTime {
            machine_id: String::new(),
            boot_id: boot_id.to_owned(),
            monotonic_millis: u64::try_from(monotonic.as_millis()).unwrap_or(u64::MAX),
            wall_seconds: wall.as_secs(),
        };
    }

    /// The reading `by` later on both clocks, which is time passing.
    pub fn advanced(&self, by: Duration) -> ClaimTime {
        return ClaimTime {
            machine_id: self.machine_id.clone(),
            boot_id: self.boot_id.clone(),
            monotonic_millis: self.monotonic_millis.saturating_add(u64::try_from(by.as_millis()).unwrap_or(u64::MAX)),
            wall_seconds: self.wall_seconds.saturating_add(by.as_secs()),
        };
    }

    /// The reading `by` earlier on both clocks, which is how a test makes a claim whose holder last renewed that long ago.
    pub fn rewound(&self, by: Duration) -> ClaimTime {
        return ClaimTime {
            machine_id: self.machine_id.clone(),
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

    /// The reading on the machine with the given ID, which is how a test puts two readings on one machine or on two.
    pub fn on_machine(&self, machine_id: &str) -> ClaimTime {
        return ClaimTime { machine_id: machine_id.to_owned(), ..self.clone() };
    }

    /// The reading a wall-clock time stands for where there is nothing else, such as the modification time of a claim an
    /// earlier release stamped.
    pub fn from_wall(wall: SystemTime) -> ClaimTime {
        return ClaimTime {
            machine_id: String::new(),
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
    ///  Whether `earlier` was read in a boot this reading is not in, on the same machine, so whoever took it is gone.
    /// Why:
    ///   A restart ends every process of the host, so the holder of an earlier boot's claim is dead and waiting out its
    ///  lease on the wall clock only strands the claim when the clock was set back at boot. A different boot ID of another
    ///  machine says nothing about the holder, which may be running there, so the answer is yes only when both machine IDs
    ///  are known and equal. Without a boot ID or a machine ID on either side there is nothing to compare, and the answer is
    ///  no: the lease decides.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// now.machineId !== '' && now.machineId === then.machineId
    ///   && now.bootId !== '' && then.bootId !== '' && now.bootId !== then.bootId
    /// ```
    pub fn is_past_the_boot_of(&self, earlier: &ClaimTime) -> bool {
        let same_known_machine: bool = !self.machine_id.is_empty() && self.machine_id == earlier.machine_id;
        return same_known_machine && !self.boot_id.is_empty() && !earlier.boot_id.is_empty() && self.boot_id != earlier.boot_id;
    }

    /// The reading as the file-name parts `boot~monotonic~wall~machine`; a reading with no machine ID leaves the last part
    /// out, which is the form every release before machine IDs wrote and reads.
    pub fn encode(&self) -> String {
        let without_machine: String = format!("{}{FIELD_SEPARATOR}{}{FIELD_SEPARATOR}{}", self.boot_id, self.monotonic_millis, self.wall_seconds);
        if self.machine_id.is_empty() {
            return without_machine;
        }
        return format!("{without_machine}{FIELD_SEPARATOR}{}", self.machine_id);
    }

    /// The reading `encode` wrote, or nothing for any other text. The form without a machine ID, which releases before
    /// machine IDs wrote into claims that may still be on disk, reads as a reading with none.
    pub fn decode(text: &str) -> Option<ClaimTime> {
        let parts: Vec<&str> = text.split(FIELD_SEPARATOR).collect();
        if parts.len() != PARTS_WITHOUT_MACHINE && parts.len() != PARTS_WITH_MACHINE {
            return None;
        }
        let boot_id: &str = parts[0];
        let monotonic_millis: u64 = parts[1].parse().ok()?;
        let wall_seconds: u64 = parts[2].parse().ok()?;
        let machine_id: &str = parts.get(PARTS_WITH_MACHINE - 1).copied().unwrap_or("");
        let boot_id_fits: bool = boot_id.is_empty() || is_compact_hex_id(boot_id);
        let machine_id_fits: bool = parts.len() == PARTS_WITHOUT_MACHINE || is_compact_hex_id(machine_id);
        if !boot_id_fits || !machine_id_fits {
            return None;
        }
        return Some(ClaimTime { machine_id: machine_id.to_owned(), boot_id: boot_id.to_owned(), monotonic_millis, wall_seconds });
    }
}

/// Clock controls stay out of the release executable.
#[cfg(test)]
#[path = "claim_time_tests.rs"]
mod tests;

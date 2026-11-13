//! What:
//!  The least traffic a push's connection to its remote may carry and still count as progress:
//!  the environment variable that sets it,
//!  its default,
//!  and the reading of a positive whole number of bytes.
//! Why:
//!  "Stop only on no progress" meets "nothing should ever wait forever" in a server that keeps a
//!  connection alive and trickles a byte now and then:
//! to the meter every byte is work,
//! so neither silence window ever runs out.
//! The owner's answer of 2026-11-12
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "A slow server and a typed push") is "Minimum rate":
//! progress means at least a stated
//!  number of bytes within a stated window,
//! and under it the push stops with a message.
//!  The window is the one that bounds silence in the state the run is in:
//! the connected window
//! (`CONNECTED_PUSH_QUIET_WINDOW`,
//! `service/wait_ceilings.rs`) while a connection is open,
//! and the stall window
//!  (`CLI_GIT_PUSH_DEADLINE_SECONDS`) while none is,
//! because a phase of a push that carries no bytes at all is already
//!  bounded by exactly that window,
//! and a floor judged over a shorter window would end an honest server's silent
//!  check sooner than silence alone does.
//!  The bytes are `push_progress.rs`'s:
//! the transport's own,
//! both directions,
//! which are the
//!  socket's counters over a URL and the run's own pipe over a path,
//! and never the account a program prints of its own waiting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const minimumRateBytes = (env) => positiveCount(env, 'CLI_GIT_PUSH_MINIMUM_RATE_BYTES') ?? 1024;
//! ```

/// The reading of a positive whole number shared with the deadline settings.
use crate::push_deadline_and_patience_settings::positive_count;
/// Owned OS text of the environment values.
use std::ffi::OsString;

/// Environment variable that sets,
/// in whole bytes,
/// the least traffic a push's connection may carry in one connected window and still count as progress.
pub const PUSH_MINIMUM_RATE_VARIABLE: &str = "CLI_GIT_PUSH_MINIMUM_RATE_BYTES";

/// The least traffic a push's connection may carry in one connected window and still count as progress,
/// when the variable is absent or unusable.
/// Citation:
/// measured on 2026-11-12 against loopback servers that Git's own `git http-backend` served,
/// and against a remote that is a path
/// (`doc/planning/cli-git-shipped-limits-audit.md` records the numbers beside the rule,
/// and `package/git-policy/cli/SPEC.md`,
/// "The least traffic that counts as progress",
/// carries the method and the figures).
///  The traffic a real server sends while it does nothing is Git's own keepalive,
/// five bytes every five seconds by default (`receive.keepAlive`,
/// `builtin/receive-pack.c`),
/// which is 150 bytes in the 150 second connected window and 30 bytes in the 30 second stall window,
/// and at least 1 second per
///  keepalive,
/// so 750 bytes is the most a keepalive alone can carry in the connected window.
///  A server that trickles an answer,
/// the audit's F14,
/// moved one byte every three seconds,
/// which is 50 bytes in the connected window.
///  An honest push of this repository's own sizes,
/// 0.35 MB for one commit and 5.5 MB for a hundred (measured with
/// `git pack-objects` on this checkout),
/// moved far more than the floor at every rate measured,
/// 54 KB/s and 2.5 KB/s shaped,
/// and the slowest line the floor still lets through is 1024 bytes per window,
/// about 7 bytes per second over a connection and 34 over a path,
/// on which that 5.5 MB push would take days and still land.
///  So the floor sits above what a keepalive can carry and orders of magnitude below an honest transfer,
/// and a person whose line really is slower lowers it
/// (`PUSH_MINIMUM_RATE_VARIABLE`).
pub const DEFAULT_PUSH_MINIMUM_RATE_BYTES: u64 = 1024;

/// What:
///  The least traffic a push's connection may carry in one connected window and still count as progress.
/// Why:
///   A person on a line slower than the floor lowers the number;
/// a value that is zero,
/// negative,
/// fractional,
/// empty or not a number is no setting at all,
///  never "no floor",
/// so a typo cannot remove the rule,
/// as with every other value of this kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveCount(env, 'CLI_GIT_PUSH_MINIMUM_RATE_BYTES') ?? 1024
/// ```
pub fn push_minimum_rate_bytes(environment: &[(OsString, OsString)]) -> u64 {
    let mut found: Option<&OsString> = None;
    for (name, value) in environment {
        if name.to_str() == Some(PUSH_MINIMUM_RATE_VARIABLE) {
            found = Some(value);
        }
    }
    let Some(value) = found else {
        return DEFAULT_PUSH_MINIMUM_RATE_BYTES;
    };
    return positive_count(PUSH_MINIMUM_RATE_VARIABLE, value.as_os_str()).unwrap_or(DEFAULT_PUSH_MINIMUM_RATE_BYTES);
}

/// Minimum-rate reading is controlled in the test module.
#[cfg(test)]
#[path = "push_minimum_rate_tests.rs"]
mod tests;

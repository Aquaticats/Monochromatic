//! What:
//!  The least traffic a push's connection to its remote may carry and still count as progress:
//!  the environment variable that sets it,
//!  its default,
//!  and the reading of a positive whole number of bytes.
//! Why:
//!  "Stop only on no progress" meets "nothing should ever wait forever" in a server that keeps a
//!  connection alive and trickles a byte now and then:
//! to the meter every byte is work, so neither silence window ever runs out.
//! The owner's answer of 2026-11-12
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "A slow server and a typed push") is "Minimum rate":
//! progress means at least a stated
//!  number of bytes within a stated window,
//! and under it the push stops with a message.
//!  The window is the one a connected run may stay silent for
//! (`CONNECTED_PUSH_QUIET_WINDOW`,
//! `service/wait_ceilings.rs`),
//! because a phase of a push that carries no bytes at all is
//!  already bounded by exactly that window,
//! and a floor judged over a shorter window would end an honest server's
//!  silent check sooner than silence alone does.
//!  Where the bytes counted as progress are `push_progress.rs`'s:
//! the transport's own
//!  counters,
//! both directions,
//! never the account a program prints of its own waiting.
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
/// measured on 2026-11-12 against loopback servers that Git's own `git http-backend` served
/// (`doc/planning/cli-git-shipped-limits-audit.md` records the numbers beside the rule,
/// and `package/git-policy/cli/SPEC.md`,
/// "The least traffic that counts as progress",
/// carries the method and the figures):
///
/// - the traffic a real server sends while it does nothing is Git's own keepalive,
/// five bytes every five seconds by default (`receive.keepAlive`,
/// `builtin/receive-pack.c`),
/// which is 150 bytes in the 150 second window,
/// and at least 1 second per keepalive,
/// so 750 bytes is the most a keepalive alone can carry in that window;
/// - a server that trickles an answer,
/// the audit's F14,
/// moved one byte every three seconds,
/// which is 50 bytes in the window;
/// - an honest push of this repository's own sizes,
/// 0.35 MB for one commit and 5.5 MB for a hundred (measured with
/// `git pack-objects` on this checkout),
/// moved far more than the floor at every rate measured,
/// 66 KB/s shaped down to 2 KB/s,
/// and the slowest line the floor still lets through is
///  1024 bytes per 150 seconds,
/// about 7 bytes per second,
/// on which that 5.5 MB push would take about nine days and still land.
///
/// So the floor sits above what a keepalive can carry and eight orders of magnitude below an
///  honest transfer,
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

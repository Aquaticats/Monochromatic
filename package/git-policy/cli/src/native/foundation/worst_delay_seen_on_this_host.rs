//! What:
//!  The longest delay of a small file operation measured on this host,
//! and the owner's multiple of it that the wrapper waits before it
//!  gives up on something that is only slow:
//! the held-name wait and the wait before a stop forces its end.
//! Why:
//!  A check and a rename took 6.03 s of wall time once,
//! in a run of 20,000 on 2026-11-07 under a load average of about 70 inside a
//!  scope capped at sixteen processors (`foundation/private_storage_hold_time_measurement_tests.rs`,
//! `LONGEST_HOLD_SEEN_MICROS`;
//! the
//!  plan,
//! `doc/planning/git-policy-cli-full-tokio.md`,
//! "The coordinator's three settlements after T5",
//! item c).
//! Whether the disk or the
//!  scheduler held the thread was not measured:
//! a rerun in a scope capped at twenty-four processors at a load average of about twenty
//!  saw at most 94 us.
//! Asked how much longer than that worst case the wrapper should wait before giving up (once for the wait before a
//!  force-kill of a Git asked to stop,
//! once for the held-name wait),
//! the owner picked,
//! 2026-11-07,
//! verbatim:
//! "3x worst case".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const LONGEST_DELAY_SEEN_MS = 6031.986; export const OWNER_MARGIN = 3; export const WORST_CASE_WITH_MARGIN_MS = LONGEST_DELAY_SEEN_MS * OWNER_MARGIN;
//! ```

use std::time::Duration;

/// The longest delay of a check and a rename seen in any run of the measurements,
/// as wall time.
pub const LONGEST_DELAY_SEEN: Duration = Duration::from_micros(6_031_986);

/// How many times the longest delay seen the wrapper waits before it gives up:
/// the owner's "3x worst case".
pub const OWNER_MARGIN: u32 = 3;

/// How long the wrapper waits for something that is only slow before it gives up:
/// the longest delay seen times the owner's margin.
pub const WORST_CASE_WITH_MARGIN: Duration = LONGEST_DELAY_SEEN.saturating_mul(OWNER_MARGIN);

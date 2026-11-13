//! What:
//!  The three values a kept reference is judged by:
//!  how old it may become,
//!  how many bytes the objects only kept references reach may weigh,
//!  and how young a reference stays out of the size rule's reach,
//!  with the measurement each rests on and the variables that change them.
//! Why:
//!  The owner decided on 2026-11-12
//!  ("Keep the reference, auto prune by size and age",
//!  `doc/handover/cli-git-rust-implementation.md`,
//!  the answer of 2026-11-12T11:58:45-05:00)
//!  that the wrapper prunes them by itself and named no values.
//!  The values below are proposed from one reading of this repository
//!  (the commands and numbers are in `kept_references_reading.rs`
//!  and in `doc/troubleshooting/cli-git-kept-references-are-pruned-by-size-and-age.md`),
//!  and are put to the owner before a release carries them.
//!  They sit here, in one place, so the owner can read and change them together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limits = { maxAge: 30 * DAY, maxBytes: 64 * MiB, floor: HOUR };
//! ```

use crate::push_deadline_and_patience_settings::positive_seconds;
use std::ffi::OsString;
use std::time::Duration;

/// Tag used for diagnostics from this module.
const TAG: &str = "keptReferenceLimits";

/// How old a kept reference may become before a pass prunes it.
///
/// Measured on this host on 2026-11-12, read-only, with
/// `git for-each-ref refs/cli-git/ --format='%(refname) %(objectname) %(creatordate:iso-strict)'`
/// and `git rev-list --count <commit> --not --branches --remotes --tags`:
/// this repository held five kept references,
/// written between 2026-11-07T13:01:46-05:00 and 2026-11-07T13:48:16-05:00,
/// so the oldest was 6.5 days old and none of the five had reached a branch.
/// Thirty days is about five times the oldest age this repository ever reached,
/// so a person has a month to recover a commit the wrapper could not land.
/// At this repository's rate (five references in 5.7 days, one every 1.2 days)
/// it bounds the folder at about 25 references;
/// a repository with ten times the activity bounds it at about 250,
/// and sees the same ages, because the age rule does not depend on the rate.
pub const KEPT_REFERENCE_MAX_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// How many bytes the objects only kept references reach may weigh before the oldest references go.
///
/// Measured on this host on 2026-11-12, read-only, with the commands in
/// `kept_references_reading.rs`:
/// the objects reachable only through this repository's five kept references were
/// 27 objects (5 commits, 13 trees, 9 blobs),
/// 7,573 bytes on disk by `%(objectsize:disk)` and 258,055 bytes before compression,
/// so one kept reference cost 1,515 bytes on disk there.
/// Sixty-four mebibytes is about 43,000 of them at that weight,
/// which the age rule reaches long before the size rule does,
/// so the size rule's job is a kept commit that carries large content:
/// the largest object of those five was a 2,023-byte blob,
/// while a kept commit of a worktree holding an artifact can carry megabytes,
/// and ten such commits of 6 MiB each bring the limit to its bound.
/// A repository with ten times the activity sees the age rule bind at the same weight;
/// one with ten times the content per kept commit sees this rule bind on every pass.
pub const KEPT_REFERENCE_MAX_BYTES: u64 = 64 * 1024 * 1024;

/// How young a kept reference is never pruned by the size rule,
/// whatever the objects weigh.
/// Why:
///   A kept reference is written at the moment a worktree goes,
///  so a burst of removals writes a burst of references,
///  and the size rule removes the oldest first:
///  a commit kept at the start of a burst must not be the price of the burst that follows it.
///  Measured on this host on 2026-11-12:
///  the cleanup of 68 unused worktrees (`clean-build-trees-of-unused-pi-worktrees-1138`)
///  ran against this repository from 11:38 and ended before 11:49:32
///  (`doc/handover/cli-git-rust-implementation.md`,
///  "The cleaning of unused worktrees, ended"),
///  so a burst of removals takes tens of minutes,
///  and an hour covers several of them.
///  The age rule still takes such a reference thirty days later,
///  so nothing is kept for ever by this floor.
pub const KEPT_REFERENCE_FLOOR: Duration = Duration::from_secs(60 * 60);

/// The variable that sets how old a kept reference may become, in whole seconds.
pub const MAX_AGE_VARIABLE: &str = "CLI_GIT_KEPT_REFERENCE_MAX_AGE_SECONDS";

/// The variable that sets the bytes the kept objects may weigh.
pub const MAX_BYTES_VARIABLE: &str = "CLI_GIT_KEPT_REFERENCE_MAX_BYTES";

/// The variable that sets how young a kept reference stays out of the size rule's reach, in whole seconds.
pub const FLOOR_VARIABLE: &str = "CLI_GIT_KEPT_REFERENCE_FLOOR_SECONDS";

/// What:
///  The values one pass judges a repository's kept references by.
/// Why:
///   The pass and the listing both read them once,
///  so what a person is told is what the next pass does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptReferenceLimits = { maxAge: Temporal.Duration; maxBytes: number; floor: Temporal.Duration };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeptReferenceLimits {
    /// How old a kept reference may become.
    pub max_age: Duration,
    /// How many bytes the objects only kept references reach may weigh.
    pub max_bytes: u64,
    /// How young a kept reference stays out of the size rule's reach.
    pub floor: Duration,
}

/// What:
///  A positive whole number of bytes from one environment value.
/// Why:
///   A zero,
///  negative,
///  fractional,
///  empty or non-text value is no setting at all,
///  so a typo cannot remove the limit and let the folder grow.
///  `duration_seconds` reads the two ages through `positive_seconds`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const positiveBytes = (name, value) => /^[0-9]+$/.test(value) && BigInt(value) > 0n ? BigInt(value) : undefined;
/// ```
pub fn positive_bytes(name: &str, value: &std::ffi::OsStr) -> Option<u64> {
    let parsed: Option<u64> = value.to_str()?.trim().parse::<u64>().ok();
    return match parsed {
        Some(count) if count > 0 => Some(count),
        Some(_) | None => {
            crate::diagnostic_log::debug(TAG, format!("{name} is not a positive whole number of bytes; ignoring it").as_str());
            None
        }
    };
}

/// What:
///  The last value the environment gives `name`,
///  as `std::env::var_os` reads it.
/// Why:
///   A variable set twice is one value for the process,
///  and the last one wins as it does for every other variable the wrapper reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const variable = (name) => process.env[name];
/// ```
fn variable(name: &str) -> Option<OsString> {
    return std::env::var_os(name);
}

/// What:
///  The limits a pass uses:
///  the three defaults above,
///  each replaced by its variable when that names a positive whole number.
/// Why:
///   The variables are how the owner, a person, and a control change the values without a build,
///  and how a control reaches the size rule with a fixture's few kilobytes.
///  A value that does not parse keeps the default,
///  so a typo never removes a bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const limitsFromTheEnvironment = () => ({ maxAge: seconds(MAX_AGE) ?? DEFAULT, maxBytes: bytes(MAX_BYTES) ?? DEFAULT, floor: seconds(FLOOR) ?? DEFAULT });
/// ```
pub fn limits_from_the_environment() -> KeptReferenceLimits {
    let max_age: Duration = match variable(MAX_AGE_VARIABLE) {
        Some(value) => positive_seconds(MAX_AGE_VARIABLE, value.as_os_str()).unwrap_or(KEPT_REFERENCE_MAX_AGE),
        None => KEPT_REFERENCE_MAX_AGE,
    };
    let max_bytes: u64 = match variable(MAX_BYTES_VARIABLE) {
        Some(value) => positive_bytes(MAX_BYTES_VARIABLE, value.as_os_str()).unwrap_or(KEPT_REFERENCE_MAX_BYTES),
        None => KEPT_REFERENCE_MAX_BYTES,
    };
    let floor: Duration = match variable(FLOOR_VARIABLE) {
        Some(value) => positive_seconds(FLOOR_VARIABLE, value.as_os_str()).unwrap_or(KEPT_REFERENCE_FLOOR),
        None => KEPT_REFERENCE_FLOOR,
    };
    return KeptReferenceLimits { max_age, max_bytes, floor };
}

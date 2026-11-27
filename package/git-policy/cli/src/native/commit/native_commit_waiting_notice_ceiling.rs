//! What:
//!  The ceiling of the commit Git builds in the scratch repository,
//!  as the setting a person gives it.
//! Why:
//!  A person who knows their own disk may name how long that commit is waited for,
//!  and every other way of setting one would make a second place to look;
//!  so one variable is read here,
//!  and the number it holds is what the notice and the wait that owns the step both count from.
//!  `scratch_step_stall_decided_once.rs` is what a ceiling above the stall bound is read through.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ceiling = () => positiveSeconds(process.env.CLI_GIT_SCRATCH_COMMIT_CEILING_SECONDS) ?? SCRATCH_COMMIT_STEP_CEILING;
//! ```

use std::time::Duration;

/// The environment variable that sets the ceiling of the step in whole seconds.
pub const SCRATCH_CEILING_VARIABLE: &str = "CLI_GIT_SCRATCH_COMMIT_CEILING_SECONDS";

/// The whole seconds a setting of `SCRATCH_CEILING_VARIABLE` names.
fn seconds_in(text: String) -> Option<u64> {
    return text.trim().parse::<u64>().ok();
}

/// The ceiling of the step:
/// `SCRATCH_CEILING_VARIABLE` when it names a positive number of seconds,
/// else `SCRATCH_COMMIT_STEP_CEILING`.
pub fn scratch_ceiling() -> Duration {
    return match std::env::var(SCRATCH_CEILING_VARIABLE)
        .ok()
        .and_then(seconds_in)
    {
        Some(seconds) if seconds > 0 => Duration::from_secs(seconds),
        _ => crate::wait_ceilings::SCRATCH_COMMIT_STEP_CEILING,
    };
}

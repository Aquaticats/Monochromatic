//! What:
//! The one call that applies the run-record retention to a directory with the real clock.
//! Why:
//! It keeps `lean_run_steps.rs` free of the policy struct's field names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function sweepRecordsDirectory(directory: string, dryRun: boolean, ...): RecordsOutcome
//! ```

/// Paths.
use std::path::Path;

use crate::lean_guard::GuardEnvironment;
use crate::lean_identity::{Deadline, now_seconds};
use crate::lean_run_records::{RecordsOutcome, RetentionPolicy, sweep_run_records};

/// What:
/// `pub fn sweep_records_directory(directory: &Path, dry_run: bool, deadline: &Deadline, environment: &GuardEnvironment, max_age_seconds: i64, max_bytes: u64, protect_seconds: i64) -> RecordsOutcome`
/// sweeps one verification directory and prints its removal lines.
/// Why:
/// Each removal's proof reaches the log as it happens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function sweepRecordsDirectory(directory: string, dryRun: boolean, deadline: Deadline, env: GuardEnvironment, maxAge: number, maxBytes: number, protect: number): RecordsOutcome
/// ```
pub fn sweep_records_directory(
    directory: &Path,
    dry_run: bool,
    deadline: &Deadline,
    environment: &GuardEnvironment,
    max_age_seconds: i64,
    max_bytes: u64,
    protect_seconds: i64,
) -> RecordsOutcome {
    let mut log: Vec<String> = Vec::new();
    let policy: RetentionPolicy = RetentionPolicy { max_age_seconds, max_bytes, protect_seconds };
    let outcome: RecordsOutcome = sweep_run_records(directory, &policy, environment, now_seconds(), dry_run, deadline, &mut log);
    for line in &log {
        eprintln!("cargo-shadow lean: {line}");
    }
    return outcome;
}

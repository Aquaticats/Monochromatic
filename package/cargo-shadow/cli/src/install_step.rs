//! What:
//! The shadow's install step (stub for the red test run).
//! Why:
//! (stub)

use std::path::Path;

use crate::cargo_run::CargoRunner;
use crate::lockfile::LockedVersion;
use crate::registry::Registry;

/// What the install step did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepOutcome {
    /// Nothing to do here, with the reason.
    Skipped(String),
    /// The recorded fingerprints still match: no cargo was started.
    FastPath,
    /// The authoritative check found the lockfile satisfied.
    Passed,
    /// The lockfile was stale and was updated.
    Updated {
        /// The repo-relative crate root.
        crate_dir: String,
        /// The registry versions newly locked.
        added: Vec<LockedVersion>,
    },
    /// The step refused or failed, with the message to show.
    Failed(String),
}

/// What the step needs.
pub struct StepRequest<'a> {
    /// Where cargo will work.
    pub start_dir: &'a Path,
    /// Fail on a stale lockfile instead of updating it.
    pub frozen: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
}

/// Stub.
pub fn run_install_step(_request: &StepRequest) -> StepOutcome {
    return StepOutcome::Skipped("stub".to_string());
}

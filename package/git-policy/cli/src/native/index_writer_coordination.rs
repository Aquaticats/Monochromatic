//! What:
//!  Coordination of a forwarded index writer with concurrent landings:
//!  the landing lock is taken
//!       after recovery,
//!  a foreign `index.lock` is waited for under the classification rules,
//!  and the
//!       lock is handed to real Git through the landing lease until Git returns.
//! Why:
//!  An index writer must not interleave with a landing commit,
//!  and a nested wrapper invoked by the
//!      coordinated Git (for example `git rebase --exec 'git add'`) must not wait on the lock its
//!      ancestor holds (`src/index-lock/index-writer-coordination.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using coordination = await coordinateIndexWriter({ command, gitPath, timeoutMs });
//! ```

/// Foreign-lock evidence readers used to explain the failure.
use super::index_lock_wait::{
    WaitFailure, live_holder_timeout_message, unproven_owner_message, wait_for_index_lock,
};
/// The host side effects of the wait.
use super::index_lock_wait_host::HostWaitEffects;
/// Index-writer classification of a resolved command.
use super::index_writer_commands::is_index_writer;
use super::diagnostic_log::debug;
/// The landing-location query.
use super::landing_location::{LandingLocation, resolve_landing_location};
/// The engine failure event,
///  rendered for standard error.
use super::diagnostics::{EngineFailureCode, render_engine_failure};
/// Startup recovery run before the landing lock is taken.
use super::transaction_gate::{Gate, recover_for_command};
/// Running real Git for the location query and recovery.
use super::transaction_git::GitContext;
/// The owner lock and its silent wait.
use super::live_lock::acquire_live_owner_lock;
use super::owner_lock::OwnerLock;
/// The landing lease and its variable.
use super::transaction_lease::{LANDING_LEASE_VARIABLE, format_landing_lease, has_valid_landing_lease};
/// Registry names and creation.
use super::transaction_registry::{LANDING_LOCK_NAME, TRANSACTION_ROOT_NAME, ensure_transaction_root};
use super::commit_recovery::recovery_directories;
/// The resolved forwarded command.
use super::forwarded_command::ResolvedCommand;
/// The repository shape the invocation selects.
use super::worktree_identity::WorktreeIdentity;
/// The once-per-process Git version check.
use super::git_version::{GitRelease, check_git_version};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Instant` starts the wait's clock.
use std::time::Instant;
use std::path::Path;

/// What:
///  The landing lock held by an index writer,
///  and the lease real Git inherits.
/// Why:
///   The lock is released after Git returns;
///  the lease tells nested wrappers the ancestor holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexWriterCoordination = AsyncDisposable & { environment: Record<string, string> };
/// ```
pub struct Coordination {
    /// The landing lock,
    ///  released when the coordination ends.
    lock: OwnerLock,
    /// The lease variable and its value.
    pub lease: (OsString, OsString),
}

/// What:
///  Why a coordinated command must not run.
/// Why:
///   Each case has its own stderr event and exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CoordinationFailure = IndexLockUnprovenOwnerError | Error;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinationFailure {
    /// A dead or unproven foreign owner outlasted the budget.
    UnprovenOwner(String),
    /// Something the coordination had to read could not be read or recovered.
    Stop(String),
}

impl CoordinationFailure {
    /// What:
    ///  The engine failure event for this failure,
    ///  as standard error.
    /// Why:
    ///   The incumbent reports both as engine failures with exit status 2.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// renderPolicyEvents([createEngineFailureEvent({ code, message })])
    /// ```
    pub fn stderr(&self) -> String {
        match self {
            CoordinationFailure::UnprovenOwner(message) => {
                return render_engine_failure(0, EngineFailureCode::IndexLockUnprovenOwner, message.as_str());
            }
            CoordinationFailure::Stop(message) => {
                return render_engine_failure(0, EngineFailureCode::ContentUnavailable, message.as_str());
            }
        }
    }
}

/// What:
///  The inputs of one coordination.
/// Why:
///   Named fields keep the call readable and satisfy the parameter-object rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// coordinateIndexWriter({ command, gitPath, timeoutMs, environment })
/// ```
pub struct CoordinationInput<'a> {
    /// Git context of the invocation.
    pub context: &'a GitContext,
    /// The repository shape the invocation selects.
    pub identity: &'a WorktreeIdentity,
    /// The resolved forwarded command.
    pub command: &'a ResolvedCommand,
    /// The environment the invocation inherited,
    ///  carrying any lease.
    pub environment: &'a [(OsString, OsString)],
    /// `indexLock.unprovenOwnerTimeoutMs`.
    pub timeout_ms: u64,
}

/// What:
///  Whether the command's index is the real index,
///  and where its landing lock lives.
/// Why:
///   `coordinateIndexWriter` coordinates only with the real index,
///  and only outside an ancestor's lease.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (location.indexPath !== location.realIndexPath) return NO_COORDINATION;
/// ```
fn coordinated_location(
    context: &GitContext,
    command: &ResolvedCommand,
    environment: &[(OsString, OsString)],
) -> Result<Option<LandingLocation>, CoordinationFailure> {
    let global: &[OsString] = &command.arguments[..command.subcommand_index];
    let location: LandingLocation = match resolve_landing_location(context, global) {
        Ok(Some(found)) => found,
        Ok(None) => return Ok(None),
        Err(reason) => return Err(CoordinationFailure::Stop(reason)),
    };
    if location.index_path != location.real_index_path {
        return Ok(None);
    }
    let lock_directory: std::path::PathBuf = location.registry_root.join(LANDING_LOCK_NAME);
    if has_valid_landing_lease(environment, lock_directory.as_path()) {
        return Ok(None);
    }
    return Ok(Some(location));
}

/// What:
///  Take the landing lock,
///  waiting while a live landing holds it.
/// Why:
///   Recovery runs before the acquisition because the native recovery takes the landing lock itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireLocationLandingLock({ gitPath, location });
/// ```
fn acquire_landing(context: &GitContext, identity: &WorktreeIdentity, location: &LandingLocation) -> Result<OwnerLock, CoordinationFailure> {
    match recover_for_command(context, identity) {
        Gate::Clear | Gate::Live(_) => {}
        Gate::Stop(stderr) => return Err(CoordinationFailure::Stop(stderr)),
    }
    if let Err(error) = ensure_transaction_root(location.registry_root.as_path()) {
        return Err(CoordinationFailure::Stop(error.0));
    }
    let lock_directory: std::path::PathBuf = location.registry_root.join(LANDING_LOCK_NAME);
    match acquire_live_owner_lock(lock_directory.as_path(), "the landing lock") {
        Ok(lock) => return Ok(lock),
        Err(error) => return Err(CoordinationFailure::Stop(error.to_string())),
    }
}

/// What:
///  Wait for any foreign `index.lock` to clear,
///  under the classification rules.
/// Why:
///   The lock is held meanwhile,
///  so no landing starts beside the index writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexLock({ realIndexPath, timeoutMs, consequence, attempt });
/// ```
pub fn wait_for_lock(
    real_index: &Path,
    timeout_ms: u64,
    consequence: &str,
) -> Result<(), CoordinationFailure> {
    // `mut` lets the wait advance its clock and record its reports.
    let mut effects: HostWaitEffects = HostWaitEffects {
        real_index: real_index.to_path_buf(),
        started: Instant::now(),
    };
    match wait_for_index_lock(&mut effects, timeout_ms, consequence) {
        Ok(()) => return Ok(()),
        Err(WaitFailure::UnprovenOwner {
            evidence,
            verdict,
            timeout_ms: budget,
            consequence: reason,
        }) => {
            return Err(CoordinationFailure::UnprovenOwner(unproven_owner_message(
                &evidence,
                &verdict,
                budget,
                reason.as_str(),
            )));
        }
        Err(WaitFailure::LiveHolderTimedOut {
            lock_path,
            pid,
            command,
            waited_ms,
            consequence: reason,
            ..
        }) => {
            return Err(CoordinationFailure::Stop(live_holder_timeout_message(
                lock_path.as_path(),
                pid,
                command.as_deref(),
                waited_ms,
                reason.as_str(),
            )));
        }
        Err(WaitFailure::Read(reason)) => return Err(CoordinationFailure::Stop(reason)),
    }
}

/// What:
///  Coordinate one forwarded command:
///  nothing for a non-writer,
///  otherwise hold the landing lock
///       and return the lease real Git inherits.
/// Why:
///   `coordinateIndexWriter`;
///  the lock is released by the caller after Git returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function coordinateIndexWriter({ command, gitPath, timeoutMs, environment }): Promise<IndexWriterCoordination>;
/// ```
pub fn coordinate_index_writer(input: &CoordinationInput<'_>) -> Result<Option<Coordination>, CoordinationFailure> {
    return coordinate_index_writer_checked(input, check_git_version);
}

/// What:
///  The coordination with the Git version answer supplied,
///  so the controls can refuse an old Git.
/// Why:
///   The process-wide check answers once per process and cannot be made to refuse inside a test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// coordinateIndexWriter({ ..., requireGit: checkGitVersion })
/// ```
pub fn coordinate_index_writer_checked(
    input: &CoordinationInput<'_>,
    version: fn(&GitContext) -> Result<GitRelease, String>,
) -> Result<Option<Coordination>, CoordinationFailure> {
    if !is_index_writer(input.command) {
        debug("coordinateIndexWriter", "not an index writer; no coordination");
        return Ok(None);
    }
    let location: LandingLocation = match coordinated_location(input.context, input.command, input.environment)? {
        Some(found) => found,
        None => {
            debug("coordinateIndexWriter", "no real-index landing location, or an inherited lease; no coordination");
            return Ok(None);
        }
    };
    debug("coordinateIndexWriter", format!("coordinating git {} with {}", command_name(input.command), location.registry_root.display()).as_str());
    // The version check runs once per process, before the first work that depends on Git's behavior.
    if let Err(reason) = version(input.context) {
        return Err(CoordinationFailure::Stop(reason));
    }
    let lock: OwnerLock = acquire_landing(input.context, input.identity, &location)?;
    let consequence: String = format!("did not run git {}", command_name(input.command));
    match wait_for_lock(location.real_index_path.as_path(), input.timeout_ms, consequence.as_str()) {
        Ok(()) => {}
        Err(failure) => {
            release_quietly(lock);
            return Err(failure);
        }
    }
    let lease_value: String = format_landing_lease(
        lock.directory().to_string_lossy().as_ref(),
        lock.token(),
    );
    return Ok(Some(Coordination {
        lock,
        lease: (OsString::from(LANDING_LEASE_VARIABLE), OsString::from(lease_value)),
    }));
}

/// What:
///  The subcommand's name as text,
///  for the diagnostics.
/// Why:
///   A message names the command the wrapper did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// String(command.subcommand)
/// ```
fn command_name(command: &ResolvedCommand) -> String {
    return String::from_utf8_lossy(command.word()).into_owned();
}

/// What:
///  Take the landing lock of a worktree's repository for a direct fix,
///  after recovery.
/// Why:
///   A direct fix changes the repository,
///  so no landing may run beside it;
///  outside a repository
///       there is no landing to exclude.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireSelectedLandingLock({ gitPath, globalArgs });
/// ```
pub fn hold_landing_lock(
    context: &GitContext,
    identity: &WorktreeIdentity,
) -> Result<Option<OwnerLock>, CoordinationFailure> {
    let Some((git_dir, _)) = recovery_directories(identity) else {
        return Ok(None);
    };
    let location: LandingLocation = LandingLocation {
        git_dir: git_dir.to_path_buf(),
        index_path: git_dir.join("index"),
        real_index_path: git_dir.join("index"),
        registry_root: git_dir.join(TRANSACTION_ROOT_NAME),
    };
    let lock: OwnerLock = acquire_landing(context, identity, &location)?;
    return Ok(Some(lock));
}

/// What:
///  Release a lock whose failure cannot be reported further.
/// Why:
///   A release error after a failed wait is dropped;
///  the wait's failure is the one that matters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lock[Symbol.asyncDispose]();
/// ```
fn release_quietly(lock: OwnerLock) {
    let _ = lock.release();
}

impl Coordination {
    /// What:
    ///  Release the landing lock after real Git returned.
    /// Why:
    ///   Releasing reports a failure the caller may still print.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await coordination[Symbol.asyncDispose]();
    /// ```
    pub fn release(self) -> Result<(), String> {
        match self.lock.release() {
            Ok(()) => return Ok(()),
            Err(error) => return Err(error.to_string()),
        }
    }
}

/// Coordination controls stay out of the release executable.
#[cfg(test)]
#[path = "index_writer_coordination_tests.rs"]
mod tests;

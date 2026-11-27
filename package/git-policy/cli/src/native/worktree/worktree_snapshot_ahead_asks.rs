//! What:
//!  The seams through which an add reaches the commit service that makes its snapshots ahead,
//!  and the records that ask the service for one.
//! Why:
//!  The commit service makes a copy on its own time,
//!  from a work item the add writes,
//!  so it is the service's thread that waits for the btrfs transaction commit and not the person's.
//!  A control must prove the ask with no service and no session bus,
//!  so the enqueue-and-wake is injected,
//!  and the ceiling is a value a control can shorten without changing what an add does.
//!  A record is written before the service is told,
//!  so a control asserts the record without a service,
//!  and a wake that cannot be made leaves the records for the next command of the repository.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_snapshot_ahead_names.rs` holds the names a copy carries and the pool it stands in,
//!  and `worktree_snapshot_ahead.rs` holds the answer one add's look came to and the hand-over of
//!  the copies it proved stale.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const request = askWith(gitDir, root, pool); // the record only; the caller wakes the service
//! await wakeTheService({ context, gitDir });
//! ```

use crate::diagnostic_log::debug;
use crate::hexadecimal_of_bytes::hexadecimal_of_bytes;
use crate::iso_time::now_iso;
use crate::random_id::random_uuid;
use crate::service_client::{
    Activation,
    ClientConfig,
    START_WAIT,
    manager_watch_for,
    wake,
};
use crate::service_messages::render;
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::provision;
use crate::service_route::{
    Route,
    route_from_environment,
};
use crate::service_work::work_directory;
use crate::subvolume_sha::sha256;
use crate::transaction_git::GitContext;
use crate::wait_ceilings::{
    ANSWER_CEILING,
    SHORT_EXTRA_WAIT,
    START_CEILING,
    WITHDRAW_CEILING,
    waiting_notice_after,
};
use crate::work_item::{
    WorkItem,
    WorkKind,
    enqueue,
};
use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

/// Tag used for diagnostics from this module.
pub(crate) const TAG: &str = "worktreeSnapshotAhead";

/// The reference an item of this kind names,
/// which is no branch:
/// the work queue keeps one reference-shaped field on every record,
/// and no push is ever made for this kind.
pub const AHEAD_REFERENCE: &str = "refs/worktree/cli-git-service/snapshot-ahead";

/// What:
///  How the add reaches the service that makes the snapshots ahead,
/// and how long it waits for one.
/// Why:
///   A control must prove the claim with no service and no session bus,
/// so the enqueue-and-wake is injected,
/// and the
///  ceiling is a value a control can shorten without changing what an add does.
/// The ask answers the id of the record it wrote,
/// when
///  it wrote one,
/// so the wait can see the service give that request up for good and stop waiting for a
///  copy that can never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AheadService = { handOver: (stale: string[]) => Result<void>; ask: () => Result<string | undefined>; ceiling: Duration };
/// ```
pub struct AheadService<'a> {
    /// Ask the repository's commit service for one snapshot ahead,
    /// answering the id of the record it wrote;
    /// the error is why it could not be told.
    pub ask: &'a dyn Fn() -> Result<Option<String>, String>,
    /// Hand the service every name this add proved stale or keeps no longer,
    /// and wake it;
    /// the error is why it
    ///  could not be told.
    pub hand_over: &'a dyn Fn(&[OsString]) -> Result<(), String>,
    /// The longest the add waits for one to arrive.
    pub ceiling: Duration,
}

/// What:
///  The reason a seam that reaches no service answers with.
/// Why:
///   A caller with no service to ask is told so in words that name what it would have asked,
/// and the add then takes
///  its own snapshot instead of waiting for one that cannot come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotAsk = () => err('this add asks no commit service for a snapshot ahead');
/// ```
fn cannot_ask() -> Result<Option<String>, String> {
    return Err(String::from(
        "this add asks no commit service for a snapshot ahead",
    ));
}

/// What:
///  The reason a seam that reaches no service answers with,
/// for the names a control hands it.
/// Why:
///   A control still records what an add wanted deleted,
/// and the words say why nothing acted on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotHandOver = (stale) => err(`this add hands ${stale.length} name(s) to no commit service`);
/// ```
fn cannot_hand_over(stale: &[OsString]) -> Result<(), String> {
    return Err(format!(
        "this add hands {} name(s) to no commit service",
        stale.len()
    ));
}

impl AheadService<'_> {
    /// What:
    ///  A seam that reaches nothing and waits for nothing:
    /// the add takes its own snapshot.
    /// Why:
    ///   A control must take a snapshot with no service and no session bus,
    /// and that is also what an add does where no
    ///  service can be started (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
    /// "Step 3").
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// AheadService.nothing() // { handOver: cannotAsk, ceiling: 0 }
    /// ```
    pub fn nothing() -> AheadService<'static> {
        return AheadService {
            ask: &cannot_ask,
            hand_over: &cannot_hand_over,
            ceiling: Duration::ZERO,
        };
    }
}

/// What:
///  The item that asks the commit service for one snapshot ahead of `source_root`,
/// made in `pool`.
/// Why:
///   The record outlives the command,
/// so an add that finds nothing,
/// and a service that was not running,
///  still leave the request for the repository's next round,
/// and the add after this one finds the pool filled.
/// The identity
///  is random,
/// so no add's request hides behind another's in the queue's idempotency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = (commonDir, root, pool) => ({ kind: 'snapshot-ahead', repository: root, gitDir: commonDir, ref: AHEAD_REFERENCE, oid: sha256(random()), parent: pool, created: now, tries: 0 });
/// ```
pub fn request_item(
    git_dir: &Path,
    source_root: &Path,
    pool: &Path,
) -> Result<WorkItem, String> {
    let unique: String = match random_uuid() {
        Ok(made) => made.replace('-', ""),
        Err(error) => {
            return Err(format!(
                "no identity can be made for a request of a snapshot ahead ({})",
                error.0
            ));
        }
    };
    let digest: String = hexadecimal_of_bytes(&sha256(unique.as_bytes()));
    return Ok(WorkItem {
        kind: WorkKind::SnapshotAhead,
        repository: source_root.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(AHEAD_REFERENCE),
        oid: digest,
        parent: pool.display().to_string(),
        created: now_iso(),
        tries: 0,
        transport: false,
    });
}

/// What:
///  Wake the commit service of the worktree whose own Git directory is `git_dir`,
/// saying nothing of its own,
/// as an add wakes it for its trash.
/// Why:
///   The route and the records must name one worktree:
/// the service that answers a worktree drains that worktree's own
///  work directory,
/// so a record written under another directory is refused there.
/// Why:
///   A wake that cannot be made leaves the records for the next command of the repository,
/// which is every commit
///  and every add,
/// so a snapshot that arrives later is never this add's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wakeService({ context, commonDir });
/// ```
pub(crate) fn wake_the_service(
    context: &GitContext,
    git_dir: &Path,
) -> Result<(), String> {
    let Some(Route {
        address,
        provision: host,
    }) = route_from_environment(context.real_git.as_path(), git_dir)
    else {
        return Err(String::from(
            "no commit service can be started from this session",
        ));
    };
    if let Some(setup) = host.as_ref()
        && let Err(failure) = provision(setup)
    {
        return Err(render(&failure));
    }
    let config: ClientConfig = ClientConfig {
        address,
        activation: Activation::Managed,
        start_wait: START_WAIT,
        heartbeat_silence: HEARTBEAT_SILENCE,
        start_watch: host.as_ref().map(manager_watch_for),
        keep_fresh: None,
        answer_ceiling: ANSWER_CEILING,
        withdraw_ceiling: WITHDRAW_CEILING,
        waiting_notice_after: waiting_notice_after(),
        extra_wait: SHORT_EXTRA_WAIT,
        start_ceiling: START_CEILING,
    };
    let mut silent: std::io::Sink = std::io::sink();
    return match wake(&config, git_dir, None, &[], &mut silent) {
        Ok(()) => Ok(()),
        Err(failure) => Err(render(&failure)),
    };
}

/// What:
///  The record of one request for a snapshot ahead,
/// which the caller wakes the service for.
/// Why:
///   The record is written before the service is told,
/// so a control asserts the record without a service,
/// and the caller answers under the id it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// askWith(gitDir, root, pool) // the record only; the caller wakes the service
/// ```
pub fn ask_with(
    git_dir: &Path,
    source_root: &Path,
    pool: &Path,
) -> Result<WorkItem, String> {
    let request: WorkItem = request_item(git_dir, source_root, pool)?;
    if let Err(reason) = enqueue(work_directory(git_dir).as_path(), &request) {
        return Err(format!(
            "the request of a snapshot ahead could not be written: {reason}"
        ));
    }
    debug(
        TAG,
        format!(
            "{}: asked for a snapshot ahead in {}",
            source_root.display(),
            pool.display()
        )
        .as_str(),
    );
    return Ok(request);
}

//! What:
//!  The authoritative ref updates a `git push` would send,
//! discovered by a dry-run push whose report Git writes itself.
//! Why:
//!  The manual-push policies must scan exactly what Git would publish,
//! so the probe asks Git:
//!  `git push --dry-run --porcelain` reports each destination and ref update (`manual_push_porcelain`),
//! the local
//!  objects come from that report or from one `rev-parse`,
//! and `git ls-remote --refs` confirms each destination value.
//!  A remote ref that changed between negotiation and confirmation stops the probe.
//!  No program of ours is started for this,
//! and no hook or shell text is written:
//! an earlier version recorded Git's
//!  update lines with a shell hook (`doc/planning/cli-git-no-self-invocation.md`).
//!  Every Git child here is bounded by the query deadline,
//! so a remote that never answers cannot hold the push.
//!  What one destination's confirmation holds sits in
//! `manual_push_probe_confirmation.rs`,
//!  and the steps that confirm every destination side by side in
//! `manual_push_probe_destination_blocks.rs`,
//!  so each file fits the package's length limit (`rust/max-lines`);
//!  this file re-exports their faces so no caller changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const updates = await probeManualPushUpdates({ gitPath, cwd, args });
//! ```

/// The probe's Git runner,
/// its bounded queries and its environment.
use crate::auto_push_single_flight::{
    PushGit,
    run_query,
};
use crate::command_push::push_shape;
/// The branches a destination reported,
/// which the upload plan reads without a connection of its own.
use crate::lfs_destination_confirms_tracking_refs::DestinationHeads;
/// Removes userinfo from text that may name a remote.
use crate::lfs_output_mask::mask_url_secrets_text;
/// The destination Git would pick for a bare push,
/// and the questions the remote resolver asks through it.
use crate::lfs_upload_plan::{
    GitQuery,
    resolve_push_remote,
};
use crate::manual_push_cause::ManualPushCause;
/// The porcelain report's parser,
/// and the block Git printed for one destination.
use crate::manual_push_porcelain::{
    PorcelainBlock,
    parse_porcelain,
};
/// One destination's confirmation,
/// the step that confirms it,
/// and the pattern that makes it list every branch head.
pub use crate::manual_push_probe_confirmation::{
    ALL_HEADS_PATTERN,
    ConfirmedDestination,
    confirm_group,
};
/// The work one destination's block is given,
/// and the events that keep the confirmations in block order.
use crate::manual_push_probe_destination_blocks::{
    ConfirmEachDestination,
    KeepConfirmedUntilTheFirstFailure,
};
pub use crate::manual_push_probe_error::{
    GitRefusal,
    ManualPushProbeError,
};
use crate::manual_push_probe_error::{
    cause_of_query,
    probe_error,
};
pub use crate::manual_push_update_and_query_limits::{
    LOCAL_QUERY_GAIN_STOPS_AT,
    MEASURED_WHOLE_HISTORY_QUERY,
    MOST_CONCURRENT_LOCAL_QUERIES,
    MOST_CONCURRENT_REMOTE_QUERIES,
    PushUpdate,
};
use crate::parallel_budget::{
    BudgetRequest,
    HostResources,
    worker_budget,
};
use crate::parallel_runner::{
    OrderedRun,
    run_ordered,
};
/// Owned OS strings for the probe's Git arguments.
use std::ffi::OsString;

/// The bytes one query holds at its peak:
/// `MEASURED_WHOLE_HISTORY_QUERY` rounded up to a power of two.
/// The coordinator's pick
///  under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Parallelism caps
///  and memory estimates").
/// A history that grows past a few tens of thousands more commits needs a larger figure.
pub const QUERY_MEMORY: u64 = MEASURED_WHOLE_HISTORY_QUERY.next_power_of_two();

/// The probe's Git arguments and the destination locations it confirms.
use crate::manual_push_probe_arguments::{
    locations_of,
    looks_like_location,
    probe_arguments,
};

/// What:
///  One manual-push discovery:
///  the updates the destination confirmed,
///  and the destination's branches when the push went to
///  exactly one destination.
/// Why:
///   The manual-push policies read the updates,
/// and the Git LFS upload plan reads the branches,
/// and
///  both come from the one connection the confirmation makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProbedPush = { updates: PushUpdate[]; destination?: DestinationHeads };
/// ```
#[derive(Clone, Debug)]
pub struct ProbedPush {
    /// The confirmed updates,
    /// in the order Git printed the destinations.
    pub updates: Vec<PushUpdate>,
    /// The branches the destination reported,
    /// so the upload plan asks the destination nothing.
    pub destination: Option<DestinationHeads>,
}

/// What:
///  Discover the exact updates the push would send,
/// confirmed against each destination,
/// and the destination's
///  branches.
/// Why:
///   A typed push already asks the destination for its refs once;
/// carrying those answers to the upload
///  plan is what keeps a typed push at `main`'s connection count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function probeManualPushAndDestination({ gitPath, cwd, args }): Promise<ProbedPush>;
/// ```
pub fn probe_manual_push_and_destination(
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<ProbedPush, ManualPushProbeError> {
    return probe_manual_push_and_destination_on(
        &HostResources::read(),
        git,
        global_prefix,
        post_subcommand,
    );
}

/// What:
///  Discover the exact updates the push would send,
/// confirmed against each destination.
/// Why:
///   The lifecycle's candidates and published-set reads both start from these updates;
/// a caller that
///  also plans an upload takes `probe_manual_push_and_destination`,
/// which reads the same connection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function probeManualPushUpdates({ gitPath, cwd, args }): Promise<PushUpdate[]>;
/// ```
pub fn probe_manual_push_updates(
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<Vec<PushUpdate>, ManualPushProbeError> {
    return probe_manual_push_and_destination(git, global_prefix, post_subcommand).map(|probed| {
        return probed.updates;
    });
}

/// `probe_manual_push_updates` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub fn probe_manual_push_updates_on(
    host: &HostResources,
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<Vec<PushUpdate>, ManualPushProbeError> {
    return probe_manual_push_and_destination_on(host, git, global_prefix, post_subcommand).map(
        |probed| {
            return probed.updates;
        },
    );
}

/// `probe_manual_push_and_destination` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub fn probe_manual_push_and_destination_on(
    host: &HostResources,
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<ProbedPush, ManualPushProbeError> {
    let arguments: Vec<OsString> = probe_arguments(global_prefix, post_subcommand);
    let output = match run_query(git, arguments.as_slice()) {
        Ok(found) => found,
        Err(error) => {
            return Err(probe_error(
                cause_of_query(&error),
                format!("probe push could not run: {error}"),
            ));
        }
    };
    let report: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let blocks = match parse_porcelain(report.as_str()) {
        Ok(found) => found,
        Err(message) => return Err(probe_error(ManualPushCause::UnexpectedOutput, message)),
    };
    if blocks.is_empty() {
        if output.code == Some(0) {
            return Ok(ProbedPush {
                updates: Vec::new(),
                destination: None,
            });
        }
        let said: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let message: String = mask_url_secrets_text(
            format!("Git could not determine push updates: {}", said.trim_end()).as_str(),
        );
        // Git ends `push --dry-run --porcelain` before any destination is reported only when it refuses the push itself
        //  (`transport_push` returns before it prints a block, and `die` and usage errors end the process); a status
        //  from a signal, or a refusal without a word, is not Git saying no.
        return Err(match output.code {
            Some(code) if !said.trim().is_empty() => ManualPushProbeError {
                message,
                refusal: Some(GitRefusal {
                    code,
                    stderr: said,
                }),
                cause: ManualPushCause::SelfExplained,
            },
            Some(_) | None => probe_error(ManualPushCause::GitSilent, message),
        });
    }
    let mut named = None;
    if let Some(shape) = push_shape(post_subcommand) {
        named = shape.repository;
    }
    let mut typed_location: Option<String> = None;
    if let Some(typed) = named.as_ref() {
        typed_location = Some(typed.to_string_lossy().into_owned());
    }
    // The remote resolver asks its questions through a query that carries the Git context and the global options.
    let mut asking: RemoteResolverQuery<'_> = RemoteResolverQuery {
        git,
        global_prefix,
    };
    let resolved_remote = match resolve_push_remote(named, &mut asking) {
        Ok(found) => found,
        Err(error) => {
            return Err(probe_error(
                ManualPushCause::SelfExplained,
                format!(
                    "Git could not tell which remote this push goes to: {}",
                    error.0
                ),
            ));
        }
    };
    let remote_name: String = resolved_remote.to_string_lossy().into_owned();
    // A destination the caller typed as an URL or a path is its own location, with its credentials; Git's printed `To`
    //  line has them removed, so it is never used as an address. A remote name costs one query for its push URLs.
    let locations: Vec<String> = match typed_location {
        Some(typed) if looks_like_location(typed.as_str()) => vec![typed],
        _ => locations_of(git, global_prefix, remote_name.as_str()),
    };
    if locations.len() != blocks.len() {
        return Err(probe_error(
            ManualPushCause::SelfExplained,
            mask_url_secrets_text(
                format!(
                    "Git reported {} destinations but {remote_name} has {} push locations",
                    blocks.len(),
                    locations.len()
                )
                .as_str(),
            ),
        ));
    }
    // Each destination is its own `To` block with its own `ls-remote`, so the confirmations are independent reads and run side by
    //  side, at most `MOST_CONCURRENT_REMOTE_QUERIES` at once; the updates come back in the order Git printed the blocks, and the
    //  first block that fails decides the error.
    let destinations: Vec<(&PorcelainBlock, &String)> =
        blocks.iter().zip(locations.iter()).collect();
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for index in 0..destinations.len() {
        lanes.push(vec![index]);
    }
    let workers: usize = worker_budget(&BudgetRequest {
        items: destinations.len(),
        per_item_memory: QUERY_MEMORY,
        ceiling: Some(MOST_CONCURRENT_REMOTE_QUERIES),
        cap: None,
        host,
    });
    let mut confirmed: Vec<ConfirmedDestination> = Vec::new();
    let mut failure: Option<ManualPushProbeError> = None;
    run_ordered(OrderedRun {
        items: destinations.as_slice(),
        lanes: lanes.as_slice(),
        workers,
        work: &ConfirmEachDestination {
            git,
            global_prefix,
            remote_name: remote_name.as_str(),
        },
        on_event: &mut KeepConfirmedUntilTheFirstFailure {
            confirmed: &mut confirmed,
            failure: &mut failure,
        },
    });
    if let Some(error) = failure {
        return Err(error);
    }
    // One destination's answer is the upload plan's question;
    // with several destinations no single answer names the place the upload
    //  stops, and the plan does not ask such a remote's tracking refs anyway.
    let destination: Option<DestinationHeads> = match confirmed.as_slice() {
        [only] => DestinationHeads::from_listing(remote_name.as_str(), only.listing.as_str()),
        _ => None,
    };
    let mut updates: Vec<PushUpdate> = Vec::new();
    for one in confirmed {
        updates.extend(one.updates);
    }
    return Ok(ProbedPush {
        updates,
        destination,
    });
}

/// What:
///  The questions the remote resolver asks Git,
/// each as a bounded query run with the global options in front.
/// Why:
///   The resolver asks through the `GitQuery` trait;
/// the Git context and the global options a question needs are two borrowed
///  values,
/// so a struct stands where a closure that captured them stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RemoteResolverQuery implements GitQuery { ask(args: string[]): string {} }
/// ```
struct RemoteResolverQuery<'a> {
    /// The coordinator's Git,
    /// whose queries are bounded.
    git: &'a PushGit,
    /// The global options that go before every question.
    global_prefix: &'a [OsString],
}

impl GitQuery for RemoteResolverQuery<'_> {
    fn ask(
        &mut self,
        query: &[&str],
    ) -> Result<String, String> {
        let mut owned: Vec<OsString> = self.global_prefix.to_vec();
        for part in query {
            owned.push(OsString::from(part));
        }
        let answered = match run_query(self.git, owned.as_slice()) {
            Ok(found) => found,
            Err(error) => return Err(error.to_string()),
        };
        let text: String = String::from_utf8_lossy(answered.stdout.as_slice())
            .trim()
            .to_owned();
        return match answered.code {
            Some(0) => Ok(text),
            Some(1) if text.is_empty() => Ok(text),
            code => Err(format!("exited with {code:?}")),
        };
    }
}

/// The probe's arguments and its confirmation rules are controlled in the test module.
#[cfg(test)]
#[path = "manual_push_probe_tests.rs"]
mod tests;

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
//!  update lines with a `#!/bin/sh` hook (`doc/planning/cli-git-no-self-invocation.md`).
//!  Every Git child here is bounded by the query deadline,
//! so a remote that never answers cannot hold the push.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const updates = await probeManualPushUpdates({ gitPath, cwd, args });
//! ```

/// The probe's Git runner,
/// its bounded queries and its environment.
use crate::auto_push_single_flight::{PushGit, run_query};
/// The destination Git would pick for a bare push,
/// and the porcelain report's parser.
use crate::lfs_upload_plan::{GitQuery, resolve_push_remote};
/// Removes userinfo from text that may name a remote,
/// and the cause a failure carries.
use crate::lfs_output_mask::mask_url_secrets_text;
use crate::manual_push_cause::ManualPushCause;
pub use crate::manual_push_probe_error::{GitRefusal, ManualPushProbeError};
use crate::manual_push_probe_error::{cause_of_query, probe_error};
use crate::manual_push_porcelain::{PorcelainBlock, PorcelainUpdate, RemoteBefore, parse_porcelain};
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::line_sink::LineSink;
use crate::parallel_runner::{Event, OrderedEvents, OrderedRun, OrderedWork, run_ordered};
/// The branches a destination reported,
/// which the upload plan reads without a connection of its own.
use crate::lfs_destination_confirms_tracking_refs::DestinationHeads;
use crate::command_push::push_shape;
/// Owned OS strings for the probe's Git arguments.
use std::ffi::OsString;
/// Maps each ref the remote listed to the object name it reported.
use std::collections::HashMap;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// The parser of the ls-remote listing,
/// beside the other listing parsers.
use crate::manual_push_range_published_tips::oids_by_ref;
pub use crate::manual_push_update_and_query_limits::{LOCAL_QUERY_GAIN_STOPS_AT, MEASURED_WHOLE_HISTORY_QUERY, MOST_CONCURRENT_LOCAL_QUERIES, MOST_CONCURRENT_REMOTE_QUERIES, PushUpdate};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

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
use crate::manual_push_probe_arguments::{answer, locations_of, looks_like_location, probe_arguments};

/// What:
///  The object names of local refs,
/// in order,
/// from one `rev-parse`.
/// Why:
///   A new ref's report line has no object name;
/// the ref's own value is what the push sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await git(['rev-parse', ...names])).split('\n')
/// ```
fn local_objects(git: &PushGit, global_prefix: &[OsString], names: &[&str]) -> Result<Vec<String>, ManualPushProbeError> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("rev-parse"));
    arguments.extend(names.iter().map(OsString::from));
    let Some(listing) = answer(git, arguments.as_slice()) else {
        return Err(probe_error(ManualPushCause::ContentMissing, format!("the local objects of {names:?} could not be read")));
    };
    let objects: Vec<String> = listing.lines().map(str::to_owned).collect();
    if objects.len() != names.len() {
        return Err(probe_error(ManualPushCause::UnexpectedOutput, format!("rev-parse answered {} names for {}", objects.len(), names.len())));
    }
    return Ok(objects);
}

/// The pattern that makes one confirmation query list every branch head too,
/// which is the answer the Git LFS upload plan
///  needs about this clone's tracking refs.
/// `git ls-remote` matches a pattern against the tail of a ref name,
/// so this one adds the destination's
///  heads to the refs the report named.
pub const ALL_HEADS_PATTERN: &str = "refs/heads/*";

/// What:
///  One destination's confirmation:
///  the updates it confirmed,
///  and everything its one `ls-remote` reported.
/// Why:
///   The upload plan for a typed push asks the destination which of this clone's tracking refs it
///  reports;
/// that question is answered by the same connection if the confirmation lists every branch head,
///  and the listing is carried so the plan asks nothing more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfirmedDestination = { updates: PushUpdate[]; listing: string };
/// ```
#[derive(Clone, Debug)]
pub struct ConfirmedDestination {
    /// The updates this destination confirmed.
    pub updates: Vec<PushUpdate>,
    /// The listing its one `ls-remote` printed,
    /// which names every branch head as well as the pushed refs.
    pub listing: String,
}

pub fn confirm_group(
    git: &PushGit,
    remote_location: &str,
    remote_name: &str,
    updates: &[(PorcelainUpdate, Option<String>)],
) -> Result<ConfirmedDestination, ManualPushProbeError> {
    let mut arguments: Vec<OsString> = vec![
        OsString::from("ls-remote"),
        OsString::from("--refs"),
        OsString::from(remote_location),
    ];
    for (update, _local) in updates {
        arguments.push(OsString::from(&update.to));
    }
    // The branch heads of the destination answer the LFS upload plan's question about this clone's tracking refs, so they
    //  travel in this query rather than in one of their own.
    arguments.push(OsString::from(ALL_HEADS_PATTERN));
    let output = match run_query(git, arguments.as_slice()) {
        Ok(found) => found,
        Err(error) => return Err(probe_error(cause_of_query(&error), mask_url_secrets_text(format!("ls-remote of {remote_location} failed: {error}").as_str()))),
    };
    if output.code != Some(0) {
        return Err(probe_error(
            ManualPushCause::RemoteUnreachable,
            mask_url_secrets_text(
                format!("ls-remote of {remote_location} failed: {}", String::from_utf8_lossy(&output.stderr).trim_end()).as_str(),
            ),
        ));
    }
    let listing: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let listed: HashMap<&str, &str> = oids_by_ref(listing.as_str());
    let mut confirmed: Vec<PushUpdate> = Vec::new();
    for (update, local) in updates {
        crate::collected_elements_looked_at::note_looked_at(1);
        let authoritative: Option<String> = listed.get(update.to.as_str()).copied().map(str::to_owned);
        let changed: bool = match &update.before {
            RemoteBefore::Absent => authoritative.is_some(),
            RemoteBefore::Known(reported) => authoritative.as_ref() != Some(reported),
            RemoteBefore::Unknown => false,
        };
        // What: a destination that moved to exactly the object this update sends asks for nothing.
        // Why:  the commit's own auto-push publishes the same commit while a typed push is on its way, so the one
        //       change a destination can take between Git's dry run and this read is that push landing; the person asked
        //       for this object, the destination holds it, and the update adds no commit to the range the policies read.
        // A destination that moved to any other value is a change this push cannot account for.
        if changed && authoritative.as_ref() != local.as_ref() {
            return Err(probe_error(ManualPushCause::RemoteMoved, format!("Remote ref changed during manual-push discovery: {}", update.to)));
        }
        if changed {
            debug(TAG, &format!("{} already holds the object this push sends, so nothing is left to publish from it", update.to));
        }
        confirmed.push(PushUpdate {
            local_oid: local.clone(),
            remote_oid: authoritative,
            remote_name: remote_name.to_owned(),
            remote_ref: update.to.clone(),
        });
    }
    return Ok(ConfirmedDestination { updates: confirmed, listing });
}

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
    return probe_manual_push_and_destination_on(&HostResources::read(), git, global_prefix, post_subcommand);
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
    return probe_manual_push_and_destination(git, global_prefix, post_subcommand).map(|probed| { return probed.updates; });
}

/// `probe_manual_push_updates` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub fn probe_manual_push_updates_on(
    host: &HostResources,
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<Vec<PushUpdate>, ManualPushProbeError> {
    return probe_manual_push_and_destination_on(host, git, global_prefix, post_subcommand).map(|probed| { return probed.updates; });
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
        Err(error) => return Err(probe_error(cause_of_query(&error), format!("probe push could not run: {error}"))),
    };
    let report: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let blocks = match parse_porcelain(report.as_str()) {
        Ok(found) => found,
        Err(message) => return Err(probe_error(ManualPushCause::UnexpectedOutput, message)),
    };
    if blocks.is_empty() {
        if output.code == Some(0) {
            return Ok(ProbedPush { updates: Vec::new(), destination: None });
        }
        let said: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let message: String = mask_url_secrets_text(format!("Git could not determine push updates: {}", said.trim_end()).as_str());
        // Git ends `push --dry-run --porcelain` before any destination is reported only when it refuses the push itself
        //  (`transport_push` returns before it prints a block, and `die` and usage errors end the process); a status
        //  from a signal, or a refusal without a word, is not Git saying no.
        return Err(match output.code {
            Some(code) if !said.trim().is_empty() => {
                ManualPushProbeError { message, refusal: Some(GitRefusal { code, stderr: said }), cause: ManualPushCause::SelfExplained }
            }
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
    let mut asking: RemoteResolverQuery<'_> = RemoteResolverQuery { git, global_prefix };
    let resolved_remote = match resolve_push_remote(named, &mut asking) {
        Ok(found) => found,
        Err(error) => return Err(probe_error(ManualPushCause::SelfExplained, format!("Git could not tell which remote this push goes to: {}", error.0))),
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
                format!("Git reported {} destinations but {remote_name} has {} push locations", blocks.len(), locations.len()).as_str(),
            ),
        ));
    }
    // Each destination is its own `To` block with its own `ls-remote`, so the confirmations are independent reads and run side by
    //  side, at most `MOST_CONCURRENT_REMOTE_QUERIES` at once; the updates come back in the order Git printed the blocks, and the
    //  first block that fails decides the error.
    let destinations: Vec<(&PorcelainBlock, &String)> = blocks.iter().zip(locations.iter()).collect();
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for index in 0..destinations.len() {
        lanes.push(vec![index]);
    }
    let workers: usize = worker_budget(&BudgetRequest { items: destinations.len(), per_item_memory: QUERY_MEMORY, ceiling: Some(MOST_CONCURRENT_REMOTE_QUERIES), cap: None, host });
    let mut confirmed: Vec<ConfirmedDestination> = Vec::new();
    let mut failure: Option<ManualPushProbeError> = None;
    run_ordered(OrderedRun {
        items: destinations.as_slice(),
        lanes: lanes.as_slice(),
        workers,
        work: &ConfirmEachDestination { git, global_prefix, remote_name: remote_name.as_str() },
        on_event: &mut KeepConfirmedUntilTheFirstFailure { confirmed: &mut confirmed, failure: &mut failure },
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
    return Ok(ProbedPush { updates, destination });
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
    fn ask(&mut self, query: &[&str]) -> Result<String, String> {
        let mut owned: Vec<OsString> = self.global_prefix.to_vec();
        for part in query {
            owned.push(OsString::from(part));
        }
        let answered = match run_query(self.git, owned.as_slice()) {
            Ok(found) => found,
            Err(error) => return Err(error.to_string()),
        };
        let text: String = String::from_utf8_lossy(answered.stdout.as_slice()).trim().to_owned();
        return match answered.code {
            Some(0) => Ok(text),
            Some(1) if text.is_empty() => Ok(text),
            code => Err(format!("exited with {code:?}")),
        };
    }
}

/// What:
///  The confirmation of one destination,
/// run by a worker of the ordered runner.
/// Why:
///   The Git context,
/// the global options and the remote name are what each confirmation reads;
/// the runner shares them among its
///  worker threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConfirmEachDestination implements OrderedWork<[Block, string], Result<ConfirmedDestination>> { run(i, destination) { return confirmBlock(git, globalPrefix, remoteName, ...destination); } }
/// ```
struct ConfirmEachDestination<'a> {
    /// The coordinator's Git,
    /// whose queries are bounded.
    git: &'a PushGit,
    /// The global options that go before every question.
    global_prefix: &'a [OsString],
    /// The remote the push goes to.
    remote_name: &'a str,
}

impl OrderedWork<(&PorcelainBlock, &String), Result<ConfirmedDestination, ManualPushProbeError>> for ConfirmEachDestination<'_> {
    fn run(&self, _index: usize, destination: &(&PorcelainBlock, &String), _say: &mut dyn LineSink) -> Result<ConfirmedDestination, ManualPushProbeError> {
        return confirm_block(self.git, self.global_prefix, self.remote_name, destination.0, destination.1.as_str());
    }
}

/// What:
///  Keeps the confirmed updates in block order and the first failure;
/// once a block has failed,
/// later results are dropped.
/// Why:
///   The runner reports on the calling thread only and in block order,
/// so the first block that fails decides the error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class KeepConfirmedUntilTheFirstFailure implements OrderedEvents<Result<ConfirmedDestination>> { confirmed: ConfirmedDestination[]; failure?: Error }
/// ```
struct KeepConfirmedUntilTheFirstFailure<'a> {
    /// The confirmations so far,
    /// in block order.
    confirmed: &'a mut Vec<ConfirmedDestination>,
    /// The first failure,
    /// when a block has failed.
    failure: &'a mut Option<ManualPushProbeError>,
}

impl OrderedEvents<Result<ConfirmedDestination, ManualPushProbeError>> for KeepConfirmedUntilTheFirstFailure<'_> {
    fn on_event(&mut self, event: Event<'_, Result<ConfirmedDestination, ManualPushProbeError>>) {
        if let Event::End(_index, outcome) = event {
            match outcome {
                Ok(destination) if self.failure.is_none() => self.confirmed.push(destination),
                Err(error) if self.failure.is_none() => *self.failure = Some(error),
                Ok(_) | Err(_) => {}
            }
        }
    }
}

/// What:
///  Resolve the local objects of one destination's block and confirm its values with `ls-remote`.
/// Why:
///   A block is everything the probe needs for one destination,
/// so blocks can be confirmed independently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function confirmBlock({ git, globalPrefix, remoteName, block, location }): Promise<ConfirmedDestination>;
/// ```
fn confirm_block(git: &PushGit, global_prefix: &[OsString], remote_name: &str, block: &PorcelainBlock, location: &str) -> Result<ConfirmedDestination, ManualPushProbeError> {
    let mut unnamed: Vec<&str> = Vec::new();
    for update in &block.updates {
        if !update.from.is_empty() && update.after.is_none() {
            unnamed.push(update.from.as_str());
        }
    }
    let mut resolved = local_objects(git, global_prefix, unnamed.as_slice())?.into_iter();
    let mut group: Vec<(PorcelainUpdate, Option<String>)> = Vec::new();
    for update in &block.updates {
        // A name Git already printed is used as it is; the others take the next object the single `rev-parse` answered.
        let local: Option<String> = if update.from.is_empty() {
            None
        } else if update.after.is_some() {
            update.after.clone()
        } else {
            resolved.next()
        };
        group.push((update.clone(), local));
    }
    return confirm_group(git, location, remote_name, group.as_slice());
}

/// The probe's arguments and its confirmation rules are controlled in the test module.
#[cfg(test)]
#[path = "manual_push_probe_tests.rs"]
mod tests;

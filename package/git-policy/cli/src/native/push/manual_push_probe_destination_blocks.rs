//! What:
//!  One destination's block of Git's own dry-run report,
//!  confirmed:
//!  the local objects a new ref needs,
//!  the updates Git printed for that destination,
//!  and the two pieces that confirm every destination side by side
//!  in the order Git printed them.
//! Why:
//!   Each destination is its own `To` block with its own `ls-remote`,
//!  so the blocks are independent reads and run at once;
//!  the runner shares the Git context,
//!  the global options and the remote name
//!  among its worker threads;
//!  it reports on the calling thread only,
//!  in block order,
//!  and the first block that fails decides the error.
//!  These steps sit in a file of their own
//!  because the probe's own file is more than the package's length limit allows one (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runOrdered({ items: destinations, workers, work: confirmEachDestination, onEvent: keepConfirmedUntilTheFirstFailure });
//! ```

/// The probe's Git runner,
/// its bounded queries and its environment.
use crate::auto_push_single_flight::PushGit;
/// The sink a worker writes a line to.
use crate::line_sink::LineSink;
/// Why a manual push's scope could not be established,
/// as the cause its hint is chosen by.
use crate::manual_push_cause::ManualPushCause;
/// The block Git's own report printed for one destination,
/// and one update in it.
use crate::manual_push_porcelain::{
    PorcelainBlock,
    PorcelainUpdate,
};
/// The probe's Git arguments,
/// and the answer of one query of them.
use crate::manual_push_probe_arguments::answer;
/// One destination's confirmation,
/// and the step that confirms it.
use crate::manual_push_probe_confirmation::{
    ConfirmedDestination,
    confirm_group,
};
/// The probe's failure type and the way a failure is built.
use crate::manual_push_probe_error::{
    ManualPushProbeError,
    probe_error,
};
/// The ordered runner's work and event traits,
/// and the event it reports.
use crate::parallel_runner::{
    Event,
    OrderedEvents,
    OrderedWork,
};
/// Owned OS strings for the probe's Git arguments.
use std::ffi::OsString;

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
fn local_objects(
    git: &PushGit,
    global_prefix: &[OsString],
    names: &[&str],
) -> Result<Vec<String>, ManualPushProbeError> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("rev-parse"));
    arguments.extend(names.iter().map(OsString::from));
    let Some(listing) = answer(git, arguments.as_slice()) else {
        return Err(probe_error(
            ManualPushCause::ContentMissing,
            format!("the local objects of {names:?} could not be read"),
        ));
    };
    let objects: Vec<String> = listing.lines().map(str::to_owned).collect();
    if objects.len() != names.len() {
        return Err(probe_error(
            ManualPushCause::UnexpectedOutput,
            format!(
                "rev-parse answered {} names for {}",
                objects.len(),
                names.len()
            ),
        ));
    }
    return Ok(objects);
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
fn confirm_block(
    git: &PushGit,
    global_prefix: &[OsString],
    remote_name: &str,
    block: &PorcelainBlock,
    location: &str,
) -> Result<ConfirmedDestination, ManualPushProbeError> {
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
pub struct ConfirmEachDestination<'a> {
    /// The coordinator's Git,
    /// whose queries are bounded.
    pub git: &'a PushGit,
    /// The global options that go before every question.
    pub global_prefix: &'a [OsString],
    /// The remote the push goes to.
    pub remote_name: &'a str,
}

impl OrderedWork<(&PorcelainBlock, &String), Result<ConfirmedDestination, ManualPushProbeError>>
    for ConfirmEachDestination<'_>
{
    fn run(
        &self,
        _index: usize,
        destination: &(&PorcelainBlock, &String),
        _say: &mut dyn LineSink,
    ) -> Result<ConfirmedDestination, ManualPushProbeError> {
        return confirm_block(
            self.git,
            self.global_prefix,
            self.remote_name,
            destination.0,
            destination.1.as_str(),
        );
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
pub struct KeepConfirmedUntilTheFirstFailure<'a> {
    /// The confirmations so far,
    /// in block order.
    pub confirmed: &'a mut Vec<ConfirmedDestination>,
    /// The first failure,
    /// when a block has failed.
    pub failure: &'a mut Option<ManualPushProbeError>,
}

impl OrderedEvents<Result<ConfirmedDestination, ManualPushProbeError>>
    for KeepConfirmedUntilTheFirstFailure<'_>
{
    fn on_event(
        &mut self,
        event: Event<'_, Result<ConfirmedDestination, ManualPushProbeError>>,
    ) {
        if let Event::End(_index, outcome) = event {
            match outcome {
                Ok(destination) if self.failure.is_none() => self.confirmed.push(destination),
                Err(error) if self.failure.is_none() => *self.failure = Some(error),
                Ok(_) | Err(_) => {}
            }
        }
    }
}

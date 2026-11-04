//! What:
//!  The lane lifecycle at its call sites (plan step 2): the client's step after a request is accepted (create the receipt, ring a running
//!  service, otherwise launch the flusher through the manager route with the fork as fallback), the syncer's pass (retry the recorded
//!  receipt errors, then force every receipt), the replacement owner's startup sweep (the same pass plus one read of the state log), and the
//!  planned-shutdown seal (a final pass, then a reclaiming checkpoint of everything the lane holds).
//! Why:
//!  Each is the first caller of a component that had none. The receipt is the registration of the obligation to force
//!  (`commit_flusher.rs`), so it exists before anything is rung or launched, and no step depends on a message having arrived. The running
//!  owner's syncer retries recorded errors instead of leaving them to the next start (coordinator pick of 2026-11-04, veto open). The seal is
//!  the state log's real reclaimer: it keeps the state it found and replaces the segments its document is shown to contain. Nothing here is
//!  reachable from `git commit`: the shipped commit path goes through the resident service, and the cutover to this protocol is plan step 3.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // clientAfterAccept(lane, id, ring, route): createReceipt; if (ring(lane)) return 'rung'; return launchRouted(lane, route)
//! ```

use super::commit_flusher::{FlushError, Flushed, FlusherLaunchError, RECEIPT_ERRORS, flush_lane};
pub use super::commit_flusher_unit::{ManagerRoute, Routed, launch_routed};
pub use super::commit_state_log::capture_lane_closure;
use super::commit_state_log::{ClosureEntry, LaneDocument, LaneState, Selected, StateError, StateWriter, select};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The syncer's in-flight copy of the record of receipt errors: the lines it took and is retrying.
pub const RETRYING: &str = "receipt-errors.retrying";

/// Creates `lane/receipt/<id>`, the registration of the obligation to force; the receipt is visible when this returns.
pub fn create_receipt(lane: &Path, id: &str) -> io::Result<PathBuf> {
    if !super::commit_integrity::is_identity(id) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "a receipt is named by a request identity"));
    }
    let directory: PathBuf = lane.join(super::commit_flusher::RECEIPT_DIR);
    fs::create_dir_all(&directory)?;
    let path: PathBuf = directory.join(id);
    OpenOptions::new().write(true).create(true).truncate(false).open(&path)?;
    return Ok(path);
}

/// What the client's step did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientStep {
    /// A running service took the ring and carries the receipt.
    Rung,
    /// No service answered; the flusher was started this way.
    Launched(Routed),
}

/// Why the client's step failed.
#[derive(Debug)]
pub enum ClientError {
    /// The receipt could not be created: the request is not registered for forcing.
    Receipt(io::Error),
    /// The flusher could not be started by either route; the receipts of the lane were recorded as not forced.
    Launch(FlusherLaunchError),
}

/// What:
///  The client's step after its request is accepted: creates the receipt, rings a running service, and when none takes the ring starts the
///  flusher (the manager route first, the double fork as the fallback).
/// Why:
///  The ring is a courtesy to a service that already runs (it forces on its own schedule); the receipt, not the ring, is the obligation, so a
///  ring that errors or is not answered launches the flusher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clientAfterAccept(lane, id, ring, route): ClientStep; // throws ClientError
/// ```
pub fn client_after_accept(lane: &Path, id: &str, ring: &mut dyn FnMut(&Path) -> io::Result<bool>, route: &mut dyn ManagerRoute) -> Result<ClientStep, ClientError> {
    create_receipt(lane, id).map_err(ClientError::Receipt)?;
    if ring(lane).unwrap_or(false) {
        return Ok(ClientStep::Rung);
    }
    return launch_routed(lane, route).map(ClientStep::Launched).map_err(ClientError::Launch);
}

/// What one syncer pass did.
#[derive(Debug, Eq, PartialEq)]
pub struct SyncPass {
    /// Receipts the pass forced.
    pub forced: usize,
    /// Recorded error lines it took up and cleared.
    pub retried: usize,
    /// Another flusher held the lane's lock, so the pass cleared nothing and the record is kept.
    pub held: bool,
}

/// The names (first field) of the lines of a record.
fn names_of(text: &str) -> BTreeSet<&str> {
    return text.lines().filter_map(|line| { return line.split('\t').next(); }).collect();
}

/// Puts the lines of the in-flight copy back into the record, once per name, and removes the copy.
fn merge_back(lane: &Path) -> io::Result<()> {
    let taken: String = fs::read_to_string(lane.join(RETRYING))?;
    let current: String = fs::read_to_string(lane.join(RECEIPT_ERRORS)).unwrap_or_default();
    let known: BTreeSet<&str> = names_of(&current);
    let mut record = OpenOptions::new().create(true).append(true).open(lane.join(RECEIPT_ERRORS))?;
    for line in taken.lines() {
        if line.split('\t').next().is_some_and(|name| { return !known.contains(name); }) {
            record.write_all(format!("{}\n", line).as_bytes())?;
        }
    }
    fs::remove_file(lane.join(RETRYING))?;
    return Ok(());
}

/// One syncer pass with a hook between taking the record and forcing (a test puts a late line there).
pub fn syncer_pass_with(lane: &Path, after_take: &mut dyn FnMut()) -> Result<SyncPass, FlushError> {
    // A copy a dead syncer left is retried as it is; otherwise the record is renamed, so a line recorded during the pass lands in a new file.
    if !lane.join(RETRYING).exists() {
        match fs::rename(lane.join(RECEIPT_ERRORS), lane.join(RETRYING)) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(FlushError::Io(error)),
        }
    }
    after_take();
    let retried: usize = fs::read_to_string(lane.join(RETRYING)).map_or(0, |text| { return text.lines().count(); });
    let outcome = flush_lane(lane);
    match outcome {
        Ok(Flushed::Finished { forced }) | Ok(Flushed::Handed { forced }) => {
            match fs::remove_file(lane.join(RETRYING)) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(FlushError::Io(error)),
            }
            return Ok(SyncPass { forced, retried, held: false });
        }
        Ok(Flushed::Held) => {
            if lane.join(RETRYING).exists() {
                merge_back(lane)?;
            }
            return Ok(SyncPass { forced: 0, retried: 0, held: true });
        }
        Err(error) => {
            if lane.join(RETRYING).exists() {
                merge_back(lane)?;
            }
            return Err(error);
        }
    }
}

/// The syncer's pass: retries the recorded errors and forces every receipt in the lane.
pub fn syncer_pass(lane: &Path) -> Result<SyncPass, FlushError> {
    return syncer_pass_with(lane, &mut || {});
}

/// Why a lifecycle step failed.
#[derive(Debug)]
pub enum LifecycleError {
    /// The state log could not be read or written.
    State(StateError),
    /// Forcing failed.
    Flush(FlushError),
    /// A lane file could not be read.
    Io(io::Error),
}

impl From<StateError> for LifecycleError {
    fn from(error: StateError) -> LifecycleError {
        return LifecycleError::State(error);
    }
}

impl From<FlushError> for LifecycleError {
    fn from(error: FlushError) -> LifecycleError {
        return LifecycleError::Flush(error);
    }
}

impl From<io::Error> for LifecycleError {
    fn from(error: io::Error) -> LifecycleError {
        return LifecycleError::Io(error);
    }
}

/// What a replacement owner found at its start.
#[derive(Debug, Eq, PartialEq)]
pub struct Recovery {
    /// The generation of the state log's newest eligible record; `None` for a lane with no log.
    pub generation: Option<u64>,
    /// Receipts the sweep forced.
    pub forced: usize,
    /// Recorded error lines it took up.
    pub retried: usize,
    /// Interrupted disposals it finished.
    pub disposals: usize,
}

/// What:
///  The replacement owner's startup sweep: reads the state log once and forces every receipt in the directory, whether or not the record
///  names it, taking up the copy a dead syncer left.
/// Why:
///  A receipt in the directory is the obligation; the record is only a hint, so a receipt whose error line was lost is still served.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startupSweep(lane: string, state: string): Recovery;
/// ```
pub fn startup_sweep(lane: &Path, state: &Path) -> Result<Recovery, LifecycleError> {
    let generation: Option<u64> = select(state)?.map(|chosen| { return chosen.generation; });
    let pass: SyncPass = syncer_pass(lane)?;
    // A disposal the person authorized and a crash interrupted is finished from its durable `disposed` line; a root that is not in the
    // checked representation cannot be replaced, so the sweep leaves such a lane to the person.
    let disposals: usize = if generation.is_some() { super::commit_disposal::resume_disposals(lane, state)? } else { 0 };
    return Ok(Recovery { generation, forced: pass.forced, retried: pass.retried, disposals });
}

/// What the seal wrote.
#[derive(Debug, Eq, PartialEq)]
pub struct Sealed {
    /// The generation of the sealing checkpoint.
    pub generation: u64,
}

/// The identities (and saved-content keys) of the previous record's items that the lane no longer holds, for the writer to name as retired.
fn dropped_since(previous: &Selected, closure: &[ClosureEntry]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for old in &previous.items {
        if closure.iter().any(|entry| { return entry.path() == old.path(); }) {
            continue;
        }
        if let Some(leaf) = old.path().file_name().and_then(|name| { return name.to_str(); }) {
            names.push(leaf.chars().take(32).collect());
        }
    }
    return names;
}

/// What:
///  The planned-shutdown seal: a final pass over the receipts, then a reclaiming checkpoint of everything the lane holds, carrying the state
///  of the newest checked document (a newest document that is not in the checked representation contributes no state, and its segment is
///  not reclaimed).
/// Why:
///  The seal is the state log's real reclaimer. Requests the lane no longer holds are named as retired, and the component accepts them only
///  where a ledger line in the checkpoint's own closure proves the discharge; a state that names a request the lane no longer holds is not a
///  document, so the seal fails and the older segments stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sealForShutdown(lane: string, state: string): Sealed; // throws LifecycleError
/// ```
pub fn seal_for_shutdown(lane: &Path, state: &Path) -> Result<Sealed, LifecycleError> {
    syncer_pass(lane)?;
    let closure: Vec<ClosureEntry> = capture_lane_closure(lane)?;
    let previous: Option<Selected> = select(state)?;
    let carried: LaneState = previous.as_ref().and_then(|chosen| { return LaneDocument::parse(&chosen.payload); }).map_or_else(LaneState::default, |document| { return document.state().clone(); });
    let retired: Vec<String> = previous.as_ref().map_or_else(Vec::new, |chosen| { return dropped_since(chosen, &closure); });
    let document: LaneDocument = LaneDocument::from_closure(&closure)
        .ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the lane holds an append-only file the checked representation cannot express"); })?
        .with_state(carried);
    let generation: u64 = previous.as_ref().map_or(1, |chosen| { return chosen.generation + 1; });
    let mut writer: StateWriter = StateWriter::acquire(state)?;
    let candidate = writer.begin_checkpoint_reclaiming_retiring(generation, &document, &closure, &retired)?;
    writer.complete_checkpoint(&candidate)?;
    return Ok(Sealed { generation });
}

/// What:
///  The departure's acknowledgment of the state log, with the retry the races need: forces the segment recovery selects
///  (`commit_state_log::sync_prefix`) and returns the prefix only when `state_covers` says the root recovery selects now is that record or
///  descends from it. A segment unlinked between the selection and the open (`ENOENT`) and a root that changed to one that does not cover the
///  capture both capture again, at most `attempts` times in all.
/// Why:
///  The seal and any other reclaimer run concurrently with a departure. Forcing a file that is no longer a root is not an acknowledgment,
///  and a return on the survival of the old file would promise a record the next recovery does not select (review of step 1, finding 5).
///  The attempts are the caller's: the bound of the wait that contains this call is the caller's register entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function departureAcknowledge(state: string, attempts: number): Prefix; // throws
/// ```
pub fn departure_acknowledge(state: &Path, attempts: u32) -> io::Result<super::commit_state_log::Prefix> {
    let mut last: io::Error = io::Error::other("the departure was given no attempt");
    for _attempt in 0..attempts {
        match super::commit_state_log::sync_prefix(state) {
            Ok(prefix) => match super::commit_state_log::state_covers(state, &prefix) {
                Ok(true) => return Ok(prefix),
                Ok(false) => last = io::Error::other("the state log's root changed to one that does not cover the captured record"),
                Err(error) => return Err(io::Error::other(format!("the state log could not be read: {:?}", error))),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => last = error,
            Err(error) => return Err(error),
        }
    }
    return Err(last);
}

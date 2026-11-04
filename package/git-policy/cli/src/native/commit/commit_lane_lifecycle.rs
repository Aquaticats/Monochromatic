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
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

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
    // The unit runs in the manager's working directory, not this client's: the obligation is created and launched under one absolute path.
    let absolute: PathBuf = std::path::absolute(lane).map_err(ClientError::Receipt)?;
    let lane: &Path = absolute.as_path();
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

/// One syncer pass with a hook between reading the record and forcing (a test writes a late line there).
///
/// The record is never renamed or replaced: a recorder that holds it open would write into a file nobody reads. The pass reads it, forces
/// every receipt of the lane (the record names a subset), and only if that succeeded takes the record's exclusive lock without waiting and
/// empties it in place when it is still exactly what the pass read. Recorders append under the shared lock, so none writes while it is
/// emptied; a line recorded meanwhile makes the length differ and nothing is removed (the next pass retries the old lines again, which is
/// idempotent).
pub fn syncer_pass_with(lane: &Path, after_read: &mut dyn FnMut()) -> Result<SyncPass, FlushError> {
    let record: PathBuf = lane.join(RECEIPT_ERRORS);
    let read: Vec<u8> = match fs::read(&record) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(FlushError::Io(error)),
    };
    let retried: usize = read.iter().filter(|byte| { return **byte == b'\n'; }).count();
    after_read();
    match flush_lane(lane)? {
        Flushed::Held => return Ok(SyncPass { forced: 0, retried: 0, held: true }),
        Flushed::Finished { forced } | Flushed::Handed { forced } => {
            if retried > 0 {
                clear_if_unchanged(&record, read.len())?;
            }
            return Ok(SyncPass { forced, retried, held: false });
        }
    }
}

/// Empties the record in place when it is `length` bytes long and its exclusive lock is free; otherwise leaves it.
fn clear_if_unchanged(record: &Path, length: usize) -> Result<(), FlushError> {
    let file = match OpenOptions::new().read(true).write(true).open(record) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(FlushError::Io(error)),
    };
    match file.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => return Ok(()),
        Err(std::fs::TryLockError::Error(error)) => return Err(FlushError::Io(error)),
    }
    if file.metadata()?.len() == length as u64 {
        file.set_len(0)?;
    }
    return Ok(());
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
    /// The generation of the state log's newest eligible record; `None` for a lane with no log or a log that could not be read.
    pub generation: Option<u64>,
    /// Why the state log could not be read (a damaged or conflicting log), reported beside the receipts that were served anyway.
    pub state_error: Option<String>,
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
    // The receipts are served first and whatever the state log is: a damaged log is reported, never a reason to leave a receipt unforced.
    let pass: SyncPass = syncer_pass(lane)?;
    let (generation, state_error): (Option<u64>, Option<String>) = match select(state) {
        Ok(chosen) => (chosen.map(|found| { return found.generation; }), None),
        Err(error) => (None, Some(format!("{:?}", error))),
    };
    // A disposal the person authorized and a crash interrupted is finished from its durable `disposed` line; a root that is not in the
    // checked representation cannot be replaced, so the sweep leaves such a lane to the person.
    let disposals: usize = if generation.is_some() { super::commit_disposal::resume_disposals(lane, state)? } else { 0 };
    return Ok(Recovery { generation, state_error, forced: pass.forced, retried: pass.retried, disposals });
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

/// Forces what the acknowledged record retains (the request files, the saved content, the marks, the ledger and the order file), then
/// confirms again that the record is still covered: a retained file that a power loss dropped would make the durable record ineligible.
fn force_closure(state: &Path, prefix: &super::commit_state_log::Prefix) -> io::Result<()> {
    let lane: PathBuf = state.parent().map_or_else(|| { return PathBuf::from("."); }, PathBuf::from);
    return super::commit_state_log::force_items(&lane, &prefix.items);
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
            Ok(prefix) => {
                // The closure is forced before the coverage is confirmed, so the confirmation is of a record whose dependencies are durable.
                force_closure(state, &prefix)?;
                match super::commit_state_log::state_covers(state, &prefix) {
                    Ok(true) => return Ok(prefix),
                    Ok(false) => last = io::Error::other("the state log's root changed to one that does not cover the captured record"),
                    Err(error) => return Err(io::Error::other(format!("the state log could not be read: {:?}", error))),
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => last = error,
            Err(error) => return Err(error),
        }
    }
    return Err(last);
}

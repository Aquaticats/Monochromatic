//! What:
//!  The disposal of saved content on the component: the person's explicit disposal of one saved item appends a `disposed` line to the
//!  lane ledger, checkpoints a replacement root that no longer names the item (its closure holds the ledger prefix with the line, which
//!  completion forces together with the candidate and its directory entry), and only then unlinks the content.
//! Why:
//!  Recovery validates every retained saved-content entry of a root, so a root that still names an item whose file is gone is ineligible
//!  whatever the ledger says (review of step 1, finding 6; `model_disposal_r23`). The order that keeps an eligible root at every power
//!  loss is: line durable, replacement root durable, unlink. Nothing here is called by retirement: the only constructor of the
//!  authorization is the person's explicit command, and the item must be one the newest root holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // retireSavedContent(lane, state, auth): appendDisposed; checkpointRetiring(item); unlinkRetired(lane, name)
//! ```

use super::commit_identity::fault_point;
use super::commit_ledger::{LedgerEntry, LedgerOwner};
use super::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, capture_lane_closure, select};
use std::io;
use std::path::{Path, PathBuf};

/// The person's explicit disposal of one saved item, by its name (the checksum). It has one constructor, and retirement never calls it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisposalAuthorization {
    name: String,
}

impl DisposalAuthorization {
    /// The authorization the person's disposal command makes for the saved item `name`.
    pub fn explicit(name: &str) -> DisposalAuthorization {
        return DisposalAuthorization { name: name.to_string() };
    }
}

/// An item whose replacement root is durable and whose file is still to be unlinked. It is bound to its lane and state directory and consumed
/// by the unlink, which checks it against the current root first.
#[derive(Debug, Eq, PartialEq)]
pub struct Retired {
    lane: PathBuf,
    state: PathBuf,
    name: String,
}

/// The ledger key of a saved item: the first characters of its name, as the discharge rule reads it.
fn key_of(name: &str) -> String {
    return name.chars().take(32).collect();
}

/// Whether the ledger already holds the `disposed` line for `key` (complete valid lines only).
fn has_disposed_line(ledger: &Path, key: &str) -> io::Result<bool> {
    let bytes: Vec<u8> = match std::fs::read(ledger) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let entries = super::commit_ledger::complete_entries(&bytes).ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the ledger has a torn or damaged line; repair it first"); })?;
    return Ok(entries.iter().any(|entry| { return entry.id == key && entry.outcome == "disposed"; }));
}

/// The newest root, parsed: its generation, the selected record and its checked document.
fn newest_checked_root(state: &Path) -> io::Result<(super::commit_state_log::Selected, LaneDocument)> {
    let selected = select(state).map_err(|error| { return io::Error::other(format!("the state log could not be read: {:?}", error)); })?.ok_or_else(|| { return io::Error::new(io::ErrorKind::NotFound, "no state log"); })?;
    let document: LaneDocument = LaneDocument::parse(&selected.payload).ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the newest root is not in the checked representation, so it cannot be replaced by a root that drops an item"); })?;
    return Ok((selected, document));
}

/// Writes the replacement root that no longer names `name`, with the ledger prefix that holds the `disposed` line in its closure.
fn replace_root_without(lane: &Path, state: &Path, name: &str) -> io::Result<()> {
    let (selected, document) = newest_checked_root(state)?;
    let closure: Vec<ClosureEntry> = capture_lane_closure(lane)?.into_iter().filter(|entry| { return entry.path().file_name().and_then(|leaf| { return leaf.to_str(); }) != Some(name); }).collect();
    let replacement: LaneDocument = LaneDocument::from_closure(&closure)
        .ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the lane holds an append-only file the checked representation cannot express"); })?
        .with_state(document.state().clone());
    let mut writer: StateWriter = StateWriter::acquire(state)?;
    let candidate = writer.begin_checkpoint_reclaiming_retiring(selected.generation + 1, &replacement, &closure, &[key_of(name)])?;
    return writer.complete_checkpoint(&candidate);
}

/// What:
///  Steps one and two of the disposal: the `disposed` line (once), then the replacement root. Returns the item whose file is still to be
///  unlinked.
/// Why:
///  A crash after this and before the unlink leaves the content on disk and a root that does not name it, which loses nothing. A crash
///  after the line alone leaves the line durable and the root naming the item; `resume_disposals` finishes it from the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retireSavedContent(lane: string, state: string, auth: DisposalAuthorization): Retired; // throws
/// ```
pub fn retire_saved_content(lane: &Path, state: &Path, authorization: &DisposalAuthorization) -> io::Result<Retired> {
    let (_selected, document) = newest_checked_root(state)?;
    if !document.saved().contains(&authorization.name) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the newest root does not hold that saved content"));
    }
    let key: String = key_of(&authorization.name);
    let ledger: PathBuf = lane.join(super::commit_state_log::LANE_LEDGER);
    if !has_disposed_line(&ledger, &key)? {
        let mut owner: LedgerOwner = LedgerOwner::acquire_in(lane, &ledger).map_err(|error| { return io::Error::other(format!("the ledger could not be owned: {:?}", error)); })?;
        owner
            .append(lane, &ledger, &LedgerEntry { id: key, outcome: "disposed".to_string(), commit: "-".to_string(), time: crate::iso_time::now_iso() })
            .map_err(|error| { return io::Error::other(format!("the disposed line could not be appended: {:?}", error)); })?;
    }
    fault_point("disposal:after-line");
    replace_root_without(lane, state, &authorization.name)?;
    fault_point("disposal:after-root");
    return Ok(Retired { lane: std::path::absolute(lane)?, state: std::path::absolute(state)?, name: authorization.name.clone() });
}

/// Step three: unlinks the retired item's file, and forces the directory so the removal is not undone by a power loss after the person
/// was told the content is gone. The token is consumed, and the current root is read first: a root that names the item again (a stale token)
/// refuses the unlink.
pub fn unlink_retired(retired: Retired) -> io::Result<()> {
    let (_selected, document) = newest_checked_root(&retired.state)?;
    if document.saved().contains(&retired.name) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the current root names this saved content again, so the retirement is stale and nothing is deleted"));
    }
    let directory: PathBuf = retired.lane.join("saved");
    match std::fs::remove_file(directory.join(&retired.name)) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fault_point("disposal:after-unlink");
    return super::commit_flusher::sync_path(&directory);
}

/// The whole disposal: the line, the replacement root, the unlink.
pub fn dispose_saved_content(lane: &Path, state: &Path, authorization: &DisposalAuthorization) -> io::Result<()> {
    let retired: Retired = retire_saved_content(lane, state, authorization)?;
    return unlink_retired(retired);
}

/// What:
///  Finishes every disposal the person authorized and a crash interrupted: for each file in `saved/` whose `disposed` line is in the
///  ledger, writes the replacement root if the newest root still names it, then unlinks the file. Returns how many files it unlinked.
/// Why:
///  The durable `disposed` line is the record of the person's authorization, so the replacement owner can finish the procedure without
///  asking again. Saved content with no such line is never touched, whatever the root says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resumeDisposals(lane: string, state: string): number;
/// ```
pub fn resume_disposals(lane: &Path, state: &Path) -> io::Result<usize> {
    let directory: PathBuf = lane.join("saved");
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    let ledger: PathBuf = lane.join(super::commit_state_log::LANE_LEDGER);
    let mut finished: usize = 0;
    for listed in entries {
        let name: String = listed?.file_name().to_string_lossy().into_owned();
        let is_content: bool = name.len() == 64 && super::commit_integrity::unhex(&name).is_some();
        if !is_content || !has_disposed_line(&ledger, &key_of(&name))? {
            continue;
        }
        let (_selected, document) = newest_checked_root(state)?;
        if document.saved().contains(&name) {
            replace_root_without(lane, state, &name)?;
        }
        unlink_retired(Retired { lane: std::path::absolute(lane)?, state: std::path::absolute(state)?, name })?;
        finished += 1;
    }
    return Ok(finished);
}

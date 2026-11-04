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

use super::commit_identity::{fault_point, record_event_lazy};
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

/// The complete entries of the ledger; a missing ledger is empty, and a torn or damaged line is an error.
fn ledger_entries(ledger: &Path) -> io::Result<Vec<LedgerEntry>> {
    let bytes: Vec<u8> = match super::commit_state_log::read_regular(ledger) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    return super::commit_ledger::complete_entries(&bytes).ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the ledger has a torn or damaged line; repair it first"); });
}

/// Whether a disposal of `key` is authorized and not yet ended: the last of its disposal lines is a `disposed` line.
fn disposal_pending(ledger: &Path, key: &str) -> io::Result<bool> {
    return Ok(super::commit_ledger::pending_disposals(&ledger_entries(ledger)?).contains(key));
}

/// Appends one disposal line for `key` as the ledger's owner.
fn append_disposal_line(lane: &Path, ledger: &Path, key: &str, outcome: &str) -> io::Result<()> {
    let mut owner: LedgerOwner = LedgerOwner::acquire_in(lane, ledger).map_err(|error| { return io::Error::other(format!("the ledger could not be owned: {:?}", error)); })?;
    return owner
        .append(lane, ledger, &LedgerEntry { id: key.to_string(), outcome: outcome.to_string(), commit: "-".to_string(), time: crate::iso_time::now_iso() })
        .map_err(|error| { return io::Error::other(format!("the {} line could not be appended: {:?}", outcome, error)); });
}

/// Ends the authorization of `key`: the content was removed, or it is retained again by a root written after the authorization. The line
/// is what makes content saved again afterwards a new incarnation. A failure to write it is logged: the unlink it follows has happened, and
/// the next recovery that finds the authorization still pending applies the same rule and writes it.
fn end_authorization(lane: &Path, key: &str) {
    let ledger: PathBuf = lane.join(super::commit_state_log::LANE_LEDGER);
    match disposal_pending(&ledger, key) {
        Ok(true) => {
            if let Err(error) = append_disposal_line(lane, &ledger, key, super::commit_ledger::OUTCOME_DISPOSAL_DONE) {
                crate::service_error!("the end of the disposal of {key} could not be recorded: {error}");
            }
        }
        Ok(false) => {}
        Err(error) => crate::service_error!("the ledger could not be read to end the disposal of {key}: {error}"),
    }
}

/// The newest root, parsed: its generation, the selected record and its checked document.
fn newest_checked_root(state: &Path) -> io::Result<(super::commit_state_log::Selected, LaneDocument)> {
    let selected = select(state).map_err(|error| { return io::Error::other(format!("the state log could not be read: {:?}", error)); })?.ok_or_else(|| { return io::Error::new(io::ErrorKind::NotFound, "no state log"); })?;
    let document: LaneDocument = LaneDocument::parse(&selected.payload).ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the newest root is not in the checked representation, so it cannot be replaced by a root that drops an item"); })?;
    return Ok((selected, document));
}

/// Writes the replacement root: the lane as it is now captured (which never includes saved content whose `disposed` line exists), with the
/// state the newest root carried, retiring every saved item the previous root named and the new one does not. `writer` is the state writer
/// the caller holds, so no seal and no other disposal captures or publishes in between.
fn replace_root_without_disposed(lane: &Path, writer: &mut StateWriter, state: &Path) -> io::Result<()> {
    let (selected, document) = newest_checked_root(state)?;
    let closure: Vec<ClosureEntry> = capture_lane_closure(lane)?;
    let replacement: LaneDocument = LaneDocument::from_closure(&closure)
        .ok_or_else(|| { return io::Error::new(io::ErrorKind::InvalidData, "the lane holds an append-only file the checked representation cannot express"); })?
        .with_state(document.state().clone());
    // Several disposals can be pending at once (their lines durable, their replacements not): every omitted item is named, or the non-drop
    // check would refuse the one whose name was left out.
    // The previous root holds an item by its document or by its retained set alone (the public append admits a set that over-approximates the
    // document), and the non-drop check needs the name of each item the replacement leaves out, whichever way it was held.
    let held_by_closure = selected.items.iter().filter_map(|entry| { return entry.path().strip_prefix("saved").ok().and_then(|name| { return name.to_str(); }).map(str::to_string); });
    let retired: Vec<String> = document
        .saved()
        .iter()
        .cloned()
        .chain(held_by_closure)
        .filter(|held| { return !replacement.saved().contains(held); })
        .map(|held| { return key_of(&held); })
        .collect::<std::collections::BTreeSet<String>>()
        .into_iter()
        .collect();
    let candidate = writer.begin_checkpoint_reclaiming_retiring(selected.generation + 1, &replacement, &closure, &retired)?;
    fault_point("disposal:after-candidate");
    return writer.complete_checkpoint(&candidate);
}

/// What:
///  Steps one and two of the disposal: the `disposed` line (once), then the replacement root. Returns the item whose file is still to be
///  unlinked.
/// Why:
///  The state writer is taken first and kept to the end of the replacement, so a seal cannot capture between the line and the root (a seal
///  that began before the line would publish a closure that still names the item) and refuses while this runs. A crash after this and before
///  the unlink leaves the content on disk and a root that does not name it, which loses nothing. A crash after the line alone leaves the line
///  durable and the root naming the item; `resume_disposals` finishes it from the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retireSavedContent(lane: string, state: string, auth: DisposalAuthorization): Retired; // throws
/// ```
pub fn retire_saved_content(lane: &Path, state: &Path, authorization: &DisposalAuthorization) -> io::Result<Retired> {
    let mut writer: StateWriter = StateWriter::acquire(state)?;
    // Before any line is written: a saved directory that is a link is not the lane's, and nothing in it is the person's to dispose of here.
    super::commit_state_log::open_lane_subdirectory(lane, "saved")?;
    let (selected, document) = newest_checked_root(state)?;
    // Held by the document or by the retained set alone, as recovery and the unlink read it.
    let held_by_closure: bool = selected.items.iter().any(|entry| { return entry.path() == Path::new("saved").join(&authorization.name); });
    if !document.saved().contains(&authorization.name) && !held_by_closure {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the newest root does not hold that saved content"));
    }
    let key: String = key_of(&authorization.name);
    let ledger: PathBuf = lane.join(super::commit_state_log::LANE_LEDGER);
    if !disposal_pending(&ledger, &key)? {
        append_disposal_line(lane, &ledger, &key, super::commit_ledger::OUTCOME_DISPOSED)?;
    }
    fault_point("disposal:after-line");
    replace_root_without_disposed(lane, &mut writer, state)?;
    fault_point("disposal:after-root");
    return Ok(Retired { lane: std::path::absolute(lane)?, state: std::path::absolute(state)?, name: authorization.name.clone() });
}

/// Step three: unlinks the retired item's file, and forces the directory so the removal is not undone by a power loss after the person
/// was told the content is gone. The state writer is held from the revalidation to the end: the current root is read (a root that names the
/// item again refuses the unlink), the root that recovery would select and everything it retains are forced (a visible root that omits the
/// item is not yet a durable cover; a candidate a crash left uncompleted is such a root), and only then the file goes. The token is consumed.
pub fn unlink_retired(retired: Retired) -> io::Result<()> {
    let _writer: StateWriter = StateWriter::acquire(&retired.state)?;
    let (selected, document) = newest_checked_root(&retired.state)?;
    // The document's references are not the whole retained set: the public append admits a set that over-approximates them, and recovery's
    // eligibility depends on every retained entry. A root that retains the item in its closure alone would be made ineligible by the unlink.
    let retained_in_closure: bool = selected.items.iter().any(|entry| { return entry.path() == Path::new("saved").join(&retired.name); });
    if document.saved().contains(&retired.name) || retained_in_closure {
        // The retention is newer than the authorization: the authorization lapses with the stale token.
        end_authorization(&retired.lane, &key_of(&retired.name));
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the current root retains this saved content again, so the retirement is stale and nothing is deleted"));
    }
    if !disposal_pending(&retired.lane.join(super::commit_state_log::LANE_LEDGER), &key_of(&retired.name))? {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no disposal of this saved content is authorized any more, so nothing is deleted"));
    }
    let prefix = super::commit_state_log::sync_prefix(&retired.state)?;
    super::commit_state_log::force_items(&retired.lane, &prefix.items)?;
    let directory: PathBuf = retired.lane.join("saved");
    let path: PathBuf = directory.join(&retired.name);
    // The directory is opened here without following a link and the name is removed relative to that descriptor: a `saved` that became a
    // link since the checks removes nothing outside the lane.
    let Some(opened) = super::commit_state_log::open_lane_subdirectory(&retired.lane, "saved")? else { return Ok(()) };
    fault_point("disposal:before-unlink");
    if remove_in(&opened, &retired.name)? {
        record_event_lazy(|| { return format!("unlink:{}", path.display()); });
    }
    fault_point("disposal:after-unlink");
    opened.sync_all()?;
    end_authorization(&retired.lane, &key_of(&retired.name));
    return Ok(());
}

/// Removes the entry `name` of the opened directory `directory`: `true` when it removed one, `false` when the entry was gone already.
fn remove_in(directory: &std::fs::File, name: &str) -> io::Result<bool> {
    use std::os::fd::AsRawFd;
    let entry: std::ffi::CString = std::ffi::CString::new(name).map_err(|error| { return io::Error::new(io::ErrorKind::InvalidInput, error); })?;
    // SAFETY: `unlinkat` takes a descriptor, a NUL-terminated name that lives through the call, and flags; it touches no other memory.
    if unsafe { libc::unlinkat(directory.as_raw_fd(), entry.as_ptr(), 0) } == 0 {
        return Ok(true);
    }
    let error: io::Error = io::Error::last_os_error();
    return if error.kind() == io::ErrorKind::NotFound { Ok(false) } else { Err(error) };
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
    let Some(opened) = super::commit_state_log::open_lane_subdirectory(lane, "saved")? else { return Ok(0) };
    let listing = std::fs::read_dir(super::commit_state_log::listing_path(&opened))?;
    let ledger: PathBuf = lane.join(super::commit_state_log::LANE_LEDGER);
    let entries: Vec<LedgerEntry> = ledger_entries(&ledger)?;
    let authorized = super::commit_ledger::pending_disposals(&entries);
    let mut pending: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed?.file_name().to_string_lossy().into_owned();
        let is_content: bool = name.len() == 64 && super::commit_integrity::unhex(&name).is_some();
        if is_content && authorized.contains(&key_of(&name)) {
            pending.push(name);
        }
    }
    if pending.is_empty() {
        return Ok(0);
    }
    // One replacement root omits every pending item at once; a writer that is busy means another owner is at it, and the next sweep retries.
    match StateWriter::acquire(state) {
        Ok(mut writer) => {
            let (selected, document) = newest_checked_root(state)?;
            let retained = |name: &String| -> bool { return document.saved().contains(name) || selected.items.iter().any(|entry| { return entry.path() == Path::new("saved").join(name); }); };
            // A root that retains the item and was written after the authorization (its ledger prefix holds every `disposed` line of the key)
            // is a retention the person did not authorize the disposal of: the authorization lapses and the item stays.
            let held_ledger = super::commit_state_log::retained_ledger_entries(lane, &selected.items);
            let renewed = |name: &String| -> bool {
                let key: String = key_of(name);
                return retained(name) && held_ledger.as_ref().is_some_and(|held| { return super::commit_ledger::disposed_lines(held, &key) == super::commit_ledger::disposed_lines(&entries, &key); });
            };
            let lapsed: Vec<String> = pending.iter().filter(|name| { return renewed(name); }).cloned().collect();
            for name in &lapsed {
                end_authorization(lane, &key_of(name));
            }
            pending.retain(|name| { return !lapsed.contains(name); });
            if pending.iter().any(retained) {
                replace_root_without_disposed(lane, &mut writer, state)?;
            }
        }
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(0),
        Err(error) => return Err(error),
    }
    let mut finished: usize = 0;
    for name in pending {
        match unlink_retired(Retired { lane: std::path::absolute(lane)?, state: std::path::absolute(state)?, name }) {
            Ok(()) => finished += 1,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error),
        }
    }
    return Ok(finished);
}

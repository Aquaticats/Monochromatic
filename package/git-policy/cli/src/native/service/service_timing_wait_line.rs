//! What:
//!  The words and figures of the line about the wait for the index lock:
//!  what the wait found,
//!  when the line is worth writing at all,
//!  and the read of a step's duration from a line already in the journal.
//! Why:
//!   A landing that waits for another program's hold on the index is the slow commit people report,
//!   so the wait is written when it ends rather than with the answer,
//!   and a landing that fails afterwards still leaves what it waited for.
//!  The figures are split into the acquisition's own work,
//!  because a wait whose whole is far longer than their sum spent its time outside it
//!  (the gate of 2026-11-11T10:29-05:00:
//!  one landing's wait took 95,277 ms while these parts accounted for 3,003 ms of it).
//!  The key of the index step was once `sync_ms`,
//!  so a reader of the journal asks for the new key and falls back to the old one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! waitLine(seq, repository, total, notes, taken) // 'waited for the index lock batch=7 ...' | undefined
//! ```
use crate::service_index_found::WaitNotes;
use std::path::Path;
use std::time::Duration;

/// The words for what the wait for the lock found,
/// empty when the lock was free at the first placement:
/// the placements that found it held,
/// the time spent readying the acquisition (`lock_setup_ms`),
/// flushing the marker (`lock_stage_ms`),
/// waiting for a holder (`lock_place_ms`)
/// and removing the staged file once the wait was decided (`lock_after_ms`),
/// and the first and the last description of the held lock.
/// Why the four figures are split:
///  a wait whose whole is far longer than their sum has spent its time outside the acquisition's own work,
///  which the line must let a reader see
/// (the gate of 2026-11-11T10:29-05:00:
/// one landing's wait took 95,277 ms while these parts accounted for 3,003 ms of it).
pub fn lock_notes_fields(notes: &WaitNotes) -> String {
    if notes.looks == 0 {
        return format!(
            " lock_stage_ms={} lock_stage_kind={} lock_place_ms={} lock_setup_ms={} lock_after_ms={}",
            notes.staging.as_millis(),
            notes.stage_kind,
            notes.placing.as_millis(),
            notes.setup.as_millis(),
            notes.after.as_millis()
        );
    }
    let first: &str = notes.found.as_deref().unwrap_or("none");
    let last: &str = notes.last.as_deref().unwrap_or("none");
    return format!(
        " lock_stage_ms={} lock_stage_kind={} lock_place_ms={} lock_setup_ms={} lock_after_ms={} lock_looks={} lock_found={first:?} lock_last={last:?}",
        notes.staging.as_millis(),
        notes.stage_kind,
        notes.placing.as_millis(),
        notes.setup.as_millis(),
        notes.after.as_millis(),
        notes.looks
    );
}

/// The wait for the index lock that is written down even when no holder was met:
/// the making of the marker or a wait that long is a slow landing's first suspect.
///  Exempt from a citation under `LQM` (class:
/// notice pacing):
/// it times a line the person reads or a log threshold,
/// and
///  nothing waits on it beyond the wait that already exists.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const SLOW_LOCK_STEP: Duration = Duration::from_secs(1);

/// What:
///  The line for the end of a wait for the index lock,
/// or nothing when the lock was free at the first placement and the acquisition was quick.
/// Why:
///   Written when the wait ends,
/// not with the answer,
/// so a landing that fails afterwards still leaves what it waited for.
/// It names the repository the batch is in,
/// the total,
/// how it split between readying the acquisition,
/// flushing the marker,
/// waiting for a holder and removing the staged file,
/// how many times the lock was found held,
/// and what the first and the last look found (`lock_found`,
/// `lock_last`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitLine(seq, repository, total, notes, taken) // 'waited batch=7 repository=/repo result=taken ...' | undefined
/// ```
pub fn wait_line(
    seq: u64,
    repository: &Path,
    total: Duration,
    notes: &WaitNotes,
    taken: bool,
) -> Option<String> {
    if notes.looks == 0 && notes.staging < SLOW_LOCK_STEP && total < SLOW_LOCK_STEP {
        return None;
    }
    return Some(format!(
        "waited for the index lock batch={seq} repository={repository:?} result={} lock_wait_ms={}{}",
        if taken { "taken" } else { "refused" },
        total.as_millis(),
        lock_notes_fields(notes)
    ));
}

/// The key under which `answered` lines written before the index step was named by its role state its duration.
const LEGACY_INDEX_UPDATE_KEY: &str = "sync_ms";

/// What:
///  The duration,
/// in whole milliseconds,
/// that an `answered` line gives for removing the conclusion state and building and
///  installing the new index.
/// Why:
///   The step was first logged as `sync_ms`,
/// which reads as a disk sync although it is the follow-up of the landing that holds
///  the index lock;
/// it is `index_update_ms` now.
/// Lines already in the journal keep the old key,
/// so a reader asks for the new
///  key and falls back to the old one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(/ index_update_ms=(\d+)/.exec(line)?.[1] ?? / sync_ms=(\d+)/.exec(line)?.[1])
/// ```
pub fn index_update_ms_of(line: &str) -> Option<u64> {
    if let Some(found) = digits_after_key(line, "index_update_ms") {
        return Some(found);
    }
    return digits_after_key(line, LEGACY_INDEX_UPDATE_KEY);
}

/// What:
///  The number that follows ` key=` in a log line,
/// nothing when the key is absent or no digits follow.
/// Why:
///  `index_update_ms_of` reads the current key first and the legacy key after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(new RegExp(` ${key}=(\\d+)`).exec(line)?.[1])
/// ```
fn digits_after_key(
    line: &str,
    key: &str,
) -> Option<u64> {
    let word: String = format!(" {key}=");
    let from: usize = line.find(word.as_str())? + word.len();
    let digits: String = line[from..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    return digits.parse::<u64>().ok();
}

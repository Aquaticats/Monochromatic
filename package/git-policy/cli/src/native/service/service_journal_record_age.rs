//! What:
//!  How old the record of a batch that landed is,
//! judged on the reading of the clocks the record itself carries,
//! and on its modification time only where it carries none:
//! the reading a finished record was written by,
//! and whether a client of it may still read it.
//! Why:
//!  A finished record is the only place a request's identity and the commit it became are written together,
//! so a client whose answer was lost reads it (`service_commit_looking_for_what_landed.rs`),
//! and the cleanup of a removed worktree's journal keeps it for `A_FINISHED_RECORD_IS_READ`
//! (`service_files_of_a_stopped_instance.rs`).
//! That age was read from the file's modification time,
//! which is the wall clock,
//! and the wall clock steps:
//! a time sync,
//! a person,
//! or a machine that resumed from sleep moved it forward by more than the window,
//! and the record of a batch that landed a moment ago then looked older than every client of it
//! while those clients were inside a wait measured on the monotonic clock.
//! The cleanup then removed the only record that tells such a client which commit its request became,
//! and the command reported a commit that landed as one whose result is not known.
//! A record therefore carries the reading of the clocks it was written by:
//! the boot and the machine it belongs to,
//! the monotonic clock,
//! and the wall clock,
//! the same reading a claim's lease is judged by (`claim_time.rs`).
//! Two readings of one boot are compared on the monotonic clock,
//! which never steps and does not count a suspended host's sleep;
//! a reading of an earlier boot of the same machine is past at once,
//! because a restart ended every client of that boot;
//! and the wall clock judges only what neither of those settles,
//! which is a record an earlier release wrote and one written before a host had a boot ID.
//! The reading is taken when the batch's intent is written,
//! which is before its record is published and after the client sent its request,
//! so the age this judges is never shorter than the wait the client is inside.
//! Nothing here removes anything:
//! it answers how long ago a record was written and whether a client may still read it,
//! from a reading of the clocks and from the record's own file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stampLine = (now) => `stamp=${now.encode()}\n`;
//! const stampOf = (text) => decode(lineValue(text, 'stamp'));
//! const aClientMayStillRead = (stamp, modified, now) =>
//!   stamp ? !now.isPastTheBootOf(stamp) && now.elapsedSince(stamp) < A_FINISHED_RECORD_IS_READ : withinTheTimeAClientLooks(modified);
//! const aClientMayStillReadFinishedBatch = (path, now) =>
//!   aClientMayStillRead(undefined, modifiedTimeOf(path), now) || aClientMayStillRead(stampOf(readText(path)), modifiedTimeOf(path), now);
//! ```

use crate::claim_time::ClaimTime;
use crate::service_journal_intent_text_and_parse::intent_value;
use crate::wait_ceilings::A_FINISHED_RECORD_IS_READ;
use std::path::Path;
use std::time::SystemTime;

/// The key of the line a record carries the reading of the clocks it was written by under.
pub const STAMP_KEY: &str = "stamp";

/// What:
///  The line a record carries the reading of the clocks it was written by.
/// Why:
///   The reading is written with the record,
/// so it survives a restart and a copied journal,
/// and a reader that does not know the key skips the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = stampLine(now); // 'stamp=boot~mono~wall~machine\n'
/// ```
pub fn stamp_line(now: &ClaimTime) -> String {
    return format!("{STAMP_KEY}={}\n", now.encode());
}

/// What:
///  The reading of the clocks the text of a record carries,
///  or nothing for a record an earlier release wrote,
///  for one whose line cannot be read,
///  and for text that is no record at all.
/// Why:
///   A record with no readable reading is judged by its modification time,
/// which is what every release before this one did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stamp = stampOf(text); // ClaimTime or undefined
/// ```
pub fn stamp_of(text: &str) -> Option<ClaimTime> {
    let stamped: &str = intent_value(text, STAMP_KEY)?;
    return ClaimTime::decode(stamped);
}

/// What:
///  Whether a client of a record whose age is unproven may still read it:
///  yes while the modification time is within `A_FINISHED_RECORD_IS_READ` of now,
///  for a reading of the wall clock that cannot be read counts as recent,
///  because no age is proven and an unproven age never discards.
/// Why:
///   This is the answer for a record with no reading of its own,
/// and for the folder that holds the records,
/// whose own modification time says when a record was last published into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const within = (modified) => modified.ok && now - modified < A_FINISHED_RECORD_IS_READ;
/// ```
fn is_within_the_time_a_client_looks(modified: std::io::Result<SystemTime>) -> bool {
    let Ok(written) = modified else { return true };
    return match written.elapsed() {
        Ok(age) => age < A_FINISHED_RECORD_IS_READ,
        // A time after now is a clock that moved, so no age is proven either.
        Err(_in_the_future) => true,
    };
}

/// What:
///  Whether a client of a finished record may still read it,
///  given the reading of the clocks it carries,
///  its modification time,
///  and a reading of the clocks now.
/// Why:
///   A record written in this boot is aged on the monotonic clock,
/// so a wall clock that stepped forward or back changes nothing about the wait a client is inside;
/// a record of an earlier boot of the same machine was written before a restart ended every client of it.
/// Only a record with no reading,
/// or one whose boot cannot be compared,
/// is left to the wall clock,
/// which is the answer this release gave before records carried a reading.
/// Nothing removes a record here,
/// and a record whose age is not proven is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aClientMayStillRead(stamp, modified, now) // true: keep the record for a client that may still look
/// ```
pub fn a_client_may_still_read(stamp: Option<&ClaimTime>, modified: std::io::Result<SystemTime>, now: &ClaimTime) -> bool {
    let Some(written) = stamp else { return is_within_the_time_a_client_looks(modified) };
    // A restart ended every client of the boot the record was written in, so no client of it is left to read it.
    if now.is_past_the_boot_of(written) {
        return false;
    }
    return now.elapsed_since(written) < A_FINISHED_RECORD_IS_READ;
}

/// What:
///  The time a path was last written,
///  and the operating system's reason when it cannot say.
/// Why:
///   A journal's folder and the records in it are judged by their own times where no reading of the clocks is kept,
///  and an unreadable time proves no age.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modified = (path) => statSync(path).mtimeMs;
/// ```
pub fn modified_time_of(path: &Path) -> std::io::Result<SystemTime> {
    let metadata = std::fs::metadata(path)?;
    return metadata.modified();
}

/// The reading of the clocks the finished record at `path` carries,
/// from the record's own text,
/// and nothing when it carries none or cannot be read.
fn reading_of_a_finished_record(path: &Path) -> Option<ClaimTime> {
    let text: String = std::fs::read_to_string(path).ok()?;
    return stamp_of(text.as_str());
}

/// What:
///  Whether a client of the finished record at `path` may still read it.
/// Why:
///   A record whose modification time already says a client may still look costs no read,
/// and one the wall clock calls old is asked about through the reading it carries,
/// because a wall clock that stepped forward makes a record of this boot look old
/// while the client of it is still inside a wait measured on the monotonic clock.
/// The reading is only ever one more reason to keep a record:
/// no client can look for a commit past `A_FINISHED_RECORD_IS_READ` of the clock it waits on,
/// but a record the wall clock still keeps is kept whether its reading is old or not,
/// so a clock that moved costs a journal that outlives its clients and never a record taken too soon.
/// A record whose text cannot be read is therefore kept while its modification time is young and removed once that is old,
/// which is the only answer its age proves;
/// the sweep of a removed worktree's journal asks the same question and keeps the whole journal on a record it cannot read
/// (`service_files_of_a_stopped_instance.rs`),
/// because there it guards work a person may want rather than spending one record's memory.
/// Two questions are asked of the modification time, so a record past its word is read and stated once per answer;
/// nothing here removes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aClientMayStillRead(finishedRecordReading(path), modifiedTimeOf(path), now)
/// ```
pub fn a_client_may_still_read_finished_batch(path: &Path, now: &ClaimTime) -> bool {
    if a_client_may_still_read(None, modified_time_of(path), now) {
        return true;
    }
    return a_client_may_still_read(reading_of_a_finished_record(path).as_ref(), modified_time_of(path), now);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_record_age_tests.rs"]
mod tests;

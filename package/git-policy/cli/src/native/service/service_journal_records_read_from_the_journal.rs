//! What:
//!  What the records in the service's journal say,
//!  and which of them a journal may forget:
//! the index a batch started from,
//! whether its index update is installed,
//! the batches listed in order,
//! the finished records removed by their age and count,
//! and the request identities every finished batch remembers.
//! Why:
//!  A restart and a client whose answer was lost both read the journal's files (`service_journal.rs`),
//! and a bad read must never be taken for a missing record:
//! an index update that cannot be read is not replayed,
//! and a record that cannot be parsed keeps the journal it is in.
//!  The removal of finished records sits with the reading that judges them,
//! because only a record a client of it can no longer read may go.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = readStart(dir, 7); // 'recorded' | 'absent' | 'unreadable'
//! const removed = pruneDone(dir, DONE_RETAINED, now); // the count it removed
//! ```

use super::service_journal_file_names::{
    CLAIM_EXTENSION,
    DONE_EXTENSION,
    INDEXED_EXTENSION,
    START_EXTENSION,
    TEMPORARY_EXTENSION,
    batch_path,
};
use crate::claim_time::ClaimTime;
use crate::service_journal_intent_text_and_parse::{
    Intent,
    parse_intent,
};
use crate::service_journal_record_age::a_client_may_still_read_finished_batch;
use crate::service_land_start::{
    IndexWitness,
    decode,
};
use std::collections::HashMap;
use std::fs::{
    self,
};
use std::path::{
    Path,
    PathBuf,
};

/// How old a temporary file must be before a listing treats it as the remains of a killed write and removes it.
/// A write takes
///  milliseconds;
/// the grace covers a writer stalled on a slow disk or a loaded host.
/// Measured:
/// the longest journal write step of
///  1,302 landings in the service journal (`journal_ms`,
/// 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z) was 40,188 ms,
/// the 99th
///  percentile 3,792 ms,
/// so ten minutes is fifteen times the longest.
/// Kept as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const TEMPORARY_GRACE: std::time::Duration = std::time::Duration::from_secs(10 * 60);

/// How many finished batches a journal keeps whatever their age.
/// What:
///  The count a finished record must be beyond before its age can remove it,
///  and the number of records a repository that lands rarely keeps for ever.
/// Why:
///   A request is answered as already landed only by the record its batch left,
/// so the record must outlive the client that may send the request again.
/// That client's whole life is `A_FINISHED_RECORD_IS_READ` (`wait_ceilings.rs`),
/// which decides most removals;
/// this count decides the others:
///  a client that was stalled for hours
/// (a host suspended under it,
/// a command held at a debugger,
/// an agent's turn that resumed the next day)
/// outlives any age a service may state,
/// and its record stays while fewer than this many batches have landed since it.
/// It is also what a quiet repository keeps without end,
/// which bounds the state folder and the read a service makes at its start.
/// Measured:
/// in the service journal (1,085 landings of the `cli-git@` units,
/// 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z) the busiest
/// unit landed 4 batches in its busiest 10 seconds,
/// 4 in its busiest minute and 15 in its busiest ten minutes,
/// and 349 in all,
/// so this many batches are a busy hour of that unit's traffic.
/// Kept at the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Journal retention and capture memory").
pub const DONE_RETAINED: usize = 1024;

/// What a batch's start record says.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Started {
    /// No usable record:
    /// an older batch,
    /// a landing that could not write one,
    /// or a record that is damaged.
    /// No replay is made.
    Absent,
    /// The index file the landing started from.
    Recorded(IndexWitness),
    /// A record exists and cannot be read now,
    /// so nothing is decided.
    Unreadable,
}

/// The start record of the batch `seq`;
/// a missing file or damaged text is no record,
/// and a file that cannot be read now is
///  unreadable.
pub fn read_start(
    dir: &Path,
    seq: u64,
) -> Started {
    return match fs::read_to_string(batch_path(dir, seq, START_EXTENSION)) {
        Ok(text) => decode(text.trim_end_matches('\n')).map_or(Started::Absent, Started::Recorded),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Started::Absent,
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Started::Absent,
        Err(_) => Started::Unreadable,
    };
}

/// Drops the start record of a batch that is finished or forgotten.
pub fn remove_start(
    dir: &Path,
    seq: u64,
) {
    let _removed = fs::remove_file(batch_path(dir, seq, START_EXTENSION));
}

/// What a batch's completion record says about its index update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Indexed {
    /// No record:
    /// the update was not recorded as installed.
    Absent,
    /// The record names the commit:
    /// the update was installed.
    Installed,
    /// A record exists and cannot be read,
    /// or names another commit:
    /// nothing can be said about the index.
    Unreadable,
}

/// What:
///  What the completion record of the open batch `seq` that landed `new` says.
/// Why:
///   Only a missing record means the update was not installed.
/// A record that is there and cannot be read says nothing,
/// and a
///  restart that took it for none would replay the update over whatever was staged since,
/// so the caller stops on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await indexedState(dir, seq, newCommit) // 'absent' | 'installed' | 'unreadable'
/// ```
pub fn indexed_state(
    dir: &Path,
    seq: u64,
    new: &str,
) -> Indexed {
    return match fs::read_to_string(batch_path(dir, seq, INDEXED_EXTENSION)) {
        Ok(text) if text.trim_end() == new => Indexed::Installed,
        Ok(_) => Indexed::Unreadable,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Indexed::Absent,
        Err(_) => Indexed::Unreadable,
    };
}

/// What:
///  Whether a leftover temporary or claim file is old enough to be the remains of a write that was killed.
/// Why:
///  `list_intents` removes only such files;
/// a younger one may be another instance's write in progress.
/// A file whose age cannot be
///  read is not abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.now() - statSync(path).mtimeMs > TEMPORARY_GRACE_MS
/// ```
fn is_abandoned(entry: &fs::DirEntry) -> bool {
    let Ok(meta) = entry.metadata() else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    return match modified.elapsed() {
        Ok(age) => age > TEMPORARY_GRACE,
        Err(_in_the_future) => false,
    };
}

/// What:
///  The number a listed intent sorts by:
/// its batch number,
/// or the length of its path when it could not be read.
/// Why:
///  `list_intents` sorts so the batches come in their order and the damaged ones sort apart;
/// the sort is stable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// found.sort((a, b) => key(a) - key(b))
/// ```
fn sort_key_of_found(listed: &(PathBuf, Option<Intent>)) -> u64 {
    let (path, intent) = listed;
    return match intent {
        Some(parsed) => parsed.seq,
        None => path.to_string_lossy().len() as u64,
    };
}

/// What:
///  The batch number and path of a directory entry that is a finished batch file,
/// nothing for any other entry.
/// Why:
///  `prune_done` touches only files named as a batch number with the finished extension.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^\d+\.done$/.test(name) && isFile(entry) ? [Number(stem), path] : undefined
/// ```
fn finished_batch_of_entry(entry: fs::DirEntry) -> Option<(u64, PathBuf)> {
    let path: PathBuf = entry.path();
    if path.extension()? != DONE_EXTENSION || !entry.file_type().ok()?.is_file() {
        return None;
    }
    let seq: u64 = path.file_stem()?.to_str()?.parse().ok()?;
    return Some((seq, path));
}

/// What:
///  The sort key that puts the newest finished batch first.
/// Why:
///  `prune_done` keeps the newest batches and removes the rest,
/// so it lists them newest first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finished.sort((a, b) => b[0] - a[0])
/// ```
fn newest_first_key(finished: &(u64, PathBuf)) -> std::cmp::Reverse<u64> {
    return std::cmp::Reverse(finished.0);
}

/// What:
///  The intents in the journal with an extension,
/// in batch order,
/// and the abandoned temporary files removed on the way.
/// Why:
///   Recovery reads the open ones;
/// startup reads the done ones for the request identities.
/// A temporary file younger than the
///  grace may be another instance's write in progress,
/// so it stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listIntents(dir, 'intent')
/// ```
pub fn list_intents(
    dir: &Path,
    extension: &str,
) -> Vec<(PathBuf, Option<Intent>)> {
    return list_intents_removing_abandoned(dir, extension, true);
}

/// What:
///  `list_intents` that removes nothing from the journal.
/// Why:
///   A journal another service may still be writing (an earlier helper's) is read,
/// never tidied:
/// its temporary and claim files
///  are that service's,
/// however old they look,
/// and removing one fails its rename or hands its batch number to a second landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listIntentsReadOnly(dir, 'intent')
/// ```
pub fn list_intents_read_only(
    dir: &Path,
    extension: &str,
) -> Vec<(PathBuf, Option<Intent>)> {
    return list_intents_removing_abandoned(dir, extension, false);
}

/// `list_intents`,
/// with the removal of abandoned temporary and claim files chosen by the caller.
fn list_intents_removing_abandoned(
    dir: &Path,
    extension: &str,
    remove_abandoned: bool,
) -> Vec<(PathBuf, Option<Intent>)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, Option<Intent>)> = Vec::new();
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some(TEMPORARY_EXTENSION | CLAIM_EXTENSION) => {
                // Only the remains of a write or a landing that was killed: a younger file may be another instance's write in
                //  progress, and removing it would fail that instance's rename or hand its number to a second landing.
                let abandoned: bool = remove_abandoned && is_abandoned(&entry);
                if abandoned {
                    let _removed = fs::remove_file(&path);
                }
            }
            Some(found_extension) if found_extension == extension => {
                // A file that cannot be read or parsed is listed with no intent.
                let parsed: Option<Intent> = match fs::read_to_string(&path) {
                    Ok(text) => parse_intent(text.as_str()),
                    Err(_unreadable) => None,
                };
                found.push((path, parsed));
            }
            _ => {}
        }
    }
    found.sort_by_key(sort_key_of_found);
    return found;
}

/// What:
///  Removes the files of the finished batches that no client of them can still read,
///  keeping the newest `retained` whatever their age,
///  and gives how many it removed.
/// Why:
///   A finished batch's file is only memory of the request identities it landed.
/// Nothing else removed them,
/// so a journal grew by
///  a file per batch for as long as the worktree lived.
/// Batch numbers increase with time,
/// so the newest are the largest,
/// and this rule is load-bearing here:
/// a batch number is taken as the first free one above every number the journal holds,
/// and so is never taken again while its record stays.
///  A record is removed only when both hold:
/// it is beyond the newest `retained`,
/// and no client of it may still read it (`A_FINISHED_RECORD_IS_READ` of the clock that client waits on,
/// `service_journal_record_age.rs`).
/// A burst of landings pushes a record past the count while its client is still inside its wait,
/// and a repository that lands rarely keeps an old record past the age,
/// so neither takes the memory of a request a client may still send again.
/// A removal that fails leaves its record.
///  Only files named as a batch number with the finished extension are touched,
/// and this runs at a service's start,
/// never on the path a commit waits on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pruneDone(dir, DONE_RETAINED, now) // 3: removed the three oldest no client can still read
/// ```
pub fn prune_done(
    dir: &Path,
    retained: usize,
    now: &ClaimTime,
) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut finished: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter_map(finished_batch_of_entry)
        .collect();
    finished.sort_by_key(newest_first_key);
    // Each record past the count is asked about once, and only the removals that worked are counted.
    let mut removed: usize = 0;
    for (_, path) in finished.into_iter().skip(retained) {
        if a_client_may_still_read_finished_batch(path.as_path(), now) {
            continue;
        }
        if fs::remove_file(path).is_ok() {
            removed += 1;
        }
    }
    return removed;
}

/// What:
///  The records of the newest finished batches in the journal,
/// newest first,
/// read without removing anything.
/// Why:
///   A client whose answer was lost looks for the commit its own request made,
/// and that batch is one of the
///  newest:
/// the request was sent seconds ago.
/// Reading only those keeps a look that repeats until its bound cheap,
///  where reading the journal's
/// `DONE_RETAINED` records would re-read every one of them at each look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// newestFinishedBatchesReadOnly(dir, 16)
/// ```
pub fn newest_finished_batches_read_only(
    dir: &Path,
    newest: usize,
) -> Vec<(PathBuf, Option<Intent>)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut finished: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter_map(finished_batch_of_entry)
        .collect();
    finished.sort_by_key(newest_first_key);
    let mut found: Vec<(PathBuf, Option<Intent>)> = Vec::new();
    for (_seq, path) in finished.into_iter().take(newest) {
        let parsed: Option<Intent> = finished_record_of(path.as_path());
        found.push((path, parsed));
    }
    return found;
}

/// What:
///  The record a finished batch's file holds,
/// nothing when the file cannot be read or does not parse.
/// Why:
///   A record that cannot be read or parsed is listed with no intent,
/// as the other listings do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finishedRecordOf(path) // the record, or undefined
/// ```
fn finished_record_of(path: &Path) -> Option<Intent> {
    let text: String = fs::read_to_string(path).ok()?;
    return parse_intent(text.as_str());
}

/// The request identities of every finished batch,
/// each with the commit it became.
pub fn landed_commits(dir: &Path) -> HashMap<String, String> {
    return landed_in(list_intents(dir, DONE_EXTENSION));
}

/// The request identities of every finished batch of a journal another service may still write,
/// read without removing anything.
pub fn landed_commits_read_only(dir: &Path) -> HashMap<String, String> {
    return landed_in(list_intents_read_only(dir, DONE_EXTENSION));
}

/// The identities of the finished batches listed,
/// each with the commit it became.
fn landed_in(listed: Vec<(PathBuf, Option<Intent>)>) -> HashMap<String, String> {
    let mut landed: HashMap<String, String> = HashMap::new();
    for (_, intent) in listed {
        if let Some(parsed) = intent {
            landed.extend(parsed.ids.into_iter().zip(parsed.commits));
        }
    }
    return landed;
}

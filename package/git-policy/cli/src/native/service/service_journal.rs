//! What:
//!  The resident service's journal:
//! one file per batch of commits,
//! with the one durable write the batch needs,
//! and the
//!  decision a restart makes about a batch it finds open.
//! Why:
//!  The branch moves only after the intent is on disk,
//! so after any kill the journal and the branch say whether the
//!  batch landed;
//! a restart finishes the one step that may be missing (the index update) or drops the intent,
//! and a
//!  request that landed is remembered by its identity so a retry is answered,
//! not landed twice
//!  (`doc/planning/cli-git-resident-service.md`,
//! "Durability and crash recovery").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeIntent(dir, intent); // file fsync, rename, directory fsync: the batch's only synchronous write
//! ```

pub use crate::service_journal_intent_text_and_parse::{Intent, intent_text, parse_intent};
use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::service_durable::WhatTheSyncsBecame;
use crate::service_land_start::{IndexWitness, decode, encode};
use crate::wait_ceilings::JOURNAL_WRITE_CEILING;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Extension of an intent whose batch may be unfinished.
pub const OPEN_EXTENSION: &str = "intent";

/// Extension of a finished batch,
/// kept so its request identities are remembered.
pub const DONE_EXTENSION: &str = "done";

/// Extension of a file being written;
/// never read as an intent.
pub const TEMPORARY_EXTENSION: &str = "tmp";

/// Extension of the empty file that holds a batch number for the service landing it,
/// from the moment the number is taken until
///  the landing ends;
/// never read as an intent.
pub const CLAIM_EXTENSION: &str = "claim";

/// Extension of the record a landing writes before it imports,
/// naming the worktree whose scratch reference the import is about to
///  create (`service_scratch_ref_marker_of_a_batch.rs`);
/// never read as an intent.
pub const SCRATCH_EXTENSION: &str = "scratch";

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

/// What a restart does with an open intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Recovery {
    /// The batch never landed;
    /// forget the intent.
    Drop,
    /// A reference lock keeps the batch off its branch,
    /// and its record says so:
    /// nothing is decided about it until the lock goes
    /// (`held_commit_settle.rs`).
    Hold,
    /// The batch landed;
    /// make sure the index shows it,
    /// then remember the identities.
    Finish,
}

/// What:
///  The decision for an open intent from the branch's current value.
/// Why:
///   The branch is the commit point:
/// still at the old value means nothing happened,
/// at the new value or past it
///  (`new_is_ancestor_of_current`) means the batch landed,
/// anywhere else means it cannot land now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recoveryFor(intent, current, newIsAncestorOfCurrent)
/// ```
pub fn recovery_for(intent: &Intent, current: &str, new_is_ancestor_of_current: bool) -> Recovery {
    if current == intent.new || (current != intent.old && new_is_ancestor_of_current) {
        return Recovery::Finish;
    }
    return Recovery::Drop;
}

/// The path of a batch's file with an extension.
pub fn batch_path(dir: &Path, seq: u64, extension: &str) -> PathBuf {
    return dir.join(format!("{seq}.{extension}"));
}

/// A batch number held for one landing;
/// the number is free to take again when this goes out of scope.
#[derive(Debug)]
pub struct BatchClaim {
    seq: u64,
    path: PathBuf,
}

impl BatchClaim {
    /// The number this claim holds.
    pub fn seq(&self) -> u64 {
        return self.seq;
    }
}

/// The claim file goes with the landing,
/// whichever way it ended;
/// the intent (or the finished batch) keeps the number taken.
impl Drop for BatchClaim {
    fn drop(&mut self) {
        let _removed = fs::remove_file(&self.path);
    }
}

/// Whether the journal already holds a record under the number:
/// an open intent,
/// a finished batch,
/// or the record of a scratch
///  reference.
fn number_has_record(dir: &Path, seq: u64) -> bool {
    // A record of a scratch reference outlives its landing only when that reference was left where it is, because a commit only
    //  the reference holds could not be proven safe elsewhere. A number whose reference may still be such a holder is not taken
    //  again: the next import would move that reference and lose what it held.
    return batch_path(dir, seq, OPEN_EXTENSION).exists() || batch_path(dir, seq, DONE_EXTENSION).exists() || batch_path(dir, seq, SCRATCH_EXTENSION).exists();
}

/// What:
///  Takes the first batch number from `first` up that no landing in the journal holds or has held,
/// and holds it.
/// Why:
///   Services that share a journal count from clock readings,
/// which can be equal,
/// and a count that starts a few microseconds
///  behind another's reaches the other's numbers after a few batches.
/// A batch's files are named by its number,
/// so two landings
///  with one number would write one intent over the other and one finished record over the other,
/// and a request that landed
///  would be forgotten.
/// The file is created with `O_EXCL`,
/// so exactly one service owns a number;
/// a number with an intent or a
///  finished batch is skipped too,
/// as the claim of a landing that has ended is gone while its record stays.
/// The loop ends
///  because the journal holds finitely many files and each pass moves to a number past one it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let seq = first
/// while (!(await createExclusive(claimPath(dir, seq))) || hasRecord(dir, seq)) seq += 1 // removes its own claim when it skips
/// return claim(seq)
/// ```
///
/// # Errors
///
/// The error of a claim file that cannot be created for a reason other than the number being held.
pub fn claim_batch(dir: &Path, first: u64) -> std::io::Result<BatchClaim> {
    let mut seq: u64 = first;
    loop {
        let path: PathBuf = batch_path(dir, seq, CLAIM_EXTENSION);
        match File::options().write(true).create_new(true).open(&path) {
            Ok(_created) => {
                let claim: BatchClaim = BatchClaim { seq, path };
                if !number_has_record(dir, seq) {
                    return Ok(claim);
                }
                // A landing that ended left its record under this number and took its claim away: the number is not free.
                drop(claim);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        seq = match seq.checked_add(1) {
            Some(next) => next,
            None => return Err(std::io::Error::other("the journal holds every batch number")),
        };
    }
}

/// What:
///  Writes an intent durably:
/// the file is synced,
/// renamed into place,
/// and the directory is synced.
/// Why:
///   This is the only synchronous write of a batch,
/// whatever its size.
/// The flush decisions are the landing's,
/// read here,
/// so a job of the blocking pool writes exactly what this face writes
/// (`write_intent_awaiting`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeIntent(dir, intent)
/// ```
pub fn write_intent(dir: &Path, intent: &Intent) -> std::io::Result<()> {
    let mut became: WhatTheSyncsBecame = WhatTheSyncsBecame::default();
    let written: std::io::Result<()> = write_intent_carrying(dir, intent, crate::service_durable::syncs_are_deferred_here(), &mut became);
    became.keep();
    return written;
}

/// What:
///  `write_intent` with the landing's deferral given and what its syncs became left to the caller.
/// Why:
///   A job of the blocking pool cannot read the landing's state
/// (`landing_state_carried_by_its_task.rs`),
/// so the task that asks for the write tells it whether the landing defers,
/// and records the syncs the job made when it answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeIntentCarrying(dir, intent, deferring, became)
/// ```
///
/// # Errors
///
/// The error of the write,
/// of the flush or of the rename.
fn write_intent_carrying(dir: &Path, intent: &Intent, deferring: bool, became: &mut WhatTheSyncsBecame) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, intent.seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(intent_text(intent).as_bytes())?;
    crate::service_durable::flush_file_carrying(&file, "intent-file", deferring, became)?;
    fs::rename(&temporary, batch_path(dir, intent.seq, OPEN_EXTENSION))?;
    return crate::service_durable::flush_directory_carrying(dir, "intent-dir", deferring, became);
}

/// What:
///  `write_intent` for a caller that is a task of the runtime:
/// the same write,
/// as a job of the blocking pool,
/// awaited no longer than `JOURNAL_WRITE_CEILING`.
/// Why:
///   A task may not do the write itself:
/// a disk that stopped answering would hold a worker the runtime needs,
/// and no timer on that worker ends it.
/// The landing's deferral travels into the job and the syncs the job made travel back,
/// so the intent is flushed where and when the synchronous face flushes it,
/// and the answer's figures are the same.
/// A job that gives no answer ends the landing it belongs to:
/// nothing lands on a batch whose intent may be missing,
/// and the next service decides the batch from the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeIntentAwaiting(dir, intent)
/// ```
///
/// # Errors
///
/// The error of the write,
/// of the flush or of the rename,
/// and the error of a job that did not run or gave no answer.
pub async fn write_intent_awaiting(dir: &Path, intent: &Intent) -> std::io::Result<()> {
    let job: TheIntentOfABatch = TheIntentOfABatch { dir: dir.to_path_buf(), intent: intent.clone(), deferring: crate::service_durable::syncs_are_deferred_here() };
    return match run_a_job_awaited_no_longer_than(JOURNAL_WRITE_CEILING, job, write_the_intent_of_a_batch).await {
        TheAnswerOfAJobAwaited::Ran((written, became)) => {
            became.keep();
            written
        }
        TheAnswerOfAJobAwaited::NotRun => Err(a_record_the_system_refused_a_thread_for("intent", intent.seq)),
        TheAnswerOfAJobAwaited::NoAnswer => Err(a_record_whose_write_gave_no_answer("intent", intent.seq)),
    };
}

/// The intent one job of the blocking pool writes,
/// with the landing's deferral as the task that asked for it found it.
pub(crate) struct TheIntentOfABatch {
    dir: PathBuf,
    intent: Intent,
    deferring: bool,
}

/// The intent of a batch, written and flushed as one job of the blocking pool.
fn write_the_intent_of_a_batch(job: TheIntentOfABatch) -> (std::io::Result<()>, WhatTheSyncsBecame) {
    let mut became: WhatTheSyncsBecame = WhatTheSyncsBecame::default();
    let written: std::io::Result<()> = write_intent_carrying(job.dir.as_path(), &job.intent, job.deferring, &mut became);
    return (written, became);
}

/// The extension of the record that a batch's index update is installed.
pub const INDEXED_EXTENSION: &str = "indexed";

/// What:
///  Records durably that the index update of the open batch `seq`,
/// which landed the commit `new`,
/// is installed.
/// Why:
///   The service writes it while it still holds the index lock,
/// so no other program can have changed the installed index when
///  the record exists,
/// and a restart that finds it does not replay the update over what was staged after the lock was given up.
///  The record names the commit,
/// so a record left behind by a batch number that is used again does not stop another batch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await markIndexed(dir, seq, newCommit)
/// ```
pub fn mark_indexed(dir: &Path, seq: u64, new: &str) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(format!("{new}\n").as_bytes())?;
    crate::service_durable::flush_file(&file, "indexed-file")?;
    fs::rename(&temporary, batch_path(dir, seq, INDEXED_EXTENSION))?;
    crate::service_fault::note("indexed-recorded");
    return crate::service_durable::flush_directory(dir, "indexed-dir");
}

/// The extension of the record of the live index file a batch started from.
pub const START_EXTENSION: &str = "start";

/// What:
///  Records the witness of the live index file as the landing found it under the index lock.
/// Why:
///   A replay of the batch is made only over an index that is still this file (`service_land_start.rs`),
/// so what another session
///  staged since is never overwritten.
/// It is written after the lock is held and before the branch moves.
/// It is not synced:
/// a
///  power loss that loses it leaves a batch with no record,
/// which no replay is made for,
/// and a record that is lost is the safe
///  side,
/// so the landing does not wait for the disk for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeStart(dir, seq, witness)
/// ```
pub fn write_start(dir: &Path, seq: u64, witness: &IndexWitness) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(format!("{}\n", encode(witness)).as_bytes())?;
    return fs::rename(&temporary, batch_path(dir, seq, START_EXTENSION));
}

/// What:
///  `write_start` for a caller that is a task of the runtime:
/// the same record,
/// as a job of the blocking pool,
/// awaited no longer than `JOURNAL_WRITE_CEILING`.
/// Why:
///   The landing writes this record while it holds the index lock and before its branch moves,
/// so a disk that stopped answering would hold that worker and the lock with it.
/// The record is not synced,
/// so the job carries no flush of the landing's;
/// a job that did not run or gave no answer leaves a batch no replay is made for,
/// which is the answer a record that could not be written already gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeStartAwaiting(dir, seq, witness)
/// ```
///
/// # Errors
///
/// The error of the write or of the rename,
/// and the error of a job that did not run or gave no answer.
pub async fn write_start_awaiting(dir: &Path, seq: u64, witness: &IndexWitness) -> std::io::Result<()> {
    let job: TheStartRecordOfABatch = TheStartRecordOfABatch { dir: dir.to_path_buf(), seq, witness: witness.clone() };
    return match run_a_job_awaited_no_longer_than(JOURNAL_WRITE_CEILING, job, write_the_start_record_of_a_batch).await {
        TheAnswerOfAJobAwaited::Ran(written) => written,
        TheAnswerOfAJobAwaited::NotRun => Err(a_record_the_system_refused_a_thread_for("start record", seq)),
        TheAnswerOfAJobAwaited::NoAnswer => Err(a_record_whose_write_gave_no_answer("start record", seq)),
    };
}

/// The record of the index file a batch started from,
/// as one job of the blocking pool writes it.
pub(crate) struct TheStartRecordOfABatch {
    dir: PathBuf,
    seq: u64,
    witness: IndexWitness,
}

/// The start record of a batch, written as one job of the blocking pool.
fn write_the_start_record_of_a_batch(job: TheStartRecordOfABatch) -> std::io::Result<()> {
    return write_start(job.dir.as_path(), job.seq, &job.witness);
}

/// What:
///  The error of a journal record the system gave no thread to insert.
/// Why:
///   The answer is a failure the person reads,
/// not a wait:
/// the job will not run and changed nothing,
/// so the batch has no record of this step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`the ${record} of batch ${seq} was not written: no thread could be started`)
/// ```
fn a_record_the_system_refused_a_thread_for(record: &str, seq: u64) -> std::io::Error {
    return std::io::Error::other(format!(
        "the {record} of batch {seq} was not written, because the system refused a thread for the step that writes it and took nothing of that step on. \
         Hint: check the process limit (`ulimit -u`) and the control group's `pids.max`, free some, and run the command again"
    ));
}

/// What:
///  The error of a journal record whose write gave no answer in time.
/// Why:
///   A step that started on another thread or outlasted `JOURNAL_WRITE_CEILING` may have written the record,
/// so nothing lands on the answer and the batch is decided from the branch the next time the service starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`the ${record} of batch ${seq} may or may not be on disk`)
/// ```
fn a_record_whose_write_gave_no_answer(record: &str, seq: u64) -> std::io::Error {
    return std::io::Error::other(format!(
        "the {record} of batch {seq} was not written in time, so whether it is on disk is not known; that batch lands only if its branch moved, \
         and it did not, so the next command decides the batch from the branch. Hint: check the disk the Git directory is on, and run the command again"
    ));
}

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
pub fn read_start(dir: &Path, seq: u64) -> Started {
    return match fs::read_to_string(batch_path(dir, seq, START_EXTENSION)) {
        Ok(text) => decode(text.trim_end_matches('\n')).map_or(Started::Absent, Started::Recorded),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Started::Absent,
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Started::Absent,
        Err(_) => Started::Unreadable,
    };
}

/// Drops the start record of a batch that is finished or forgotten.
pub fn remove_start(dir: &Path, seq: u64) {
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
pub fn indexed_state(dir: &Path, seq: u64, new: &str) -> Indexed {
    return match fs::read_to_string(batch_path(dir, seq, INDEXED_EXTENSION)) {
        Ok(text) if text.trim_end() == new => Indexed::Installed,
        Ok(_) => Indexed::Unreadable,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Indexed::Absent,
        Err(_) => Indexed::Unreadable,
    };
}

/// What:
///  Marks a batch finished and drops its record of the index update.
/// Why:
///   The rename is made durable before the record goes:
/// a lost rename with the record already gone would let a restart replay
///  an index update that was installed.
/// A sync that fails keeps the record and the error says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await markDone(dir, seq) // rename intent to done; fsync dir; unlink indexed
/// ```
pub fn mark_done(dir: &Path, seq: u64) -> std::io::Result<()> {
    fs::rename(batch_path(dir, seq, OPEN_EXTENSION), batch_path(dir, seq, DONE_EXTENSION))?;
    crate::service_durable::flush_directory(dir, "journal-dir")?;
    crate::service_fault::note("remove-indexed");
    let _record_removed = fs::remove_file(batch_path(dir, seq, INDEXED_EXTENSION));
    remove_start(dir, seq);
    return Ok(());
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
pub fn list_intents(dir: &Path, extension: &str) -> Vec<(PathBuf, Option<Intent>)> {
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
pub fn list_intents_read_only(dir: &Path, extension: &str) -> Vec<(PathBuf, Option<Intent>)> {
    return list_intents_removing_abandoned(dir, extension, false);
}

/// `list_intents`,
/// with the removal of abandoned temporary and claim files chosen by the caller.
fn list_intents_removing_abandoned(dir: &Path, extension: &str, remove_abandoned: bool) -> Vec<(PathBuf, Option<Intent>)> {
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

/// How many finished batches a journal keeps.
/// A request is answered as already landed only when its own client sends it again
///  after a broken connection:
/// at most `RECONNECTS` times,
/// each after a pause of milliseconds and a wait for a service to
///  start of at most `START_WAIT` (`service_client.rs`),
/// so the resend of a landed request arrives within seconds of the
///  landing.
/// A service lands a batch in the time of a few Git processes,
/// so this many batches reach back past any such resend.
///  Measured:
/// in the service journal (1,085 landings of the `cli-git@` units,
/// 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z) the busiest
///  unit landed 4 batches in its busiest 10 seconds,
/// 4 in its busiest minute and 15 in its busiest ten minutes,
/// and 349 in all,
/// so
///  this many reach back over sixty times the busiest ten minutes.
/// Kept as the coordinator's pick under the owner's delegation,
///  open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const DONE_RETAINED: usize = 1024;

/// What:
///  Removes the files of the oldest finished batches,
/// keeping the newest `retained`,
/// and gives how many it removed.
/// Why:
///   A finished batch's file is only memory of the request identities it landed.
/// Nothing else removed them,
/// so a journal grew by
///  a file per batch for as long as the worktree lived.
/// Batch numbers increase with time,
/// so the oldest are the smallest.
///  Only files named as a batch number with the finished extension are touched,
/// and a file that cannot be removed is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pruneDone(dir, DONE_RETAINED) // 3: removed the three oldest
/// ```
pub fn prune_done(dir: &Path, retained: usize) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut finished: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter_map(finished_batch_of_entry)
        .collect();
    finished.sort_by_key(newest_first_key);
    // Each removal is tried in turn, and only the ones that worked are counted.
    let mut removed: usize = 0;
    for (_, path) in finished.into_iter().skip(retained) {
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
pub fn newest_finished_batches_read_only(dir: &Path, newest: usize) -> Vec<(PathBuf, Option<Intent>)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut finished: Vec<(u64, PathBuf)> = entries.flatten().filter_map(finished_batch_of_entry).collect();
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

/// Journal controls stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_tests.rs"]
mod tests;

/// Controls for the awaited journal writes stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_awaiting_tests.rs"]
mod awaiting_tests;

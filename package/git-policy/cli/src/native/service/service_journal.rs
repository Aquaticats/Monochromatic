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
//! The names its files have are `service_journal_file_names.rs`,
//! and what those files say when they are read is
//! `service_journal_records_read_from_the_journal.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeIntent(dir, intent); // file fsync, rename, directory fsync: the batch's only synchronous write
//! ```

pub use super::service_journal_file_names::{
    CLAIM_EXTENSION,
    CONCLUSION_COPY_EXTENSION,
    DONE_EXTENSION,
    INDEXED_EXTENSION,
    OPEN_EXTENSION,
    SCRATCH_EXTENSION,
    START_EXTENSION,
    TEMPORARY_EXTENSION,
    batch_path,
};
pub use super::service_journal_records_read_from_the_journal::{
    DONE_RETAINED,
    Indexed,
    Started,
    TEMPORARY_GRACE,
    indexed_state,
    landed_commits,
    landed_commits_read_only,
    list_intents,
    list_intents_read_only,
    newest_finished_batches_read_only,
    prune_done,
    read_start,
    remove_start,
};
pub use crate::service_journal_intent_text_and_parse::{
    Intent,
    intent_text,
    parse_intent,
};
use crate::service_land_start::{
    IndexWitness,
    encode,
};
use std::fs::{
    self,
    File,
};
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

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
pub fn recovery_for(
    intent: &Intent,
    current: &str,
    new_is_ancestor_of_current: bool,
) -> Recovery {
    if current == intent.new || (current != intent.old && new_is_ancestor_of_current) {
        return Recovery::Finish;
    }
    return Recovery::Drop;
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
fn number_has_record(
    dir: &Path,
    seq: u64,
) -> bool {
    // A record of a scratch reference outlives its landing only when that reference was left where it is, because a commit only
    //  the reference holds could not be proven safe elsewhere. A number whose reference may still be such a holder is not taken
    //  again: the next import would move that reference and lose what it held.
    return batch_path(dir, seq, OPEN_EXTENSION).exists()
        || batch_path(dir, seq, DONE_EXTENSION).exists()
        || batch_path(dir, seq, SCRATCH_EXTENSION).exists();
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
pub fn claim_batch(
    dir: &Path,
    first: u64,
) -> std::io::Result<BatchClaim> {
    let mut seq: u64 = first;
    loop {
        let path: PathBuf = batch_path(dir, seq, CLAIM_EXTENSION);
        match File::options().write(true).create_new(true).open(&path) {
            Ok(_created) => {
                let claim: BatchClaim = BatchClaim {
                    seq,
                    path,
                };
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
            None => {
                return Err(std::io::Error::other(
                    "the journal holds every batch number",
                ));
            }
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
/// The record also carries the reading of the clocks it was written by,
/// so a cleanup that asks how old the batch's finished record is reads the monotonic clock of this boot
/// and not a wall clock that may have stepped
/// (`service_journal_record_age.rs`).
/// The reading is taken here,
/// before the record is published and after the client sent its request,
/// so the age it gives is never shorter than the wait that client is inside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeIntent(dir, intent) // the intent's lines and its clock reading, synced, renamed, and the directory synced
/// ```
pub fn write_intent(
    dir: &Path,
    intent: &Intent,
) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, intent.seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    let text: String = format!(
        "{}{}",
        intent_text(intent),
        crate::service_journal_record_age::stamp_line(&crate::claim_time::ClaimTime::read())
    );
    file.write_all(text.as_bytes())?;
    crate::service_durable::flush_file(&file, "intent-file")?;
    fs::rename(&temporary, batch_path(dir, intent.seq, OPEN_EXTENSION))?;
    return crate::service_durable::flush_directory(dir, "intent-dir");
}

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
pub fn mark_indexed(
    dir: &Path,
    seq: u64,
    new: &str,
) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(format!("{new}\n").as_bytes())?;
    crate::service_durable::flush_file(&file, "indexed-file")?;
    fs::rename(&temporary, batch_path(dir, seq, INDEXED_EXTENSION))?;
    crate::service_fault::note("indexed-recorded");
    return crate::service_durable::flush_directory(dir, "indexed-dir");
}

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
pub fn write_start(
    dir: &Path,
    seq: u64,
    witness: &IndexWitness,
) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(format!("{}\n", encode(witness)).as_bytes())?;
    return fs::rename(&temporary, batch_path(dir, seq, START_EXTENSION));
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
pub fn mark_done(
    dir: &Path,
    seq: u64,
) -> std::io::Result<()> {
    fs::rename(
        batch_path(dir, seq, OPEN_EXTENSION),
        batch_path(dir, seq, DONE_EXTENSION),
    )?;
    crate::service_durable::flush_directory(dir, "journal-dir")?;
    crate::service_fault::note("remove-indexed");
    let _record_removed = fs::remove_file(batch_path(dir, seq, INDEXED_EXTENSION));
    remove_start(dir, seq);
    return Ok(());
}

/// Journal controls stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_tests.rs"]
mod tests;

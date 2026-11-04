//! What:
//!  The resident service's journal: one file per batch of commits, with the one durable write the batch needs, and the
//!  decision a restart makes about a batch it finds open.
//! Why:
//!  The branch moves only after the intent is on disk, so after any kill the journal and the branch say whether the
//!  batch landed; a restart finishes the one step that may be missing (the index update) or drops the intent, and a
//!  request that landed is remembered by its identity so a retry is answered, not landed twice
//!  (`doc/planning/cli-git-resident-service.md`, "Durability and crash recovery").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeIntent(dir, intent); // file fsync, rename, directory fsync: the batch's only synchronous write
//! ```

use super::repo_read_layout::dot_git_target;
use super::service_land_start::{IndexWitness, decode, encode};
use super::service_protocol::Cleanup;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Extension of an intent whose batch may be unfinished.
pub const OPEN_EXTENSION: &str = "intent";

/// Extension of a finished batch, kept so its request identities are remembered.
pub const DONE_EXTENSION: &str = "done";

/// Extension of a file being written; never read as an intent.
const TEMPORARY_EXTENSION: &str = "tmp";

/// Extension of the empty file that holds a batch number for the service landing it, from the moment the number is taken until
///  the landing ends; never read as an intent.
pub const CLAIM_EXTENSION: &str = "claim";

/// How old a temporary file must be before a listing treats it as the remains of a killed write and removes it. A write takes
///  milliseconds; the grace covers a writer stalled on a slow disk or a loaded host. Measured: the longest journal write step of
///  1,302 landings in the service journal (`journal_ms`, 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z) was 40,188 ms, the 99th
///  percentile 3,792 ms, so ten minutes is fifteen times the longest. Kept as the coordinator's pick under the owner's delegation, open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`, "Sizes and ages of the service's own housekeeping").
pub const TEMPORARY_GRACE: std::time::Duration = std::time::Duration::from_secs(10 * 60);

/// What:
///  What the service decided to do for a batch, written before the branch moves.
/// Why:
///   Enough to finish the batch after a kill: where the branch was, where it goes, which requests, which index paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Intent = { seq; repository; commonDir; reference; old; new; ids; paths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intent {
    /// Number of the batch, unique and increasing within one journal.
    pub seq: u64,
    /// Worktree root whose index the batch updates.
    pub repository: PathBuf,
    /// Common git directory.
    pub common_dir: PathBuf,
    /// The branch.
    pub reference: String,
    /// The branch before the batch.
    pub old: String,
    /// The branch after the batch.
    pub new: String,
    /// Identities of the requests in the batch.
    pub ids: Vec<String>,
    /// The commit each request became, in the same order as the identities.
    pub commits: Vec<String>,
    /// Paths whose index entries the batch updates.
    pub paths: Vec<String>,
    /// Whether the index entries to update are the paths that differ between `old` and `new`, read from the repository
    ///  when they are needed (a commit Git built has no list of changes to carry, and its names are bytes).
    pub derived: bool,
    /// The conclusion state the landing removes after the branch moved; nothing for any other batch.
    pub cleanup: Option<Cleanup>,
    /// The real index whose lock the landing held, so a restart can remove its own stale lock and finish the index.
    pub index: PathBuf,
    /// The worktree's own Git directory, which names the instance that serves the batch; an intent written before it was
    ///  recorded reads as the Git directory of its repository's worktree.
    pub git_dir: PathBuf,
}

/// What a restart does with an open intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Recovery {
    /// The batch never landed; forget the intent.
    Drop,
    /// The batch landed; make sure the index shows it, then remember the identities.
    Finish,
}

/// What:
///  The decision for an open intent from the branch's current value.
/// Why:
///   The branch is the commit point: still at the old value means nothing happened, at the new value or past it
///  (`new_is_ancestor_of_current`) means the batch landed, anywhere else means it cannot land now.
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

/// The text of an intent: `key=value` lines, tabs between list items.
pub fn intent_text(intent: &Intent) -> String {
    let cleanup: String = intent.cleanup.as_ref().map_or_else(String::new, |found| {
        return format!("{}\t{}\t{}", found.git_dir.display(), found.scratch.display(), u8::from(found.reftable));
    });
    return format!(
        "seq={}\nrepository={}\ncommon={}\nref={}\nold={}\nnew={}\nids={}\ncommits={}\npaths={}\nderived={}\ncleanup={}\nindex={}\ngitdir={}\n",
        intent.seq,
        intent.repository.display(),
        intent.common_dir.display(),
        intent.reference,
        intent.old,
        intent.new,
        intent.ids.join("\t"),
        intent.commits.join("\t"),
        intent.paths.join("\t"),
        u8::from(intent.derived),
        cleanup,
        intent.index.display(),
        intent.git_dir.display(),
    );
}

/// What:
///  The worktree's own Git directory, read from the worktree's `.git` entry: the entry itself when it is a directory, the
///  directory a `gitdir:` file names for a linked worktree, and `.git` under the worktree when it cannot be read.
/// Why:
///   An intent written before it recorded its Git directory has only the repository to name the worktree by; the worktree's
///  instance is the hash of this directory, so the recovery that finds the intent must resolve it the way a commit does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitDirOfWorktree('/work/repo') // '/work/repo/.git', or '/work/main/.git/worktrees/repo' for a linked worktree
/// ```
fn git_dir_of_worktree(repository: &Path) -> PathBuf {
    return match dot_git_target(repository) {
        Some(Some((git_dir, _))) => git_dir,
        Some(None) | None => repository.join(".git"),
    };
}

/// What:
///  Reads an intent's text.
/// Why:
///   A damaged file is no intent: recovery drops it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseIntent(text) // undefined when damaged
/// ```
pub fn parse_intent(text: &str) -> Option<Intent> {
    let value = |key: &str| -> Option<&str> { return text.lines().find_map(|line| { return line.strip_prefix(key)?.strip_prefix('='); }); };
    let list = |key: &str| -> Option<Vec<String>> {
        return Some(value(key)?.split('\t').filter(|item| { return !item.is_empty(); }).map(str::to_string).collect());
    };
    let index: PathBuf = value("index").map_or_else(|| { return PathBuf::from(value("repository").unwrap_or("")).join(".git").join("index"); }, PathBuf::from);
    let repository: PathBuf = PathBuf::from(value("repository")?);
    // An intent written before the Git directory was recorded names its worktree by the repository it was made for, never
    //  by the directory its index sits in, which `GIT_INDEX_FILE` can put anywhere.
    let worktree_git_dir: PathBuf = value("gitdir").map_or_else(|| { return git_dir_of_worktree(repository.as_path()); }, PathBuf::from);
    return Some(Intent {
        seq: value("seq")?.parse().ok()?,
        repository,
        common_dir: PathBuf::from(value("common")?),
        reference: value("ref")?.to_string(),
        old: value("old")?.to_string(),
        new: value("new")?.to_string(),
        ids: list("ids")?,
        commits: list("commits")?,
        paths: list("paths")?,
        // Intents written before these fields existed name neither.
        derived: value("derived") == Some("1"),
        index,
        git_dir: worktree_git_dir,
        cleanup: value("cleanup").and_then(|entry| {
            let mut parts = entry.split('\t');
            let (git_dir, scratch, reftable) = (parts.next()?, parts.next()?, parts.next()?);
            return Some(Cleanup { git_dir: PathBuf::from(git_dir), scratch: PathBuf::from(scratch), reftable: reftable == "1" });
        }),
    });
}

/// The path of a batch's file with an extension.
pub fn batch_path(dir: &Path, seq: u64, extension: &str) -> PathBuf {
    return dir.join(format!("{seq}.{extension}"));
}

/// A batch number held for one landing; the number is free to take again when this goes out of scope.
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

/// The claim file goes with the landing, whichever way it ended; the intent (or the finished batch) keeps the number taken.
impl Drop for BatchClaim {
    fn drop(&mut self) {
        let _removed = fs::remove_file(&self.path);
    }
}

/// Whether the journal already holds a record under the number: an open intent or a finished batch.
fn number_has_record(dir: &Path, seq: u64) -> bool {
    return batch_path(dir, seq, OPEN_EXTENSION).exists() || batch_path(dir, seq, DONE_EXTENSION).exists();
}

/// What:
///  Takes the first batch number from `first` up that no landing in the journal holds or has held, and holds it.
/// Why:
///   Services that share a journal count from clock readings, which can be equal, and a count that starts a few microseconds
///  behind another's reaches the other's numbers after a few batches. A batch's files are named by its number, so two landings
///  with one number would write one intent over the other and one finished record over the other, and a request that landed
///  would be forgotten. The file is created with `O_EXCL`, so exactly one service owns a number; a number with an intent or a
///  finished batch is skipped too, as the claim of a landing that has ended is gone while its record stays. The loop ends
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
        seq = seq.checked_add(1).ok_or_else(|| { return std::io::Error::other("the journal holds every batch number"); })?;
    }
}

/// What:
///  Writes an intent durably: the file is synced, renamed into place, and the directory is synced.
/// Why:
///   This is the only synchronous write of a batch, whatever its size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeIntent(dir, intent)
/// ```
pub fn write_intent(dir: &Path, intent: &Intent) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, intent.seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(intent_text(intent).as_bytes())?;
    super::service_durable::flush_file(&file, "intent-file")?;
    fs::rename(&temporary, batch_path(dir, intent.seq, OPEN_EXTENSION))?;
    return super::service_durable::flush_directory(dir, "intent-dir");
}

/// The extension of the record that a batch's index update is installed.
pub const INDEXED_EXTENSION: &str = "indexed";

/// What:
///  Records durably that the index update of the open batch `seq`, which landed the commit `new`, is installed.
/// Why:
///   The service writes it while it still holds the index lock, so no other program can have changed the installed index when
///  the record exists, and a restart that finds it does not replay the update over what was staged after the lock was given up.
///  The record names the commit, so a record left behind by a batch number that is used again does not stop another batch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await markIndexed(dir, seq, newCommit)
/// ```
pub fn mark_indexed(dir: &Path, seq: u64, new: &str) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(format!("{new}\n").as_bytes())?;
    super::service_durable::flush_file(&file, "indexed-file")?;
    fs::rename(&temporary, batch_path(dir, seq, INDEXED_EXTENSION))?;
    crate::service_fault::note("indexed-recorded");
    return super::service_durable::flush_directory(dir, "indexed-dir");
}

/// The extension of the record of the live index file a batch started from.
pub const START_EXTENSION: &str = "start";

/// What:
///  Records the witness of the live index file as the landing found it under the index lock.
/// Why:
///   A replay of the batch is made only over an index that is still this file (`service_land_start.rs`), so what another session
///  staged since is never overwritten. It is written after the lock is held and before the branch moves. It is not synced: a
///  power loss that loses it leaves a batch with no record, which no replay is made for, and a record that is lost is the safe
///  side, so the landing does not wait for the disk for it.
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

/// What a batch's start record says.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Started {
    /// No usable record: an older batch, a landing that could not write one, or a record that is damaged. No replay is made.
    Absent,
    /// The index file the landing started from.
    Recorded(IndexWitness),
    /// A record exists and cannot be read now, so nothing is decided.
    Unreadable,
}

/// The start record of the batch `seq`; a missing file or damaged text is no record, and a file that cannot be read now is
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
    /// No record: the update was not recorded as installed.
    Absent,
    /// The record names the commit: the update was installed.
    Installed,
    /// A record exists and cannot be read, or names another commit: nothing can be said about the index.
    Unreadable,
}

/// What:
///  What the completion record of the open batch `seq` that landed `new` says.
/// Why:
///   Only a missing record means the update was not installed. A record that is there and cannot be read says nothing, and a
///  restart that took it for none would replay the update over whatever was staged since, so the caller stops on it.
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
///   The rename is made durable before the record goes: a lost rename with the record already gone would let a restart replay
///  an index update that was installed. A sync that fails keeps the record and the error says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await markDone(dir, seq) // rename intent to done; fsync dir; unlink indexed
/// ```
pub fn mark_done(dir: &Path, seq: u64) -> std::io::Result<()> {
    fs::rename(batch_path(dir, seq, OPEN_EXTENSION), batch_path(dir, seq, DONE_EXTENSION))?;
    super::service_durable::flush_directory(dir, "journal-dir")?;
    crate::service_fault::note("remove-indexed");
    let _record_removed = fs::remove_file(batch_path(dir, seq, INDEXED_EXTENSION));
    remove_start(dir, seq);
    return Ok(());
}

/// What:
///  The intents in the journal with an extension, in batch order, and the abandoned temporary files removed on the way.
/// Why:
///   Recovery reads the open ones; startup reads the done ones for the request identities. A temporary file younger than the
///  grace may be another instance's write in progress, so it stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listIntents(dir, 'intent')
/// ```
pub fn list_intents(dir: &Path, extension: &str) -> Vec<(PathBuf, Option<Intent>)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, Option<Intent>)> = Vec::new();
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        match path.extension().and_then(|text| { return text.to_str(); }) {
            Some(TEMPORARY_EXTENSION | CLAIM_EXTENSION) => {
                // Only the remains of a write or a landing that was killed: a younger file may be another instance's write in
                //  progress, and removing it would fail that instance's rename or hand its number to a second landing.
                let abandoned: bool = entry
                    .metadata()
                    .and_then(|meta| { return meta.modified(); })
                    .is_ok_and(|modified| { return modified.elapsed().is_ok_and(|age| { return age > TEMPORARY_GRACE; }); });
                if abandoned {
                    let _removed = fs::remove_file(&path);
                }
            }
            Some(found_extension) if found_extension == extension => {
                let parsed: Option<Intent> = fs::read_to_string(&path).ok().and_then(|text| { return parse_intent(text.as_str()); });
                found.push((path, parsed));
            }
            _ => {}
        }
    }
    found.sort_by_key(|(path, intent)| {
        return intent.as_ref().map_or_else(|| { return path.to_string_lossy().len() as u64; }, |parsed| { return parsed.seq; });
    });
    return found;
}

/// How many finished batches a journal keeps. A request is answered as already landed only when its own client sends it again
///  after a broken connection: at most `RECONNECTS` times, each after a pause of milliseconds and a wait for a service to
///  start of at most `START_WAIT` (`service_client.rs`), so the resend of a landed request arrives within seconds of the
///  landing. A service lands a batch in the time of a few Git processes, so this many batches reach back past any such resend.
///  Measured: in the service journal (1,085 landings of the `cli-git@` units, 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z) the busiest
///  unit landed 4 batches in its busiest 10 seconds, 4 in its busiest minute and 15 in its busiest ten minutes, and 349 in all, so
///  this many reach back over sixty times the busiest ten minutes. Kept as the coordinator's pick under the owner's delegation,
///  open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`, "Sizes and ages of the service's own housekeeping").
pub const DONE_RETAINED: usize = 1024;

/// What:
///  Removes the files of the oldest finished batches, keeping the newest `retained`, and gives how many it removed.
/// Why:
///   A finished batch's file is only memory of the request identities it landed. Nothing else removed them, so a journal grew by
///  a file per batch for as long as the worktree lived. Batch numbers increase with time, so the oldest are the smallest.
///  Only files named as a batch number with the finished extension are touched, and a file that cannot be removed is left.
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
        .filter_map(|entry| {
            let path: PathBuf = entry.path();
            if path.extension()? != DONE_EXTENSION || !entry.file_type().ok()?.is_file() {
                return None;
            }
            let seq: u64 = path.file_stem()?.to_str()?.parse().ok()?;
            return Some((seq, path));
        })
        .collect();
    finished.sort_by_key(|(seq, _)| { return std::cmp::Reverse(*seq); });
    return finished.into_iter().skip(retained).filter(|(_, path)| { return fs::remove_file(path).is_ok(); }).count();
}

/// The request identities of every finished batch, each with the commit it became.
pub fn landed_commits(dir: &Path) -> HashMap<String, String> {
    let mut landed: HashMap<String, String> = HashMap::new();
    for (_, intent) in list_intents(dir, DONE_EXTENSION) {
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

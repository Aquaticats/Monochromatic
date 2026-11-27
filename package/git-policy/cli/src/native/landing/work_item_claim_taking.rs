//! What:
//!  The steps that take one pending record out of the queue:
//! the oldest pending item,
//! the oldest pending item of some kinds,
//! every pending item of one round from a single listing,
//! and the one step all three share,
//! with the stamp that keeps a claim's lease readable.
//! Why:
//!  A rename is the whole exclusion between two consumers of one queue,
//! and the three ways of taking work differ only in the listing they walk,
//! so they sit together and read the record they take (`work_item_record.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const one = claimNextOf(workDir, now, kinds); const all = claimAllOf(workDir, kinds);
//! ```

use crate::claim_time::ClaimTime;
use crate::private_storage::create_private_directory_all;
use crate::service_timing::log;
use crate::work_item_kinds_and_their_names::WorkKind;
use crate::work_item_record::{
    Claimed,
    WorkItem,
    record_of_a_later_release,
};
use crate::work_item_state_directories_and_claim_names::{
    CLAIMED,
    FAILED,
    PENDING,
    RECOVERING,
    claim_file_name,
    entries_oldest_first,
    id_of_entry,
    new_generation,
    state_directory,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::SystemTime;

/// What:
///  Set the modification time of the file at `path` to now.
/// Why:
///   A release that cannot read the reading in a claim's name ages the claim by the file's modification time,
/// which a rename
///  keeps.
/// Claims and renewals are renames,
/// so without this such a reader sees the time the item was queued,
/// not the last
///  renewal,
/// and takes a live holder's claim.
/// A failure is returned,
/// because a claim or renewal that went on without the stamp
///  would be one that such a reader takes (a file another user owns cannot be stamped).
///  The file is opened as a plain one (`crate::plain_file::open_plain_file`):
/// a queue entry that is a named pipe is a record this command cannot read rather than a wait with no end,
/// and a claim step that waited for a writer would hold the round for as long as the pipe has no reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await utimesPlain(path, new Date(), new Date()) // refuses a name that is not a plain file
/// ```
pub(crate) fn stamp_modification_time(path: &Path) -> std::io::Result<()> {
    return crate::plain_file::open_plain_file(path)?.set_modified(SystemTime::now());
}

/// How a claim or renewal stamps the file it holds.
pub(crate) type Stamp<'a> = &'a dyn Fn(&Path) -> std::io::Result<()>;

/// The path of the file that holds the claim of `id` now,
/// or nothing when it is not claimed or being recovered.
pub fn find_claim(
    work_dir: &Path,
    id: &str,
) -> Option<PathBuf> {
    // `claimed` first, then `recovering`, each oldest first; the first entry that is the claim of `id`.
    for state in [CLAIMED, RECOVERING] {
        for (name, _modified) in entries_oldest_first(work_dir, state) {
            if id_of_entry(name.as_str()) == id {
                return Some(state_directory(work_dir, state).join(name));
            }
        }
    }
    return None;
}

/// What:
///  Claim the oldest pending item by renaming it into `claimed` under a name that carries a fresh generation and the
///  reading `now`.
/// A record that does not decode is moved to `failed` so it cannot block the queue.
/// Why:
///   A rename is atomic:
/// of two consumers racing for one item,
/// the loser sees the file gone and takes the next one.
/// The
///  generation and the reading are in the name,
/// so the claim is whole from the instant it exists;
/// nothing is written to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'pending', id), join(workDir, 'claimed', claimFileName(id, newGeneration(), now)))
/// ```
pub fn claim_next(
    work_dir: &Path,
    now: &ClaimTime,
) -> Result<Option<Claimed>, String> {
    return claim_next_of(
        work_dir,
        now,
        &[
            WorkKind::Review,
            WorkKind::Push,
            WorkKind::Maintenance,
            WorkKind::Trash,
            WorkKind::Prewarm,
            WorkKind::Relink,
            WorkKind::SubvolumeDelete,
            WorkKind::SnapshotAhead,
        ],
    );
}

/// What:
///  Claim the oldest pending item of the given kinds,
/// leaving every other kind where it is.
/// Why:
///   Consumers of different kinds share one queue;
/// the kind starts the file name,
/// so a consumer never opens,
/// moves or
///   stamps an item that belongs to another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'pending', id), join(workDir, 'claimed', name)) // only ids that start with a handled kind
/// ```
pub fn claim_next_of(
    work_dir: &Path,
    now: &ClaimTime,
    kinds: &[WorkKind],
) -> Result<Option<Claimed>, String> {
    return claim_next_stamped(work_dir, now, kinds, &stamp_modification_time);
}

/// What:
///  Claims every pending item of the given kinds,
/// from one listing of `pending`,
/// oldest first.
/// Why:
///   A drain that claims its work with `claim_next_of` lists and sorts `pending` for each claim,
/// so a queue of N items
///  costs N listings and N sorts (`RG2`).
/// The round takes the listing once and tries every entry of it,
/// so a drain of N
///  items costs one listing.
/// Each claim still carries the reading of the moment it is taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimed = []; for (const name of entriesOldestFirst(workDir, 'pending')) { const item = await take(name, now()); if (item) claimed.push(item) }; return claimed
/// ```
pub fn claim_all_of(
    work_dir: &Path,
    kinds: &[WorkKind],
) -> Result<Vec<Claimed>, String> {
    let mut claimed: Vec<Claimed> = Vec::new();
    for (id, queued) in entries_oldest_first(work_dir, PENDING) {
        // Each claim carries the reading of the moment it is taken, not of the start of the round, which a long round makes
        //  most of a lease old.
        if let Some(taken) = take_entry(
            work_dir,
            id.as_str(),
            queued,
            kinds,
            &ClaimTime::read(),
            &stamp_modification_time,
        )? {
            claimed.push(taken);
        }
    }
    return Ok(claimed);
}

/// What:
///  `claim_next_of` with the stamp of the claim's modification time given,
/// so a test can make the stamp fail.
/// Why:
///   The entry is taken into `recovering` under its claim name first,
/// where no reader of `claimed` looks and where the
///  entry is this instance's alone;
/// only there is it stamped,
/// and only then published into `claimed`.
/// A claim that cannot be
///  stamped or published goes back to `pending` with the modification time it had,
/// so a queue that cannot be drained keeps
///  its true age and the backlog notice still fires.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(pending, staged);
/// await stamp(staged);
/// await rename(staged, join(workDir, 'claimed', name))
/// ```
pub(crate) fn claim_next_stamped(
    work_dir: &Path,
    now: &ClaimTime,
    kinds: &[WorkKind],
    stamp: Stamp<'_>,
) -> Result<Option<Claimed>, String> {
    for (id, queued) in entries_oldest_first(work_dir, PENDING) {
        if let Some(taken) = take_entry(work_dir, id.as_str(), queued, kinds, now, stamp)? {
            return Ok(Some(taken));
        }
    }
    return Ok(None);
}

/// What:
///  Claims the pending entry `id` of one of `kinds`,
/// or answers nothing when it is no entry to claim.
/// Why:
///   The two ways of claiming,
/// one item at a time and a whole round,
/// differ only in the listing they walk,
/// so one function
///  takes an entry and each caller drives it through its own listing.
/// `queued` is the entry's modification time,
///  which a claim that cannot be taken puts back.
///
/// # Errors
///
/// A generation that cannot be made,
/// or a claim that cannot be read.
fn take_entry(
    work_dir: &Path,
    id: &str,
    queued: SystemTime,
    kinds: &[WorkKind],
    now: &ClaimTime,
    stamp: Stamp<'_>,
) -> Result<Option<Claimed>, String> {
    let mut found_kind: Option<&WorkKind> = None;
    for candidate in kinds {
        if id.starts_with(format!("{}-", candidate.name()).as_str()) {
            found_kind = Some(candidate);
            break;
        }
    }
    let Some(kind) = found_kind else {
        return Ok(None);
    };
    // An id that is not a kind and what that kind names could make names that do not fit; it never becomes a claim.
    if !crate::work_item_ids_and_names::is_id_suffix(kind, &id[kind.name().len() + 1..]) {
        // A later release may add a kind whose name begins with one this build knows, and its record is that release's.
        let waiting: PathBuf = state_directory(work_dir, PENDING).join(id);
        if left_for_a_later_release(waiting.as_path()) {
            log(format!("the queued entry {id:?} is a record of a later release, so it is left for that release").as_str());
            return Ok(None);
        }
        quarantine_pending(
            work_dir,
            id,
            format!(
                "is not a kind and a suffix of at most {} characters: an object name, and for a push a branch digest after it",
                crate::work_item_ids_and_names::MAX_ID_SUFFIX_LENGTH
            )
            .as_str(),
        );
        return Ok(None);
    }
    let name: String = claim_file_name(id, new_generation()?.as_str(), now);
    let pending_path: PathBuf = state_directory(work_dir, PENDING).join(id);
    let staged: PathBuf = state_directory(work_dir, RECOVERING).join(name.as_str());
    let claimed_path: PathBuf = state_directory(work_dir, CLAIMED).join(name.as_str());
    if create_private_directory_all(state_directory(work_dir, RECOVERING).as_path()).is_err()
        || std::fs::rename(pending_path.as_path(), staged.as_path()).is_err()
    {
        return Ok(None);
    }
    let stamped: std::io::Result<()> = stamp(staged.as_path());
    let published: std::io::Result<()> = match stamped {
        Ok(()) => std::fs::rename(staged.as_path(), claimed_path.as_path()),
        Err(error) => Err(error),
    };
    if let Err(error) = published {
        // A queue directory holds records and nothing else, so a name that is not a plain file is one no round can read: it
        // goes where nothing runs it, with a line naming it, rather than back into the queue it would block for ever.
        if error.kind() == std::io::ErrorKind::InvalidInput {
            let failed: PathBuf = state_directory(work_dir, FAILED);
            let moved: bool = create_private_directory_all(failed.as_path()).is_ok()
                && std::fs::rename(staged.as_path(), failed.join(id)).is_ok();
            log(format!(
                "the queued entry {id:?} is not a plain file (a named pipe, a device or a directory), so it was {}",
                if moved { "moved to failed" } else { "left in recovering because moving it failed" }
            )
            .as_str());
            return Ok(None);
        }
        log(
            format!("the claim of {id} could not be taken, so the item stays queued: {error}")
                .as_str(),
        );
        // The staged file is this instance's alone:
        // the age it had is put back before it is queued again,
        // whatever the stamp did.
        let opened: std::io::Result<std::fs::File> =
            crate::plain_file::open_plain_file(staged.as_path());
        let restored: std::io::Result<()> = match opened {
            Ok(file) => file.set_modified(queued),
            Err(open_error) => Err(open_error),
        };
        let requeued: std::io::Result<()> = match restored {
            Ok(()) => std::fs::rename(staged.as_path(), pending_path.as_path()),
            Err(restore_error) => Err(restore_error),
        };
        if let Err(requeue_error) = requeued {
            log(format!("the staged claim of {id} could not be queued again; its lease will run out and recover it: {requeue_error}").as_str());
        }
        return Ok(None);
    }
    let text: String = match crate::plain_file::read_plain_text(claimed_path.as_path()) {
        Ok(read) => read,
        Err(error) => return Err(format!("reading the claim of {id} failed: {error}")),
    };
    return match WorkItem::decode(text.as_str()) {
        // The record is the work the file name says;
        // a record of other work would be settled as done under this id.
        Ok(item) if item.id() == id || item.id_before_the_branch() == id => Ok(Some(Claimed {
            id: id.to_owned(),
            name,
            item,
        })),
        _ => {
            // A record of a kind or a version this build does not know is a later release's, and it is queued again for
            // that release rather than settled in `failed`, where nothing reads it again.
            if left_for_a_later_release(claimed_path.as_path()) {
                log(format!("the claim of {id} is a record of a later release, so it is queued again for that release").as_str());
                let _ = std::fs::rename(
                    claimed_path.as_path(),
                    state_directory(work_dir, PENDING).join(id),
                );
                return Ok(None);
            }
            let _ = std::fs::rename(
                claimed_path.as_path(),
                state_directory(work_dir, FAILED).join(id),
            );
            Ok(None)
        }
    };
}

/// What:
///  Whether the record at `path` was written by a release later than this one.
/// Why:
///  The answer decides whether a consumer settles the record in `failed`,
///  where nothing reads it again,
///  or leaves it for the release that wrote it.
///  A file that cannot be read is no later release's record:
///  the consumer's own quarantine is what an unreadable file answers to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const later = (path) => recordOfALaterRelease(readFileSync(path, 'utf8'));
/// ```
fn left_for_a_later_release(path: &Path) -> bool {
    return match crate::plain_file::read_plain_text(path) {
        Ok(text) => record_of_a_later_release(text.as_str()),
        Err(_unreadable) => false,
    };
}

/// Moves the entry `id` of `pending` to `failed` and says why,
/// where nothing can be done about a failure of either.
fn quarantine_pending(
    work_dir: &Path,
    id: &str,
    reason: &str,
) {
    let failed: PathBuf = state_directory(work_dir, FAILED);
    let moved: bool = create_private_directory_all(failed.as_path()).is_ok()
        && std::fs::rename(state_directory(work_dir, PENDING).join(id), failed.join(id)).is_ok();
    log(format!(
        "the queued entry {id:?} {reason}, so it was {}",
        if moved {
            "moved to failed"
        } else {
            "left in pending because moving it failed"
        }
    )
    .as_str());
}

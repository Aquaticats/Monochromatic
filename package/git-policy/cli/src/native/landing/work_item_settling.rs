//! What:
//!  What a consumer does with the item it holds:
//! move its claim to `done` or to another state,
//! start the claim's lease again,
//! and how long the oldest pending item has waited.
//! Why:
//!  A settlement reaches only the claim it was made for,
//! because the claim's file name carries its generation,
//! so every step here renames one claim file of the record it was given (`work_item_record.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! settle(workDir, claimed, 'done'); const renewed = renewClaim(workDir, claimed.name, now);
//! ```

use crate::claim_time::ClaimTime;
use crate::service_timing::log;
use crate::work_item_claim_taking::{
    Stamp,
    stamp_modification_time,
};
use crate::work_item_record::Claimed;
use crate::work_item_state_directories_and_claim_names::{
    CLAIMED,
    PENDING,
    claim_file_name,
    entries_oldest_first,
    id_of_entry,
    new_generation,
    parse_claim_name,
    state_directory,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    SystemTime,
};

/// What:
///  Move a claimed item to `done` (or to `failed`),
/// after the consumer's result is durable.
/// Why:
///   A crash before this rename leaves the item claimed,
/// and its lease runs out;
/// the work is then done again,
/// so a
///   handler must tolerate a repeat.
/// The rename names the claim's generation,
/// so a holder whose claim was taken over and
///  claimed again by another settles nothing of the other's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'claimed', claimed.name), join(workDir, 'done', claimed.id))
/// ```
pub fn settle(
    work_dir: &Path,
    claimed: &Claimed,
    state: &str,
) -> Result<(), String> {
    return settle_named(work_dir, claimed.name.as_str(), state);
}

/// `settle` for the claim file `name`,
/// whose id is the part of the name before the generation.
pub fn settle_named(
    work_dir: &Path,
    name: &str,
    state: &str,
) -> Result<(), String> {
    let id: &str = id_of_entry(name);
    return match std::fs::rename(
        state_directory(work_dir, CLAIMED).join(name),
        state_directory(work_dir, state).join(id),
    ) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("moving {id} to {state} failed: {error}")),
    };
}

/// What:
///  Start the lease of the claim file `name` again at `now`,
/// returning its new name;
/// nothing when the claim is no longer in
///  `claimed` under that name or its modification time could not be set.
/// Why:
///   A holder that is still working must not look abandoned:
/// every consumer of a directory that several consumers drain reads
///   the reading in the name as the proof of life.
/// The renewal is a rename of the exact name the holder last wrote,
/// so a
///  claim that was taken over,
/// settled,
/// or taken over and claimed again by someone else is not found and is never brought
///  back or renewed on the other's behalf;
/// and an instance that looked at the old name and means to take it finds nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(workDir, 'claimed', name), join(workDir, 'claimed', claimFileName(id, token, now)))
/// ```
pub fn renew_claim(
    work_dir: &Path,
    name: &str,
    now: &ClaimTime,
) -> Option<String> {
    return renew_claim_stamped(work_dir, name, now, &stamp_modification_time);
}

/// `renew_claim` with the stamp of the claim's modification time given,
/// so a test can make the stamp fail.
/// A renewal whose
///  stamp failed is no renewal:
/// an older reader would still see the old modification time and take the claim.
pub(crate) fn renew_claim_stamped(
    work_dir: &Path,
    name: &str,
    now: &ClaimTime,
    stamp: Stamp<'_>,
) -> Option<String> {
    let id: &str = id_of_entry(name);
    // A claim by an earlier release has no generation; its renewal gives it one.
    let token: String = match parse_claim_name(name) {
        Some((token, _)) => token.to_owned(),
        None => new_generation().ok()?,
    };
    let renewed: String = claim_file_name(id, token.as_str(), now);
    let directory: PathBuf = state_directory(work_dir, CLAIMED);
    if let Err(error) = stamp(directory.join(name).as_path()) {
        log(format!("the claim {name} could not be renewed: {error}").as_str());
        return None;
    }
    return match std::fs::rename(directory.join(name), directory.join(renewed.as_str())) {
        Ok(()) => Some(renewed),
        Err(_refused) => None,
    };
}

/// What:
///  How long the oldest pending item has waited,
/// or `None` when nothing is pending.
/// Why:
///   The wrapper's one-line notice that no consumer is draining the queue is this age past a bound;
/// a process id in a
///   file would be meaningless across process-id namespaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// now - oldestPending.mtime
/// ```
pub fn oldest_pending_age(
    work_dir: &Path,
    now: SystemTime,
) -> Option<Duration> {
    let pending: Vec<(String, SystemTime)> = entries_oldest_first(work_dir, PENDING);
    let (_name, created) = pending.first()?;
    return Some(now.duration_since(*created).unwrap_or(Duration::ZERO));
}

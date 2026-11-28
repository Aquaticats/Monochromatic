//! What:
//!  The refusal a removal owes a lock whose words are not its add's,
//!  read before anything is written.
//! Why:
//!  Git refuses a locked registration to its own removal and refuses it to one force,
//!  so Git's own message names `-f -f` as the spelling that gets past a lock
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "What Git 2.56.1 does"),
//!  and that spelling here runs the wrapper's removal instead of Git's.
//!  The wrapper's order reads the lock at its Git settle alone,
//!  which comes after the snapshot batch,
//!  the head destroy and the origin attempt,
//!  and the settle refuses words it cannot recompose from the add's nonce
//!  (the same design's S3),
//!  so a person's lock met only there would be met after the clone's history went.
//!  Measured at `8bdae8a0d8`:
//!  with a person's lock on a slot's registration,
//!  a removal destroyed the clone's own snapshots,
//!  the origin snapshot and the head,
//!  and only then refused.
//!  The entry therefore reads the lock under the slot's claim,
//!  before its first write,
//!  and words this add did not write refuse the removal as the add's own resume
//!  refuses them.
//!  The read is the lock design's own,
//!  so the removal and the add's resume cannot disagree about whose a lock is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! refuseALockThisAddDidNotWrite(commonDir, slot); // or nothing, or the stop
//! ```

use crate::worktree_clone_binding_record::lock_words::{
    LockStanding,
    lock_standing_of,
};
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_managed_lock::lock_and_pointer_readings::read_the_lock;
use crate::worktree_clone_managed_lock::reading_a_bounded_file::Reading;
use crate::worktree_clone_managed_lock::recorded_lock_facts::read_the_recorded_facts;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether a standing lock keeps this removal from starting,
///  and the stop that says so.
/// Why:
///  A lock with no words of the add's is a lock a person set,
///  and the settle would refuse it after every destroy,
///  so the question is asked here where refusing costs nothing.
///  Absence,
///  the add's whole sentence and its cut-short prefix are all the wrapper's own:
///  the settle writes a lost lock again and completes a short one.
///
/// # Throws
/// The lock's own words,
/// the read that refused them,
/// or the binding the lock's sentence is recomposed from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseALockThisAddDidNotWrite(commonDir, slot);
/// ```
pub(super) fn refuse_a_lock_this_add_did_not_write(
    common_dir: &Path,
    slot: &str,
) -> Result<(), RemovalStop> {
    let home: PathBuf = home_folder(common_dir);
    let facts = read_the_recorded_facts(common_dir, slot).map_err(|refusal| {
        return RemovalStop::NoBinding {
            slot: String::from(slot),
            reason: refusal.words(slot, home.as_path()),
        };
    })?;
    let Some(registration) = facts.registration.as_ref() else {
        return Ok(());
    };
    let cause: String = match read_the_lock(registration) {
        Reading::Absent => return Ok(()),
        Reading::Refused(refused) => refused.words(),
        Reading::Bytes(bytes) => match lock_standing_of(bytes.as_slice(), &facts.nonce) {
            LockStanding::Verified
            | LockStanding::CutShort {
                ..
            } => return Ok(()),
            LockStanding::Foreign => {
                let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
                let line: &str = text.lines().next().unwrap_or("");
                if line.is_empty() {
                    String::from(
                        "its file is empty, which is also what a person's `git worktree lock` with no reason leaves",
                    )
                } else {
                    format!("its words are `{line}`")
                }
            }
        },
    };
    return Err(RemovalStop::TheLocksWordsAreNotThisAdds {
        slot: String::from(slot),
        cause,
    });
}

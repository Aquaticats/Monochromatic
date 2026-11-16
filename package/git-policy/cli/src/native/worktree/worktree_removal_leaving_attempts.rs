//! What:
//!  The attempts one leaving may cost:
//!  a slot taken by the one command that makes the attempt,
//!  published before the attempt with an exclusive create or an atomic rename,
//!  and marked once the attempt is made,
//!  so a slot a killed command took but never used is taken over by the next command.
//! Why:
//!  A budget holds only if every attempt is counted by the one who makes it,
//!  at the moment it is made,
//!  in a way two commands cannot both pass:
//!  a number read and then written back lets four commands decide on the same count and then all act
//!  (`Leavings` and the look that fills it),
//!  and a count written after the attempt lets a command killed between the two spend the budget on nothing,
//!  leaving a delete no command queues again.
//!  So a slot is a file, published atomically **before** the act it counts:
//!  - `reserved-<leaving>.<n>` holds the command that will make attempt `n`,
//!    published with an exclusive create so two commands can never take the same number;
//!  - `claimed-<leaving>.<n>` is a reservation whose command was proved gone before it acted,
//!    renamed here by the one command that takes the attempt over
//!    (`renameat2` with `RENAME_NOREPLACE`, so exactly one wins);
//!  - `made-<leaving>.<n>` is published once the attempt was made.
//!  A slot whose owner may still be running is another command's attempt and is left alone;
//!  a slot whose owner is proved gone is taken over and its attempt made,
//!  so a kill between the slot and the act costs the act's lateness and never the delete.
//!  Nothing here reads a number and writes it back:
//!  the number a command gets is the name it published, and the budget is the set of names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = takeAnAttempt({ commonDir, id, owner });   // { n } | undefined
//! if (slot === undefined) return spentWords;
//! await makeTheAttempt();                                  // queue the delete, or ask the service
//! markTheAttemptMade({ commonDir, id, n: slot.n });        // after the act
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced, write_private_file_unsynced};
use crate::service_work::MAX_TRIES;
use crate::worktree_removal_finishes_an_interrupted_destroy::owner_may_be_running;

/// The folder of the common Git directory that holds one file per attempt slot of each leaving.
pub(crate) const ATTEMPTS_FOLDER: &str = "monochromatic-git-policy-cli/leaving-attempts";

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeavingAttempts";

/// The number one leaving's attempts are numbered up to,
/// which is the budget the look promises a person reads of.
const THE_BUDGET: u32 = MAX_TRIES;

/// What:
///  The name of the reservation of attempt `number` of one leaving.
/// Why:
///   The name is what the exclusive create decides on,
///  so one file is one attempt and two commands cannot take the same number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(commonDir, ATTEMPTS_FOLDER, `reserved-${id}.${n}`);
/// ```
fn reserved_path(common_dir: &std::path::Path, id: &str, number: u32) -> std::path::PathBuf {
    return common_dir.join(ATTEMPTS_FOLDER).join(format!("reserved-{id}.{number}"));
}

/// What:
///  The name a reservation takes when a command takes the attempt over.
/// Why:
///   The rename is the claim,
///  and it is published with `RENAME_NOREPLACE`,
///  so of any number of commands that find the same dead reservation exactly one takes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(commonDir, ATTEMPTS_FOLDER, `claimed-${id}.${n}`);
/// ```
fn claimed_path(common_dir: &std::path::Path, id: &str, number: u32) -> std::path::PathBuf {
    return common_dir.join(ATTEMPTS_FOLDER).join(format!("claimed-{id}.{number}"));
}

/// What:
///  The name that says attempt `number` of one leaving was made.
/// Why:
///   The budget counts attempts that were made,
///  and this is the only file that says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(commonDir, ATTEMPTS_FOLDER, `made-${id}.${n}`);
/// ```
fn made_path(common_dir: &std::path::Path, id: &str, number: u32) -> std::path::PathBuf {
    return common_dir.join(ATTEMPTS_FOLDER).join(format!("made-{id}.{number}"));
}

/// What:
///  The owner line of a slot,
///  which is the command that took it.
/// Why:
///   A slot whose command is gone without an attempt is the next command's to make,
///  and a slot whose command may still be running is that command's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = readOwner(path); // "pid:birth", or undefined when the file holds none
/// ```
fn read_owner(path: &std::path::Path) -> Option<String> {
    let bytes: Vec<u8> = std::fs::read(path).ok()?;
    let text: String = String::from_utf8(bytes).ok()?;
    let trimmed: &str = text.trim_end();
    return (!trimmed.is_empty()).then(|| return trimmed.to_owned());
}

/// What:
///  Publish `owner` over a slot the caller has just claimed,
///  and answer whether the slot is now this command's.
/// Why:
///   A claimed slot may hold the owner of the command that died,
///  and two commands can both find it dead:
///  each publishes its own owner and then reads the slot back,
///  and only the one whose owner stands there goes on to make the attempt.
///  The write is the atomic replacing write,
///  so a slot always holds one command's name and never half of two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = await writeOwnerAndVerify(path, owner); // replace, then read back
/// ```
fn write_owner_and_verify(path: &std::path::Path, owner: &str) -> bool {
    if let Err(error) = replace_private_file_unsynced(path, owner.as_bytes()) {
        debug(TAG, format!("the attempt slot {} cannot be taken ({error})", path.display()).as_str());
        return false;
    }
    return read_owner(path).as_deref() == Some(owner);
}

/// What:
///  Rename a dead reservation to its claimed name,
///  refusing a name that already stands,
///  and answer whether this command made the rename.
/// Why:
///   The reservation is renamed away by the first command that takes it,
///  so no other command can take the same attempt,
///  and a claimed name that already stands is another command's attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimed = renameNoReplace(reserved, claimedPath); // false when either name is not as expected
/// ```
fn claim_by_rename(reserved: &std::path::Path, claimed: &std::path::Path) -> bool {
    let Some(directory) = reserved.parent() else { return false };
    let (Some(from), Some(to)) = (reserved.file_name(), claimed.file_name()) else { return false };
    let held: std::fs::File = match crate::fs_beneath::open_root(directory) {
        Ok(opened) => opened,
        Err(error) => {
            debug(TAG, format!("{} cannot be opened, so no attempt is taken over ({error})", directory.display()).as_str());
            return false;
        }
    };
    return crate::fs_beneath::rename_at(&held, from, &held, to, false).is_ok();
}

/// What:
///  Take one attempt slot of this leaving for this command,
///  or answer nothing when every attempt a record may cost was taken or made.
/// Why:
///   The slot is published before the act it counts,
///  so a command killed between the two leaves a slot the next command takes over and makes the attempt for,
///  and a command killed after the act leaves a slot that is already spent.
///  A free number is published exclusively,
///  a dead reservation is claimed by a rename only one command can win,
///  and a dead claim is taken by publishing this command's owner over it and reading it back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = takeAnAttempt({ commonDir, id, owner }); // { n } | undefined
/// ```
pub(crate) fn take_an_attempt(common_dir: &std::path::Path, id: &str, owner: &str) -> Option<u32> {
    let folder: std::path::PathBuf = common_dir.join(ATTEMPTS_FOLDER);
    if let Err(error) = create_private_directory_all(folder.as_path()) {
        warn(TAG, format!("the attempts of {id} cannot be taken, because {} cannot be made ({error})", folder.display()).as_str());
        return None;
    }
    for number in 1..=THE_BUDGET {
        if made_path(common_dir, id, number).is_file() {
            continue;
        }
        let reserved: std::path::PathBuf = reserved_path(common_dir, id, number);
        let claimed: std::path::PathBuf = claimed_path(common_dir, id, number);
        if reserved.is_file() {
            if owner_may_be_running(read_owner(reserved.as_path()).as_ref()) {
                continue;
            }
            if !claim_by_rename(reserved.as_path(), claimed.as_path()) {
                continue;
            }
            if !write_owner_and_verify(claimed.as_path(), owner) {
                continue;
            }
            debug(TAG, format!("the attempt {number} of {id} was left by a command that is gone, so it is taken over").as_str());
            return Some(number);
        }
        if claimed.is_file() {
            if owner_may_be_running(read_owner(claimed.as_path()).as_ref()) {
                continue;
            }
            if !write_owner_and_verify(claimed.as_path(), owner) {
                continue;
            }
            debug(TAG, format!("the attempt {number} of {id} was claimed by a command that is gone, so it is taken over").as_str());
            return Some(number);
        }
        match write_private_file_unsynced(reserved.as_path(), owner.as_bytes()) {
            Ok(()) => return Some(number),
            // Another command took this number between the look and the publish, so the next one is asked for.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                warn(TAG, format!("the attempt {number} of {id} cannot be taken ({error})").as_str());
                return None;
            }
        }
    }
    return None;
}

/// What:
///  Mark the attempt this command took as made,
///  and drop the slot that named it.
/// Why:
///   An attempt is counted by the act it counts,
///  so this is called after the delete is queued or the service is asked,
///  and a slot a command took but never made stays claimable by the next command.
///  A slot that cannot be dropped costs one file and nothing else:
///  the made name is what the budget counts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// markTheAttemptMade({ commonDir, id, n }); // make made-<id>.<n>, drop reserved- and claimed-
/// ```
pub(crate) fn mark_the_attempt_made(common_dir: &std::path::Path, id: &str, number: u32) {
    let made: std::path::PathBuf = made_path(common_dir, id, number);
    match write_private_file_unsynced(made.as_path(), b"") {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            warn(TAG, format!("the made attempt {} of {id} is not recorded ({error})", made.display()).as_str());
        }
    }
    let _dropped_reservation = std::fs::remove_file(reserved_path(common_dir, id, number));
    let _dropped_claim = std::fs::remove_file(claimed_path(common_dir, id, number));
}

/// What:
///  Give back the slot a command took for an act that did not happen,
///  so the attempt it would have made is still the person's to have.
/// Why:
///   A delete that cannot be queued and an ask that cannot be made cost nothing,
///  and a slot spent by an act that failed would shorten the budget by one for no attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// releaseTheAttempt({ commonDir, id, n }); // drop reserved- and claimed-
/// ```
pub(crate) fn release_the_attempt(common_dir: &std::path::Path, id: &str, number: u32) {
    let _dropped_reservation = std::fs::remove_file(reserved_path(common_dir, id, number));
    let _dropped_claim = std::fs::remove_file(claimed_path(common_dir, id, number));
}

/// What:
///  How many of the attempts one leaving may cost are taken,
///  whether they were made or a command that died still holds them.
/// Why:
///   The number is the budget a person reads of,
///  and it is counted from the names three commands can publish, never from a number one of them wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = attemptsTaken(commonDir, id); // made + reserved + claimed
/// ```
pub(crate) fn attempts_taken(common_dir: &std::path::Path, id: &str) -> u32 {
    let mut taken: u32 = 0;
    for number in 1..=THE_BUDGET {
        if made_path(common_dir, id, number).is_file() || reserved_path(common_dir, id, number).is_file() || claimed_path(common_dir, id, number).is_file() {
            taken += 1;
        }
    }
    return taken;
}

/// What:
///  Drop every attempt slot of one leaving,
///  once nothing of this program's stands where it named.
/// Why:
///  The slots are the budget of one leaving,
///  and the next leaving of the same place must start with all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheAttempts(commonDir, id);
/// ```
pub(crate) fn forget_the_attempts(common_dir: &std::path::Path, id: &str) {
    for number in 1..=THE_BUDGET {
        let _made = std::fs::remove_file(made_path(common_dir, id, number));
        let _reserved = std::fs::remove_file(reserved_path(common_dir, id, number));
        let _claimed = std::fs::remove_file(claimed_path(common_dir, id, number));
    }
    let _kept = std::fs::remove_dir(common_dir.join(ATTEMPTS_FOLDER));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_leaving_attempts_tests.rs"]
mod tests;

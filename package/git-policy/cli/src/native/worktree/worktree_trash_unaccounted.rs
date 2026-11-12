//! What:
//!  What every later command in a worktree says about the trash of that worktree:
//! which directory nothing will empty,
//!  and which trash it can record again so the commit service does.
//! Why:
//!   The emptying (`worktree_trash_dispose.rs`) removes what a record names and proves,
//! and a trash it never
//!  reaches is disk a person cannot find without being told;
//! the scan here is the other half,
//! names every trash the
//!  records do not account for,
//! and deletes nothing.
//! It runs on every command in a worktree whose
//!  trash parent is there,
//! so every read it makes is a plain file read with an open that cannot wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! nameUnaccountedTrash({ commonDir, worktree, notices, wake }); // name what nothing empties; record what can be proven
//! ```

use crate::held_push_release_wakes_the_service::WakeTheService;
use crate::plain_file::read_plain_file;
use crate::service_work::work_directory;
use crate::work_item::{CLAIMED, FAILED, PENDING, RECOVERING, WorkItem, WorkKind, state_directory};
use crate::worktree_trash::TRASH_PARENT;
use crate::worktree_trash_dispose::{PendingTrash, RECORD_LIMIT};
use crate::worktree_trash_hand_over::Recorded;
use crate::worktree_trash_manifest::is_trash_name;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  Name every trash in this worktree that nothing will empty,
/// read from the records of the repository's common directory.
/// Why:
///   The add hands its trash to the commit service (`worktree_trash_hand_over.rs`),
/// which empties it after the command returns,
/// so a trash still there is one whose service could not empty it,
/// and nothing else mentions it again.
/// The trash is a full copy of the directories the snapshot left out,
/// so one that stays is disk a person cannot find without being told.
/// A record an earlier release wrote accounts for its trash while the service still works on it,
/// because its own handler empties that one (`empty_trash`);
/// the service settles a trash it cannot empty as failed and never tries it again,
/// so a failed record accounts for nothing here.
/// A record that is still pending after one claim lease accounts for nothing either:
/// a service woken for the repository claims a record within the wake that asked it to,
/// so a pending record older than
///  `CLAIM_LEASE` is one no service ever looked at,
/// and naming it is the only way the person learns of it.
/// This scan deletes nothing:
/// a directory nothing recorded cannot be proven to be one the leave-out moved.
///
/// The record it writes is one no service looks at by itself,
/// so the repository's service is woken for it exactly as a
///  hand-over wakes one:
/// the service waits for wakes and due retries and never for a record to appear,
/// so a record nobody is
///  told about waits until some unrelated wake,
/// which need never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nameUnaccountedTrash(commonDir, worktree, notices, wake); // and wake(commonDir) when a record was written
/// ```
pub fn name_unaccounted_trash(git_dir: &Path, worktree: &Path, notices: &mut dyn Write, wake: &dyn WakeTheService) {
    // A worktree whose trash parent is gone has nothing to name, and that is the ordinary case.
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    if std::fs::symlink_metadata(&parent).is_err() {
        return;
    }
    let work: PathBuf = work_directory(git_dir);
    let mut accounted: Vec<String> = Vec::new();
    for state in [PENDING, CLAIMED, RECOVERING, FAILED] {
        let Ok(listing) = std::fs::read_dir(state_directory(work.as_path(), state)) else { continue };
        for entry in listing.flatten() {
            let record: PathBuf = entry.path();
            // A record is a plain file: a named pipe or a device in a state directory is somebody's own entry, and
            // opening it would wait for a writer that never comes, on every command in this worktree.
            let Ok(bytes) = read_plain_file(record.as_path(), RECORD_LIMIT) else { continue };
            let Ok(text) = String::from_utf8(bytes) else { continue };
            let Ok(item) = WorkItem::decode(text.as_str()) else { continue };
            if item.kind != WorkKind::Trash {
                continue;
            }
            // A record the service still works on accounts for its trash: its own handler empties that one, and naming
            // it here would send the person to look at work that is not theirs. A failed record is settled for good, so
            // it accounts for nothing. A pending record is only the service's work until one claim lease has passed
            // without anything claiming it, which means no service ever looked at it.
            let the_service_owns_it: bool = state == CLAIMED || state == RECOVERING || (state == PENDING && !waited_longer_than_a_claim(record.as_path()));
            if !the_service_owns_it {
                continue;
            }
            accounted.push(item.oid.clone());
        }
    }
    if settle_unrecorded_trash(git_dir, worktree, &accounted, notices) {
        wake.wake(git_dir);
    }
}

/// What:
///  Whether a record's age is past the lease a claim of it would have been renewed within.
/// Why:
///   The lease is the drain's own (`CLAIM_LEASE`),
/// so the age a record is judged stale at and the age the service renews a claim at are one number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPastAClaimLease = (age) => age > CLAIM_LEASE_MS;
/// ```
fn is_past_a_claim_lease(age: std::time::Duration) -> bool {
    return age > crate::service_work::CLAIM_LEASE;
}

/// What:
///  Whether this record has been pending for longer than one claim lease without being claimed.
/// Why:
///   The lease is the one the drain already keeps (`CLAIM_LEASE`),
/// so no new number decides when a record is stale:
/// the service renews a claim every third of it,
/// and a record a woken service would have claimed within the wake that
/// woke it stays pending only when no service ever read it.
/// A record whose age cannot be read is left alone,
/// because a reason to name it
///  that cannot be shown is none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitedLongerThanAClaim = (record) => Date.now() - mtimeMs(record) > CLAIM_LEASE_MS;
/// ```
fn waited_longer_than_a_claim(record: &Path) -> bool {
    let Ok(found) = std::fs::metadata(record) else { return false };
    let Ok(written) = found.modified() else { return false };
    return written.elapsed().is_ok_and(is_past_a_claim_lease);
}

/// What:
///  Record every trash in the trash parent of `worktree` that a record does not account for and that says what it is,
///  so the commit service empties it,
///  name every trash-shaped directory that cannot say it or whose record is settled,
/// which is left where it is,
///  and answer whether a record now waits for the service to be told.
/// Why:
///   A command killed between publishing the snapshot and writing its record leaves a trash nothing names,
/// and the
///  manifest inside the trash is the proof the wrapper made it:
/// the next command in this worktree turns that proof into
///  the record the service drains,
/// so the trash is emptied without a person having to find it.
/// What cannot show a manifest is the
///  person's own directory or a trash of a release before that manifest,
/// and removing it would be a guess,
/// so it is only named,
///  with the advice to look at it.
/// The id of a trash's record is the trash's name,
/// so a record the queue already holds is this trash's record,
/// and the line says
///  which of the three the queue answered:
/// a record this command wrote is reported as recorded now,
/// one that waits without a
///  service having read it as already in the queue,
/// and a record the service settled as failed as a trash nothing will
///  empty,
/// so no command promises an emptying no service will claim.
/// Only a directory named the way the wrapper names a trash is looked at at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settleUnrecordedTrash = ({ gitDir, worktree, accounted }) => { for (const name of readdir(join(worktree, TRASH_PARENT))) { if (accounted.includes(name) || !isTrashName(name)) continue; const manifest = readTrashManifest(worktree, name); if (manifest) recordTheTrash({ destination: worktree, commonDir: gitDir, manifest }); else nameItForThePerson(join(worktree, TRASH_PARENT, name)); } };
/// ```
fn settle_unrecorded_trash(git_dir: &Path, worktree: &Path, accounted: &[String], notices: &mut dyn Write) -> bool {
    let parent: PathBuf = worktree.join(TRASH_PARENT);
    let Ok(listing) = std::fs::read_dir(&parent) else { return false };
    let mut waiting: bool = false;
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if accounted.contains(&name) || !is_trash_name(name.as_str()) {
            continue;
        }
        let Some(manifest) = crate::worktree_trash_own_manifest::read(worktree, name.as_str()) else {
            let _told = writeln!(notices, "{}", left_words(entry.path().as_path(), "there is no record the service still works on, and the trash cannot say what it is"));
            continue;
        };
        let pending: PendingTrash = PendingTrash { destination: worktree.to_path_buf(), common_dir: git_dir.to_path_buf(), manifest, taken_over: 0, quiet: true };
        let said: String = match crate::worktree_trash_hand_over::record_the_trash(&pending) {
            Ok(Recorded::Written) => {
                // A record nobody is told about waits for a wake that need never come, so the writing of one is what the
                //  repository's service is told; a record that was already there tells it nothing new.
                waiting = true;
                format!(
                    "monochromatic-git-policy-cli: {} holds directories a snapshot left out that no record named, and the trash says what it is, so it is recorded now and the commit service empties it, one worktree's trashes at a time.",
                    entry.path().display()
                )
            }
            // The queue's id is the trash's name, so a record it already holds is this trash's record: one that waits is
            //  emptied by the service, and one it gave up on never is; only the second is news, and the words say which.
            Ok(Recorded::There) => format!(
                "monochromatic-git-policy-cli: {} holds directories a snapshot left out, and the trash says what it is, and a record of it is already in the queue, so the commit service empties it when it next runs for this repository, one trash at a time.",
                entry.path().display()
            ),
            Ok(Recorded::GivenUp) => left_words(entry.path().as_path(), "the commit service gave up on the record of this trash and never claims it again"),
            Err(reason) => left_words(entry.path().as_path(), format!("the record of the trash cannot be written: {reason}").as_str()),
        };
        let _told = writeln!(notices, "{said}");
    }
    return waiting;
}

/// What:
///  The line that names a trash nothing will empty,
/// with the reason it stays where it is.
/// Why:
///   Three reasons leave a trash alone:
/// it cannot say the wrapper made it,
/// the record of it cannot be written,
/// and
///  the service gave up on its record;
/// the person reads the same instruction for all three,
/// so the sentence has one shape and
///  only the reason changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftWords = (path, reason) => `... ${path} holds directories a snapshot left out that nothing will empty now (${reason}), so they are left as they are: look at them, and remove them yourself if nothing in them is wanted.`;
/// ```
fn left_words(path: &Path, reason: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} holds directories a snapshot left out that nothing will empty now ({reason}), so they are left as they are: look at them, and remove them yourself if nothing in them is wanted.",
        path.display()
    );
}

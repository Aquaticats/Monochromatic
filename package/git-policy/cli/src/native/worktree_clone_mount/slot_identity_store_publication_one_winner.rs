//! What:
//!  The one call that gives a staging file its record's name,
//!  the loser's answer when two recorders meet on one name,
//!  the fallback for a folder that makes no hard links,
//!  and the refusal a control forces in place of either call.
//! Why:
//!  `linkat` answers `EEXIST` to the second recorder of one name,
//!  which is the whole answer to two sweeps meeting on one slot:
//!  the loser removes its own staging name,
//!  syncs the folder and reads the record that won.
//!  A folder that refuses a hard link publishes with `renameat2` and
//!  `RENAME_NOREPLACE`,
//!  which answers `EEXIST` just the same,
//!  and a folder that refuses both refuses the write rather than publish by a
//!  rename that replaces another run's record.
//!  The call is skipped rather than overridden afterwards,
//!  because a link that ran would already have published the record
//!  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "Two writers with two intents").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! linkOrRename(store, staging, name, staged); // EEXIST becomes the loser's SlotAlreadyRecorded
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::slot_identity_store::SlotIdentityStore;
use super::slot_identity_store_publication_staging_file::{
    finish_the_publish,
    remove_own_staging_entry,
};
use super::slot_identity_store_records::PARENT_RECORD_NAME;
use super::slot_identity_store_refusals::{
    StoreRefusal,
    write_refusal,
};
use crate::publishing_entries_in_one_open_folder::{
    link_entry,
    link_refused_because_the_folder_makes_none,
    rename_entry_never_replacing,
    sync_folder,
};
use crate::rename_that_never_replaces::rename_unsupported;
use std::ffi::OsStr;

// What:     the three hooks the controls use on this file's calls, declared
//           with the rest of the publication's hooks in the module the
//           controls name; a test build only, because this file runs them only
//           in a test build.
// Why:      a test host's folders make hard links and the no-replace rename, so
//           both refusals this file falls back from must be injectable.
//
// In TS you'd write (pseudocode):
// ```ts
// import { BETWEEN_THE_BYTES_AND_THE_LINK, THE_LINK_IS_ANSWERED_WITH, THE_RENAME_IS_ANSWERED_WITH } from './publication.js';
// ```
#[cfg(test)]
use super::slot_identity_store_publication::{
    BETWEEN_THE_BYTES_AND_THE_LINK,
    THE_LINK_IS_ANSWERED_WITH,
    THE_RENAME_IS_ANSWERED_WITH,
};

/// What:
///  Publish the staging file under its record name,
///  by one link,
///  or by the rename that never replaces where the folder refuses a link.
/// Why:
///   The link is the accepted publication;
///  the rename is the fallback a folder without hard links needs,
///  and it answers `EEXIST` the same way,
///  so the one-winner property holds on both.
///  A folder that refuses both refuses the write,
///  and nothing falls back to a rename that replaces,
///  because a publication that may replace another's record is the refresh this layout forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linkOrRename(store, staging, name); // EEXIST becomes the loser's SlotAlreadyRecorded
/// ```
pub(super) fn link_or_rename(
    store: &SlotIdentityStore,
    staging: &OsStr,
    name: &OsStr,
    staged: std::fs::File,
    for_record: &str,
) -> Result<(), StoreRefusal> {
    #[cfg(test)]
    BETWEEN_THE_BYTES_AND_THE_LINK.with(|hook| {
        // Taken out, so a second publication the hook runs does not run it again.
        let held = hook.borrow_mut().take();
        if let Some(between) = held {
            between();
        }
    });
    drop(staged);
    #[cfg(test)]
    let forced_link: Option<i32> = THE_LINK_IS_ANSWERED_WITH.with(|answer| return answer.get());
    #[cfg(not(test))]
    let forced_link: Option<i32> = None;
    let linked: std::io::Result<()> = match a_forced_answer(forced_link) {
        Some(answer) => Err(answer),
        None => link_entry(store.folder(), staging, name),
    };
    return match linked {
        Ok(()) => finish_the_publish(store, staging),
        Err(error) if error.raw_os_error() == Some(libc::EEXIST) => {
            lose_the_race(store, staging, name)
        }
        Err(error) if link_refused_because_the_folder_makes_none(&error) => {
            publish_by_the_rename(store, staging, name, for_record)
        }
        Err(error) => {
            remove_own_staging_entry(store, staging);
            Err(StoreRefusal::Write {
                path: store.path().to_path_buf(),
                reason: format!(
                    "the folder refused the link that gives the record its name ({error}), and nothing was published"
                ),
            })
        }
    };
}

/// What:
///  The refusal a control forces in place of one call's own answer,
///  or nothing in the release executable.
/// Why:
///  A folder on a test host makes hard links and the rename that never replaces,
///  so the two refusals a publication falls back from must be injectable;
///  the call is skipped rather than overridden afterwards,
///  because a link that ran would already have published the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forced = forcedAnswer(); // undefined, or the error the control chose
/// ```
pub(super) fn a_forced_answer(answer: Option<i32>) -> Option<std::io::Error> {
    #[cfg(not(test))]
    let _ = answer;
    return answer.map(|code| return std::io::Error::from_raw_os_error(code));
}

/// The fallback publication of a folder without hard links.
pub(super) fn publish_by_the_rename(
    store: &SlotIdentityStore,
    staging: &OsStr,
    name: &OsStr,
    for_record: &str,
) -> Result<(), StoreRefusal> {
    #[cfg(test)]
    let forced_rename: Option<i32> = THE_RENAME_IS_ANSWERED_WITH.with(|answer| return answer.get());
    #[cfg(not(test))]
    let forced_rename: Option<i32> = None;
    let renamed: std::io::Result<()> = match a_forced_answer(forced_rename) {
        Some(answer) => Err(answer),
        None => rename_entry_never_replacing(store.folder(), staging, name),
    };
    return match renamed {
        Ok(()) => {
            sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error))
        }
        Err(error) if error.raw_os_error() == Some(libc::EEXIST) => {
            lose_the_race(store, staging, name)
        }
        Err(error) => {
            remove_own_staging_entry(store, staging);
            let unsupported: bool = rename_unsupported(&error);
            Err(StoreRefusal::Write {
                path: store.path().join(for_record),
                reason: if unsupported {
                    format!(
                        "the folder refused both the link that gives the record its name and the rename that never replaces ({error}), so nothing was published; the folder makes no hard links and no no-replace rename, and nothing is published by a rename that could replace another run's record"
                    )
                } else {
                    format!(
                        "the folder refused the rename that never replaces ({error}), and nothing was published"
                    )
                },
            })
        }
    };
}

/// What:
///  The loser of two recorders meeting on one name:
///  it removes its own staging name,
///  flushes the folder,
///  reads the record that won,
///  and answers it.
/// Why:
///   The kernel answers exactly one writer,
///  and the winner's record is what every attach compares against,
///  so the loser carries on with it instead of failing the sweep;
///  the flush is what makes the winner's name durable before the loser reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// loseTheRace(store, staging, name); // throws SlotAlreadyRecorded carrying the winner
/// ```
pub(super) fn lose_the_race(
    store: &SlotIdentityStore,
    staging: &OsStr,
    name: &OsStr,
) -> Result<(), StoreRefusal> {
    remove_own_staging_entry(store, staging);
    sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error))?;
    let slot: String = name.to_string_lossy().into_owned();
    // `.parent` is the one other name this step can lose on, and its record holds the parent's identity.
    let winner: CloneFolderIdentity = if slot == PARENT_RECORD_NAME {
        store.parent()?
    } else {
        store.slot(slot.as_str())?
    };
    return Err(StoreRefusal::SlotAlreadyRecorded {
        path: store.path().join(&slot),
        slot,
        identity: winner,
    });
}

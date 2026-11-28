//! What:
//!  The slot identity store's folder listing,
//!  what each record under it answered,
//!  the recorded parent identity against the live one,
//!  and each listed slot's recorded identity against its live folder.
//! Why:
//!  The store is where the route records the folder identities an attach
//!  compares against,
//!  so a record filed under the wrong name,
//!  a name the slot grammar refuses and a folder whose identity moved are three
//!  states a person acts on differently
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `doc/decision/cli-git-zfs-slot-identity-store-format.md`).
//!  Nothing here removes anything:
//!  the two staging forms a killed writer left are plan step 18's removal and
//!  the report reads first
//!  (`crate::doctor_zfs_route::store_staging_removal`).
//!  The store's own descriptor is opened by the caller,
//!  because the route reads it under root control and a control reads a
//!  fixture folder through the grammar's own open,
//!  and this file only names the entries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theStoreAndTheIdentities({ places, layout, store, unread });
//! // { store: { entries, parent, slots }, parentIdentity: { recorded, live }, slots: [{ slot, refusal }] }
//! ```

/// The places the store is named by.
use super::the_route_places::TheRoutePlaces;
/// The unknown a read that failed answers.
use super::the_unknown_a_failed_read_answers::the_unknowns_of;
/// One check's stable name,
/// and the report line a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// What the store's parent record answered,
/// and the live folder the walk held.
use crate::doctor_zfs_route::identity_checks::{
    LiveIdentity,
    ParentIdentityFacts,
    RecordedIdentity,
    SlotIdentityFacts,
};
/// What the store's folder listing answered.
use crate::doctor_zfs_route::store_listing::{
    ListedEntry,
    RecordFacts,
    SlotRecordFacts,
    StoreListingFacts,
};
/// Opening one folder and listing its entries by their names.
use crate::foundation::entries_of_one_open_folder::{
    entry_names,
    open_folder,
};
/// One slot's walk,
/// the request it takes,
/// and the refusal a walk answers.
use crate::worktree_clone_mount::{
    CloneSlotRefusal,
    HoldCloneSlotRequest,
    hold_clone_slot_under_root_control,
    hold_the_worktrees_parent,
};
/// The name the parent's record is filed under,
/// and the rule that tells a staging name from a slot's.
use crate::worktree_clone_mount::{
    PARENT_RECORD_NAME,
    is_known_staging_name,
};
/// The store folder a caller opened,
/// and the refusal of one that could not be opened.
use crate::worktree_clone_mount::{
    SlotIdentityStore,
    StoreRefusal,
};
/// The three names the route's layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
/// The clock and the epoch an entry's age is read against.
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

/// What:
///  The checks that read the store's folder listing.
/// Why:
///  The listing is the set of names every one of them walks,
///  so a folder nobody could list leaves all four unproved,
///  and each answers its own unknown rather than a pass over no entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_CHECKS_OF_THE_LISTING = ['store-entries', 'staging-forms', 'unknown-dot-entry', 'slot-identity'];
/// ```
const THE_CHECKS_OF_THE_LISTING: &[Check] = &[
    Check::StoreEntries,
    Check::StagingForms,
    Check::UnknownDotEntry,
    Check::SlotIdentity,
];

/// What:
///  Everything the store folder answers,
///  in one value.
/// Why:
///  The listing,
///  the parent's record and each slot's walk all come from one folder and one
///  checkout,
///  so reading them together keeps a report from comparing one listing's names
///  with another listing's records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheStoreAndTheIdentities = { store: StoreListingFacts; parentIdentity: ParentIdentityFacts; slots: SlotIdentityFacts[] };
/// ```
pub(crate) struct TheStoreAndTheIdentities {
    /// The folder's listing and what each record under it answered.
    pub store: StoreListingFacts,
    /// The recorded parent identity against the live `.claude/worktrees`.
    pub parent_identity: ParentIdentityFacts,
    /// One value per slot the listing named,
    /// holding the walk's refusal where it refused.
    pub slots: Vec<SlotIdentityFacts>,
}

/// What:
///  What one read of the store is given:
///  where the store and the checkout are,
///  the store folder itself when it opened,
///  and where a failed read's unknowns go.
/// Why:
///  The store is opened by the caller so the route's root-control open and a
///  control's own open reach the same reader,
///  and the sink keeps every unknown in one list in report order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheStoreReadInput = { places; layout; store; unread };
/// ```
pub(crate) struct TheStoreReadInput<'a> {
    /// The places the store folder is named by.
    pub places: &'a TheRoutePlaces,
    /// The three names the route's layout holds,
    /// whose checkout the walks start at.
    pub layout: &'a SweepConfiguration,
    /// The store folder,
    /// or why it could not be opened.
    pub store: Result<&'a SlotIdentityStore, &'a StoreRefusal>,
    /// Where the unknowns of a failed read go.
    pub unread: &'a mut Vec<CheckReport>,
}

/// What:
///  The store's listing,
///  the parent identity and every listed slot's identity.
/// Why:
///  A store that is not there is a machine the route was never set up on,
///  and it holds no record and no slot,
///  while a store this run could not open or list leaves four checks unproved,
///  so the two are told apart before anything is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = theStoreAndTheIdentities({ places, layout, store, unread });
/// ```
pub(crate) fn the_store_and_the_identities(
    input: TheStoreReadInput<'_>
) -> TheStoreAndTheIdentities {
    let parent_identity: ParentIdentityFacts = the_parent_identity(&input);
    let store: Result<&SlotIdentityStore, &StoreRefusal> = input.store;
    let folder: &SlotIdentityStore = match store {
        Ok(held) => held,
        Err(StoreRefusal::Missing {
            ..
        }) => {
            return TheStoreAndTheIdentities {
                store: no_store_at_all(),
                parent_identity,
                slots: Vec::new(),
            };
        }
        Err(refusal) => {
            let why: String = refusal.message();
            input
                .unread
                .extend(the_unknowns_of(THE_CHECKS_OF_THE_LISTING, why.as_str()));
            return TheStoreAndTheIdentities {
                store: StoreListingFacts {
                    entries: Vec::new(),
                    parent: RecordFacts::Absent,
                    slots: Vec::new(),
                },
                parent_identity,
                slots: Vec::new(),
            };
        }
    };
    let entries: Vec<ListedEntry> = match the_listing(input.places) {
        Ok(listed) => listed,
        Err(why) => {
            input
                .unread
                .extend(the_unknowns_of(THE_CHECKS_OF_THE_LISTING, why.as_str()));
            return TheStoreAndTheIdentities {
                store: StoreListingFacts {
                    entries: Vec::new(),
                    parent: RecordFacts::Absent,
                    slots: Vec::new(),
                },
                parent_identity,
                slots: Vec::new(),
            };
        }
    };
    let slots: Vec<SlotRecordFacts> = the_slot_records(entries.as_slice(), folder);
    let identities: Vec<SlotIdentityFacts> = the_slot_identities(&input, entries.as_slice());
    return TheStoreAndTheIdentities {
        store: StoreListingFacts {
            parent: the_parent_record(folder),
            entries,
            slots,
        },
        parent_identity,
        slots: identities,
    };
}

/// What:
///  A store folder that is not there:
///  no entry,
///  no parent record and no slot record.
/// Why:
///  A machine the route was never set up on lists nothing,
///  and a report over it must not claim a record it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const empty = { entries: [], parent: { kind: 'absent' }, slots: [] };
/// ```
fn no_store_at_all() -> StoreListingFacts {
    return StoreListingFacts {
        entries: Vec::new(),
        parent: RecordFacts::Absent,
        slots: Vec::new(),
    };
}

/// What:
///  The recorded parent identity against the live `.claude/worktrees` folder.
/// Why:
///  A record that is not there and one this account could not read are two
///  answers,
///  and the walk's own refusal already names the path component it stopped at,
///  so neither is guessed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pair = theParentIdentity(input); // { recorded, live }
/// ```
fn the_parent_identity(input: &TheStoreReadInput<'_>) -> ParentIdentityFacts {
    let recorded: RecordedIdentity = match input.store {
        Err(refusal) => RecordedIdentity::Unreadable {
            why: refusal.message(),
        },
        Ok(folder) => match folder.parent() {
            Ok(identity) => RecordedIdentity::Read(Box::new(identity)),
            Err(StoreRefusal::NoParentRecord {
                ..
            }) => RecordedIdentity::Absent {
                why: String::from("no parent record is filed"),
            },
            Err(refusal) => RecordedIdentity::Unreadable {
                why: refusal.message(),
            },
        },
    };
    let live: LiveIdentity = match hold_the_worktrees_parent(input.layout.checkout.as_path()) {
        Ok(held) => LiveIdentity::Held(Box::new(held.identity)),
        Err(refusal) => LiveIdentity::Refused(refusal),
    };
    return ParentIdentityFacts {
        recorded,
        live,
    };
}

/// What:
///  Every name the store folder lists,
///  with its age in whole seconds,
///  or the reason the folder could not be listed.
/// Why:
///  The two staging forms are reported with their age and a stranger's dot
///  entry with its own,
///  and the folder's own descriptor is what keeps the listing on the folder a
///  person named.
///  An entry whose own metadata could not be read keeps a name and loses its
///  age,
///  which is what the report says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = theListing(places); // [{ name, modifiedSecondsAgo: 12 }, …]
/// ```
fn the_listing(places: &TheRoutePlaces) -> Result<Vec<ListedEntry>, String> {
    let path: &std::path::Path = places.store.as_path();
    let folder: std::fs::File = open_folder(path).map_err(|error| {
        return format!(
            "the identity store in {} could not be opened for its listing ({error}), so no record and no staging form was proved",
            path.display()
        );
    })?;
    let names: Vec<std::ffi::OsString> = entry_names(&folder).map_err(|error| {
        return format!(
            "the identity store in {} could not be listed ({error}), so no record and no staging form was proved",
            path.display()
        );
    })?;
    let now: u64 = the_seconds_since_the_epoch();
    let mut entries: Vec<ListedEntry> = Vec::new();
    for name in names {
        let text: String = name.to_string_lossy().into_owned();
        let age: Option<u64> = the_age_of(&folder, name.as_os_str(), now);
        entries.push(ListedEntry {
            name: text,
            modified_seconds_ago: age,
        });
    }
    return Ok(entries);
}

/// What:
///  One entry's modification time in whole seconds before now,
///  or nothing when its own metadata could not be read.
/// Why:
///  The report says how long a staging form has stood,
///  and an entry that is gone between the listing and the look loses its age
///  and keeps its name rather than ending the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const age = theAgeOf(folder, 'name', nowSeconds); // 12 | undefined
/// ```
fn the_age_of(
    folder: &std::fs::File,
    name: &std::ffi::OsStr,
    now: u64,
) -> Option<u64> {
    let facts: rustix::fs::Stat =
        rustix::fs::statat(folder, name, rustix::fs::AtFlags::SYMLINK_NOFOLLOW).ok()?;
    let written: u64 = u64::try_from(facts.st_mtime).ok()?;
    // What:     `saturating_sub` answers zero for a time in the future rather than wrapping round, which a
    //           subtraction on unsigned numbers would do.
    // Why:      A clock that stepped back must not report an age of billions of seconds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return now > written ? now - written : 0;
    // ```
    return Some(now.saturating_sub(written));
}

/// What:
///  The seconds since the epoch by this machine's clock.
/// Why:
///  An entry's age is that clock's answer minus the file's own modification
///  time,
///  and nothing here needs a calendar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = Math.floor(Date.now() / 1000);
/// ```
fn the_seconds_since_the_epoch() -> u64 {
    // What:     `duration_since` answers the span from the epoch, or an error for a clock before it.
    // Why:      A clock before 1970 answers zero, which reads as "no age", and no age is what the report says
    //           for a time it cannot place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const now = SystemTime.now().durationSince(UNIX_EPOCH)?.seconds ?? 0;
    // ```
    let since: std::time::Duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    return since.as_secs();
}

/// What:
///  What the parent's record answered,
///  as the listing's own fact.
/// Why:
///  A misfiled `.parent` is one of the store checks' findings,
///  so the record's key is kept beside the identity the parent check reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = theParentRecord(folder); // { kind: 'read', recorded: '.parent' } | …
/// ```
fn the_parent_record(folder: &SlotIdentityStore) -> RecordFacts {
    return match folder.parent() {
        Ok(_) => RecordFacts::Read {
            recorded: String::from(PARENT_RECORD_NAME),
        },
        Err(StoreRefusal::RecordUnderAnotherName {
            recorded,
            ..
        }) => RecordFacts::Misfiled {
            recorded,
        },
        Err(StoreRefusal::NoParentRecord {
            ..
        }) => RecordFacts::Absent,
        Err(refusal) => RecordFacts::Unreadable {
            why: refusal.message(),
        },
    };
}

/// What:
///  One record fact per listed slot name.
/// Why:
///  A record filed under another name is the one state only a read of every
///  record finds,
///  and a staging name and a dot entry are never read as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slots = theSlotRecords(entries, folder); // [{ name, facts }]
/// ```
fn the_slot_records(
    entries: &[ListedEntry],
    folder: &SlotIdentityStore,
) -> Vec<SlotRecordFacts> {
    let mut slots: Vec<SlotRecordFacts> = Vec::new();
    for entry in entries {
        if !is_a_slot_name(entry.name.as_str()) {
            continue;
        }
        slots.push(SlotRecordFacts {
            name: entry.name.clone(),
            facts: the_record_of(folder, entry.name.as_str()),
        });
    }
    return slots;
}

/// What:
///  What one slot's record answered when it was read as its own name.
/// Why:
///  The listing reports a record that parses,
///  one filed under another name and one this account could not read,
///  and the three send a person to different work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = theRecordOf(folder, 'one'); // { kind: 'read', recorded: 'one' } | …
/// ```
fn the_record_of(
    folder: &SlotIdentityStore,
    name: &str,
) -> RecordFacts {
    return match folder.slot(name) {
        Ok(_) => RecordFacts::Read {
            recorded: String::from(name),
        },
        Err(StoreRefusal::RecordUnderAnotherName {
            recorded,
            ..
        }) => RecordFacts::Misfiled {
            recorded,
        },
        Err(StoreRefusal::NoSlotRecord {
            ..
        }) => RecordFacts::Absent,
        Err(refusal) => RecordFacts::Unreadable {
            why: refusal.message(),
        },
    };
}

/// What:
///  Each listed slot's recorded identity against its live folder.
/// Why:
///  A changed identity is never permission to refresh the record,
///  so the report names the field the walk stopped at,
///  and only a walk that held the slot and found every field equal passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identities = theSlotIdentities(input, entries); // [{ slot, refusal? }]
/// ```
fn the_slot_identities(
    input: &TheStoreReadInput<'_>,
    entries: &[ListedEntry],
) -> Vec<SlotIdentityFacts> {
    let mut identities: Vec<SlotIdentityFacts> = Vec::new();
    for entry in entries {
        if !is_a_slot_name(entry.name.as_str()) {
            continue;
        }
        let request: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest {
            checkout: input.layout.checkout.as_path(),
            slot: entry.name.as_str(),
            store: input.places.store.as_path(),
        };
        let refusal: Option<CloneSlotRefusal> = hold_clone_slot_under_root_control(&request).err();
        identities.push(SlotIdentityFacts {
            slot: entry.name.clone(),
            refusal,
        });
    }
    return identities;
}

/// What:
///  Whether a listed name is a slot's own record name.
/// Why:
///  `.parent`,
///  every dot name and both staging forms are names this layout accounts for,
///  and none of them is a slot a walk can be made for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSlot = (name) => name !== '.parent' && !name.startsWith('.') && !isKnownStagingName(name);
/// ```
fn is_a_slot_name(name: &str) -> bool {
    if name == PARENT_RECORD_NAME || name.starts_with('.') {
        return false;
    }
    return !is_known_staging_name(name);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_store_and_the_identities_tests.rs"]
mod tests;

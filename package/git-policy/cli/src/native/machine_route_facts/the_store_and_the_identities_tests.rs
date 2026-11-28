//! What:
//!  The controls of the store folder and the identities it records:
//!  the listing with its ages,
//!  the parent's record against the live folder,
//!  one slot's record against its live folder,
//!  a staging name and a stranger's dot name,
//!  a record filed under another name,
//!  a folder that is not there,
//!  and a folder nobody could list.
//! Why:
//!  The store is what every attach compares against,
//!  and its checks report a misfiled record,
//!  a name the grammar refuses and a folder whose identity moved as three
//!  different findings,
//!  so each answer must be read from a folder a control wrote and never from
//!  this machine's store.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theStoreAndTheIdentities({ places, layout, store, unread });
//! expect(answer.store.parent).toEqual({ kind: 'read', recorded: '.parent' });
//! ```

/// One fixture of the whole route.
use super::super::one_fixtures_places_and_readers::{
    THE_SLOT,
    TheFixturesReaders,
    write_file,
};
/// The read a control judges,
/// and the value it answers.
use super::{
    TheStoreAndTheIdentities,
    TheStoreReadInput,
    the_store_and_the_identities,
};
/// The check a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// What the parent's record answered.
use crate::doctor_zfs_route::identity_checks::RecordedIdentity;
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    CONTAINER_DATASET,
};
/// What the store's listing answered.
use crate::doctor_zfs_route::store_listing::RecordFacts;
/// The store folder a control opened.
use crate::worktree_clone_mount::SlotIdentityStore;
/// The three names a layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
/// The owned paths these controls use.
use std::path::PathBuf;

/// One layout naming the design's datasets at a fixture path.
fn the_layout(fixture: &TheFixturesReaders) -> SweepConfiguration {
    return SweepConfiguration {
        parent_dataset: String::from(CONTAINER_DATASET),
        checkout: fixture.checkout.clone(),
        checkout_dataset: String::from(CHECKOUT_DATASET),
    };
}

/// What:
///  A store the release's writer filled lists its records,
///  reads the parent's record,
///  and walks the live folders the records name.
/// Why:
///  These are the pass an attach needs,
///  and a control that saw only the listing would not prove the walks reach the
///  folder the record names.
///  The slot's walk opens the store under root control,
///  which a fixture's folder cannot answer,
///  so this control shows the walk refusing that term and the guest arm proves
///  the pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answer.parentIdentity.recorded.kind).toBe('read');
/// expect(answer.slots).toEqual([{ slot: 'one', refusal: undefined }]);
/// ```
#[test]
fn a_store_of_the_release_s_writer_answers_the_records_and_the_live_folders() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("store-healthy");
    let layout: SweepConfiguration = the_layout(&fixture);
    let store: SlotIdentityStore = fixture.store().expect("the store opens");
    let mut unread: Vec<CheckReport> = Vec::new();
    let answer: TheStoreAndTheIdentities = the_store_and_the_identities(TheStoreReadInput {
        places: &fixture.places,
        layout: &layout,
        store: Ok(&store),
        unread: &mut unread,
    });
    assert!(unread.is_empty(), "{unread:?}");
    assert_eq!(
        answer.store.parent,
        RecordFacts::Read {
            recorded: String::from(".parent"),
        }
    );
    assert_eq!(answer.store.slots.len(), 1);
    assert_eq!(answer.store.slots[0].name, THE_SLOT);
    assert_eq!(
        answer.store.slots[0].facts,
        RecordFacts::Read {
            recorded: String::from(THE_SLOT),
        }
    );
    assert_eq!(answer.slots.len(), 1);
    assert_eq!(answer.slots[0].slot, THE_SLOT);
    // The slot's walk opens the store under root control, and a fixture's folder belongs to the account running the
    // control, so the walk refuses the folder here; the guest arm, running as root, is what proves the pass.
    assert!(
        matches!(
            answer.slots[0].refusal,
            Some(crate::worktree_clone_mount::CloneSlotRefusal::Store(_))
        ),
        "{:?}",
        answer.slots[0].refusal
    );
    assert!(
        matches!(answer.parent_identity.recorded, RecordedIdentity::Read(_)),
        "{:?}",
        answer.parent_identity.recorded
    );
    let names: Vec<&str> = answer
        .store
        .entries
        .iter()
        .map(|entry| return entry.name.as_str())
        .collect();
    assert!(names.contains(&".parent"), "{names:?}");
    assert!(names.contains(&"one"), "{names:?}");
}

/// What:
///  A staging name and a stranger's dot name are listed with their ages,
///  and neither is read as a slot's record.
/// Why:
///  A staging name is never a record and a stranger's dot entry is reported and
///  left in place,
///  so a listing that read either would report a misfiled record nobody wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answer.store.entries.map((e) => e.name)).toContain('.partial-write');
/// expect(answer.store.slots.map((s) => s.name)).not.toContain('.partial-write');
/// ```
#[test]
fn a_staging_name_and_a_stranger_s_dot_name_are_listed_and_not_read() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("store-staging");
    let staging: PathBuf = fixture
        .places
        .store
        .join(".0123456789abcdef0123456789abcdef");
    write_file(staging.as_path(), "left by a killed writer");
    let stranger: PathBuf = fixture.places.store.join(".partial-write");
    write_file(stranger.as_path(), "a stranger's file");
    let layout: SweepConfiguration = the_layout(&fixture);
    let store: SlotIdentityStore = fixture.store().expect("the store opens");
    let mut unread: Vec<CheckReport> = Vec::new();
    let answer: TheStoreAndTheIdentities = the_store_and_the_identities(TheStoreReadInput {
        places: &fixture.places,
        layout: &layout,
        store: Ok(&store),
        unread: &mut unread,
    });
    let names: Vec<&str> = answer
        .store
        .entries
        .iter()
        .map(|entry| return entry.name.as_str())
        .collect();
    assert!(
        names.contains(&".0123456789abcdef0123456789abcdef"),
        "{names:?}"
    );
    assert!(names.contains(&".partial-write"), "{names:?}");
    let slots: Vec<&str> = answer
        .store
        .slots
        .iter()
        .map(|slot| return slot.name.as_str())
        .collect();
    assert_eq!(slots, vec![THE_SLOT]);
    let ages: Vec<Option<u64>> = answer
        .store
        .entries
        .iter()
        .map(|entry| return entry.modified_seconds_ago)
        .collect();
    assert!(
        ages.iter().all(|age| return age.is_some()),
        "every entry keeps its age: {ages:?}"
    );
}

/// What:
///  A record filed under another name is reported as misfiled,
///  naming the key it carries.
/// Why:
///  A misfiled record is the one state only a read of every record finds,
///  and only the key it carries tells a person which record to move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answer.store.slots.find((s) => s.name === 'two').facts).toEqual({ kind: 'misfiled', recorded: 'one' });
/// ```
#[test]
fn a_record_filed_under_another_name_is_misfiled() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("store-misfiled");
    let recorded: Vec<u8> = std::fs::read(fixture.places.store.join(THE_SLOT)).expect("the record");
    let copy: PathBuf = fixture.places.store.join("two");
    std::fs::write(copy.as_path(), recorded.as_slice()).expect("the copy");
    let layout: SweepConfiguration = the_layout(&fixture);
    let store: SlotIdentityStore = fixture.store().expect("the store opens");
    let mut unread: Vec<CheckReport> = Vec::new();
    let answer: TheStoreAndTheIdentities = the_store_and_the_identities(TheStoreReadInput {
        places: &fixture.places,
        layout: &layout,
        store: Ok(&store),
        unread: &mut unread,
    });
    let misfiled = answer
        .store
        .slots
        .iter()
        .find(|slot| return slot.name == "two")
        .expect("the copy is listed");
    let RecordFacts::Misfiled {
        recorded: carried,
    } = &misfiled.facts
    else {
        panic!("the copy is misfiled: {:?}", misfiled.facts);
    };
    assert_eq!(carried, THE_SLOT);
}

/// What:
///  A store folder that is not there answers no entry,
///  no parent record and no slot,
///  and no check an unknown.
/// Why:
///  A machine the route was never set up on lists nothing,
///  which is a state a person reads as "nothing here" rather than a read that
///  failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theStoreAndTheIdentities({ places, layout, store: Err(Missing), unread })).toMatchObject({ store: { entries: [], parent: { kind: 'absent' }, slots: [] } });
/// ```
#[test]
fn a_store_that_is_not_there_answers_no_entry_and_no_unknown() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("store-absent");
    std::fs::remove_dir_all(fixture.places.store.as_path()).expect("the store goes");
    let layout: SweepConfiguration = the_layout(&fixture);
    let refusal: crate::worktree_clone_mount::StoreRefusal =
        SlotIdentityStore::open_under_root_control(fixture.places.store.as_path())
            .expect_err("a folder that is not there is refused");
    let mut unread: Vec<CheckReport> = Vec::new();
    let answer: TheStoreAndTheIdentities = the_store_and_the_identities(TheStoreReadInput {
        places: &fixture.places,
        layout: &layout,
        store: Err(&refusal),
        unread: &mut unread,
    });
    assert!(answer.store.entries.is_empty());
    assert_eq!(answer.store.parent, RecordFacts::Absent);
    assert!(answer.store.slots.is_empty());
    assert!(answer.slots.is_empty());
    assert!(unread.is_empty(), "{unread:?}");
}

/// What:
///  A store another account could write answers the four checks that read its
///  listing an unknown naming the refusal,
///  and the parent's record an unreadable answer.
/// Why:
///  A folder the account could rewrite decides nothing,
///  so the four checks report unknown rather than the pass an empty listing
///  gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unread.map((r) => r.check)).toEqual(['store-entries', 'staging-forms', 'unknown-dot-entry', 'slot-identity']);
/// ```
#[test]
fn a_store_outside_root_control_answers_four_unknowns() {
    let fixture: TheFixturesReaders = TheFixturesReaders::made("store-not-root");
    let layout: SweepConfiguration = the_layout(&fixture);
    let refusal: crate::worktree_clone_mount::StoreRefusal =
        SlotIdentityStore::open_under_root_control(fixture.places.store.as_path())
            .expect_err("a folder this account owns is not root's");
    let mut unread: Vec<CheckReport> = Vec::new();
    let answer: TheStoreAndTheIdentities = the_store_and_the_identities(TheStoreReadInput {
        places: &fixture.places,
        layout: &layout,
        store: Err(&refusal),
        unread: &mut unread,
    });
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![
            Check::StoreEntries,
            Check::StagingForms,
            Check::UnknownDotEntry,
            Check::SlotIdentity,
        ]
    );
    assert!(
        matches!(
            answer.parent_identity.recorded,
            RecordedIdentity::Unreadable { .. }
        ),
        "{:?}",
        answer.parent_identity.recorded
    );
}

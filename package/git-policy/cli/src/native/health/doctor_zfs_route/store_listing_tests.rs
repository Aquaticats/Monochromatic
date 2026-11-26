//! What:
//!  Controls of the store folder's listing:
//!  the entry count,
//!  a misfiled record,
//!  a foreign entry,
//!  the two known staging forms with their count and age,
//!  and a dot entry that matches neither.
//! Why:
//!  A misfiled record and a foreign name are the two states only a reader of
//!  every name and every record finds,
//!  and a third dot form would prove a writer this layout never accounted for,
//!  so each control writes the names and records it chose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(check_store_entries(withSlots([{ name: 'one', facts: { kind: 'misfiled', recorded: 'two' } }]))).toMatchObject({ field: 'record' });
//! ```

use super::{
    ListedEntry,
    RecordFacts,
    SlotRecordFacts,
    StoreListingFacts,
    check_staging_forms,
    check_store_entries,
    check_unknown_dot_entries,
    is_foreign_entry,
    is_unknown_dot_entry,
    known_staging_entries,
};
use crate::health::doctor_zfs_route::route_test_facts::healthy_store;

/// One entry with an age.
fn entry(
    name: &str,
    age: u64,
) -> ListedEntry {
    return ListedEntry {
        name: String::from(name),
        modified_seconds_ago: Some(age),
    };
}

/// The healthy store with one entry added,
/// and its slot records kept.
fn with_entry(
    name: &str,
    age: u64,
) -> StoreListingFacts {
    let mut store: StoreListingFacts = healthy_store();
    store.entries.push(entry(name, age));
    return store;
}

//region The entry count and the records

/// The healthy store's count names both records and both names.
#[test]
fn the_healthy_store_reports_its_entry_and_record_count() {
    let outcome: super::CheckOutcome = check_store_entries(&healthy_store());
    let super::CheckOutcome::Pass {
        detail,
    } = outcome
    else {
        panic!("the healthy store passes");
    };
    assert!(detail.contains('2'), "{detail}");
    assert!(detail.contains("2 record"), "{detail}");
}

/// A record filed under another name is a finding that names both.
#[test]
fn a_misfiled_record_is_a_finding_naming_both_names() {
    let mut store: StoreListingFacts = healthy_store();
    store.slots = vec![SlotRecordFacts {
        name: String::from("one"),
        facts: RecordFacts::Misfiled {
            recorded: String::from("two"),
        },
    }];
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_store_entries(&store)
    else {
        panic!("a misfiled record is a finding");
    };
    assert_eq!(field, "record");
    assert!(
        detail.contains('`') && detail.contains("one") && detail.contains("two"),
        "{detail}"
    );
}

/// A foreign name is a finding naming the name.
#[test]
fn a_foreign_name_is_a_finding_naming_the_name() {
    let store: StoreListingFacts = with_entry("has space", 5);
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_store_entries(&store)
    else {
        panic!("a foreign name is a finding");
    };
    assert_eq!(field, "name");
    assert!(detail.contains("has space"), "{detail}");
}

/// A record this account could not read leaves the answer unknown.
#[test]
fn an_unreadable_record_is_unknown() {
    let mut store: StoreListingFacts = healthy_store();
    store.slots = vec![SlotRecordFacts {
        name: String::from("one"),
        facts: RecordFacts::Unreadable {
            why: String::from("permission denied"),
        },
    }];
    assert!(matches!(
        check_store_entries(&store),
        super::CheckOutcome::Unknown { .. }
    ));
}

/// A missing parent record is not unknown:
/// the count says so and the parent check sends a person to the adoption.
#[test]
fn a_missing_parent_record_is_counted_rather_than_unknown() {
    let mut store: StoreListingFacts = healthy_store();
    store.parent = RecordFacts::Absent;
    store.entries.retain(|entry| return entry.name != ".parent");
    let super::CheckOutcome::Pass {
        detail,
    } = check_store_entries(&store)
    else {
        panic!("a missing parent record is still a readable listing");
    };
    assert!(detail.contains("no parent record"), "{detail}");
}

//endregion

//region The staging forms

/// A store with no staging form passes.
#[test]
fn a_store_without_staging_passes() {
    assert!(matches!(
        check_staging_forms(&healthy_store()),
        super::CheckOutcome::Pass { .. }
    ));
}

/// Both known forms are one finding with their count and ages.
#[test]
fn both_known_staging_forms_are_one_finding_with_count_and_age() {
    let mut store: StoreListingFacts = healthy_store();
    store
        .entries
        .push(entry(".0123456789abcdef0123456789abcdef", 120));
    store
        .entries
        .push(entry("..parent.0123456789abcdef.partial", 30));
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_staging_forms(&store)
    else {
        panic!("staging forms are a finding");
    };
    assert_eq!(field, "staging");
    assert!(detail.contains('2'), "{detail}");
    assert!(detail.contains("120s old"), "{detail}");
    assert!(detail.contains("30s old"), "{detail}");
}

/// A staging form whose age could not be read says so rather than passing for
/// fresh.
#[test]
fn a_staging_form_without_an_age_says_so() {
    let mut store: StoreListingFacts = healthy_store();
    store.entries.push(ListedEntry {
        name: String::from(".0123456789abcdef0123456789abcdef"),
        modified_seconds_ago: None,
    });
    let super::CheckOutcome::Finding {
        detail,
        ..
    } = check_staging_forms(&store)
    else {
        panic!("a staging form is a finding");
    };
    assert!(detail.contains("age unreadable"), "{detail}");
}

/// Only the two known forms are staging entries.
#[test]
fn only_the_two_known_forms_are_staging_entries() {
    let mut store: StoreListingFacts = healthy_store();
    store
        .entries
        .push(entry(".0123456789abcdef0123456789abcdef", 1));
    store
        .entries
        .push(entry(".ABCDEF0123456789abcdef0123456789", 1));
    store.entries.push(entry("..parent..partial", 1));
    assert_eq!(known_staging_entries(&store).len(), 1);
}

//endregion

//region The dot entry nothing accounts for

/// A third dot form is a finding naming it and is never removed.
#[test]
fn a_third_dot_form_is_a_finding_naming_it() {
    let store: StoreListingFacts = with_entry(".partial-write", 9);
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_unknown_dot_entries(&store)
    else {
        panic!("a third dot form is a finding");
    };
    assert_eq!(field, "name");
    assert!(detail.contains(".partial-write"), "{detail}");
    assert!(detail.contains("left in place"), "{detail}");
}

/// `.parent` and the known forms are not reported as unknown dot entries.
#[test]
fn the_known_dot_names_are_not_unknown() {
    let mut store: StoreListingFacts = healthy_store();
    store
        .entries
        .push(entry(".0123456789abcdef0123456789abcdef", 1));
    assert!(matches!(
        check_unknown_dot_entries(&store),
        super::CheckOutcome::Pass { .. }
    ));
}

/// The dot rule tells a known form from a stranger's name.
#[test]
fn the_dot_rule_tells_a_known_form_from_a_stranger() {
    assert!(is_unknown_dot_entry(".partial-write"));
    assert!(!is_unknown_dot_entry(".parent"));
    assert!(!is_unknown_dot_entry(".0123456789abcdef0123456789abcdef"));
    assert!(!is_unknown_dot_entry("one"));
}

/// The foreign rule accepts a plain slot name and refuses a name the slot
/// grammar refuses.
#[test]
fn the_foreign_rule_follows_the_slot_grammar() {
    assert!(!is_foreign_entry("one"));
    assert!(!is_foreign_entry(".parent"));
    assert!(!is_foreign_entry(".0123456789abcdef0123456789abcdef"));
    assert!(is_foreign_entry("has space"));
    assert!(is_foreign_entry("has/slash"));
}

//endregion

//! What:
//!  Controls of the `snapdir` report:
//!  `disabled` passes on both datasets the step names,
//!  `hidden`,
//!  `visible`,
//!  the default and a value this build does not name each answer their own
//!  state,
//!  the container's line says what the clones inherit,
//!  the command is printed and the module floor is stated,
//!  and a dataset nobody read is unknown rather than a finding.
//! Why:
//!  The setting is the only thing standing between an account and a mount
//!  table it can grow by listing a snapshot directory,
//!  so the words a person meets here must name the value,
//!  the command and the module floor without ever claiming this machine's
//!  module is old enough or new enough.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkSnapdir(containerWith(2))).toMatchObject({ kind: 'pass' });
//! ```

use super::{SNAPDIR_DISABLED, SNAPDIR_HIDDEN, SNAPDIR_PROPERTY, SNAPDIR_VISIBLE, SnapdirSetting, check_snapdir, snapdir_report, snapdir_setting, snapdir_word};
use crate::health::doctor_zfs_route::check_outcome::{Check, CheckOutcome};
use crate::health::doctor_zfs_route::route_test_facts::{CHECKOUT, CHECKOUT_DATASET, CONTAINER_DATASET, DatasetBuilder, dataset};
use crate::health::doctor_zfs_route::zfs_checks::{DatasetFacts, DatasetRole, Reading};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{CloneObjectFacts, PropertyValue};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;
use std::path::PathBuf;

/// The checkout's role,
/// which is the path the design mounts the checkout dataset at.
fn checkout_role() -> DatasetRole {
    return DatasetRole::Checkout { mount_point: PathBuf::from(CHECKOUT) };
}

/// A checkout dataset carrying the properties a control chose,
/// with `snapdir` rewritten to the property value it hands in.
fn checkout_with(snapdir: PropertyValue) -> DatasetFacts {
    return dataset(checkout_role(), with_snapdir(DatasetBuilder::checkout().build(), snapdir));
}

/// The container carrying the properties a control chose.
fn container_with(snapdir: PropertyValue) -> DatasetFacts {
    return dataset(DatasetRole::Container, with_snapdir(DatasetBuilder::container().build(), snapdir));
}

/// One dataset's facts with `snapdir` replaced by the value a control chose.
fn with_snapdir(mut facts: CloneObjectFacts, snapdir: PropertyValue) -> CloneObjectFacts {
    facts.properties = property_with(snapdir);
    return facts;
}

/// A property list holding exactly one `snapdir` pair,
/// in the shape OpenZFS writes a property,
/// or holding nothing for the default.
fn property_with(snapdir: PropertyValue) -> NvList {
    let mut properties: NvList = NvList::new();
    match snapdir {
        PropertyValue::Absent => {}
        PropertyValue::Number(number) => {
            let mut pair: NvList = NvList::new();
            pair.add_u64("value", number);
            pair.add_str("source", "local");
            properties.add_nvlist(SNAPDIR_PROPERTY, pair);
        }
        PropertyValue::Text(text) => {
            let mut pair: NvList = NvList::new();
            pair.add_str("value", text.as_str());
            pair.add_str("source", "local");
            properties.add_nvlist(SNAPDIR_PROPERTY, pair);
        }
        PropertyValue::Unreadable { why } => {
            let mut pair: NvList = NvList::new();
            pair.add_str("source", why.as_str());
            properties.add_nvlist(SNAPDIR_PROPERTY, pair);
        }
    }
    return properties;
}

/// The three words the property's index values stand for,
/// and that a fourth value stands for none.
#[test]
fn each_index_value_is_one_word() {
    assert_eq!(snapdir_word(SNAPDIR_HIDDEN), Some("hidden"));
    assert_eq!(snapdir_word(SNAPDIR_VISIBLE), Some("visible"));
    assert_eq!(snapdir_word(SNAPDIR_DISABLED), Some("disabled"));
    assert_eq!(snapdir_word(3), None);
}

/// A reply that leaves the property out is the default,
/// not an unread value.
#[test]
fn an_absent_property_is_the_default() {
    assert_eq!(snapdir_setting(&property_with(PropertyValue::Absent)), SnapdirSetting::Default);
}

/// A number is read as its word,
/// and a text value is read as the word it carries.
#[test]
fn a_number_and_its_word_read_the_same() {
    assert_eq!(snapdir_setting(&property_with(PropertyValue::Number(SNAPDIR_DISABLED))), SnapdirSetting::Word("disabled"));
    assert_eq!(snapdir_setting(&property_with(PropertyValue::Text(String::from("disabled")))), SnapdirSetting::Word("disabled"));
}

/// A value this build does not name is its own state,
/// never guessed into a word.
#[test]
fn an_unnamed_value_is_not_a_word() {
    assert_eq!(snapdir_setting(&property_with(PropertyValue::Number(7))), SnapdirSetting::UnknownValue(String::from("7")));
    assert_eq!(snapdir_setting(&property_with(PropertyValue::Text(String::from("sometimes")))), SnapdirSetting::UnknownValue(String::from("sometimes")));
}

/// A property pair this reader cannot read leaves the setting unproved.
#[test]
fn an_unreadable_pair_leaves_the_setting_unproved() {
    assert!(matches!(snapdir_setting(&property_with(PropertyValue::Unreadable { why: String::from("no value") })), SnapdirSetting::Unreadable { .. }));
}

/// `disabled` on the checkout passes and names the dataset.
#[test]
fn a_disabled_checkout_passes() {
    let answer: CheckOutcome = check_snapdir(&checkout_with(PropertyValue::Number(SNAPDIR_DISABLED)));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a disabled checkout passes: {answer:?}");
    };
    assert!(detail.contains(CHECKOUT_DATASET), "{detail}");
}

/// `hidden` is a finding naming the word,
/// the command and the module floor.
#[test]
fn a_hidden_checkout_is_a_finding_with_the_command() {
    let answer: CheckOutcome = check_snapdir(&checkout_with(PropertyValue::Number(SNAPDIR_HIDDEN)));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a hidden checkout is a finding: {answer:?}");
    };
    assert_eq!(field, "snapdir");
    assert!(detail.contains("`hidden`"), "{detail}");
    assert!(detail.contains(format!("zfs set snapdir=disabled {CHECKOUT_DATASET}").as_str()), "{detail}");
    assert!(detail.contains("OpenZFS 2.3.0"), "{detail}");
    assert!(detail.contains("cannot be made at all"), "{detail}");
}

/// The default the reply leaves out is reported as `hidden`,
/// which is the state a person sees in `zfs get`.
#[test]
fn the_default_is_reported_as_hidden() {
    let answer: CheckOutcome = check_snapdir(&checkout_with(PropertyValue::Absent));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("the default is a finding: {answer:?}");
    };
    assert_eq!(field, "snapdir");
    assert!(detail.contains("unset"), "{detail}");
    assert!(detail.contains("default `hidden`"), "{detail}");
}

/// `visible` is a finding naming that word.
#[test]
fn a_visible_checkout_is_a_finding_naming_visible() {
    let answer: CheckOutcome = check_snapdir(&checkout_with(PropertyValue::Number(SNAPDIR_VISIBLE)));
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a visible checkout is a finding: {answer:?}");
    };
    assert_eq!(field, "snapdir");
    assert!(detail.contains("`visible`"), "{detail}");
}

/// The container's finding says that every clone inherits its value.
#[test]
fn the_containers_line_says_what_the_clones_inherit() {
    let answer: CheckOutcome = check_snapdir(&container_with(PropertyValue::Absent));
    let CheckOutcome::Finding { detail, .. } = answer else {
        panic!("an unset container is a finding: {answer:?}");
    };
    assert!(detail.contains("every clone below it inherits"), "{detail}");
    assert!(detail.contains(format!("zfs set snapdir=disabled {CONTAINER_DATASET}").as_str()), "{detail}");
}

/// The container's pass says that every clone inherits its value too,
/// so a person reading only the passing line still learns it covers the
/// clones.
#[test]
fn the_containers_pass_names_the_inheritance() {
    let answer: CheckOutcome = check_snapdir(&container_with(PropertyValue::Number(SNAPDIR_DISABLED)));
    let CheckOutcome::Pass { detail } = answer else {
        panic!("a disabled container passes: {answer:?}");
    };
    assert!(detail.contains("every clone below it inherits"), "{detail}");
}

/// A value this build does not name is unknown rather than a finding about a
/// word nobody read.
#[test]
fn an_unnamed_value_is_unknown() {
    assert!(matches!(check_snapdir(&checkout_with(PropertyValue::Number(9))), CheckOutcome::Unknown { .. }));
}

/// A dataset whose reply could not be read is unknown.
#[test]
fn an_unreadable_dataset_is_unknown() {
    let facts: DatasetFacts = DatasetFacts { role: checkout_role(), name: String::from(CHECKOUT_DATASET), reading: Reading::Unreadable { why: String::from("permission denied") } };
    assert!(matches!(check_snapdir(&facts), CheckOutcome::Unknown { .. }));
}

/// A dataset that is gone is a finding naming the dataset,
/// because no value can stand on it either way.
#[test]
fn a_gone_dataset_is_a_finding() {
    let facts: DatasetFacts = DatasetFacts { role: DatasetRole::Container, name: String::from(CONTAINER_DATASET), reading: Reading::Missing { why: String::from("dataset does not exist") } };
    let CheckOutcome::Finding { field, .. } = check_snapdir(&facts) else {
        panic!("a gone container is a finding");
    };
    assert_eq!(field, "dataset");
}

/// The role decides the check's name,
/// and a clone has no line of its own because it inherits the container's.
#[test]
fn the_role_decides_the_checks_name() {
    let checkout = snapdir_report(&checkout_with(PropertyValue::Number(SNAPDIR_DISABLED))).expect("the checkout reports");
    assert_eq!(checkout.check, Check::CheckoutSnapdir);
    assert_eq!(checkout.check.name(), "checkout-snapdir");
    let container = snapdir_report(&container_with(PropertyValue::Number(SNAPDIR_DISABLED))).expect("the container reports");
    assert_eq!(container.check, Check::ContainerSnapdir);
    assert_eq!(container.check.name(), "container-snapdir");
    let clone: DatasetFacts = dataset(DatasetRole::Clone, DatasetBuilder::clone().build());
    assert!(snapdir_report(&clone).is_none());
}

/// A clone reported directly names the container its value comes from,
/// so a caller that hands one in reads where the setting belongs rather than a
/// sentence about the checkout.
#[test]
fn a_clone_names_the_container_its_value_comes_from() {
    let facts: DatasetFacts = dataset(DatasetRole::Clone, with_snapdir(DatasetBuilder::clone().build(), PropertyValue::Absent));
    let answer: CheckOutcome = check_snapdir(&facts);
    let CheckOutcome::Finding { detail, .. } = answer else {
        panic!("a clone at the default is a finding: {answer:?}");
    };
    assert!(detail.contains("inherits the container's value"), "{detail}");
}

/// The healthy route's own datasets carry the decided value,
/// so the builder and this check agree on what a healthy dataset is.
#[test]
fn the_healthy_datasets_carry_disabled() {
    for facts in crate::health::doctor_zfs_route::route_test_facts::healthy_datasets() {
        if matches!(facts.role, DatasetRole::Clone) {
            continue;
        }
        let CheckOutcome::Pass { detail } = check_snapdir(&facts) else {
            panic!("a healthy dataset passes the snapdir check: {:?}", check_snapdir(&facts));
        };
        assert!(detail.contains("`disabled`"), "{detail}");
    }
}

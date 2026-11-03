//! The unit tests of the release a `cargo update --precise` names.

use crate::lockfile::LockedVersion;
use crate::precise_target::precise_targets;

fn locked(name: &str, version: &str) -> LockedVersion {
    return LockedVersion { name: name.to_string(), version: version.to_string() };
}

fn words(text: &str) -> Vec<String> {
    return text.split_whitespace().map(str::to_string).collect();
}

#[test]
fn every_spelling_of_the_package_option_names_the_same_entry() {
    let entries = [locked("dep", "1.0.1"), locked("other", "1.0.1"), locked("dep", "2.0.0")];
    for spelling in ["update -p dep --precise 1.0.1", "update --package dep --precise=1.0.1", "update -pdep --precise 1.0.1", "update -p=dep@1.0.1 --precise 1.0.1", "update --package=dep --precise 1.0.1"] {
        assert_eq!(precise_targets(&words(spelling), &entries), vec![locked("dep", "1.0.1")], "{spelling}");
    }
}

#[test]
fn a_source_url_spec_names_the_package_after_the_hash() {
    let entries = [locked("dep", "1.0.1")];
    let args = words("update --package https://example.invalid/index#dep@1.0.1 --precise 1.0.1");

    assert_eq!(precise_targets(&args, &entries), vec![locked("dep", "1.0.1")]);
}

#[test]
fn arguments_without_a_precise_release_name_nothing() {
    let entries = [locked("dep", "1.0.1")];

    assert!(precise_targets(&words("update -p dep"), &entries).is_empty());
    assert!(precise_targets(&words("update --precise"), &entries).is_empty());
}

#[test]
fn a_precise_release_that_is_not_locked_names_nothing() {
    let entries = [locked("dep", "1.0.0")];

    assert!(precise_targets(&words("update -p dep --precise 1.0.1"), &entries).is_empty());
}

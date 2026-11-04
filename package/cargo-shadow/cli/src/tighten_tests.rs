//! The unit tests of the catalog floor raise that `cargo update --breaking` performs,
//! as `pnpm update --latest` writes the new
//! range to the catalog entry.

use std::collections::BTreeSet;

use crate::lockfile::LockedVersion;
use crate::tighten::{raise_floors, raised_names};

fn locked(name: &str, version: &str) -> LockedVersion {
    return LockedVersion { name: name.to_string(), version: version.to_string() };
}

fn names(items: &[&str]) -> BTreeSet<String> {
    return items.iter().map(|item| { return item.to_string(); }).collect();
}

const CATALOG: &str = "catalogPrune = true\n\n[catalog]\nserde = \">=1.0.200\" # keep\nclap = \">=4.6.7\"\npinned = \"=1.2.3\"\n\n[catalogs.windows]\nserde = \">=0.1.0\"\n";

#[test]
fn a_floor_is_raised_to_the_highest_locked_version_and_its_comment_is_kept() {
    let (source, changes) = raise_floors(CATALOG, &[locked("serde", "1.0.210"), locked("serde", "1.0.205")], &names(&["serde"])).expect("raises");

    assert!(source.contains("serde = \">=1.0.210\" # keep"), "{source}");
    assert_eq!(changes, vec!["serde: >=1.0.200 to >=1.0.210".to_string()]);
}

#[test]
fn only_changed_names_default_catalog_floors_and_higher_versions_are_touched() {
    let all = [locked("serde", "1.0.210"), locked("clap", "4.6.0"), locked("pinned", "9.9.9")];

    let (source, changes) = raise_floors(CATALOG, &all, &names(&["clap", "pinned", "absent"])).expect("raises");

    assert_eq!(source, CATALOG, "a lower locked version, an exact requirement and an unknown name change nothing");
    assert!(changes.is_empty());
    assert!(source.contains("[catalogs.windows]\nserde = \">=0.1.0\""));
}

#[test]
fn the_named_catalog_is_never_touched() {
    let (source, _) = raise_floors(CATALOG, &[locked("serde", "1.0.210")], &names(&["serde"])).expect("raises");

    assert!(source.contains("[catalogs.windows]\nserde = \">=0.1.0\""));
}

#[test]
fn a_name_is_raised_when_its_highest_locked_version_rose() {
    let before = [locked("a", "1.0.0"), locked("b", "2.0.0")];
    let after = [locked("a", "1.1.0"), locked("b", "2.0.0"), locked("c", "0.1.0")];

    assert_eq!(raised_names(&before, &after), names(&["a", "c"]));
}

#[test]
fn text_that_is_not_toml_is_an_error() {
    assert!(raise_floors("[catalog\n", &[], &names(&[])).is_err());
}

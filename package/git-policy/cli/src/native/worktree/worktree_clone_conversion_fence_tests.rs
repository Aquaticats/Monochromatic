//! Controls for the fence's reads that need no Git:
//! which paths name an attributes file,
//! and which untracked attributes file above a tracked path refuses the rung.

use super::*;
use crate::subvolume_index::StoredStat;
use crate::test_support::{fixture, remove};
use crate::worktree_clone_conversion_rules::{AutoCrlf, EolAttribute, EolSetting, TextAttribute};

/// A tracked entry of one path,
/// with the stat fields a clean fixture writes.
fn entry(path: &str, size: u32) -> StatEntry {
    return StatEntry { path: path.as_bytes().to_vec(), flags: 0, mode: 0o100644, stored: StoredStat { ctime_seconds: 1, ctime_nanos: 0, mtime_seconds: 1, mtime_nanos: 0, ino: 1, uid: 0, gid: 0, size } };
}

/// The same entry, as a symbolic link.
fn link_entry(path: &str) -> StatEntry {
    return StatEntry { mode: 0o120000, ..entry(path, 11) };
}

/// The attributes Git answered for these paths.
fn attributes_map(named: &[(&str, PathAttributes)]) -> HashMap<Vec<u8>, PathAttributes> {
    return named.iter().map(|(path, held)| return (path.as_bytes().to_vec(), held.clone())).collect();
}

/// The attributes of a path whose own rule may add carriage returns.
fn converting() -> PathAttributes {
    return PathAttributes { text: TextAttribute::Set, eol: EolAttribute::Crlf, ..PathAttributes::default() };
}

/// The configuration this host's clean fixtures run under.
fn no_conversion() -> ConversionRules {
    return ConversionRules { auto_crlf: AutoCrlf::False, core_eol: EolSetting::Unset };
}

#[test]
fn the_decline_count_reads_the_attributes_and_never_counts_a_link() {
    let entries = vec![entry("a.txt", 18), entry("b.txt", 18), link_entry("link.txt"), entry("plain.txt", 18)];
    let attributes = attributes_map(&[("a.txt", converting()), ("b.txt", converting()), ("link.txt", converting())]);
    // `plain.txt` has no attribute and no rule, and a link is written as its target whatever its attributes say.
    assert_eq!(converting_paths(entries.as_slice(), &attributes, &no_conversion()), 2);
}

#[test]
fn the_decline_rule_needs_more_than_the_floor_and_more_than_a_quarter() {
    // The review's six boundary cases, derived from the two constants the rule is written with rather than repeated:
    // at the floor the rule never fires, exactly a quarter does not decline, and one path more does.
    let at_floor: usize = DECLINE_PATHS_FLOOR;
    let over_floor: usize = DECLINE_PATHS_FLOOR + 1;
    let quarter: usize = over_floor * DECLINE_SHARE_DIVISOR;
    assert!(!declines(at_floor, at_floor + DECLINE_SHARE_DIVISOR), "a thousand converting paths never cross the floor");
    assert!(declines(over_floor, over_floor + DECLINE_SHARE_DIVISOR), "one path over the floor and over a quarter declines");
    assert!(!declines(over_floor, quarter), "exactly a quarter does not decline: the comparison is strict");
    assert!(declines(over_floor, quarter - 1), "one path more than a quarter declines");
    assert!(!declines(at_floor / 25, at_floor / 25), "a small repository keeps its rung whatever its share");
    assert!(!declines(0, 0), "an empty source has no converting path");
}

#[test]
fn only_the_two_spellings_of_an_attributes_path_are_attributes_paths() {
    assert!(is_attributes_path(b".gitattributes"));
    assert!(is_attributes_path(b"sub/.gitattributes"));
    for path in [b"gitattributes".as_slice(), b"sub/x.gitattributes", b".gitattributes.bak", b"sub/.gitattributes/x"] {
        assert!(!is_attributes_path(path), "{}", String::from_utf8_lossy(path));
    }
}

#[test]
fn a_tracked_attributes_file_refuses_nothing() {
    let root = fixture("fence-tracked-attributes");
    std::fs::write(root.join(".gitattributes"), "* text=auto eol=lf\n").expect("attributes");
    let entries = vec![entry(".gitattributes", 20), entry("a.txt", 18)];
    assert_eq!(first_untracked_attributes(entries.as_slice(), root.as_path()), None);
    remove(root.as_path());
}

#[test]
fn an_untracked_attributes_file_above_a_tracked_path_refuses_the_rung() {
    let root = fixture("fence-untracked-attributes");
    std::fs::create_dir_all(root.join("sub")).expect("directory");
    std::fs::write(root.join("sub/.gitattributes"), "*.txt ident\n").expect("attributes");
    let entries = vec![entry("sub/a.txt", 18), entry("b.txt", 18)];
    let reason = first_untracked_attributes(entries.as_slice(), root.as_path()).expect("a refusal");
    assert!(reason.starts_with("sub/.gitattributes is in the source and not in its index"), "{reason}");
    remove(root.as_path());
}

#[test]
fn an_attributes_file_beside_paths_no_entry_holds_refuses_nothing() {
    let root = fixture("fence-unrelated-attributes");
    std::fs::create_dir_all(root.join("untracked")).expect("directory");
    std::fs::write(root.join("untracked/.gitattributes"), "*.txt ident\n").expect("attributes");
    let entries = vec![entry("a.txt", 18)];
    assert_eq!(first_untracked_attributes(entries.as_slice(), root.as_path()), None);
    remove(root.as_path());
}

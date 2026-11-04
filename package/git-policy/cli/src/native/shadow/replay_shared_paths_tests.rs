//! Controls for shared-path listing:
//!  raw record parsing, the regular-file predicates, and the listing against real repositories.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// An entry with a mode and object of the given text.
fn entry(mode: &str, oid: &str) -> Option<TreeEntry> {
    return Some(TreeEntry {
        mode: String::from(mode),
        oid: String::from(oid),
    });
}

/// Raw records keep the absent side as `None` and the path bytes as written.
#[test]
fn raw_records_parse_with_absent_sides() {
    let changes: Vec<RawChange> = parse_raw_changes(
        ":000000 100644 0000 1111 A\0added.txt\0:100644 000000 2222 0000 D\0gone.txt\0:100644 100755 3333 4444 M\0mode.txt\0",
    )
    .expect("records");
    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0].from, None);
    assert_eq!(changes[0].to, entry("100644", "1111"));
    assert_eq!(changes[1].to, None);
    assert_eq!(changes[2].path, "mode.txt");
    assert_eq!(changes[2].from, entry("100644", "3333"));
    assert_eq!(changes[2].to, entry("100755", "4444"));
}

/// A header without its path, or without five fields, fails.
#[test]
fn malformed_raw_records_fail() {
    assert!(parse_raw_changes(":100644 100644 a b M").is_err());
    assert!(parse_raw_changes(":100644 100644 a M\0x\0").is_err());
    assert!(parse_raw_changes("").expect("empty").is_empty());
}

/// Only `100644` and `100755` are regular files; a symbolic link or a gitlink never is.
#[test]
fn regular_file_modes_are_the_only_regular_files() {
    assert!(is_regular_mode("100644"));
    assert!(is_regular_mode("100755"));
    assert!(!is_regular_mode("120000"));
    assert!(!is_regular_mode("160000"));
}

/// Regular triples need all three present; a text candidate needs landed and prepared present and regular,
///  and the base regular or absent.
#[test]
fn the_predicates_follow_the_entries() {
    let regular = |oid: &str| { return entry("100644", oid); };
    let all = SharedPath {
        path: String::from("p"),
        base: regular("a"),
        landed: regular("b"),
        prepared: regular("c"),
    };
    assert!(is_regular_triple(&all));
    assert!(is_text_candidate(&all));
    let added_both: SharedPath = SharedPath {
        path: String::from("p"),
        base: None,
        landed: regular("b"),
        prepared: regular("c"),
    };
    assert!(!is_regular_triple(&added_both));
    assert!(is_text_candidate(&added_both));
    let link: SharedPath = SharedPath {
        path: String::from("p"),
        base: regular("a"),
        landed: entry("120000", "b"),
        prepared: regular("c"),
    };
    assert!(!is_text_candidate(&link));
    let deleted: SharedPath = SharedPath {
        path: String::from("p"),
        base: regular("a"),
        landed: None,
        prepared: regular("c"),
    };
    assert!(!is_text_candidate(&deleted));
    assert!(same_entry(&None, &None));
    assert!(!same_entry(&None, &regular("a")));
}

/// Listing a repository: the landed listing's order, the paths both sides changed,
///  and each side's entry, with deletions and absent base entries.
#[test]
fn shared_paths_list_the_paths_both_sides_changed() {
    let root: PathBuf = fixture("replay-shared-paths");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("d.txt"), "d\n").expect("d");
    std::fs::write(repo.join("f.txt"), "f\n").expect("f");
    std::fs::write(repo.join("c.txt"), "c\n").expect("c");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join("d.txt"), "prepared d\n").expect("prepared d");
    std::fs::write(repo.join("f.txt"), "prepared f\n").expect("prepared f");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=prepared"]);
    let prepared: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("d.txt"), "landed d\n").expect("landed d");
    std::fs::remove_file(repo.join("f.txt")).expect("remove f");
    std::fs::write(repo.join("e.txt"), "e\n").expect("e");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let current: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let shared: Vec<SharedPath> = list_shared_paths(
        &git_context(),
        repo.join(".git").as_path(),
        repo.as_path(),
        base.as_str(),
        current.as_str(),
        prepared.as_str(),
    )
    .expect("shared paths");
    let paths: Vec<&str> = shared.iter().map(|entry| { return entry.path.as_str(); }).collect();
    assert_eq!(paths, vec!["d.txt", "f.txt"]);
    assert!(shared[0].base.is_some() && shared[0].landed.is_some() && shared[0].prepared.is_some());
    assert_eq!(shared[1].landed, None);
    assert!(shared[1].prepared.is_some());
    remove(root.as_path());
}

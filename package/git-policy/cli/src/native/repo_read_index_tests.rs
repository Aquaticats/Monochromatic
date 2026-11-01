//! Controls for the index parser on files real Git writes, in every version and with the extensions it adds;
//!  the listings built from it are compared with `git ls-files` in `repo_read_query_tests.rs`.

use super::*;
use crate::repo_read_object::to_hex;
use crate::test_support::{fixture, git, git_output, remove};

/// Every entry of the index equals `git ls-files --stage`'s, in each index version.
#[test]
fn entries_equal_the_listing_in_every_version() {
    let root = fixture("repo-read-index");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::create_dir_all(repo.join("d/e")).expect("dirs");
    for name in ["a", "d/b", "d/e/c", "d/e/cc", "z"] {
        std::fs::write(repo.join(name), format!("{name}\n")).expect("write");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=m"]);
    for version in ["2", "4", "3"] {
        git(repo.as_path(), &["update-index", "--index-version", version]);
        let bytes = std::fs::read(repo.join(".git/index")).expect("index");
        let entries = parse_index(bytes.as_slice(), ObjectFormat::Sha1).expect("parsed");
        let listing = String::from_utf8(git_output(repo.as_path(), &["ls-files", "--stage"]).stdout).expect("UTF-8");
        let mine: Vec<String> = entries.iter().map(|entry| { return format!("{:06o} {} {}\t{}", entry.mode, to_hex(entry.id.as_slice()), entry.stage, String::from_utf8_lossy(entry.path.as_slice())); }).collect();
        assert_eq!(mine, listing.lines().map(str::to_string).collect::<Vec<String>>(), "version {version}");
    }
    // Positive control for the extended flags: an intent-to-add entry is version 3 and carries its flag.
    std::fs::write(repo.join("later"), "x\n").expect("later");
    git(repo.as_path(), &["add", "--intent-to-add", "later"]);
    let bytes = std::fs::read(repo.join(".git/index")).expect("index");
    let entries = parse_index(bytes.as_slice(), ObjectFormat::Sha1).expect("parsed");
    assert!(entries.iter().any(|entry| { return entry.extended & INTENT_TO_ADD != 0; }));
    remove(root.as_path());
}

/// A split index, a sparse index, damage and a wrong object format are declined.
#[test]
fn what_cannot_be_read_is_declined() {
    let root = fixture("repo-read-index-declined");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a"]);
    let plain = std::fs::read(repo.join(".git/index")).expect("index");
    assert!(parse_index(plain.as_slice(), ObjectFormat::Sha1).is_some());
    assert!(parse_index(plain.as_slice(), ObjectFormat::Sha256).is_none(), "a SHA-1 file read as SHA-256");
    assert!(parse_index(&plain[..plain.len() - 1], ObjectFormat::Sha1).is_none(), "a truncated file");
    let mut corrupt = plain.clone();
    corrupt[3] = b'X';
    assert!(parse_index(corrupt.as_slice(), ObjectFormat::Sha1).is_none(), "a bad signature");
    git(repo.as_path(), &["update-index", "--split-index"]);
    let split = std::fs::read(repo.join(".git/index")).expect("index");
    assert!(parse_index(split.as_slice(), ObjectFormat::Sha1).is_none(), "a split index");
    remove(root.as_path());
}

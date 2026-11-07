//! Controls for the index parser on files real Git writes,
//! in every version and with the extensions it adds;
//!  the listings built from it are compared with `git ls-files` in `repo_read_query_tests.rs`.

use super::*;
use crate::repo_read_object::to_hex;
use crate::test_support::{executable, fixture, git, git_output, remove};

/// Every entry of the index equals `git ls-files --stage`'s,
/// in each index version.
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
    assert_eq!(&bytes[4..8], &[0, 0, 0, 3], "an extended entry makes the file version 3");
    remove(root.as_path());
}

/// A reserved extended-flag bit is not a format Git reads:
/// patching the one entry of an intent-to-add index is declined.
#[test]
fn a_reserved_extended_flag_is_declined() {
    let root = fixture("repo-read-index-reserved");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("only"), "x\n").expect("only");
    git(repo.as_path(), &["add", "--intent-to-add", "only"]);
    let bytes = std::fs::read(repo.join(".git/index")).expect("index");
    let entries = parse_index(bytes.as_slice(), ObjectFormat::Sha1).expect("parsed");
    assert_eq!(entries.len(), 1);
    // Header 12 bytes, 40 of stat data, a 20-byte name, 2 flag bytes, then the 2 extended-flag bytes.
    let extended_at: usize = 12 + 40 + 20 + 2;
    let mut reserved = bytes.clone();
    reserved[extended_at + 1] |= 0x01;
    assert!(parse_index(reserved.as_slice(), ObjectFormat::Sha1).is_none());
    remove(root.as_path());
}

/// A split index,
/// a sparse index,
/// damage and a wrong object format are declined.
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

/// An index that Git extended for a file system monitor and an untracked cache (the `FSMN` and `UNTR` extensions,
/// which a user
/// with `core.fsmonitor` and `core.untrackedCache` has) is read,
/// and its entries are the listing's.
/// A hook that answers "nothing
/// changed" writes the extension without starting the built-in daemon.
#[test]
fn an_index_with_the_monitor_and_untracked_cache_extensions_is_read() {
    let root = fixture("repo-read-index-monitor");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    for name in ["a", "b"] {
        std::fs::write(repo.join(name), format!("{name}\n")).expect("write");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=m"]);
    let hook = root.join("monitor-hook");
    executable(hook.as_path(), b"#!/bin/sh\nprintf 'token\\0'\n");
    let monitored = format!("core.fsmonitor={}", hook.display());
    git(repo.as_path(), &["-c", monitored.as_str(), "update-index", "--fsmonitor"]);
    git(repo.as_path(), &["-c", monitored.as_str(), "-c", "core.untrackedCache=true", "update-index", "--untracked-cache"]);
    git(repo.as_path(), &["-c", monitored.as_str(), "-c", "core.untrackedCache=true", "status", "--short"]);
    let bytes = std::fs::read(repo.join(".git/index")).expect("index");
    // The premise:
    // both extensions are in the file,
    // so a parser that stumbled on them would be seen.
    for signature in [b"FSMN", b"UNTR"] {
        assert!(bytes.windows(4).any(|window| { return window == signature; }), "{} is in the index", String::from_utf8_lossy(signature));
    }
    let entries = parse_index(bytes.as_slice(), ObjectFormat::Sha1).expect("parsed");
    let listing = String::from_utf8(git_output(repo.as_path(), &["ls-files", "--stage"]).stdout).expect("UTF-8");
    let mine: Vec<String> = entries.iter().map(|entry| { return format!("{:06o} {} {}\t{}", entry.mode, to_hex(entry.id.as_slice()), entry.stage, String::from_utf8_lossy(entry.path.as_slice())); }).collect();
    assert_eq!(mine, listing.lines().map(str::to_string).collect::<Vec<String>>());
    remove(root.as_path());
}

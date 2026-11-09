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

/// Where the body of extension `signature` starts and how long it is.
fn extension_body(bytes: &[u8], signature: &[u8; 4]) -> Option<(usize, usize)> {
    let at: usize = bytes.windows(4).position(|window| { return window == signature; })?;
    return Some((at + 8, usize::try_from(be32(bytes, at + 4)?).ok()?));
}

/// The index Git writes with `index.threads` set,
/// with `change` applied to the `IEOT` body and the
/// trailing SHA-1 resealed.
fn with_ieot_body(written: &[u8], change: impl Fn(&mut [u8], usize)) -> Vec<u8> {
    let (body, _size) = extension_body(written, b"IEOT").expect("the offset table is in the index");
    let mut changed: Vec<u8> = written[..written.len() - 20].to_vec();
    change(changed.as_mut_slice(), body);
    let digest: [u8; 20] = crate::subvolume_sha::sha1(changed.as_slice());
    changed.extend_from_slice(digest.as_slice());
    return changed;
}

/// An offset table (`IEOT`) is answered only when it names the entries this reader parsed,
/// because Git reads the entries through the table when threaded loading is on.
/// Measured on Git 2.56.0 with the index below and `index.threads=4`:
/// a version other than 1 prints `error: invalid IEOT version 2` and answers,
/// a first block one byte off ends Git without a word,
/// and a first block pointing into the header prints `fatal: unknown index entry format 0xc0110000`.
#[test]
fn an_offset_table_git_reads_another_way_is_declined() {
    let root = fixture("repo-read-index-offset-table");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::create_dir_all(repo.join("d/e")).expect("dirs");
    for name in ["d/a", "d/b", "d/e/c", "d/ee"] {
        std::fs::write(repo.join(name), format!("{name}\n")).expect("write");
    }
    git(repo.as_path(), &["add", "--all"]);
    // A threaded index write records the offset table and the end of the entries it is found through.
    git(repo.as_path(), &["-c", "index.threads=4", "-c", "index.recordOffsetTable=true", "update-index", "--index-version", "4"]);
    let written = std::fs::read(repo.join(".git/index")).expect("index");
    let (body, size) = extension_body(written.as_slice(), b"IEOT").expect("the offset table is in the index");
    assert_eq!(be32(written.as_slice(), body), Some(1), "Git writes version 1");
    assert!(size >= 12, "Git writes at least one block");
    assert!(parse_index(written.as_slice(), ObjectFormat::Sha1).is_some(), "Git's own offset table is answered");
    let version_two = with_ieot_body(written.as_slice(), |changed, area| { changed[area + 3] = 2; });
    assert!(parse_index(version_two.as_slice(), ObjectFormat::Sha1).is_none(), "a version Git words is declined");
    let shifted = with_ieot_body(written.as_slice(), |changed, area| {
        let first = be32(changed, area + 4).expect("the first block's offset");
        changed[area + 4..area + 8].copy_from_slice(&(first + 1).to_be_bytes());
    });
    assert!(parse_index(shifted.as_slice(), ObjectFormat::Sha1).is_none(), "a table naming another byte is declined");
    let short = with_ieot_body(written.as_slice(), |changed, area| {
        let count = be32(changed, area + 12).expect("the first block's count");
        changed[area + 12..area + 16].copy_from_slice(&(count + 1).to_be_bytes());
    });
    assert!(parse_index(short.as_slice(), ObjectFormat::Sha1).is_none(), "blocks that overrun the entries are declined");
    remove(root.as_path());
}

/// The name-length value Git reads as "the path carries its own length",
/// so a version-4 entry may strip a shared prefix without stating the decoded length.
const COMPUTED_NAME_LENGTH: u16 = 0x0fff;

/// The bytes an extension's header takes:
/// its signature and its length.
const EXTENSION_HEADER_LENGTH: usize = 8;

/// Where an entry's mode starts:
/// after its timestamps,
/// device and inode.
const MODE_IN_ENTRY: usize = 24;

/// The bytes an entry's flags take after its object name.
const FLAGS_IN_ENTRY: usize = 2;

/// One version-4 entry head:
/// `mode` at its fixed offset,
/// every stat field and the object name zero,
/// and `declared_length` in the flags.
fn version_4_entry_head(mode: u32, declared_length: u16) -> Vec<u8> {
    let object_name: usize = name_length(ObjectFormat::Sha1);
    let mut head: Vec<u8> = vec![0; STAT_LENGTH + object_name + FLAGS_IN_ENTRY];
    head[MODE_IN_ENTRY..MODE_IN_ENTRY + 4].copy_from_slice(&mode.to_be_bytes());
    head[STAT_LENGTH + object_name..].copy_from_slice(&declared_length.to_be_bytes());
    return head;
}

/// `body` under `signature` as one index extension writes them.
fn extension(signature: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut written: Vec<u8> = signature.to_vec();
    written.extend_from_slice(&u32::try_from(body.len()).expect("a body this control writes").to_be_bytes());
    written.extend_from_slice(body);
    return written;
}

/// The path of every record of a `git ls-files --stage -z` listing.
fn staged_paths(listing: &[u8]) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for record in listing.split(|byte| return *byte == 0) {
        let Some(tab) = record.iter().position(|byte| return *byte == b'\t') else {
            continue;
        };
        paths.push(String::from_utf8_lossy(&record[tab + 1..]).into_owned());
    }
    return paths;
}

/// Git's offset table restarts the path at every block,
/// because Git decodes a block with no preceding entry;
/// a version-4 file whose block begins on a compressed path is a decline,
/// so Git answers it instead.
/// Measured on Git 2.56.0 with the hand-built file below:
/// `index.threads=3` lists `a/a` and `z`,
/// where `index.threads=1` lists `a/a` and `a/z`.
#[test]
fn a_version_4_offset_block_that_starts_on_a_compressed_path_is_declined() {
    let root = fixture("repo-read-index-compressed-block");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    // The first entry carries its whole path;
    // the second strips the `a/` of it, as version 4 writes a prefix the two share.
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(b"DIRC");
    payload.extend_from_slice(&4u32.to_be_bytes());
    payload.extend_from_slice(&2u32.to_be_bytes());
    assert_eq!(payload.len(), HEADER_LENGTH);
    let first_at: usize = payload.len();
    payload.extend(version_4_entry_head(0o100_644, 3));
    payload.extend_from_slice(&[0, b'a', b'/', b'a', 0]);
    let second_at: usize = payload.len();
    payload.extend(version_4_entry_head(0o100_644, COMPUTED_NAME_LENGTH));
    payload.extend_from_slice(&[1, b'z', 0]);
    let extensions_at: usize = payload.len();
    // One block per entry, so the second block begins on the compressed path.
    let mut offset_table: Vec<u8> = 1u32.to_be_bytes().to_vec();
    for begins in [first_at, second_at] {
        offset_table.extend_from_slice(&u32::try_from(begins).expect("an offset this control writes").to_be_bytes());
        offset_table.extend_from_slice(&1u32.to_be_bytes());
    }
    let table: Vec<u8> = extension(b"IEOT", offset_table.as_slice());
    payload.extend_from_slice(&table);
    // The end-of-entries extension names where the extensions start and hashes their headers,
    // which is how Git finds the table before it reads the entries.
    let mut end_of_entries: Vec<u8> = u32::try_from(extensions_at).expect("an offset this control writes").to_be_bytes().to_vec();
    end_of_entries.extend_from_slice(&crate::subvolume_sha::sha1(&table[..EXTENSION_HEADER_LENGTH]));
    payload.extend(extension(b"EOIE", end_of_entries.as_slice()));
    payload.extend_from_slice(&crate::subvolume_sha::sha1(payload.as_slice()));
    std::fs::write(repo.join(".git/index"), payload.as_slice()).expect("the hand-built index");
    // The premise: the table changes what Git lists, so the file is one this reader must not answer alone.
    let threaded: Vec<String> = staged_paths(
        git(repo.as_path(), &["-c", "index.threads=3", "ls-files", "--stage", "-z"]).stdout.as_slice(),
    );
    assert_eq!(threaded, vec![String::from("a/a"), String::from("z")], "Git reads the second block with no preceding entry");
    let sequential: Vec<String> = staged_paths(
        git(repo.as_path(), &["-c", "index.threads=1", "ls-files", "--stage", "-z"]).stdout.as_slice(),
    );
    assert_eq!(sequential, vec![String::from("a/a"), String::from("a/z")], "without the table Git joins the stripped prefix");
    assert!(
        parse_index(payload.as_slice(), ObjectFormat::Sha1).is_none(),
        "a block beginning on a compressed path is declined"
    );
    remove(root.as_path());
}

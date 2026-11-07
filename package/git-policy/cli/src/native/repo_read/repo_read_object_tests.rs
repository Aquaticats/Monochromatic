//! Controls for the loose object and pack entry inflater:
//!  a stream with bytes after its end is Git's `garbage at end of loose object` and is left to Git,
//!  and no damaged stream (a byte changed at every position,
//! a stream cut at every length) panics the reader.

use super::*;
use crate::test_support::{fixture, git, git_output, remove, repository};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The loose object files of a repository with a commit,
/// a tree and several blobs,
/// big enough to hold a
///  dynamic-Huffman block as well as stored and fixed ones.
fn loose_files(root: &Path) -> (PathBuf, Vec<PathBuf>) {
    let repo = repository(root, "objects");
    let mut text = String::new();
    for line in 0..400 {
        text.push_str(format!("line {line} of the blob with some repeated words and some words only once {}\n", line * 7919 % 101).as_str());
    }
    std::fs::write(repo.join("big.txt"), text).expect("big");
    std::fs::write(repo.join("small.txt"), "one\n").expect("small");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=with files"]);
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(repo.join(".git/objects")).expect("objects") {
        let directory = entry.expect("entry").path();
        if directory.file_name().is_some_and(|name| { return name.len() == 2; }) {
            for file in std::fs::read_dir(&directory).expect("fan-out") {
                files.push(file.expect("file").path());
            }
        }
    }
    files.sort();
    return (repo, files);
}

/// Bytes after the end of a loose stream are an error in Git,
/// so the reader must not answer from the stream before them.
#[test]
fn bytes_after_a_loose_stream_are_left_to_git() {
    let root = fixture("repo-read-object-garbage");
    let (repo, files) = loose_files(root.as_path());
    for file in &files {
        let name = format!("{}{}", file.parent().and_then(|parent| { return parent.file_name(); }).expect("fan-out").to_string_lossy(), file.file_name().expect("name").to_string_lossy());
        let original = std::fs::read(file).expect("read");
        assert!(parse_loose(original.as_slice()).is_some(), "{name}: the intact object is read");
        let mut extended = original.clone();
        extended.extend_from_slice(b"trailing bytes");
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o644)).expect("writable");
        std::fs::write(file, &extended).expect("rewrite");
        let verdict = git_output(repo.as_path(), &["cat-file", "-p", name.as_str()]);
        if !verdict.status.success() {
            assert!(parse_loose(extended.as_slice()).is_none(), "{name}: Git refuses the object ({}) but the reader answered", String::from_utf8_lossy(&verdict.stderr).trim());
        }
        std::fs::write(file, &original).expect("restore");
    }
    remove(root.as_path());
}

/// A stream with one byte changed at any position,
/// or cut at any length,
/// is read or refused,
/// never a panic.
#[test]
fn damaged_streams_never_panic() {
    let root = fixture("repo-read-object-damage");
    let (_, files) = loose_files(root.as_path());
    let mut tried: usize = 0;
    for file in &files {
        let original = std::fs::read(file).expect("read");
        for cut in 0..=original.len() {
            tried += 1;
            let _ = std::panic::catch_unwind(|| { return (parse_loose(&original[..cut]), inflate_exact(&original[..cut], original.len())); }).expect("a cut stream must not panic");
        }
        for position in 0..original.len() {
            for pattern in [0xffu8, 0x01, 0x80, 0x5a] {
                let mut damaged = original.clone();
                damaged[position] ^= pattern;
                tried += 1;
                let _ = std::panic::catch_unwind(|| { return (parse_loose(damaged.as_slice()), inflate_exact(damaged.as_slice(), original.len())); }).expect("a damaged stream must not panic");
            }
        }
    }
    assert!(tried > files.len(), "no damaged stream was tried");
    remove(root.as_path());
}

/// A zlib stream that stores `content` in blocks of at most 65535 bytes,
/// which decodes in time proportional to its length.
fn stored_stream(content: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0x78, 0x01];
    let mut chunks = content.chunks(65535).peekable();
    while let Some(chunk) = chunks.next() {
        let length: u16 = u16::try_from(chunk.len()).expect("a chunk fits");
        bytes.push(u8::from(chunks.peek().is_none()));
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(&(!length).to_le_bytes());
        bytes.extend_from_slice(chunk);
    }
    let (mut low, mut high): (u32, u32) = (1, 0);
    for byte in content {
        low = (low + u32::from(*byte)) % 65521;
        high = (high + low) % 65521;
    }
    bytes.extend_from_slice(&((high << 16) | low).to_be_bytes());
    return bytes;
}

/// The decoder's cost for a stream that rebuilds its tables for every block without producing output grows with the
///  stream's length,
/// not its output (upstream issue 198 of the decoder crate),
/// so a stream longer than the bound is left to
///  Git;
/// one under it,
/// of the same kind,
/// is read.
#[test]
fn a_stream_longer_than_the_bound_is_left_to_git() {
    let under: Vec<u8> = (0..INFLATE_INPUT_LIMIT / 2).map(|index| { return (index % 251) as u8; }).collect();
    let over: Vec<u8> = (0..INFLATE_INPUT_LIMIT + 4096).map(|index| { return (index % 251) as u8; }).collect();
    let short = stored_stream(under.as_slice());
    assert!(short.len() < INFLATE_INPUT_LIMIT);
    assert_eq!(inflate_exact(short.as_slice(), under.len()), Some(under), "the control must be readable");
    let long = stored_stream(over.as_slice());
    assert!(long.len() > INFLATE_INPUT_LIMIT);
    assert_eq!(inflate_exact(long.as_slice(), over.len()), None);
}

//! Index files and extensions real Git wrote,
//! taken from a fixture,
//! for the controls of the readers and writers.

use crate::test_support::{age, fixture, git, remove};

/// The bytes of the extension `signature` in an index file,
/// or `None` when the file has none.
pub(crate) fn extension_of(index: &[u8], signature: &[u8; 4], hash_length: usize) -> Option<Vec<u8>> {
    // Walk the file the way the reader does, from the end of the entries, to find the extension without the reader's help.
    let mut at: usize = 12;
    let count: usize = u32::from_be_bytes(index[8..12].try_into().ok()?) as usize;
    let version: u32 = u32::from_be_bytes(index[4..8].try_into().ok()?);
    let mut previous: Vec<u8> = Vec::new();
    for _ in 0..count {
        let flags: u16 = u16::from_be_bytes(index[at + 40 + hash_length..at + 42 + hash_length].try_into().ok()?);
        let head: usize = 42 + hash_length + if flags & 0x4000 != 0 { 2 } else { 0 };
        if version == 4 {
            let (strip, rest) = crate::index_stat::decode_varint(&index[at + head..])?;
            let used: usize = index.len() - rest.len() - (at + head);
            let end: usize = rest.iter().position(|byte| { return *byte == 0; })?;
            previous.truncate(previous.len() - strip as usize);
            previous.extend_from_slice(&rest[..end]);
            at += head + used + end + 1;
        } else {
            let end: usize = index[at + head..].iter().position(|byte| { return *byte == 0; })?;
            at += (head + end + 8) & !7;
        }
    }
    let stop: usize = index.len() - hash_length;
    while at < stop {
        let size: usize = u32::from_be_bytes(index[at + 4..at + 8].try_into().ok()?) as usize;
        if &index[at..at + 4] == signature {
            return Some(index[at + 8..at + 8 + size].to_vec());
        }
        at += 8 + size;
    }
    return None;
}

/// A repository with an untracked cache that lists untracked files and directories;
/// its index file.
fn repository_with_untracked_cache(name: &str) -> (std::path::PathBuf, Vec<u8>) {
    let root: std::path::PathBuf = fixture(name);
    let repo: std::path::PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    for path in ["a", "d/x", "d/y", "e/deep/z"] {
        let file = repo.join(path);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
        std::fs::write(&file, b"x\n").expect("write");
        age(&file);
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    git(repo.as_path(), &["update-index", "--force-untracked-cache"]);
    for path in ["top-untracked", "d/sub-untracked/file", "d/another", "e/deep/loose"] {
        let file = repo.join(path);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
        std::fs::write(&file, b"u\n").expect("write");
    }
    git(repo.as_path(), &["status", "--short"]);
    let bytes: Vec<u8> = std::fs::read(repo.join(".git").join("index")).expect("index");
    return (root, bytes);
}

/// The untracked-cache extension Git wrote in a fresh fixture.
pub(crate) fn untracked_extension(name: &str) -> Vec<u8> {
    let (root, bytes): (std::path::PathBuf, Vec<u8>) = repository_with_untracked_cache(name);
    let found: Vec<u8> = extension_of(bytes.as_slice(), b"UNTR", 20).expect("Git wrote an untracked cache");
    remove(root.as_path());
    return found;
}

/// A whole index file in the given version with the untracked cache,
/// the cache-tree and real stat data.
pub(crate) fn index_with_extensions(name: &str, version: &str) -> Vec<u8> {
    let (root, _): (std::path::PathBuf, Vec<u8>) = repository_with_untracked_cache(name);
    let repo: std::path::PathBuf = root.join("repo");
    git(repo.as_path(), &["update-index", "--index-version", version]);
    let bytes: Vec<u8> = std::fs::read(repo.join(".git").join("index")).expect("index");
    remove(root.as_path());
    return bytes;
}

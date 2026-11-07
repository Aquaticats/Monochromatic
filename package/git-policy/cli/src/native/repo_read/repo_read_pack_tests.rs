//! Controls for the delta decoder and the pack header reads;
//!  the whole-pack comparison with real Git is in `repo_read_odb_tests.rs`.

use super::*;

/// A delta that copies,
/// inserts and copies again gives the bytes the encoding states.
#[test]
fn a_delta_copies_and_inserts() {
    let base: &[u8] = b"hello world";
    // Source size 11, target size 14: copy "hello", insert "!!!", copy " world".
    let mut delta: Vec<u8> = vec![11, 14];
    // Copy: offset 0 (no offset bits), size 5 (bit 4 set, one size byte).
    delta.extend_from_slice(&[0x90, 5]);
    // Insert three literal bytes.
    delta.extend_from_slice(&[3, b'!', b'!', b'!']);
    // Copy: offset 5 (bit 0), size 6 (bit 4).
    delta.extend_from_slice(&[0x91, 5, 6]);
    assert_eq!(apply_delta(base, delta.as_slice()), Some(b"hello!!! world".to_vec()));
}

/// A delta whose sizes,
/// ranges or opcodes disagree with its base is refused.
#[test]
fn a_bad_delta_is_refused() {
    let base: &[u8] = b"abcdef";
    assert!(apply_delta(base, &[7, 1, 1, b'x']).is_none(), "wrong base size");
    assert!(apply_delta(base, &[6, 2, 1, b'x']).is_none(), "result shorter than stated");
    assert!(apply_delta(base, &[6, 1, 0x91, 5, 5]).is_none(), "copy past the base");
    assert!(apply_delta(base, &[6, 1, 0]).is_none(), "opcode zero is reserved");
    assert!(apply_delta(base, &[6, 1, 2, b'x']).is_none(), "insert past the delta");
}

/// An index with a bad magic or a short size is refused without panicking.
#[test]
fn a_bad_index_is_refused() {
    let root = crate::test_support::fixture("repo-read-pack-bad");
    let index = root.join("pack-x.idx");
    std::fs::write(&index, b"not an index").expect("write");
    std::fs::write(root.join("pack-x.pack"), b"PACK").expect("write");
    assert!(Pack::open(index.as_path(), 20).is_none());
    let mut short: Vec<u8> = b"\xfftOc\0\0\0\x02".to_vec();
    short.extend(std::iter::repeat_n(0_u8, 256 * 4));
    short[8 + 255 * 4 + 3] = 5;
    std::fs::write(&index, short).expect("write");
    assert!(Pack::open(index.as_path(), 20).is_none(), "five objects claimed, none stored");
    crate::test_support::remove(root.as_path());
}

/// A repository of several hundred objects packed into one pack,
/// as the pack's index path,
/// and the names of its objects.
fn packed_repository(root: &std::path::Path) -> (std::path::PathBuf, Vec<Vec<u8>>) {
    use crate::test_support::{git, git_output};
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    for number in 0..400 {
        std::fs::write(repo.join(format!("file-{number}.txt")), format!("content of file {number}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=many"]);
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    git(repo.as_path(), &["prune-packed"]);
    let index = std::fs::read_dir(repo.join(".git/objects/pack"))
        .expect("pack dir")
        .flatten()
        .map(|entry| { return entry.path(); })
        .find(|path| { return path.extension().is_some_and(|extension| { return extension == "idx"; }); })
        .expect("an index");
    let listing = git_output(repo.as_path(), &["cat-file", "--batch-all-objects", "--batch-check=%(objectname)"]);
    let names: Vec<Vec<u8>> = String::from_utf8(listing.stdout)
        .expect("UTF-8")
        .lines()
        .map(|line| { return crate::repo_read_object::from_hex(line.as_bytes(), crate::repo_read_layout::ObjectFormat::Sha1).expect("name"); })
        .collect();
    return (index, names);
}

/// A lookup reads the index a constant few times however many objects the pack holds:
/// one read narrows the search to the
///  names that share the first byte and one more fetches the offset,
/// so a name the pack does not hold costs one read.
#[test]
fn a_lookup_reads_the_index_a_few_times() {
    let root = crate::test_support::fixture("repo-read-pack-reads");
    let (index, names) = packed_repository(root.as_path());
    let pack = Pack::open(index.as_path(), 20).expect("pack");
    let opened = pack.index_reads();
    // Positive control: the pack holds many objects, so a search that probes name by name needs far more than the bound.
    assert!(names.len() > 400, "the fixture pack holds many objects");
    for name in &names {
        let before = pack.index_reads();
        assert!(pack.find(name.as_slice()).is_some(), "a name of the pack is found");
        assert!(pack.index_reads() - before <= 2, "a found name costs at most two reads of the index");
    }
    // The names of one first byte are read once however many lookups land in them: every lookup reads its own offset, and
    // each distinct first byte adds one read of its bucket.
    let buckets: std::collections::HashSet<u8> = names.iter().map(|name| { return name[0]; }).collect();
    assert_eq!(pack.index_reads() - opened, (names.len() + buckets.len()) as u64, "one read of the offset per name and one per bucket");
    let mut absent: Vec<u8> = names[0].clone();
    absent[19] ^= 1;
    let before = pack.index_reads();
    assert!(pack.find(absent.as_slice()).is_none(), "a name the pack does not hold is not found");
    assert_eq!(pack.index_reads() - before, 0, "an absent name whose bucket was read costs no read of the index");
    crate::test_support::remove(root.as_path());
}

/// Opening a pack reads its index twice at most,
/// however many objects it holds:
/// the header and the table of counts that
/// follows it are one stretch of the file.
/// The wrapper opens the object store afresh for each question it answers (a dozen for
/// one commit,
/// over every pack of the repository),
/// and each open used to read the table in 256 pieces of four bytes.
#[test]
fn opening_a_pack_reads_its_index_a_few_times() {
    let root = crate::test_support::fixture("repo-read-pack-open");
    let (index, _names) = packed_repository(root.as_path());
    let pack = Pack::open(index.as_path(), 20).expect("pack");
    // Positive control: the counter counts what the open read, so an open that read the table piece by piece is seen.
    assert!(pack.index_reads() > 0, "the open reads the index");
    assert!(pack.index_reads() <= 2, "opening a pack read its index {} times", pack.index_reads());
    crate::test_support::remove(root.as_path());
}

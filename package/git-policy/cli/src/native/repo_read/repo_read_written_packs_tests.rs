//! What:
//!  The controls of a pack whose bytes the test writes itself:
//!  an entry stored whole is read,
//!  and an index entry that points outside its pack,
//!  a delta whose distance comes back to its own entry,
//!  a chain longer than the pack's header says the pack holds,
//!  a size that disagrees with the bytes,
//!  and a name that does not match its bytes
//!  each make the reader give up rather than answer.
//! Why:
//!   The reader trusts a store only when the bytes it hands back are the object that was asked for,
//!  and Git will not write such packs,
//!  so the only way to hold one is to write it.
//!  Every control first shows the same reader answering the same builder's well-formed pack,
//!  so a refusal is the malformed part and not the builder.

use super::*;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::object_name;
use crate::repo_read_odb::ObjectDatabase;
use crate::test_support::fixture;
use std::path::{Path, PathBuf};

/// What:
///  One entry a control writes:
///  an object stored whole,
///  or a delta over the entry a distance back,
///  or a delta over an object by name.
/// Why:
///   Each shape is spelled by its own kind code and its own bytes.
#[derive(Clone)]
struct Written {
    /// The entry's kind code:
    /// 1 commit,
    /// 2 tree,
    /// 3 blob,
    /// 4 tag,
    /// 6 a delta over the entry a distance back,
    /// 7 a delta over an object by name.
    code: u8,
    /// The bytes the entry's stream holds before compression:
    /// an object's content, or the delta's instructions.
    payload: Vec<u8>,
    /// How many bytes back the base of a code 6 entry starts.
    distance: u64,
    /// The name of the base of a code 7 entry.
    base: Vec<u8>,
    /// What the header states as the entry's size,
    /// or `None` to state the payload's length.
    stated_size: Option<usize>,
}

/// An object stored whole in a pack.
fn whole(code: u8, content: &[u8]) -> Written {
    return Written { code, payload: content.to_vec(), distance: 0, base: Vec::new(), stated_size: None };
}

/// The tree content of one file,
/// whose object name is `blob`.
fn tree_with(blob: &[u8], path: &str) -> Vec<u8> {
    let mut tree: Vec<u8> = format!("100644 {path}\0").into_bytes();
    tree.extend_from_slice(blob);
    return tree;
}

/// The kind a whole entry's code names.
fn kind_of(code: u8) -> ObjectKind {
    return match code {
        1 => ObjectKind::Commit,
        2 => ObjectKind::Tree,
        3 => ObjectKind::Blob,
        _ => ObjectKind::Tag,
    };
}

/// The name a whole entry's object hashes to.
fn name_of(entry: &Written) -> Vec<u8> {
    return object_name(kind_of(entry.code), entry.payload.as_slice(), ObjectFormat::Sha1);
}

/// The distance of a delta over the entry that many bytes back,
/// spelled seven bits at a time,
/// most significant first,
/// with every digit but the first marked as continuing.
fn distance_digits(distance: u64) -> Vec<u8> {
    let mut digits: Vec<u8> = vec![u8::try_from(distance & 0x7f).expect("seven bits")];
    let mut rest: u64 = distance;
    while rest > 0x7f {
        rest = (rest >> 7) - 1;
        digits.push(u8::try_from(rest & 0x7f).expect("seven bits"));
    }
    digits.reverse();
    let last: usize = digits.len() - 1;
    for digit in &mut digits[..last] {
        *digit |= 0x80;
    }
    return digits;
}

/// The header of one pack entry:
/// the kind in the high bits,
/// the size seven bits at a time,
/// low first.
fn entry_header(entry: &Written) -> Vec<u8> {
    let mut size: usize = entry.stated_size.unwrap_or(entry.payload.len());
    let mut header: Vec<u8> = Vec::new();
    let mut first: u8 = entry.code << 4 | u8::try_from(size & 15).expect("four bits");
    size >>= 4;
    while size > 0 {
        header.push(first | 0x80);
        first = u8::try_from(size & 0x7f).expect("seven bits");
        size >>= 7;
    }
    header.push(first);
    return header;
}

/// What:
///  The bytes of a pack holding `entries` in that order,
///  the offset each entry starts at,
///  and the count its header states.
/// Why:
///   A control names the offsets its index stores,
/// and one of them must point somewhere the pack does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { bytes, offsets } = packBytes(entries, statedCount);
/// ```
fn pack_bytes(entries: &[Written], stated_count: u32) -> (Vec<u8>, Vec<u64>) {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(b"PACK");
    bytes.extend_from_slice(&[0, 0, 0, 2]);
    bytes.extend_from_slice(&stated_count.to_be_bytes());
    let mut offsets: Vec<u64> = Vec::new();
    for entry in entries {
        offsets.push(bytes.len() as u64);
        bytes.extend_from_slice(entry_header(entry).as_slice());
        if entry.code == 6 {
            bytes.extend_from_slice(distance_digits(entry.distance).as_slice());
        }
        if entry.code == 7 {
            bytes.extend_from_slice(entry.base.as_slice());
        }
        bytes.extend_from_slice(miniz_oxide::deflate::compress_to_vec_zlib(entry.payload.as_slice(), 6).as_slice());
    }
    let trailer: [u8; 20] = crate::subvolume_sha::sha1(bytes.as_slice());
    bytes.extend_from_slice(trailer.as_slice());
    return (bytes, offsets);
}

/// What:
///  The bytes of a version 2 index naming `pairs`,
///  which must be sorted by name.
/// Why:
///   The reader binary-searches the names in that order,
/// and a control changes nothing but an offset or a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const index = indexBytes([[name, offset]]);
/// ```
fn index_bytes(pairs: &[(Vec<u8>, u64)]) -> Vec<u8> {
    let mut bytes: Vec<u8> = b"\xfftOc\0\0\0\x02".to_vec();
    for bucket in 0..256 {
        let count: u32 = u32::try_from(pairs.iter().filter(|(name, _)| { return usize::from(name[0]) <= bucket; }).count()).expect("a count");
        bytes.extend_from_slice(&count.to_be_bytes());
    }
    for (name, _) in pairs {
        bytes.extend_from_slice(name.as_slice());
    }
    // Every entry's checksum is zero: the reader never looks at it.
    bytes.extend(std::iter::repeat_n(0_u8, pairs.len() * 4));
    for (_, offset) in pairs {
        bytes.extend_from_slice(&u32::try_from(*offset).expect("an offset in the pack's first four gigabytes").to_be_bytes());
    }
    bytes.extend_from_slice(&[0; 20]);
    let checksum: [u8; 20] = crate::subvolume_sha::sha1(bytes.as_slice());
    bytes.extend_from_slice(checksum.as_slice());
    return bytes;
}

/// A repository directory holding exactly the pack and index the control wrote.
fn write_store(root: &Path, pack: &[u8], index: &[u8]) -> PathBuf {
    let git_dir: PathBuf = root.join("written");
    let pack_dir: PathBuf = git_dir.join("objects/pack");
    std::fs::create_dir_all(pack_dir.as_path()).expect("the pack directory");
    std::fs::write(pack_dir.join("pack-written.pack"), pack).expect("the pack");
    std::fs::write(pack_dir.join("pack-written.idx"), index).expect("the index");
    return git_dir;
}

/// The database of a repository whose only pack is the one the control wrote.
fn opened(git_dir: &Path) -> ObjectDatabase {
    return ObjectDatabase::open(git_dir, None, ObjectFormat::Sha1).expect("the written pack opens");
}

/// A tree and a blob stored whole,
/// the names sorted,
/// and the store written out.
fn store_of_tree_and_blob(root: &Path) -> (PathBuf, Vec<Written>) {
    let blob: Written = whole(3, b"one\n");
    let tree: Written = whole(2, tree_with(name_of(&blob).as_slice(), "one.txt").as_slice());
    let entries: Vec<Written> = vec![tree, blob];
    let (pack, offsets) = pack_bytes(entries.as_slice(), 2);
    let mut pairs: Vec<(Vec<u8>, u64)> = entries.iter().zip(offsets.iter()).map(|(entry, offset)| { return (name_of(entry), *offset); }).collect();
    pairs.sort();
    let git_dir: PathBuf = write_store(root, pack.as_slice(), index_bytes(pairs.as_slice()).as_slice());
    return (git_dir, entries);
}

/// What:
///  A pack the test wrote,
///  holding a tree and a blob stored whole,
///  read as the objects they are.
/// Why:
///   The positive control of every refusal here:
///  when nothing is malformed the reader answers this builder's bytes,
///  so a refusal names the malformed part and not the builder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (git, tools) = storeOfTreeAndBlob(root); expect(opened(git).read(nameOf(tools[1]))).toBe(b'one\n');
/// ```
#[test]
fn a_pack_the_test_wrote_is_read() {
    let root: PathBuf = fixture("repo-read-written-pack");
    let (git_dir, entries) = store_of_tree_and_blob(root.as_path());
    let database: ObjectDatabase = opened(git_dir.as_path());
    assert_eq!(database.read(name_of(&entries[1]).as_slice()).expect("the blob is read").bytes, b"one\n");
    let tree = database.read(name_of(&entries[0]).as_slice()).expect("the tree is read");
    assert_eq!(tree.kind, ObjectKind::Tree);
    assert_eq!(tree.bytes, entries[0].payload);
    crate::test_support::remove(root.as_path());
}

/// What:
///  An index whose entry for a name points past the end of the pack,
///  and one whose entry points inside the pack's own header.
/// Why:
///   A store that points outside itself holds no object to answer,
/// and every read it bounds by the pack file's own length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (git, tools) = storeOfTreeAndBlob(root); const beyond = packLength + 5;
/// ```
#[test]
fn an_index_entry_that_points_outside_its_pack_is_refused() {
    let root: PathBuf = fixture("repo-read-written-outside");
    let (git_dir, entries) = store_of_tree_and_blob(root.as_path());
    let (pack, offsets) = pack_bytes(entries.as_slice(), 2);
    for (label, offset) in [("past the end", pack.len() as u64 + 5), ("inside the header", 4)] {
        let mut pairs: Vec<(Vec<u8>, u64)> = vec![(name_of(&entries[0]), offset), (name_of(&entries[1]), offsets[1])];
        pairs.sort();
        std::fs::write(git_dir.join("objects/pack/pack-written.idx"), index_bytes(pairs.as_slice())).expect("the index");
        let database: ObjectDatabase = opened(git_dir.as_path());
        assert!(database.read(name_of(&entries[0]).as_slice()).is_none(), "{label}: nothing is answered for that entry");
        assert_eq!(database.read(name_of(&entries[1]).as_slice()).expect("the entry the index states faithfully is read").bytes, b"one\n");
    }
    crate::test_support::remove(root.as_path());
}

/// A delta whose distance is zero,
/// so its base is the entry itself:
/// a chain that comes back to where it began.
#[test]
fn a_delta_that_comes_back_to_its_own_entry_is_refused() {
    let root: PathBuf = fixture("repo-read-written-loop");
    // A copy of every byte of a thirty-byte base: valid instructions, so only the loop can refuse this read.
    let base: Vec<u8> = vec![b'x'; 30];
    let delta: Vec<u8> = vec![30, 30, 0x90, 30];
    let mut looping = whole(2, base.as_slice());
    looping.code = 6;
    looping.payload = delta;
    looping.distance = 0;
    let entries: Vec<Written> = vec![looping];
    let (pack, offsets) = pack_bytes(entries.as_slice(), 1);
    let pairs: Vec<(Vec<u8>, u64)> = vec![(name_of(&whole(2, base.as_slice())), offsets[0])];
    let git_dir: PathBuf = write_store(root.as_path(), pack.as_slice(), index_bytes(pairs.as_slice()).as_slice());
    let database: ObjectDatabase = opened(git_dir.as_path());
    assert!(database.read(pairs[0].0.as_slice()).is_none(), "a chain that comes back to its own entry answers nothing");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A chain of three entries inside a pack whose own header says it holds two:
///  a chain no pack of that size can hold.
/// Why:
///   A delta's base lies before its own entry,
/// so the chain is written forwards and each distance is the gap the builder reported;
///  the index names the two ends of the chain,
///  and reading the far one walks one entry more than the header states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const base = whole('tree', 'x'.repeat(30)); const second = ofsDelta(baseOffset); const third = ofsDelta(secondOffset);
/// ```
#[test]
fn a_delta_chain_longer_than_the_pack_holds_is_refused() {
    let root: PathBuf = fixture("repo-read-written-top-heavy");
    let base: Vec<u8> = vec![b'x'; 30];
    let mut second_bytes: Vec<u8> = base.clone();
    second_bytes.push(b'b');
    let mut third_bytes: Vec<u8> = second_bytes.clone();
    third_bytes.push(b'c');
    // Copy the whole base and insert the one byte that makes the next version.
    let second_delta: Vec<u8> = vec![30, 31, 0x90, 30, 1, b'b'];
    let third_delta: Vec<u8> = vec![31, 32, 0x90, 31, 1, b'c'];
    let first: Written = whole(2, base.as_slice());
    let mut second: Written = whole(2, base.as_slice());
    second.code = 6;
    second.payload = second_delta;
    second.distance = 1;
    let mut third: Written = second.clone();
    third.payload = third_delta;
    third.distance = 1;
    let mut entries: Vec<Written> = vec![first, second, third];
    // The gaps need the offsets, and an offset needs only the digit count of the distance, not its value.
    let (_ignored, one_digit_gaps) = pack_bytes(entries.as_slice(), 2);
    entries[1].distance = one_digit_gaps[1] - one_digit_gaps[0];
    entries[2].distance = one_digit_gaps[2] - one_digit_gaps[1];
    let (pack, offsets) = pack_bytes(entries.as_slice(), 2);
    assert_eq!(offsets[1] - offsets[0], entries[1].distance, "the distance still fits one digit");
    let mut pairs: Vec<(Vec<u8>, u64)> = vec![
        (object_name(ObjectKind::Tree, second_bytes.as_slice(), ObjectFormat::Sha1), offsets[1]),
        (object_name(ObjectKind::Tree, third_bytes.as_slice(), ObjectFormat::Sha1), offsets[2]),
    ];
    pairs.sort();
    let git_dir: PathBuf = write_store(root.as_path(), pack.as_slice(), index_bytes(pairs.as_slice()).as_slice());
    // Positive control: the shorter chain is the same builder's and answers, so the longer one is what is refused.
    let (shorter_pack, shorter_offsets) = pack_bytes(entries[..2].to_vec().as_slice(), 2);
    let mut shorter_pairs: Vec<(Vec<u8>, u64)> = vec![
        (name_of(&entries[0]), shorter_offsets[0]),
        (object_name(ObjectKind::Tree, second_bytes.as_slice(), ObjectFormat::Sha1), shorter_offsets[1]),
    ];
    shorter_pairs.sort();
    std::fs::write(git_dir.join("objects/pack/pack-written.pack"), &shorter_pack).expect("the shorter pack");
    std::fs::write(git_dir.join("objects/pack/pack-written.idx"), index_bytes(shorter_pairs.as_slice())).expect("the shorter index");
    let shorter: ObjectDatabase = opened(git_dir.as_path());
    let second_name: Vec<u8> = object_name(ObjectKind::Tree, second_bytes.as_slice(), ObjectFormat::Sha1);
    assert_eq!(shorter.read(second_name.as_slice()).expect("two entries in a pack of two are read").bytes, second_bytes);
    std::fs::write(git_dir.join("objects/pack/pack-written.pack"), &pack).expect("the longer pack");
    std::fs::write(git_dir.join("objects/pack/pack-written.idx"), index_bytes(pairs.as_slice())).expect("the longer index");
    let third_name: Vec<u8> = object_name(ObjectKind::Tree, third_bytes.as_slice(), ObjectFormat::Sha1);
    // Each read opens the pack afresh: an entry the earlier read kept would finish the walk before the bound is reached,
    // which is the cache doing its work and not the chain being short enough.
    assert!(opened(git_dir.as_path()).read(third_name.as_slice()).is_none(), "a chain of three in a pack of two answers nothing");
    // The shorter chain is still two entries in a pack of two, so it answers.
    assert_eq!(opened(git_dir.as_path()).read(second_name.as_slice()).expect("a chain of two in a pack of two is read").bytes, second_bytes);
    crate::test_support::remove(root.as_path());
}

/// An entry whose header states a size its bytes do not fill,
/// one byte too many and one too few.
#[test]
fn an_entry_whose_stated_size_disagrees_with_its_bytes_is_refused() {
    let root: PathBuf = fixture("repo-read-written-size");
    let content: Vec<u8> = b"one\n".to_vec();
    for stated in [Some(content.len() + 1), Some(content.len() - 1)] {
        let mut entry: Written = whole(3, content.as_slice());
        entry.stated_size = stated;
        let entries: Vec<Written> = vec![entry];
        let (pack, offsets) = pack_bytes(entries.as_slice(), 1);
        let pairs: Vec<(Vec<u8>, u64)> = vec![(name_of(&whole(3, content.as_slice())), offsets[0])];
        let git_dir: PathBuf = write_store(root.as_path(), pack.as_slice(), index_bytes(pairs.as_slice()).as_slice());
        let database: ObjectDatabase = opened(git_dir.as_path());
        assert!(database.read(pairs[0].0.as_slice()).is_none(), "a stated size of {stated:?} answers nothing");
    }
    crate::test_support::remove(root.as_path());
}

/// What:
///  An index that names other bytes than the entry holds,
///  and a loose file whose path names other bytes than it holds.
/// Why:
///   Neither store may answer for a name it does not hold,
///  or a proof built on that answer would compare an index with a tree nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (git, tools) = storeOfTreeAndBlob(root); const other = mistyped(nameOf(tools[0]));
/// ```
#[test]
fn an_object_whose_name_does_not_match_its_bytes_is_refused() {
    let root: PathBuf = fixture("repo-read-written-name");
    let (git_dir, entries) = store_of_tree_and_blob(root.as_path());
    let (_pack, offsets) = pack_bytes(entries.as_slice(), 2);
    // A name one bit away from the tree's own, stored for the tree's entry.
    let mut mistyped: Vec<u8> = name_of(&entries[0]);
    let last: usize = mistyped.len() - 1;
    mistyped[last] ^= 1;
    let mut pairs: Vec<(Vec<u8>, u64)> = vec![(mistyped.clone(), offsets[0]), (name_of(&entries[1]), offsets[1])];
    pairs.sort();
    std::fs::write(git_dir.join("objects/pack/pack-written.idx"), index_bytes(pairs.as_slice())).expect("the index");
    let database: ObjectDatabase = opened(git_dir.as_path());
    assert!(database.read(mistyped.as_slice()).is_none(), "a pack entry that holds other bytes than its name answers nothing");
    assert!(database.read(name_of(&entries[0]).as_slice()).is_none(), "and the name the bytes spell is not in that pack");
    // The loose store: a file whose path names one object while its bytes are another's.
    let objects: PathBuf = git_dir.join("objects");
    let blob_name: Vec<u8> = name_of(&entries[1]);
    let tree_name: Vec<u8> = name_of(&entries[0]);
    let hex: String = crate::repo_read_object::to_hex(tree_name.as_slice());
    std::fs::create_dir_all(objects.join(&hex[..2])).expect("the loose directory");
    let liar: Vec<u8> = framed(3, b"one\n");
    std::fs::write(objects.join(&hex[..2]).join(&hex[2..]), &liar).expect("the loose file");
    let blob_hex: String = crate::repo_read_object::to_hex(blob_name.as_slice());
    std::fs::create_dir_all(objects.join(&blob_hex[..2])).expect("the blob's loose directory");
    std::fs::write(objects.join(&blob_hex[..2]).join(&blob_hex[2..]), &liar).expect("the blob's loose file");
    std::fs::remove_file(git_dir.join("objects/pack/pack-written.pack")).expect("the pack goes");
    std::fs::remove_file(git_dir.join("objects/pack/pack-written.idx")).expect("the index goes");
    let loose = ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).expect("the loose store opens");
    assert!(loose.read(tree_name.as_slice()).is_none(), "a loose file that holds other bytes than its path names answers nothing");
    assert_eq!(loose.read(blob_name.as_slice()).expect("the same bytes are read at the name they spell").bytes, b"one\n");
    crate::test_support::remove(root.as_path());
}

/// The zlib stream Git stores a loose object as:
/// `<kind> <size>\0<content>`.
fn framed(code: u8, content: &[u8]) -> Vec<u8> {
    let header: String = format!("{} {}\0", kind_of(code).word(), content.len());
    let mut whole_object: Vec<u8> = header.into_bytes();
    whole_object.extend_from_slice(content);
    return miniz_oxide::deflate::compress_to_vec_zlib(whole_object.as_slice(), 6);
}

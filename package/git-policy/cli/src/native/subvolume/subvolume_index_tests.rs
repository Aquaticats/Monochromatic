//! Controls for the index stat rewrite on built bytes: versions 2, 3 and 4, both hash lengths and the zero checksum,
//!  the entries that must keep their stat fields, the extensions that are kept, dropped or make the rewrite decline,
//!  and every malformed shape. The file system never enters; the lookup is a closure.

use super::*;

/// How the built index checksums itself.
#[derive(Clone, Copy)]
enum Checksum {
    Sha1,
    Sha256,
    Zero20,
    Zero32,
}

/// The stat fields of a built entry.
#[derive(Clone)]
struct Built {
    path: &'static str,
    mode: u32,
    flags: u16,
    size: u32,
    mtime_seconds: u32,
    mtime_nanos: u32,
}

/// A regular file entry with the stat fields the lookups below compare against.
fn regular(path: &'static str) -> Built {
    return Built { path, mode: 0o100_644, flags: 0, size: 10, mtime_seconds: 100, mtime_nanos: 7 };
}

/// What `lstat` would report for every path now: the same size and time, new inode and change time.
fn fresh() -> StatFields {
    return StatFields { ctime_seconds: 900, ctime_nanos: 901, mtime_seconds: 100, mtime_nanos: 7, dev: 5, ino: 6, uid: 1000, gid: 1001, size: 10 };
}

/// The bytes of a built entry's fixed part: old stat fields that the rewrite must replace.
fn entry_bytes(entry: &Built, hash_length: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for value in [1u32, 2, entry.mtime_seconds, entry.mtime_nanos, 3, 4, entry.mode, 8, 9, entry.size] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes.extend(std::iter::repeat_n(0xab, hash_length));
    bytes.extend_from_slice(&entry.flags.to_be_bytes());
    if entry.flags & 0x4000 != 0 {
        bytes.extend_from_slice(&0u16.to_be_bytes());
    }
    return bytes;
}

/// Build an index file.
fn build(version: u32, entries: &[Built], extensions: &[(&[u8; 4], Vec<u8>)], checksum: Checksum) -> Vec<u8> {
    let hash_length: usize = if matches!(checksum, Checksum::Sha256 | Checksum::Zero32) { 32 } else { 20 };
    let mut bytes: Vec<u8> = b"DIRC".to_vec();
    bytes.extend_from_slice(&version.to_be_bytes());
    bytes.extend_from_slice(&u32::try_from(entries.len()).expect("count").to_be_bytes());
    let mut previous: Vec<u8> = Vec::new();
    for entry in entries {
        let start: usize = bytes.len();
        bytes.extend(entry_bytes(entry, hash_length));
        let path: &[u8] = entry.path.as_bytes();
        if version == 4 {
            let common: usize = previous.iter().zip(path).take_while(|(left, right)| { return left == right; }).count();
            let strip: usize = previous.len() - common;
            assert!(strip < 0x80, "the built paths strip less than a varint byte");
            bytes.push(u8::try_from(strip).expect("strip"));
            bytes.extend_from_slice(&path[common..]);
            bytes.push(0);
            previous = path.to_vec();
        } else {
            bytes.extend_from_slice(path);
            bytes.push(0);
            while (bytes.len() - start) % 8 != 0 {
                bytes.push(0);
            }
        }
    }
    for (signature, payload) in extensions {
        bytes.extend_from_slice(signature.as_slice());
        bytes.extend_from_slice(&u32::try_from(payload.len()).expect("size").to_be_bytes());
        bytes.extend_from_slice(payload.as_slice());
    }
    match checksum {
        Checksum::Sha1 => bytes.extend_from_slice(&sha1(bytes.as_slice())),
        Checksum::Sha256 => bytes.extend_from_slice(&sha256(bytes.as_slice())),
        Checksum::Zero20 => bytes.extend_from_slice(&[0u8; 20]),
        Checksum::Zero32 => bytes.extend_from_slice(&[0u8; 32]),
    }
    return bytes;
}

/// The paths of a rewritten file's entries and the stat fields stored for each, read back with the same walker the
///  rewrite uses plus raw offsets.
fn stored(bytes: &[u8], hash_length: usize) -> Vec<(Vec<u8>, [u32; 9])> {
    let (entries, _end) = walk_entries(bytes, hash_length).expect("walk");
    return entries
        .iter()
        .map(|entry: &Located| {
            let field = |index: usize| -> u32 { return be32(bytes, entry.start + index * 4).expect("field"); };
            return (entry.path.clone(), [field(0), field(1), field(2), field(3), field(4), field(5), field(7), field(8), field(9)]);
        })
        .collect();
}

/// The fields a rewritten entry holds: change time, device, inode, owner and group replaced, the rest kept.
const REWRITTEN: [u32; 9] = [900, 901, 100, 7, 5, 6, 1000, 1001, 10];

/// The fields of a built entry the rewrite did not touch.
const UNTOUCHED: [u32; 9] = [1, 2, 100, 7, 3, 4, 8, 9, 10];

/// Unwrap a rewrite.
fn rewritten(outcome: IndexOutcome) -> (Vec<u8>, usize, usize) {
    let IndexOutcome::Rewritten { bytes, updated, total } = outcome else { panic!("the index was skipped: {outcome:?}") };
    return (bytes, updated, total);
}

/// Unwrap a skip.
fn skipped(outcome: IndexOutcome) -> String {
    let IndexOutcome::Skipped(reason) = outcome else { panic!("the index was rewritten") };
    return reason;
}

/// A version 2 index with SHA-1: matching entries get the new fields, the checksum is valid, and nothing else moves.
#[test]
fn a_version_2_index_gets_fresh_stat_fields_and_a_valid_checksum() {
    let input: Vec<u8> = build(2, &[regular("README.md"), regular("src/main.rs")], &[(b"TREE", vec![1, 2, 3])], Checksum::Sha1);
    let (output, updated, total) = rewritten(rewrite_index_stat(input.as_slice(), &mut |_path: &[u8]| { return Some(fresh()); }));
    assert_eq!((updated, total), (2, 2));
    assert_eq!(stored(output.as_slice(), 20), vec![(b"README.md".to_vec(), REWRITTEN), (b"src/main.rs".to_vec(), REWRITTEN)]);
    assert_eq!(output.len(), input.len(), "entry and extension lengths never change");
    assert_eq!(&output[output.len() - 20..], sha1(&output[..output.len() - 20]).as_slice());
    assert!(output.windows(4).any(|window: &[u8]| { return window == b"TREE"; }), "the cache-tree extension is kept");
}

/// An entry keeps its fields when the file is missing, differs in size or in time, has no size, is not a regular file
///  or a link, or is a conflict stage; a link and a time without nanoseconds are rewritten.
#[test]
fn only_entries_whose_file_still_matches_are_rewritten() {
    let mut stage: Built = regular("conflict");
    stage.flags = 0x1000;
    let mut empty: Built = regular("empty");
    empty.size = 0;
    let mut directory: Built = regular("gitlink");
    directory.mode = 0o160_000;
    let mut link: Built = regular("symlink");
    link.mode = 0o120_000;
    let mut plain_time: Built = regular("plain-time");
    plain_time.mtime_nanos = 0;
    let entries: Vec<Built> = vec![regular("missing"), regular("bigger"), regular("later"), stage, empty, directory, link, plain_time];
    let input: Vec<u8> = build(2, entries.as_slice(), &[], Checksum::Sha1);
    let mut lookup = |path: &[u8]| -> Option<StatFields> {
        return match path {
            b"missing" => None,
            b"bigger" => Some(StatFields { size: 11, ..fresh() }),
            b"later" => Some(StatFields { mtime_nanos: 8, ..fresh() }),
            b"plain-time" => Some(StatFields { mtime_nanos: 123_456, ..fresh() }),
            _ => Some(fresh()),
        };
    };
    let (output, updated, total) = rewritten(rewrite_index_stat(input.as_slice(), &mut lookup));
    assert_eq!((updated, total), (2, 8));
    for (path, fields) in stored(output.as_slice(), 20) {
        let expected: [u32; 9] = match path.as_slice() {
            b"symlink" => REWRITTEN,
            b"plain-time" => [900, 901, 100, 0, 5, 6, 1000, 1001, 10],
            b"empty" => [1, 2, 100, 7, 3, 4, 8, 9, 0],
            _ => UNTOUCHED,
        };
        assert_eq!(fields, expected, "{}", String::from_utf8_lossy(path.as_slice()));
    }
}

/// Version 3 entries with the extended-flags word are walked; the same word in a version 2 index is not a layout.
#[test]
fn extended_flags_belong_to_version_3_only() {
    let mut extended: Built = regular("intent");
    extended.flags = 0x4000;
    let v3: Vec<u8> = build(3, &[extended.clone(), regular("after")], &[], Checksum::Sha1);
    let (output, updated, _total) = rewritten(rewrite_index_stat(v3.as_slice(), &mut |_path: &[u8]| { return Some(fresh()); }));
    assert_eq!(updated, 2);
    assert_eq!(stored(output.as_slice(), 20).len(), 2);
    let v2: Vec<u8> = build(2, &[extended], &[], Checksum::Sha1);
    assert!(skipped(rewrite_index_stat(v2.as_slice(), &mut |_path: &[u8]| { return Some(fresh()); })).contains("documented layout"));
}

/// Version 4 paths are compressed against the previous one; the lookup still receives the whole path.
#[test]
fn version_4_paths_are_expanded_before_the_lookup() {
    let input: Vec<u8> = build(4, &[regular("dir/alpha"), regular("dir/beta"), regular("other")], &[], Checksum::Sha1);
    let mut asked: Vec<Vec<u8>> = Vec::new();
    let (output, updated, _total) = rewritten(rewrite_index_stat(input.as_slice(), &mut |path: &[u8]| {
        asked.push(path.to_vec());
        return Some(fresh());
    }));
    assert_eq!(asked, vec![b"dir/alpha".to_vec(), b"dir/beta".to_vec(), b"other".to_vec()]);
    assert_eq!(updated, 3);
    assert_eq!(output.len(), input.len());
}

/// A SHA-256 index is recognized by its checksum and rewritten with one; an all-zero checksum stays all zero.
#[test]
fn both_hash_lengths_and_the_zero_checksum_are_kept() {
    let wide: Vec<u8> = build(2, &[regular("a")], &[], Checksum::Sha256);
    let (output, updated, _total) = rewritten(rewrite_index_stat(wide.as_slice(), &mut |_path: &[u8]| { return Some(fresh()); }));
    assert_eq!(updated, 1);
    assert_eq!(&output[output.len() - 32..], sha256(&output[..output.len() - 32]).as_slice());
    assert_eq!(stored(output.as_slice(), 32), vec![(b"a".to_vec(), REWRITTEN)]);
    let zero: Vec<u8> = build(2, &[regular("a")], &[], Checksum::Zero20);
    let (kept_zero, zero_updated, _zero_total) = rewritten(rewrite_index_stat(zero.as_slice(), &mut |_path: &[u8]| { return Some(fresh()); }));
    assert_eq!(zero_updated, 1);
    assert_eq!(&kept_zero[kept_zero.len() - 20..], &[0u8; 20]);
}

/// The derived extensions are dropped, the others kept in order, and the checksum covers what remains.
#[test]
fn derived_extensions_are_dropped_and_the_others_kept() {
    let extensions: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"TREE", vec![1]),
        (b"UNTR", vec![2, 2]),
        (b"REUC", vec![3]),
        (b"FSMN", vec![4]),
        (b"EOIE", vec![5; 24]),
        (b"IEOT", vec![6; 8]),
    ];
    let input: Vec<u8> = build(2, &[regular("a")], extensions.as_slice(), Checksum::Sha1);
    let (output, _updated, _total) = rewritten(rewrite_index_stat(input.as_slice(), &mut |_path: &[u8]| { return Some(fresh()); }));
    let tail: &[u8] = &output[HEADER + 64..output.len() - 20];
    let signatures: Vec<&[u8]> = vec![&tail[..4], &tail[9..13]];
    assert_eq!(signatures, vec![b"TREE".as_slice(), b"REUC".as_slice()]);
    assert_eq!(tail.len(), 9 + 9);
    assert_eq!(&output[output.len() - 20..], sha1(&output[..output.len() - 20]).as_slice());
}

/// A split or sparse index changes what the entries mean, so the rewrite declines and names the extension.
#[test]
fn split_and_sparse_indexes_are_declined() {
    for signature in [b"link", b"sdir"] {
        let input: Vec<u8> = build(2, &[regular("a")], &[(signature, vec![0])], Checksum::Sha1);
        let reason: String = skipped(rewrite_index_stat(input.as_slice(), &mut |_path: &[u8]| { return Some(fresh()); }));
        assert!(reason.contains(String::from_utf8_lossy(signature).as_ref()), "{reason}");
    }
}

/// Every malformed shape is declined with a reason and never panics.
#[test]
fn malformed_files_are_declined_with_a_reason() {
    let good: Vec<u8> = build(2, &[regular("a")], &[], Checksum::Sha1);
    let lookup = |_path: &[u8]| -> Option<StatFields> { return Some(fresh()); };
    let mut not_an_index: Vec<u8> = good.clone();
    not_an_index[0] = b'X';
    assert!(skipped(rewrite_index_stat(not_an_index.as_slice(), &mut { lookup })).contains("no DIRC signature"));
    assert!(skipped(rewrite_index_stat(b"DIR", &mut { lookup })).contains("no DIRC signature"));
    for version in [1u32, 5] {
        let mut wrong: Vec<u8> = good.clone();
        wrong[4..8].copy_from_slice(&version.to_be_bytes());
        assert!(skipped(rewrite_index_stat(wrong.as_slice(), &mut { lookup })).contains(format!("version {version}").as_str()));
    }
    let mut corrupt: Vec<u8> = good.clone();
    corrupt[20] ^= 1;
    assert!(skipped(rewrite_index_stat(corrupt.as_slice(), &mut { lookup })).contains("checksum matches neither"));
    let mut cut_entry: Vec<u8> = build(2, &[regular("a")], &[], Checksum::Zero20);
    cut_entry[8..12].copy_from_slice(&5u32.to_be_bytes());
    assert!(skipped(rewrite_index_stat(cut_entry.as_slice(), &mut { lookup })).contains("checksum matches neither"));
    // An all-zero checksum proves nothing, so a file that does not parse to its end under a hash length is declined.
    let zero_with_cut_extension: Vec<u8> = {
        let mut bytes: Vec<u8> = build(2, &[regular("a")], &[], Checksum::Zero20);
        let at: usize = bytes.len() - 20;
        bytes.splice(at..at, *b"TREE\0\0\0");
        bytes
    };
    assert!(skipped(rewrite_index_stat(zero_with_cut_extension.as_slice(), &mut { lookup })).contains("checksum matches neither"));
    // With a checksum that verifies, a cut-off or oversized extension is reported as such.
    let with_checksum = |extension: Vec<u8>| -> Vec<u8> {
        let mut bytes: Vec<u8> = build(2, &[regular("a")], &[], Checksum::Sha1);
        bytes.truncate(bytes.len() - 20);
        bytes.extend_from_slice(extension.as_slice());
        let digest: [u8; 20] = sha1(bytes.as_slice());
        bytes.extend_from_slice(&digest);
        return bytes;
    };
    let cut_extension: Vec<u8> = with_checksum(b"TREE\0\0\0".to_vec());
    assert!(skipped(rewrite_index_stat(cut_extension.as_slice(), &mut { lookup })).contains("an extension header is cut off") || skipped(rewrite_index_stat(cut_extension.as_slice(), &mut { lookup })).contains("runs past the end"));
    let long_extension: Vec<u8> = {
        let mut extension: Vec<u8> = b"TREE".to_vec();
        extension.extend_from_slice(&1000u32.to_be_bytes());
        with_checksum(extension)
    };
    assert!(skipped(rewrite_index_stat(long_extension.as_slice(), &mut { lookup })).contains("runs past the end"));
}

/// A file that is exactly a header is long enough to be judged by its signature and version, so the decline names what
///  is wrong with it (no checksum fits) and not the missing signature; one byte less cannot hold the signature.
#[test]
fn a_file_of_exactly_a_header_is_judged_by_its_content_and_one_byte_less_is_not_an_index() {
    let lookup = |_path: &[u8]| -> Option<StatFields> { return Some(fresh()); };
    let mut header: Vec<u8> = b"DIRC".to_vec();
    header.extend_from_slice(&2u32.to_be_bytes());
    header.extend_from_slice(&0u32.to_be_bytes());
    assert_eq!(header.len(), HEADER);
    let exact: String = skipped(rewrite_index_stat(header.as_slice(), &mut { lookup }));
    assert!(exact.contains("checksum matches neither") && !exact.contains("no DIRC signature"), "{exact}");
    let short: String = skipped(rewrite_index_stat(&header[..HEADER - 1], &mut { lookup }));
    assert!(short.contains("no DIRC signature"), "{short}");
}

/// The 124-byte SHA-256 index with an all-zero checksum whose entry walk also succeeds as a SHA-1 index: read as SHA-1 its
///  object name hides a second entry layout and an `UNTR` extension that ends exactly at the checksum.
fn ambiguous_sha256_index() -> Vec<u8> {
    let mut bytes: Vec<u8> = b"DIRC".to_vec();
    bytes.extend_from_slice(&2u32.to_be_bytes());
    bytes.extend_from_slice(&1u32.to_be_bytes());
    for value in [1u32, 2, 100, 7, 3, 4, 0o100_644, 8, 9, 10] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes.extend_from_slice(&[0u8; 20]);
    bytes.extend_from_slice(&[0x00, 0x01, 0x78, 0x00, b'U', b'N', b'T', b'R', 0x00, 0x00, 0x00, 0x14]);
    bytes.extend_from_slice(&4u16.to_be_bytes());
    bytes.extend_from_slice(b"real\0\0");
    bytes.extend_from_slice(&[0u8; 32]);
    assert_eq!(bytes.len(), 124);
    return bytes;
}

/// A zero checksum does not say which hash an index uses. A file that parses under both lengths is declined when the
///  repository's format is not known, and read with the repository's format when it is.
#[test]
fn an_ambiguous_zero_checksum_is_declined_unless_the_repository_hash_is_known() {
    let input: Vec<u8> = ambiguous_sha256_index();
    let lookup = |_path: &[u8]| -> Option<StatFields> { return Some(fresh()); };
    let reason: String = skipped(rewrite_index_stat(input.as_slice(), &mut { lookup }));
    assert!(reason.contains("both SHA-1 and SHA-256"), "{reason}");
    let known: RewriteContext = RewriteContext { hash_length: Some(32), index_mtime: None };
    let (output, updated, total) = rewritten(rewrite_index_stat_with(input.as_slice(), &known, &mut { lookup }, &mut |_path: &[u8], _stored: &StoredStat| { return true; }));
    assert_eq!((output.len(), updated, total), (124, 1, 1), "no byte of a SHA-256 index is dropped");
    assert_eq!(stored(output.as_slice(), 32)[0].0, b"real".to_vec());
    assert_eq!(&output[output.len() - 32..], &[0u8; 32], "an index written without a checksum keeps none");
}

/// An empty SHA-256 index with an all-zero checksum is read as SHA-256 on its own, and a known format that contradicts a
///  verifying checksum is declined.
#[test]
fn an_empty_skip_hash_sha256_index_and_a_contradicting_format_are_handled() {
    let empty: Vec<u8> = build(2, &[], &[], Checksum::Zero32);
    let lookup = |_path: &[u8]| -> Option<StatFields> { return Some(fresh()); };
    let (output, _updated, total) = rewritten(rewrite_index_stat(empty.as_slice(), &mut { lookup }));
    assert_eq!((output.len(), total), (empty.len(), 0));
    let sha1_index: Vec<u8> = build(2, &[regular("a")], &[], Checksum::Sha1);
    let wrong: RewriteContext = RewriteContext { hash_length: Some(32), index_mtime: None };
    let reason: String = skipped(rewrite_index_stat_with(sha1_index.as_slice(), &wrong, &mut { lookup }, &mut |_path: &[u8], _stored: &StoredStat| { return true; }));
    assert!(reason.contains("does not match the repository's hash"), "{reason}");
}

/// An entry as new as the index file is racy and keeps its fields; an older one is rewritten; the reference check can
///  keep an entry too, and is given the fields the entry stores.
#[test]
fn racy_entries_and_untrusted_entries_keep_their_stat_fields() {
    let input: Vec<u8> = build(2, &[regular("README.md")], &[], Checksum::Sha1);
    let lookup = |_path: &[u8]| -> Option<StatFields> { return Some(fresh()); };
    let always = |_path: &[u8], _stored: &StoredStat| -> bool { return true; };
    for (index_seconds, expected) in [(100u32, 0usize), (101, 1), (99, 0)] {
        let context: RewriteContext = RewriteContext { hash_length: None, index_mtime: Some((index_seconds, 0)) };
        let (output, updated, _total) = rewritten(rewrite_index_stat_with(input.as_slice(), &context, &mut { lookup }, &mut { always }));
        assert_eq!(updated, expected, "index time {index_seconds}");
        let fields: [u32; 9] = stored(output.as_slice(), 20)[0].1;
        assert_eq!(fields, if expected == 1 { REWRITTEN } else { UNTOUCHED }, "index time {index_seconds}");
    }
    let mut seen: Vec<StoredStat> = Vec::new();
    let (_output, updated, _total) = rewritten(rewrite_index_stat_with(input.as_slice(), &RewriteContext::default(), &mut { lookup }, &mut |_path: &[u8], recorded: &StoredStat| {
        seen.push(*recorded);
        return false;
    }));
    assert_eq!(updated, 0);
    assert_eq!(seen, vec![StoredStat { ctime_seconds: 1, ctime_nanos: 2, mtime_seconds: 100, mtime_nanos: 7, ino: 4, uid: 8, gid: 9, size: 10 }]);
}

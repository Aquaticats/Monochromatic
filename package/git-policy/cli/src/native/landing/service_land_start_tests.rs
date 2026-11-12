use super::*;
use crate::test_support::{fixture, remove};

/// A witness survives the text it is recorded in,
/// and an absent file has one too.
#[test]
fn a_witness_round_trips_through_its_text() {
    let file: IndexWitness = IndexWitness::File { inode: 7, size: 9, modified: (1, 2), changed: (3, 4), trailer: String::from("00ff") };
    assert_eq!(decode(encode(&file).as_str()), Some(file));
    assert_eq!(decode(encode(&IndexWitness::Absent).as_str()), Some(IndexWitness::Absent));
}

/// A damaged text is no witness,
/// so a replay never reads half a record as the whole.
#[test]
fn a_damaged_text_is_no_witness() {
    for text in ["", "files:1:2:3:4:5:6:00", "file:1:2:3:4:5:6", "file:x:2:3:4:5:6:00", "file:1:2:3:4:5:6:zz", "file:1:2:3:4:5:6:00:extra"] {
        assert_eq!(decode(text), None, "{text:?}");
    }
}

/// The file a rename put at the path is another file even when it holds the same bytes,
/// and an untouched file is the same one.
/// An index every program replaces by rename is thereby told from one nobody wrote since.
#[test]
fn a_replacement_with_the_same_bytes_is_another_witness_and_an_untouched_file_is_the_same() {
    let root: std::path::PathBuf = fixture("service-land-witness");
    let index: std::path::PathBuf = root.join("index");
    assert_eq!(witness_of(index.as_path()).expect("absent"), IndexWitness::Absent);
    std::fs::write(&index, b"DIRC bytes of an index that ends in its checksum").expect("write");
    let first: IndexWitness = witness_of(index.as_path()).expect("first");
    assert_eq!(witness_of(index.as_path()).expect("again"), first, "an untouched file reads the same twice");
    let copy: std::path::PathBuf = root.join("index.copy");
    std::fs::copy(&index, &copy).expect("copy");
    std::fs::rename(&copy, &index).expect("replace by rename");
    let second: IndexWitness = witness_of(index.as_path()).expect("second");
    assert_ne!(second, first, "the same bytes under a new inode are a write");
    let IndexWitness::File { trailer, .. } = &second else { panic!("{second:?}") };
    assert_eq!(trailer.len(), 64, "the last bytes of a file of this size are kept in full");
    remove(root.as_path());
}

/// What:
///  The witness the awaited face reads is the one the synchronous face reads,
/// for a file that is there and for one that is not,
/// and a look no job gave in time is an error rather than an absent index.
/// Why:
///   A replay decides from this record whether it may write the index at all,
/// so a face that read another witness would replay over staging it never saw,
/// and one that answered "absent" for a look nobody made would discard staging.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await witnessOfAwaiting(file)).toEqual(witnessOf(file)); expect(await witnessOfAwaiting(gone)).toEqual(witnessOf(gone))
/// ```
#[test]
fn the_awaited_witness_is_the_one_the_synchronous_face_reads() {
    let root: std::path::PathBuf = fixture("service-land-witness-awaiting");
    let index: std::path::PathBuf = root.join("index");
    std::fs::write(&index, b"DIRC bytes of an index that ends in its checksum").expect("write");
    let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let awaited: std::io::Result<IndexWitness> = runtime.block_on(witness_of_awaiting(index.as_path()));
    assert_eq!(awaited.expect("the awaited look"), witness_of(index.as_path()).expect("the synchronous look"), "the two faces read one witness");
    let gone: std::path::PathBuf = root.join("not-an-index");
    let awaited_gone: std::io::Result<IndexWitness> = runtime.block_on(witness_of_awaiting(gone.as_path()));
    assert_eq!(awaited_gone.expect("the awaited look at a file that is not there"), witness_of(gone.as_path()).expect("the synchronous look"), "a file that is not there is absent on both faces");
    remove(root.as_path());
}

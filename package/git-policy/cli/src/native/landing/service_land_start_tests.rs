use super::*;
use crate::test_support::{fixture, remove};

/// A witness survives the text it is recorded in, and an absent file has one too.
#[test]
fn a_witness_round_trips_through_its_text() {
    let file: IndexWitness = IndexWitness::File { inode: 7, size: 9, modified: (1, 2), changed: (3, 4), trailer: String::from("00ff") };
    assert_eq!(decode(encode(&file).as_str()), Some(file));
    assert_eq!(decode(encode(&IndexWitness::Absent).as_str()), Some(IndexWitness::Absent));
}

/// A damaged text is no witness, so a replay never reads half a record as the whole.
#[test]
fn a_damaged_text_is_no_witness() {
    for text in ["", "files:1:2:3:4:5:6:00", "file:1:2:3:4:5:6", "file:x:2:3:4:5:6:00", "file:1:2:3:4:5:6:zz", "file:1:2:3:4:5:6:00:extra"] {
        assert_eq!(decode(text), None, "{text:?}");
    }
}

/// The file a rename put at the path is another file even when it holds the same bytes, and an untouched file is the same one.
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

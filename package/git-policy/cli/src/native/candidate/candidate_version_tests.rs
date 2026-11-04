//! What:
//!  Controls for building and reading a candidate version.
//! Why:
//!  Identity must equal list position,
//!  duplicates must resolve to the first record,
//!      and lookups by pathname must match bytes exactly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(buildVersion(3n, records).candidates[1].identity).toEqual({ generation: 3n, index: 1 });
//! ```

/// Import the builder under test and the values it produces.
use super::{Candidate, CandidateIdentity, CandidateVersion, build_version};
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};
use crate::candidate_record::{CandidateChange, CandidateRecord};

/// A validated object name made of one repeated digit.
fn object(digit: u8) -> ObjectId {
    return parse_object_id(&[digit; 40]).expect("fixture name");
}

/// One added regular-file record.
fn added(path: &[u8], digit: u8) -> CandidateRecord {
    return CandidateRecord {
        path: path.to_vec(),
        mode: CandidateMode::Regular,
        change: CandidateChange::Added,
        object: Some(object(digit)),
    };
}

/// Candidates keep listing order,
///  and each identity is the generation plus the list position.
#[test]
fn identity_is_generation_and_position() {
    let deleted: CandidateRecord = CandidateRecord {
        path: b"gone".to_vec(),
        mode: CandidateMode::Executable,
        change: CandidateChange::Deleted,
        object: None,
    };
    let version: CandidateVersion =
        build_version(7, vec![added(b"b", b'1'), deleted, added(b"a\xff", b'2')]);
    assert_eq!(
        version.candidates(),
        [
            Candidate {
                identity: CandidateIdentity {
                    generation: 7,
                    index: 0
                },
                path: b"b".to_vec(),
                mode: CandidateMode::Regular,
                change: CandidateChange::Added,
                object: Some(object(b'1')),
            },
            Candidate {
                identity: CandidateIdentity {
                    generation: 7,
                    index: 1
                },
                path: b"gone".to_vec(),
                mode: CandidateMode::Executable,
                change: CandidateChange::Deleted,
                object: None,
            },
            Candidate {
                identity: CandidateIdentity {
                    generation: 7,
                    index: 2
                },
                path: b"a\xff".to_vec(),
                mode: CandidateMode::Regular,
                change: CandidateChange::Added,
                object: Some(object(b'2')),
            },
        ]
    );
}

/// The first record of a pathname wins;
///  later ones neither replace it nor take a position.
#[test]
fn first_record_of_a_pathname_wins() {
    let version: CandidateVersion = build_version(
        0,
        vec![
            added(b"same", b'1'),
            added(b"other", b'2'),
            added(b"same", b'3'),
            added(b"last", b'4'),
        ],
    );
    assert_eq!(version.candidates().len(), 3);
    assert_eq!(version.candidates()[0].object, Some(object(b'1')));
    assert_eq!(version.candidates()[2].path, b"last");
    // Positions stay dense after the duplicate was skipped.
    assert_eq!(version.candidates()[2].identity.index, 2);
    assert_eq!(
        version
            .candidate_at_path(b"same")
            .expect("first record")
            .object,
        Some(object(b'1'))
    );
}

/// Lookup is by exact bytes:
///  a present pathname returns its own candidate,
///  anything else nothing.
#[test]
fn lookup_matches_exact_pathname_bytes() {
    let version: CandidateVersion =
        build_version(1, vec![added(b"dir/file", b'1'), added(b"n\xffame", b'2')]);
    for (position, candidate) in version.candidates().iter().enumerate() {
        let found: &Candidate = version
            .candidate_at_path(candidate.path.as_slice())
            .expect("listed pathname");
        assert_eq!(found.identity.index, position);
        assert_eq!(found, candidate);
    }
    for absent in [
        b"dir".as_slice(),
        b"dir/",
        b"dir/file ",
        b"DIR/FILE",
        b"name",
        b"",
    ] {
        assert!(version.candidate_at_path(absent).is_none(), "{absent:?}");
    }
    // A version without candidates answers every lookup with nothing.
    let empty: CandidateVersion = build_version(1, Vec::new());
    assert!(empty.candidates().is_empty());
    assert!(empty.candidate_at_path(b"dir/file").is_none());
}

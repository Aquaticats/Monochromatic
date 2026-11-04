//! What:
//!  Controls for the stage-record parser,
//!  the delta between two index states and the
//!       two ways of turning entries into candidate records.
//! Why:
//!  A record misread,
//!  a changed path missed or a deletion invented would let staged
//!      content go unchecked or report content that is not staged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(stagedDelta(before, after).map(p => p.path)).toEqual(['b', 'a']);
//! ```

/// Import the module under test.
use super::{
    ChangedPath, StageRecord, delta_candidates, parse_stage_records, scope_candidates, staged_delta,
};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};
use crate::candidate_record::{CandidateChange, CandidateRecord};

/// An object name made of one repeated hexadecimal digit.
fn object(digit: u8) -> ObjectId {
    return parse_object_id(&[digit; 40]).expect("forty hexadecimal digits");
}

/// A regular-file record at `stage`.
fn record(path: &str, digit: u8, stage: u8) -> StageRecord {
    return StageRecord {
        mode: CandidateMode::Regular,
        object: object(digit),
        stage,
        path: path.as_bytes().to_vec(),
    };
}

/// The changed pathnames as text,
///  in order.
fn changed_paths(changed: &[ChangedPath]) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for path in changed {
        paths.push(String::from_utf8_lossy(path.path.as_slice()).into_owned());
    }
    return paths;
}

/// A candidate record of the baseline listing.
fn baseline(path: &str, change: CandidateChange) -> CandidateRecord {
    let object: Option<ObjectId> = if change == CandidateChange::Deleted {
        None
    } else {
        Some(object(b'e'))
    };
    return CandidateRecord {
        path: path.as_bytes().to_vec(),
        mode: CandidateMode::Executable,
        change,
        object,
    };
}

/// The failure a call returned,
///  which the control requires.
fn failure<T: std::fmt::Debug>(result: Result<T, CandidateError>) -> CandidateError {
    match result {
        Ok(value) => panic!("expected a failure, got {value:?}"),
        Err(error) => return error,
    }
}

/// Git's listing format is read field by field,
///  a tab inside a pathname included.
#[test]
fn stage_records_are_read_from_git_output() {
    assert_eq!(parse_stage_records(b"").expect("empty listing"), Vec::new());
    let sha256: String = "b".repeat(64);
    let output: Vec<u8> = [
        format!("100644 {} 0\tplain.txt\0", "a".repeat(40)).as_bytes(),
        format!("100755 {sha256} 2\twith\ttab\0").as_bytes(),
        format!("120000 {} 3\t", "c".repeat(40)).as_bytes(),
        b"\xff-not-utf8\0".as_slice(),
        format!("160000 {} 1\tsub\0", "d".repeat(40)).as_bytes(),
    ]
    .concat();
    let records: Vec<StageRecord> = parse_stage_records(output.as_slice()).expect("listing");
    assert_eq!(
        records,
        vec![
            record("plain.txt", b'a', 0),
            StageRecord {
                mode: CandidateMode::Executable,
                object: parse_object_id(sha256.as_bytes()).expect("SHA-256 name"),
                stage: 2,
                path: b"with\ttab".to_vec(),
            },
            StageRecord {
                mode: CandidateMode::Symlink,
                object: object(b'c'),
                stage: 3,
                path: b"\xff-not-utf8".to_vec(),
            },
            StageRecord {
                mode: CandidateMode::Gitlink,
                object: object(b'd'),
                stage: 1,
                path: b"sub".to_vec(),
            },
        ]
    );
}

/// Output that is not a complete stage listing is refused whole,
///  naming the record by position.
#[test]
fn malformed_stage_listings_are_refused() {
    let name: String = "a".repeat(40);
    let cases: Vec<(Vec<u8>, CandidateFailure, &str)> = vec![
        (
            format!("100644 {name} 0\tcut").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 is cut short",
        ),
        (
            format!("100644 {name} 0 no-tab\0").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 has no tab",
        ),
        (
            format!("100644 {name} 0\tok\0100644 {name}\tshort\0").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 1 does not have three metadata fields",
        ),
        (
            format!("100644 {name} 0 extra\tlong\0").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 does not have three metadata fields",
        ),
        (
            format!("100644 {name} 0\t\0").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 does not have three metadata fields and a pathname",
        ),
        (
            format!("040000 {name} 0\tdir\0").into_bytes(),
            CandidateFailure::UnsupportedMode,
            "record 0 has a file mode",
        ),
        (
            format!("100644 {} 0\tshort-name\0", "a".repeat(39)).into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 does not carry a complete object name",
        ),
        (
            format!("100644 {name} 4\tstage-four\0").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 does not carry a stage",
        ),
        (
            format!("100644 {name} 00\ttwo-digits\0").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 does not carry a stage",
        ),
        (
            format!("100644 {name} \tno-stage\0").into_bytes(),
            CandidateFailure::ListingMalformed,
            "record 0 does not carry a stage",
        ),
    ];
    for (output, cause, wording) in cases {
        let error: CandidateError = failure(parse_stage_records(output.as_slice()));
        assert_eq!(error.failure, cause, "{wording}");
        assert!(
            error.message.contains(wording),
            "{wording}: {}",
            error.message
        );
        assert!(
            !error.message.contains("no-tab")
                && !error.message.contains("short-name")
                && !error.message.contains("stage-four"),
            "a pathname reached the message: {}",
            error.message
        );
    }
}

/// Unchanged states have no delta;
///  each kind of change is found and reported in the installed wrapper's order.
#[test]
fn the_delta_holds_exactly_the_changed_paths_in_report_order() {
    let before: Vec<StageRecord> = vec![
        record("a.txt", b'1', 0),
        record("m.txt", b'2', 0),
        record("r.txt", b'3', 0),
        record("z.txt", b'4', 0),
    ];
    assert_eq!(
        staged_delta(before.as_slice(), before.as_slice()),
        Vec::new()
    );
    let after: Vec<StageRecord> = vec![
        record("a.txt", b'1', 0),
        record("b-new.txt", b'5', 0),
        record("m.txt", b'6', 0),
        record("y-new.txt", b'7', 0),
        record("z.txt", b'8', 0),
    ];
    let changed: Vec<ChangedPath> = staged_delta(before.as_slice(), after.as_slice());
    // Paths the first state held come first, in its order; new paths follow in theirs.
    assert_eq!(
        changed_paths(changed.as_slice()),
        vec!["m.txt", "r.txt", "z.txt", "b-new.txt", "y-new.txt"]
    );
    assert_eq!(changed[0].after, vec![record("m.txt", b'6', 0)]);
    assert_eq!(
        changed[1].after,
        Vec::new(),
        "a removed path has no records"
    );
    assert_eq!(changed[2].after, vec![record("z.txt", b'8', 0)]);
    assert_eq!(changed[3].after, vec![record("b-new.txt", b'5', 0)]);
    // A mode change alone is a change.
    let mut mode_changed: Vec<StageRecord> = before.clone();
    mode_changed[0].mode = CandidateMode::Executable;
    assert_eq!(
        changed_paths(staged_delta(before.as_slice(), mode_changed.as_slice()).as_slice()),
        vec!["a.txt"]
    );
}

/// Conflict stages are compared as one list per path:
///  resolving counts,
///  an untouched conflict does not.
#[test]
fn conflict_stages_are_compared_per_path() {
    // Groups of one, three and one record, so every group boundary is crossed.
    let conflicted: Vec<StageRecord> = vec![
        record("before.txt", b'1', 0),
        record("conflict.txt", b'a', 1),
        record("conflict.txt", b'b', 2),
        record("conflict.txt", b'c', 3),
        record("other.txt", b'2', 0),
    ];
    assert_eq!(
        staged_delta(conflicted.as_slice(), conflicted.as_slice()),
        Vec::new()
    );
    let other_staged: Vec<StageRecord> = vec![
        record("before.txt", b'1', 0),
        record("conflict.txt", b'a', 1),
        record("conflict.txt", b'b', 2),
        record("conflict.txt", b'c', 3),
        record("other.txt", b'9', 0),
    ];
    assert_eq!(
        changed_paths(staged_delta(conflicted.as_slice(), other_staged.as_slice()).as_slice()),
        vec!["other.txt"]
    );
    let resolved: Vec<StageRecord> = vec![
        record("before.txt", b'1', 0),
        record("conflict.txt", b'd', 0),
        record("other.txt", b'2', 0),
    ];
    let changed: Vec<ChangedPath> = staged_delta(conflicted.as_slice(), resolved.as_slice());
    assert_eq!(changed_paths(changed.as_slice()), vec!["conflict.txt"]);
    assert_eq!(changed[0].after, vec![record("conflict.txt", b'd', 0)]);
    // One stage of a conflict changing is a change of the whole path.
    let mut one_stage: Vec<StageRecord> = conflicted.clone();
    one_stage[3].object = object(b'f');
    assert_eq!(
        changed_paths(staged_delta(conflicted.as_slice(), one_stage.as_slice()).as_slice()),
        vec!["conflict.txt"]
    );
    // A conflict at the start and at the end of the listing is grouped too.
    let edges: Vec<StageRecord> = vec![
        record("a.txt", b'a', 2),
        record("a.txt", b'b', 3),
        record("b.txt", b'c', 0),
        record("c.txt", b'd', 1),
        record("c.txt", b'e', 2),
    ];
    assert_eq!(staged_delta(edges.as_slice(), edges.as_slice()), Vec::new());
    let edge_resolved: Vec<StageRecord> = vec![
        record("a.txt", b'a', 2),
        record("a.txt", b'b', 3),
        record("b.txt", b'c', 0),
        record("c.txt", b'f', 0),
    ];
    let edge_changed: Vec<ChangedPath> = staged_delta(edges.as_slice(), edge_resolved.as_slice());
    assert_eq!(changed_paths(edge_changed.as_slice()), vec!["c.txt"]);
}

/// Present paths become additions or modifications by the baseline;
///  removed paths become deletions only when the baseline has them.
#[test]
fn delta_candidates_follow_the_baseline() {
    let changed: Vec<ChangedPath> = vec![
        ChangedPath {
            path: b"added.txt".to_vec(),
            after: vec![record("added.txt", b'1', 0)],
        },
        ChangedPath {
            path: b"modified.txt".to_vec(),
            after: vec![record("modified.txt", b'2', 0)],
        },
        ChangedPath {
            path: b"same-as-head.txt".to_vec(),
            after: vec![record("same-as-head.txt", b'3', 0)],
        },
        ChangedPath {
            path: b"deleted.txt".to_vec(),
            after: Vec::new(),
        },
        ChangedPath {
            path: b"never-committed.txt".to_vec(),
            after: Vec::new(),
        },
        ChangedPath {
            path: b"listed-but-not-deleted.txt".to_vec(),
            after: Vec::new(),
        },
    ];
    let listed: Vec<CandidateRecord> = vec![
        baseline("added.txt", CandidateChange::Added),
        baseline("modified.txt", CandidateChange::Modified),
        baseline("deleted.txt", CandidateChange::Deleted),
        baseline("listed-but-not-deleted.txt", CandidateChange::Modified),
    ];
    let candidates: Vec<CandidateRecord> =
        delta_candidates(changed.as_slice(), listed.as_slice()).expect("candidates");
    assert_eq!(
        candidates,
        vec![
            CandidateRecord {
                path: b"added.txt".to_vec(),
                mode: CandidateMode::Regular,
                change: CandidateChange::Added,
                object: Some(object(b'1')),
            },
            CandidateRecord {
                path: b"modified.txt".to_vec(),
                mode: CandidateMode::Regular,
                change: CandidateChange::Modified,
                object: Some(object(b'2')),
            },
            CandidateRecord {
                path: b"same-as-head.txt".to_vec(),
                mode: CandidateMode::Regular,
                change: CandidateChange::Modified,
                object: Some(object(b'3')),
            },
            baseline("deleted.txt", CandidateChange::Deleted),
        ]
    );
}

/// A path the operation leaves unmerged has no single content and stops the prediction.
#[test]
fn an_unmerged_changed_path_is_refused() {
    let unmerged: Vec<ChangedPath> = vec![
        ChangedPath {
            path: b"ok.txt".to_vec(),
            after: vec![record("ok.txt", b'1', 0)],
        },
        ChangedPath {
            path: b"secret-name.txt".to_vec(),
            after: vec![
                record("secret-name.txt", b'2', 2),
                record("secret-name.txt", b'3', 3),
            ],
        },
    ];
    let error: CandidateError = failure(delta_candidates(unmerged.as_slice(), &[]));
    assert_eq!(error.failure, CandidateFailure::UnmergedPath);
    assert!(
        error.message.contains("record 1 is left unmerged"),
        "{}",
        error.message
    );
    assert!(!error.message.contains("secret-name"), "{}", error.message);
    // A single entry at a conflict stage is not a merged entry either.
    let one_stage: Vec<ChangedPath> = vec![ChangedPath {
        path: b"stage.txt".to_vec(),
        after: vec![record("stage.txt", b'4', 1)],
    }];
    let single: CandidateError = failure(delta_candidates(one_stage.as_slice(), &[]));
    assert_eq!(single.failure, CandidateFailure::UnmergedPath);
    assert!(single.message.contains("record 0"), "{}", single.message);
}

/// Every merged entry of a scope is a candidate;
///  the baseline decides addition or modification.
#[test]
fn scope_candidates_hold_every_entry() {
    let entries: Vec<StageRecord> = vec![
        record("new.txt", b'1', 0),
        record("tracked.txt", b'2', 0),
        record("unchanged.txt", b'3', 0),
    ];
    let listed: Vec<CandidateRecord> = vec![
        baseline("new.txt", CandidateChange::Added),
        baseline("tracked.txt", CandidateChange::Modified),
    ];
    let candidates: Vec<CandidateRecord> =
        scope_candidates(entries.as_slice(), listed.as_slice()).expect("candidates");
    let mut changes: Vec<CandidateChange> = Vec::new();
    for candidate in &candidates {
        changes.push(candidate.change);
        assert_eq!(candidate.mode, CandidateMode::Regular);
    }
    assert_eq!(
        changes,
        vec![
            CandidateChange::Added,
            CandidateChange::Modified,
            CandidateChange::Modified
        ]
    );
    assert_eq!(candidates[2].object, Some(object(b'3')));
    assert_eq!(candidates[0].path, b"new.txt".to_vec());
    let conflicted: Vec<StageRecord> = vec![record("fine.txt", b'1', 0), record("c.txt", b'2', 3)];
    let error: CandidateError = failure(scope_candidates(conflicted.as_slice(), &[]));
    assert_eq!(error.failure, CandidateFailure::UnmergedPath);
    assert!(error.message.contains("record 1"), "{}", error.message);
}

//! Controls for journal record encoding:
//!  exact bytes and round trips through the parsers.

use super::*;
use crate::transaction_journal_parse::{
    parse_index_lock, parse_landing, parse_prepared, parse_preparing, parse_ref_updated,
};

/// A preparing record with every field set.
pub(crate) fn sample_preparing() -> PreparingRecord {
    return PreparingRecord {
        transaction_id: String::from("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10"),
        mode: TransactionMode::ExplicitPath,
        base: Base::Commit(String::from("1111111111111111111111111111111111111111")),
        symbolic_head: SymbolicHead::Branch(String::from("refs/heads/main")),
        target_ref: String::from("refs/heads/main"),
        conclusion: Conclusion::None,
        repository_root: String::from("/repo"),
        git_dir: String::from("/repo/.git"),
        common_dir: String::from("/repo/.git"),
        real_index_path: String::from("/repo/.git/index"),
        object_directory: String::from("/repo/.git/objects"),
        ref_format: RefFormat::Files,
        empty_tree_oid: String::from("4b825dc642cb6eb9a060e54bf8d69288fbee4904"),
        shadow_path: String::from("/repo/.git/cli-git/shadow/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10"),
        selected_pathspecs: vec![b"a.txt".to_vec(), b"dir/b \"c\".txt".to_vec()],
        invoked_at: String::from("2026-10-06T21:34:01.123Z"),
    };
}

/// An added-path record.
pub(crate) fn sample_added() -> AddedPath {
    return AddedPath {
        path: b"c.txt".to_vec(),
        git_mode: String::from("100644"),
        original_oid: String::from("2222222222222222222222222222222222222222"),
        intended_oid: String::from("3333333333333333333333333333333333333333"),
    };
}

/// A lock identity.
pub(crate) fn sample_lock() -> LockIdentity {
    return LockIdentity {
        file: FileIdentity {
            device: String::from("64769"),
            inode: String::from("1234567"),
        },
        fs_id: String::from("fs-uuid_0f4e"),
    };
}

/// `preparing.json` has the incumbent's field order and round-trips.
#[test]
fn preparing_records_round_trip() {
    let record: PreparingRecord = sample_preparing();
    let encoded: String = encode_preparing(&record);
    assert_eq!(
        encoded,
        "{\"schemaVersion\":2,\"state\":\"preparing\",\"transactionId\":\"0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10\",\
         \"mode\":\"explicit-path\",\"base\":{\"kind\":\"commit\",\"oid\":\"1111111111111111111111111111111111111111\"},\
         \"symbolicHead\":{\"kind\":\"branch\",\"ref\":\"refs/heads/main\"},\"targetRef\":\"refs/heads/main\",\
         \"conclusion\":\"none\",\"repositoryRoot\":\"/repo\",\"gitDir\":\"/repo/.git\",\"commonDir\":\"/repo/.git\",\
         \"realIndexPath\":\"/repo/.git/index\",\"objectDirectory\":\"/repo/.git/objects\",\"refFormat\":\"files\",\
         \"emptyTreeOid\":\"4b825dc642cb6eb9a060e54bf8d69288fbee4904\",\
         \"shadowPath\":\"/repo/.git/cli-git/shadow/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10\",\
         \"selectedPathspecs\":[\"a.txt\",\"dir/b \\\"c\\\".txt\"],\"invokedAt\":\"2026-10-06T21:34:01.123Z\"}\n"
    );
    assert_eq!(
        parse_preparing(encoded.as_bytes()).expect("round trip"),
        record
    );
    for (mode, conclusion, format, base, head) in [
        (
            TransactionMode::Index,
            Conclusion::Amend,
            RefFormat::Reftable,
            Base::Unborn,
            SymbolicHead::Detached,
        ),
        (
            TransactionMode::Index,
            Conclusion::Merge,
            RefFormat::Files,
            Base::Unborn,
            SymbolicHead::Detached,
        ),
        (
            TransactionMode::Index,
            Conclusion::CherryPick,
            RefFormat::Files,
            Base::Unborn,
            SymbolicHead::Detached,
        ),
        (
            TransactionMode::Index,
            Conclusion::Revert,
            RefFormat::Files,
            Base::Unborn,
            SymbolicHead::Detached,
        ),
    ] {
        let variant: PreparingRecord = PreparingRecord {
            mode,
            conclusion,
            ref_format: format,
            base,
            symbolic_head: head,
            ..sample_preparing()
        };
        let text: String = encode_preparing(&variant);
        assert_eq!(
            parse_preparing(text.as_bytes()).expect("variant"),
            variant,
            "{text}"
        );
    }
}

/// The wire names are the incumbent's.
#[test]
fn wire_names_are_the_incumbents() {
    assert_eq!(mode_name(TransactionMode::ExplicitPath), "explicit-path");
    assert_eq!(mode_name(TransactionMode::Index), "index");
    assert_eq!(conclusion_name(Conclusion::None), "none");
    assert_eq!(conclusion_name(Conclusion::Amend), "amend");
    assert_eq!(conclusion_name(Conclusion::Merge), "merge");
    assert_eq!(conclusion_name(Conclusion::CherryPick), "cherry-pick");
    assert_eq!(conclusion_name(Conclusion::Revert), "revert");
    assert_eq!(ref_format_name(RefFormat::Files), "files");
    assert_eq!(ref_format_name(RefFormat::Reftable), "reftable");
    assert_eq!(encode_base(&Base::Unborn), "{\"kind\":\"unborn\"}");
}

/// `prepared.json` encodes its lists and round-trips.
#[test]
fn prepared_records_round_trip() {
    let record: PreparedRecord = PreparedRecord {
        shadow_path: String::from("/repo/.git/cli-git/shadow/x"),
        prepared_oid: String::from("4444444444444444444444444444444444444444"),
        signed: true,
        intended_tree_oid: String::from("5555555555555555555555555555555555555555"),
        committed_paths: vec![b"a.txt".to_vec(), b"c.txt".to_vec()],
        added_paths: vec![sample_added()],
        selected_worktree_paths: Vec::new(),
    };
    let encoded: String = encode_prepared(&record);
    assert_eq!(
        encoded,
        "{\"schemaVersion\":2,\"state\":\"prepared\",\"shadowPath\":\"/repo/.git/cli-git/shadow/x\",\
         \"preparedOid\":\"4444444444444444444444444444444444444444\",\"signed\":true,\
         \"intendedTreeOid\":\"5555555555555555555555555555555555555555\",\"committedPaths\":[\"a.txt\",\"c.txt\"],\
         \"addedPaths\":[{\"path\":\"c.txt\",\"gitMode\":\"100644\",\
         \"originalOid\":\"2222222222222222222222222222222222222222\",\
         \"intendedOid\":\"3333333333333333333333333333333333333333\"}],\"selectedWorktreePaths\":[]}\n"
    );
    assert_eq!(
        parse_prepared(encoded.as_bytes()).expect("round trip"),
        record
    );
    let unsigned: PreparedRecord = PreparedRecord {
        signed: false,
        added_paths: vec![sample_added(), sample_added()],
        selected_worktree_paths: vec![sample_added()],
        ..record
    };
    assert_eq!(
        parse_prepared(encode_prepared(&unsigned).as_bytes()).expect("unsigned"),
        unsigned
    );
}

/// `index-lock-<n>.json` round-trips.
#[test]
fn index_lock_records_round_trip() {
    let record: IndexLockRecord = IndexLockRecord {
        attempt: 3,
        lock: sample_lock(),
    };
    let encoded: String = encode_index_lock(&record);
    assert_eq!(
        encoded,
        "{\"schemaVersion\":2,\"state\":\"index-locked\",\"attempt\":3,\
         \"lock\":{\"device\":\"64769\",\"inode\":\"1234567\",\"fsId\":\"fs-uuid_0f4e\"}}\n"
    );
    assert_eq!(
        parse_index_lock(encoded.as_bytes()).expect("round trip"),
        record
    );
}

/// `landing-<n>.json` omits absent optional fields and round-trips both operations.
#[test]
fn landing_records_round_trip() {
    let record: LandingRecord = LandingRecord {
        attempt: 1,
        operation: LandingOperation::Commit,
        expected_old: Base::Commit(String::from("1111111111111111111111111111111111111111")),
        new_oid: Some(String::from("6666666666666666666666666666666666666666")),
        landed_tree_oid: String::from("7777777777777777777777777777777777777777"),
        pre_landing_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("2"),
        },
        post_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("3"),
        },
        lock: sample_lock(),
        pack_name: Some(String::from("8888888888888888888888888888888888888888")),
        added_paths: Vec::new(),
        selected_worktree_paths: vec![sample_added()],
        pre_landing_digest: None,
        post_digest: None,
    };
    let encoded: String = encode_landing(&record);
    assert_eq!(
        encoded,
        "{\"schemaVersion\":2,\"state\":\"landing\",\"attempt\":1,\"operation\":\"commit\",\
         \"expectedOld\":{\"kind\":\"commit\",\"oid\":\"1111111111111111111111111111111111111111\"},\
         \"newOid\":\"6666666666666666666666666666666666666666\",\
         \"landedTreeOid\":\"7777777777777777777777777777777777777777\",\
         \"preLandingIndex\":{\"device\":\"1\",\"inode\":\"2\"},\"postIndex\":{\"device\":\"1\",\"inode\":\"3\"},\
         \"lock\":{\"device\":\"64769\",\"inode\":\"1234567\",\"fsId\":\"fs-uuid_0f4e\"},\
         \"packName\":\"8888888888888888888888888888888888888888\",\"addedPaths\":[],\
         \"selectedWorktreePaths\":[{\"path\":\"c.txt\",\"gitMode\":\"100644\",\
         \"originalOid\":\"2222222222222222222222222222222222222222\",\
         \"intendedOid\":\"3333333333333333333333333333333333333333\"}]}\n"
    );
    assert_eq!(
        parse_landing(encoded.as_bytes()).expect("round trip"),
        record
    );
    let normalization: LandingRecord = LandingRecord {
        operation: LandingOperation::NormalizeOnly,
        new_oid: None,
        pack_name: None,
        expected_old: Base::Unborn,
        ..record
    };
    let text: String = encode_landing(&normalization);
    assert!(!text.contains("newOid") && !text.contains("packName"));
    assert!(text.contains("\"operation\":\"normalize-only\""));
    assert_eq!(
        parse_landing(text.as_bytes()).expect("normalization"),
        normalization
    );
}

/// The snapshot digests are the last keys of `landing-<n>.json`, print as a length and 16 hexadecimal digits, and
///  read back exactly; a record without them still reads, with none.
#[test]
fn landing_digests_round_trip_after_the_older_fields() {
    let record: LandingRecord = LandingRecord {
        attempt: 1,
        operation: LandingOperation::Commit,
        expected_old: Base::Unborn,
        new_oid: Some(String::from("6666666666666666666666666666666666666666")),
        landed_tree_oid: String::from("7777777777777777777777777777777777777777"),
        pre_landing_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("2"),
        },
        post_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("3"),
        },
        lock: sample_lock(),
        pack_name: None,
        added_paths: Vec::new(),
        selected_worktree_paths: Vec::new(),
        pre_landing_digest: Some(crate::content_digest::digest_bytes(b"pre")),
        post_digest: Some(crate::content_digest::digest_bytes(b"post")),
    };
    let encoded: String = encode_landing(&record);
    assert!(
        encoded.ends_with(&format!(
            "\"selectedWorktreePaths\":[],\"preLandingDigest\":{{\"length\":3,\"fnv1a64\":\"{}\"}},\
             \"postDigest\":{{\"length\":4,\"fnv1a64\":\"{}\"}}}}\n",
            crate::content_digest::digest_bytes(b"pre").hash_text(),
            crate::content_digest::digest_bytes(b"post").hash_text(),
        )),
        "{encoded}"
    );
    assert_eq!(parse_landing(encoded.as_bytes()).expect("round trip"), record);
    let without: LandingRecord = LandingRecord {
        pre_landing_digest: None,
        post_digest: None,
        ..record
    };
    let older: String = encode_landing(&without);
    assert!(!older.contains("Digest"), "{older}");
    assert_eq!(parse_landing(older.as_bytes()).expect("older record"), without);
}

/// `ref-updated.json` names the landed commit.
#[test]
fn ref_updated_records_round_trip() {
    let encoded: String = encode_ref_updated("9999999999999999999999999999999999999999");
    assert_eq!(
        encoded,
        "{\"schemaVersion\":2,\"state\":\"ref-updated\",\"landedOid\":\"9999999999999999999999999999999999999999\"}\n"
    );
    assert_eq!(
        parse_ref_updated(encoded.as_bytes()).expect("round trip"),
        "9999999999999999999999999999999999999999"
    );
}

/// A name that is not UTF-8 is written as its readable text with the base64 of the exact bytes beside it:
///  every entry of a list, one field in a record, and only when some name needs it.
#[test]
fn names_that_are_not_utf8_are_written_with_their_exact_bytes() {
    let record: PreparedRecord = PreparedRecord {
        shadow_path: String::from("/repo/.git/cli-git/shadow/x"),
        prepared_oid: String::from("4444444444444444444444444444444444444444"),
        signed: false,
        intended_tree_oid: String::from("5555555555555555555555555555555555555555"),
        committed_paths: vec![b"a.txt".to_vec(), b"caf\xe9.txt".to_vec()],
        added_paths: vec![AddedPath {
            path: b"caf\xe9.txt".to_vec(),
            ..sample_added()
        }],
        selected_worktree_paths: vec![sample_added()],
    };
    let encoded: String = encode_prepared(&record);
    assert_eq!(
        encoded,
        "{\"schemaVersion\":2,\"state\":\"prepared\",\"shadowPath\":\"/repo/.git/cli-git/shadow/x\",\
         \"preparedOid\":\"4444444444444444444444444444444444444444\",\"signed\":false,\
         \"intendedTreeOid\":\"5555555555555555555555555555555555555555\",\
         \"committedPaths\":[\"a.txt\",\"caf\u{fffd}.txt\"],\"committedPathBytes\":[\"YS50eHQ=\",\"Y2Fm6S50eHQ=\"],\
         \"addedPaths\":[{\"path\":\"caf\u{fffd}.txt\",\"pathBytes\":\"Y2Fm6S50eHQ=\",\"gitMode\":\"100644\",\
         \"originalOid\":\"2222222222222222222222222222222222222222\",\
         \"intendedOid\":\"3333333333333333333333333333333333333333\"}],\
         \"selectedWorktreePaths\":[{\"path\":\"c.txt\",\"gitMode\":\"100644\",\
         \"originalOid\":\"2222222222222222222222222222222222222222\",\
         \"intendedOid\":\"3333333333333333333333333333333333333333\"}]}\n"
    );
    assert_eq!(parse_prepared(encoded.as_bytes()).expect("round trip"), record);
    let preparing: PreparingRecord = PreparingRecord {
        selected_pathspecs: vec![b"caf\xe9.txt".to_vec()],
        ..sample_preparing()
    };
    let text: String = encode_preparing(&preparing);
    assert!(text.contains("\"selectedPathspecs\":[\"caf\u{fffd}.txt\"],\"selectedPathspecBytes\":[\"Y2Fm6S50eHQ=\"],\"invokedAt\""), "{text}");
    assert_eq!(parse_preparing(text.as_bytes()).expect("preparing"), preparing);
    let landing: LandingRecord = LandingRecord {
        attempt: 1,
        operation: LandingOperation::NormalizeOnly,
        expected_old: Base::Unborn,
        new_oid: None,
        landed_tree_oid: String::from("7777777777777777777777777777777777777777"),
        pre_landing_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("2"),
        },
        post_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("3"),
        },
        lock: sample_lock(),
        pack_name: None,
        added_paths: Vec::new(),
        selected_worktree_paths: record.added_paths.clone(),
        pre_landing_digest: None,
        post_digest: None,
    };
    assert_eq!(parse_landing(encode_landing(&landing).as_bytes()).expect("landing"), landing);
}

/// A list whose names are all UTF-8 has no byte list, so the incumbent's records stay as they were.
#[test]
fn names_that_are_utf8_get_no_byte_fields() {
    let preparing: String = encode_preparing(&sample_preparing());
    assert!(!preparing.contains("Bytes"), "{preparing}");
    let prepared: String = encode_prepared(&PreparedRecord {
        shadow_path: String::from("s"),
        prepared_oid: String::from("p"),
        signed: false,
        intended_tree_oid: String::from("t"),
        committed_paths: vec!["café.txt".as_bytes().to_vec(), "a\\b.txt".as_bytes().to_vec(), "\u{fffd}.txt".as_bytes().to_vec()],
        added_paths: vec![AddedPath {
            path: "café.txt".as_bytes().to_vec(),
            ..sample_added()
        }],
        selected_worktree_paths: Vec::new(),
    });
    assert!(!prepared.contains("Bytes"), "{prepared}");
}

/// A record whose byte fields disagree with the readable text, or are not what the writer prints, is refused.
#[test]
fn damaged_byte_fields_are_refused() {
    let good: String = encode_prepared(&PreparedRecord {
        shadow_path: String::from("s"),
        prepared_oid: String::from("p"),
        signed: false,
        intended_tree_oid: String::from("t"),
        committed_paths: vec![b"caf\xe9.txt".to_vec()],
        added_paths: vec![AddedPath {
            path: b"caf\xe9.txt".to_vec(),
            ..sample_added()
        }],
        selected_worktree_paths: Vec::new(),
    });
    assert!(parse_prepared(good.as_bytes()).is_ok());
    let refused = |changed: String| -> String {
        return parse_prepared(changed.as_bytes()).expect_err("refused").0;
    };
    // Bytes whose readable text is another name.
    assert_eq!(
        refused(good.replace("\"committedPathBytes\":[\"Y2Fm6S50eHQ=\"]", "\"committedPathBytes\":[\"YS50eHQ=\"]")),
        "Transaction record prepared.json has a malformed committedPathBytes field."
    );
    assert_eq!(
        refused(good.replace("\"pathBytes\":\"Y2Fm6S50eHQ=\"", "\"pathBytes\":\"YS50eHQ=\"")),
        "Transaction record prepared.json#addedPaths has a malformed pathBytes field."
    );
    // Text that is not padded base64.
    assert_eq!(
        refused(good.replace("\"committedPathBytes\":[\"Y2Fm6S50eHQ=\"]", "\"committedPathBytes\":[\"Y2Fm6S50eHQ\"]")),
        "Transaction record prepared.json has a malformed committedPathBytes field."
    );
    // A list of another length, and an entry that is not text.
    assert_eq!(
        refused(good.replace("\"committedPathBytes\":[\"Y2Fm6S50eHQ=\"]", "\"committedPathBytes\":[]")),
        "Transaction record prepared.json has a malformed committedPathBytes field."
    );
    assert_eq!(
        refused(good.replace("\"committedPathBytes\":[\"Y2Fm6S50eHQ=\"]", "\"committedPathBytes\":[7]")),
        "Transaction record prepared.json has a malformed committedPathBytes field."
    );
    assert_eq!(
        refused(good.replace("\"pathBytes\":\"Y2Fm6S50eHQ=\"", "\"pathBytes\":7")),
        "Transaction record prepared.json#addedPaths has a malformed pathBytes field."
    );
}

/// A record the incumbent wrote has text only, which is then the exact name,
///  including a name that holds the replacement character itself.
#[test]
fn a_record_without_byte_fields_reads_its_text_as_the_exact_name() {
    let text: String = encode_prepared(&PreparedRecord {
        shadow_path: String::from("s"),
        prepared_oid: String::from("p"),
        signed: false,
        intended_tree_oid: String::from("t"),
        committed_paths: vec!["caf\u{fffd}.txt".as_bytes().to_vec()],
        added_paths: Vec::new(),
        selected_worktree_paths: Vec::new(),
    });
    let parsed: PreparedRecord = parse_prepared(text.as_bytes()).expect("parsed");
    assert_eq!(parsed.committed_paths, vec!["caf\u{fffd}.txt".as_bytes().to_vec()]);
}

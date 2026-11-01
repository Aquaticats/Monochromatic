//! Controls for the journal parsers:
//!  every refusal the incumbent makes,
//!  with its message.

use super::*;
use crate::transaction_journal_encode::{encode_landing, encode_preparing};

/// The message of a refusal.
fn refusal(result: Result<impl std::fmt::Debug, RecoveryError>) -> String {
    match result {
        Ok(value) => panic!("accepted {value:?}"),
        Err(error) => return error.0,
    }
}

/// Replace one `"key":value` pair of an encoded record.
fn with_field(record: &str, from: &str, to: &str) -> String {
    assert!(record.contains(from), "{from} in {record}");
    return record.replacen(from, to, 1);
}

/// The head check refuses every document that is not a version-2 record of the right state.
#[test]
fn the_head_must_match() {
    for text in [
        "",
        "not json",
        "[]",
        "{\"schemaVersion\":1,\"state\":\"ref-updated\",\"landedOid\":\"x\"}",
        "{\"schemaVersion\":\"2\",\"state\":\"ref-updated\",\"landedOid\":\"x\"}",
        "{\"state\":\"ref-updated\",\"landedOid\":\"x\"}",
        "{\"schemaVersion\":2,\"state\":\"landing\",\"landedOid\":\"x\"}",
        "{\"schemaVersion\":2,\"landedOid\":\"x\"}",
    ] {
        assert_eq!(
            refusal(parse_ref_updated(text.as_bytes())),
            "Transaction record ref-updated.json is not a schema-version-2 ref-updated record.",
            "{text}"
        );
    }
    assert_eq!(
        refusal(parse_ref_updated(
            b"{\"schemaVersion\":2,\"state\":\"ref-updated\",\"landedOid\":\"\xff\"}"
        )),
        "Transaction record ref-updated.json is not a schema-version-2 ref-updated record."
    );
    assert_eq!(
        parse_ref_updated(
            b"\xef\xbb\xbf{\"schemaVersion\":2.0,\"state\":\"ref-updated\",\"landedOid\":\"x\"}"
        )
        .expect("a byte-order mark and 2.0 are accepted"),
        "x"
    );
    assert_eq!(
        refusal(parse_ref_updated(
            b"{\"schemaVersion\":2,\"state\":\"ref-updated\",\"landedOid\":7}"
        )),
        "Transaction record ref-updated.json has a malformed landedOid field."
    );
}

/// `preparing.json`:
///  unknown enumerations and malformed nested objects are refused.
#[test]
fn preparing_refusals_name_the_field() {
    let good: String =
        encode_preparing(&crate::transaction_journal_encode::tests::sample_preparing());
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "\"mode\":\"explicit-path\"",
                "\"mode\":\"paths\""
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed mode, conclusion, or refFormat field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "\"conclusion\":\"none\"",
                "\"conclusion\":\"squash\""
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed mode, conclusion, or refFormat field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "\"refFormat\":\"files\"",
                "\"refFormat\":\"packed\""
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed mode, conclusion, or refFormat field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(good.as_str(), "\"mode\":\"explicit-path\"", "\"mode\":1").as_bytes()
        )),
        "Transaction record preparing.json has a malformed mode field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "{\"kind\":\"branch\",\"ref\":\"refs/heads/main\"}",
                "{\"kind\":\"other\"}"
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed symbolicHead field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "{\"kind\":\"branch\",\"ref\":\"refs/heads/main\"}",
                "{\"kind\":\"branch\"}"
            )
            .as_bytes()
        )),
        "Transaction record preparing.json#symbolicHead has a malformed ref field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "\"symbolicHead\":{\"kind\":\"branch\",\"ref\":\"refs/heads/main\"}",
                "\"symbolicHead\":[]"
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed symbolicHead field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "{\"kind\":\"commit\",\"oid\":\"1111111111111111111111111111111111111111\"}",
                "{\"kind\":\"tree\"}"
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed base field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "{\"kind\":\"commit\",\"oid\":\"1111111111111111111111111111111111111111\"}",
                "{\"kind\":\"commit\"}"
            )
            .as_bytes()
        )),
        "Transaction record preparing.json#base has a malformed oid field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "\"selectedPathspecs\":[\"a.txt\",",
                "\"selectedPathspecs\":[1,"
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed selectedPathspecs field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                "\"selectedPathspecs\":[\"a.txt\",\"dir/b \\\"c\\\".txt\"]",
                "\"selectedPathspecs\":\"a.txt\""
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed selectedPathspecs field."
    );
    assert_eq!(
        refusal(parse_preparing(
            with_field(
                good.as_str(),
                ",\"invokedAt\":\"2026-10-06T21:34:01.123Z\"",
                ""
            )
            .as_bytes()
        )),
        "Transaction record preparing.json has a malformed invokedAt field."
    );
    // The transaction ID is read before the base, so a record wrong in both names the ID.
    let both: String = with_field(
        with_field(
            good.as_str(),
            "\"transactionId\":\"0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10\"",
            "\"transactionId\":null",
        )
        .as_str(),
        "{\"kind\":\"commit\",\"oid\":\"1111111111111111111111111111111111111111\"}",
        "{}",
    );
    assert_eq!(
        refusal(parse_preparing(both.as_bytes())),
        "Transaction record preparing.json has a malformed transactionId field."
    );
}

/// `landing-<n>.json`:
///  operations,
///  attempts and nested records are checked.
#[test]
fn landing_refusals_name_the_field() {
    let good: String = encode_landing(&LandingRecord {
        attempt: 2,
        operation: LandingOperation::Commit,
        expected_old: Base::Unborn,
        new_oid: Some(String::from("n")),
        landed_tree_oid: String::from("t"),
        pre_landing_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("2"),
        },
        post_index: FileIdentity {
            device: String::from("1"),
            inode: String::from("3"),
        },
        lock: LockIdentity {
            file: FileIdentity {
                device: String::from("1"),
                inode: String::from("4"),
            },
            fs_id: String::from("f"),
        },
        pack_name: None,
        added_paths: vec![AddedPath {
            path: b"p".to_vec(),
            git_mode: String::from("100755"),
            original_oid: String::from("o"),
            intended_oid: String::from("i"),
        }],
        selected_worktree_paths: Vec::new(),
        pre_landing_digest: Some(crate::content_digest::digest_bytes(b"pre")),
        post_digest: None,
    });
    assert_eq!(
        parse_landing(good.as_bytes()).expect("good").added_paths[0].git_mode,
        "100755"
    );
    // A digest that is present must be exactly what the writer prints.
    let digest_text: String = crate::content_digest::digest_bytes(b"pre").hash_text();
    for damaged in [
        format!("\"preLandingDigest\":{{\"length\":3,\"fnv1a64\":\"{}\"}}", digest_text.to_uppercase()),
        format!("\"preLandingDigest\":{{\"length\":-3,\"fnv1a64\":\"{digest_text}\"}}"),
        format!("\"preLandingDigest\":{{\"length\":3.5,\"fnv1a64\":\"{digest_text}\"}}"),
        String::from("\"preLandingDigest\":\"abc\""),
    ] {
        let original: String = format!("\"preLandingDigest\":{{\"length\":3,\"fnv1a64\":\"{digest_text}\"}}");
        assert_eq!(
            refusal(parse_landing(with_field(good.as_str(), original.as_str(), damaged.as_str()).as_bytes())),
            "Transaction record landing record has a malformed preLandingDigest field.",
            "{damaged}"
        );
    }
    // A digest without its hash names the nested field.
    let original: String = format!("\"preLandingDigest\":{{\"length\":3,\"fnv1a64\":\"{digest_text}\"}}");
    assert_eq!(
        refusal(parse_landing(
            with_field(good.as_str(), original.as_str(), "\"preLandingDigest\":{\"length\":3}").as_bytes()
        )),
        "Transaction record landing record#preLandingDigest has a malformed fnv1a64 field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(good.as_str(), ",\"newOid\":\"n\"", "").as_bytes()
        )),
        "Transaction record landing record has a malformed operation or newOid field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(
                good.as_str(),
                "\"operation\":\"commit\"",
                "\"operation\":\"abort\""
            )
            .as_bytes()
        )),
        "Transaction record landing record has a malformed operation or newOid field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(good.as_str(), "\"newOid\":\"n\"", "\"newOid\":5").as_bytes()
        )),
        "Transaction record landing record has a malformed newOid field."
    );
    for attempt in ["0", "-1", "1.5", "\"1\"", "9007199254740992"] {
        assert_eq!(
            refusal(parse_landing(
                with_field(
                    good.as_str(),
                    "\"attempt\":2",
                    format!("\"attempt\":{attempt}").as_str()
                )
                .as_bytes()
            )),
            "Transaction record landing record has a malformed attempt field.",
            "{attempt}"
        );
    }
    assert_eq!(
        parse_landing(with_field(good.as_str(), "\"attempt\":2", "\"attempt\":1e0").as_bytes())
            .expect("1e0 is the integer 1")
            .attempt,
        1
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(
                good.as_str(),
                "\"gitMode\":\"100755\"",
                "\"gitMode\":\"120000\""
            )
            .as_bytes()
        )),
        "Transaction record landing record has a malformed addedPaths field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(good.as_str(), "\"originalOid\":\"o\",", "").as_bytes()
        )),
        "Transaction record landing record#addedPaths has a malformed originalOid field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(good.as_str(), "\"addedPaths\":[{\"path\":\"p\",\"gitMode\":\"100755\",\"originalOid\":\"o\",\"intendedOid\":\"i\"}]", "\"addedPaths\":[7]").as_bytes()
        )),
        "Transaction record landing record has a malformed addedPaths field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(
                good.as_str(),
                "\"selectedWorktreePaths\":[]",
                "\"selectedWorktreePaths\":{}"
            )
            .as_bytes()
        )),
        "Transaction record landing record has a malformed selectedWorktreePaths field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(good.as_str(), "\"fsId\":\"f\"", "\"fsId\":false").as_bytes()
        )),
        "Transaction record landing record#lock has a malformed fsId field."
    );
    assert_eq!(
        refusal(parse_landing(
            with_field(
                good.as_str(),
                "\"postIndex\":{\"device\":\"1\",\"inode\":\"3\"}",
                "\"postIndex\":null"
            )
            .as_bytes()
        )),
        "Transaction record landing record has a malformed postIndex field."
    );
}

/// `prepared.json` and `index-lock-<n>.json` refusals.
#[test]
fn prepared_and_lock_refusals_name_the_field() {
    let prepared: &str = "{\"schemaVersion\":2,\"state\":\"prepared\",\"shadowPath\":\"s\",\"preparedOid\":\"p\",\
         \"signed\":\"yes\",\"intendedTreeOid\":\"t\",\"committedPaths\":[],\"addedPaths\":[],\"selectedWorktreePaths\":[]}";
    assert_eq!(
        refusal(parse_prepared(prepared.as_bytes())),
        "Transaction record prepared.json has a malformed signed field."
    );
    assert!(
        !parse_prepared(prepared.replace("\"yes\"", "false").as_bytes())
            .expect("valid")
            .signed
    );
    let lock: &str = "{\"schemaVersion\":2,\"state\":\"index-locked\",\"attempt\":1,\"lock\":{\"device\":\"1\",\"inode\":2,\"fsId\":\"f\"}}";
    assert_eq!(
        refusal(parse_index_lock(lock.as_bytes())),
        "Transaction record index-lock record#lock has a malformed inode field."
    );
    assert_eq!(
        refusal(parse_index_lock(
            b"{\"schemaVersion\":2,\"state\":\"landing\"}"
        )),
        "Transaction record index-lock record is not a schema-version-2 index-locked record."
    );
}

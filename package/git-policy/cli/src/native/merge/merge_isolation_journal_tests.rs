//! Controls for the isolated merge's intent record.

use super::*;
use crate::test_support::{fixture, remove};

/// The intent every control uses.
fn intent(namespace: Option<&str>, target: Option<&str>) -> MergeIntent {
    return MergeIntent {
        owner: TransactionOwner {
            transaction_id: String::from("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10"),
            owner_pid: 4242,
            owner_identity: String::from("linux:7"),
            created_at: String::from("2026-10-29T00:00:00.000Z"),
            owner_pid_namespace: namespace.map(String::from),
        },
        repository_root: String::from("/work/tree \"quoted\""),
        target_ref: target.map(String::from),
        old_oid: String::from("a".repeat(40)),
        new_oid: String::from("b".repeat(40)),
    };
}

/// A record reads back exactly, with and without the optional fields.
#[test]
fn an_intent_round_trips() {
    for (namespace, target) in [
        (None, Some("refs/heads/main")),
        (Some("pid:[4026531836]"), Some("refs/heads/feature/x")),
        (None, None),
    ] {
        let original: MergeIntent = intent(namespace, target);
        let text: String = encode_merge_intent(&original);
        assert!(text.ends_with("}\n"));
        assert_eq!(parse_merge_intent(text.as_bytes()), Some(original));
    }
}

/// A damaged record is not an intent.
#[test]
fn a_malformed_intent_is_refused() {
    let good: String = encode_merge_intent(&intent(None, Some("refs/heads/main")));
    for broken in [
        String::new(),
        String::from("[]"),
        String::from("not json"),
        good.replace("\"schemaVersion\":1", "\"schemaVersion\":2"),
        good.replace("merge-intent", "landing"),
        good.replace("\"ownerPid\":4242", "\"ownerPid\":0"),
        good.replace("\"ownerIdentity\":\"linux:7\"", "\"ownerIdentity\":\"\""),
        good.replace(&"a".repeat(40), ""),
        good.replace("\"newOid\"", "\"other\""),
        good.replace("\"repositoryRoot\":\"/work/tree \\\"quoted\\\"\"", "\"repositoryRoot\":\"\""),
    ] {
        assert_eq!(parse_merge_intent(broken.as_bytes()), None, "{broken}");
    }
    assert_eq!(parse_merge_intent(&[0xff, 0xfe]), None);
}

/// A record is written once, listed, and removed; a second write of the same merge is refused.
#[test]
fn a_record_is_written_listed_and_removed() {
    let root: PathBuf = fixture("merge-intent-files");
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(git_dir.as_path()).expect("git dir");
    assert_eq!(list_merge_intents(git_dir.as_path()).expect("absent directory"), Vec::<PathBuf>::new());
    let first: MergeIntent = intent(None, Some("refs/heads/main"));
    let path: PathBuf = write_merge_intent(git_dir.as_path(), &first).expect("first write");
    assert_eq!(path, intent_path(git_dir.as_path(), first.owner.transaction_id.as_str()));
    assert_eq!(
        parse_merge_intent(std::fs::read(path.as_path()).expect("read").as_slice()),
        Some(first.clone())
    );
    assert!(write_merge_intent(git_dir.as_path(), &first).is_err(), "records are never rewritten");
    let mut second: MergeIntent = intent(None, None);
    second.owner.transaction_id = String::from("11111111-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let other: PathBuf = write_merge_intent(git_dir.as_path(), &second).expect("second write");
    std::fs::write(intent_directory(git_dir.as_path()).join("notes.txt"), b"not a record").expect("stray file");
    assert_eq!(list_merge_intents(git_dir.as_path()).expect("listing"), vec![path.clone(), other.clone()]);
    remove_merge_intent(path.as_path()).expect("remove");
    remove_merge_intent(path.as_path()).expect("removing twice is not an error");
    assert_eq!(list_merge_intents(git_dir.as_path()).expect("listing"), vec![other]);
    remove(&root);
}

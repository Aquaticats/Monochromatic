//! Controls for the check of an index snapshot against the digest its landing record holds.

use super::*;
use crate::content_digest::digest_bytes;
use crate::test_support::{fixture, remove};
use crate::transaction_journal::{Base, FileIdentity};
use crate::transaction_journal_encode::tests::{sample_lock, sample_preparing};

/// A commit landing record that holds no digests.
fn landing(operation: LandingOperation) -> LandingRecord {
    return LandingRecord {
        attempt: 1,
        operation,
        expected_old: Base::Unborn,
        new_oid: if operation == LandingOperation::Commit {
            Some(String::from("6666666666666666666666666666666666666666"))
        } else {
            None
        },
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
        pre_landing_digest: None,
        post_digest: None,
    };
}

/// A snapshot that differs in length, in bytes of the same length, or by being empty is refused with the original
///  file name, the landing record and the transaction directory named; an identical one, or one with no recorded
///  digest, is accepted.
#[test]
fn a_snapshot_that_differs_from_its_digest_is_refused_by_name() {
    let root: std::path::PathBuf = fixture("snapshot-digest");
    let written: &[u8] = b"DIRC post-index bytes that were written";
    let record: crate::content_digest::ContentDigest = digest_bytes(written);
    let preparing: PreparingRecord = sample_preparing();
    let landed: LandingRecord = landing(LandingOperation::Commit);
    // The pinned link recovery reads is named `<snapshot>.recovery`.
    let pinned: std::path::PathBuf = root.join("post-1.index.recovery");
    let check = |role: SnapshotRole, recorded: Option<crate::content_digest::ContentDigest>| -> Result<(), RecoveryError> {
        return verify_snapshot(root.as_path(), &preparing, &landed, pinned.as_path(), role, recorded);
    };
    std::fs::write(&pinned, written).expect("snapshot");
    assert_eq!(check(SnapshotRole::Post, Some(record)), Ok(()));
    assert_eq!(check(SnapshotRole::Post, None), Ok(()));
    let damages: [(&str, Vec<u8>); 3] = [
        ("emptied", Vec::new()),
        ("halved", written[..written.len() / 2].to_vec()),
        ("zero-filled to the same length", vec![0; written.len()]),
    ];
    for (name, bytes) in damages {
        std::fs::write(&pinned, &bytes).expect("damaged snapshot");
        assert_eq!(check(SnapshotRole::Post, None), Ok(()), "{name}: no digest, no check");
        let message: String = check(SnapshotRole::Post, Some(record)).expect_err(name).0;
        assert!(
            message.starts_with(&format!(
                "The post-index snapshot {} does not match the length and hash that landing-1.json recorded for it",
                root.join("post-1.index").display()
            )),
            "{name}: {message}"
        );
        assert!(!message.contains(".recovery"), "{name}: {message}");
        assert!(message.contains("recovery did not install it as the real index"), "{name}: {message}");
        assert!(message.contains(&format!("Recovery retained the transaction at {}", root.display())), "{name}: {message}");
        assert!(message.contains(&format!("recorded {} bytes with hash {}", record.length, record.hash_text())), "{name}: {message}");
        assert!(
            message.contains(&format!("Commit {} is already on {}", "6666666666666666666666666666666666666666", preparing.target_ref)),
            "{name}: {message}"
        );
        let other: String = check(SnapshotRole::PreLanding, Some(record)).expect_err(name).0;
        assert!(other.starts_with("The pre-landing index snapshot "), "{name}: {other}");
        assert!(other.contains("cannot prove from it that the real index is unchanged"), "{name}: {other}");
    }
    remove(root.as_path());
}

/// A normalization says its branch was not moved, and a snapshot that cannot be read names the file.
#[test]
fn a_normalization_names_the_branch_it_left_alone_and_a_missing_snapshot_names_the_file() {
    let root: std::path::PathBuf = fixture("snapshot-normalization");
    let preparing: PreparingRecord = sample_preparing();
    let landed: LandingRecord = landing(LandingOperation::NormalizeOnly);
    let pinned: std::path::PathBuf = root.join("post-1.index.recovery");
    std::fs::write(&pinned, b"changed").expect("snapshot");
    let message: String = verify_snapshot(
        root.as_path(),
        &preparing,
        &landed,
        pinned.as_path(),
        SnapshotRole::Post,
        Some(digest_bytes(b"written")),
    )
    .expect_err("changed")
    .0;
    assert!(message.contains(&format!("{} was not moved by this normalization", preparing.target_ref)), "{message}");
    assert!(!message.contains("is already on"), "{message}");
    std::fs::remove_file(&pinned).expect("remove");
    let unreadable: String = verify_snapshot(
        root.as_path(),
        &preparing,
        &landed,
        pinned.as_path(),
        SnapshotRole::Post,
        Some(digest_bytes(b"written")),
    )
    .expect_err("missing")
    .0;
    assert!(unreadable.starts_with(&format!("reading the index snapshot {} failed", pinned.display())), "{unreadable}");
    remove(root.as_path());
}

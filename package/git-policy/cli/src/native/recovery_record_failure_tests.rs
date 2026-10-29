//! Controls for the message of a journal record that cannot be read.

use super::*;
use crate::recovery_landing::tests::{ID, Scene};
use crate::recovery_landing::recover_dead_transaction;
use crate::transaction_journal::LandingOperation;
use crate::transaction_journal_encode::encode_ref_updated;

/// The parser's own failure, whose sentence the message keeps as its detail.
fn parser_failure(name: &str) -> RecoveryError {
    return RecoveryError(format!("Transaction record {name} is not a schema-version-2 {name} record."));
}

/// Every kind of record: the message names the file, the transaction and the directory, keeps the parser's sentence,
///  says what to remove or restore and what the wrapper then does, and never mentions fetching history.
#[test]
fn every_kind_names_the_file_the_transaction_and_a_way_out() {
    let directory: &str = "/repo/.git/cli-git-transactions/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";
    let cases: [(RecordKind, &str, &str, &[&str]); 4] = [
        (
            RecordKind::Preparing,
            "preparing.json",
            "preparing.json is the first record of a transaction",
            &["remove", "discards the transaction", "git reset", "index.lock"],
        ),
        (
            RecordKind::Landing,
            "landing-2.json",
            "A landing record names the post-index",
            &["restore it at", "recovers the transaction", "git reset"],
        ),
        (
            RecordKind::IndexLock,
            "index-lock-1.json",
            "This record names the index.lock file the transaction created",
            &["Remove", "handles the transaction from its other records", "remove it yourself"],
        ),
        (
            RecordKind::RefUpdated,
            "ref-updated.json",
            "This record is written after the branch moved",
            &["Remove", "looks in the branch's reflog", "finishes the transaction"],
        ),
    ];
    for (kind, file, opening, steps) in cases {
        let path: std::path::PathBuf = Path::new(directory).join(file);
        let message: String = name_record_failure(parser_failure(file), path.as_path(), kind).0;
        assert!(
            message.starts_with(
                "Cannot recover transaction 0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10, so every guarded command stops with exit status 2"
            ),
            "{message}"
        );
        assert!(message.contains(&format!("{} cannot be read (Transaction record {file} is not a schema-version-2", path.display())), "{message}");
        assert!(message.contains(&format!("Recovery retained the transaction at {directory}.")), "{message}");
        assert!(message.contains(opening), "{opening}: {message}");
        for step in steps {
            assert!(message.contains(step), "{step}: {message}");
        }
        assert!(!message.contains("fetch"), "{message}");
        assert!(!message.contains("..") && !message.contains(" ."), "{message}");
    }
}

/// A scene whose transaction reached the `ref-updated` phase, with one record replaced by `bytes`;
///  recovery fails with a message naming that record's path.
fn damaged(name: &str, file: &str, bytes: &[u8]) -> (Scene, String) {
    let scene: Scene = Scene::new(name, false);
    scene.capture();
    scene.land(LandingOperation::Commit, Vec::new());
    scene.move_branch();
    std::fs::write(scene.directory.join("ref-updated.json"), encode_ref_updated(scene.prepared.as_str())).expect("marker");
    std::fs::write(scene.directory.join(file), bytes).expect("damaged record");
    let message: String = recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, i64::from(std::process::id()))
        .expect_err("a damaged record fails closed")
        .0;
    return (scene, message);
}

/// Recovery wraps the failure of each of the four records it reads, naming that record's path.
#[test]
fn recovery_names_the_path_of_each_damaged_record() {
    for (file, kind_phrase) in [
        ("preparing.json", "preparing.json is the first record of a transaction"),
        ("landing-1.json", "A landing record names the post-index"),
        ("ref-updated.json", "This record is written after the branch moved"),
    ] {
        for bytes in [&b""[..], &b"{\"schemaVersion\":2"[..]] {
            let (scene, message): (Scene, String) = damaged("record-failure-recovery", file, bytes);
            assert!(message.contains(&scene.directory.join(file).display().to_string()), "{file}: {message}");
            assert!(message.contains(kind_phrase), "{file}: {message}");
            assert!(scene.directory.join(file).exists(), "the damaged record is left for the person");
            scene.finish();
        }
    }
    // An index-lock record is read when the landing record is absent.
    let scene: Scene = Scene::new("record-failure-index-lock", false);
    scene.capture();
    scene.lock_index();
    std::fs::write(scene.directory.join("index-lock-1.json"), b"").expect("damaged record");
    let message: String = recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, i64::from(std::process::id()))
        .expect_err("a damaged record fails closed")
        .0;
    assert!(message.contains(&scene.directory.join("index-lock-1.json").display().to_string()), "{message}");
    assert!(message.contains("This record names the index.lock file the transaction created"), "{message}");
    scene.finish();
}

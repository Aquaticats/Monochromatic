//! Install-log checks: exact line encoding, reopening, unfinished trailing appends and corruption.

use super::*;
use crate::private_storage::write_private_file;
use crate::test_support::{fixture, remove};

/// One created-entry record.
fn created(relative_path: &str, device: u64, inode: u64) -> InstalledPath {
    return InstalledPath {
        device,
        inode,
        relative_path: String::from(relative_path),
        selected: true,
    };
}

/// Intents and creations are appended as one JSON line each, in the incumbent's spelling, and read back on reopen.
#[test]
fn appended_records_are_read_back_on_reopen() {
    let root: PathBuf = fixture("install-log-round-trip");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-x");
    std::fs::create_dir(stage.as_path()).expect("stage");
    {
        let mut opened: OpenedLog = open_install_log(stage.as_path()).expect("open");
        assert!(opened.recorded.intended.is_empty());
        assert!(opened.recorded.created.is_empty());
        opened
            .log
            .append_intents(&[String::from("a"), String::from("b")])
            .expect("intents");
        opened
            .log
            .append_creations(&[created("a", 1, 2)])
            .expect("creations");
        opened
            .log
            .append_intents(&[])
            .expect("empty intents write nothing");
    }
    let text: String = std::fs::read_to_string(stage.join(INSTALL_LOG_NAME)).expect("log");
    assert_eq!(
        text,
        "{\"intended\":[\"a\",\"b\"]}\n{\"created\":[{\"device\":\"1\",\"inode\":\"2\",\"relativePath\":\"a\",\"selected\":true}]}\n"
    );
    let reopened: OpenedLog = open_install_log(stage.as_path()).expect("reopen");
    assert_eq!(
        reopened.recorded.intended,
        vec![String::from("a"), String::from("b")]
    );
    assert_eq!(reopened.recorded.created, vec![created("a", 1, 2)]);
    remove(root.as_path());
}

/// The log's file is private, and a new log's directory entry is synced.
#[cfg(unix)]
#[test]
fn log_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("install-log-private");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir(stage.as_path()).expect("stage");
    let _opened: OpenedLog = open_install_log(stage.as_path()).expect("open");
    assert_eq!(
        std::fs::metadata(stage.join(INSTALL_LOG_NAME))
            .expect("log")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    remove(root.as_path());
}

/// An unfinished trailing append is ignored on reopen and cut from the file before new records follow it.
#[test]
fn unfinished_trailing_append_is_truncated() {
    let root: PathBuf = fixture("install-log-trailing");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir(stage.as_path()).expect("stage");
    let complete: &str = "{\"intended\":[\"kept\"]}\n";
    write_private_file(
        stage.join(INSTALL_LOG_NAME).as_path(),
        format!("{complete}{{\"intended\":[\"half").as_bytes(),
    )
    .expect("log");
    let mut opened: OpenedLog = open_install_log(stage.as_path()).expect("reopen");
    assert_eq!(opened.recorded.intended, vec![String::from("kept")]);
    opened
        .log
        .append_intents(&[String::from("next")])
        .expect("append");
    drop(opened);
    assert_eq!(
        std::fs::read_to_string(stage.join(INSTALL_LOG_NAME)).expect("log"),
        format!("{complete}{{\"intended\":[\"next\"]}}\n")
    );
    remove(root.as_path());
}

/// A complete line that is not a record fails closed, naming the log, and the log is kept.
#[test]
fn corrupt_line_fails_closed() {
    let root: PathBuf = fixture("install-log-corrupt");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir(stage.as_path()).expect("stage");
    let path: PathBuf = stage.join(INSTALL_LOG_NAME);
    write_private_file(path.as_path(), b"{\"other\":1}\n").expect("log");
    let failure: WorktreeCopyError = open_install_log(stage.as_path()).expect_err("corrupt");
    assert_eq!(
        failure.0,
        format!(
            "cli-git: worktree-copy install log is corrupt: {}.",
            quote(path.to_string_lossy().as_ref())
        )
    );
    assert!(path.exists());
    remove(root.as_path());
}

/// A recorded intent with an unsafe path is refused as the incumbent refuses it.
#[test]
fn unsafe_intent_is_refused() {
    let root: PathBuf = fixture("install-log-unsafe");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir(stage.as_path()).expect("stage");
    write_private_file(
        stage.join(INSTALL_LOG_NAME).as_path(),
        b"{\"intended\":[\"../up\"]}
",
    )
    .expect("log");
    let failure: WorktreeCopyError = open_install_log(stage.as_path()).expect_err("unsafe");
    assert_eq!(
        failure.0,
        "cli-git: Git returned unsafe ignored path \"../up\"."
    );
    remove(root.as_path());
}

//! Controls for the nonce reflog search,
//!  parsed and against real reflogs.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// Only subjects that start with the prefix count,
///  each commit once,
///  in order.
#[test]
fn nonce_lines_parse_strictly() {
    let output: &str = "aaa\0commit (cli-git n): one\n\
                        bbb\0reset: moving to commit (cli-git n): x\n\
                        aaa\0commit (cli-git n): again\n\
                        ccc\0commit (cli-git n): two\n\n";
    assert_eq!(
        nonce_oids(output, "commit (cli-git n):").expect("parsed"),
        vec![String::from("aaa"), String::from("ccc")]
    );
    assert_eq!(nonce_oids("", "p").expect("empty"), Vec::<String>::new());
    assert_eq!(
        nonce_oids("no separator\n", "p").expect_err("malformed").0,
        "Git returned malformed transaction reflog output."
    );
}

/// The search finds the nonce entry at any depth and fails closed on an unreadable reflog.
#[test]
fn nonce_entries_are_found_at_any_depth() {
    let root: PathBuf = fixture("recovery-reflog");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(
        repo.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=second"],
    );
    let second: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(
        repo.as_path(),
        &[
            "update-ref",
            "-m",
            "commit (cli-git 0b6c): landed",
            "refs/heads/main",
            first.as_str(),
            second.as_str(),
        ],
    );
    for message in ["later one", "later two"] {
        git(
            repo.as_path(),
            &["commit", "--quiet", "--allow-empty", "--message", message],
        );
    }
    let context: crate::transaction_git::GitContext = git_context();
    assert_eq!(
        list_nonce_reflog_oids(
            &context,
            repo.as_path(),
            "refs/heads/main",
            "commit (cli-git 0b6c):"
        )
        .expect("search"),
        vec![first.clone()]
    );
    assert_eq!(
        list_nonce_reflog_oids(
            &context,
            repo.as_path(),
            "refs/heads/main",
            "commit (cli-git ffff):"
        )
        .expect("search"),
        Vec::<String>::new()
    );
    let failure: RecoveryError = list_nonce_reflog_oids(
        &context,
        repo.as_path(),
        "refs/heads/none",
        "commit (cli-git 0b6c):",
    )
    .expect_err("unknown ref");
    assert!(
        failure
            .0
            .starts_with("refs/heads/none reflog is unreadable: "),
        "{}",
        failure.0
    );
    let unstartable: crate::transaction_git::GitContext = crate::transaction_git::GitContext {
        real_git: root.join("no-git"),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    assert!(
        list_nonce_reflog_oids(&unstartable, repo.as_path(), "refs/heads/main", "x")
            .expect_err("no Git")
            .0
            .starts_with("refs/heads/main reflog is unreadable: ")
    );
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let message: &std::ffi::OsStr = std::ffi::OsStr::from_bytes(b"commit (cli-git 0b6c): \xff");
        git(
            repo.as_path(),
            &[
                std::ffi::OsStr::new("update-ref"),
                std::ffi::OsStr::new("-m"),
                message,
                std::ffi::OsStr::new("refs/heads/main"),
                std::ffi::OsStr::new(second.as_str()),
            ],
        );
        assert_eq!(
            list_nonce_reflog_oids(
                &context,
                repo.as_path(),
                "refs/heads/main",
                "commit (cli-git 0b6c):"
            )
            .expect_err("not UTF-8")
            .0,
            "refs/heads/main reflog output is not UTF-8."
        );
    }
    remove(root.as_path());
}

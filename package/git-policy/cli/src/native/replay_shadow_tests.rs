//! Controls for the shadow runner:
//!  the shadow selection, standard input, a private index, and failures.

use super::*;
use crate::test_support::{fixture, git_context, git_text, remove, repository};

/// The runner selects the shadow store, passes input, and returns the command's output.
#[test]
fn the_runner_selects_the_shadow_and_passes_input() {
    let root: PathBuf = fixture("replay-shadow-runner");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context = git_context();
    let shadow: PathBuf = repo.join(".git");
    let head: String = String::from_utf8_lossy(
        &run_shadow_checked(&context, shadow.as_path(), repo.as_path(), &["rev-parse", "HEAD"], None, None, &[])
            .expect("head")
            .stdout,
    )
    .trim()
    .to_string();
    assert_eq!(head, git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim());
    let blob: String = String::from_utf8_lossy(
        &run_shadow_checked(
            &context,
            shadow.as_path(),
            repo.as_path(),
            &["hash-object", "--stdin"],
            Some(b"hello\n"),
            None,
            &[],
        )
        .expect("hash")
        .stdout,
    )
    .trim()
    .to_string();
    assert_eq!(blob, "ce013625030ba8dba906f756967f9e9ca394464a");
    remove(root.as_path());
}

/// A failing shadow command is an error that names the command.
#[test]
fn a_failing_shadow_command_is_an_error() {
    let root: PathBuf = fixture("replay-shadow-failure");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context = git_context();
    let error = run_shadow_checked(&context, repo.join(".git").as_path(), repo.as_path(), &["rev-parse", "nope^{commit}"], None, None, &[])
        .expect_err("unknown revision");
    assert!(error.0.contains("rev-parse"), "{}", error.0);
    remove(root.as_path());
}

/// Each replay names its private index by number.
#[test]
fn replay_indexes_are_named_by_replay_number() {
    assert_eq!(replay_base_index(Path::new("/tx"), 2), PathBuf::from("/tx/replay-base-2.index"));
}

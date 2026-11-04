//! Controls for the preparation-time conclusion copy.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// A bare shadow repository in the fixture, with the given ref backend.
fn bare_shadow(root: &Path, name: &str, ref_format: &str) -> PathBuf {
    let shadow: PathBuf = root.join(name);
    let format_argument: String = format!("--ref-format={ref_format}");
    git(
        root,
        &["init", "--bare", "--quiet", "--template=", format_argument.as_str(), name],
    );
    return shadow;
}

/// Present plain entries are copied into the shadow and the comparison copies,
///  and absent ones are skipped.
#[test]
fn present_entries_are_copied_and_absent_ones_skipped() {
    let root: PathBuf = fixture("conclusion-copy-files");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::write(git_dir.join("SQUASH_MSG"), "squash\n").expect("squash");
    let shadow: PathBuf = bare_shadow(root.as_path(), "shadow", "files");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    copy_conclusion_state(
        &git_context(),
        repo.as_path(),
        git_dir.as_path(),
        shadow.as_path(),
        transaction.as_path(),
        RefFormat::Files,
    )
    .expect("copy");
    assert_eq!(std::fs::read_to_string(shadow.join("SQUASH_MSG")).expect("shadow"), "squash\n");
    assert_eq!(
        std::fs::read_to_string(transaction.join(CONCLUSION_COPY_DIRECTORY).join("SQUASH_MSG")).expect("copy"),
        "squash\n"
    );
    assert!(std::fs::symlink_metadata(shadow.join("MERGE_HEAD")).is_err());
    assert_eq!(
        std::fs::read_to_string(transaction.join(CONCLUSION_COPY_DIRECTORY).join(STORE_RECORD_FILENAME))
            .expect("record"),
        "{}\n"
    );
    remove(root.as_path());
}

/// A sequencer directory is copied recursively into both places.
#[test]
fn sequencer_is_copied_recursively() {
    let root: PathBuf = fixture("conclusion-copy-sequencer");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::create_dir_all(git_dir.join("sequencer").join("nested")).expect("sequencer");
    std::fs::write(git_dir.join("sequencer").join("todo"), "pick abc\n").expect("todo");
    std::fs::write(git_dir.join("sequencer").join("nested").join("head"), "x\n").expect("nested");
    let shadow: PathBuf = bare_shadow(root.as_path(), "shadow", "files");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    copy_conclusion_state(
        &git_context(),
        repo.as_path(),
        git_dir.as_path(),
        shadow.as_path(),
        transaction.as_path(),
        RefFormat::Files,
    )
    .expect("copy");
    assert_eq!(
        std::fs::read_to_string(shadow.join("sequencer").join("nested").join("head")).expect("shadow nested"),
        "x\n"
    );
    assert_eq!(
        std::fs::read_to_string(transaction.join(CONCLUSION_COPY_DIRECTORY).join("sequencer").join("todo"))
            .expect("copy todo"),
        "pick abc\n"
    );
    remove(root.as_path());
}

/// A reftable store-held pseudoref is copied through the ref store and recorded by value.
#[test]
fn reftable_store_held_pseudoref_is_copied_through_the_ref_store() {
    let root: PathBuf = fixture("conclusion-copy-reftable");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-ref", "AUTO_MERGE", head.as_str()]);
    let shadow: PathBuf = bare_shadow(root.as_path(), "shadow", "reftable");
    std::fs::write(shadow.join("objects").join("info").join("alternates"), format!("{}\n", repo.join(".git").join("objects").display())).expect("alternates");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    copy_conclusion_state(
        &git_context(),
        repo.as_path(),
        repo.join(".git").as_path(),
        shadow.as_path(),
        transaction.as_path(),
        RefFormat::Reftable,
    )
    .expect("copy");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "AUTO_MERGE"]), head);
    assert_eq!(
        std::fs::read_to_string(transaction.join(CONCLUSION_COPY_DIRECTORY).join(STORE_RECORD_FILENAME))
            .expect("record"),
        format!("{{\"AUTO_MERGE\":\"{head}\"}}\n")
    );
    remove(root.as_path());
}

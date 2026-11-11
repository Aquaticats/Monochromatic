//! Controls for the preparation-time conclusion copy.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// A bare shadow repository in the fixture,
/// with the given ref backend.
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
    // A files-backend repository keeps no conclusion name in its ref store, so the record names nothing and is not written.
    assert!(
        std::fs::symlink_metadata(transaction.join(CONCLUSION_COPY_DIRECTORY).join(STORE_RECORD_FILENAME)).is_err(),
        "a record that names nothing is not written"
    );
    remove(root.as_path());
}

/// What:
///  A merge conclusion's entries are copied into the shadow and for comparison without a flush of the disk.
/// Why:
///   The shadow's copy is read by this process's own `git commit` immediately after it is written,
///  and the comparison copy by the landing that this same command asks for;
///  the service copies both into the batch's journal before the branch moves
///  (`conclusion_state_copy.rs`),
///  so nothing recovers them from this directory.
///  A copy a power cut never gave back keeps the worktree's own entry instead of removing it:
/// no content is lost.
///  Measured on this host:
/// eight client fsyncs of these throwaway files,
///  2 to 53 s over native Git in ten rounds,
/// five of ten over native plus five seconds
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
///  "A conclusion commit's copies wait for no flush").
#[test]
fn a_merge_conclusion_copies_its_entries_without_flushing_the_disk() {
    let root: PathBuf = fixture("conclusion-copy-no-flush");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::write(git_dir.join("MERGE_HEAD"), "0d5ee4b0b6f4b8f4b0c5ee4b0b6f4b8f4b0c5ee4\n").expect("merge head");
    std::fs::write(git_dir.join("MERGE_MSG"), "Merge branch 'side'\n").expect("merge message");
    std::fs::write(git_dir.join("MERGE_MODE"), "\n").expect("merge mode");
    let shadow: PathBuf = bare_shadow(root.as_path(), "shadow", "files");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    let _earlier = crate::private_storage::take_file_syncs();
    copy_conclusion_state(
        &git_context(),
        repo.as_path(),
        git_dir.as_path(),
        shadow.as_path(),
        transaction.as_path(),
        RefFormat::Files,
    )
    .expect("copy");
    let synced: Vec<PathBuf> = crate::private_storage::take_file_syncs();
    assert!(synced.is_empty(), "nothing of a merge conclusion's copy is flushed: {synced:?}");
    assert_eq!(std::fs::read_to_string(shadow.join("MERGE_HEAD")).expect("shadow"), "0d5ee4b0b6f4b8f4b0c5ee4b0b6f4b8f4b0c5ee4\n");
    assert_eq!(
        std::fs::read_to_string(transaction.join(CONCLUSION_COPY_DIRECTORY).join("MERGE_HEAD")).expect("copy"),
        "0d5ee4b0b6f4b8f4b0c5ee4b0b6f4b8f4b0c5ee4\n"
    );
    remove(root.as_path());
}

/// What:
///  A files-backend copy of a worktree with no conclusion state names nothing in the store,
///  so no record is written and no file of the copy is synced.
/// Why:
///   The record's only job is to name the values the ref store holds.
/// This backend holds none,
/// so the bytes would say nothing,
/// and a commit that writes and flushes them waits for the disk for no answer.
#[test]
fn a_files_copy_names_nothing_in_the_store_so_it_writes_no_record_and_syncs_no_file() {
    let root: PathBuf = fixture("conclusion-copy-names-nothing");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let shadow: PathBuf = bare_shadow(root.as_path(), "shadow", "files");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    let _earlier = crate::private_storage::take_file_syncs();
    copy_conclusion_state(
        &git_context(),
        repo.as_path(),
        git_dir.as_path(),
        shadow.as_path(),
        transaction.as_path(),
        RefFormat::Files,
    )
    .expect("copy");
    let synced: Vec<PathBuf> = crate::private_storage::take_file_syncs();
    assert!(synced.is_empty(), "nothing in the copy is flushed: {synced:?}");
    assert!(
        std::fs::symlink_metadata(transaction.join(CONCLUSION_COPY_DIRECTORY).join(STORE_RECORD_FILENAME)).is_err(),
        "the record names nothing, so it is not written"
    );
    remove(root.as_path());
}

/// What:
///  A reftable copy keeps the record and its flush even when no conclusion is in progress.
/// Why:
///   This backend keeps three pseudorefs in its ref store,
/// so the record is the only place their copied values are written down;
/// losing a torn one would leave the landing cleaning up the wrong pseudoref.
#[test]
fn a_reftable_copy_keeps_the_record_and_its_flush() {
    let root: PathBuf = fixture("conclusion-copy-reftable-names-nothing");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let shadow: PathBuf = bare_shadow(root.as_path(), "shadow", "reftable");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    let _earlier = crate::private_storage::take_file_syncs();
    copy_conclusion_state(
        &git_context(),
        repo.as_path(),
        repo.join(".git").as_path(),
        shadow.as_path(),
        transaction.as_path(),
        RefFormat::Reftable,
    )
    .expect("copy");
    let record: PathBuf = transaction.join(CONCLUSION_COPY_DIRECTORY).join(STORE_RECORD_FILENAME);
    assert_eq!(std::fs::read_to_string(record.as_path()).expect("record"), "{}\n");
    let synced: Vec<PathBuf> = crate::private_storage::take_file_syncs();
    assert!(synced.contains(&record), "the reftable record is flushed: {synced:?}");
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

/// What:
///  A worktree that has nothing to conclude gets no copies directory at all,
///  so the one call that creates it is not made.
/// Why:
///  The single `mkdir` of that directory was the largest stretch of a build in the records
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
///  "What each part of a commit's private repository is for,
/// and how many commits need it"),
///  and a commit that concludes nothing has nothing the directory could hold.
///  The landing answers an absent directory as no conclusion state at all,
///  so the absence says what the empty directory said.
#[test]
fn a_worktree_with_nothing_to_conclude_gets_no_copies_directory() {
    let root: PathBuf = fixture("conclusion-copy-nothing");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
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
    let copies: PathBuf = transaction.join(CONCLUSION_COPY_DIRECTORY);
    assert!(
        std::fs::symlink_metadata(copies.as_path()).is_err(),
        "nothing to conclude makes no copies directory: {}",
        copies.display()
    );
    remove(root.as_path());
}

/// What:
///  A worktree whose only conclusion state is the sequencer's gets the copies directory,
///  and the directory a commit that does conclude gets holds its entries.
/// Why:
///  The directory follows the state,
/// and not the other way round:
///  the sequencer's files are the one conclusion entry that is a directory.
#[test]
fn only_a_conclusion_with_something_to_copy_gets_a_copies_directory() {
    let root: PathBuf = fixture("conclusion-copy-sequencer-only");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    std::fs::create_dir_all(git_dir.join("sequencer")).expect("sequencer");
    std::fs::write(git_dir.join("sequencer").join("todo"), "pick abc\n").expect("todo");
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
        std::fs::read_to_string(transaction.join(CONCLUSION_COPY_DIRECTORY).join("sequencer").join("todo"))
            .expect("copied sequencer"),
        "pick abc\n"
    );
    remove(root.as_path());
}

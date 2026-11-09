//! What:
//!  The notes of how each build of a private commit index was made:
//! their order,
//! what the timing record writes,
//! and that the word every build reports is true of the index it built.
//! Why:
//!   A record that says "kept" of a build that left every entry without stat data would send the next slow commit
//!  after the wrong cause,
//! so each way of building the index is run here and the built index is asked for its stat data.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // noteIndexBuild('first', true); expect(drainIndexBuilds()).toEqual([{ kind: 'first', statDataKept: true }]);
//! ```

use super::*;
use crate::commit_index::{IndexSelection, initialize_commit_index, initialize_retry_index};
use crate::private_index_request::CaptureIndexes;
use crate::test_support::{REAL_GIT, fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::TransactionMode;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The line `git ls-files --debug` prints for the modification time of an entry that was never refreshed.
const UNSET_MODIFICATION_TIME: &str = "mtime: 0:0";

/// The builds this process noted,
/// which a control starts with none of.
fn noted() -> Vec<IndexBuild> {
    return drain_index_builds();
}

/// Builds are kept in the order they were noted,
/// and taking them empties the list,
/// so one record describes one command.
#[test]
fn each_build_is_noted_in_the_order_built_and_taken_once() {
    assert_eq!(noted(), Vec::new(), "a control starts with no build");
    note_index_build(IndexBuildKind::First, true);
    note_index_build(IndexBuildKind::Retry, false);
    note_index_build(IndexBuildKind::Retry, true);
    let taken: Vec<IndexBuild> = noted();
    assert_eq!(
        taken,
        vec![
            IndexBuild { kind: IndexBuildKind::First, stat_data_kept: true },
            IndexBuild { kind: IndexBuildKind::Retry, stat_data_kept: false },
            IndexBuild { kind: IndexBuildKind::Retry, stat_data_kept: true },
        ]
    );
    assert_eq!(noted(), Vec::new(), "the record takes the builds of its own command");
}

/// A build states its kind as the word the record carries,
/// and an empty list is the empty array,
/// so a reader never has to tell a missing field from a commit that built no index.
#[test]
fn the_record_states_the_kind_and_an_empty_list_for_no_build() {
    assert_eq!(IndexBuildKind::First.word(), "first");
    assert_eq!(IndexBuildKind::Retry.word(), "retry");
    assert_eq!(index_builds_json(&[]), "[]");
    let builds: Vec<IndexBuild> =
        vec![IndexBuild { kind: IndexBuildKind::Retry, stat_data_kept: false }, IndexBuild { kind: IndexBuildKind::First, stat_data_kept: true }];
    assert_eq!(index_builds_json(builds.as_slice()), "[{\"kind\":\"retry\",\"statDataKept\":false},{\"kind\":\"first\",\"statDataKept\":true}]");
}

/// A repository with `a.txt` and `b.txt` committed,
/// and the private indexes beside the real index.
fn committed_repository(root: &Path) -> (PathBuf, CaptureIndexes, String) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    git(repo.as_path(), &["add", "a.txt", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("captured.index"),
        commit_index: repo.join(".git").join("commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    return (repo, indexes, base);
}

/// What `git ls-files --debug` prints for `path` in the private index.
fn debug_listing(repo: &Path, index: &Path, path: &str) -> String {
    let output = Command::new(REAL_GIT)
        .current_dir(repo)
        .env("GIT_INDEX_FILE", index)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["ls-files", "--debug", "--", path])
        .output()
        .expect("ls-files");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8(output.stdout).expect("UTF-8");
}

/// The selection an explicit-path commit of `paths` makes,
/// borrowing `paths` and `base` for as long as it lives.
fn explicit_path<'selection>(paths: &'selection [OsString], base: &'selection str) -> IndexSelection<'selection> {
    return IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: paths,
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
}

/// A build that starts from a copy of the real index is noted as keeping its stat data,
/// and the built index does hold the cached entry.
#[test]
fn a_build_from_the_real_index_is_noted_as_keeping_the_stat_data() {
    let root: PathBuf = fixture("index-builds-kept");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    let named: [OsString; 1] = [OsString::from("b.txt")];
    let selection: IndexSelection<'_> = explicit_path(&named, base.as_str());
    initialize_commit_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, &selection).expect("initialize");
    assert_eq!(noted(), vec![IndexBuild { kind: IndexBuildKind::First, stat_data_kept: true }]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(!listing.contains(UNSET_MODIFICATION_TIME), "the build kept the stat data it reported: {listing}");
    remove(root.as_path());
}

/// A build from the tree alone is noted as not keeping the stat data,
/// and the built index does hold every entry without it:
/// the word the record carries is the fact an unchanged entry shows.
#[test]
fn a_build_from_the_tree_alone_is_noted_as_not_keeping_the_stat_data() {
    let root: PathBuf = fixture("index-builds-tree-alone");
    let (repo, indexes, base) = committed_repository(root.as_path());
    // A mark the copy cannot carry makes the retry build the index from the tree alone (`private_index_seed`).
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", format!("{base}^{{tree}}").as_str()]);
    initialize_retry_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, tree.as_str()).expect("retry index");
    assert_eq!(noted(), vec![IndexBuild { kind: IndexBuildKind::Retry, stat_data_kept: false }]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(listing.contains(UNSET_MODIFICATION_TIME), "a build from the tree alone holds no stat data: {listing}");
    remove(root.as_path());
}

/// The same retry with an index it can copy is noted as keeping the stat data,
/// so the two ways of building a retry index are told apart in the record.
#[test]
fn a_retry_from_a_copy_of_the_real_index_is_noted_as_keeping_the_stat_data() {
    let root: PathBuf = fixture("index-builds-retry-kept");
    let (repo, indexes, base) = committed_repository(root.as_path());
    let tree: String = git_text(repo.as_path(), &["rev-parse", format!("{base}^{{tree}}").as_str()]);
    initialize_retry_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, tree.as_str()).expect("retry index");
    assert_eq!(noted(), vec![IndexBuild { kind: IndexBuildKind::Retry, stat_data_kept: true }]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(!listing.contains(UNSET_MODIFICATION_TIME), "a copy of the real index keeps its stat data: {listing}");
    remove(root.as_path());
}

/// A rebuild after the branch moved keeps an intent-to-add entry whose mode and object name equal the tree's own entry:
/// the reset leaves such a mark as Git's own reset does,
/// and the commit then leaves that path out of its tree
/// (measured in the gate image on 2026-11-09:
/// after `read-tree --reset` the entry still carries flags `20004000`,
/// and `write-tree` over an index a later `add` invalidated lists only the named file).
#[test]
fn a_retry_keeps_the_mark_that_equals_the_tree_for_the_scratch_commit() {
    let root: PathBuf = fixture("index-builds-intent-empty-retry");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    std::fs::write(repo.join("empty"), "").expect("empty file");
    git(repo.as_path(), &["add", "empty"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=empty"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["rm", "--cached", "--quiet", "empty"]);
    git(repo.as_path(), &["add", "--intent-to-add", "empty"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", format!("{base}^{{tree}}").as_str()]);
    initialize_retry_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, tree.as_str()).expect("retry index");
    let bytes: Vec<u8> = std::fs::read(indexes.commit_index.as_path()).expect("the rebuilt index");
    let index = crate::index_full::parse_full_index(bytes.as_slice(), 20).expect("an index this reader takes");
    let entry = index.entries.into_iter().find(|entry| { return entry.path == b"empty"; }).expect("the rebuilt index holds the tracked file");
    assert_ne!(entry.extended & crate::index_full::INTENT_TO_ADD, 0, "the reset leaves a mark equal to the tree as Git's own reset does");
    remove(root.as_path());
}

/// The selection an index commit makes:
/// the whole index,
/// nothing named.
fn whole_index(base: &str) -> IndexSelection<'_> {
    return IndexSelection {
        mode: TransactionMode::Index,
        pathspecs: &[],
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: true,
        stage_tracked_changes: false,
        base_revision: base,
    };
}

/// An index commit copies the real index into its own,
/// so it is noted as keeping the stat data the real index holds.
#[test]
fn an_index_commit_is_noted_as_keeping_the_stat_data_of_the_index_it_copied() {
    let root: PathBuf = fixture("index-builds-index-mode");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    initialize_commit_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, &whole_index(base.as_str()))
        .expect("initialize");
    assert_eq!(noted(), vec![IndexBuild { kind: IndexBuildKind::First, stat_data_kept: true }]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(!listing.contains(UNSET_MODIFICATION_TIME), "the copy holds the real index's stat data: {listing}");
    remove(root.as_path());
}

/// A repository nothing was ever staged in has no index to copy,
/// and the index commit's own index is empty:
/// the record states that no stat data was there to keep,
/// so a reader is never told a build kept what never existed.
#[test]
fn an_index_commit_in_a_repository_with_no_index_is_noted_as_keeping_none() {
    let root: PathBuf = fixture("index-builds-no-index");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("captured.index"),
        commit_index: repo.join(".git").join("commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    assert!(!repo.join(".git").join("index").exists(), "a repository nothing was staged in holds no index");
    initialize_commit_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, &whole_index("HEAD"))
        .expect("initialize");
    assert_eq!(noted(), vec![IndexBuild { kind: IndexBuildKind::First, stat_data_kept: false }]);
    remove(root.as_path());
}

/// An index whose only marks are intent-to-add entries seeds a copy too,
/// so the record says the stat data was kept,
/// and the reset drops the mark whose path the base does not hold.
#[test]
fn an_intent_to_add_entry_is_noted_as_a_build_that_kept_the_stat_data() {
    let root: PathBuf = fixture("index-builds-intent");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("added.txt"), "new\n").expect("the file the person added with intent");
    git(repo.as_path(), &["add", "--intent-to-add", "--", "added.txt"]);
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    let named: [OsString; 1] = [OsString::from("b.txt")];
    let selection: IndexSelection<'_> = explicit_path(&named, base.as_str());
    initialize_commit_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, &selection).expect("initialize");
    assert_eq!(noted(), vec![IndexBuild { kind: IndexBuildKind::First, stat_data_kept: true }]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(!listing.contains(UNSET_MODIFICATION_TIME), "the copy holds the real index's stat data: {listing}");
    assert_eq!(
        debug_listing(repo.as_path(), indexes.commit_index.as_path(), "added.txt"),
        String::new(),
        "the reset drops the mark the base does not hold"
    );
    remove(root.as_path());
}

/// A rebuild from an index that holds an intent-to-add entry is noted as keeping the stat data as well,
/// which is the build a moved branch pays several times.
#[test]
fn a_retry_from_an_index_that_holds_an_intent_to_add_entry_is_noted_as_keeping_the_stat_data() {
    let root: PathBuf = fixture("index-builds-intent-retry");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("added.txt"), "new\n").expect("the file the person added with intent");
    git(repo.as_path(), &["add", "--intent-to-add", "--", "added.txt"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", format!("{base}^{{tree}}").as_str()]);
    initialize_retry_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, tree.as_str()).expect("retry index");
    assert_eq!(noted(), vec![IndexBuild { kind: IndexBuildKind::Retry, stat_data_kept: true }]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(!listing.contains(UNSET_MODIFICATION_TIME), "the copy holds the real index's stat data: {listing}");
    assert_eq!(
        debug_listing(repo.as_path(), indexes.commit_index.as_path(), "added.txt"),
        String::new(),
        "the reset drops the mark the base does not hold"
    );
    remove(root.as_path());
}

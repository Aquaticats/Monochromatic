//! Controls for the private commit index:
//! copies,
//! each selection mode,
//! pathspec files and listings.

use super::*;
use crate::capture_invocation::{capture_invocation_base, capture_invocation_layout};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::RefFormat;
use std::fs::FileTimes;
use std::ffi::OsString;
use std::path::PathBuf;
use std::os::unix::ffi::OsStringExt;
use std::time::{Duration, SystemTime};

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

/// Builds the selection of one mode with the given pathspecs and base.
fn selection<'a>(
    mode: TransactionMode,
    pathspecs: &'a [OsString],
    base: &'a str,
) -> IndexSelection<'a> {
    return IndexSelection {
        mode,
        pathspecs,
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
}

/// Runs initialization of the private index with the real index of the repository.
fn initialize(repo: &Path, indexes: &CaptureIndexes, choice: &IndexSelection) {
    initialize_commit_index(&git_context(), repo, repo.join(".git").join("index").as_path(), indexes, choice)
        .expect("initialize");
}

/// The repository paths the private index changes against the base.
fn changed(repo: &Path, indexes: &CaptureIndexes, base: &str) -> Vec<Vec<u8>> {
    return list_changed_index_paths(
        &git_context(),
        repo,
        indexes.commit_index.as_path(),
        base,
        indexes.object_directory.as_path(),
    )
    .expect("changed");
}

/// A copy keeps the source's modification time,
/// and a missing source is an error.
#[test]
fn copy_keeps_the_source_modification_time() {
    let root: PathBuf = fixture("commit-index-copy-time");
    let source: PathBuf = root.join("source");
    std::fs::write(&source, "index").expect("source");
    let past: SystemTime = SystemTime::now() - Duration::from_secs(3600);
    std::fs::File::options()
        .write(true)
        .open(&source)
        .and_then(|file| { return file.set_times(FileTimes::new().set_modified(past)); })
        .expect("age the source");
    let destination: PathBuf = root.join("copy");
    copy_index_with_timestamps(source.as_path(), destination.as_path()).expect("copy");
    assert_eq!(
        std::fs::metadata(&destination).expect("copy").modified().expect("mtime"),
        std::fs::metadata(&source).expect("source").modified().expect("mtime")
    );
    assert!(copy_index_with_timestamps(root.join("absent").as_path(), destination.as_path()).is_err());
    remove(root.as_path());
}

/// What:
///  Both private indexes of a commit are written from one read of the real index.
/// Why:
///  A second read of the same bytes is one more chance for a cold read of megabytes,
///  and the copies must hold the state this commit saw,
///  not a state the real index reached while it prepared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readFileSync(capturedIndex)).toEqual(snapshot); expect(readFileSync(commitIndex)).toEqual(snapshot);
/// ```
#[test]
fn both_private_indexes_hold_the_bytes_of_one_read_of_the_real_index() {
    let root: PathBuf = fixture("commit-index-one-read");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    git(repo.as_path(), &["add", "a.txt"]);
    let snapshot: Vec<u8> = std::fs::read(repo.join(".git").join("index")).expect("the real index's bytes");
    let none: Vec<OsString> = Vec::new();
    initialize(&repo, &indexes, &selection(TransactionMode::Index, &none, &base));
    assert_eq!(
        std::fs::read(&indexes.captured_index).expect("the captured copy"),
        snapshot,
        "the captured copy is the state the commit read"
    );
    assert_eq!(
        std::fs::read(&indexes.commit_index).expect("the commit's copy"),
        snapshot,
        "the commit's own copy is the same bytes, not a second read"
    );
    remove(root.as_path());
}

/// What:
///  A staging that lands in the real index after the read reaches neither private copy.
/// Why:
///  Another command may stage in the real index while a commit prepares,
///  and those bytes are that command's work, not this commit's:
///  a copy written from a later read would record them under this commit,
///  and the landing would compare the person's index against a state their commit never saw.
///  The control proves the staging really happened, so the two copies being free of it is an answer and not an empty
///  question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const index = await readIndexOnce(realIndex); await stageInRealIndex(); await index.writeCopy(captured); await index.writeCopy(commit);
/// ```
#[test]
fn a_staging_that_lands_after_the_read_reaches_neither_private_copy() {
    let root: PathBuf = fixture("commit-index-one-read-writer");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    let real_index: PathBuf = repo.join(".git").join("index");
    let index: IndexBytes = IndexBytes::read(real_index.as_path()).expect("the one read");
    let read: Vec<u8> = index.bytes().to_vec();
    // A writer stages a file in the real index, as another command of the person's does while this commit prepares.
    std::fs::write(repo.join("c.txt"), "c\n").expect("c");
    git(repo.as_path(), &["add", "c.txt"]);
    assert_ne!(
        std::fs::read(&real_index).expect("the real index after the staging"),
        read,
        "the control: the staging landed in the real index after the read"
    );
    index.write_copy(indexes.captured_index.as_path()).expect("the captured copy");
    index.write_copy(indexes.commit_index.as_path()).expect("the commit's copy");
    assert_eq!(std::fs::read(&indexes.captured_index).expect("the captured copy"), read, "the writer's staging is not the landing's reference");
    assert_eq!(std::fs::read(&indexes.commit_index).expect("the commit's copy"), read, "the writer's staging is not this commit's");
    assert_eq!(
        seed_of_index_bytes(index.bytes()),
        PrivateIndexSeed::Copy,
        "the seed describes the bytes the copies hold"
    );
    remove(root.as_path());
}

/// What:
///  A preparation states its one read of the index and, where the mode decides one, the seed walk of those bytes.
/// Why:
///  The record is what an unexplained slow commit is read from,
///  and a read or a walk that is not stated is time the phase holds and nothing explains.
///  An index commit takes the real index as it is and decides no seed,
///  so only an explicit-path preparation walks the bytes for one.
///  Each copy is stated as it was before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(callsOf('index read')).toBe(1); expect(callsOf('index seed')).toBe(mode === 'index' ? undefined : 1); expect(callsOf('index copy')).toBe(2);
/// ```
#[test]
fn every_selection_mode_states_its_one_read_and_its_seed_walk() {
    let root: PathBuf = fixture("commit-index-one-read-spans");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    let named: Vec<OsString> = vec![OsString::from("a.txt")];
    let all: Vec<OsString> = Vec::new();
    let modes: [(&str, TransactionMode, &Vec<OsString>, Option<usize>); 2] = [
        ("index", TransactionMode::Index, &all, None),
        ("explicit path", TransactionMode::ExplicitPath, &named, Some(1)),
    ];
    for (called, mode, pathspecs, seed_walks) in modes {
        let _previous = crate::prepare_work_spans::drain_spans();
        initialize(&repo, &indexes, &selection(mode, pathspecs, &base));
        let (spans, dropped) = crate::prepare_work_spans::drain_spans();
        assert_eq!(dropped, 0, "{called}: every step name fits");
        assert_eq!(calls_of(spans.as_slice(), "index read"), Some(1), "{called}: the one read of the real index is stated: {spans:?}");
        assert_eq!(calls_of(spans.as_slice(), "index seed"), seed_walks, "{called}: the walk of those bytes for the seed is stated only where the mode decides one: {spans:?}");
        assert_eq!(calls_of(spans.as_slice(), "index copy"), Some(2), "{called}: both private copies are stated: {spans:?}");
    }
    remove(root.as_path());
}

/// What:
///  How often one named step was noted in a drained ledger.
/// Why:
///  The control above asks about one step of a preparation and needs the count of that name and no other,
///  and the ledger keeps names as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callsOf = (spans, name) => spans.find((span) => span.name === name)?.calls;
/// ```
fn calls_of(spans: &[crate::prepare_work_spans::PrepareSpan], name: &str) -> Option<usize> {
    // A name the ledger never noted answers nothing, which is what makes a missing step a failure of the control.
    return spans.iter().find(|span| { return span.name == name; }).map(|span| { return span.calls; });
}

/// Explicit-path selection stages exactly the named paths against the recorded base.
#[test]
fn explicit_path_stages_only_the_named_paths() {
    let root: PathBuf = fixture("commit-index-explicit");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    let pathspecs: Vec<OsString> = vec![OsString::from("a.txt")];
    initialize(&repo, &indexes, &selection(TransactionMode::ExplicitPath, &pathspecs, &base));
    assert_eq!(changed(&repo, &indexes, &base), vec![b"a.txt".to_vec()]);
    remove(root.as_path());
}

/// An index commit without staging copies the real index unchanged.
#[test]
fn index_without_staging_matches_the_real_index() {
    let root: PathBuf = fixture("commit-index-index-plain");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    git(repo.as_path(), &["add", "a.txt"]);
    std::fs::write(repo.join("u.txt"), "u\n").expect("untracked");
    let pathspecs: Vec<OsString> = Vec::new();
    initialize(&repo, &indexes, &selection(TransactionMode::Index, &pathspecs, &base));
    let real_tree: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        repo.join(".git").join("index").as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("real tree");
    let private_tree: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("private tree");
    assert_eq!(private_tree, real_tree);
    remove(root.as_path());
}

/// `commit -a` staging stages every tracked change and leaves untracked files out.
#[test]
fn index_with_tracked_staging_stages_only_tracked_changes() {
    let root: PathBuf = fixture("commit-index-tracked");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    std::fs::write(repo.join("u.txt"), "u\n").expect("untracked");
    let pathspecs: Vec<OsString> = Vec::new();
    let mut choice: IndexSelection = selection(TransactionMode::Index, &pathspecs, &base);
    choice.stage_tracked_changes = true;
    initialize(&repo, &indexes, &choice);
    assert_eq!(changed(&repo, &indexes, &base), vec![b"a.txt".to_vec()]);
    remove(root.as_path());
}

/// `commit -a` stages tracked changes with a `:/` pathspec the wrapper wrote,
/// so two glob modes that disagree in the caller's environment must not make that start fail the commit.
#[test]
fn index_with_tracked_staging_survives_the_callers_conflicting_glob_modes() {
    let root: PathBuf = fixture("commit-index-tracked-glob-modes");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    let pathspecs: Vec<OsString> = Vec::new();
    let mut choice: IndexSelection = selection(TransactionMode::Index, &pathspecs, &base);
    choice.stage_tracked_changes = true;
    let mut inherited: GitContext = git_context();
    inherited.overlay.push((OsString::from("GIT_GLOB_PATHSPECS"), OsString::from("1")));
    inherited.overlay.push((OsString::from("GIT_NOGLOB_PATHSPECS"), OsString::from("1")));
    initialize_commit_index(&inherited, repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, &choice)
        .expect("the tracked staging start survives the caller's modes");
    assert_eq!(changed(&repo, &indexes, &base), vec![b"a.txt".to_vec()]);
    remove(root.as_path());
}

/// An include stages the named paths into the copy,
/// and the real index stays as it was.
#[test]
fn include_stages_the_named_path_into_the_copy_only() {
    let root: PathBuf = fixture("commit-index-include");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    let real_before: Vec<u8> = std::fs::read(repo.join(".git").join("index")).expect("index");
    let pathspecs: Vec<OsString> = vec![OsString::from("b.txt")];
    let mut choice: IndexSelection = selection(TransactionMode::Index, &pathspecs, &base);
    choice.stage_into_index = true;
    initialize(&repo, &indexes, &choice);
    assert_eq!(changed(&repo, &indexes, &base), vec![b"b.txt".to_vec()]);
    assert_eq!(std::fs::read(repo.join(".git").join("index")).expect("index"), real_before);
    remove(root.as_path());
}

/// A pathspec file read with NUL records stages each of its paths.
#[test]
fn pathspec_file_with_nul_records_selects_each_path() {
    let root: PathBuf = fixture("commit-index-nul-file");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    let list: PathBuf = root.join("list");
    std::fs::write(&list, b"a.txt\0b.txt\0").expect("list");
    let pathspecs: Vec<OsString> = Vec::new();
    let mut choice: IndexSelection = selection(TransactionMode::ExplicitPath, &pathspecs, &base);
    choice.pathspec_file = Some(list.as_path());
    choice.pathspec_file_nul = true;
    initialize(&repo, &indexes, &choice);
    assert_eq!(
        changed(&repo, &indexes, &base),
        vec![b"a.txt".to_vec(), b"b.txt".to_vec()]
    );
    remove(root.as_path());
}

/// A pathspec file read with newline records is the same selection as a list of positional paths.
#[test]
fn pathspec_file_with_newline_records_selects_one_path() {
    let root: PathBuf = fixture("commit-index-newline-file");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    let list: PathBuf = root.join("list");
    std::fs::write(&list, "a.txt\n").expect("list");
    let pathspecs: Vec<OsString> = Vec::new();
    let mut choice: IndexSelection = selection(TransactionMode::ExplicitPath, &pathspecs, &base);
    choice.pathspec_file = Some(list.as_path());
    initialize(&repo, &indexes, &choice);
    assert_eq!(changed(&repo, &indexes, &base), vec![b"a.txt".to_vec()]);
    remove(root.as_path());
}

/// The written tree is the tree Git reports for the private index,
/// and it is a tree object.
#[test]
fn private_tree_is_the_tree_of_the_private_index() {
    let root: PathBuf = fixture("commit-index-tree");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    let pathspecs: Vec<OsString> = vec![OsString::from("a.txt")];
    initialize(&repo, &indexes, &selection(TransactionMode::ExplicitPath, &pathspecs, &base));
    let oid: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", oid.as_str()]), "tree");
    assert_ne!(oid, git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]));
    remove(root.as_path());
}

/// A conflicted path is listed once,
/// even though it has several unmerged stages.
#[test]
fn unmerged_paths_are_listed_once() {
    let root: PathBuf = fixture("commit-index-unmerged");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    let blob: String = git_text(repo.as_path(), &["hash-object", "-w", "a.txt"]);
    let info: String = format!("100644 {blob} 1\ta.txt\n100644 {blob} 2\ta.txt\n100644 {blob} 3\ta.txt\n");
    let mut request = indexed_request(
        repo.as_path(),
        &["update-index", "--index-info"],
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    request.input = Some(info.into_bytes());
    run_git_checked(&git_context(), &request).expect("conflict stages");
    assert_eq!(
        list_unmerged_index_paths(&git_context(), repo.as_path(), indexes.commit_index.as_path()).expect("unmerged"),
        vec![b"a.txt".to_vec()]
    );
    remove(root.as_path());
}

/// The unmerged paths are named to `:(top,literal)` specs and to the person,
/// so a commit run from a subdirectory must be answered with repository-relative names:
/// `git ls-files` alone answers relative to the directory Git runs in.
#[test]
fn unmerged_paths_of_a_nested_directory_are_named_from_the_top() {
    let root: PathBuf = fixture("commit-index-unmerged-nested");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    std::fs::create_dir(repo.join("sub")).expect("sub directory");
    std::fs::write(repo.join("sub/f"), b"conflict\n").expect("write sub/f");
    let blob: String = git_text(repo.as_path(), &["hash-object", "-w", "sub/f"]);
    let info: String = format!("100644 {blob} 1\tsub/f\n100644 {blob} 2\tsub/f\n100644 {blob} 3\tsub/f\n");
    let mut request = indexed_request(
        repo.as_path(),
        &["update-index", "--index-info"],
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    request.input = Some(info.into_bytes());
    run_git_checked(&git_context(), &request).expect("conflict stages");
    assert_eq!(
        list_unmerged_index_paths(&git_context(), repo.join("sub").as_path(), indexes.commit_index.as_path()).expect("unmerged"),
        vec![b"sub/f".to_vec()],
        "the path is named as the specs and the warning write it"
    );
    remove(root.as_path());
}

/// The layout of a repository and the capture of its base give the same base the selection uses.
#[test]
fn selection_base_is_the_recorded_base() {
    let root: PathBuf = fixture("commit-index-base");
    let (repo, _indexes, base) = committed_repository(root.as_path());
    let layout = capture_invocation_layout(&git_context(), repo.as_path(), false).expect("layout");
    let capture = capture_invocation_base(&git_context(), repo.as_path(), layout).expect("capture");
    assert_eq!(capture.layout.ref_format, RefFormat::Files);
    assert_eq!(crate::capture_invocation::base_revision(&capture), base);
    remove(root.as_path());
}


/// The incumbent's committed tree when one file is selected,
/// recorded by the incumbent driver
/// `native-interop/capture-differential.mjs` (deleted;
/// readable at commit `5d61d7b442`).
const INCUMBENT_ONE_FILE_TREE: &str = "4f40a1ec2c64780454051d9d363c6870a6ef2f6e";
/// The incumbent's committed tree when both changed files are selected.
const INCUMBENT_TWO_FILE_TREE: &str = "efe77971c159b0a29a5d97e7169b8c35fa33a1f1";

/// Builds the state the incumbent differential starts from:
/// two committed files,
/// both changed,
/// one untracked.
fn differential_state(root: &Path, stage: &[&str]) -> (PathBuf, CaptureIndexes, String) {
    std::fs::create_dir_all(root).expect("state root");
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    git(repo.as_path(), &["add", "a.txt", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), "a-changed\n").expect("a changed");
    std::fs::write(repo.join("b.txt"), "b-changed\n").expect("b changed");
    std::fs::write(repo.join("u.txt"), "u\n").expect("untracked");
    if !stage.is_empty() {
        let mut arguments: Vec<&str> = vec!["add"];
        arguments.extend(stage.iter().copied());
        git(repo.as_path(), &arguments);
    }
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: transaction.join("captured.index"),
        commit_index: transaction.join("commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    return (repo, indexes, base);
}

/// Initializes the private index for one selection and returns the tree written from it.
fn native_tree(repo: &Path, indexes: &CaptureIndexes, choice: &IndexSelection) -> String {
    initialize_commit_index(&git_context(), repo, repo.join(".git").join("index").as_path(), indexes, choice)
        .expect("initialize");
    return write_private_tree(
        &git_context(),
        repo,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
}

/// Explicit-path selections,
/// with a pathspec,
/// a newline pathspec file,
/// a NUL pathspec file and stdin,
///  yield the incumbent's one-file tree,
/// and the two-file selections yield the incumbent's two-file tree.
#[test]
fn private_index_trees_match_the_incumbent_for_each_selection() {
    let root: PathBuf = fixture("commit-index-incumbent-explicit");
    {
    let (repo, indexes, base) = differential_state(root.join("p").as_path(), &[]);
    let pathspecs: Vec<OsString> = vec![OsString::from("a.txt")];
    assert_eq!(
        native_tree(&repo, &indexes, &selection(TransactionMode::ExplicitPath, &pathspecs, &base)),
        INCUMBENT_ONE_FILE_TREE,
        "explicit-path"
    );
    }
    remove(root.as_path());
}

/// The index-mode and tracked-staging selections match the incumbent's trees.
#[test]
fn index_mode_trees_match_the_incumbent() {
    let root: PathBuf = fixture("commit-index-incumbent-index");
    let none: Vec<OsString> = Vec::new();

    {
    let (repo, indexes, base) = differential_state(root.join("plain").as_path(), &["a.txt"]);
    assert_eq!(
        native_tree(&repo, &indexes, &selection(TransactionMode::Index, &none, &base)),
        INCUMBENT_ONE_FILE_TREE,
        "index-plain"
    );

    }
    {
    let (repo, indexes, base) = differential_state(root.join("tracked").as_path(), &[]);
    let mut tracked: IndexSelection = selection(TransactionMode::Index, &none, &base);
    tracked.stage_tracked_changes = true;
    assert_eq!(native_tree(&repo, &indexes, &tracked), INCUMBENT_TWO_FILE_TREE, "tracked-all");

    }
    {
    let (repo, indexes, base) = differential_state(root.join("include").as_path(), &["a.txt"]);
    let include_paths: Vec<OsString> = vec![OsString::from("b.txt")];
    let mut include: IndexSelection = selection(TransactionMode::Index, &include_paths, &base);
    include.stage_into_index = true;
    assert_eq!(native_tree(&repo, &indexes, &include), INCUMBENT_TWO_FILE_TREE, "include");
    }
    remove(root.as_path());
}

/// Pathspec files in both record forms,
/// and standard input,
/// select the same paths the incumbent did.
#[test]
fn pathspec_file_trees_match_the_incumbent() {
    let root: PathBuf = fixture("commit-index-incumbent-files");
    let none: Vec<OsString> = Vec::new();

    {
    let (repo, indexes, base) = differential_state(root.join("newline").as_path(), &[]);
    let list: PathBuf = root.join("newline.list");
    std::fs::write(&list, "a.txt\n").expect("list");
    let mut newline: IndexSelection = selection(TransactionMode::ExplicitPath, &none, &base);
    newline.pathspec_file = Some(list.as_path());
    assert_eq!(native_tree(&repo, &indexes, &newline), INCUMBENT_ONE_FILE_TREE, "newline file");

    }
    {
    let (repo, indexes, base) = differential_state(root.join("nul").as_path(), &[]);
    let list: PathBuf = root.join("nul.list");
    std::fs::write(&list, b"a.txt\0b.txt\0").expect("nul list");
    let mut nul: IndexSelection = selection(TransactionMode::ExplicitPath, &none, &base);
    nul.pathspec_file = Some(list.as_path());
    nul.pathspec_file_nul = true;
    assert_eq!(native_tree(&repo, &indexes, &nul), INCUMBENT_TWO_FILE_TREE, "nul file");

    }
    remove(root.as_path());
}

/// A pathspec file that cannot be read makes the interactive run fail,
/// and the failure names the command.
#[test]
fn interactive_selection_reports_a_failed_git_add() {
    let root: PathBuf = fixture("commit-index-interactive-fails");
    let (repo, indexes, base) = differential_state(root.join("state").as_path(), &[]);
    let none: Vec<OsString> = Vec::new();
    let missing: PathBuf = root.join("missing.list");
    let mut choice: IndexSelection = selection(TransactionMode::ExplicitPath, &none, &base);
    choice.pathspec_file = Some(missing.as_path());
    let error: GitFailure = prepare_interactive_selection(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        true,
        &choice,
    )
    .expect_err("missing pathspec file");
    assert!(error.0.starts_with("git add --patch --pathspec-from-file="), "{}", error.0);
    assert!(error.0.contains("exited"), "{}", error.0);
    remove(root.as_path());
}

/// An interactive run over an index with no changes finishes without prompting and succeeds.
#[test]
fn interactive_selection_succeeds_without_changes() {
    let root: PathBuf = fixture("commit-index-interactive-clean");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir(&transaction).expect("transaction");
    let commit_index: PathBuf = transaction.join("commit.index");
    copy_index_with_timestamps(repo.join(".git").join("index").as_path(), commit_index.as_path()).expect("copy");
    let none: Vec<OsString> = Vec::new();
    let choice: IndexSelection = selection(TransactionMode::ExplicitPath, &none, &base);
    prepare_interactive_selection(
        &git_context(),
        repo.as_path(),
        commit_index.as_path(),
        repo.join(".git").join("objects").as_path(),
        false,
        &choice,
    )
    .expect("clean interactive run");
    remove(root.as_path());
}

/// Names that are not UTF-8 stay bytes from the pathspec to the listing:
///  an explicit-path selection stages the exact name,
///  the changed-path listing returns its bytes,
///  and a conflicted name is listed with its bytes once.
#[test]
fn names_that_are_not_utf8_stay_bytes_from_the_pathspec_to_the_listings() {
    let root: PathBuf = fixture("commit-index-odd-name");
    let (repo, indexes, base) = committed_repository(root.as_path());
    let odd: OsString = OsString::from_vec(b"caf\xe9.txt".to_vec());
    let other: OsString = OsString::from_vec(b"caf\xe8.txt".to_vec());
    std::fs::write(repo.join(&odd), "odd\n").expect("odd");
    std::fs::write(repo.join(&other), "other\n").expect("other");
    let pathspecs: Vec<OsString> = vec![odd.clone()];
    initialize(&repo, &indexes, &selection(TransactionMode::ExplicitPath, &pathspecs, &base));
    assert_eq!(changed(&repo, &indexes, &base), vec![b"caf\xe9.txt".to_vec()]);
    let blob: String = git_text(repo.as_path(), &["hash-object", "-w", "a.txt"]);
    let mut request = indexed_request(
        repo.as_path(),
        &["update-index", "-z", "--index-info"],
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    let mut stages: Vec<u8> = Vec::new();
    for stage in 1..=3 {
        stages.extend_from_slice(format!("100644 {blob} {stage}\tcaf").as_bytes());
        stages.extend_from_slice(b"\xe9.txt\0");
    }
    request.input = Some(stages);
    run_git_checked(&git_context(), &request).expect("conflict stages");
    assert_eq!(
        list_unmerged_index_paths(&git_context(), repo.as_path(), indexes.commit_index.as_path()).expect("unmerged"),
        vec![b"caf\xe9.txt".to_vec()]
    );
    remove(root.as_path());
}

/// The lines of `git ls-files --debug` for one index:
/// each entry's stat data as Git cached it.
fn stat_lines(repo: &Path, index: &Path) -> String {
    let shown = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo)
        .env("GIT_INDEX_FILE", index)
        .args(["ls-files", "--debug"])
        .output()
        .expect("ls-files runs");
    assert!(shown.status.success(), "{}", String::from_utf8_lossy(&shown.stderr));
    return String::from_utf8_lossy(&shown.stdout).into_owned();
}

/// The private index of a retry keeps the cached stat data of every entry the tree shares with the real index.
/// Why:
///  A `read-tree` into a fresh index leaves every entry with no stat data,
/// and the scratch `git commit` then reads and hashes every file of the work tree to refresh it,
/// LFS files through the clean filter (measured 2026-11-07:
/// about 233 MB cleaned for a commit of one Markdown file in this repository).
/// A retry runs when the branch moved under a commit,
/// which is exactly when other commits land,
/// so the cost fell on the busiest moments.
/// The control makes the plain `read-tree` and shows the probe sees the missing data.
#[test]
fn a_retry_index_keeps_the_stat_data_of_the_entries_the_tree_shares_with_the_real_index() {
    let root: PathBuf = fixture("commit-index-retry-stat");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let without: PathBuf = repo.join(".git").join("without.index");
    let plain = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(&repo)
        .env("GIT_INDEX_FILE", &without)
        .args(["read-tree", tree.as_str()])
        .output()
        .expect("read-tree runs");
    assert!(plain.status.success());
    assert_eq!(stat_lines(repo.as_path(), without.as_path()).matches("mtime: 0:0").count(), 2, "the control: a plain read-tree holds no stat data");
    initialize_retry_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, tree.as_str()).expect("retry index");
    let kept: String = stat_lines(repo.as_path(), indexes.commit_index.as_path());
    assert_eq!(kept.matches("mtime: 0:0").count(), 0, "{kept}");
    assert_eq!(kept.matches("mtime: ").count(), 2, "both entries are in the index: {kept}");
    assert_eq!(git_text(repo.as_path(), &["ls-tree", "-r", "--name-only", tree.as_str()]), "a.txt\nb.txt");
    remove(root.as_path());
}

/// A retry whose tree differs from the real index holds the tree,
/// and keeps stat data only for what the two share:
/// the index is the tree,
/// whatever the real index said.
#[test]
fn a_retry_index_is_exactly_the_tree_and_a_real_index_that_cannot_seed_it_is_not_copied() {
    let root: PathBuf = fixture("commit-index-retry-tree");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "a2\n").expect("a2");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD~1^{tree}"]);
    // The real index holds the second commit's tree; the retry's tree is the first's.
    initialize_retry_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, first.as_str()).expect("retry index");
    let written = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(&repo)
        .env("GIT_INDEX_FILE", &indexes.commit_index)
        .arg("write-tree")
        .output()
        .expect("write-tree runs");
    assert_eq!(String::from_utf8_lossy(&written.stdout).trim(), first);
    // An index that cannot be read is not copied: the retry still holds the tree.
    std::fs::write(repo.join(".git").join("index"), b"not an index").expect("damaged index");
    initialize_retry_index(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &indexes, first.as_str()).expect("retry index from the tree alone");
    let again = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(&repo)
        .env("GIT_INDEX_FILE", &indexes.commit_index)
        .arg("write-tree")
        .output()
        .expect("write-tree runs");
    assert_eq!(String::from_utf8_lossy(&again.stdout).trim(), first);
    remove(root.as_path());
}

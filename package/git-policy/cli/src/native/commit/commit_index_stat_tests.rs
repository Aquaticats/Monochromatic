//! Controls for the cached stat data of the explicit-path private index:
//!  an entry the base shares with the real index keeps its stat data,
//! so the shadow commit does not read the file again,
//!  and an entry the worktree changed since the real index recorded it does not keep a stale one.

use super::*;
use crate::capture_invocation::{InvocationCapture, capture_invocation_base, capture_invocation_layout};
use crate::commit_preparation::{PreparationError, PreparationRequest, run_native_commit_within};
use crate::shadow_create::create_shadow_repository;
use crate::test_support::{REAL_GIT, age, executable, fixture, git, git_context, git_text, remove, renew_inode, repository};
use crate::transaction_git::GitContext;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The line `git ls-files --debug` prints for the modification time of an entry that was never refreshed.
const UNSET_MODIFICATION_TIME: &str = "mtime: 0:0";

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

/// Initializes the private index for an explicit-path commit of `paths`.
fn initialize_explicit(repo: &Path, indexes: &CaptureIndexes, base: &str, paths: &[OsString]) {
    let choice: IndexSelection = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: paths,
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
    initialize_commit_index(&git_context(), repo, repo.join(".git").join("index").as_path(), indexes, &choice)
        .expect("initialize");
}

/// A path the commit does not name keeps the real index's stat data in the private index.
#[test]
fn an_unchanged_entry_keeps_its_cached_stat_data() {
    let root: PathBuf = fixture("commit-index-stat-kept");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("b.txt")]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(listing.contains("mtime:"), "the listing names the entry: {listing}");
    assert!(!listing.contains(UNSET_MODIFICATION_TIME), "a.txt must keep its stat data: {listing}");
    remove(root.as_path());
}

/// The same control run through the real index shows what "kept" looks like,
/// so the control above can fail:
///  a private index built the old way,
/// by a plain `read-tree` of the base,
/// holds no stat data at all.
#[test]
fn a_plain_read_tree_leaves_no_stat_data() {
    let root: PathBuf = fixture("commit-index-stat-positive-control");
    let (repo, indexes, base) = committed_repository(root.as_path());
    let plain: Output = Command::new(REAL_GIT)
        .current_dir(repo.as_path())
        .env("GIT_INDEX_FILE", indexes.commit_index.as_path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["read-tree", base.as_str()])
        .output()
        .expect("read-tree");
    assert!(plain.status.success());
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(listing.contains(UNSET_MODIFICATION_TIME), "{listing}");
    remove(root.as_path());
}

/// A path the worktree changed after the real index recorded it never keeps a stale stat entry as unchanged:
///  the private index still turns into the tree of the base plus the named path.
#[test]
fn a_named_path_is_staged_from_the_worktree_not_from_the_cached_entry() {
    let root: PathBuf = fixture("commit-index-stat-named");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt")]);
    let tree: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    let content: String = git_text(repo.as_path(), &["show", format!("{tree}:a.txt").as_str()]);
    assert_eq!(content, "changed");
    remove(root.as_path());
}

/// The content of `path` in the tree the private index produces.
fn private_content(repo: &Path, indexes: &CaptureIndexes, path: &str) -> String {
    let tree: String = write_private_tree(
        &git_context(),
        repo,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    return git_text(repo, &["show", format!("{tree}:{path}").as_str()]);
}

/// A repository with `a.txt`,
/// `b.txt` and `d/x.txt` committed.
fn repository_with_directory(root: &Path) -> (PathBuf, CaptureIndexes, String) {
    let (repo, indexes, _base) = committed_repository(root);
    std::fs::create_dir(repo.join("d")).expect("directory");
    std::fs::write(repo.join("d").join("x.txt"), "x\n").expect("x");
    git(repo.as_path(), &["add", "d/x.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=directory"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (repo, indexes, base);
}

/// A file marked assume-unchanged that the commit names is staged from the worktree,
/// as Git's own partial commit does;
///  without that,
/// the mark in the copied index makes `git add` skip it and the named file is silently left out.
#[test]
fn a_named_assume_unchanged_file_is_committed() {
    let root: PathBuf = fixture("commit-index-assume-named");
    let (repo, indexes, base) = committed_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed b\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt"), OsString::from("b.txt")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "changed a");
    assert_eq!(private_content(repo.as_path(), &indexes, "b.txt"), "changed b");
    remove(root.as_path());
}

/// A file marked assume-unchanged that the commit does not name stays as the base has it.
#[test]
fn an_unnamed_assume_unchanged_file_stays_as_the_base_has_it() {
    let root: PathBuf = fixture("commit-index-assume-unnamed");
    let (repo, indexes, base) = committed_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed b\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("b.txt")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "a");
    assert_eq!(private_content(repo.as_path(), &indexes, "b.txt"), "changed b");
    remove(root.as_path());
}

/// A directory named by the commit takes the marked files inside it too.
#[test]
fn a_named_directory_takes_its_assume_unchanged_files() {
    let root: PathBuf = fixture("commit-index-assume-directory");
    let (repo, indexes, base) = repository_with_directory(root.as_path());
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "d/x.txt"]);
    std::fs::write(repo.join("d").join("x.txt"), "changed x\n").expect("change x");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("d")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "d/x.txt"), "changed x");
    remove(root.as_path());
}

/// A path marked skip-worktree by hand that the commit names does not fail the commit,
/// as before the copy.
#[test]
fn a_named_skip_worktree_file_does_not_fail_the_commit() {
    let root: PathBuf = fixture("commit-index-skip-named");
    let (repo, indexes, base) = committed_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--skip-worktree", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed b\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt"), OsString::from("b.txt")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "changed a");
    assert_eq!(private_content(repo.as_path(), &indexes, "b.txt"), "changed b");
    remove(root.as_path());
}

/// A tracked empty file that the real index holds as intent-to-add stays in the tree the private index produces when the
/// commit names another path:
/// the entry's mode and object name equal the base's,
/// so the reset would leave the mark as Git's own reset does,
/// and the copy clears that path before the reset so the base's own entry comes back.
/// A tree built from a surviving mark leaves the path out once staging the named file invalidates the cached tree,
/// which native Git's own tree does
/// (`an_intent_to_add_entry_equal_to_the_base_stays_in_the_commit_where_native_git_drops_it`,
/// `binary_commit_tests.rs`).
#[test]
fn an_intent_to_add_entry_equal_to_the_base_keeps_its_path_in_the_commit() {
    let root: PathBuf = fixture("commit-index-intent-empty");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    std::fs::write(repo.join("empty"), "").expect("empty file");
    git(repo.as_path(), &["add", "empty"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=empty"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["rm", "--cached", "--quiet", "empty"]);
    git(repo.as_path(), &["add", "--intent-to-add", "empty"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt")]);
    let tree: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    assert_eq!(
        git_text(repo.as_path(), &["ls-tree", "-r", tree.as_str(), "--", "empty"]),
        "100644 blob e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\tempty",
        "the tree holds the entry the base holds, which the person never asked to remove"
    );
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "changed a");
    remove(root.as_path());
}

/// An index no mark touches seeds the private index as a plain copy,
/// so the safeguard costs nothing for it;
/// an index whose only marks are intent-to-add entries seeds it too,
/// copied as it is;
/// an assume-unchanged entry sends the build to the base,
/// and bytes this crate cannot read seed nothing.
#[test]
fn only_an_index_without_a_hidden_entry_seeds_the_private_index() {
    let root: PathBuf = fixture("commit-index-seeds");
    let (repo, _indexes, _base) = committed_repository(root.as_path());
    let real: PathBuf = repo.join(".git").join("index");
    assert_eq!(seed_of_index_bytes(&index_bytes_of(real.as_path())), PrivateIndexSeed::Copy);
    std::fs::write(repo.join("added.txt"), "new\n").expect("the file the person added with intent");
    git(repo.as_path(), &["add", "--intent-to-add", "--", "added.txt"]);
    assert_eq!(
        seed_of_index_bytes(&index_bytes_of(real.as_path())),
        PrivateIndexSeed::CopyWithIntentToAdd(vec![b"added.txt".to_vec()]),
        "an intent-to-add entry keeps the copy"
    );
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    assert_eq!(seed_of_index_bytes(&index_bytes_of(real.as_path())), PrivateIndexSeed::Base);
    assert_eq!(seed_of_index_bytes(b"not an index"), PrivateIndexSeed::Base, "bytes this crate cannot read seed nothing");
    remove(root.as_path());
}

/// What:
///  The bytes of one file,
///  read the way a control hands an index's state to the seed decision.
/// Why:
///  The controls below ask the seed decision about the bytes the real index holds,
///  so they must read that file rather than name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const indexBytes = (path: string) => readFileSync(path);
/// ```
fn index_bytes_of(index: &Path) -> Vec<u8> {
    return std::fs::read(index).expect("the index's bytes");
}

//region The stat data of an index the move to another disk left stale

/// How many files the worktree of the stale-index controls holds.
const STALE_FILES: usize = 12;

/// How long the clean filter of those files takes,
/// in milliseconds,
/// so a refresh that must read them costs a time the test can put a ceiling on.
///  The sleep is what the ceiling is measured against,
/// and a sleep does not grow when the host runs many controls at once,
/// where the step under the ceiling does:
/// in a selection of 96 controls the landing controls took 62 s against 4.6 s alone, 13.5 times,
/// and the scratch step's own cost is about 0.1 s here,
/// so a ceiling of 825 ms (eleven files at 100 ms, three quarters of the read)
/// left room for only eight times that cost and ended the step instead.
/// At 400 ms the same ceiling is 3,300 ms,
/// which leaves the step 33 times its own cost before it is ended,
/// and a commit that pays the reads inside the step still takes 4,400 ms or more.
const CLEAN_FILTER_MILLISECONDS: u64 = 400;

/// The fixed transaction ID of the shape the shadow's name takes.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// A repository whose index records stat data the worktree no longer has,
/// as the move to the 860 drive left every worktree,
/// with a shadow and a private index ready to prepare the commit of one named path.
struct Stale {
    /// Repository the commit selects.
    repo: PathBuf,
    /// Shadow repository of the commit.
    shadow: PathBuf,
    /// Invocation capture.
    capture: InvocationCapture,
    /// Private indexes and object store.
    indexes: CaptureIndexes,
    /// The commit's own scratch directory.
    scratch: PathBuf,
    /// Base commit of the invocation.
    base: String,
}

/// The ceiling of the scratch commit in these controls:
/// three quarters of what reading every stale file costs,
/// so a commit that pays the refresh in its own step is stopped and one that does not is not,
/// with the rest of the room left for a busy host (`CLEAN_FILTER_MILLISECONDS`).
///
/// The commit names one path,
/// and the preparation stages that path from the worktree before the commit runs,
/// so the files the commit would read are all but the named one.
fn stale_ceiling() -> std::time::Duration {
    let stale_files: u64 = u64::try_from(STALE_FILES - 1).expect("a file count fits");
    return std::time::Duration::from_millis(stale_files * CLEAN_FILTER_MILLISECONDS * STALE_CEILING_PART / STALE_CEILING_WHOLE);
}

/// The part of the stale read the ceiling allows,
/// and its whole.
const STALE_CEILING_PART: u64 = 3;
const STALE_CEILING_WHOLE: u64 = 4;

/// A repository of `STALE_FILES` files behind a clean filter,
/// every file with a new inode,
/// one of them changed in the worktree,
/// and a shadow ready to prepare the commit of that one.
fn stale_fixture(root: &Path) -> Stale {
    let repo: PathBuf = repository(root, "repo");
    git(repo.as_path(), &["config", "filter.slow.clean", format!("sleep 0.{CLEAN_FILTER_MILLISECONDS}; cat").as_str()]);
    git(repo.as_path(), &["config", "filter.slow.smudge", "cat"]);
    std::fs::write(repo.join(".gitattributes"), "* filter=slow\n").expect("attributes");
    for number in 1..=STALE_FILES {
        std::fs::write(repo.join(format!("f{number}.txt")), format!("file {number}\n")).expect("a file of the worktree");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    for number in 1..=STALE_FILES {
        renew_inode(repo.join(format!("f{number}.txt")).as_path());
    }
    std::fs::write(repo.join("f1.txt"), "changed\n").expect("the change the commit names");
    let context = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let transaction: PathBuf = root.join(format!("transaction-{TRANSACTION}"));
    std::fs::create_dir_all(transaction.as_path()).expect("transaction");
    let shadow: PathBuf = create_shadow_repository(&context, repo.as_path(), &capture, TRANSACTION, transaction.as_path()).expect("shadow");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: transaction.join("captured.index"),
        commit_index: transaction.join("commit.index"),
        object_directory: shadow.join("objects"),
    };
    return Stale { repo, shadow, capture, indexes, scratch: transaction, base };
}

/// Stages the named path into the private index,
/// as a commit of that path prepares it.
fn prepare_stale(stale: &Stale) {
    prepare_stale_with(&git_context(), stale);
}

/// Runs the scratch commit of the named path under the stale index's ceiling.
fn commit_stale(stale: &Stale) -> Result<i32, PreparationError> {
    let commit_args: Vec<String> = vec![String::from("--quiet"), String::from("--message=stale")];
    let request: PreparationRequest<'_> = PreparationRequest {
        capture: &stale.capture,
        shadow: stale.shadow.as_path(),
        commit_index: stale.indexes.commit_index.as_path(),
        scratch: stale.scratch.as_path(),
        global_args: &[],
        commit_args: commit_args.as_slice(),
        object_directory: None,
    };
    return run_native_commit_within(&git_context(), stale.repo.as_path(), &request, stale_ceiling());
}

/// A commit over an index whose stat data the worktree no longer matches finishes inside the ceiling of its step:
///  the refresh of those files is paid where the index is copied,
/// not by the `git commit` the ceiling ends.
#[test]
fn a_commit_over_an_index_whose_stat_data_is_stale_finishes() {
    let root: PathBuf = fixture("commit-index-stat-stale-ceiling");
    let stale: Stale = stale_fixture(root.as_path());
    prepare_stale(&stale);
    match commit_stale(&stale) {
        Ok(code) => assert_eq!(code, 0, "the scratch commit exited {code}"),
        Err(error) => panic!("the commit over a stale index did not finish: {error}"),
    }
    remove(root.as_path());
}

/// The refresh reaches the private index,
/// and never the real one:
/// the commit after this one is not made faster by this commit's preparation,
/// and the real index is written only by a landing,
/// which holds its lock anyway.
///  The named file is left out of this control:
/// a path whose bytes changed keeps the stat data the index recorded for it,
/// so the next `git add` still sees it as changed.
#[test]
fn a_preparation_keeps_the_refresh_in_the_private_index_and_leaves_the_real_one_alone() {
    let root: PathBuf = fixture("commit-index-stat-stale-private");
    let stale: Stale = stale_fixture(root.as_path());
    let real: PathBuf = stale.repo.join(".git").join("index");
    let before: Vec<u8> = std::fs::read(real.as_path()).expect("the real index");
    prepare_stale(&stale);
    assert!(
        std::fs::read(real.as_path()).expect("the real index") == before,
        "a preparation must not write the real index, and the bytes of {} changed",
        real.display()
    );
    let inode: u64 = std::fs::metadata(stale.repo.join("f2.txt")).expect("the file").ino();
    let listing: String = debug_listing(stale.repo.as_path(), stale.indexes.commit_index.as_path(), "f2.txt");
    assert!(
        listing.contains(format!("ino: {inode}").as_str()),
        "the private index must record the inode the file has now, {inode}, so the scratch commit reads no file: {listing}"
    );
    remove(root.as_path());
}

/// A repository of `STALE_FILES` files whose index records stat data the worktree no longer has,
/// behind a clean filter that makes reading them slow,
/// as `stale_fixture` makes it,
/// and a Git that answers every refresh with `answering` instead of refreshing.
///  The rest of the command line goes to the real Git,
/// so only the refresh is the control's.
fn shim_git_that_answers_the_refresh(root: &Path, answering: &str) -> GitContext {
    let shim: PathBuf = root.join("shim-git");
    executable(
        shim.as_path(),
        format!("#!/bin/sh\ncase \"$*\" in\n  *\"update-index --refresh\"*) {answering} ;;\nesac\nexec {REAL_GIT} \"$@\"\n").as_bytes(),
    );
    let mut context: GitContext = git_context();
    context.real_git = shim;
    return context;
}

/// Staging of the named path with a context of the control's own,
/// so a Git that answers the refresh its own way is the one that prepares the commit.
fn prepare_stale_with(context: &GitContext, stale: &Stale) {
    let pathspecs: Vec<OsString> = vec![OsString::from("f1.txt")];
    let selection: IndexSelection = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: pathspecs.as_slice(),
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: stale.base.as_str(),
    };
    let real: PathBuf = stale.repo.join(".git").join("index");
    initialize_commit_index(context, stale.repo.as_path(), real.as_path(), &stale.indexes, &selection).expect("the preparation must not fail");
}

/// A rebuild after the branch moved keeps its refresh in its own private index too,
/// and leaves the real index alone:
///  a rebuild runs when other commits land,
/// which is the moment the real index lock is busiest.
#[test]
fn a_retry_keeps_the_refresh_in_the_private_index_and_leaves_the_real_one_alone() {
    let root: PathBuf = fixture("commit-index-stat-retry-private");
    let stale: Stale = stale_fixture(root.as_path());
    let real: PathBuf = stale.repo.join(".git").join("index");
    let before: Vec<u8> = std::fs::read(real.as_path()).expect("the real index");
    let tree: String = git_text(stale.repo.as_path(), &["rev-parse", format!("{}^{{tree}}", stale.base).as_str()]);
    initialize_retry_index(&git_context(), stale.repo.as_path(), real.as_path(), &stale.indexes, tree.as_str()).expect("retry index");
    assert!(
        std::fs::read(real.as_path()).expect("the real index") == before,
        "a rebuild must not write the real index, and the bytes of {} changed",
        real.display()
    );
    let inode: u64 = std::fs::metadata(stale.repo.join("f2.txt")).expect("the file").ino();
    let listing: String = debug_listing(stale.repo.as_path(), stale.indexes.commit_index.as_path(), "f2.txt");
    assert!(
        listing.contains(format!("ino: {inode}").as_str()),
        "the rebuilt index must record the inode the file has now, {inode}: {listing}"
    );
    remove(root.as_path());
}

/// A refresh Git refuses leaves the private index exactly as the copy and the staging made it,
/// and the preparation still answers success:
/// the stat data is a cache,
/// so a commit that cannot refresh it is slower and never failed.
///  The control above is what the recorded inode looks like when the refresh was kept,
/// and this one when it was not.
#[test]
fn a_private_refresh_that_fails_leaves_the_private_index_as_the_copy_made_it() {
    let root: PathBuf = fixture("commit-index-stat-private-failed");
    let stale: Stale = stale_fixture(root.as_path());
    let real: PathBuf = stale.repo.join(".git").join("index");
    let before: Vec<u8> = std::fs::read(real.as_path()).expect("the real index");
    let context: GitContext = shim_git_that_answers_the_refresh(root.as_path(), "exit 128");
    prepare_stale_with(&context, &stale);
    assert!(
        std::fs::read(real.as_path()).expect("the real index") == before,
        "the real index is never written by a preparation, and the bytes of {} changed",
        real.display()
    );
    let inode: u64 = std::fs::metadata(stale.repo.join("f2.txt")).expect("the file").ino();
    let listing: String = debug_listing(stale.repo.as_path(), stale.indexes.commit_index.as_path(), "f2.txt");
    assert!(
        !listing.contains(format!("ino: {inode}").as_str()),
        "a refresh Git refused records nothing, so the private index keeps the inode the copy carried: {listing}"
    );
    remove(root.as_path());
}

/// A refresh whose Git was killed records nothing either,
/// and the preparation still answers success.
#[test]
fn a_killed_private_refresh_leaves_the_private_index_as_the_copy_made_it() {
    let root: PathBuf = fixture("commit-index-stat-private-killed");
    let stale: Stale = stale_fixture(root.as_path());
    let context: GitContext = shim_git_that_answers_the_refresh(root.as_path(), "kill -9 $$");
    prepare_stale_with(&context, &stale);
    let inode: u64 = std::fs::metadata(stale.repo.join("f2.txt")).expect("the file").ino();
    let listing: String = debug_listing(stale.repo.as_path(), stale.indexes.commit_index.as_path(), "f2.txt");
    assert!(
        !listing.contains(format!("ino: {inode}").as_str()),
        "a refresh that was killed records nothing, so the private index keeps the inode the copy carried: {listing}"
    );
    remove(root.as_path());
}

/// A refresh the step's ceiling ends is a refresh that was not kept:
/// the preparation answers success at once instead of waiting for the Git it stopped,
/// and the private index holds what the copy carried.
#[test]
fn a_private_refresh_the_ceiling_ends_leaves_the_private_index_as_the_copy_made_it() {
    let root: PathBuf = fixture("commit-index-stat-private-ceiling");
    let stale: Stale = stale_fixture(root.as_path());
    let context: GitContext = shim_git_that_answers_the_refresh(root.as_path(), "sleep 30");
    let ceiling: std::time::Duration = std::time::Duration::from_millis(500);
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(ceiling)); });
    let began: std::time::Instant = std::time::Instant::now();
    prepare_stale_with(&context, &stale);
    let spent: std::time::Duration = began.elapsed();
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    assert!(
        spent < std::time::Duration::from_secs(20),
        "the step was ended at {ceiling:?} instead of waiting for a Git that sleeps 30 s, but the preparation took {spent:?}"
    );
    let inode: u64 = std::fs::metadata(stale.repo.join("f2.txt")).expect("the file").ino();
    let listing: String = debug_listing(stale.repo.as_path(), stale.indexes.commit_index.as_path(), "f2.txt");
    assert!(
        !listing.contains(format!("ino: {inode}").as_str()),
        "a refresh the ceiling ended records nothing, so the private index keeps the inode the copy carried: {listing}"
    );
    remove(root.as_path());
}

/// An index another program has locked is no obstacle to a preparation at all:
/// the preparation reads the real index and never writes it,
/// so a lock a person's own `git commit` holds costs the commit nothing.
#[test]
fn an_index_another_program_holds_does_not_stop_the_preparation() {
    let root: PathBuf = fixture("commit-index-stat-stale-locked");
    let stale: Stale = stale_fixture(root.as_path());
    let lock: PathBuf = stale.repo.join(".git").join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("a lock another program holds");
    prepare_stale(&stale);
    match commit_stale(&stale) {
        Ok(code) => assert_eq!(code, 0, "the scratch commit exited {code}"),
        Err(error) => panic!("the commit over an index another program locked did not finish: {error}"),
    }
    remove(root.as_path());
}

//endregion

//region An intent-to-add entry and the files a commit is not asked to read

/// The clean filter of the fixture:
/// it writes down every path Git cleans and passes the content through,
/// the way `git-lfs` answers the clean filter of every file it is asked about.
const CLEAN_FILTER: &str = "#!/bin/sh\nprintf '%s\\n' \"$1\" >> \"$0.log\"\ncat\n";

/// How many filtered files the fixture tracks.
const FILTERED_FILES: usize = 8;

/// A repository whose tracked files pass through a clean filter that records every run of it,
/// with one intent-to-add entry in its index and a shadow ready to prepare the commit of a named path.
struct Filtered {
    /// Repository the commit selects.
    repo: PathBuf,
    /// Shadow repository of the commit.
    shadow: PathBuf,
    /// Invocation capture.
    capture: InvocationCapture,
    /// Private indexes and object store.
    indexes: CaptureIndexes,
    /// The commit's own scratch directory.
    scratch: PathBuf,
    /// Recorded base.
    base: String,
    /// The clean filter script,
    /// whose `.log` beside it holds the paths it cleaned.
    filter: PathBuf,
}

/// The file the fixture's clean filter writes each path it cleans to.
fn clean_log(filter: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = filter.as_os_str().to_os_string();
    name.push(".log");
    return PathBuf::from(name);
}

/// What:
///  Builds the fixture:
///  `FILTERED_FILES` filtered files the commit does not name,
///  `other.txt` changed in the worktree,
///  and `ita.txt` added with `git add --intent-to-add`.
/// Why:
///   The intent-to-add entry is what made the private index be built from the base alone,
///  and only the clean filter can show whether a build asked Git to read a file.
fn filtered_fixture(root: &Path) -> Filtered {
    let repo: PathBuf = repository(root, "repo");
    let filter: PathBuf = root.join("clean-filter.sh");
    executable(filter.as_path(), CLEAN_FILTER.as_bytes());
    git(repo.as_path(), &["config", "filter.counter.clean", format!("'{}' %f", filter.display()).as_str()]);
    git(repo.as_path(), &["config", "filter.counter.required", "true"]);
    std::fs::write(repo.join(".gitattributes"), "*.dat filter=counter\n").expect("attributes");
    for number in 1..=FILTERED_FILES {
        std::fs::write(repo.join(format!("f{number}.dat")), format!("filtered {number}\n")).expect("a filtered file");
        // A file as new as the index file would be hashed by Git whatever its recorded stat data says.
        age(repo.join(format!("f{number}.dat")).as_path());
    }
    std::fs::write(repo.join("other.txt"), "other base\n").expect("other");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("ita.txt"), "intent content\n").expect("the added file");
    git(repo.as_path(), &["add", "--intent-to-add", "--", "ita.txt"]);
    std::fs::write(repo.join("other.txt"), "other changed\n").expect("the change the commit names");
    let context = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let transaction: PathBuf = root.join(format!("transaction-{TRANSACTION}"));
    std::fs::create_dir_all(transaction.as_path()).expect("transaction");
    let shadow: PathBuf =
        create_shadow_repository(&context, repo.as_path(), &capture, TRANSACTION, transaction.as_path()).expect("shadow");
    // The real object store, as the preparation of a commit names it: the trees and objects the private index makes are
    // read back by the controls below, as they are read back by the policies of a commit.
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: transaction.join("captured.index"),
        commit_index: transaction.join("commit.index"),
        object_directory: capture.layout.object_directory.clone(),
    };
    return Filtered { repo, shadow, capture, indexes, scratch: transaction, base, filter };
}

/// The paths the clean filter wrote down,
/// in the order it cleaned them.
fn cleaned(filter: &Path) -> Vec<String> {
    return match std::fs::read_to_string(clean_log(filter)) {
        Ok(text) => text.lines().map(String::from).collect(),
        Err(_) => Vec::new(),
    };
}

/// Forgets every path the clean filter wrote down,
/// so a control sees only what one step cleaned.
fn forget_cleaned(filter: &Path) {
    match std::fs::remove_file(clean_log(filter)) {
        Ok(()) => {}
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "clear the clean filter log"),
    }
}

/// Prepares the private index of the commit of `paths` and runs its scratch commit.
fn prepare_and_commit(filtered: &Filtered, paths: &[OsString]) -> Result<i32, PreparationError> {
    let selection: IndexSelection = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: paths,
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: filtered.base.as_str(),
    };
    let real: PathBuf = filtered.repo.join(".git").join("index");
    initialize_commit_index(&git_context(), filtered.repo.as_path(), real.as_path(), &filtered.indexes, &selection)
        .expect("private index");
    let commit_args: Vec<String> = vec![String::from("--quiet"), String::from("--message=filtered")];
    let request: PreparationRequest<'_> = PreparationRequest {
        capture: &filtered.capture,
        shadow: filtered.shadow.as_path(),
        commit_index: filtered.indexes.commit_index.as_path(),
        scratch: filtered.scratch.as_path(),
        global_args: &[],
        commit_args: commit_args.as_slice(),
        object_directory: None,
    };
    return run_native_commit_within(&git_context(), filtered.repo.as_path(), &request, SCRATCH_CEILING);
}

/// How long the scratch commit of a control may take:
/// long enough that no file of the fixture is a slow read,
/// and short enough that a control which hangs fails instead of waiting.
const SCRATCH_CEILING: std::time::Duration = std::time::Duration::from_secs(60);

/// The scratch commit reads nothing the commit did not name,
/// even though the real index holds an intent-to-add entry:
///  the private index keeps the real index's stat data,
/// and the clean filter of the other tracked files never runs.
#[test]
fn an_intent_to_add_entry_does_not_cost_the_commit_a_read_of_unnamed_files() {
    let root: PathBuf = fixture("commit-index-stat-intent-filter");
    let filtered: Filtered = filtered_fixture(root.as_path());
    forget_cleaned(filtered.filter.as_path());
    let code: i32 = prepare_and_commit(&filtered, &[OsString::from("other.txt")]).expect("the scratch commit finished");
    assert_eq!(code, 0, "the scratch commit exited {code}");
    let cleaned: Vec<String> = cleaned(filtered.filter.as_path());
    assert!(cleaned.is_empty(), "the scratch commit cleaned files it did not name: {cleaned:?}");
    remove(root.as_path());
}

/// Runs real Git in the fixture against `index`,
/// as the private-index steps of a commit do,
/// and requires success.
fn git_on_index(repo: &Path, index: &Path, arguments: &[&str]) {
    let output: Output = Command::new(REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_INDEX_FILE", index)
        .args(arguments)
        .output()
        .expect("real Git on the private index");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}

/// The same fixture over an index built from the base alone,
/// the way an index that cannot seed is built,
/// cleans every filtered file:
/// so the control above can fail,
/// and the entry that stops the copy is the intent-to-add one.
#[test]
fn an_index_built_from_the_base_alone_cleans_every_filtered_file() {
    let root: PathBuf = fixture("commit-index-stat-intent-filter-control");
    let filtered: Filtered = filtered_fixture(root.as_path());
    git_on_index(
        filtered.repo.as_path(),
        filtered.indexes.commit_index.as_path(),
        &["read-tree", filtered.base.as_str()],
    );
    git_on_index(
        filtered.repo.as_path(),
        filtered.indexes.commit_index.as_path(),
        &["add", "--all", "--", "other.txt"],
    );
    forget_cleaned(filtered.filter.as_path());
    let commit_args: Vec<String> = vec![String::from("--quiet"), String::from("--message=control")];
    let request: PreparationRequest<'_> = PreparationRequest {
        capture: &filtered.capture,
        shadow: filtered.shadow.as_path(),
        commit_index: filtered.indexes.commit_index.as_path(),
        scratch: filtered.scratch.as_path(),
        global_args: &[],
        commit_args: commit_args.as_slice(),
        object_directory: None,
    };
    run_native_commit_within(&git_context(), filtered.repo.as_path(), &request, SCRATCH_CEILING).expect("the scratch commit finished");
    let cleaned: Vec<String> = cleaned(filtered.filter.as_path());
    assert_eq!(cleaned.len(), FILTERED_FILES, "the base-alone index must clean every filtered file: {cleaned:?}");
    remove(root.as_path());
}

/// A tracked file the real index holds as intent-to-add keeps the base's content in the tree of a commit that names
/// another path,
/// and is not a path the commit changes:
///  the private index carries no intent-to-add entry,
/// and a tree built from one leaves that path out.
#[test]
fn an_intent_to_add_tracked_file_is_not_changed_by_a_commit_that_names_another_path() {
    let root: PathBuf = fixture("commit-index-stat-intent-tracked");
    let filtered: Filtered = filtered_fixture(root.as_path());
    std::fs::write(filtered.repo.join("tracked.txt"), "tracked base\n").expect("a tracked file");
    git(filtered.repo.as_path(), &["add", "--", "tracked.txt"]);
    git(filtered.repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let base: String = git_text(filtered.repo.as_path(), &["rev-parse", "HEAD"]);
    git(filtered.repo.as_path(), &["rm", "--cached", "--quiet", "--", "tracked.txt"]);
    git(filtered.repo.as_path(), &["add", "--intent-to-add", "--", "tracked.txt"]);
    std::fs::write(filtered.repo.join("tracked.txt"), "tracked changed\n").expect("content the commit does not take");
    let selection: IndexSelection = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: &[OsString::from("other.txt")],
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base.as_str(),
    };
    let real: PathBuf = filtered.repo.join(".git").join("index");
    initialize_commit_index(&git_context(), filtered.repo.as_path(), real.as_path(), &filtered.indexes, &selection)
        .expect("private index");
    let tree: String = write_private_tree(
        &git_context(),
        filtered.repo.as_path(),
        filtered.indexes.commit_index.as_path(),
        filtered.indexes.object_directory.as_path(),
    )
    .expect("tree");
    let content: String = git_text(filtered.repo.as_path(), &["show", format!("{tree}:tracked.txt").as_str()]);
    assert_eq!(content, "tracked base", "the commit keeps the base's content of a path it does not name");
    let changed: Vec<Vec<u8>> = list_changed_index_paths(
        &git_context(),
        filtered.repo.as_path(),
        filtered.indexes.commit_index.as_path(),
        base.as_str(),
        filtered.indexes.object_directory.as_path(),
    )
    .expect("the changed paths of the private index");
    assert_eq!(changed, vec![b"other.txt".to_vec()], "only the named path is changed by the commit");
    remove(root.as_path());
}

//endregion

//region Every selection against real Git

/// One way of committing,
/// as the wrapper prepares it and as real Git is given it.
struct CommitCase {
    /// Directory name of this case's fixtures.
    name: &'static str,
    /// The private index selection the wrapper prepares.
    mode: TransactionMode,
    /// The paths the command names.
    pathspecs: &'static [&'static str],
    /// Whether an index commit stages the named paths.
    stage_into_index: bool,
    /// Whether an index commit stages every tracked change.
    stage_tracked_changes: bool,
    /// Real Git's arguments after the message.
    arguments: &'static [&'static str],
}

/// The five ways of committing the same fixture is compared in:
/// a named path the person changed,
/// a named path held as intent-to-add,
/// `-a`,
/// `--include`,
/// and a plain commit.
const COMMIT_CASES: [CommitCase; 5] = [
    CommitCase {
        name: "named",
        mode: TransactionMode::ExplicitPath,
        pathspecs: &["a.txt"],
        stage_into_index: false,
        stage_tracked_changes: false,
        arguments: &["--", "a.txt"],
    },
    CommitCase {
        name: "named-intent",
        mode: TransactionMode::ExplicitPath,
        pathspecs: &["ita.txt"],
        stage_into_index: false,
        stage_tracked_changes: false,
        arguments: &["--", "ita.txt"],
    },
    CommitCase {
        name: "all",
        mode: TransactionMode::Index,
        pathspecs: &[],
        stage_into_index: false,
        stage_tracked_changes: true,
        arguments: &["--all"],
    },
    CommitCase {
        name: "include",
        mode: TransactionMode::Index,
        pathspecs: &["a.txt"],
        stage_into_index: true,
        stage_tracked_changes: false,
        arguments: &["--include", "--", "a.txt"],
    },
    CommitCase {
        name: "plain",
        mode: TransactionMode::Index,
        pathspecs: &[],
        stage_into_index: false,
        stage_tracked_changes: false,
        arguments: &[],
    },
];

/// What:
///  The fixture every case commits:
///  a tracked file the person removed from the index and added with intent,
///  a new file added with intent,
///  one changed file the cases name,
///  and one staged change a plain commit carries.
/// Why:
///   The same repository is built twice per case,
/// once for the wrapper and once for real Git,
///  so the two trees are comparable object for object.
fn compared_fixture(root: &Path) -> (PathBuf, CaptureIndexes, String) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    std::fs::write(repo.join("tracked.txt"), "tracked base\n").expect("the tracked file");
    std::fs::write(repo.join("empty"), "").expect("the tracked empty file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["rm", "--cached", "--quiet", "--", "tracked.txt"]);
    git(repo.as_path(), &["add", "--intent-to-add", "--", "tracked.txt"]);
    std::fs::write(repo.join("ita.txt"), "intent content\n").expect("the file added with intent");
    git(repo.as_path(), &["add", "--intent-to-add", "--", "ita.txt"]);
    std::fs::write(repo.join("tracked.txt"), "tracked changed\n").expect("the change no case named");
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("the change the named cases take");
    std::fs::write(repo.join("b.txt"), "staged b\n").expect("the change the index holds");
    git(repo.as_path(), &["add", "--", "b.txt"]);
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("captured.index"),
        commit_index: repo.join(".git").join("commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    return (repo, indexes, base);
}

/// The tree the wrapper's private index produces for `case`.
fn wrapper_tree(root: &Path, case: &CommitCase) -> String {
    let (repo, indexes, base): (PathBuf, CaptureIndexes, String) = compared_fixture(root);
    let named: Vec<OsString> = case.pathspecs.iter().map(|spec| { return OsString::from(spec); }).collect();
    let selection: IndexSelection = IndexSelection {
        mode: case.mode,
        pathspecs: named.as_slice(),
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: case.stage_into_index,
        stage_tracked_changes: case.stage_tracked_changes,
        base_revision: base.as_str(),
    };
    let real: PathBuf = repo.join(".git").join("index");
    initialize_commit_index(&git_context(), repo.as_path(), real.as_path(), &indexes, &selection).expect("private index");
    return write_private_tree(&git_context(), repo.as_path(), indexes.commit_index.as_path(), indexes.object_directory.as_path())
        .expect("tree");
}

/// The tree real Git commits for `case` in the same fixture.
fn real_git_tree(root: &Path, case: &CommitCase) -> String {
    let (repo, _indexes, _base): (PathBuf, CaptureIndexes, String) = compared_fixture(root);
    let mut arguments: Vec<&str> = vec!["commit", "--quiet", "--message=case"];
    arguments.extend(case.arguments.iter().copied());
    git(repo.as_path(), arguments.as_slice());
    return git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
}

/// Every selection commits the same tree as real Git,
/// with an intent-to-add entry on a tracked file and on a new file in the index:
///  the wrapper's private index is the intended tree,
///  so this compares what each side would commit,
/// not the real index a partial commit also updates.
#[test]
fn every_selection_commits_the_real_gits_tree_when_the_index_holds_intent_to_add_entries() {
    let root: PathBuf = fixture("commit-index-stat-intent-compare");
    for case in &COMMIT_CASES {
        let wrapper_root: PathBuf = root.join(format!("wrapper-{}", case.name));
        std::fs::create_dir(wrapper_root.as_path()).expect("wrapper fixture");
        let real_root: PathBuf = root.join(format!("real-{}", case.name));
        std::fs::create_dir(real_root.as_path()).expect("real fixture");
        let wrapper: String = wrapper_tree(wrapper_root.as_path(), case);
        let real: String = real_git_tree(real_root.as_path(), case);
        // The fixture's own HEAD is unchanged by a preparation, so its tree is what the comparison must see changed.
        let base: String = git_text(wrapper_root.join("repo").as_path(), &["rev-parse", "HEAD^{tree}"]);
        assert_ne!(wrapper, base, "the {} commit must change the tree, or this comparison proves nothing", case.name);
        assert_eq!(wrapper, real, "the {} selection must commit real Git's tree", case.name);
    }
    remove(root.as_path());
}

//endregion

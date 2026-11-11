//! Controls for the awaited faces of the preparation's own Git steps:
//! the two faces answer one tree and one staging,
//! a task can ask them where the synchronous faces are refused,
//! and a preparation with no unmerged file starts no Git at all.

use super::*;
use crate::commit_index::list_unmerged_index_paths;
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};

/// The lines of `git ls-files --stage` for one index:
/// every entry the index holds and the stage it holds it at.
/// Why:
///  A conflicted index holds one entry per side,
/// so the control reads the whole staging of the private index
///  rather than only its paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync(REAL_GIT, ['ls-files', '--stage'], { cwd: repo, env: { GIT_INDEX_FILE: index } })
/// ```
fn staged_entries(repo: &Path, index: &Path) -> String {
    let shown = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo)
        .env("GIT_INDEX_FILE", index)
        .args(["ls-files", "--stage"])
        .output()
        .expect("`ls-files` runs");
    assert!(shown.status.success(), "{}", String::from_utf8_lossy(&shown.stderr));
    return String::from_utf8_lossy(&shown.stdout).into_owned();
}

/// What:
///  A repository stopped inside a merge that conflicts,
/// with the private index and object store a preparation of it would use,
/// and the unmerged paths the preparation lists.
/// Why:
///  The staging start is the preparation's own step and it only has work when the index holds a conflict,
/// so a control
///  needs a real conflicted index and the same private state the preparation makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); branchAndConflict(repo); return { repo, indexes, unmerged };
/// ```
fn a_conflicted_repository(root: &Path) -> (PathBuf, CaptureIndexes, Vec<Vec<u8>>) {
    let repo: PathBuf = repository(root, "repo");
    let context: GitContext = git_context();
    std::fs::write(repo.join("a.txt").as_path(), b"base\n").expect("the base copy");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("a.txt").as_path(), b"other\n").expect("the other branch's copy");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=other"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt").as_path(), b"main\n").expect("this branch's copy");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let merging = git_output(repo.as_path(), &["merge", "--no-edit", "other"]);
    assert!(!merging.status.success(), "the two branches must conflict: {}", String::from_utf8_lossy(&merging.stderr));
    let real_index: PathBuf = repo.join(".git").join("index");
    let private: PathBuf = root.join("commit.index");
    std::fs::copy(real_index.as_path(), private.as_path()).expect("the private index starts as a copy of the real one");
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: real_index.clone(),
        commit_index: private,
        object_directory: repo.join(".git").join("objects"),
    };
    let unmerged: Vec<Vec<u8>> = list_unmerged_index_paths(&context, repo.as_path(), indexes.commit_index.as_path()).expect("the unmerged paths of the conflicted index");
    return (repo, indexes, unmerged);
}

/// What:
///  Both faces read the same base tree,
/// a task can ask the awaited one,
/// and a base that names no commit fails the same way through either.
/// Why:
///   The synchronous face is what the preparation calls today and the awaited one is what it calls once it is a task,
/// so a
///  difference here would compare the commit's own tree with another branch's and end a commit that holds changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(baseTreeOf(git, repo, 'HEAD'), await baseTreeOfAwaiting(git, repo, 'HEAD'))
/// ```
#[test]
fn the_awaited_base_tree_answers_what_the_synchronous_one_answers() {
    let root: PathBuf = fixture("prepare-awaiting-base-tree");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let synchronous: String = base_tree_of(&context, repo.as_path(), "HEAD").expect("the base tree");
    let awaited: String = crate::lazy_runtime::run_on_runtime(base_tree_of_awaiting(&context, repo.as_path(), "HEAD")).expect("the runtime").expect("the base tree");
    assert_eq!(synchronous, tree, "the synchronous face reads the tree the base names");
    assert_eq!(awaited, synchronous, "both faces read one base tree");
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let asked: Result<String, TransactionStop> = runtime.block_on(async { return base_tree_of_awaiting(&context, repo.as_path(), "HEAD").await; });
    assert_eq!(asked.expect("the awaited face asks Git inside a task"), tree, "the awaited face reads the base tree inside a task");
    let refused: Result<String, TransactionStop> = runtime.block_on(async { return base_tree_of(&context, repo.as_path(), "HEAD"); });
    assert!(refused.is_err(), "the synchronous face cannot start its Git from inside the runtime, which is why a task asks the awaited face");
    // A base that names no commit is Git's to refuse, in Git's own words, through either face.
    let missing: Result<String, TransactionStop> =
        crate::lazy_runtime::run_on_runtime(base_tree_of_awaiting(&context, repo.as_path(), "refs/heads/does-not-exist")).expect("the runtime");
    let refused_by_git: Result<String, TransactionStop> = base_tree_of(&context, repo.as_path(), "refs/heads/does-not-exist");
    assert_eq!(
        format!("{refused_by_git:?}"),
        format!("{missing:?}"),
        "both faces word a refusal by Git the same way"
    );
    assert!(refused_by_git.is_err(), "a base that names no commit is refused rather than read as a tree");
    remove(root.as_path());
}

/// What:
///  Both faces stage the working copy of every unmerged file into the private index the same way,
/// and the
///  staging a task asks for leaves one entry per file with no conflict stage.
/// Why:
///   The owner's rule is "Warn and finish the merge",
/// so the commit that follows the staging is the one Git would have
///  built for a resolved merge;
/// a face that staged another copy would record conflict markers of another side,
/// and one
///  that left the conflict stages in place would make Git refuse the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = staged(index); stageTheUnmergedFiles(...); const bySync = staged(index); copy(realIndex, index); await stageTheUnmergedFilesAwaiting(...); assert.equal(staged(index), bySync)
/// ```
#[test]
fn the_awaited_staging_stages_what_the_synchronous_one_stages() {
    let root: PathBuf = fixture("prepare-awaiting-staging");
    let (repo, indexes, unmerged): (PathBuf, CaptureIndexes, Vec<Vec<u8>>) = a_conflicted_repository(root.as_path());
    let context: GitContext = git_context();
    assert!(!unmerged.is_empty(), "the conflicted index names the file both sides changed");
    let conflicted: String = staged_entries(repo.as_path(), indexes.commit_index.as_path());
    assert!(conflicted.lines().count() > unmerged.len(), "the control starts from an index that holds one entry per side: {conflicted}");
    stage_the_unmerged_files(&context, repo.as_path(), &indexes, unmerged.as_slice()).expect("the synchronous staging");
    let by_synchronous: String = staged_entries(repo.as_path(), indexes.commit_index.as_path());
    // The private index of the second face is a fresh copy of the real one, so each face starts from the same conflict.
    std::fs::copy(indexes.captured_index.as_path(), indexes.commit_index.as_path()).expect("a fresh private index");
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let staged: Result<(), TransactionStop> = runtime.block_on(async { return stage_the_unmerged_files_awaiting(&context, repo.as_path(), &indexes, unmerged.as_slice()).await; });
    staged.expect("the awaited face stages the unmerged files inside a task");
    let by_awaited: String = staged_entries(repo.as_path(), indexes.commit_index.as_path());
    assert_eq!(by_awaited, by_synchronous, "both faces stage the working copy of every unmerged file the same way");
    assert_eq!(by_awaited.lines().count(), unmerged.len(), "the staging leaves one entry per file and no conflict stage: {by_awaited}");
    let worktree: String = git_text(repo.as_path(), &["hash-object", "a.txt"]);
    assert!(by_awaited.contains(worktree.as_str()), "the entry holds the bytes the work tree carries, as `git add` stages them: {by_awaited}");
    std::fs::copy(indexes.captured_index.as_path(), indexes.commit_index.as_path()).expect("a fresh private index");
    let refused: Result<(), TransactionStop> = runtime.block_on(async { return stage_the_unmerged_files(&context, repo.as_path(), &indexes, unmerged.as_slice()); });
    assert!(refused.is_err(), "the synchronous face cannot start its Git from inside the runtime, which is why a task asks the awaited face");
    remove(root.as_path());
}

/// What:
///  A preparation with no unmerged file starts no Git at all,
/// through either face.
/// Why:
///  The staging is skipped when the index holds no conflict,
/// and a start that was not needed is a child the commit pays for;
/// the
///  context here names a program that does not exist,
/// so a face that started one would fail instead of answering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nowhere = { ...gitContext, realGit: '/nonexistent' }; assert.deepEqual(stageTheUnmergedFiles(nowhere, repo, indexes, []), undefined)
/// ```
#[test]
fn a_commit_with_no_unmerged_file_starts_no_staging() {
    let root: PathBuf = fixture("prepare-awaiting-no-staging");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let nowhere: GitContext = GitContext { real_git: PathBuf::from("/nonexistent/this-control-starts-no-git"), ..git_context() };
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("index"),
        commit_index: root.join("commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    stage_the_unmerged_files(&nowhere, repo.as_path(), &indexes, &[]).expect("nothing is staged, so nothing is started");
    let awaited: Result<(), TransactionStop> = crate::lazy_runtime::run_on_runtime(stage_the_unmerged_files_awaiting(&nowhere, repo.as_path(), &indexes, &[])).expect("the runtime");
    awaited.expect("the awaited face starts nothing for a commit with no unmerged file");
    remove(root.as_path());
}

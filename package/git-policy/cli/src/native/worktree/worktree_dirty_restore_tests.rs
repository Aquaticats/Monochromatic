//! Controls for the restore of a snapshot of a main checkout with changes:
//! the proof accepts what Git's checkout leaves and
//! names what it does not,
//! the Git runs are exactly the two the design lists,
//! the ignored files of the source survive a commit that ignores nothing,
//! an edit the reset cannot see is replaced by the commit's bytes,
//! also when the index recorded no nanoseconds,
//! and a restore never runs in the source.

use super::*;
use crate::test_support::{
    REAL_GIT,
    fixture,
    git,
    git_context,
    git_output,
    remove,
    repository,
};
use crate::worktree_clean::StatEntry;
use crate::worktree_copy::{
    GitStatus,
    GitStep,
};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_ladder_test_support::{
    as_a_git_without_nanoseconds_wrote_it,
    hide_an_edit,
};
use crate::worktree_snapshot_index_step::install_index;
use std::path::{
    Path,
    PathBuf,
};

/// The words every step starts with:
/// `-C` and the stage.
const LEADING_WORDS: usize = 2;

/// What:
///  The commit `revision` names in `repo`,
/// as hexadecimal.
/// Why:
///   The fixture names the two commits it restores between by what they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = commitOf(repo, 'HEAD')
/// ```
fn commit_of(
    repo: &Path,
    revision: &str,
) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", revision]).stdout)
        .trim()
        .to_string();
}

/// What:
///  Copy the files of `from` into `to`,
/// leaving out the top-level `.git`,
/// the way a snapshot's files reach the stage.
/// Why:
///   The stage is the source's files with the new worktree's Git directory,
/// not the source's `.git`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cpSync(from, to, { recursive: true, filter: (name) => name !== '.git' })
/// ```
fn copy_files(
    from: &Path,
    to: &Path,
) {
    std::fs::create_dir_all(to).expect("stage directory");
    for listed in std::fs::read_dir(from).expect("the source lists") {
        let entry: std::fs::DirEntry = listed.expect("an entry");
        let name: std::ffi::OsString = entry.file_name();
        if name == ".git" {
            continue;
        }
        let copy: PathBuf = to.join(&name);
        if entry.file_type().expect("a kind").is_dir() {
            copy_files(entry.path().as_path(), copy.as_path());
        } else {
            std::fs::copy(entry.path(), copy.as_path()).expect("a copied file");
        }
    }
}

/// What:
///  A repository beside the fixture,
/// and its Git directory,
/// to be the source a restore is told about.
/// Why:
///   The restore's content decision reads the source's configuration and attributes files,
/// so the source it is
///  handed must be a real repository;
/// the stage's own directory and the new worktree's Git directory are both
///  taken by the test,
/// so the source is a third one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = join(repository(root, 'elsewhere'), '.git');
/// ```
fn elsewhere_git_dir(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "elsewhere");
    return repo.join(".git");
}

/// What:
///  The administrative directory of the worktree at `destination`,
/// read from the pointer Git wrote.
/// Why:
///   The restore runs under the Git directory Git made,
/// and the fixture installs the stage's index into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const admin = readFileSync(join(destination, '.git'), 'utf8').replace('gitdir: ', '').trim()
/// ```
fn admin_of(destination: &Path) -> PathBuf {
    let pointer: String = std::fs::read_to_string(destination.join(".git")).expect("Git's pointer");
    let target: &str = pointer
        .strip_prefix("gitdir: ")
        .expect("a gitdir pointer")
        .trim();
    return PathBuf::from(target);
}

/// What:
///  A runner that runs the ladder's own steps as real Git runs them.
/// Why:
///   The restore's behavior is Git's own reset and clean,
/// so a control that records the words instead proves nothing about the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = (step) => spawnSync(realGit, step.arguments, { env: { ...process.env, ...step.environment } })
/// ```
fn real_steps(root: &Path) -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, WorktreeCopyError> {
    let home: PathBuf = root.join("home");
    std::fs::create_dir_all(home.as_path()).expect("a home for the Git child");
    return move |step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> {
        let GitStep::Other {
            arguments,
            environment,
        } = step
        else {
            panic!("only the ladder's own steps are expected")
        };
        let status: std::process::ExitStatus = std::process::Command::new(REAL_GIT)
            .args(*arguments)
            .envs(environment.iter().cloned())
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", home.as_path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", home.join("no-global-config").as_path())
            .status()
            .expect("real Git starts");
        return Ok(GitStatus {
            code: status.code(),
        });
    };
}

/// A target for the commit `HEAD` names in `repo`,
/// whose Git directory is its own.
fn target_of(repo: &Path) -> Target {
    let commit: String = String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_string();
    return Target {
        admin: repo.join(".git"),
        index: None,
        pointer: Vec::new(),
        commit,
        differs: false,
        line: String::new(),
    };
}

/// A repository with one committed file.
fn committed(root: &Path) -> PathBuf {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "restored")).expect("canonical");
    std::fs::write(repo.join("a.txt"), b"a\n").expect("file");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    git(repo.as_path(), &["update-index", "--refresh"]);
    return repo;
}

/// The proof accepts a worktree that equals its commit,
/// a carried ignored path,
/// and names each way it does not.
#[test]
fn the_proof_accepts_what_gits_checkout_leaves_and_names_what_it_does_not() {
    let root: PathBuf = crate::test_support::fixture("restore-proof");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    assert_eq!(
        prove_restored(&git_context(), &target, repo.as_path(), &[]),
        Ok(()),
        "positive control"
    );
    std::fs::write(repo.join("untracked.txt"), b"u").expect("untracked");
    let untracked: String = prove_restored(&git_context(), &target, repo.as_path(), &[])
        .expect_err("an untracked file");
    assert!(
        untracked.contains("untracked.txt") && untracked.contains("still untracked"),
        "{untracked}"
    );
    assert_eq!(
        prove_restored(
            &git_context(),
            &target,
            repo.as_path(),
            &[b"untracked.txt".to_vec()]
        ),
        Ok(()),
        "the same file is carried"
    );
    std::fs::create_dir(repo.join("cache")).expect("ignored directory");
    std::fs::write(repo.join("cache/data.bin"), b"d").expect("ignored file");
    assert!(
        prove_restored(
            &git_context(),
            &target,
            repo.as_path(),
            &[b"untracked.txt".to_vec()]
        )
        .expect_err("a file inside an ignored directory")
        .contains("cache"),
        "a carried directory's name is not the name of a file in it"
    );
    assert_eq!(
        prove_restored(
            &git_context(),
            &target,
            repo.as_path(),
            &[b"untracked.txt".to_vec(), b"cache/".to_vec()]
        ),
        Ok(()),
        "a carried directory covers what it holds"
    );
    std::fs::remove_dir_all(repo.join("cache")).expect("remove");
    std::fs::remove_file(repo.join("untracked.txt")).expect("remove");
    git(repo.as_path(), &["rm", "--cached", "--quiet", "a.txt"]);
    assert!(
        prove_restored(&git_context(), &target, repo.as_path(), &[])
            .expect_err("a staged deletion")
            .contains("index")
    );
    git(repo.as_path(), &["add", "a.txt"]);
    std::fs::write(repo.join("a.txt"), b"changed\n").expect("edit");
    assert!(
        prove_restored(&git_context(), &target, repo.as_path(), &[])
            .expect_err("an edit")
            .contains("differs")
    );
    remove(root.as_path());
}

/// The restore lists the stage's ignored paths,
/// then runs a captured `clean --force -d`,
/// then `read-tree --reset -u <commit>`
///  through the lifecycle's runner,
/// in that order,
/// each in the stage under the new worktree's Git directory;
/// nothing else is run through the runner,
/// and the `clean`
///  carries neither `--quiet` nor `-q`.
#[test]
fn the_restore_runs_read_tree_through_the_runner_and_a_clean_without_quiet_in_the_stage() {
    let root: PathBuf = crate::test_support::fixture("restore-steps");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    let stage: PathBuf = repo.clone();
    let mut seen: Vec<Vec<String>> = Vec::new();
    let mut environments: Vec<Vec<(String, String)>> = Vec::new();
    let mut runner = |step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> {
        let GitStep::Other {
            arguments,
            environment,
        } = step
        else {
            panic!("only the ladder's own steps are expected")
        };
        seen.push(
            arguments
                .iter()
                .map(|word| {
                    return word.to_string_lossy().into_owned();
                })
                .collect(),
        );
        environments.push(
            environment
                .iter()
                .map(|(name, value)| {
                    return (
                        name.to_string_lossy().into_owned(),
                        value.to_string_lossy().into_owned(),
                    );
                })
                .collect(),
        );
        return Ok(GitStatus {
            code: Some(0),
        });
    };
    let elsewhere_git: PathBuf = elsewhere_git_dir(root.as_path());
    let other_root: PathBuf = elsewhere_git
        .parent()
        .expect("the Git directory's parent")
        .to_path_buf();
    let source = SnapshotSource {
        root: other_root.clone(),
        git_dir: elsewhere_git.clone(),
        common_dir: elsewhere_git.clone(),
        clean: crate::worktree_clean::CleanSource {
            head_commit: target.commit.clone(),
            head_tree: Vec::new(),
            tracked: 0,
            index_seconds: 0,
            hash_length: 20,
            attribute_blobs: Vec::new(),
            index_bytes: Vec::new(),
        },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path())
            .expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let (recording, log) = crate::recording_git::recording_context(root.as_path());
    restore_dirty(&source, &target, stage.as_path(), &mut runner, &recording, &crate::fixture_content_decision::content_environment()).unwrap_or_else(|reason| {
        let differences: std::process::Output = git_output(stage.as_path(), &["diff-files"]);
        panic!("the proof holds for a worktree that equals its commit: {reason}; Git says {} and the file holds {:?}", String::from_utf8_lossy(differences.stdout.as_slice()), std::fs::read(stage.join("a.txt")))
    });
    let words = |step: &Vec<String>| -> Vec<String> {
        return step[LEADING_WORDS..].to_vec();
    };
    let expected: Vec<Vec<String>> = vec![
        ["read-tree", "--reset", "-u", target.commit.as_str()]
            .map(String::from)
            .to_vec(),
    ];
    assert_eq!(
        seen.iter().map(words).collect::<Vec<Vec<String>>>(),
        expected,
        "exactly this step goes through the runner"
    );
    let cleans: Vec<String> = crate::recording_git::recorded_starts(log.as_path())
        .into_iter()
        .filter(|start| {
            return start.split(' ').any(|word| {
                return word == "clean";
            });
        })
        .collect();
    assert_eq!(cleans.len(), 1, "one captured clean: {cleans:?}");
    assert!(cleans[0].ends_with("clean --force -d"), "{cleans:?}");
    assert_eq!(
        crate::recording_git::quiet_starts(log.as_path()),
        Vec::<String>::new(),
        "no start carries --quiet or -q"
    );
    for (arguments, environment) in seen.iter().zip(&environments) {
        assert_eq!(
            &arguments[..LEADING_WORDS],
            ["-C", stage.to_string_lossy().as_ref()]
        );
        assert!(
            environment.contains(&(
                String::from("GIT_DIR"),
                target.admin.to_string_lossy().into_owned()
            )),
            "{environment:?}"
        );
        assert!(
            environment.contains(&(
                String::from("GIT_WORK_TREE"),
                stage.to_string_lossy().into_owned()
            )),
            "{environment:?}"
        );
    }
    remove(root.as_path());
}

/// A restore that would run in the source's own directory or Git directory runs nothing.
#[test]
fn a_restore_never_runs_in_the_source() {
    let root: PathBuf = crate::test_support::fixture("restore-guard");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    let mut runner = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> {
        panic!("no Git run is expected")
    };
    let source = |source_root: PathBuf, git_dir: PathBuf| -> SnapshotSource {
        return SnapshotSource {
            root: source_root,
            git_dir: git_dir.clone(),
            common_dir: git_dir,
            clean: crate::worktree_clean::CleanSource {
                head_commit: target.commit.clone(),
                head_tree: Vec::new(),
                tracked: 0,
                index_seconds: 0,
                hash_length: 20,
                attribute_blobs: Vec::new(),
                index_bytes: Vec::new(),
            },
            nested: Vec::new(),
            config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path())
                .expect("configuration"),
            trash_ignored: false,
            dirty: None,
        };
    };
    let in_source = restore_dirty(
        &source(repo.clone(), repo.join(".git")),
        &target,
        repo.as_path(),
        &mut runner,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    )
    .expect_err("the stage is the source");
    assert!(in_source.contains("source's own directory"), "{in_source}");
    let elsewhere: PathBuf = root.join("elsewhere");
    let in_git_dir = restore_dirty(
        &source(elsewhere.clone(), repo.join(".git")),
        &target,
        elsewhere.as_path(),
        &mut runner,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    )
    .expect_err("the Git directory is the source's");
    assert!(
        in_git_dir.contains("source's own directory"),
        "{in_git_dir}"
    );
    remove(root.as_path());
}

/// A failing step or a failing proof is an error,
/// so the rung ends and Git's checkout makes the worktree.
#[test]
fn a_failed_step_or_a_failed_proof_is_an_error() {
    let root: PathBuf = crate::test_support::fixture("restore-failure");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    let elsewhere_git: PathBuf = elsewhere_git_dir(root.as_path());
    let other_root: PathBuf = elsewhere_git
        .parent()
        .expect("the Git directory's parent")
        .to_path_buf();
    let source = SnapshotSource {
        root: other_root.clone(),
        git_dir: elsewhere_git.clone(),
        common_dir: elsewhere_git.clone(),
        clean: crate::worktree_clean::CleanSource {
            head_commit: target.commit.clone(),
            head_tree: Vec::new(),
            tracked: 0,
            index_seconds: 0,
            hash_length: 20,
            attribute_blobs: Vec::new(),
            index_bytes: Vec::new(),
        },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path())
            .expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let mut failing = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> {
        return Ok(GitStatus {
            code: Some(128),
        });
    };
    assert!(
        restore_dirty(
            &source,
            &target,
            repo.as_path(),
            &mut failing,
            &git_context(),
            &crate::fixture_content_decision::content_environment()
        )
        .expect_err("the reset failed")
        .contains("read-tree")
    );
    let mut succeeding = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> {
        return Ok(GitStatus {
            code: Some(0),
        });
    };
    std::fs::write(repo.join("left.txt"), b"left").expect("a file the clean left behind");
    // A Git whose `clean` does nothing, so the file stays and the proof is what finds it.
    let idle_clean: PathBuf = root.join("idle-clean-git");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in *\" clean \"*) exit 0;; esac\nexec {} \"$@\"\n",
        crate::test_support::REAL_GIT
    );
    crate::test_support::executable(idle_clean.as_path(), script.as_bytes());
    let mut leaving: GitContext = git_context();
    leaving.real_git = idle_clean;
    assert!(
        restore_dirty(
            &source,
            &target,
            repo.as_path(),
            &mut succeeding,
            &leaving,
            &crate::fixture_content_decision::content_environment()
        )
        .expect_err("the proof fails")
        .contains("left.txt")
    );
    remove(root.as_path());
}

/// What:
///  A restore keeps the ignored files of the source that the new commit does not ignore,
/// and its proof accepts them as carried.
/// Why:
///   The stage's ignore rules are the source's until the reset brings the new commit's,
/// so a clean after the reset deletes ignored files the new commit does not cover:
/// a person's cache,
/// local configuration or environment file that the new branch knows nothing about.
///   The control is the clean source,
/// which hands the same directory over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await restoreDirty(source, target, stage, realGit, context)
/// assert(readFileSync(join(stage, 'cache/data.bin')))
/// ```
#[test]
fn a_restore_keeps_the_ignored_files_the_new_commit_does_not_ignore() {
    let root: PathBuf = fixture("restore-kept-ignored");
    let repo: PathBuf = repository(root.as_path(), "source");
    std::fs::write(repo.join(".gitignore"), b"cache/\n").expect("the source's ignore rule");
    std::fs::write(repo.join("tracked.txt"), b"first\n").expect("a tracked file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let first: String = commit_of(repo.as_path(), "HEAD");
    // The commit the new worktree checks out ignores nothing: a branch that predates the rule, or one that removed it.
    std::fs::write(repo.join(".gitignore"), b"").expect("no ignore rule");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let second: String = commit_of(repo.as_path(), "HEAD");
    // The main checkout the snapshot is taken from: the first commit, one uncommitted edit, and its ignored state.
    git(repo.as_path(), &["reset", "--hard", first.as_str()]);
    std::fs::write(repo.join(".gitignore"), b"cache/\n").expect("the source's ignore rule again");
    git(repo.as_path(), &["update-index", "--refresh"]);
    std::fs::create_dir(repo.join("cache")).expect("the ignored directory");
    std::fs::write(repo.join("cache/data.bin"), b"ignored bytes\n").expect("the ignored file");
    std::fs::write(repo.join("tracked.txt"), b"first\nedited\n").expect("the uncommitted edit");
    std::fs::write(repo.join("untracked.txt"), b"not ignored\n").expect("an untracked file");
    // Git's own registration without a checkout, and the source's index installed as the rung installs it.
    let destination: PathBuf = root.join("topic");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--no-checkout",
            "--quiet",
            destination.to_string_lossy().as_ref(),
            second.as_str(),
        ],
    );
    let admin: PathBuf = admin_of(destination.as_path());
    std::fs::copy(repo.join(".git/index"), admin.join("index")).expect("the stage's index");
    let stage: PathBuf = root.join("stage");
    copy_files(repo.as_path(), stage.as_path());
    let target: Target = Target {
        admin: admin.clone(),
        index: None,
        pointer: Vec::new(),
        commit: second,
        differs: false,
        line: String::new(),
    };
    let source = SnapshotSource {
        root: repo.clone(),
        git_dir: repo.join(".git"),
        common_dir: repo.join(".git"),
        clean: crate::worktree_clean::CleanSource {
            head_commit: first,
            head_tree: Vec::new(),
            tracked: 0,
            index_seconds: 0,
            hash_length: 20,
            attribute_blobs: Vec::new(),
            index_bytes: Vec::new(),
        },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path())
            .expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let mut steps = real_steps(root.as_path());
    let outcome: Result<(), String> = restore_dirty(
        &source,
        &target,
        stage.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    );
    assert_eq!(outcome, Ok(()), "the restore and its proof hold");
    assert_eq!(
        std::fs::read(stage.join("cache/data.bin")).expect("the ignored file is still there"),
        b"ignored bytes\n"
    );
    assert_eq!(
        std::fs::read(stage.join("tracked.txt")).expect("the tracked file is the commit's"),
        b"first\n"
    );
    assert!(
        !stage.join("untracked.txt").exists(),
        "an untracked file that no ignore rule covers goes"
    );
    remove(root.as_path());
}

/// What:
///  A restore given a private index reads and writes that file and never the
///  worktree's own name,
///  whatever stands there.
/// Why:
///   The ZFS clone route writes the fill's index under `index.pending` and gives
///  it the worktree's name only once the tree is proven
///  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
///  "The restore" step 5 and T6b):
///  the reset,
///  the settle's content read and the proof would each believe a name the
///  worktree does not hold,
///  or a file another command left there.
///   The three arms are the three sites:
///  no index under the real name at all,
///  a file there that is no index,
///  and a real index there whose entries name other blobs.
///  The control is the same fixture with the private name not given,
///  which is the build before this change:
///  it refuses on the second arm,
///  so the guard is
///  what the first three arms pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(restoreDirty(source, { ...target, index: pending }, stage)).toEqual({ ok: true });
/// expect(restoreDirty(source, target, stage)).not.toEqual({ ok: true });
/// ```
#[test]
fn the_restore_reads_the_private_index_and_never_the_real_name() {
    let root: PathBuf = crate::test_support::fixture("restore-private-index");
    let repo: PathBuf = repository(root.as_path(), "source");
    std::fs::write(repo.join("a.txt"), COMMITTED).expect("the tracked file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let elsewhere: PathBuf = repository(root.as_path(), "elsewhere");
    std::fs::write(elsewhere.join("a.txt"), b"other bytes\n").expect("the decoy's file");
    git(elsewhere.as_path(), &["add", "--", "a.txt"]);
    git(
        elsewhere.as_path(),
        &["commit", "--quiet", "--message=their own"],
    );
    let commit: String = commit_of(repo.as_path(), "HEAD");
    let destination: PathBuf = root.join("topic");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            destination.to_string_lossy().as_ref(),
            commit.as_str(),
        ],
    );
    let admin: PathBuf = admin_of(destination.as_path());
    let stage: PathBuf = root.join("stage");
    copy_files(repo.as_path(), stage.as_path());
    std::fs::copy(repo.join(".git/index"), admin.join("index.pending")).expect("the private index");
    std::fs::copy(repo.join(".git/index"), admin.join("index"))
        .expect("the worktree's own index, as Git's own add leaves it");
    let source = SnapshotSource {
        root: repo.clone(),
        git_dir: repo.join(".git"),
        common_dir: repo.join(".git"),
        clean: crate::worktree_clean::CleanSource {
            head_commit: commit.clone(),
            head_tree: Vec::new(),
            tracked: 1,
            index_seconds: 0,
            hash_length: 20,
            attribute_blobs: Vec::new(),
            index_bytes: std::fs::read(repo.join(".git/index")).expect("the source's index"),
        },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path())
            .expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let private: Target = Target {
        admin: admin.clone(),
        index: Some(admin.join("index.pending")),
        pointer: Vec::new(),
        commit: commit.clone(),
        differs: false,
        line: String::new(),
    };
    let real: Target = Target {
        admin: admin.clone(),
        index: None,
        pointer: Vec::new(),
        commit,
        differs: false,
        line: String::new(),
    };
    let mut steps = real_steps(root.as_path());
    // Arm one: no index under the real name at all.
    let without_a_real_index: Result<(), String> = restore_dirty(
        &source,
        &private,
        stage.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    );
    assert_eq!(
        without_a_real_index,
        Ok(()),
        "the private name is the one the restore reads"
    );
    assert_eq!(
        std::fs::read(stage.join("a.txt")).expect("the file"),
        COMMITTED
    );
    // Arm two: a file that is no index stands under the real name. The control
    // is the same fixture without the private name, which reads it and refuses.
    std::fs::write(admin.join("index"), b"not an index").expect("the control's file");
    let with_a_decoy: Result<(), String> = restore_dirty(
        &source,
        &private,
        stage.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    );
    assert_eq!(
        with_a_decoy,
        Ok(()),
        "a file the restore never reads cannot refuse it"
    );
    let control: String = restore_dirty(
        &source,
        &real,
        stage.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    )
    .expect_err("the real name is what the build before this change read");
    assert!(
        control.contains("ignored files"),
        "the control reads the name the private index keeps the restore off: {control}"
    );
    // Arm three: a real index whose entries name blobs of another repository.
    std::fs::copy(elsewhere.join(".git/index"), admin.join("index"))
        .expect("the decoy's real index");
    let with_another_repositorys_index: Result<(), String> = restore_dirty(
        &source,
        &private,
        stage.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    );
    assert_eq!(
        with_another_repositorys_index,
        Ok(()),
        "another repository's entries never decide this restore"
    );
    assert_eq!(
        std::fs::read(stage.join("a.txt")).expect("the file"),
        COMMITTED,
        "the reset wrote the private index's blob"
    );
    remove(root.as_path());
}

/// The note counts the paths in the singular and the plural.
#[test]
fn the_note_counts_paths() {
    let one = ChangedPaths {
        count: 1,
        tracked: 1,
        first: Some(String::from("a")),
        attributes_changed: false,
    };
    let many = ChangedPaths {
        count: 3,
        tracked: 3,
        first: Some(String::from("a")),
        attributes_changed: false,
    };
    assert!(
        left_out_note(&one).contains(format!("({} path)", one.count).as_str()),
        "{}",
        left_out_note(&one)
    );
    assert!(
        left_out_note(&many).contains(format!("({} paths)", many.count).as_str()),
        "{}",
        left_out_note(&many)
    );
    assert!(left_out_note(&one).ends_with('\n'));
}

/// What:
///  A carried path under a directory Git reports as one collapsed name counts as carried,
/// while a directory holding
///  one file the restore did not carry is still refused.
/// Why:
///   The proof lists untracked names with `--directory`,
/// so Git answers `cache/` when everything below it is untracked,
///  while the restore carried `cache/private/token`;
/// a comparison that only looked below a carried name refused the
///  correct restore and sent the rung to a full checkout.
/// The whole content decides the ancestor,
/// so one file
///  that was not carried still ends the rung.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(proveRestored({ carried: ['cache/private/token'] })).toEqual({ ok: true });
/// writeFileSync('cache/other.txt'); expect(proveRestored({ carried: ['cache/private/token'] }).reason).toContain('cache/');
/// ```
#[test]
fn a_collapsed_untracked_directory_is_carried_only_when_all_of_it_is() {
    let root: PathBuf = crate::test_support::fixture("restore-carried-ancestor");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    std::fs::create_dir_all(repo.join("cache/private")).expect("the ignored directories");
    std::fs::write(repo.join("cache/private/token"), b"t").expect("the carried file");
    let carried: Vec<Vec<u8>> = vec![b"cache/private/token".to_vec()];
    assert_eq!(
        prove_restored(&git_context(), &target, repo.as_path(), carried.as_slice()),
        Ok(()),
        "Git reports cache/ and everything under it is carried"
    );
    std::fs::write(repo.join("cache/other.txt"), b"o").expect("a file the restore did not carry");
    let refused: String =
        prove_restored(&git_context(), &target, repo.as_path(), carried.as_slice())
            .expect_err("a directory holding a file that was not carried");
    assert!(
        refused.contains("cache/"),
        "the refusal names the directory Git reported: {refused}"
    );
    remove(root.as_path());
}

/// The bytes the fixture commits,
/// and the same-length bytes an edit puts in their place.
const COMMITTED: &[u8] = b"first\n";
const EDITED: &[u8] = b"other\n";

/// What:
///  A stage whose index and files are one instant,
///  with an edit Git's own stat check cannot see,
/// and the
///  pieces a restore of it is told about.
/// Why:
///   Both controls of the hidden edit need the same stage,
/// its administrative directory,
/// the index
///  the snapshot carries,
/// the source and the target;
/// they differ only in the index they install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stage, admin, seed, source, target } = hiddenEditFixture(root);
/// ```
fn hidden_edit_fixture(root: &Path) -> (PathBuf, PathBuf, Vec<u8>, SnapshotSource, Target) {
    let repo: PathBuf = repository(root, "stage");
    std::fs::write(repo.join("a.txt"), COMMITTED).expect("the tracked file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            "../topic",
        ],
    );
    let admin: PathBuf = admin_of(root.join("topic").as_path());
    // The edit is made after the index recorded the file, and Git's own check cannot see it.
    assert!(
        hide_an_edit(repo.as_path(), "a.txt", COMMITTED, EDITED),
        "the fixture hides its edit from Git's stat check"
    );
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the edit"), EDITED);
    assert!(
        git_output(repo.as_path(), &["diff-files", "--quiet"])
            .status
            .success(),
        "the fixture's edit is hidden from Git's own check"
    );
    let seed: Vec<u8> =
        std::fs::read(repo.join(".git/index")).expect("the index the snapshot carries");
    let head: String = commit_of(repo.as_path(), "HEAD");
    let elsewhere_git: PathBuf = elsewhere_git_dir(root);
    let elsewhere: PathBuf = elsewhere_git
        .parent()
        .expect("the Git directory's parent")
        .to_path_buf();
    let source = SnapshotSource {
        root: elsewhere.clone(),
        git_dir: elsewhere_git.clone(),
        common_dir: elsewhere_git,
        clean: crate::worktree_clean::CleanSource {
            head_commit: head.clone(),
            head_tree: Vec::new(),
            tracked: 1,
            index_seconds: 0,
            hash_length: 20,
            attribute_blobs: Vec::new(),
            index_bytes: seed.clone(),
        },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path())
            .expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let target: Target = Target {
        admin: admin.clone(),
        index: None,
        pointer: Vec::new(),
        commit: head,
        differs: false,
        line: String::new(),
    };
    return (repo, admin, seed, source, target);
}

/// The assertions a restore of a hidden edit has to hold:
/// the stage holds the commit's bytes,
/// and Git's own check of the
///  installed index calls the stage clean.
/// Why:
///   Both controls publish the same state and differ only in the entry the index records,
/// so the judgement of
///  that state is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readFileSync(join(stage, 'a.txt'))).toEqual(COMMITTED); expect(git(stage, ['diff-files', '--quiet'])).toBeClean();
/// ```
fn assert_the_stage_holds_the_commits_bytes(
    repo: &Path,
    admin: &Path,
) {
    assert_eq!(
        std::fs::read(repo.join("a.txt")).expect("the restored file"),
        COMMITTED,
        "the copy holds the commit's bytes, as Git's own checkout makes them"
    );
    // The stage carries the new worktree's Git pointer in the product, so Git reads the installed index there and not
    //  the source's own.
    let in_the_stage: [String; 4] = [
        "--git-dir",
        admin.to_string_lossy().as_ref(),
        "--work-tree",
        repo.to_string_lossy().as_ref(),
    ]
    .map(String::from);
    let differences: Vec<String> = in_the_stage
        .iter()
        .cloned()
        .chain(["diff-files"].map(String::from))
        .collect();
    let after: std::process::Output = git_output(repo, differences.as_slice());
    let recorded: Vec<String> = in_the_stage
        .iter()
        .cloned()
        .chain(["ls-files", "--debug"].map(String::from))
        .collect();
    assert!(
        after.status.success() && after.stdout.is_empty(),
        "the restored worktree is clean, but Git says {}; the index records {}",
        String::from_utf8_lossy(after.stdout.as_slice()),
        String::from_utf8_lossy(git_output(repo, recorded.as_slice()).stdout.as_slice())
    );
}

/// What:
///  A restore replaces a file an edit changed in a way the reset cannot see with the commit's bytes,
/// and its
///  proof holds.
/// Why:
///   The stage is a snapshot of the main checkout:
/// its index and its files are one instant,
/// and an edit made
///  afterwards of the same length,
/// in the same second as the entry that recorded the file,
/// keeps every field
///  `git read-tree --reset -u` compares,
/// so the reset leaves the edited bytes and `git diff-files --quiet`
///  reports nothing.
/// Native `git worktree add` writes the commit's bytes into an empty destination,
/// so a worktree
///  holding the edit is one a later commit would publish as a change nobody reviewed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stage = repo; hideAnEdit(stage, 'a.txt', 'first\n', 'other\n');
/// installIndex({ source, stage, admin, ownIndex: readFileSync(join(stage, '.git/index')) });
/// await restoreDirty({ source, target, stage, runGit, context, conversion });
/// expect(readFileSync(join(stage, 'a.txt'))).toBe('first\n');
/// ```
#[test]
fn a_restore_replaces_an_edit_that_keeps_its_files_size_and_times() {
    let root: PathBuf = fixture("restore-hidden-edit");
    let (repo, admin, seed, source, target) = hidden_edit_fixture(root.as_path());
    let held: std::fs::File =
        open_root(admin.as_path()).expect("the administrative directory opens");
    install_index(
        &source,
        repo.as_path(),
        &held,
        Some(seed.as_slice()),
        &crate::fixture_content_decision::content_environment(),
        None,
        &std::collections::HashSet::new(),
    )
    .expect("the index installs");
    drop(held);
    let mut steps = real_steps(root.as_path());
    let outcome: Result<(), String> = restore_dirty(
        &source,
        &target,
        repo.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    );
    assert_eq!(outcome, Ok(()), "the restore and its proof hold");
    assert_the_stage_holds_the_commits_bytes(repo.as_path(), admin.as_path());
    remove(root.as_path());
}

/// What:
///  A restore replaces a hidden edit even when the index recorded no nanoseconds,
/// which is how a Git built
///  without nanosecond support writes its entries.
/// Why:
///   The recorded stat data of such an entry equals a file of the same length,
/// inode and second even after an
///  edit,
/// because zero is written for the counts it cannot express and matches any present count;
/// the entry proves
///  nothing about the bytes,
/// so the walk must read the file instead of trusting the fields,
/// or the restore publishes
///  an edit the commit does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installIndex({ ...fixture, ownIndex: asAGitWithoutNanosecondsWroteIt(seed, 20) });
/// await restoreDirty({ ... });
/// expect(readFileSync(join(stage, 'a.txt'))).toEqual(COMMITTED);
/// ```
#[test]
fn a_restore_replaces_a_hidden_edit_when_the_index_recorded_no_nanoseconds() {
    let root: PathBuf = fixture("restore-hidden-edit-zero-nanos");
    let (repo, admin, seed, mut source, target) = hidden_edit_fixture(root.as_path());
    let without_nanoseconds: Vec<u8> = as_a_git_without_nanoseconds_wrote_it(seed.as_slice(), 20);
    source.clean.index_bytes = without_nanoseconds.clone();
    let held: std::fs::File =
        open_root(admin.as_path()).expect("the administrative directory opens");
    install_index(
        &source,
        repo.as_path(),
        &held,
        Some(without_nanoseconds.as_slice()),
        &crate::fixture_content_decision::content_environment(),
        None,
        &std::collections::HashSet::new(),
    )
    .expect("the index installs");
    drop(held);
    // The installed entry is the one the fixture wrote: the counts are zero, so the entry's stat data proves nothing about the bytes.
    let installed: Vec<u8> = std::fs::read(admin.join("index")).expect("the installed index");
    let recorded: Vec<StatEntry> =
        stored_stats(installed.as_slice(), 20).expect("the entries are plain");
    let edited: &StatEntry = recorded
        .iter()
        .find(|entry: &&StatEntry| {
            return entry.path == b"a.txt";
        })
        .expect("the entry of the edited file");
    assert_eq!(
        (edited.stored.ctime_nanos, edited.stored.mtime_nanos),
        (0, 0),
        "the installed index records no nanoseconds"
    );
    let mut steps = real_steps(root.as_path());
    let outcome: Result<(), String> = restore_dirty(
        &source,
        &target,
        repo.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    );
    assert_eq!(outcome, Ok(()), "the restore and its proof hold");
    assert_the_stage_holds_the_commits_bytes(repo.as_path(), admin.as_path());
    remove(root.as_path());
}

/// What:
///  The source and the target one restore of a copy made from the repository at
///  `repo` is told about,
///  with the new worktree's own administrative directory read from the pointer
///  Git wrote at `destination`,
///  and the stage's index copied there as the fill installs it.
/// Why:
///  A restore reads the new worktree's index,
///  its target commit and the configuration of the source,
///  and the controls that refuse a copy need only those three facts;
///  the stage is the repository itself,
///  which is what a snapshot of it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { source, target } = theSourceAndTargetOf({ root, repo, destination });
/// ```
fn the_source_and_target_of(
    (root, repo, destination): (&Path, &Path, &Path)
) -> (SnapshotSource, Target) {
    let admin: PathBuf = admin_of(destination);
    std::fs::copy(repo.join(".git/index"), admin.join("index")).expect("the stage's index");
    let commit: String = commit_of(repo, "HEAD");
    let elsewhere_git: PathBuf = elsewhere_git_dir(root);
    let elsewhere: PathBuf = elsewhere_git
        .parent()
        .expect("the Git directory's parent")
        .to_path_buf();
    let source = SnapshotSource {
        root: elsewhere,
        git_dir: elsewhere_git.clone(),
        common_dir: elsewhere_git,
        clean: crate::worktree_clean::CleanSource {
            head_commit: commit.clone(),
            head_tree: Vec::new(),
            tracked: 1,
            index_seconds: 0,
            hash_length: 20,
            attribute_blobs: Vec::new(),
            index_bytes: std::fs::read(repo.join(".git/index")).expect("the source's index"),
        },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo)
            .expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let target: Target = Target {
        admin,
        index: None,
        pointer: Vec::new(),
        commit,
        differs: false,
        line: String::new(),
    };
    return (source, target);
}

/// What:
///  A copy holding an untracked Git repository of its own ends the restore with
///  the proof's refusal,
///  after the clean kept the repository.
/// Why:
///  This is the failure the route's preflight declines before the claim:
///  `git clean --force -d` keeps a directory that holds a `.git` entry,
///  the proof lists it as still untracked,
///  and a slot that reached this far keeps its claim,
///  its clone and its origin snapshot,
///  because nothing of this release settles them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(restoreDirty(source, target, stage)).toContain('still untracked');
/// expect(existsSync('nested/.git')).toBe(true);
/// ```
#[test]
fn a_restore_refuses_a_copy_that_holds_an_untracked_repository() {
    let root: PathBuf = fixture("restore-untracked-repository");
    let repo: PathBuf = committed(root.as_path());
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(nested.as_path()).expect("the repository's directory");
    git(nested.as_path(), &["init", "--quiet"]);
    std::fs::write(nested.join("inside.txt"), b"inside\n").expect("the repository's own file");
    let destination: PathBuf = root.join("topic");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--no-checkout",
            "--quiet",
            destination.to_string_lossy().as_ref(),
            "HEAD",
        ],
    );
    let (source, target) =
        the_source_and_target_of((root.as_path(), repo.as_path(), destination.as_path()));
    let mut steps = real_steps(root.as_path());
    let refusal: String = restore_dirty(
        &source,
        &target,
        repo.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    )
    .expect_err("the proof names the repository as untracked");
    assert!(
        refusal.contains("nested") && refusal.contains("still untracked"),
        "{refusal}"
    );
    assert!(
        nested.join(".git").exists(),
        "the clean kept the repository, which is why the proof refused"
    );
    remove(root.as_path());
}

/// What:
///  A copy whose tracked file has become a directory ends the restore where the
///  file is removed for the reset,
///  and says what stands there.
/// Why:
///  The restore removes the files its reset cannot see as files or links,
///  and `unlinkat` without `AT_REMOVEDIR` refuses a directory,
///  so the copy is never proven and the slot keeps what it made.
///  Git's own checkout would simply write the commit's file there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(restoreDirty(source, target, stage)).toContain('a.txt');
/// ```
#[test]
fn a_restore_refuses_a_copy_whose_tracked_file_is_a_directory() {
    let root: PathBuf = fixture("restore-directory-at-a-file");
    let repo: PathBuf = committed(root.as_path());
    std::fs::remove_file(repo.join("a.txt")).expect("the tracked file goes");
    std::fs::create_dir(repo.join("a.txt")).expect("the directory takes its name");
    std::fs::write(repo.join("a.txt").join("inside.txt"), b"inside\n").expect("a file inside it");
    let destination: PathBuf = root.join("topic");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--no-checkout",
            "--quiet",
            destination.to_string_lossy().as_ref(),
            "HEAD",
        ],
    );
    let (source, target) =
        the_source_and_target_of((root.as_path(), repo.as_path(), destination.as_path()));
    let mut steps = real_steps(root.as_path());
    let refusal: String = restore_dirty(
        &source,
        &target,
        repo.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    )
    .expect_err("the removal cannot unlink a directory");
    assert!(
        refusal.contains("a.txt") && refusal.contains("directory"),
        "{refusal}"
    );
    remove(root.as_path());
}

/// What:
///  A copy holding an untracked path the account cannot remove ends the restore
///  at its clean,
///  before the reset and the proof.
/// Why:
///  `git clean --force -d` fails to unlink a file inside a directory it cannot
///  write,
///  and this is the class of failure the route's preflight cannot rule out by
///  reading the source:
///  the immutable bit,
///  a full disk and a filesystem that stops answering end a
///  step the same way.
///  The settle of plan step 14 is its answer,
///  and the route's decline is for the
///  shapes the proof and the removal refuse from the source's own bytes
///  (`add_source_the_restore_cannot_prove.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync('d', 0o500); expect(restoreDirty(source, target, stage)).toContain('clean --force -d');
/// ```
#[test]
fn a_restore_ends_at_the_clean_it_cannot_finish() {
    let root: PathBuf = fixture("restore-unwritable-directory");
    let repo: PathBuf = committed(root.as_path());
    let held: PathBuf = repo.join("d");
    std::fs::create_dir(held.as_path()).expect("the untracked directory");
    std::fs::write(held.join("held.txt"), b"held\n").expect("a file inside it");
    std::fs::set_permissions(
        held.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o500),
    )
    .expect("the directory loses its write permission");
    let destination: PathBuf = root.join("topic");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--no-checkout",
            "--quiet",
            destination.to_string_lossy().as_ref(),
            "HEAD",
        ],
    );
    let (source, target) =
        the_source_and_target_of((root.as_path(), repo.as_path(), destination.as_path()));
    let mut steps = real_steps(root.as_path());
    let refusal: String = restore_dirty(
        &source,
        &target,
        repo.as_path(),
        &mut steps,
        &git_context(),
        &crate::fixture_content_decision::content_environment(),
    )
    .expect_err("the clean cannot unlink the file");
    std::fs::set_permissions(
        held.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("the fixture gives the permission back");
    assert!(
        refusal.contains("clean --force -d") && refusal.contains("ended with"),
        "{refusal}"
    );
    remove(root.as_path());
}

//! Controls for the restore of a snapshot of a main checkout with changes:
//! the proof accepts what Git's checkout leaves and
//! names what it does not,
//! the Git runs are exactly the two the design lists,
//! the ignored files of the source survive a commit that ignores nothing,
//! an edit the reset cannot see is replaced by the commit's bytes,
//! and a restore never runs in the source.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git, git_context, git_output, remove, repository};
use crate::worktree_copy::{GitStatus, GitStep};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_ladder_test_support::hide_an_edit;
use crate::worktree_snapshot_index_step::install_index;
use std::path::{Path, PathBuf};

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
fn commit_of(repo: &Path, revision: &str) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", revision]).stdout).trim().to_string();
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
fn copy_files(from: &Path, to: &Path) {
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
    let target: &str = pointer.strip_prefix("gitdir: ").expect("a gitdir pointer").trim();
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
        let GitStep::Other { arguments, environment } = step else { panic!("only the ladder's own steps are expected") };
        let status: std::process::ExitStatus = std::process::Command::new(REAL_GIT)
            .args(*arguments)
            .envs(environment.iter().cloned())
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", home.as_path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", home.join("no-global-config").as_path())
            .status()
            .expect("real Git starts");
        return Ok(GitStatus { code: status.code() });
    };
}

/// A target for the commit `HEAD` names in `repo`,
/// whose Git directory is its own.
fn target_of(repo: &Path) -> Target {
    let commit: String = String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout).trim().to_string();
    return Target { admin: repo.join(".git"), pointer: Vec::new(), commit, differs: false, line: String::new() };
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
    assert_eq!(prove_restored(&git_context(), &target, repo.as_path(), &[]), Ok(()), "positive control");
    std::fs::write(repo.join("untracked.txt"), b"u").expect("untracked");
    let untracked: String = prove_restored(&git_context(), &target, repo.as_path(), &[]).expect_err("an untracked file");
    assert!(untracked.contains("untracked.txt") && untracked.contains("still untracked"), "{untracked}");
    assert_eq!(prove_restored(&git_context(), &target, repo.as_path(), &[b"untracked.txt".to_vec()]), Ok(()), "the same file is carried");
    std::fs::create_dir(repo.join("cache")).expect("ignored directory");
    std::fs::write(repo.join("cache/data.bin"), b"d").expect("ignored file");
    assert!(prove_restored(&git_context(), &target, repo.as_path(), &[b"untracked.txt".to_vec()]).expect_err("a file inside an ignored directory").contains("cache"), "a carried directory's name is not the name of a file in it");
    assert_eq!(prove_restored(&git_context(), &target, repo.as_path(), &[b"untracked.txt".to_vec(), b"cache/".to_vec()]), Ok(()), "a carried directory covers what it holds");
    std::fs::remove_dir_all(repo.join("cache")).expect("remove");
    std::fs::remove_file(repo.join("untracked.txt")).expect("remove");
    git(repo.as_path(), &["rm", "--cached", "--quiet", "a.txt"]);
    assert!(prove_restored(&git_context(), &target, repo.as_path(), &[]).expect_err("a staged deletion").contains("index"));
    git(repo.as_path(), &["add", "a.txt"]);
    std::fs::write(repo.join("a.txt"), b"changed\n").expect("edit");
    assert!(prove_restored(&git_context(), &target, repo.as_path(), &[]).expect_err("an edit").contains("differs"));
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
        let GitStep::Other { arguments, environment } = step else { panic!("only the ladder's own steps are expected") };
        seen.push(arguments.iter().map(|word| { return word.to_string_lossy().into_owned(); }).collect());
        environments.push(environment.iter().map(|(name, value)| { return (name.to_string_lossy().into_owned(), value.to_string_lossy().into_owned()); }).collect());
        return Ok(GitStatus { code: Some(0) });
    };
    let elsewhere_git: PathBuf = elsewhere_git_dir(root.as_path());
    let other_root: PathBuf = elsewhere_git.parent().expect("the Git directory's parent").to_path_buf();
    let source = SnapshotSource {
        root: other_root.clone(),
        git_dir: elsewhere_git.clone(),
        common_dir: elsewhere_git.clone(),
        clean: crate::worktree_clean::CleanSource { head_commit: target.commit.clone(), head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let (recording, log) = crate::recording_git::recording_context(root.as_path());
    restore_dirty(&source, &target, stage.as_path(), &mut runner, &recording, &crate::fixture_content_decision::content_environment()).unwrap_or_else(|reason| {
        let differences: std::process::Output = git_output(stage.as_path(), &["diff-files"]);
        panic!("the proof holds for a worktree that equals its commit: {reason}; Git says {} and the file holds {:?}", String::from_utf8_lossy(differences.stdout.as_slice()), std::fs::read(stage.join("a.txt")))
    });
    let words = |step: &Vec<String>| -> Vec<String> { return step[LEADING_WORDS..].to_vec(); };
    let expected: Vec<Vec<String>> = vec![["read-tree", "--reset", "-u", target.commit.as_str()].map(String::from).to_vec()];
    assert_eq!(seen.iter().map(words).collect::<Vec<Vec<String>>>(), expected, "exactly this step goes through the runner");
    let cleans: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.split(' ').any(|word| { return word == "clean"; }); }).collect();
    assert_eq!(cleans.len(), 1, "one captured clean: {cleans:?}");
    assert!(cleans[0].ends_with("clean --force -d"), "{cleans:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    for (arguments, environment) in seen.iter().zip(&environments) {
        assert_eq!(&arguments[..LEADING_WORDS], ["-C", stage.to_string_lossy().as_ref()]);
        assert!(environment.contains(&(String::from("GIT_DIR"), target.admin.to_string_lossy().into_owned())), "{environment:?}");
        assert!(environment.contains(&(String::from("GIT_WORK_TREE"), stage.to_string_lossy().into_owned())), "{environment:?}");
    }
    remove(root.as_path());
}

/// A restore that would run in the source's own directory or Git directory runs nothing.
#[test]
fn a_restore_never_runs_in_the_source() {
    let root: PathBuf = crate::test_support::fixture("restore-guard");
    let repo: PathBuf = committed(root.as_path());
    let target: Target = target_of(repo.as_path());
    let mut runner = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> { panic!("no Git run is expected") };
    let source = |source_root: PathBuf, git_dir: PathBuf| -> SnapshotSource {
        return SnapshotSource {
            root: source_root,
            git_dir: git_dir.clone(),
            common_dir: git_dir,
            clean: crate::worktree_clean::CleanSource { head_commit: target.commit.clone(), head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
            nested: Vec::new(),
            config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
            trash_ignored: false,
            dirty: None,
        };
    };
    let in_source = restore_dirty(&source(repo.clone(), repo.join(".git")), &target, repo.as_path(), &mut runner, &git_context(), &crate::fixture_content_decision::content_environment()).expect_err("the stage is the source");
    assert!(in_source.contains("source's own directory"), "{in_source}");
    let elsewhere: PathBuf = root.join("elsewhere");
    let in_git_dir = restore_dirty(&source(elsewhere.clone(), repo.join(".git")), &target, elsewhere.as_path(), &mut runner, &git_context(), &crate::fixture_content_decision::content_environment()).expect_err("the Git directory is the source's");
    assert!(in_git_dir.contains("source's own directory"), "{in_git_dir}");
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
    let other_root: PathBuf = elsewhere_git.parent().expect("the Git directory's parent").to_path_buf();
    let source = SnapshotSource {
        root: other_root.clone(),
        git_dir: elsewhere_git.clone(),
        common_dir: elsewhere_git.clone(),
        clean: crate::worktree_clean::CleanSource { head_commit: target.commit.clone(), head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let mut failing = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> { return Ok(GitStatus { code: Some(128) }); };
    assert!(restore_dirty(&source, &target, repo.as_path(), &mut failing, &git_context(), &crate::fixture_content_decision::content_environment()).expect_err("the reset failed").contains("read-tree"));
    let mut succeeding = |_step: &GitStep<'_>| -> Result<GitStatus, WorktreeCopyError> { return Ok(GitStatus { code: Some(0) }); };
    std::fs::write(repo.join("left.txt"), b"left").expect("a file the clean left behind");
    // A Git whose `clean` does nothing, so the file stays and the proof is what finds it.
    let idle_clean: PathBuf = root.join("idle-clean-git");
    let script: String = format!("#!/bin/sh\ncase \" $* \" in *\" clean \"*) exit 0;; esac\nexec {} \"$@\"\n", crate::test_support::REAL_GIT);
    crate::test_support::executable(idle_clean.as_path(), script.as_bytes());
    let mut leaving: GitContext = git_context();
    leaving.real_git = idle_clean;
    assert!(restore_dirty(&source, &target, repo.as_path(), &mut succeeding, &leaving, &crate::fixture_content_decision::content_environment()).expect_err("the proof fails").contains("left.txt"));
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
    git(repo.as_path(), &["worktree", "add", "--no-checkout", "--quiet", destination.to_string_lossy().as_ref(), second.as_str()]);
    let admin: PathBuf = admin_of(destination.as_path());
    std::fs::copy(repo.join(".git/index"), admin.join("index")).expect("the stage's index");
    let stage: PathBuf = root.join("stage");
    copy_files(repo.as_path(), stage.as_path());
    let target: Target = Target { admin: admin.clone(), pointer: Vec::new(), commit: second, differs: false, line: String::new() };
    let source = SnapshotSource {
        root: repo.clone(),
        git_dir: repo.join(".git"),
        common_dir: repo.join(".git"),
        clean: crate::worktree_clean::CleanSource { head_commit: first, head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let mut steps = real_steps(root.as_path());
    let outcome: Result<(), String> = restore_dirty(&source, &target, stage.as_path(), &mut steps, &git_context(), &crate::fixture_content_decision::content_environment());
    assert_eq!(outcome, Ok(()), "the restore and its proof hold");
    assert_eq!(std::fs::read(stage.join("cache/data.bin")).expect("the ignored file is still there"), b"ignored bytes\n");
    assert_eq!(std::fs::read(stage.join("tracked.txt")).expect("the tracked file is the commit's"), b"first\n");
    assert!(!stage.join("untracked.txt").exists(), "an untracked file that no ignore rule covers goes");
    remove(root.as_path());
}

/// The note counts the paths in the singular and the plural.
#[test]
fn the_note_counts_paths() {
    let one = ChangedPaths { count: 1, first: Some(String::from("a")), attributes_changed: false };
    let many = ChangedPaths { count: 3, first: Some(String::from("a")), attributes_changed: false };
    assert!(left_out_note(&one).contains(format!("({} path)", one.count).as_str()), "{}", left_out_note(&one));
    assert!(left_out_note(&many).contains(format!("({} paths)", many.count).as_str()), "{}", left_out_note(&many));
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
    assert_eq!(prove_restored(&git_context(), &target, repo.as_path(), carried.as_slice()), Ok(()), "Git reports cache/ and everything under it is carried");
    std::fs::write(repo.join("cache/other.txt"), b"o").expect("a file the restore did not carry");
    let refused: String = prove_restored(&git_context(), &target, repo.as_path(), carried.as_slice()).expect_err("a directory holding a file that was not carried");
    assert!(refused.contains("cache/"), "the refusal names the directory Git reported: {refused}");
    remove(root.as_path());
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
    let original: &[u8] = b"first\n";
    let edited: &[u8] = b"other\n";
    let repo: PathBuf = repository(root.as_path(), "stage");
    std::fs::write(repo.join("a.txt"), original).expect("the tracked file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let admin: PathBuf = admin_of(root.join("topic").as_path());
    // The edit is made after the index recorded the file, and Git's own check cannot see it.
    assert!(hide_an_edit(repo.as_path(), "a.txt", original, edited), "the fixture hides its edit from Git's stat check");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the edit"), edited);
    assert!(git_output(repo.as_path(), &["diff-files", "--quiet"]).status.success(), "the fixture's edit is hidden from Git's own check");
    let seed: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("the index the snapshot carries");
    let head: String = commit_of(repo.as_path(), "HEAD");
    let elsewhere_git: PathBuf = elsewhere_git_dir(root.as_path());
    let elsewhere: PathBuf = elsewhere_git.parent().expect("the Git directory's parent").to_path_buf();
    let source = SnapshotSource {
        root: elsewhere.clone(),
        git_dir: elsewhere_git.clone(),
        common_dir: elsewhere_git.clone(),
        clean: crate::worktree_clean::CleanSource { head_commit: head.clone(), head_tree: Vec::new(), tracked: 1, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: seed.clone() },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let target: Target = Target { admin: admin.clone(), pointer: Vec::new(), commit: head, differs: false, line: String::new() };
    let held: std::fs::File = open_root(admin.as_path()).expect("the administrative directory opens");
    install_index(&source, repo.as_path(), &held, Some(seed.as_slice()), &crate::fixture_content_decision::content_environment(), None).expect("the index installs");
    drop(held);
    let mut steps = real_steps(root.as_path());
    let outcome: Result<(), String> = restore_dirty(&source, &target, repo.as_path(), &mut steps, &git_context(), &crate::fixture_content_decision::content_environment());
    assert_eq!(outcome, Ok(()), "the restore and its proof hold");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the restored file"), original, "the copy holds the commit's bytes, as Git's own checkout makes them");
    // The stage carries the new worktree's Git pointer in the product, so Git reads the installed index there and not
    //  the source's own.
    let in_the_stage: [String; 4] = ["--git-dir", admin.to_string_lossy().as_ref(), "--work-tree", repo.to_string_lossy().as_ref()].map(String::from);
    let differences: Vec<String> = in_the_stage.iter().cloned().chain(["diff-files"].map(String::from)).collect();
    let after: std::process::Output = git_output(repo.as_path(), differences.as_slice());
    let recorded: Vec<String> = in_the_stage.iter().cloned().chain(["ls-files", "--debug"].map(String::from)).collect();
    assert!(
        after.status.success() && after.stdout.is_empty(),
        "the restored worktree is clean, but Git says {}; the index records {}",
        String::from_utf8_lossy(after.stdout.as_slice()),
        String::from_utf8_lossy(git_output(repo.as_path(), recorded.as_slice()).stdout.as_slice())
    );
    remove(root.as_path());
}

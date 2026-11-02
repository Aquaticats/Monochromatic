//! Controls for the reading of Git's `--no-checkout` result and the tree-difference step: the line, the commit, whether the
//!  trees differ, and each refusal.

use super::*;
use crate::config_snapshot::ConfigEntry;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, tracked};
use crate::worktree_snapshot_source::{SnapshotSource, common_source};

/// A clean ordinary repository with two commits and a `--no-checkout` worktree of `main` or of `HEAD~1`.
fn fixture_with_worktree(root: &Path, older: bool) -> (PathBuf, PathBuf, SnapshotSource) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    commit_all(repo.as_path(), "first");
    tracked(repo.as_path(), "shared.txt", b"second, longer\n");
    commit_all(repo.as_path(), "second");
    let source: SnapshotSource = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let mut words: Vec<&str> = vec!["worktree", "add", "--quiet", "--no-checkout", "--detach", "../topic"];
    if older {
        words.push("HEAD~1");
    }
    git(repo.as_path(), words.as_slice());
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    return (repo, destination, source);
}

/// A worktree at the source's own commit differs in nothing and gets Git's line, or none for `--quiet`.
#[test]
fn the_source_commit_is_the_target_and_the_line_is_gits() {
    let root: PathBuf = fixture("target-same");
    let (repo, destination, source) = fixture_with_worktree(root.as_path(), false);
    let mut plan = plan_to(destination.clone());
    let target: Target = prepare_target(&source, &plan, &git_context()).expect("a target");
    assert!(!target.differs);
    assert_eq!(target.commit, git_text(repo.as_path(), &["rev-parse", "HEAD"]));
    assert_eq!(target.line, format!("{}\n", git_text(repo.as_path(), &["log", "-1", "--format=HEAD is now at %h %s"])));
    assert_eq!(target.admin, repo.join(".git/worktrees/topic"));
    assert_eq!(target.pointer, std::fs::read(destination.join(".git")).expect("pointer"));
    plan.quiet = true;
    assert_eq!(prepare_target(&source, &plan, &git_context()).expect("a quiet target").line, "");
    remove(root.as_path());
}

/// An older commit differs, and the step that applies the difference asks Git for `read-tree -m -u` between the two commits
///  in the work tree with Git's own git directory and work tree variables.
#[test]
fn an_older_commit_differs_and_the_step_names_both_commits() {
    let root: PathBuf = fixture("target-older");
    let (_repo, destination, source) = fixture_with_worktree(root.as_path(), true);
    let target: Target = prepare_target(&source, &plan_to(destination.clone()), &git_context()).expect("a target");
    assert!(target.differs);
    let seen: std::cell::RefCell<Vec<(Vec<String>, Vec<(String, String)>)>> = std::cell::RefCell::new(Vec::new());
    let mut runner = |_token: Option<&str>, step: &GitStep<'_>| -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> {
        let GitStep::Other { arguments, environment } = step else { panic!("an own step is expected") };
        seen.borrow_mut().push((
            arguments.iter().map(|word: &OsString| { return word.to_string_lossy().into_owned(); }).collect(),
            environment.iter().map(|(name, value): &(OsString, OsString)| { return (name.to_string_lossy().into_owned(), value.to_string_lossy().into_owned()); }).collect(),
        ));
        return Ok(GitStatus { code: Some(0) });
    };
    apply_difference(&source, &target, destination.as_path(), None, &mut runner).expect("applied");
    let (arguments, environment) = seen.borrow()[0].clone();
    assert_eq!(&arguments[2..], ["read-tree", "-m", "-u", source.clean.head_commit.as_str(), target.commit.as_str()]);
    assert_eq!(environment, vec![(String::from("GIT_DIR"), target.admin.display().to_string()), (String::from("GIT_WORK_TREE"), destination.display().to_string())]);
    let mut failing = |_token: Option<&str>, _step: &GitStep<'_>| -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> { return Ok(GitStatus { code: Some(1) }); };
    assert!(apply_difference(&source, &target, destination.as_path(), None, &mut failing).expect_err("a failing step").contains("read-tree"));
    remove(root.as_path());
}

/// Equal trees need no step.
#[test]
fn equal_trees_need_no_step() {
    let root: PathBuf = fixture("target-no-step");
    let (_repo, destination, source) = fixture_with_worktree(root.as_path(), false);
    let target: Target = prepare_target(&source, &plan_to(destination.clone()), &git_context()).expect("a target");
    let mut never = |_token: Option<&str>, _step: &GitStep<'_>| -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> { panic!("no step is expected"); };
    apply_difference(&source, &target, destination.as_path(), None, &mut never).expect("nothing to apply");
    remove(root.as_path());
}

/// A stray file in the destination, differing attributes and a subject that needs Git each end the reading with a reason.
#[test]
fn each_refusal_names_its_reason() {
    let root: PathBuf = fixture("target-refused");
    let (repo, destination, source) = fixture_with_worktree(root.as_path(), false);
    std::fs::write(destination.join("stray"), b"x").expect("stray");
    assert!(prepare_target(&source, &plan_to(destination.clone()), &git_context()).expect_err("stray").contains("more than Git's .git file"));
    std::fs::remove_file(destination.join("stray")).expect("remove");
    assert!(prepare_target(&source, &plan_to(destination.clone()), &git_context()).is_ok(), "positive control");
    // A message with a carriage return needs Git to print the subject.
    git(repo.as_path(), &["worktree", "remove", "--force", "../topic"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=a\rb"]);
    let carriage = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "--detach", "../topic"]);
    assert!(prepare_target(&carriage, &plan_to(destination.clone()), &git_context()).expect_err("carriage return").contains("subject"));
    // Attributes that differ between the source's tree and the new HEAD.
    git(repo.as_path(), &["worktree", "remove", "--force", "../topic"]);
    tracked(repo.as_path(), ".gitattributes", b"*.txt text\n");
    commit_all(repo.as_path(), "attributes");
    let with_attributes = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "--detach", "../topic", "HEAD~1"]);
    assert!(prepare_target(&with_attributes, &plan_to(destination), &git_context()).expect_err("attributes").contains(".gitattributes"));
    remove(root.as_path());
}

/// A repository whose local configuration includes `topic.cfg` on the branch `topic` only, with the filter that file sets.
fn conditional_filter_fixture(root: &Path, also_in_source: bool) -> (PathBuf, PathBuf, SnapshotSource) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    commit_all(repo.as_path(), "first");
    std::fs::write(repo.join(".git/topic.cfg"), "[filter \"demo\"]\n\tclean = cat\n\tsmudge = cat\n").expect("the included file");
    let mut config: String = std::fs::read_to_string(repo.join(".git/config")).expect("config");
    config.push_str("[includeIf \"onbranch:topic\"]\n\tpath = topic.cfg\n");
    if also_in_source {
        config.push_str("[filter \"demo\"]\n\tclean = cat\n\tsmudge = cat\n");
    }
    std::fs::write(repo.join(".git/config"), config).expect("config written");
    let source: SnapshotSource = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    return (repo, destination, source);
}

/// A filter that only the new branch's conditional include sets would convert files differently from the source's bytes: the
///  reading of the target refuses, naming the setting; the same filter in both is the same conversion.
#[test]
fn a_filter_only_the_new_branch_sets_ends_the_reading_and_a_shared_one_does_not() {
    let root: PathBuf = fixture("target-conditional-filter");
    let (_repo, destination, source) = conditional_filter_fixture(root.as_path(), false);
    let reason: String = prepare_target(&source, &plan_to(destination), &git_context()).expect_err("different filters");
    assert!(reason.contains("filter.demo.clean") && reason.contains("differs"), "{reason}");
    remove(root.as_path());
    let shared: PathBuf = fixture("target-shared-filter");
    let (_same_repo, same_destination, same_source) = conditional_filter_fixture(shared.as_path(), true);
    assert!(prepare_target(&same_source, &plan_to(same_destination), &git_context()).is_ok(), "the same conversion in both worktrees");
    remove(shared.as_path());
}

/// Settings that are not about conversion may differ: the branch the new worktree gets its own section for.
#[test]
fn a_setting_that_is_not_a_conversion_may_differ() {
    let entry = |key: &str, value: &str| -> ConfigEntry { return ConfigEntry { scope: String::from("local"), key: String::from(key), value: Some(value.as_bytes().to_vec()) }; };
    let source: Vec<ConfigEntry> = vec![entry("core.autocrlf", "input"), entry("user.name", "a")];
    let target: Vec<ConfigEntry> = vec![entry("core.autocrlf", "input"), entry("user.name", "b"), entry("branch.topic.remote", "origin")];
    assert_eq!(conversion_agrees(source.as_slice(), target.as_slice()), Ok(()));
    for changed in [entry("core.autocrlf", "true"), entry("core.eol", "crlf"), entry("filter.lfs.clean", "x"), entry("extensions.worktreeconfig", "true")] {
        let mut other: Vec<ConfigEntry> = target.clone();
        other.push(changed.clone());
        assert!(conversion_agrees(source.as_slice(), other.as_slice()).is_err(), "{}", changed.key);
    }
}

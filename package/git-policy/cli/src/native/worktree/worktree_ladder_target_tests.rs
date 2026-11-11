//! Controls for the reading of Git's `--no-checkout` result and the tree-difference step:
//! the line,
//! the commit,
//! whether the
//!  trees differ,
//! and each refusal.

use super::*;
use crate::config_snapshot::ConfigEntry;
use crate::worktree_add_binding::{Bound, bind_registration};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, tracked};
use crate::path_bytes::path_bytes_of;
use crate::worktree_snapshot_source::{SnapshotSource, common_source};

/// Bind the add and read the target,
/// as the ladder does.
fn prepare(source: &SnapshotSource, plan: &AddPlan) -> Result<Target, String> {
    let bound: Bound = bind_registration(source.common_dir.as_path(), plan.destination.as_path())?;
    return prepare_target(source, plan, &git_context(), &bound);
}

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

/// A worktree at the source's own commit differs in nothing and gets Git's line,
/// or none for `--quiet`.
#[test]
fn the_source_commit_is_the_target_and_the_line_is_gits() {
    let root: PathBuf = fixture("target-same");
    let (repo, destination, source) = fixture_with_worktree(root.as_path(), false);
    let mut plan = plan_to(destination.clone());
    let target: Target = prepare(&source, &plan).expect("a target");
    assert!(!target.differs);
    assert_eq!(target.commit, git_text(repo.as_path(), &["rev-parse", "HEAD"]));
    assert_eq!(target.line, format!("{}\n", git_text(repo.as_path(), &["log", "-1", "--format=HEAD is now at %h %s"])));
    assert_eq!(target.admin, repo.join(".git/worktrees/topic"));
    assert_eq!(target.pointer, std::fs::read(destination.join(".git")).expect("pointer"));
    plan.quiet = true;
    assert_eq!(prepare(&source, &plan).expect("a quiet target").line, "");
    remove(root.as_path());
}

/// An older commit differs,
/// and the step that applies the difference asks Git for `read-tree -m -u` between the two commits
///  in the work tree with Git's own git directory and work tree variables.
#[test]
fn an_older_commit_differs_and_the_step_names_both_commits() {
    let root: PathBuf = fixture("target-older");
    let (_repo, destination, source) = fixture_with_worktree(root.as_path(), true);
    let target: Target = prepare(&source, &plan_to(destination.clone())).expect("a target");
    assert!(target.differs);
    let seen: std::cell::RefCell<Vec<(Vec<String>, Vec<(String, String)>)>> = std::cell::RefCell::new(Vec::new());
    let mut runner = |step: &GitStep<'_>| -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> {
        let GitStep::Other { arguments, environment } = step else { panic!("an own step is expected") };
        seen.borrow_mut().push((
            arguments.iter().map(|word: &OsString| { return word.to_string_lossy().into_owned(); }).collect(),
            environment.iter().map(|(name, value): &(OsString, OsString)| { return (name.to_string_lossy().into_owned(), value.to_string_lossy().into_owned()); }).collect(),
        ));
        return Ok(GitStatus { code: Some(0) });
    };
    apply_difference(&source, &target, destination.as_path(), &mut runner).expect("applied");
    let (arguments, environment) = seen.borrow()[0].clone();
    assert_eq!(&arguments[2..], ["read-tree", "-m", "-u", source.clean.head_commit.as_str(), target.commit.as_str()]);
    assert_eq!(environment, vec![(String::from("GIT_DIR"), target.admin.display().to_string()), (String::from("GIT_WORK_TREE"), destination.display().to_string())]);
    let mut failing = |_step: &GitStep<'_>| -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> { return Ok(GitStatus { code: Some(1) }); };
    assert!(apply_difference(&source, &target, destination.as_path(), &mut failing).expect_err("a failing step").contains("read-tree"));
    remove(root.as_path());
}

/// Equal trees need no step.
#[test]
fn equal_trees_need_no_step() {
    let root: PathBuf = fixture("target-no-step");
    let (_repo, destination, source) = fixture_with_worktree(root.as_path(), false);
    let target: Target = prepare(&source, &plan_to(destination.clone())).expect("a target");
    let mut never = |_step: &GitStep<'_>| -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> { panic!("no step is expected"); };
    apply_difference(&source, &target, destination.as_path(), &mut never).expect("nothing to apply");
    remove(root.as_path());
}

/// A stray file in the destination,
/// differing attributes and a subject that needs Git each end the reading with a reason.
#[test]
fn each_refusal_names_its_reason() {
    let root: PathBuf = fixture("target-refused");
    let (repo, destination, source) = fixture_with_worktree(root.as_path(), false);
    std::fs::write(destination.join("stray"), b"x").expect("stray");
    assert!(prepare(&source, &plan_to(destination.clone())).expect_err("stray").contains("more than Git's .git file"));
    std::fs::remove_file(destination.join("stray")).expect("remove");
    assert!(prepare(&source, &plan_to(destination.clone())).is_ok(), "positive control");
    // A message with a carriage return needs Git to print the subject.
    git(repo.as_path(), &["worktree", "remove", "--force", "../topic"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=a\rb"]);
    let carriage = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "--detach", "../topic"]);
    assert!(prepare(&carriage, &plan_to(destination.clone())).expect_err("carriage return").contains("subject"));
    // Attributes that differ between the source's tree and the new HEAD.
    git(repo.as_path(), &["worktree", "remove", "--force", "../topic"]);
    tracked(repo.as_path(), ".gitattributes", b"*.txt text\n");
    commit_all(repo.as_path(), "attributes");
    let with_attributes = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "--detach", "../topic", "HEAD~1"]);
    assert!(prepare(&with_attributes, &plan_to(destination)).expect_err("attributes").contains(".gitattributes"));
    remove(root.as_path());
}

/// A repository whose local configuration includes `topic.cfg` on the branch `topic` only,
/// with the filter that file sets.
fn conditional_filter_fixture(root: &Path) -> (PathBuf, PathBuf, SnapshotSource) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    commit_all(repo.as_path(), "first");
    std::fs::write(repo.join(".git/topic.cfg"), "[filter \"demo\"]\n\tclean = cat\n\tsmudge = cat\n").expect("the included file");
    let mut config: String = std::fs::read_to_string(repo.join(".git/config")).expect("config");
    config.push_str("[includeIf \"onbranch:topic\"]\n\tpath = topic.cfg\n");
    std::fs::write(repo.join(".git/config"), config).expect("config written");
    let source: SnapshotSource = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    return (repo, destination, source);
}

/// A filter that only the new branch's conditional include sets would convert files differently from the source's bytes:
/// the
///  reading of the target refuses,
/// naming the setting;
/// the same filter in both is the same conversion.
#[test]
fn a_filter_only_the_new_branch_sets_ends_the_reading_and_a_shared_one_does_not() {
    let root: PathBuf = fixture("target-conditional-filter");
    let (_repo, destination, source) = conditional_filter_fixture(root.as_path());
    let reason: String = prepare(&source, &plan_to(destination)).expect_err("different filters");
    assert!(reason.contains("filter.demo.clean") && reason.contains("differs"), "{reason}");
    remove(root.as_path());
    let shared: PathBuf = fixture("target-shared-filter");
    let (same_destination, same_source) = configured_fixture(shared.as_path(), "", "[filter \"demo\"]\n\tclean = cat\n\tsmudge = cat\n");
    assert!(prepare(&same_source, &plan_to(same_destination)).is_ok(), "the same conversion in both worktrees");
    remove(shared.as_path());
}

/// Settings that are not about conversion may differ:
/// the branch the new worktree gets its own section for.
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

/// A repository whose local configuration is `config`,
/// with the tracked attributes `attributes`,
/// and a `--no-checkout`
///  worktree of its branch.
fn configured_fixture(root: &Path, attributes: &str, config: &str) -> (PathBuf, SnapshotSource) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    if !attributes.is_empty() {
        tracked(repo.as_path(), ".gitattributes", attributes.as_bytes());
    }
    commit_all(repo.as_path(), "first");
    let mut text: String = std::fs::read_to_string(repo.join(".git/config")).expect("config");
    text.push_str(config);
    std::fs::write(repo.join(".git/config"), text).expect("config written");
    let source: SnapshotSource = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "--detach", "../topic"]);
    return (std::fs::canonicalize(root.join("topic")).expect("destination"), source);
}

/// The last value of a key wins,
/// so the same values in another order are another configuration:
/// a set comparison called the
///  two equal,
/// the sequence does not.
#[test]
fn the_same_filter_values_in_another_order_are_another_configuration() {
    let root: PathBuf = fixture("target-filter-order");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    commit_all(repo.as_path(), "first");
    std::fs::write(repo.join(".git/swap.cfg"), "[filter \"demo\"]\n\tclean = tr a-z A-Z\n").expect("the included file");
    let mut config: String = std::fs::read_to_string(repo.join(".git/config")).expect("config");
    config.push_str("[includeIf \"onbranch:main\"]\n\tpath = swap.cfg\n[filter \"demo\"]\n\tclean = cat\n[includeIf \"onbranch:topic\"]\n\tpath = swap.cfg\n");
    std::fs::write(repo.join(".git/config"), config).expect("config written");
    let source: SnapshotSource = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let reason: String = prepare(&source, &plan_to(destination)).expect_err("another order");
    assert!(reason.contains("filter.demo.clean") && reason.contains("another order"), "{reason}");
    remove(root.as_path());
}

/// A filter that an attribute selects is refused unless it is the standard Git LFS filter,
/// whatever the equal settings say.
#[test]
fn a_filter_an_attribute_selects_is_refused_except_the_standard_lfs_filter() {
    let demo: &str = "[filter \"demo\"]\n\tclean = cat\n\tsmudge = cat\n";
    let root: PathBuf = fixture("target-filter-used");
    let (destination, source) = configured_fixture(root.as_path(), "*.txt filter=demo\n", demo);
    let reason: String = prepare(&source, &plan_to(destination)).expect_err("a filter in use");
    assert!(reason.contains("filter demo"), "{reason}");
    remove(root.as_path());
    // The same filter configured and not selected by any attribute does not matter.
    let unused: PathBuf = fixture("target-filter-unused");
    let (idle_destination, idle_source) = configured_fixture(unused.as_path(), "*.txt text\n", demo);
    assert!(prepare(&idle_source, &plan_to(idle_destination)).is_ok(), "a configured filter that nothing selects");
    remove(unused.as_path());
    let lfs: &str = "[filter \"lfs\"]\n\tclean = git-lfs clean -- %f\n\tsmudge = git-lfs smudge -- %f\n\tprocess = git-lfs filter-process\n\trequired = true\n";
    let standard: PathBuf = fixture("target-filter-lfs");
    let (lfs_destination, lfs_source) = configured_fixture(standard.as_path(), "*.bin filter=lfs diff=lfs merge=lfs -text\n", lfs);
    assert!(prepare(&lfs_source, &plan_to(lfs_destination)).is_ok(), "the standard Git LFS definition");
    remove(standard.as_path());
    let changed: PathBuf = fixture("target-filter-lfs-changed");
    let (other_destination, other_source) = configured_fixture(changed.as_path(), "*.bin filter=lfs\n", lfs.replace("filter-process", "filter-process --skip").as_str());
    assert!(prepare(&other_source, &plan_to(other_destination)).expect_err("not the standard definition").contains("standard Git LFS"));
    remove(changed.as_path());
}

/// An attributes file named by a relative path means another file in another worktree and is refused;
/// an absolute path is the
///  same file in both,
/// and a filter its text selects is refused like a tracked one.
#[test]
fn an_attributes_file_by_a_relative_path_is_refused() {
    let root: PathBuf = fixture("target-attributes-file");
    let (destination, source) = configured_fixture(root.as_path(), "", "[core]\n\tattributesFile = ../attributes\n");
    assert!(prepare(&source, &plan_to(destination)).expect_err("relative").contains("not an absolute path"));
    remove(root.as_path());
    let absolute: PathBuf = fixture("target-attributes-absolute");
    let file: PathBuf = std::fs::canonicalize(absolute.as_path()).expect("canonical").join("attributes");
    std::fs::write(file.as_path(), "*.txt filter=demo\n").expect("attributes file");
    let (file_destination, file_source) = configured_fixture(absolute.as_path(), "", format!("[core]\n\tattributesFile = {}\n", file.display()).as_str());
    assert!(prepare(&file_source, &plan_to(file_destination)).expect_err("a filter in an absolute file").contains("filter demo"));
    remove(absolute.as_path());
}

/// The bytes of an attributes file name the filters it selects,
/// comments excluded,
/// and a name is read as its own bytes.
#[test]
fn filters_are_read_from_attribute_lines() {
    assert_eq!(
        filters_named(b"*.a filter=one\n# *.b filter=two\n*.c text filter=three -diff\n"),
        vec![b"one".to_vec(), b"three".to_vec()]
    );
    assert_eq!(filters_named(b"*.d -filter\n*.e text\n"), Vec::<Vec<u8>>::new());
}

/// A destination that was replaced after the add was bound is no longer the directory Git made,
/// even though the held handle
///  still names the old one:
/// the path is asked,
/// not the handle.
#[test]
fn a_destination_replaced_after_the_binding_ends_the_reading() {
    let root: PathBuf = fixture("target-replaced-after-binding");
    let (repo, destination, source) = fixture_with_worktree(root.as_path(), false);
    let plan = plan_to(destination.clone());
    let bound: Bound = bind_registration(source.common_dir.as_path(), destination.as_path()).expect("bound");
    assert!(prepare_target(&source, &plan, &git_context(), &bound).is_ok(), "positive control");
    std::fs::rename(destination.as_path(), root.join("topic.moved")).expect("moved away");
    std::fs::create_dir(destination.as_path()).expect("another directory");
    std::fs::write(destination.join(".git"), std::fs::read(root.join("topic.moved/.git")).expect("pointer")).expect("a copy of the pointer");
    assert!(prepare_target(&source, &plan, &git_context(), &bound).expect_err("replaced").contains("no longer the directory Git made"));
    drop(repo);
    remove(root.as_path());
}

/// The configuration entries of the standard Git LFS filter,
/// with `core.attributesFile` naming `attributes`.
fn standard_lfs_entries(attributes: Vec<u8>) -> Vec<ConfigEntry> {
    let mut entries: Vec<ConfigEntry> = Vec::new();
    for (key, value) in LFS_FILTER {
        entries.push(ConfigEntry { scope: String::from("local"), key: String::from(key), value: Some(value.as_bytes().to_vec()) });
    }
    entries.push(ConfigEntry { scope: String::from("local"), key: String::from("core.attributesfile"), value: Some(attributes) });
    return entries;
}

/// What:
///  The attributes file `core.attributesFile` names is read when its own path is not UTF-8 text.
/// Why:
///   The value was rebuilt as text,
/// so the file at that path was not found,
///  and a filter it selects would have gone unweighed:
///  the conversion would have been called equal without reading the rules that change it.
#[test]
fn an_attributes_file_whose_path_is_not_text_is_read() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("target-attributes-not-text");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    let source: SnapshotSource = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let attributes: PathBuf = root.join(OsString::from_vec(b"attributes\xff".to_vec()));
    std::fs::write(attributes.as_path(), b"* filter=lfs\n").expect("attributes file");
    let entries: Vec<ConfigEntry> = standard_lfs_entries(path_bytes_of(attributes.as_path()).to_vec());
    let texts: Vec<Vec<u8>> = untracked_attribute_texts(&source, entries.as_slice());
    assert!(texts.contains(&b"* filter=lfs\n".to_vec()), "the file the entry names is read: {texts:?}");
    assert_eq!(
        external_conversion_refusal(entries.as_slice(), texts.as_slice()),
        None,
        "the standard LFS filter is proven equal"
    );
    remove(root.as_path());
}

/// What:
///  An attributes file whose bytes are not text selects no filter this ladder proves equal.
/// Why:
///   Reading such a file as text replaces every byte that is not UTF-8 with the same character,
///  so two different rule files would look alike;
///  a filter name is compared as its own bytes here,
///  and a name that is not `lfs` is a reason not to reuse the target.
#[test]
fn an_attributes_file_that_is_not_text_names_no_proven_conversion() {
    let root: PathBuf = fixture("target-attributes-bytes");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    let source: SnapshotSource = common_source(&git_context(), &main_identity(repo.as_path())).expect("a source");
    let attributes: PathBuf = root.join("attributes");
    std::fs::write(attributes.as_path(), b"* filter=lf\xffs\n").expect("attributes file");
    let entries: Vec<ConfigEntry> = standard_lfs_entries(path_bytes_of(attributes.as_path()).to_vec());
    let texts: Vec<Vec<u8>> = untracked_attribute_texts(&source, entries.as_slice());
    let refusal: String = external_conversion_refusal(entries.as_slice(), texts.as_slice()).expect("no proven conversion");
    assert!(refusal.contains("filter lf"), "the refusal names the filter it could not prove: {refusal}");
    remove(root.as_path());
}

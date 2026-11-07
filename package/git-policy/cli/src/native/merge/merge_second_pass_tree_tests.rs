//! Controls for the second pass over a merge from objects:
//! the attribute source follows the caller's own,
//! and a tree comes
//!  back only when every conflicted file resolved,
//! with the other files of the tree untouched and no scratch index left.

use super::*;
use crate::merge_second_pass::ContextRunner;
use crate::merge_second_pass::tests::{Change, World, corpus, merge_tree_of, runner, world};
use crate::test_support::{git, git_context, git_text, remove};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// Every conflicted file resolves:
/// the tree holds the merged bytes,
/// every other path is as merge-tree wrote it,
/// and the
/// scratch index is gone.
#[test]
fn a_tree_comes_back_with_the_merged_files_and_nothing_else_changed() {
    let world: World = world(
        "second-pass-tree",
        &[("a.md", corpus("md1b", "base").as_slice()), ("clean.txt", b"one\n"), ("sub/deep.md", corpus("md1b", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice())), ("sub/deep.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice())), ("clean.txt", Change::Write(b"two\n")), ("sub/deep.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    let context: GitContext = git_context();
    let common: PathBuf = world.repo.join(".git");
    let resolved: Resolved = resolve_tree(&runner(&context, world.repo.as_path()), common.as_path(), tree.as_str(), conflicts.as_slice()).expect("every file resolves");
    assert_eq!(resolved.merged.len(), 2);
    for path in ["a.md", "sub/deep.md"] {
        let blob: String = git_text(world.repo.as_path(), &["show", format!("{}:{path}", resolved.tree).as_str()]) + "\n";
        assert_eq!(blob.as_bytes(), corpus("md1b", "word.out").as_slice(), "{path}");
    }
    assert_eq!(git_text(world.repo.as_path(), &["show", format!("{}:clean.txt", resolved.tree).as_str()]), "two");
    let leftovers: Vec<String> = std::fs::read_dir(common.join("monochromatic-git-policy-cli-scratch")).expect("the scratch directory").filter_map(|entry| { return entry.ok()?.file_name().to_str().map(String::from); }).filter(|name| { return name.starts_with("second-pass-"); }).collect();
    assert_eq!(leftovers, Vec::<String>::new());
    remove(world.root.as_path());
}

/// One file that does not resolve keeps the whole merge conflicted,
/// although the other resolves.
#[test]
fn one_file_left_over_returns_nothing() {
    let world: World = world(
        "second-pass-tree-left",
        &[("a.md", corpus("md1b", "base").as_slice()), ("b.md", corpus("md2", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice())), ("b.md", Change::Write(corpus("md2", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice())), ("b.md", Change::Write(corpus("md2", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    assert_eq!(conflicts.len(), 2);
    let context: GitContext = git_context();
    let common: PathBuf = world.repo.join(".git");
    assert_eq!(resolve_tree(&runner(&context, world.repo.as_path()), common.as_path(), tree.as_str(), conflicts.as_slice()), None);
    remove(world.root.as_path());
}

/// A runner that cannot start Git ends in nothing,
/// not in a panic.
#[test]
fn a_git_that_cannot_start_returns_nothing() {
    let world: World = world(
        "second-pass-tree-nogit",
        &[("a.md", corpus("md1b", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent/git");
    let common: PathBuf = world.repo.join(".git");
    let broken: ContextRunner<'_> = runner(&context, world.repo.as_path());
    assert_eq!(resolve_tree(&broken, common.as_path(), tree.as_str(), conflicts.as_slice()), None);
    remove(world.root.as_path());
}

/// A repository with one commit,
/// for the attribute-tree precedence controls.
fn probed_repository(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = crate::test_support::fixture(name);
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    return (root, repo);
}

/// The variables Git's own precedence leaves to the tip for a repository,
/// with `variables` as the environment and `prefix` as the
///  global options.
fn tip_variables(repo: &Path, prefix: &[OsString], variables: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    let context: GitContext = git_context();
    let runner: ContextRunner<'_> = ContextRunner { context: &context, cwd: repo, without_prefix: false, object_directory: None, environment: Vec::new() };
    let probe = AttributeProbe { directory: repo, global_prefix: prefix, variables, runner: &runner };
    return attribute_tree_variables("tip123", &probe);
}

/// The tip is named where Git would read no attribute tree;
/// `--attr-source`,
/// `GIT_ATTR_SOURCE` and a configured `attr.tree` that
///  resolves each keep the person's choice,
/// and an `attr.tree` that is no tree,
/// which Git ignores,
/// does not.
#[test]
fn the_tip_is_named_only_where_git_would_read_no_attribute_tree() {
    let (root, repo) = probed_repository("second-pass-attr-precedence");
    let tip: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("tip123"))];
    let none: Vec<(OsString, OsString)> = vec![(OsString::from("HOME"), crate::test_support::fixture_root().into_os_string())];
    assert_eq!(tip_variables(repo.as_path(), &[], none.as_slice()), tip, "nothing names an attribute tree");
    let named: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("main"))];
    assert!(tip_variables(repo.as_path(), &[], named.as_slice()).is_empty(), "the variable is the person's");
    for option in ["--attr-source=main", "--attr-source"] {
        assert!(tip_variables(repo.as_path(), &[OsString::from(option), OsString::from("main")], none.as_slice()).is_empty(), "{option}");
    }
    git(repo.as_path(), &["config", "attr.tree", "HEAD"]);
    assert!(tip_variables(repo.as_path(), &[], none.as_slice()).is_empty(), "a configured tree that resolves is the person's");
    git(repo.as_path(), &["config", "attr.tree", "no-such-tree"]);
    assert_eq!(tip_variables(repo.as_path(), &[], none.as_slice()), tip, "Git ignores an attr.tree that is no tree, so the tip is named");
    remove(root.as_path());
}

/// A runner that must not be used:
/// every start is counted and refused.
struct Refusing {
    /// How many starts were asked for.
    starts: std::cell::Cell<usize>,
}

/// Counting the starts a probe asks for.
impl GitRunner for Refusing {
    fn run(&self, _arguments: &[&str], _input: Option<&[u8]>, _index_file: Option<&Path>) -> Result<crate::merge_second_pass::GitReply, String> {
        self.starts.set(self.starts.get() + 1);
        return Err(String::from("no Git start is allowed here"));
    }
}

/// The variables of a person who sets an editor,
/// tracing and a merge verbosity do not make the in-process readers decline,
/// so the
///  attribute tree is chosen without a Git start;
/// a variable that can move an answer does,
/// and Git is then asked once.
#[test]
fn common_variables_cost_no_git_start_and_a_variable_that_moves_an_answer_costs_one() {
    let (root, repo) = probed_repository("second-pass-attr-no-start");
    let hermetic = |extra: &[(&str, &str)]| -> Vec<(OsString, OsString)> {
        let mut variables: Vec<(OsString, OsString)> = vec![
            (OsString::from("HOME"), root.join("no-home").into_os_string()),
            (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
            (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
        ];
        variables.extend(extra.iter().map(|(name, value)| { return (OsString::from(*name), OsString::from(*value)); }));
        return variables;
    };
    let starts_for = |variables: &[(OsString, OsString)]| -> (usize, Vec<(OsString, OsString)>) {
        let runner: Refusing = Refusing { starts: std::cell::Cell::new(0) };
        let probe = AttributeProbe { directory: repo.as_path(), global_prefix: &[], variables, runner: &runner };
        let chosen = attribute_tree_variables("tip123", &probe);
        return (runner.starts.get(), chosen);
    };
    let tip: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("tip123"))];
    let common: Vec<(OsString, OsString)> = hermetic(&[("GIT_EDITOR", "true"), ("GIT_TRACE", "0"), ("GIT_MERGE_VERBOSITY", "1"), ("GIT_PAGER", "cat")]);
    assert_eq!(starts_for(common.as_slice()), (0, tip), "the readers answer, and nothing names an attribute tree");
    let moving: Vec<(OsString, OsString)> = hermetic(&[("GIT_NAMESPACE", "x")]);
    assert_eq!(starts_for(moving.as_slice()).0, 1, "a variable that can move an answer sends the question to Git");
    remove(root.as_path());
}

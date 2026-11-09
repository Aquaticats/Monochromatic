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

/// The variables of a hermetic environment for the attribute-tree choice:
/// a home of the fixture's own,
/// and no global or system configuration.
fn hermetic_variables(root: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("HOME"), root.to_path_buf().into_os_string()),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
    ];
}

/// The attribute-tree choice of a merge from objects over `repo`,
/// with nothing named by the person and `prefix` as the global options.
fn probe_of<'a>(repo: &'a Path, prefix: &'a [OsString], variables: &'a [(OsString, OsString)], runner: &'a ContextRunner<'a>) -> AttributeProbe<'a> {
    return AttributeProbe { directory: repo, global_prefix: prefix, variables, runner };
}

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
    let pass: ContextRunner<'_> = runner(&context, world.repo.as_path());
    let variables: Vec<(OsString, OsString)> = hermetic_variables(world.root.as_path());
    let probe: AttributeProbe<'_> = probe_of(world.repo.as_path(), &[], variables.as_slice(), &pass);
    let resolved: Resolved = resolve_tree(&pass, common.as_path(), tree.as_str(), conflicts.as_slice(), &probe).expect("every file resolves");
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
    let pass: ContextRunner<'_> = runner(&context, world.repo.as_path());
    let variables: Vec<(OsString, OsString)> = hermetic_variables(world.root.as_path());
    let probe: AttributeProbe<'_> = probe_of(world.repo.as_path(), &[], variables.as_slice(), &pass);
    assert_eq!(resolve_tree(&pass, common.as_path(), tree.as_str(), conflicts.as_slice(), &probe), None);
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
    let variables: Vec<(OsString, OsString)> = hermetic_variables(world.root.as_path());
    let probe: AttributeProbe<'_> = probe_of(world.repo.as_path(), &[], variables.as_slice(), &broken);
    assert_eq!(resolve_tree(&broken, common.as_path(), tree.as_str(), conflicts.as_slice(), &probe), None);
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

/// A `merge` rule only the merged side carries arrives with the merge itself,
/// so only the tree the merge produced holds it:
/// the second pass reads it there,
/// and a file it excludes keeps its conflict,
/// while the same rule set to `text` lets the file resolve.
#[test]
fn a_rule_the_merge_itself_introduces_governs_the_second_pass() {
    for (value, expected) in [("binary", false), ("text", true)] {
        let name: String = format!("second-pass-tree-introduced-{value}");
        let world: World = world(
            name.as_str(),
            &[("data.lock", b"port=1 host=a zone=z\n")],
            &[("data.lock", Change::Write(b"port=9 host=a zone=z\n"))],
            &[(".gitattributes", Change::Write(format!("data.lock merge={value}\n").as_bytes())), ("data.lock", Change::Write(b"port=1 host=b zone=z\n"))],
        );
        let (tree, conflicts) = merge_tree_of(&world);
        assert_eq!(conflicts.len(), 1, "only data.lock conflicts: {conflicts:?}");
        let context: GitContext = git_context();
        let common: PathBuf = world.repo.join(".git");
        let pass: ContextRunner<'_> = runner(&context, world.repo.as_path());
        let variables: Vec<(OsString, OsString)> = hermetic_variables(world.root.as_path());
        let probe: AttributeProbe<'_> = probe_of(world.repo.as_path(), &[], variables.as_slice(), &pass);
        assert_eq!(merge_attribute_source(tree.as_str(), conflicts.as_slice(), &probe), AttributeSource::Tree(tree.as_str()), "merge={value}");
        assert_eq!(resolve_tree(&pass, common.as_path(), tree.as_str(), conflicts.as_slice(), &probe).is_some(), expected, "merge={value}");
        remove(world.root.as_path());
    }
}

/// A `.gitattributes` the merge left conflicted names no rule at all:
/// its marker file's last line would say `text`
/// while the first side excluded the file,
/// so every candidate keeps its conflict instead.
#[test]
fn a_gitattributes_the_merge_left_conflicted_names_no_rule() {
    let world: World = world(
        "second-pass-tree-conflicted-attributes",
        &[(".gitattributes", b"a.md merge=text\n"), ("a.md", corpus("md1b", "base").as_slice())],
        &[(".gitattributes", Change::Write(b"a.md merge=binary\n")), ("a.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[(".gitattributes", Change::Write(b"a.md merge=text text\n")), ("a.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    let paths: Vec<&[u8]> = conflicts.iter().map(|conflict| { return conflict.path.as_slice(); }).collect();
    assert_eq!(paths, vec![b".gitattributes".as_slice(), b"a.md".as_slice()], "both sides changed the rule line");
    let rule: String = git_text(world.repo.as_path(), &["show", format!("{tree}:.gitattributes").as_str()]);
    assert!(rule.contains("<<<<<<<"), "the rule file of the merge holds Git's markers: {rule}");
    let context: GitContext = git_context();
    let common: PathBuf = world.repo.join(".git");
    let pass: ContextRunner<'_> = runner(&context, world.repo.as_path());
    let variables: Vec<(OsString, OsString)> = hermetic_variables(world.root.as_path());
    let probe: AttributeProbe<'_> = probe_of(world.repo.as_path(), &[], variables.as_slice(), &pass);
    assert_eq!(merge_attribute_source(tree.as_str(), conflicts.as_slice(), &probe), AttributeSource::Unknown);
    assert_eq!(resolve_tree(&pass, common.as_path(), tree.as_str(), conflicts.as_slice(), &probe), None, "no candidate is merged from an unreadable rule");
    remove(world.root.as_path());
}

/// The tree the merge produced is the pass's source only where the person named none,
/// and a `.gitattributes` of any folder counts as unsettled.
#[test]
fn the_merge_result_tree_is_named_only_where_the_person_named_no_source() {
    let (root, repo) = probed_repository("second-pass-merge-source");
    let context: GitContext = git_context();
    let pass: ContextRunner<'_> = runner(&context, repo.as_path());
    let none: Vec<(OsString, OsString)> = hermetic_variables(root.as_path());
    let settled: Vec<Conflict> = vec![Conflict { path: b"a.md".to_vec(), base: None, ours: None, theirs: None }];
    let asked = |prefix: &[OsString], variables: &[(OsString, OsString)], conflicts: &[Conflict]| -> AttributeSource<'static> {
        let probe: AttributeProbe<'_> = probe_of(repo.as_path(), prefix, variables, &pass);
        return merge_attribute_source("tree123", conflicts, &probe);
    };
    assert_eq!(asked(&[], none.as_slice(), settled.as_slice()), AttributeSource::Tree("tree123"), "nothing names a source, so the merge's tree is the one");
    let nearest: Vec<Conflict> = vec![Conflict { path: b"deep/sub/.gitattributes".to_vec(), base: None, ours: None, theirs: None }];
    assert_eq!(asked(&[], none.as_slice(), nearest.as_slice()), AttributeSource::Unknown, "a rule file of any folder is unsettled while it conflicts");
    let named: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("main"))];
    assert_eq!(asked(&[], named.as_slice(), settled.as_slice()), AttributeSource::Invocation, "the variable is the person's");
    for option in ["--attr-source=main", "--attr-source"] {
        assert_eq!(asked(&[OsString::from(option), OsString::from("main")], none.as_slice(), settled.as_slice()), AttributeSource::Invocation, "{option}");
    }
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

/// What:
///  `git -c attr.tree=<spec>` names the attribute tree,
/// so the merge's own tree is not named over it.
/// Why:
///   Git's precedence puts a command-line configuration above every file,
/// and the pass read only the files
///  (cross-vendor review of this branch,
/// finding 3):
/// the choice came out `Free`,
/// the tree the merge produced was named,
/// and `check-attr --source=`
///  overrode what the person asked for.
#[test]
fn a_configuration_option_on_the_command_line_names_the_tree() {
    let (root, repo) = probed_repository("second-pass-attr-command-line");
    let none: Vec<(OsString, OsString)> = hermetic_variables(root.as_path());
    let given = |prefix: &[&str]| -> Vec<OsString> { return prefix.iter().map(|token| { return OsString::from(*token); }).collect() };
    let tip: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("tip123"))];
    assert!(tip_variables(repo.as_path(), given(&["-c", "attr.tree=HEAD"]).as_slice(), none.as_slice()).is_empty(), "`-c attr.tree` names the tree");
    assert!(tip_variables(repo.as_path(), given(&["--no-pager", "-c", "attr.tree=HEAD"]).as_slice(), none.as_slice()).is_empty(), "an option before it changes nothing");
    assert_eq!(
        tip_variables(repo.as_path(), given(&["-c", "core.autocrlf=true"]).as_slice(), none.as_slice()),
        tip,
        "a configuration option that names no attribute tree leaves Git's own answer, which names the tip"
    );
    // An `attr.tree` the command line sets to something that is no tree is ignored by Git, as a configured one is.
    assert_eq!(
        tip_variables(repo.as_path(), given(&["-c", "attr.tree=no-such-tree"]).as_slice(), none.as_slice()),
        tip,
        "Git ignores an attr.tree that is no tree, so the tip is named"
    );
    remove(root.as_path());
}

/// What:
///  `--config-env=attr.tree=NAME` names the attribute tree,
/// in both of its forms,
/// so the merge's own tree is not named over it.
/// Why:
///   Git reads `--config-env` as a configuration value above every file,
/// taking the value from the variable it names
///  (`config.c`,
/// `git_config_from_parameters`),
/// and the pass read only separate `-c` arguments:
/// the choice came out `Free`,
/// the tree the
///  merge produced was named,
/// and `check-attr --source=` overrode what the person asked for
/// (cross-vendor review round 3,
/// finding 3).
/// A variable the environment does not hold makes Git stop before it reads anything,
/// so no tree may be named
///  over it either.
#[test]
fn a_configuration_environment_option_names_the_tree() {
    let (root, repo) = probed_repository("second-pass-attr-config-env");
    let given = |prefix: &[&str]| -> Vec<OsString> { return prefix.iter().map(|token| { return OsString::from(*token); }).collect() };
    let tip: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("tip123"))];
    let holding = |name: &str, value: &str| -> Vec<(OsString, OsString)> {
        let mut variables: Vec<(OsString, OsString)> = hermetic_variables(root.as_path());
        variables.push((OsString::from(name), OsString::from(value)));
        return variables;
    };
    let none: Vec<(OsString, OsString)> = hermetic_variables(root.as_path());
    assert!(
        tip_variables(repo.as_path(), given(&["--config-env=attr.tree=RULE_TREE"]).as_slice(), holding("RULE_TREE", "HEAD").as_slice()).is_empty(),
        "the attached form names the tree, whose value is the variable's"
    );
    assert!(
        tip_variables(repo.as_path(), given(&["--config-env", "attr.tree=RULE_TREE"]).as_slice(), holding("RULE_TREE", "HEAD").as_slice()).is_empty(),
        "the separate form names it too"
    );
    assert!(
        tip_variables(repo.as_path(), given(&["--config-env=attr.tree=RULE_TREE"]).as_slice(), none.as_slice()).is_empty(),
        "the environment holds no such variable, so Git stops on it and no tree is named over the person's choice"
    );
    assert_eq!(
        tip_variables(repo.as_path(), given(&["--config-env=attr.tree=RULE_TREE"]).as_slice(), holding("RULE_TREE", "no-such-tree").as_slice()),
        tip,
        "a value no tree resolves to is ignored, as a configured one is"
    );
    assert_eq!(
        tip_variables(repo.as_path(), given(&["--config-env=attr.tree=RULE_TREE"]).as_slice(), holding("RULE_TREE", "").as_slice()),
        tip,
        "an empty value is no value, which Git ignores"
    );
    assert_eq!(
        tip_variables(repo.as_path(), given(&["--config-env=core.autocrlf=RULE_TREE"]).as_slice(), holding("RULE_TREE", "HEAD").as_slice()),
        tip,
        "a configuration entry for another key names no attribute tree"
    );
    assert_eq!(
        tip_variables(repo.as_path(), given(&["--config-env=attr.tree=x=RULE_TREE"]).as_slice(), holding("RULE_TREE", "HEAD").as_slice()),
        tip,
        "`--config-env` splits at its last `=`, so this sets the key `attr.tree=x`"
    );
    remove(root.as_path());
}

/// What:
///  `-c Attr.Tree=<spec>` names the attribute tree,
/// because a Git configuration key's letters are not significant.
/// Why:
///   Git reads `Attr.Tree` as `attr.tree` (`git config --get` and `check-attr` agree),
/// so matching the exact lower-case spelling let the pass name its own source over the one the person
///  asked for
/// (cross-vendor review round 2,
/// finding 2).
#[test]
fn a_configuration_option_in_any_letter_case_names_the_tree() {
    let (root, repo) = probed_repository("second-pass-attr-any-case");
    let none: Vec<(OsString, OsString)> = hermetic_variables(root.as_path());
    let given = |prefix: &[&str]| -> Vec<OsString> { return prefix.iter().map(|token| { return OsString::from(*token); }).collect() };
    let tip: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("tip123"))];
    assert!(tip_variables(repo.as_path(), given(&["-c", "Attr.Tree=HEAD"]).as_slice(), none.as_slice()).is_empty(), "`-c Attr.Tree` names the tree");
    assert!(tip_variables(repo.as_path(), given(&["-c", "ATTR.TREE=HEAD"]).as_slice(), none.as_slice()).is_empty(), "`-c ATTR.TREE` does too");
    assert_eq!(
        tip_variables(repo.as_path(), given(&["-c", "Attr.Tree=no-such-tree"]).as_slice(), none.as_slice()),
        tip,
        "a value no tree resolves to is ignored whatever the key's case"
    );
    remove(root.as_path());
}

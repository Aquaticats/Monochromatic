//! Controls for how a pull is read:
//! which command lines and settings make it a merge the wrapper completes.

use super::*;
use crate::test_support::{fixture, git, git_context, git_output, remove, repository};
use std::path::PathBuf;

/// Command-line words as the owned text a command holds.
fn words(list: &[&str]) -> Vec<OsString> {
    return list.iter().map(|word| { return OsString::from(*word); }).collect();
}

/// The settings that add steps the split pull does not mirror,
/// as the keys present.
fn settings(list: &[&str]) -> Vec<String> {
    return list.iter().map(|key| { return String::from(*key); }).collect();
}

/// What the rebase settings of a repository holding these `git config --add` entries say about a pull,
/// with what Git itself reads there.
fn statement_of(name: &str, entries: &[(&str, &str)]) -> (RebaseStatement, Option<String>) {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for (key, value) in entries {
        git(repo.as_path(), &["config", "--add", key, value]);
    }
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let statement: RebaseStatement = rebase_statement(&context, "main");
    // The key `config_get_rebase` reads first when it is present, as Git judges this repository.
    let named: bool = entries.iter().any(|(key, _value)| { return *key == "branch.main.rebase"; });
    let key: &str = if named { "branch.main.rebase" } else { "pull.rebase" };
    let asked = git_output(repo.as_path(), &["config", "--bool", "--get", key]);
    let read: Option<String> = if asked.status.success() {
        Some(String::from_utf8_lossy(asked.stdout.as_slice()).trim().to_owned())
    } else {
        None
    };
    remove(parent.as_path());
    return (statement, read);
}

/// Each option goes to the half of the pull that takes it,
/// the shared ones to both,
/// and the operands stay in order.
#[test]
fn options_go_to_the_half_that_takes_them() {
    let plan: PullPlan = plan_pull(words(&["--no-rebase", "--no-edit", "--prune", "-q", "origin", "main"]).as_slice()).expect("read exactly");
    assert_eq!(plan.merge, words(&["--no-edit", "-q"]));
    assert_eq!(plan.fetch, words(&["--prune", "-q"]));
    assert_eq!(plan.places, words(&["origin", "main"]));
    assert!(plan.merge_stated);
    assert!(!plan.fast_forward_stated);
    assert!(plan_pull(words(&["--no-ff"]).as_slice()).expect("read exactly").fast_forward_stated);
}

/// An option this does not read exactly,
/// an abbreviation,
/// `--`,
/// or a refspec with a destination leaves the pull to native Git.
#[test]
fn what_is_not_read_exactly_is_native_gits() {
    for rest in [
        vec!["--rebase"],
        vec!["--ff-only"],
        vec!["--autostash"],
        vec!["--dry-run"],
        vec!["-s", "ours"],
        vec!["--no-edi"],
        vec!["--"],
        vec!["origin", "main:main"],
    ] {
        assert_eq!(plan_pull(words(rest.as_slice()).as_slice()), None, "{rest:?}");
    }
    assert!(plan_pull(words(&["origin"]).as_slice()).is_some(), "a repository operand alone is read");
    assert!(plan_pull(words(&["ssh://host:22/repo"]).as_slice()).is_some(), "a colon in the repository is not a refspec");
}

/// The command line and the settings decide whether the pull merges,
/// as `cmd_pull` decides it.
#[test]
fn merging_is_stated_by_the_line_or_the_settings() {
    let typed: PullPlan = plan_pull(words(&["--no-rebase"]).as_slice()).expect("read exactly");
    let plain: PullPlan = plan_pull(words(&[]).as_slice()).expect("read exactly");
    let ff: PullPlan = plan_pull(words(&["--no-ff"]).as_slice()).expect("read exactly");
    assert!(merge_is_stated(&typed, settings(&[]).as_slice(), RebaseStatement::NotMerge), "the line overrides the setting");
    assert!(merge_is_stated(&plain, settings(&[]).as_slice(), RebaseStatement::Merge));
    assert!(!merge_is_stated(&plain, settings(&[]).as_slice(), RebaseStatement::NotMerge));
    assert!(!merge_is_stated(&plain, settings(&[]).as_slice(), RebaseStatement::Unstated), "nothing stated: Git refuses divergent branches itself");
    assert!(merge_is_stated(&ff, settings(&[]).as_slice(), RebaseStatement::Unstated), "a typed way of reconciling is a stated merge");
    assert!(!merge_is_stated(&typed, settings(&["pull.autostash"]).as_slice(), RebaseStatement::Merge), "a stash around the pull is native Git's");
    assert!(!merge_is_stated(&typed, settings(&["pull.ff"]).as_slice(), RebaseStatement::Merge), "pull.ff is Git's to combine");
    assert!(merge_is_stated(&ff, settings(&["pull.ff"]).as_slice(), RebaseStatement::Unstated), "the typed choice replaces pull.ff");
    assert!(!merge_is_stated(&typed, settings(&["submodule.recurse"]).as_slice(), RebaseStatement::Merge));
}

/// The rebase setting is read the way Git's own pull reads it:
/// the last of several values,
/// and every spelling of a boolean.
/// Why:
///   The wrapper read the first value and a short list of spellings,
/// so a valid setting Git reads as false kept the pull native,
/// where native Git's merge failed over another session's staged work.
#[test]
fn the_rebase_setting_is_read_the_way_git_reads_it() {
    let cases: [(&str, &[(&str, &str)], RebaseStatement); 21] = [
        ("none", &[], RebaseStatement::Unstated),
        ("false", &[("pull.rebase", "false")], RebaseStatement::Merge),
        ("no", &[("pull.rebase", "no")], RebaseStatement::Merge),
        ("off-shouted", &[("pull.rebase", "OFF")], RebaseStatement::Merge),
        ("zero", &[("pull.rebase", "0")], RebaseStatement::Merge),
        ("double-zero", &[("pull.rebase", "00")], RebaseStatement::Merge),
        ("hex-zero", &[("pull.rebase", "0x0")], RebaseStatement::Merge),
        ("plus-zero", &[("pull.rebase", "+0")], RebaseStatement::Merge),
        ("zero-with-unit", &[("pull.rebase", "0k")], RebaseStatement::Merge),
        ("empty", &[("pull.rebase", "")], RebaseStatement::Merge),
        ("true", &[("pull.rebase", "true")], RebaseStatement::NotMerge),
        ("one", &[("pull.rebase", "1")], RebaseStatement::NotMerge),
        ("merges", &[("pull.rebase", "merges")], RebaseStatement::NotMerge),
        ("interactive", &[("pull.rebase", "interactive")], RebaseStatement::NotMerge),
        ("unreadable", &[("pull.rebase", "nonsense")], RebaseStatement::NotMerge),
        ("duplicate-true-then-false", &[("pull.rebase", "true"), ("pull.rebase", "false")], RebaseStatement::Merge),
        ("duplicate-false-then-true", &[("pull.rebase", "false"), ("pull.rebase", "true")], RebaseStatement::NotMerge),
        ("branch-duplicate-true-then-false", &[("branch.main.rebase", "true"), ("branch.main.rebase", "false")], RebaseStatement::Merge),
        ("branch-zero", &[("branch.main.rebase", "00")], RebaseStatement::Merge),
        ("branch-false-beats-pull-true", &[("branch.main.rebase", "false"), ("pull.rebase", "true")], RebaseStatement::Merge),
        ("branch-true-beats-pull-false", &[("branch.main.rebase", "true"), ("pull.rebase", "false")], RebaseStatement::NotMerge),
    ];
    for (name, entries, expected) in cases {
        let (statement, git_reads) = statement_of(format!("pull-rebase-{name}").as_str(), entries);
        assert_eq!(statement, expected, "{name}: Git reads {git_reads:?}");
        // A case whose expectation Git reads as a boolean must expect exactly what Git answers, so the table cannot
        // drift from Git's own reading of the value.
        if let Some(read) = git_reads {
            let expected_read: &str = if expected == RebaseStatement::Merge { "false" } else { "true" };
            assert_eq!(read, expected_read, "{name}: the expectation is Git's own reading");
        }
    }
}

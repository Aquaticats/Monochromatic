//! Controls for how a pull is read:
//! which command lines and settings make it a merge the wrapper completes.

use super::*;

/// Command-line words as the owned text a command holds.
fn words(list: &[&str]) -> Vec<OsString> {
    return list.iter().map(|word| { return OsString::from(*word); }).collect();
}

/// Settings as key and value pairs.
fn settings(list: &[(&str, &str)]) -> Vec<(String, String)> {
    return list.iter().map(|(key, value)| { return (String::from(*key), String::from(*value)); }).collect();
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
    assert!(merge_is_stated(&typed, settings(&[("pull.rebase", "true")]).as_slice(), "main"), "the line overrides the setting");
    assert!(merge_is_stated(&plain, settings(&[("pull.rebase", "false")]).as_slice(), "main"));
    assert!(!merge_is_stated(&plain, settings(&[("pull.rebase", "true")]).as_slice(), "main"));
    assert!(!merge_is_stated(&plain, settings(&[("pull.rebase", "merges")]).as_slice(), "main"));
    assert!(merge_is_stated(&plain, settings(&[("pull.rebase", "true"), ("branch.main.rebase", "false")]).as_slice(), "main"), "the branch wins");
    assert!(!merge_is_stated(&plain, settings(&[("pull.rebase", "false"), ("branch.main.rebase", "true")]).as_slice(), "main"));
    assert!(merge_is_stated(&plain, settings(&[("pull.rebase", "false"), ("branch.other.rebase", "true")]).as_slice(), "main"), "another branch's setting is not this one's");
    assert!(!merge_is_stated(&plain, settings(&[]).as_slice(), "main"), "nothing stated: Git refuses divergent branches itself");
    assert!(merge_is_stated(&ff, settings(&[]).as_slice(), "main"), "a typed way of reconciling is a stated merge");
    assert!(!merge_is_stated(&typed, settings(&[("pull.autostash", "true")]).as_slice(), "main"), "a stash around the pull is native Git's");
    assert!(!merge_is_stated(&typed, settings(&[("pull.ff", "only")]).as_slice(), "main"), "pull.ff is Git's to combine");
    assert!(merge_is_stated(&ff, settings(&[("pull.ff", "only")]).as_slice(), "main"), "the typed choice replaces pull.ff");
    assert!(!merge_is_stated(&typed, settings(&[("submodule.recurse", "true")]).as_slice(), "main"));
}

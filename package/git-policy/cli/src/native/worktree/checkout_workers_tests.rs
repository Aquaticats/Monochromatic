//! Controls for the decision to pass `checkout.workers=0`:
//! each place the person can set the key,
//! the position of the option,
//! and the one step that is a checkout.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// Text arguments as owned operating-system arguments.
fn words(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// A typed `-c`,
/// in both spellings,
/// and `--config-env` stop the addition;
/// options after the subcommand do not count.
#[test]
fn a_typed_option_before_the_subcommand_is_the_persons() {
    assert!(typed_before_subcommand(words(&["-c", "checkout.workers=3", "worktree", "add"]).as_slice()));
    assert!(typed_before_subcommand(words(&["-c", "Checkout.Workers=3", "worktree", "add"]).as_slice()));
    assert!(typed_before_subcommand(words(&["--config-env=checkout.workers=WORKERS", "worktree", "add"]).as_slice()));
    assert!(!typed_before_subcommand(words(&["-c", "core.hooksPath=/x", "-C", "/r", "worktree", "add"]).as_slice()), "positive control: other options");
    assert!(!typed_before_subcommand(words(&["worktree", "add", "-b", "checkout.workers"]).as_slice()), "a branch named like the key");
}

/// The environment variables that carry configuration stop the addition;
/// unrelated variables do not.
#[test]
fn an_exported_value_is_the_persons() {
    let key: OsString = OsString::from("GIT_CONFIG_KEY_0");
    let value: OsString = OsString::from("checkout.workers");
    assert!(environment_sets([(key.as_os_str(), value.as_os_str())]));
    let parameters: OsString = OsString::from("GIT_CONFIG_PARAMETERS");
    let quoted: OsString = OsString::from("'checkout.workers=2'");
    assert!(environment_sets([(parameters.as_os_str(), quoted.as_os_str())]));
    let other_key: OsString = OsString::from("core.editor");
    assert!(!environment_sets([(key.as_os_str(), other_key.as_os_str())]), "positive control: another key");
    let unrelated: OsString = OsString::from("PATH");
    assert!(!environment_sets([(unrelated.as_os_str(), value.as_os_str())]), "a variable that carries no configuration");
}

/// The option goes in front of the subcommand and after the global options already there.
#[test]
fn the_option_goes_before_the_subcommand_and_after_the_global_options() {
    let mut arguments: Vec<OsString> = words(&["-C", "/r", "-c", "core.hooksPath=/x", "worktree", "add", "../w"]);
    insert_parallel_checkout(&mut arguments);
    assert_eq!(arguments, words(&["-C", "/r", "-c", "core.hooksPath=/x", "-c", "checkout.workers=0", "worktree", "add", "../w"]));
}

/// A global option with a separate value (`--git-dir <path>`,
/// `--work-tree <path>`) is skipped,
/// so the option still lands before `worktree`.
#[test]
fn separate_valued_global_options_do_not_move_the_subcommand() {
    let mut arguments: Vec<OsString> = words(&["--git-dir", "/r/.git", "--work-tree", "/r", "worktree", "add", "--quiet", "w"]);
    assert!(!typed_before_subcommand(arguments.as_slice()));
    insert_parallel_checkout(&mut arguments);
    assert_eq!(arguments, words(&["--git-dir", "/r/.git", "--work-tree", "/r", "-c", "checkout.workers=0", "worktree", "add", "--quiet", "w"]));
    assert!(typed_before_subcommand(words(&["--git-dir", "/r/.git", "-c", "checkout.workers=2", "worktree", "add"]).as_slice()));
}

/// Only the fallback checkout is a checkout step.
#[test]
fn only_the_reset_to_head_is_a_checkout_step() {
    assert!(is_fallback_checkout(words(&["-C", "/w", "reset", "--hard", "--no-recurse-submodules"]).as_slice()));
    assert!(!is_fallback_checkout(words(&["-C", "/w", "reset", "--mixed"]).as_slice()));
    assert!(!is_fallback_checkout(words(&["-C", "/w", "read-tree", "-m", "-u", "HEAD"]).as_slice()));
    assert!(!is_fallback_checkout(words(&["-C", "/w", "clean", "--force", "--hard"]).as_slice()));
}

/// Any value in a configuration file of the repository stops the addition,
/// whatever the value;
/// none lets it through.
#[test]
fn a_value_in_the_repository_configuration_is_the_persons() {
    let root = fixture("workers-config-read");
    let repo = repository(root.as_path(), "repo");
    let context = git_context();
    assert!(parallel_checkout_wanted(&context, repo.as_path(), words(&["worktree", "add"]).as_slice()), "positive control: nothing set");
    git(repo.as_path(), &["config", "checkout.workers", "1"]);
    assert!(!parallel_checkout_wanted(&context, repo.as_path(), words(&["worktree", "add"]).as_slice()));
    remove(root.as_path());
}

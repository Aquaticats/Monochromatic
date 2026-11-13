//! What:
//!  Which forms ask what the worktree holds in their way,
//!  and what the answer means.
//! Why:
//!   A form that asks nothing must run as it always did,
//!  a form that writes a tree must ask,
//!  and a reading Git could not answer must be a refusal rather than an empty list:
//!  a command that overwrote a file nobody saw is the loss this question exists to prevent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // entriesAFormWouldWriteOver({ facts, command: 'reset-hard', region: ['--hard'] }) === ['b.txt']
//! ```

use super::{WrittenOverRequest, entries_a_form_would_write_over};
use crate::command_test_support::os_arguments;
use crate::policy_test_support::scripted_facts;
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::OsString;

/// The entries the question answers for one form over one region.
fn ask(command: GuardedCommand, region: &[&str], in_the_way: Result<Vec<String>, String>) -> (Result<Vec<String>, String>, Vec<String>) {
    let tokens: Vec<OsString> = os_arguments(region);
    let mut facts = scripted_facts();
    facts.in_the_way = in_the_way;
    let answer: Result<Vec<String>, String> = entries_a_form_would_write_over(WrittenOverRequest {
        facts: &mut facts,
        command,
        region: tokens.as_slice(),
        shell_directory: None,
    });
    return (answer, facts.asked);
}

/// A clean deletes what its own dry run names and keeps its recorded answer,
/// so it asks this question about nothing at all.
#[test]
fn every_clean_asks_nothing() {
    for command in [GuardedCommand::Clean, GuardedCommand::CleanIgnored, GuardedCommand::CleanIgnoredOnly] {
        let (answer, asked) = ask(command, &["-f", "-d"], Ok(vec![String::from("b.txt")]));
        assert_eq!(answer, Ok(Vec::<String>::new()), "{command:?}");
        assert!(asked.is_empty(), "{command:?} asked {asked:?}");
    }
}

/// A form whose own paths are the index's (a `rm`, a `stash`, a mixed reset)
/// reaches no file Git did not put there, so it asks nothing;
/// a `mv` and a `checkout-index` name destinations the index does not hold and ask.
#[test]
fn forms_that_write_no_tree_ask_nothing() {
    for command in [
        GuardedCommand::Remove,
        GuardedCommand::Stash,
        GuardedCommand::ResetMixed,
    ] {
        let (answer, asked) = ask(command, &["--force", "a.txt"], Ok(vec![String::from("b.txt")]));
        assert_eq!(answer, Ok(Vec::<String>::new()), "{command:?}");
        assert!(asked.is_empty(), "{command:?} asked {asked:?}");
    }
}

/// A forced tree write, a path form, a forced `mv` and a forced `checkout-index` each ask about the paths
/// they write,
/// and name every entry the worktree holds there.
#[test]
fn a_written_tree_or_path_asks_and_names_what_stands_there() {
    for (command, region, kind) in [
        (GuardedCommand::ResetHard, vec!["--hard", "other"], "tree"),
        (GuardedCommand::CheckoutPaths, vec!["--force", "other"], "tree"),
        (GuardedCommand::SwitchForce, vec!["--force", "other"], "tree"),
        (GuardedCommand::CheckoutPaths, vec!["other", "--", "b.txt"], "matching-tree"),
        (GuardedCommand::RestoreWorktree, vec!["--source", "other", "--", "b.txt"], "matching-tree"),
        (GuardedCommand::RestoreWorktree, vec!["--", "b.txt"], "index"),
    ] {
        let found: Vec<String> = vec![String::from("b.txt"), String::from("nested/")];
        let (answer, asked) = ask(command, region.as_slice(), Ok(found.clone()));
        assert_eq!(answer, Ok(found), "{command:?} {region:?}");
        assert_eq!(asked, vec![format!("paths-in-the-way:{kind}")], "{command:?} {region:?}");
    }
}

/// The merge and keep modes write over only the entries Git's ignore rules cover,
/// so the question they ask is about those alone.
#[test]
fn the_merge_and_keep_modes_ask_about_the_ignored_entries() {
    for mode in ["--merge", "--keep"] {
        let (answer, asked) = ask(GuardedCommand::ResetHard, &[mode, "other"], Ok(Vec::<String>::new()));
        assert_eq!(answer, Ok(Vec::<String>::new()), "{mode}");
        assert_eq!(asked, vec![String::from("paths-in-the-way:tree-of-ignored-entries")], "{mode}");
    }
}

/// A reading that failed is the failure,
/// never an empty answer a caller would read as "nothing is in the way".
#[test]
fn a_reading_that_failed_is_the_failure() {
    let (answer, _asked) = ask(
        GuardedCommand::ResetHard,
        &["--hard"],
        Err(String::from("Git could not read the paths the index holds: fatal")),
    );
    assert_eq!(answer, Err(String::from("Git could not read the paths the index holds: fatal")));
}

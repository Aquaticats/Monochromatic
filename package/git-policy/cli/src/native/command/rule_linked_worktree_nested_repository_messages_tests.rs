//! What:
//!  The two sentences an interactive clean leaves when its menu would offer a nested Git repository,
//!  and what they say about the ways to run the clean.
//! Why:
//!  A person told that a repository stays where it is has to be told how to delete it,
//!  and a sentence that promised a copy the wrapper no longer keeps would send them looking for
//!  one:
//!  the owner decided on 2026-11-15 that a forced clean deletes a nested repository as Git does and
//!  that the wrapper logs the command and what went.
//!  One repository is said in the singular and several in the plural.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // keptOutOfAnInteractiveMenuNotice([Buffer.from('nested')]).includes('stays where it is')
//! ```

use super::{bare_repository_in_a_menu_message, kept_out_of_an_interactive_menu_notice};
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;

/// What:
///  The notice names the repository that stays,
///  says the menu no longer offers it,
///  and gives both ways to delete it deliberately,
///  neither of which is a copy the wrapper keeps.
/// Why:
///  The person reads why the clean runs with one force and what to type to delete the repository;
///  the audit line is named as the record of the deletion,
///  because no notice and no copy exists for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptOutOfAnInteractiveMenuNotice(['nested']).includes('the audit line names it');
/// ```
#[test]
fn the_notice_names_the_repository_that_stays_and_the_two_ways_to_delete_it() {
    let notice: String = kept_out_of_an_interactive_menu_notice(&[b"cache/nested".to_vec()]);
    for expected in [
        "this interactive clean would offer the Git repository cache/nested",
        "the clean runs with one force instead",
        "the Git repository stays where it is",
        "the menu offers nothing of it",
        "Run the same clean without `-i` and Git",
        "deletes it with the rest while the wrapper's audit line names it",
        "--no-enforce-worktree",
    ] {
        assert!(notice.contains(expected), "{expected} missing from {notice}");
    }
    assert!(!notice.contains("saved copy"), "the notice promises a copy the wrapper keeps: {notice}");
}

/// What:
///  Several repositories are counted, and the sentence reads in the plural.
/// Why:
///  A sentence that reads as if one repository were involved when several are is a lie about the loss,
///  and a list of thousands would not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptOutOfAnInteractiveMenuNotice(['a', 'b', 'c']).includes('they stay where they are');
/// ```
#[test]
fn several_repositories_are_named_in_the_plural_and_counted() {
    let many: Vec<Vec<u8>> = (0..SHARED_PATHS_NAMED + 2).map(|number| { return format!("repo{number}").into_bytes(); }).collect();
    let notice: String = kept_out_of_an_interactive_menu_notice(many.as_slice());
    assert!(notice.contains("the Git repositories repo0, repo1"), "{notice}");
    assert!(notice.contains("and 2 more"), "{notice}");
    assert!(notice.contains("the Git repositories stay where they are"), "{notice}");
    assert!(notice.contains("and they are deleted with the rest"), "{notice}");
    assert!(!notice.contains("repo7"), "{notice}");
}

/// What:
///  A bare repository is one force does not keep,
///  so that clean stops and the sentence says so,
///  with the two ways to run it.
/// Why:
///  Git's one force keeps a repository with a `.git` entry and deletes a bare one
///  (`git clean -fXd` at one force: `Skipping repository inner` against `Removing bare/`,
///  measured on this host with native Git 2.56.0, 2026-11-13),
///  so the cap this transform uses for the other shape protects nothing here,
///  and the person is told before the menu runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bareRepositoryInAMenuMessage(['bare/']).includes('one force does not keep a bare repository');
/// ```
#[test]
fn the_bare_repository_sentence_names_the_repository_and_the_two_ways_to_run_it() {
    let message: String = bare_repository_in_a_menu_message(&[b"bare/".to_vec()]);
    for expected in [
        "this interactive clean's menu would offer the bare Git repository bare/",
        "one force does not keep a bare repository",
        "Nothing was run",
        "Run the same clean without `-i` and Git deletes it with the rest while the audit line names it",
        "--no-enforce-worktree",
    ] {
        assert!(message.contains(expected), "{expected} missing from {message}");
    }
    assert!(!message.contains("saved copy"), "the sentence promises a copy the wrapper keeps: {message}");
}

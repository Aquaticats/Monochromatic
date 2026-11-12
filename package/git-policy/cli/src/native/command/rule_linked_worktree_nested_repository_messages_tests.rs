//! What:
//!  The one sentence a refused clean leaves:
//!  which nested repositories it would delete,
//!  what losing them costs,
//!  and every way forward,
//!  the opt-out included.
//! Why:
//!  The repository holds its own history and uncommitted files and no copy of it exists anywhere else,
//!  so the words are the only thing between the person and the loss of it;
//!  one repository is said in the singular and several in the plural.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(nestedRepositoriesWouldBeDeletedMessage([Buffer.from('nested')])).toContain('nothing was deleted');
//! ```

use super::nested_repositories_would_be_deleted_message;
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;

/// What:
///  One repository is named in the singular,
///  with what its loss costs,
///  the clean that keeps it,
///  and the deliberate deletion with the opt-out.
/// Why:
///   The person reads one repository,
///  one history and one opt-out sentence;
///  a plural sentence over one repository would misstate the loss.
#[test]
fn one_repository_is_named_in_the_singular() {
    let message: String = nested_repositories_would_be_deleted_message(&[b"nested".to_vec()]);
    for expected in [
        "this clean would delete the Git repository nested and everything in it",
        "with its own history and its uncommitted files",
        "no copy of that work exists anywhere else",
        "nothing was deleted",
        "git clean -f -d -X",
        "--no-enforce-worktree",
        "it is deleted with the rest",
    ] {
        assert!(message.contains(expected), "{expected} missing from {message}");
    }
    assert!(!message.contains("and 1 more"), "{message}");
}

/// What:
///  Several repositories are named in the plural,
///  and only the first few are listed.
/// Why:
///  A worktree can hold more repositories than a person reads in one sentence,
///  and the count says how many the rest are without listing them all.
#[test]
fn several_repositories_are_named_in_the_plural_and_counted() {
    let many: Vec<Vec<u8>> = (0..SHARED_PATHS_NAMED + 2).map(|number| { return format!("repo{number}").into_bytes(); }).collect();
    let message: String = nested_repositories_would_be_deleted_message(many.as_slice());
    assert!(message.contains("the Git repositories repo0, repo1"), "{message}");
    assert!(message.contains("and 2 more"), "{message}");
    assert!(message.contains("everything in them"), "{message}");
    assert!(message.contains("with their own histories"), "{message}");
    assert!(message.contains("they are deleted with the rest"), "{message}");
    assert!(!message.contains("repo7"), "{message}");
    assert!(!message.contains("it is deleted"), "{message}");
}

/// What:
///  A path whose bytes are not text is still named,
///  with the bytes the terminal cannot show replaced.
/// Why:
///  A refusal that named nothing is one the person cannot act on,
///  and a name the terminal can show is enough to find the directory.
#[test]
fn a_path_that_is_not_text_is_still_named() {
    let message: String = nested_repositories_would_be_deleted_message(&[b"nested/\xff\xfe".to_vec()]);
    assert!(message.contains("nested/"), "{message}");
    assert!(message.contains("nothing was deleted"), "{message}");
}

//! What:
//!  That a place with no home refuses a command that needs one,
//!  and that a place whose repository Git cannot answer refuses with Git's own
//!  reason.
//! Why:
//!  The two refusals are what a person meets before any record is read,
//!  and a sentence that swallowed the reason would send them to the code to learn
//!  what Git said.
//!  Neither control starts Git where it can be avoided:
//!  the second names an executable that is not there,
//!  which is the one answer the reader reports without a process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(route.theHome('cancel-worktree-removal').unwrapErr().code).toBe(2);
//! ```

use crate::worktree_clone_cancel_and_settle_commands::the_answer_one_word_prints::AnAnswer;
use crate::worktree_clone_cancel_and_settle_commands::the_place_the_two_commands_run_in::TheRoute;
use std::path::PathBuf;

/// What:
///  A place whose home this process cannot name.
/// Why:
///  The cancellation settles a staging folder in that home,
///  so half of its answers need one and half of them need none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = { checkout: '/repo', commonDir: '/repo/.git', home: undefined };
/// ```
fn a_place_with_no_home() -> TheRoute {
    return TheRoute {
        checkout: PathBuf::from("/repo"),
        common_dir: PathBuf::from("/repo/.git"),
        home: None,
    };
}

/// What:
///  That the home is given where it is nameable,
///  and that a place without one refuses by naming the word and the folder it
///  could not find.
/// Why:
///  A person reads which command wanted the home and why,
///  and a command that guessed a home would settle another account's cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(a_place_with_no_home().theHome('cancel-worktree-removal').unwrapErr().stderr).toContain('account home');
/// ```
#[test]
fn a_place_with_no_home_refuses_the_command_that_needs_one() {
    let refused: AnAnswer = a_place_with_no_home()
        .the_home("cancel-worktree-removal")
        .expect_err("a place with no home refuses");
    assert_eq!(refused.code, 2);
    assert!(
        refused.stderr.contains("cancel-worktree-removal"),
        "the refusal names the word a person typed: {}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains("account home"),
        "the refusal names what it could not find: {}",
        refused.stderr
    );
    let named: TheRoute = TheRoute {
        checkout: PathBuf::from("/repo"),
        common_dir: PathBuf::from("/repo/.git"),
        home: Some(PathBuf::from("/home/someone")),
    };
    assert_eq!(
        named
            .the_home("cancel-worktree-removal")
            .expect("the home is given"),
        std::path::Path::new("/home/someone")
    );
}

/// What:
///  That a repository Git cannot answer refuses with the caller's status and
///  Git's own reason in the sentence.
/// Why:
///  The wrapper starts no discovery of its own,
///  so the reason is the only thing that tells a person whether they typed the
///  command somewhere Git can answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = TheRoute.of('cancel-worktree-removal', [], [], '/nonexistent/git', undefined).unwrapErr();
/// expect(answer.stderr).toContain('/nonexistent/git');
/// ```
#[test]
fn a_repository_git_cannot_answer_refuses_with_gits_reason() {
    let refused: AnAnswer = TheRoute::of(
        "cancel-worktree-removal",
        &[],
        &[],
        std::path::Path::new("/nonexistent/git-that-is-not-there"),
        None,
    )
    .expect_err("an executable that is not there answers no repository");
    assert_eq!(refused.code, 2);
    assert!(
        refused.stdout.is_empty(),
        "a refusal writes nothing on standard output"
    );
    assert!(
        refused.stderr.contains("cancel-worktree-removal"),
        "the refusal names the word: {}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains("git-that-is-not-there"),
        "the refusal carries the reason the read could not answer: {}",
        refused.stderr
    );
}

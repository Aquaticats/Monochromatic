//! What:
//!  The rejection of a clean that would delete a nested Git repository or worktree:
//!  the sentence that names each one and every way forward,
//!  the opt-out that runs the clean as real Git runs it included.
//! Why:
//!  The repository holds its own history and uncommitted files,
//!  which exist nowhere else,
//!  and the outer worktree reports it as one ignored directory nobody looks inside.
//!  The owner's rule (2026-11-12) is that such a command is refused and names what it would destroy;
//!  a person who means it passes `--no-enforce-worktree` and the clean runs as Git runs it,
//!  and the wrapper keeps no copy.
//!  One repository is said in the singular and several in the plural,
//!  because a sentence that reads as if one repository were at stake when two are is a lie about the loss.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // nestedRepositoriesWouldBeDeletedMessage([Buffer.from('nested')])
//! ```

/// The paths a message names before it counts the rest.
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;

/// The sentence every refusal ends with:
/// the one deliberate way to delete a nested repository with the rest.
const DELIBERATE_DELETION_ADVICE: &str = "If you have decided to delete that repository on purpose, pass --no-enforce-worktree directly after `clean` (for example `git clean --no-enforce-worktree -f -f -d -X`) and it is deleted with the rest.";

/// The same advice where several repositories are at stake,
///  which cannot be said with the singular's pronouns.
const DELIBERATE_DELETION_ADVICE_MANY: &str = "If you have decided to delete those repositories on purpose, pass --no-enforce-worktree directly after `clean` (for example `git clean --no-enforce-worktree -f -f -d -X`) and they are deleted with the rest.";

/// What:
///  The rejection of a clean whose second force would delete nested Git repositories.
///  `&[Vec<u8>]` borrows the paths Git named,
///  relative to the worktree top level.
/// Why:
///   The refusal names every repository it would destroy,
///  which is the work nobody can get back,
///  and every way forward that loses nothing of it:
///  the same clean with one force,
///  which is Git's own protection of a nested repository,
///  and moving the repository out of the ignored directory first.
///  The last sentence is the deliberate deletion with the opt-out.
///  A path with bytes that are not UTF-8 is written lossily,
///  because the terminal is text and the repository is still named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedRepositoriesWouldBeDeletedMessage(repositories: Buffer[]): string;
/// ```
pub fn nested_repositories_would_be_deleted_message(repositories: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::<String>::new();
    for path in repositories.iter().take(SHARED_PATHS_NAMED) {
        named.push(String::from_utf8_lossy(path).into_owned());
    }
    let mut listed: String = named.join(", ");
    if repositories.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more", repositories.len() - SHARED_PATHS_NAMED).as_str());
    }
    if repositories.len() == 1 {
        return format!(
            "monochromatic-git-policy-cli: this clean would delete the Git repository {listed} and everything in it, with its \
             own history and its uncommitted files, and no copy of that work exists anywhere else, so the command is rejected \
             and nothing was deleted. The same clean with one force keeps every nested repository and deletes everything beside \
             it (`git clean -f -d -X`), and moving the repository out of the ignored directory first lets this command run as \
             typed. {DELIBERATE_DELETION_ADVICE}"
        );
    }
    return format!(
        "monochromatic-git-policy-cli: this clean would delete the Git repositories {listed} and everything in them, with their \
         own histories and their uncommitted files, and no copy of that work exists anywhere else, so the command is rejected \
         and nothing was deleted. The same clean with one force keeps every nested repository and deletes everything beside it \
         (`git clean -f -d -X`), and moving the repositories out of the ignored directories first lets this command run as \
         typed. {DELIBERATE_DELETION_ADVICE_MANY}"
    );
}

/// The refusal,
///  and the count it makes of several repositories.
#[cfg(test)]
#[path = "rule_linked_worktree_nested_repository_messages_tests.rs"]
mod tests;

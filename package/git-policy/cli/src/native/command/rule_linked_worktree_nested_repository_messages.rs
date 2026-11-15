//! What:
//!  The sentences an interactive clean leaves when its menu would offer a nested Git repository.
//! Why:
//!  A menu is answered after Git starts,
//!  so the wrapper cannot ask the person before Git deletes;
//!  the clean runs with one force,
//!  which keeps a repository with a `.git` entry where it is,
//!  and the person is told which repository that is and how to run the clean the other way.
//!  A forced clean that is not a menu keeps nothing and deletes the repository as Git does:
//!  its audit line names the repositories it takes
//!  (`policy_linked_worktree_nested_repositories.rs`, and the owner's answer of 2026-11-15,
//!  "Answers on waking, third round").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // keptOutOfAnInteractiveMenuNotice([Buffer.from('nested')])
//! ```

/// The paths a message names before it counts the rest.
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;

/// What:
///  The sentence for an interactive clean that runs with one force so its menu offers no nested repository.
/// Why:
///  The menu is answered after Git starts,
///  so the wrapper cannot ask the person before Git deletes;
///  the repository stays where it is,
///  the menu offers exactly what the clean deletes,
///  and the person is told which repository that is and the two ways to delete it deliberately:
///  the same clean without `-i`,
/// where Git deletes it with the rest and the audit line names it,
/// and the escape hatch,
/// which deletes it with the rest and keeps no line.
///  Nothing is written to the audit for this,
///  because the line already names the command the person typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptOutOfAnInteractiveMenuNotice(repositories: Buffer[]): string;
/// ```
pub fn kept_out_of_an_interactive_menu_notice(repositories: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::<String>::new();
    for path in repositories.iter().take(SHARED_PATHS_NAMED) {
        named.push(String::from_utf8_lossy(path).into_owned());
    }
    let mut listed: String = named.join(", ");
    if repositories.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more", repositories.len() - SHARED_PATHS_NAMED).as_str());
    }
    let one: bool = repositories.len() == 1;
    // One repository reads in the singular and several in the plural, in every clause a pronoun reaches.
    let what: &str = if one { "the Git repository" } else { "the Git repositories" };
    let subject: &str = if one { "it" } else { "they" };
    let object: &str = if one { "it" } else { "them" };
    let stays: &str = if one { "stays" } else { "stay" };
    let is: &str = if one { "is" } else { "are" };
    return format!(
        "monochromatic-git-policy-cli: this interactive clean would offer {what} {listed} and everything inside it, and a menu \
         is answered after Git starts. So the clean runs with one force instead: {what} {stays} where {subject} {is}, and the \
         menu offers nothing of {object}. The rest of the clean runs as typed. Run the same clean without `-i` and Git \
         deletes {object} with the rest while the wrapper's audit line names {object}, or pass --no-enforce-worktree directly \
         after `clean` and {subject} {is} deleted with the rest and no line is written.\n"
    );
}

/// What:
///  The sentence an interactive clean stops with when its menu would offer a bare Git repository,
///  which one force does not keep.
/// Why:
///  Git's one force keeps a repository with a `.git` entry and deletes a bare one
///  (`git clean -fXd` at one force: `Skipping repository inner` against `Removing bare/` and `bare/` gone,
///  measured on this host with native Git 2.56.0, 2026-11-13),
///  so the cap this wrapper uses for the other shape protects nothing here.
///  The person decides: the clean without a menu deletes the repository as Git does and the audit
///  line names it,
///  and the opt-out runs the menu and the deletion they choose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bareRepositoryInAMenuMessage(['bare/']);
/// ```
pub fn bare_repository_in_a_menu_message(repositories: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::<String>::new();
    for path in repositories.iter().take(SHARED_PATHS_NAMED) {
        named.push(String::from_utf8_lossy(path).into_owned());
    }
    let mut listed: String = named.join(", ");
    if repositories.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more", repositories.len() - SHARED_PATHS_NAMED).as_str());
    }
    return format!(
        "monochromatic-git-policy-cli: this interactive clean's menu would offer the bare Git repository {listed} and everything \
         inside it, and one force does not keep a bare repository: Git deletes it while the menu is answered after Git \
         started, so nothing could name it before it went. Nothing was run. Run the same clean without `-i` and Git deletes it \
         with the rest while the audit line names it, or pass --no-enforce-worktree directly after `clean` and the menu and \
         whatever you choose run as real Git runs them.\n"
    );
}

/// The notices of an interactive clean,
/// and the paths they name.
#[cfg(test)]
#[path = "rule_linked_worktree_nested_repository_messages_tests.rs"]
mod tests;

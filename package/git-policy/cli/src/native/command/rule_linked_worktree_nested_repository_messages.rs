//! What:
//!  The notice that names where a clean's nested Git repository went,
//!  and the rejection when it could not be moved there.
//! Why:
//!  A person who ran `git clean -f -f -d -X` and then looks for the repository that was in
//!  that ignored directory has only these words to find it;
//!  they name the repository,
//!  the place it is now,
//!  and how to look at it,
//!  and the rejection names where it still is and every way forward.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // savedNestedRepositoryNotice([{ path: 'nested', savedTo: '/r/.git/.../discarded/2026-11-11T03-00-00/nested' }])
//! ```

use crate::conflict_sides::{SavedCopy, SaveFailure};
/// The paths a message names before it counts the rest.
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
use std::path::Path;

/// The sentence the rejection ends with:
/// the one deliberate way to delete a nested repository with the rest.
const HATCH_ADVICE: &str = "If you have decided to delete that repository on purpose, pass --no-enforce-worktree directly after `clean` (for example `git clean --no-enforce-worktree -f -f -d -X`) and it is deleted with the rest.";

/// What:
///  Everything the rejection of a clean names.
///  A `struct` is a record with named fields.
/// Why:
///   The failure alone names whichever path it reached,
///  which is the place the copies go when the saved folder itself could not be made;
///  the repositories are what the person loses when a move fails,
///  so the record carries them beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NotSaved = { repositories: Buffer[]; failure: SaveFailure; savedRoot: string; stillMoved: SavedCopy[] };
/// ```
pub struct NotSaved<'a> {
    /// The repositories the clean would delete,
    /// as Git named them.
    pub repositories: &'a [Vec<u8>],
    /// What stopped the save.
    pub failure: &'a SaveFailure,
    /// The directory the copies live under.
    pub saved_root: &'a Path,
    /// The repositories the stopped save moved and could not put back,
    ///  each with the place it is.
    pub still_moved: &'a [SavedCopy],
}

/// What:
///  The plain-language notice for the repositories a clean's save moved,
///  one block per repository,
///  written before the clean runs.
/// Why:
///   The place is a directory under the common Git directory,
///  which nobody finds by looking at the worktree;
///  the notice says where it is and how to see what the repository holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedNestedRepositoryNotice(copies: SavedCopy[]): string;
/// ```
pub fn saved_nested_repository_notice(copies: &[SavedCopy]) -> String {
    let mut text: String = String::new();
    for copy in copies {
        let Some(place) = copy.saved_to.as_deref() else {
            continue;
        };
        text.push_str(
            format!(
                "monochromatic-git-policy-cli: this clean would delete the Git repository {} and everything in it, so it was moved \
                 whole to\n  {}\n  before the clean runs; the rest of the clean runs as typed. Look at what it holds with `git -C {} \
                 status`, and delete it yourself when you no longer need it.\n",
                copy.path,
                place.display(),
                place.display()
            )
            .as_str(),
        );
    }
    return text;
}

/// What:
///  The rejection of a clean whose nested repositories could not be moved to the saved copies.
///  `NotSaved<'_>` borrows the repositories,
///  the failure and the place the copies go.
/// Why:
///   A save that stops puts every repository it moved back,
///  so "nothing was changed" is true;
///  the one exception is a repository a put-back could not return,
///  and that one is named with the place it is.
///  The person is told which repositories are still where they were,
///  what stopped the move in the system's own words,
///  every way forward,
///  and the deliberate way to delete them anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedRepositoryNotSavedMessage(notSaved: NotSaved): string;
/// ```
pub fn nested_repository_not_saved_message(not_saved: NotSaved<'_>) -> String {
    let mut named: Vec<String> = Vec::<String>::new();
    for path in not_saved.repositories.iter().take(SHARED_PATHS_NAMED) {
        named.push(String::from_utf8_lossy(path).into_owned());
    }
    let mut listed: String = named.join(", ");
    if not_saved.repositories.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more", not_saved.repositories.len() - SHARED_PATHS_NAMED).as_str());
    }
    let one: bool = not_saved.repositories.len() == 1;
    let (what, them, where_they_are): (&str, &str, &str) = if one {
        ("the Git repository", "it", "The repository is still where it was.")
    } else {
        ("the Git repositories", "them", "The repositories are still where they were.")
    };
    // A put-back that could not return a repository leaves it below the saved copies, and the person is told exactly
    //  where, because nothing else names that place.
    let mut kept: String = String::new();
    for copy in not_saved.still_moved {
        let Some(place) = copy.saved_to.as_deref() else {
            continue;
        };
        kept.push_str(format!("\n  {} => {}", copy.path, place.display()).as_str());
    }
    if !kept.is_empty() {
        return format!(
            "monochromatic-git-policy-cli: this clean would delete {what} {listed} and everything in {them}, but {them} could not \
             all be saved first ({}: {}), so the command is rejected and the clean did not run. These repositories could not be \
             put back where they were and are kept here:{kept}\nEvery other repository is still where it was. Free the space under \
             {}, or fix its permissions, or move {them} out of the way yourself, and run the command again. {HATCH_ADVICE}",
            not_saved.failure.subject,
            not_saved.failure.reason,
            not_saved.saved_root.display()
        );
    }
    return format!(
        "monochromatic-git-policy-cli: this clean would delete {what} {listed} and everything in {them}, but nothing of {them} \
         could be saved first ({}: {}), so the command is rejected and nothing was changed. {where_they_are} Free the space under \
         {}, or fix its permissions, or move {them} out of the way yourself, and run the command again. {HATCH_ADVICE}",
        not_saved.failure.subject,
        not_saved.failure.reason,
        not_saved.saved_root.display()
    );
}


/// What:
///  The sentence for an interactive clean that runs with one force so its menu offers no nested repository.
/// Why:
///  The menu is answered after Git starts,
///  so the wrapper cannot ask the person before Git deletes;
///  the repository stays where it is,
///  the menu offers exactly what the clean deletes,
///  and the person is told which repository that is and the two ways to delete it deliberately:
///  the same clean without `-i`, which saves it whole first, and the escape hatch, which deletes it with the rest.
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
         menu offers nothing of {object}. The rest of the clean runs as typed. Run the same clean without `-i` and the wrapper \
         moves {object} whole to a saved copy before Git deletes {object}, or pass --no-enforce-worktree directly after `clean` \
         and {object} {is} deleted with the rest.\n"
    );
}

/// The notices and the rejection, and the paths they name.
#[cfg(test)]
#[path = "rule_linked_worktree_nested_repository_messages_tests.rs"]
mod tests;
